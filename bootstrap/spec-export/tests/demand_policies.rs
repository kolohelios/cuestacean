use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn spikes() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spikes")
}
fn schema() -> String {
    fs::read_to_string(spikes().join("demand-policy-schema.cue")).expect("raw schema")
}
fn program(name: &str) -> String {
    fs::read_to_string(spikes().join(format!("{name}.cue"))).expect("raw program")
}

// Complete handwritten trees, not outputs captured from either evaluator.
fn fixtures() -> Vec<(&'static str, Value)> {
    vec![
        (
            "demand-policy",
            json!({"formatVersion":1,"body":{
                "op":"if","field":"empty",
                "then":{"op":"return","action":"constraints"},
                "else":{"op":"if","field":"forced",
                    "then":{"op":"return","action":"constraints"},
                    "else":{"op":"if","field":"recipe",
                        "then":{"op":"if","field":"nested",
                            "then":{"op":"return","action":"defer"},
                            "else":{"op":"return","action":"flat"}},
                        "else":{"op":"return","action":"constraints"}}}
            }}),
        ),
        (
            "demand-policy-defer",
            json!({"formatVersion":1,
            "body":{"op":"return","action":"defer"}}),
        ),
    ]
}
fn parsed(text: &str) -> Value {
    serde_json::from_str(text).expect("export JSON")
}
fn cue(schema: &str, source: &str) -> std::process::Output {
    let dir = RawFiles::new(schema, source);
    Command::new("cue")
        .arg("export")
        .arg(dir.0.join("left.cue"))
        .arg(dir.0.join("right.cue"))
        .args(["--out", "json"])
        .output()
        .expect("mandatory pinned raw CUE oracle")
}
struct RawFiles(PathBuf);
impl RawFiles {
    fn new(schema: &str, source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "cuestacean-demand-export-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("exclusive raw input directory");
        fs::write(dir.join("left.cue"), schema).expect("raw schema copy");
        fs::write(dir.join("right.cue"), source).expect("raw program copy");
        Self(dir)
    }
    fn cli(&self, reverse: bool) -> std::process::Output {
        let names = if reverse {
            ["right.cue", "left.cue"]
        } else {
            ["left.cue", "right.cue"]
        };
        Command::new(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
            .env_clear()
            .env("PATH", "/nonexistent")
            .current_dir(&self.0)
            .args(names)
            .output()
            .expect("direct native CLI without CUE")
    }
}
impl Drop for RawFiles {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned raw files");
    }
}
fn compare(schema: &str, source: &str, expected: &Value) -> String {
    let native = export(schema, source).expect("native library export");
    assert_eq!(parsed(&native), *expected);
    assert_eq!(export(source, schema).expect("reversed sources"), native);
    assert_eq!(export(schema, source).expect("repeat library"), native);
    let upstream = cue(schema, source);
    assert!(
        upstream.status.success(),
        "{}",
        String::from_utf8_lossy(&upstream.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&upstream.stdout).expect("CUE JSON"),
        *expected
    );
    let files = RawFiles::new(schema, source);
    for reverse in [false, true, false] {
        let output = files.cli(reverse);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, native.as_bytes(), "CLI/library repeat bytes");
    }
    native
}

#[test]
fn both_policies_match_full_handwritten_trees_library_cli_and_raw_cue() {
    let schema = schema();
    for (name, expected) in fixtures() {
        compare(&schema, &program(name), &expected);
    }
}

#[test]
fn exact_two_file_boundaries_and_source_order_are_not_special_cased() {
    let schema = schema();
    for (name, expected) in fixtures() {
        let source = program(name);
        // No canonical schema, program, definition name or filename is recognized.
        let renamed_schema = schema.replace("#Policy", "#Envelope");
        let renamed_source = source.replace("#Policy", "#Envelope");
        let original = compare(&renamed_schema, &renamed_source, &expected);
        let mut lines: Vec<_> = renamed_schema.lines().collect();
        // Reorder complete definition blocks, preserving the recursive definition.
        let decision = lines
            .iter()
            .position(|line| line.starts_with("#Decision:"))
            .expect("decision definition");
        lines.swap(3, 4); // #Action and #Fact declarations
        assert!(decision > 4);
        assert_eq!(
            export(&lines.join("\n"), &renamed_source).expect("declaration order"),
            original
        );
        let reordered = renamed_source.replace("op: \"if\", field:", "field:");
        // Reorder a complete pair rather than treating JSON key order as source order.
        let reordered = reordered
            .replace("\"empty\"\n", "\"empty\", op: \"if\"\n")
            .replace("\"forced\"\n", "\"forced\", op: \"if\"\n")
            .replace("\"recipe\"\n", "\"recipe\", op: \"if\"\n")
            .replace("\"nested\"\n", "\"nested\", op: \"if\"\n");
        assert_eq!(
            export(&renamed_schema, &reordered).expect("field order"),
            original
        );
        assert!(
            export("package demandpolicy\n", &source).is_err(),
            "program requires schema"
        );
        assert_eq!(
            export(&schema, "package demandpolicy\n"),
            Err(Error::Syntax("missing root embedding"))
        );
    }
    // The two alternative root programs are not one combined input closure.
    let combined = format!(
        "{}\n{}",
        program("demand-policy"),
        program("demand-policy-defer").replacen("package demandpolicy", "", 1)
    );
    assert_eq!(export(&schema, &combined), Err(Error::Conflict));
}

