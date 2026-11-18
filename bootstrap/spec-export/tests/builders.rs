use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{path::PathBuf, process::Command};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn argument(name: &str) -> Value {
    json!({"op": "argument", "name": name})
}

fn string(value: &str) -> Value {
    json!({"op": "string", "value": value})
}

fn integer(value: i64) -> Value {
    json!({"op": "integer", "value": value})
}

fn object(fields: Vec<(Value, Value)>) -> Value {
    json!({"op": "object", "fields": fields.into_iter()
        .map(|(key, value)| json!({"key": key, "value": value}))
        .collect::<Vec<_>>()})
}

fn builder(parameters: &[(&str, &str)], result: Value) -> Value {
    json!({"formatVersion": 1, "parameters": parameters.iter()
        .map(|(name, kind)| json!({"name": name, "type": kind}))
        .collect::<Vec<_>>(), "result": result})
}

fn program(version: i64) -> Value {
    builder(
        &[("entry", "json")],
        object(vec![
            (string("formatVersion"), integer(version)),
            (string("entry"), argument("entry")),
        ]),
    )
}

// Exact builderFixtures membership plus the forced-lowering input. These are
// handwritten instruction trees, not evaluated ASTs or answers loaded from CUE.
fn fixtures() -> Vec<(&'static str, Value)> {
    vec![
        ("ast-program", program(1)),
        (
            "ast-atom",
            builder(
                &[("kind", "string")],
                object(vec![(string("kind"), argument("kind"))]),
            ),
        ),
        (
            "ast-integer",
            builder(
                &[
                    ("kind", "string"),
                    ("valueField", "string"),
                    ("value", "integer"),
                ],
                object(vec![
                    (string("kind"), argument("kind")),
                    (argument("valueField"), argument("value")),
                ]),
            ),
        ),
        (
            "ast-text",
            builder(
                &[
                    ("kind", "string"),
                    ("valueField", "string"),
                    ("value", "string"),
                ],
                object(vec![
                    (string("kind"), argument("kind")),
                    (argument("valueField"), argument("value")),
                ]),
            ),
        ),
        (
            "ast-binary",
            builder(
                &[
                    ("kind", "string"),
                    ("leftField", "string"),
                    ("rightField", "string"),
                    ("left", "json"),
                    ("right", "json"),
                ],
                object(vec![
                    (string("kind"), argument("kind")),
                    (argument("leftField"), argument("left")),
                    (argument("rightField"), argument("right")),
                ]),
            ),
        ),
        (
            "ast-ternary",
            builder(
                &[
                    ("kind", "string"),
                    ("operandField", "string"),
                    ("lowerField", "string"),
                    ("upperField", "string"),
                    ("operand", "json"),
                    ("lower", "json"),
                    ("upper", "json"),
                ],
                object(vec![
                    (string("kind"), argument("kind")),
                    (argument("operandField"), argument("operand")),
                    (argument("lowerField"), argument("lower")),
                    (argument("upperField"), argument("upper")),
                ]),
            ),
        ),
        (
            "ast-unary",
            builder(
                &[
                    ("kind", "string"),
                    ("operandField", "string"),
                    ("operand", "json"),
                ],
                object(vec![
                    (string("kind"), argument("kind")),
                    (argument("operandField"), argument("operand")),
                ]),
            ),
        ),
        (
            "ast-record-field",
            builder(
                &[("name", "string"), ("value", "json")],
                object(vec![
                    (string("name"), argument("name")),
                    (string("value"), argument("value")),
                ]),
            ),
        ),
        // Version 2 is an inner construction instruction, not the DSL version.
        ("tests/valid/ast-program-v2", program(2)),
        (
            "tests/valid/ast-escaped",
            builder(
                &[("param\"\\\nλ", "string")],
                object(vec![
                    (string("payload\"\\\nλ"), argument("param\"\\\nλ")),
                    (string("second"), argument("param\"\\\nλ")),
                ]),
            ),
        ),
        // Two argument uses with distinct keys, not duplicate source fields.
        (
            "tests/valid/ast-duplicate",
            builder(
                &[("value", "json")],
                object(vec![
                    (string("first"), argument("value")),
                    (string("second"), argument("value")),
                ]),
            ),
        ),
        // The later forced compiler policy emits 99; export must retain data 7.
        ("tests/valid/ast-literal", builder(&[], integer(7))),
    ]
}

#[test]
fn complete_builder_family_matches_handwritten_trees_and_pinned_cue() {
    let root = root();
    let schema_path = root.join("ast-schema.cue");
    let schema = std::fs::read_to_string(&schema_path).expect("raw schema");
    let cases = fixtures();
    assert_eq!(cases.len(), 12);
    for (name, expected) in cases {
        let path = root.join(format!("{name}.cue"));
        let source = std::fs::read_to_string(&path).expect("raw builder");
        let native = export(&schema, &source).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(
            serde_json::from_str::<Value>(&native).expect("native JSON"),
            expected,
            "{name}: complete instruction tree, including ordered arrays"
        );
        assert_eq!(export(&schema, &source).expect("repeat"), native, "{name}");
        assert_eq!(
            export(&source, &schema).expect("reversed files"),
            native,
            "{name}: source order must not affect bytes"
        );
        let cue = Command::new("cue")
            .arg("export")
            .arg(&schema_path)
            .arg(&path)
            .args(["--out", "json"])
            .output()
            .expect("mandatory pinned CUE from devshell");
        assert!(
            cue.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&cue.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&cue.stdout).expect("CUE JSON"),
            expected,
            "{name}: independent upstream comparison"
        );
    }
}

#[test]
fn complete_builder_family_cli_needs_only_two_raw_files() {
    let root = root();
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
    assert!(executable.is_absolute());
    let schema_path = root.join("ast-schema.cue");
    let schema = std::fs::read_to_string(&schema_path).expect("raw schema");
    for (name, expected) in fixtures() {
        let path = root.join(format!("{name}.cue"));
        let source = std::fs::read_to_string(&path).expect("raw builder");
        let native = export(&schema, &source).expect("library export");
        let mut previous = None;
        for _ in 0..2 {
            let output = Command::new(&executable)
                .env_clear()
                .env("PATH", "/nonexistent")
                .arg(&schema_path)
                .arg(&path)
                .output()
                .expect("direct native CLI");
            assert!(
                output.status.success(),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "{name}");
            assert_eq!(
                output.stdout,
                native.as_bytes(),
                "{name}: CLI/library bytes"
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).expect("CLI JSON"),
                expected,
                "{name}"
            );
            if let Some(bytes) = previous {
                assert_eq!(output.stdout, bytes, "{name}: separate process repeat");
            }
            previous = Some(output.stdout);
        }
    }
}

#[test]
fn lowered_family_work_allowance_does_not_poison_full_budget_exports() {
    let schema = std::fs::read_to_string(root().join("ast-schema.cue")).expect("raw schema");
    for (name, expected) in fixtures() {
        let source =
            std::fs::read_to_string(root().join(format!("{name}.cue"))).expect("raw builder");
        for _ in 0..2 {
            assert_eq!(
                export_with_limit(&schema, &source, 1),
                Err(Error::Limit("evaluation steps")),
                "{name}: lowered work allowance"
            );
            let full = export(&schema, &source).expect("fresh full allowance after failure");
            assert_eq!(
                serde_json::from_str::<Value>(&full).expect("full export JSON"),
                expected,
                "{name}: each invocation has a fresh budget"
            );
        }
    }
}
