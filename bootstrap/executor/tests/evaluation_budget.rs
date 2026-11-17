use std::{fs, path::Path, process::Command};

use bootstrap_executor::{Error, execute, execute_json, generated};
use serde_json::{Value, json};

#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/evaluation-reverse.rs"]
mod reverse;

fn export_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json"))
}

fn export(name: &str) -> String {
    fs::read_to_string(export_path(name)).expect("read exported rules")
}

fn zero() -> Value {
    json!({"kind":"integer", "value":0})
}

fn negate(mut entry: Value, wrappers: usize) -> Value {
    for _ in 0..wrappers {
        entry = json!({"kind":"negate", "operand":entry});
    }
    entry
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind, "left":left, "right":right})
}

fn left_spine(levels: usize) -> Value {
    let mut entry = zero();
    for _ in 0..levels {
        entry = binary("add", entry, zero());
    }
    entry
}

fn program(entry: Value) -> String {
    json!({"formatVersion":1, "entry":entry}).to_string()
}

fn normal(program: &str) -> [Result<Value, Error>; 6] {
    let rules = export("evaluation");
    [
        execute(program, &rules).map(Value::from),
        execute_json(program, &rules),
        generated::execute(program).map(Value::from),
        generated::execute_json(program),
        reverse::execute(program).map(Value::from),
        reverse::execute_json(program),
    ]
}

fn repeated(program: &str) -> [Result<Value, Error>; 4] {
    let rules = export("tests/valid/evaluation-left-twice");
    [
        execute(program, &rules).map(Value::from),
        execute_json(program, &rules),
        left_twice::execute(program).map(Value::from),
        left_twice::execute_json(program),
    ]
}

fn succeeds(results: impl IntoIterator<Item = Result<Value, Error>>) {
    for result in results {
        assert_eq!(result.expect("independently expected zero"), json!(0));
    }
}

fn fails(results: impl IntoIterator<Item = Result<Value, Error>>, expected: &Error, text: &str) {
    for result in results {
        let error = result.expect_err("independently expected error");
        assert_eq!(
            std::mem::discriminant(&error),
            std::mem::discriminant(expected)
        );
        assert_eq!(error.to_string(), text);
    }
}

#[test]
fn root_counts_toward_64_active_nodes_and_calls_have_fresh_contexts() {
    // Raw JSON bypasses the distinct source parser depth/node restrictions.
    let at_limit = program(negate(zero(), 63));
    let over_limit = program(negate(zero(), 64));
    // Two separate children reach active depth 64; finishing left frees depth for right.
    let siblings = program(binary("subtract", negate(zero(), 62), negate(zero(), 62)));
    succeeds(normal(&siblings));
    succeeds(repeated(&siblings));
    for _ in 0..2 {
        succeeds(normal(&at_limit));
        fails(
            normal(&over_limit),
            &Error::EvaluationDepth,
            "evaluation exceeds 64 active AST nodes",
        );
        succeeds(normal(&at_limit));
        succeeds(repeated(&at_limit));
        fails(
            repeated(&over_limit),
            &Error::EvaluationDepth,
            "evaluation exceeds 64 active AST nodes",
        );
        succeeds(repeated(&at_limit));
    }
}

#[test]
fn repeated_child_evaluation_shares_exactly_4096_visits() {
    // T(0)=1; T(n)=1+2*T(n-1). The actual tree has only 23 nodes.
    let visits_4095 = left_spine(11);
    let visits_4096 = program(negate(visits_4095.clone(), 1));
    let visits_4097 = program(negate(visits_4095.clone(), 2));
    for _ in 0..2 {
        succeeds(repeated(&program(visits_4095.clone())));
        succeeds(repeated(&visits_4096));
        for entry in [&visits_4097, &program(left_spine(12))] {
            fails(
                repeated(entry),
                &Error::EvaluationVisits,
                "evaluation exceeds 4096 AST visits",
            );
        }
        succeeds(repeated(&visits_4096));
    }
    // Default rules do not repeat left, and must not charge instructions as ASTs.
    succeeds(normal(&visits_4097));
    // Root + left consume 4096 visits: RHS exists, but entry fails before kind lookup.
    fails(
        repeated(&program(binary("subtract", left_spine(11), json!({})))),
        &Error::EvaluationVisits,
        "evaluation exceeds 4096 AST visits",
    );
    // No RHS entry exists here: field lookup still precedes attempting a visit.
    fails(
        repeated(&program(json!({"kind":"subtract","left":left_spine(11)}))),
        &Error::MissingField {
            field: "right".into(),
        },
        "missing field \"right\"",
    );
}

#[test]
fn ordinary_errors_and_left_first_child_priority_are_unchanged() {
    let too_deep = negate(zero(), 64);
    for (entry, expected, message) in [
        (json!({}), Error::Kind, "expression must have a string kind"),
        (
            json!({"kind":"unknownTestKind"}),
            Error::MissingRule {
                kind: "unknownTestKind".into(),
            },
            "no rule for node kind \"unknownTestKind\"",
        ),
        (
            json!({"kind":"negate"}),
            Error::MissingField {
                field: "operand".into(),
            },
            "missing field \"operand\"",
        ),
        (
            binary("subtract", json!({}), too_deep.clone()),
            Error::Kind,
            "expression must have a string kind",
        ),
        (
            binary(
                "subtract",
                negate(json!({"kind":"intType"}), 1),
                too_deep.clone(),
            ),
            Error::Arithmetic,
            "arithmetic requires compatible concrete operands",
        ),
        (
            binary("subtract", json!({"kind":"intType"}), too_deep.clone()),
            Error::EvaluationDepth,
            "evaluation exceeds 64 active AST nodes",
        ),
        (
            binary("subtract", too_deep, json!({})),
            Error::EvaluationDepth,
            "evaluation exceeds 64 active AST nodes",
        ),
        (
            negate(json!({}), 63),
            Error::Kind,
            "expression must have a string kind",
        ),
        (
            negate(json!({}), 64),
            Error::EvaluationDepth,
            "evaluation exceeds 64 active AST nodes",
        ),
    ] {
        let text = program(entry);
        // subtract is unchanged by the alternate sum rule.
        fails(normal(&text), &expected, message);
        fails(repeated(&text), &expected, message);
        succeeds(normal(&program(zero())));
        succeeds(repeated(&program(zero())));
    }
}

#[test]
fn file_clis_report_depth_failure_without_partial_stdout() {
    struct Temp(std::path::PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!(
        "cuestacean-evaluation-budget-{}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("create owned temporary directory");
    let temp = Temp(path);
    let input = temp.0.join("program.json");
    fs::write(&input, program(negate(zero(), 64))).expect("write raw AST");
    let mut reference = Command::new(env!("CARGO_BIN_EXE_bootstrap-executor"));
    reference.arg(&input).arg(export_path("evaluation"));
    let mut compiled = Command::new(env!("CARGO_BIN_EXE_bootstrap-generated"));
    compiled.arg(&input);
    for command in [&mut reference, &mut compiled] {
        let output = command.output().expect("run file CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"evaluation exceeds 64 active AST nodes\n");
    }
}
