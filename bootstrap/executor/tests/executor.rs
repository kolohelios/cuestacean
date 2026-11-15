use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bootstrap_executor::{execute, generate};

#[path = "../../generated/evaluation.rs"]
mod generated;
#[path = "../../generated/evaluation-left-twice.rs"]
mod generated_left_twice;
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
        assert_eq!(
            generated::execute(&program).expect("generated evaluator"),
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
    assert_eq!(
        generated_left_twice::execute(&program).expect("changed generated evaluator"),
        40
    );
}

// Reuse the real JSON case corpus through both file-based CLIs without adding
// a second copy of those fixtures. create_new prevents overwriting any file.
struct ProgramFile(PathBuf);

impl ProgramFile {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "cuestacean-cli-{}-{stamp}.json",
            std::process::id()
        ));
        std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create program file");
        Self(path)
    }

    fn write(&self, program: &Value) {
        std::fs::write(
            &self.0,
            serde_json::to_vec(program).expect("serialize program"),
        )
        .expect("write program file");
    }
}

impl Drop for ProgramFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn cli_parity(program: &Path) -> Output {
    let interpreted = Command::new(env!("CARGO_BIN_EXE_bootstrap-executor"))
        .arg(program)
        .arg(bootstrap().join("evaluation.json"))
        .output()
        .expect("run interpreter CLI");
    let generated = Command::new(env!("CARGO_BIN_EXE_bootstrap-generated"))
        .arg(program)
        .output()
        .expect("run generated CLI");
    assert_eq!(
        generated.status.code(),
        interpreted.status.code(),
        "{program:?}"
    );
    assert_eq!(generated.stdout, interpreted.stdout, "{program:?}");
    assert_eq!(generated.stderr, interpreted.stderr, "{program:?}");
    generated
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
    let file = ProgramFile::new();
    for case in cases {
        file.write(&case.program);
        let cli = cli_parity(&file.0);
        let program = serde_json::to_string(&case.program).expect("serialize program");
        let result = execute(&program, &rules);
        assert_eq!(
            result.as_ref().map_err(ToString::to_string),
            generated::execute(&program)
                .as_ref()
                .map_err(ToString::to_string),
            "{}: interpreted versus generated",
            case.name
        );
        match (case.expected, case.error) {
            (Some(expected), None) => {
                assert_eq!(result.expect(&case.name), expected, "{}", case.name);
                assert!(cli.status.success(), "{}", case.name);
                assert_eq!(
                    cli.stdout,
                    format!("{expected}\n").as_bytes(),
                    "{}",
                    case.name
                );
                assert!(cli.stderr.is_empty(), "{}", case.name);
            }
            (None, Some(error)) => {
                assert!(
                    result.expect_err(&case.name).to_string().contains(&error),
                    "{}: expected {error}",
                    case.name
                );
                assert_eq!(cli.status.code(), Some(2), "{}", case.name);
                assert!(cli.stdout.is_empty(), "{}", case.name);
                assert!(
                    String::from_utf8_lossy(&cli.stderr).contains(&error),
                    "{}",
                    case.name
                );
            }
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
            generate(&rules)
                .expect_err(&case.name)
                .to_string()
                .contains(&case.error)
        );
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
    assert!(
        generate(&invalid)
            .expect_err("invalid generation input")
            .to_string()
            .contains("invalid rules JSON")
    );
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
fn generated_cli_uses_baked_rules_and_reports_errors() {
    for name in ["literal", "addition"] {
        let output = cli_parity(&bootstrap().join(format!("{name}.json")));
        assert!(output.status.success());
        assert_eq!(output.stdout, b"42\n");
        assert!(output.stderr.is_empty());
    }
    for path in [
        Path::new("tests/fixtures/invalid.json"),
        Path::new("not-a-file"),
    ] {
        let output = cli_parity(path);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for args in [vec![], vec!["../addition.json", "../evaluation.json"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-generated"))
            .args(args)
            .output()
            .expect("run invalid invocation");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("usage: bootstrap-generated"));
    }
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
