use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::execute;
use serde::Deserialize;
use serde_json::Value;

fn bootstrap() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn exported_programs_and_rule_change() {
    let rules = read(bootstrap().join("evaluation.json"));
    for name in ["literal", "addition"] {
        let program = read(bootstrap().join(format!("{name}.json")));
        assert_eq!(
            execute(&program, &rules).expect("execute exported tree"),
            42
        );
    }
    let program = read(bootstrap().join("addition.json"));
    let changed = read(bootstrap().join("tests/valid/evaluation-left-twice.json"));
    // Same program and Rust binary, different CUE-exported rule: 20 + 20.
    assert_eq!(
        execute(&program, &changed).expect("execute changed rule"),
        40
    );
}

#[derive(Deserialize)]
struct ProgramCase {
    name: String,
    program: Value,
    expected: Option<i64>,
    error: Option<String>,
}

#[test]
fn integer_boundaries_and_program_errors() {
    let cases: Vec<ProgramCase> =
        serde_json::from_str(&read("tests/fixtures/program-cases.json")).expect("parse cases");
    let rules = read(bootstrap().join("evaluation.json"));
    for case in cases {
        let program = serde_json::to_string(&case.program).expect("serialize program");
        let result = execute(&program, &rules);
        match (case.expected, case.error) {
            (Some(expected), None) => {
                assert_eq!(result.expect(&case.name), expected, "{}", case.name)
            }
            (None, Some(error)) => assert!(
                result.expect_err(&case.name).to_string().contains(&error),
                "{}: expected {error}",
                case.name
            ),
            _ => panic!("{}: case must specify exactly one outcome", case.name),
        }
    }
}

#[derive(Deserialize)]
struct RuleCase {
    name: String,
    rules: Value,
    error: String,
}

#[test]
fn malformed_rules_and_json() {
    let cases: Vec<RuleCase> =
        serde_json::from_str(&read("tests/fixtures/rule-cases.json")).expect("parse cases");
    let program = read(bootstrap().join("literal.json"));
    for case in cases {
        let rules = serde_json::to_string(&case.rules).expect("serialize rules");
        assert!(
            execute(&program, &rules)
                .expect_err(&case.name)
                .to_string()
                .contains(&case.error),
            "{}",
            case.name
        );
    }
    let invalid = read("tests/fixtures/invalid.json");
    let rules = read(bootstrap().join("evaluation.json"));
    assert!(
        execute(&invalid, &rules)
            .expect_err("invalid program")
            .to_string()
            .contains("invalid program JSON")
    );
    assert!(
        execute(&program, &invalid)
            .expect_err("invalid rules")
            .to_string()
            .contains("invalid rules JSON")
    );
}

#[test]
fn cli_outputs_results_and_reports_errors() {
    let binary = env!("CARGO_BIN_EXE_bootstrap-executor");
    for (rules, expected) in [
        ("evaluation.json", "42\n"),
        ("tests/valid/evaluation-left-twice.json", "40\n"),
    ] {
        let output = Command::new(binary)
            .arg(bootstrap().join("addition.json"))
            .arg(bootstrap().join(rules))
            .output()
            .expect("run executor");
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    for args in [
        vec![],
        vec!["not-a-file", "not-a-file"],
        vec!["one", "two", "three"],
        vec!["tests/fixtures/invalid.json", "tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("run failing executor");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
