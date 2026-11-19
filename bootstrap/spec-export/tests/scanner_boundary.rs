use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("bootstrap directory")
        .to_path_buf()
}

// Every oracle invocation is a fresh public Go-CUE process with explicit inputs.
// The default observer's restricted AST profile is not this schema's oracle.
struct Sources {
    directory: PathBuf,
    schema: PathBuf,
    program: PathBuf,
}

impl Sources {
    fn new(schema: &str, program: &str, renamed: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "cuestacean-scanner-boundary-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("unique scratch sources");
        let (schema_name, program_name) = if renamed {
            ("unrelated-z.cue", "unrelated-a.cue")
        } else {
            ("scanner-schema.cue", "group-open.cue")
        };
        let sources = Self {
            schema: directory.join(schema_name),
            program: directory.join(program_name),
            directory,
        };
        fs::write(&sources.schema, schema).expect("raw scanner schema");
        fs::write(&sources.program, program).expect("raw scanner program");
        sources
    }

    fn cue(&self, paths: &[&Path]) -> Output {
        Command::new("cue")
            .current_dir(&self.directory)
            .arg("export")
            .args(paths)
            .args(["--out", "json"])
            .output()
            .expect("mandatory pinned Go CUE from explicit ROOT devshell")
    }

    fn oracle(&self) -> Output {
        self.cue(&[&self.schema, &self.program])
    }

    fn native(&self, reversed: bool) -> Output {
        let paths = if reversed {
            [&self.program, &self.schema]
        } else {
            [&self.schema, &self.program]
        };
        Command::new(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
            .current_dir(&self.directory)
            .env_clear()
            .env("PATH", "/nonexistent")
            .args(paths)
            .output()
            .expect("native CLI without upstream process access")
    }
}

impl Drop for Sources {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).expect("remove scratch sources");
    }
}

fn schema() -> String {
    fs::read_to_string(root().join("scanner-schema.cue")).expect("full raw scanner schema")
}

fn success(output: &Output, expected: &Value) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("complete upstream JSON"),
        *expected
    );
}

fn missing_reference(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no JSON on missing schema");
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("#ScanProgram"), "{diagnostic}");
    assert!(diagnostic.contains("not found"), "{diagnostic}");
}

fn public_profile_go_cue(body: &str, expected_body: Value) {
    let program = format!("package bootstrap\n#ScanProgram & {{body: [{body}]}}\n");
    let sources = Sources::new(&schema(), &program, false);
    success(
        &sources.oracle(),
        &json!({"formatVersion": 1, "body": expected_body}),
    );
}

fn semantic_rejection(body: &str) {
    let program = format!("package bootstrap\n#ScanProgram & {{body: [{body}]}}\n");
    let sources = Sources::new(&schema(), &program, false);
    let output = sources.oracle();
    assert!(!output.status.success(), "must reject: {body}");
    assert!(output.stdout.is_empty(), "no partial JSON: {body}");
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(!diagnostic.trim().is_empty(), "diagnostic required: {body}");
    assert!(
        !diagnostic.contains("not found") && !diagnostic.contains("undefined"),
        "missing references are not semantic evidence: {body}: {diagnostic}"
    );
}

#[test]
fn complete_group_open_raw_closure_and_file_identity() {
    let schema = schema();
    let program = fs::read_to_string(root().join("group-open.cue")).expect("raw group-open");
    // Independent whole tree, not a projection or an answer loaded from exports.
    let expected = json!({
        "formatVersion": 1,
        "body": [{
            "op": "if",
            "condition": {"op": "byteIn", "bytes": [40]},
            "then": [{"op": "setFlag", "value": true}, {"op": "advance"}],
            "else": []
        }]
    });
    let mut baseline = None;
    for renamed in [false, true] {
        let sources = Sources::new(&schema, &program, renamed);
        for reversed in [false, true] {
            for _ in 0..2 {
                let paths = if reversed {
                    [&sources.program as &Path, &sources.schema as &Path]
                } else {
                    [&sources.schema as &Path, &sources.program as &Path]
                };
                let output = sources.cue(&paths);
                success(&output, &expected);
                if let Some(bytes) = &baseline {
                    assert_eq!(&output.stdout, bytes, "order/repeat/rename bytes");
                } else {
                    baseline = Some(output.stdout);
                }
            }
        }
        // Definitions alone export an empty object, not a scanner program.
        success(&sources.cue(&[&sources.schema]), &json!({}));
        missing_reference(&sources.cue(&[&sources.program]));
    }
}

