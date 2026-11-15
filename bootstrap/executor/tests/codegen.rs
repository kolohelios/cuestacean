use std::path::Path;
use std::process::Command;

use bootstrap_executor::generate;

#[path = "../../generated/codegen-escaped-rules.rs"]
mod generated;

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn escaped_names_are_data_not_rust_source() {
    let boolean = r#"{"formatVersion":1,"entry":{"kind":"true"}}"#;
    assert_eq!(
        bootstrap_executor::generated::execute_json(boolean).expect("default boolean"),
        serde_json::Value::Bool(true)
    );
    assert_eq!(
        generated::execute_json(boolean).expect("changed compiled boolean rule"),
        serde_json::Value::Bool(false)
    );
    let program = read("tests/fixtures/codegen-escaped-program.json");
    let rules = read("tests/fixtures/codegen-escaped-rules.json");
    assert_eq!(
        bootstrap_executor::execute(&program, &rules).expect("interpreted"),
        7
    );
    // This module is generated from names containing quotes, backslashes,
    // a newline, and Unicode. It must compile and retain the original names.
    assert_eq!(generated::execute(&program).expect("generated"), 7);
    assert_eq!(
        generated::execute_json(&program).expect("generated scalar"),
        serde_json::Value::from(7)
    );
}

#[test]
fn generation_is_deterministic_and_cli_reports_errors() {
    let rules_path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("evaluation.json");
    let rules = read(&rules_path);
    let source = generate(&rules).expect("generate source");
    assert_eq!(source, generate(&rules).expect("repeat generation"));
    let reordered = read("tests/fixtures/codegen-reordered-rules.json");
    assert_eq!(source, generate(&reordered).expect("reordered rules"));
    for result in [
        r#"{"op":"boolean"}"#,
        r#"{"op":"boolean","value":"true"}"#,
        r#"{"op":"bool","value":true}"#,
        r#"{"op":"boolean","value":true,"extra":0}"#,
    ] {
        let malformed = format!(r#"{{"formatVersion":1,"rules":{{"x":{{"result":{result}}}}}}}"#);
        serde_json::from_str::<serde_json::Value>(&malformed).expect("well-formed JSON");
        assert!(generate(&malformed).is_err());
    }
    let binary = env!("CARGO_BIN_EXE_bootstrap-codegen");
    let output = Command::new(binary)
        .arg(&rules_path)
        .output()
        .expect("run generator");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, source.as_bytes());
    assert!(output.stderr.is_empty());
    for args in [
        vec![],
        vec!["not-a-file"],
        vec!["one", "two"],
        vec!["tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("run failing generator");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
