use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::{Error, execute, parser, runtime};
use runtime::Value as RuntimeValue;
use serde_json::{Value, json};

use bootstrap_executor::generated;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn source(category: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/{category}/subtract-{name}.cue"))
}

fn integer(n: i64) -> Value {
    json!({"kind": "integer", "value": n})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind": kind, "left": left, "right": right})
}

fn program(entry: Value) -> String {
    json!({"formatVersion": 1, "entry": entry}).to_string()
}

fn both(program: &str) -> [Result<i64, Error>; 2] {
    [
        execute(program, &export("evaluation")),
        generated::execute(program),
    ]
}

#[test]
fn checked_runtime_difference() {
    use RuntimeValue::{Int, Integer};
    for (left, right, expected) in [
        (50, 8, 42),
        (2, 7, -5),
        (-5, -7, 2),
        (i64::MIN, i64::MIN, 0),
        (i64::MAX, 0, i64::MAX),
    ] {
        assert_eq!(
            runtime::difference(Integer(left), Integer(right)).expect("concrete subtraction"),
            Integer(expected)
        );
    }
    for (left, right) in [(i64::MAX, -1), (i64::MIN, 1)] {
        let error =
            runtime::difference(Integer(left), Integer(right)).expect_err("subtraction overflow");
        assert!(matches!(error, Error::DifferenceOverflow));
        assert_eq!(
            error.to_string(),
            "signed 64-bit integer subtraction overflow"
        );
    }
    let bound = RuntimeValue::Bounds {
        lower: Some(0),
        upper: Some(0),
    };
    for (left, right) in [(Int, Integer(1)), (Integer(1), Int), (bound, Integer(1))] {
        assert!(matches!(
            runtime::difference(left, right),
            Err(Error::Arithmetic)
        ));
    }
}

#[test]
fn independent_trees_and_export_exercise_both_evaluators() {
    for result in both(&export("subtraction")) {
        assert_eq!(result.expect("exported subtraction"), 42);
    }
    for (left, right, expected) in [
        (50, 8, 42),
        (2, 7, -5),
        (-5, -7, 2),
        (i64::MIN, i64::MIN, 0),
        (i64::MAX, i64::MAX, 0),
    ] {
        let tree = program(binary("subtract", integer(left), integer(right)));
        for result in both(&tree) {
            assert_eq!(
                result.expect("evaluate subtraction tree"),
                expected,
                "{tree}"
            );
        }
    }
    for (left, right) in [(i64::MAX, -1), (i64::MIN, 1)] {
        let tree = program(binary("subtract", integer(left), integer(right)));
        for result in both(&tree) {
            assert_eq!(
                result.expect_err("subtraction must overflow").to_string(),
                "signed 64-bit integer subtraction overflow"
            );
        }
    }
    for tree in [
        binary("subtract", json!({"kind":"intType"}), integer(1)),
        binary("subtract", integer(1), json!({"kind":"intType"})),
    ] {
        for result in both(&program(tree)) {
            assert_eq!(
                result
                    .expect_err("abstract subtraction operand")
                    .to_string(),
                "arithmetic requires concrete integer operands"
            );
        }
    }
}

#[test]
fn evaluation_is_left_first_even_when_both_children_fail() {
    let overflow = binary("subtract", integer(i64::MAX), integer(-1));
    let conflict = binary("unify", integer(1), integer(2));
    for (left, right, expected) in [
        (
            overflow.clone(),
            conflict.clone(),
            "signed 64-bit integer subtraction overflow",
        ),
        (conflict, overflow, "conflicting integer values 1 and 2"),
    ] {
        for result in both(&program(binary("subtract", left, right))) {
            assert_eq!(
                result.expect_err("child evaluation must fail").to_string(),
                expected
            );
        }
    }
    // Arithmetic is checked only after evaluating both children.
    let tree = binary(
        "subtract",
        json!({"kind":"intType"}),
        binary("subtract", integer(i64::MIN), integer(1)),
    );
    for result in both(&program(tree)) {
        assert_eq!(
            result.expect_err("right child must overflow").to_string(),
            "signed 64-bit integer subtraction overflow"
        );
    }
}