#[test]
fn whole_schema_remains_unsupported_natively_without_json() {
    let schema = schema();
    let program = fs::read_to_string(root().join("group-open.cue")).expect("raw group-open");
    // Even unused optional/range/regex/bool/default syntax is outside admission.
    // Do not weaken the raw schema or accept Syntax/Conflict/Incomplete here.
    for (first, second) in [(&schema, &program), (&program, &schema)] {
        assert!(matches!(export(first, second), Err(Error::Unsupported(_))));
    }
    for renamed in [false, true] {
        let sources = Sources::new(&schema, &program, renamed);
        for reversed in [false, true] {
            let output = sources.native(reversed);
            assert!(!output.status.success());
            assert!(output.stdout.is_empty(), "native failure publishes no JSON");
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with("unsupported:"),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn ahead_and_ascii_required_prefix_boundaries() {
    public_profile_go_cue(
        r#"{op: "if", condition: {op: "byteIn", bytes: [0,127]}, then: []}"#,
        json!([{"op":"if", "condition":{"op":"byteIn", "bytes":[0,127]}, "then":[], "else":[]}]),
    );
    for ahead in [0, 64] {
        public_profile_go_cue(
            &format!(
                "{{op: \"if\", condition: {{op: \"byteIn\", bytes: [0,127], ahead: {ahead}}}, then: []}}"
            ),
            json!([{"op":"if", "condition":{"op":"byteIn", "bytes":[0,127], "ahead":ahead}, "then":[], "else":[]}]),
        );
    }
    for ahead in ["65", "-1", "true"] {
        semantic_rejection(&format!(
            "{{op: \"if\", condition: {{op: \"byteIn\", bytes: [0], ahead: {ahead}}}, then: []}}"
        ));
    }
    for bytes in ["[128]", "[-1]", "[]", "[true]"] {
        semantic_rejection(&format!(
            "{{op: \"if\", condition: {{op: \"byteIn\", bytes: {bytes}}}, then: []}}"
        ));
    }
}

#[test]
fn bool_flags_and_repeat_ranges() {
    for value in [false, true] {
        public_profile_go_cue(
            &format!(
                "{{op: \"setFlag\", value: {value}}}, {{op: \"if\", condition: {{op: \"flagIs\", value: {value}}}, then: []}}"
            ),
            json!([
                {"op":"setFlag", "value":value},
                {"op":"if", "condition":{"op":"flagIs", "value":value}, "then":[], "else":[]}
            ]),
        );
    }
    for value in ["0", "\"true\"", "[]"] {
        semantic_rejection(&format!("{{op: \"setFlag\", value: {value}}}"));
        semantic_rejection(&format!(
            "{{op: \"if\", condition: {{op: \"flagIs\", value: {value}}}, then: []}}"
        ));
    }
    for count in [0, 64] {
        public_profile_go_cue(
            &format!("{{op: \"repeat\", count: {count}, body: []}}"),
            json!([{"op":"repeat", "count":count, "body":[]}]),
        );
    }
    for count in ["65", "-1", "true"] {
        semantic_rejection(&format!("{{op: \"repeat\", count: {count}, body: []}}"));
    }
}

#[test]
fn unicode_scalar_regex_and_required_character_prefix() {
    public_profile_go_cue(
        r#"{op: "if", condition: {op: "charIn", characters: ["a","é","💚","\n"]}, then: []}"#,
        json!([{
            "op":"if", "condition":{"op":"charIn", "characters":["a","é","💚","\n"]},
            "then":[], "else":[]
        }]),
    );
    // Combining sequences contain two scalars even when displayed as one glyph.
    for characters in [r#"["ab"]"#, r#"[""]"#, r#"["é"]"#, "[]", "[1]"] {
        semantic_rejection(&format!(
            "{{op: \"if\", condition: {{op: \"charIn\", characters: {characters}}}, then: []}}"
        ));
    }
}

#[test]
fn recursive_instances_preserve_omitted_and_explicit_else() {
    public_profile_go_cue(
        r#"
        {op: "if", condition: {op: "hasByte"}, then: [
            {op: "if", condition: {op: "flagIs", value: true}, then: []},
            {op: "repeat", count: 1, body: [
                {op: "if", condition: {op: "charIn", characters: ["é"]},
                    then: [{op: "advanceChar"}], else: [{op: "break"}]}
            ]}
        ], else: [{op: "reject", reason: "outer"}]},
        {op: "while", condition: {op: "hasByte"}, body: [
            {op: "if", condition: {op: "flagIs", value: false}, then: []}
        ]}
        "#,
        json!([
            {"op":"if", "condition":{"op":"hasByte"}, "then":[
                {"op":"if", "condition":{"op":"flagIs", "value":true}, "then":[], "else":[]},
                {"op":"repeat", "count":1, "body":[
                    {"op":"if", "condition":{"op":"charIn", "characters":["é"]},
                     "then":[{"op":"advanceChar"}], "else":[{"op":"break"}]}
                ]}
            ], "else":[{"op":"reject", "reason":"outer"}]},
            {"op":"while", "condition":{"op":"hasByte"}, "body":[
                {"op":"if", "condition":{"op":"flagIs", "value":false}, "then":[], "else":[]}
            ]}
        ]),
    );
    // This characterizes flat empty-list defaults, not general default lifetime.
}

#[test]
fn rejection_reason_and_recursive_closedness() {
    public_profile_go_cue(
        r#"{op: "reject", reason: "stop"}"#,
        json!([{"op":"reject", "reason":"stop"}]),
    );
    for body in [
        r#"{op: "reject", reason: ""}"#,
        r#"{op: "reject"}"#,
        r#"{op: "reject", reason: 1}"#,
        r#"{op: "reject", reason: "stop", unknown: 1}"#,
        r#"{op: "advance", unknown: 1}"#,
        r#"{op: "if", condition: {op: "hasByte", unknown: 1}, then: []}"#,
        r#"{op: "repeat", count: 0, body: [{op: "advance", unknown: 1}]}"#,
        r#"{op: "if", condition: {op: "hasByte"}, then: [], else: [{op: "break", unknown: 1}]}"#,
    ] {
        semantic_rejection(body);
    }
    let sources = Sources::new(
        &schema(),
        "package bootstrap\n#ScanProgram & {body: [], unknown: 1}\n",
        false,
    );
    let output = sources.oracle();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("not allowed"), "{diagnostic}");
}
