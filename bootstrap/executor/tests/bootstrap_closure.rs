//! Upstream characterization for the first proposed native export closure.
//! This test runs Go CUE deliberately; it is not a no-Go-CUE implementation.
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use serde_json::{Value, json};

const ATOM: &str = include_str!("../../ast-atom.cue");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn export(program: &str) -> Output {
    let mut child = Command::new("cue")
        .arg("export")
        .arg(root().join("ast-schema.cue"))
        .args(["-", "--out", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mandatory pinned Go CUE oracle");
    child
        .stdin
        .take()
        .expect("program stdin")
        .write_all(program.as_bytes())
        .expect("write bounded characterization program");
    child.wait_with_output().expect("oracle exit")
}

fn expected() -> Value {
    json!({
        "formatVersion": 1,
        "parameters": [{"name": "kind", "type": "string"}],
        "result": {"op": "object", "fields": [{
            "key": {"op": "string", "value": "kind"},
            "value": {"op": "argument", "name": "kind"}
        }]}
    })
}

fn concrete(program: &str) -> (Value, Vec<u8>) {
    let output = export(program);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = serde_json::from_slice(&output.stdout).expect("exact JSON value");
    (value, output.stdout)
}

#[test]
fn two_file_atom_closure_has_independent_output_and_stable_bytes() {
    let (value, first) = concrete(ATOM);
    assert_eq!(value, expected());
    assert_eq!(concrete(ATOM).1, first);
    // formatVersion comes from the singleton constraint, not a default marker.
    assert!(!ATOM.contains("formatVersion"));
    let renamed = ATOM.replace("value: \"kind\"", "value: \"tag\"");
    let mut changed = expected();
    changed["result"]["fields"][0]["key"]["value"] = json!("tag");
    assert_eq!(concrete(&renamed).0, changed);
}

#[test]
fn closedness_types_and_incompleteness_are_not_missing_schema_errors() {
    for (program, diagnostic) in [
        (
            ATOM.replacen("name: \"kind\"", "name: \"\"", 1),
            "invalid value",
        ),
        (
            ATOM.replace("type: \"string\"", "type: \"bool\""),
            "disjunction",
        ),
        (
            ATOM.replace("#BuildProgram & {", "#BuildProgram & {extra: 1,"),
            "field not allowed",
        ),
        (
            ATOM.replace("op: \"argument\"", "op: \"unknown\""),
            "disjunction",
        ),
        (
            ATOM.replacen("name: \"kind\"", "name: string", 1),
            "incomplete",
        ),
    ] {
        let output = export(&program);
        assert!(!output.status.success());
        assert!(
            output.stdout.is_empty(),
            "failed export cannot publish JSON"
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(diagnostic), "{error}");
        assert!(
            !error.contains("not found") && !error.contains("undefined"),
            "schema setup failed: {error}"
        );
    }
    for fixture in ["ast-key-type", "ast-operation", "ast-parameter-type"] {
        let source = std::fs::read_to_string(root().join(format!("tests/invalid/{fixture}.cue")))
            .expect("existing negative fixture");
        let output = export(&source);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{fixture}");
        assert!(output.stdout.is_empty());
        assert!(
            !error.contains("not found") && !error.contains("undefined"),
            "missing schema is not rejection: {error}"
        );
    }
}

#[test]
fn dsl_argument_names_are_data_not_cue_references() {
    // Native builder validation checks parameter binding later. Exporting this
    // specification does not execute the represented argument instruction.
    let source = ATOM.replace(
        "op: \"argument\", name: \"kind\"",
        "op: \"argument\", name: \"missing\"",
    );
    assert_ne!(source, ATOM);
    let mut data = expected();
    data["result"]["fields"][0]["value"]["name"] = json!("missing");
    assert_eq!(concrete(&source).0, data);
}