#[test]
fn source_fixtures_have_known_trees_and_results() {
    let grammar = export("grammar");
    for (name, expected) in [
        ("direction", 42),
        ("negatives", 5),
        ("associativity", 5),
        ("grouped", 9),
        ("precedence", 9),
        ("refine", 5),
        ("minimum", 0),
        ("negative-result", -5),
    ] {
        let path = source("valid", name);
        let text = std::fs::read_to_string(&path).expect("read subtraction fixture");
        let parsed = parser::parse(&text, &grammar).expect("parse subtraction fixture");
        for result in both(&parsed.to_string()) {
            assert_eq!(
                result.expect("evaluate subtraction fixture"),
                expected,
                "{name}"
            );
        }
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(path)
            .output()
            .expect("run source CLI on valid fixture");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
    }
    let parsed = parser::parse("10-3-2", &grammar).expect("parse left-associative subtraction");
    assert_eq!(
        parsed,
        serde_json::from_str::<Value>(&program(binary(
            "subtract",
            binary("subtract", integer(10), integer(3)),
            integer(2)
        )))
        .expect("decode expected left-associative tree")
    );
    let parsed = parser::parse("10-(3-2)", &grammar).expect("parse grouped subtraction");
    assert_eq!(
        parsed,
        serde_json::from_str::<Value>(&program(binary(
            "subtract",
            integer(10),
            binary("subtract", integer(3), integer(2))
        )))
        .expect("decode expected grouped tree")
    );
}

#[test]
fn subtraction_uses_runtime_grammar_metadata() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("decode grammar");
    assert_eq!(grammar["operators"][3]["token"], "-");
    let mut disabled = grammar.clone();
    disabled["operators"]
        .as_array_mut()
        .expect("operators")
        .remove(3);
    assert_eq!(
        parser::parse("-2", &disabled.to_string()).expect("signed literal")["entry"],
        integer(-2)
    );
    assert_eq!(
        parser::parse("1-2", &disabled.to_string())
            .expect_err("disabled binary minus")
            .to_string(),
        "source byte 1: unexpected or unsupported syntax"
    );
    grammar["operators"][3]["associativity"] = "right".into();
    let tree =
        parser::parse("10-3-2", &grammar.to_string()).expect("parse right-associative subtraction");
    for result in both(&tree.to_string()) {
        assert_eq!(result.expect("evaluate right-associative subtraction"), 9);
    }
    grammar["operators"][3]["precedence"] = 30.into();
    let tree = parser::parse("10-3*2", &grammar.to_string()).expect("parse custom precedence");
    for result in both(&tree.to_string()) {
        assert_eq!(result.expect("evaluate custom precedence"), 14);
    }
    grammar["operators"][3]["kind"] = "customDifference".into();
    grammar["operators"][3]["leftField"] = "lhs".into();
    grammar["operators"][3]["rightField"] = "rhs".into();
    let tree =
        parser::parse("1--2", &grammar.to_string()).expect("parse custom subtraction constructor");
    assert_eq!(
        tree["entry"],
        json!({"kind":"customDifference", "lhs":integer(1), "rhs":integer(-2)})
    );
}

#[test]
fn malformed_source_and_failed_cli_never_emit_partial_output() {
    let grammar = export("grammar");
    for (text, expected) in [
        ("1 - 02", "source byte 6: leading zeroes are unsupported"),
        (
            "1\n-2",
            "source byte 2: line break after operand is unsupported",
        ),
        ("1 -", "source byte 3: expected signed decimal integer"),
        ("1 - * 2", "source byte 4: expected signed decimal integer"),
    ] {
        assert_eq!(
            parser::parse(text, &grammar)
                .expect_err("malformed subtraction must fail")
                .to_string(),
            expected
        );
    }
    for (category, name, diagnostic) in [
        ("rejected", "leading-zero", "leading zeroes are unsupported"),
        ("rejected", "trailing", "expected signed decimal integer"),
        (
            "rejected",
            "missing-operand",
            "expected signed decimal integer",
        ),
        (
            "rejected",
            "overflow",
            "signed 64-bit integer subtraction overflow",
        ),
        (
            "rejected",
            "underflow",
            "signed 64-bit integer subtraction overflow",
        ),
        (
            "rejected",
            "intermediate",
            "signed 64-bit integer subtraction overflow",
        ),
        (
            "incomplete",
            "int",
            "arithmetic requires concrete integer operands",
        ),
        (
            "incomplete",
            "bound",
            "arithmetic requires concrete integer operands",
        ),
        ("conflicts", "refine", "conflicting integer"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source(category, name))
            .output()
            .expect("run source CLI on failing fixture");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{name}: {output:?}"
        );
    }
}
