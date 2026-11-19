use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

// These are raw owners, not selected/extracted instructions or exported answers.
const OWNER: &str = include_str!("../../ast-integer-policy.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const FRAGMENTS: &str = include_str!("../../ast-lowering-fragments.cue");
const BODY: &str = include_str!("../../ast-lowering-body.cue");
const SELECTOR: &str = include_str!("../../ast-lowering.cue");
const FORCED: &str = include_str!("../../tests/valid/ast-lowering-forced.cue");
const TEXT: &str = "package bootstrap\n{cases: integer: {op: \"text\", value: \"x\"}}\n";
const TAG_ONLY: &str = "package bootstrap\n{cases: integer: {op: \"text\"}}\n";

// Independent whole-fragment expectations, never captured from either provider.
fn concat() -> Value {
    json!({"op":"concat","parts":[
        {"op":"text","value":"Value::from("},
        {"op":"signed","field":"value"},
        {"op":"text","value":"i64)"}
    ]})
}
fn fragment(integer: Value) -> Value {
    json!({"cases":{"integer":integer}})
}
fn parse(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("complete exact-integer JSON")
}
fn failed(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no partial JSON");
    assert!(!output.stderr.is_empty(), "mandatory failure diagnostic");
}
fn executable(path: &Path) {
    assert!(path.is_absolute(), "absolute provider required");
    fs::File::open(path).expect("readable mandatory provider");
    assert_ne!(
        fs::metadata(path)
            .expect("provider metadata")
            .permissions()
            .mode()
            & 0o111,
        0,
        "executable mandatory provider"
    );
}
struct Probe {
    directory: PathBuf,
    cue: PathBuf,
    native: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "cuestacean-integer-policy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(directory.is_absolute());
        fs::create_dir(&directory).expect("fresh owned scratch directory");
        fs::create_dir(directory.join("home")).expect("owned HOME");
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned toolchain PATH"))
            .map(|path| path.join("cue"))
            .find(|path| path.is_file())
            .expect("mandatory pinned CUE CLI")
            .canonicalize()
            .expect("absolute oracle");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&cue);
        executable(&native);
        Self {
            directory,
            cue,
            native,
        }
    }
    fn run(&self, operation: Option<&str>, inputs: &[(&str, &str)], reverse: bool) -> Output {
        let mut names = Vec::new();
        for (name, source) in inputs {
            fs::write(self.directory.join(name), source).expect("raw input file");
            names.push(*name);
        }
        if reverse {
            names.reverse();
        }
        let mut command = Command::new(if operation.is_some() {
            &self.cue
        } else {
            &self.native
        });
        command
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", self.directory.join("home"))
            .current_dir(&self.directory);
        if let Some(operation) = operation {
            command.arg(operation);
        }
        command
            .args(names)
            .output()
            .expect("fresh mandatory process")
    }
    fn full(&self, provider: Option<&str>, context: Option<&str>, reverse: bool) -> Output {
        let mut inputs = vec![
            ("schema-renamed.cue", SCHEMA),
            ("remaining-renamed.cue", FRAGMENTS),
            ("body-renamed.cue", BODY),
            ("selector-renamed.cue", SELECTOR),
        ];
        if let Some(provider) = provider {
            // A JSON root is a materialized provider, not a retained preference.
            inputs.push((
                if provider.starts_with('{') {
                    "provider.json"
                } else {
                    "owner-renamed.cue"
                },
                provider,
            ));
        }
        if let Some(context) = context {
            inputs.push(("context-renamed.cue", context));
        }
        self.run(Some("export"), &inputs, reverse)
    }
    fn full_incomplete(&self, provider: Option<&str>, context: Option<&str>) {
        failed(&self.full(provider, context, false));
        let mut inputs = vec![
            ("schema-renamed.cue", SCHEMA),
            ("remaining-renamed.cue", FRAGMENTS),
            ("body-renamed.cue", BODY),
            ("selector-renamed.cue", SELECTOR),
        ];
        if let Some(provider) = provider {
            inputs.push(("owner-renamed.cue", provider));
        }
        if let Some(context) = context {
            inputs.push(("context-renamed.cue", context));
        }
        let evaluation = self.run(Some("eval"), &inputs, true);
        assert!(
            evaluation.status.success(),
            "{}",
            String::from_utf8_lossy(&evaluation.stderr)
        );
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).expect("retire owned scratch directory");
    }
}