#[test]
fn schema_valid_mutations_export_generic_instruction_data() {
    let schema = schema();
    let source = program("demand-policy-defer");
    for action in ["constraints", "flat", "defer"] {
        compare(
            &schema,
            &source.replace("\"defer\"", &format!("\"{action}\"")),
            &json!({"formatVersion":1,"body":{"op":"return","action":action}}),
        );
    }
    for field in ["empty", "forced", "recipe", "nested"] {
        let source = format!(
            "package demandpolicy\n#Policy & {{body: {{op: \"if\", field: \"{field}\", then: {{op: \"return\", action: \"flat\"}}, else: {{op: \"return\", action: \"defer\"}}}}}}\n"
        );
        compare(
            &schema,
            &source,
            &json!({"formatVersion":1,"body":{
            "op":"if","field":field,"then":{"op":"return","action":"flat"},
            "else":{"op":"return","action":"defer"}}}),
        );
    }
    // Arbitrary schema vocabulary must also work; the exporter is not a VM validator.
    let renamed_schema = schema.replace("\"defer\"", "\"pause\"");
    compare(
        &renamed_schema,
        &source.replace("\"defer\"", "\"pause\""),
        &json!({"formatVersion":1,"body":{"op":"return","action":"pause"}}),
    );
}

#[test]
fn invalid_incomplete_and_conflicting_inputs_publish_no_json() {
    let schema = schema();
    for (source, expected) in [
        (
            "#Policy & {body: {op: \"return\", action: \"unknown\"}}",
            Error::Conflict,
        ),
        (
            "#Policy & {body: {op: \"if\", field: \"unknown\", then: {op: \"return\", action: \"defer\"}, else: {op: \"return\", action: \"defer\"}}}",
            Error::Conflict,
        ),
        ("#Policy & {body: {op: \"return\"}}", Error::Incomplete),
        (
            "#Policy & {formatVersion: 2, body: {op: \"return\", action: \"defer\"}}",
            Error::Conflict,
        ),
        (
            "#Policy & {body: {op: \"return\", action: \"defer\", extra: 1}}",
            Error::Conflict,
        ),
    ] {
        let source = format!("package demandpolicy\n{source}\n");
        assert_eq!(export(&schema, &source), Err(expected));
        let output = RawFiles::new(&schema, &source).cli(false);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty(), "no partial semantic output");
        assert!(!output.stderr.is_empty());
        let upstream = cue(&schema, &source);
        assert!(!upstream.status.success());
        assert!(upstream.stdout.is_empty());
    }
    for source in [
        "package demandpolicy\n#Policy & {",
        "package demandpolicy\nimport \"x\"\n",
    ] {
        assert!(export(&schema, source).is_err());
        let output = RawFiles::new(&schema, source).cli(false);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn lowered_shared_work_failure_recovers_without_cached_answers() {
    let schema = schema();
    for (name, expected) in fixtures() {
        let source = program(name);
        for _ in 0..2 {
            assert_eq!(
                export_with_limit(&schema, &source, 1),
                Err(Error::Limit("evaluation steps"))
            );
            let full = export(&schema, &source).expect("fresh full allowance");
            assert_eq!(parsed(&full), expected);
            let required = (1..=4096)
                .find(|limit| export_with_limit(&schema, &source, *limit).is_ok())
                .expect("unchanged hard allowance suffices");
            assert_eq!(
                export_with_limit(&schema, &source, required).expect("exact allowance"),
                full
            );
            assert_eq!(
                export_with_limit(&schema, &source, required - 1),
                Err(Error::Limit("evaluation steps"))
            );
            assert_eq!(export(&schema, &source).expect("recovery"), full);
        }
    }
}