#[test]
fn canonical_raw_two_file_owner_has_default_text_and_incomplete_paths() {
    let probe = Probe::new();
    for (context, expected) in [
        ("", Ok(fragment(concat()))),
        (
            r#"{cases: {integer: {op: "text", value: "x"}}}"#,
            Ok(fragment(json!({"op":"text","value":"x"}))),
        ),
        (
            r#"{cases: {integer: {op: "text"}}}"#,
            Err(Error::Incomplete),
        ),
    ] {
        // Append only independently authored ordinary context to the package
        // companion. The canonical owner stays intact in every observation.
        let companion = format!("{PACKAGE}\n{context}\n");
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (OWNER, companion.as_str())
            } else {
                (companion.as_str(), OWNER)
            };
            assert_eq!(
                export(a, b).map(|text| serde_json::from_str::<Value>(&text).expect("JSON")),
                expected
            );
            let inputs = [
                ("z-ordinary.cue", OWNER),
                ("a-ordinary.cue", companion.as_str()),
            ];
            for operation in [Some("export"), None] {
                for _ in 0..2 {
                    let output = probe.run(operation, &inputs, reverse);
                    match &expected {
                        Ok(value) => assert_eq!(parse(&output), *value),
                        Err(_) => failed(&output),
                    }
                }
            }
            if expected.is_err() {
                assert!(probe.run(Some("eval"), &inputs, reverse).status.success());
            }
        }
    }
}

#[test]
fn native_default_json_composes_into_the_same_complete_go_policy_as_raw_owner() {
    let probe = Probe::new();
    let native = probe.run(None, &[("package.cue", PACKAGE), ("raw.cue", OWNER)], false);
    assert_eq!(parse(&native), fragment(concat()));
    let provider = String::from_utf8(native.stdout).expect("native JSON UTF-8");
    for reverse in [false, true] {
        let raw = parse(&probe.full(Some(OWNER), None, reverse));
        let composed = parse(&probe.full(Some(&provider), None, reverse));
        assert_eq!(raw, composed, "complete schema-validated policy equality");
        assert_eq!(composed["cases"]["integer"], concat());
        assert_eq!(composed["formatVersion"], json!(1));
    }
}

#[test]
fn raw_full_policy_context_keeps_text_x_and_existing_constant99_control() {
    let probe = Probe::new();
    for (context, expected) in [
        (TEXT, json!({"op":"text","value":"x"})),
        (FORCED, json!({"op":"text","value":"Value::from(99i64)"})),
    ] {
        for reverse in [false, true] {
            let policy = parse(&probe.full(Some(OWNER), Some(context), reverse));
            assert_eq!(policy["cases"]["integer"], expected);
            assert_eq!(policy["formatVersion"], json!(1));
            assert!(
                policy["cases"]["object"].is_object(),
                "not a fragment-only observation"
            );
        }
    }
}

#[test]
fn missing_provider_and_raw_full_policy_text_tag_are_incomplete_not_setup_errors() {
    let probe = Probe::new();
    probe.full_incomplete(None, None);
    probe.full_incomplete(Some(OWNER), Some(TAG_ONLY));
}

#[test]
fn materialized_default_has_no_late_text_override_or_raw_fallback() {
    let probe = Probe::new();
    let native = probe.run(None, &[("package.cue", PACKAGE), ("raw.cue", OWNER)], false);
    assert_eq!(parse(&native), fragment(concat()));
    let provider = String::from_utf8(native.stdout).expect("native JSON UTF-8");
    for context in [TEXT, FORCED] {
        for reverse in [false, true] {
            failed(&probe.full(Some(&provider), Some(context), reverse));
            let inputs = [
                ("schema.cue", SCHEMA),
                ("remaining.cue", FRAGMENTS),
                ("body.cue", BODY),
                ("selector.cue", SELECTOR),
                ("default.json", provider.as_str()),
                ("override.cue", context),
            ];
            let evaluation = probe.run(Some("eval"), &inputs, reverse);
            assert!(
                !evaluation.status.success(),
                "Conflict, not merely Incomplete"
            );
            assert!(!evaluation.stderr.is_empty());
        }
    }
}
