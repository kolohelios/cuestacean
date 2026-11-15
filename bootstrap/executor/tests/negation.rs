use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::{Error, execute, parser, runtime, scanner};
use runtime::Value as RuntimeValue;
use serde_json::{Value, json};

#[path = "../../generated/pratt.rs"]
mod expression;
use bootstrap_executor::generated;
#[path = "../../generated/primary-low-negation.rs"]
mod low_binding;
#[path = "../../generated/primary-no-atoms.rs"]
mod no_atoms;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn source(category: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/{category}/negation-{name}.cue"))
}

fn integer(value: i64) -> Value {
    json!({"kind":"integer", "value":value})
}

fn unary(operand: Value) -> Value {
    json!({"kind":"negate", "operand":operand})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind, "left":left, "right":right})
}

fn program(entry: Value) -> Value {
    json!({"formatVersion":1, "entry":entry})
}

fn both(tree: &Value) -> [Result<i64, Error>; 2] {
    let text = tree.to_string();
    [
        execute(&text, &export("evaluation")),
        generated::execute(&text),
    ]
}

fn interpreted(source: &str, grammar: &str, primary: &str) -> Result<Value, parser::Error> {
    parser::parse_interpreted_with_primary(source, grammar, &export("pratt"), primary)
}

#[test]
fn prefix_probe_matches_the_independent_ascii_contract() {
    let control = export("negation-prefix");
    for byte in 0..=127u8 {
        for (text, matched) in [
            (String::from_utf8(vec![byte]).expect("ASCII"), byte == b'-'),
            (
                String::from_utf8(vec![b'-', byte]).expect("ASCII"),
                !byte.is_ascii_digit(),
            ),
        ] {
            let actual = scanner::negation_prefix(&text, 0).expect("compiled prefix");
            assert_eq!(actual.flag, matched, "{text:?}");
            assert_eq!(actual.offset, usize::from(matched));
            assert_eq!(
                actual,
                scanner::interpret(&control, &text, 0).expect("reference prefix")
            );
        }
    }
    let scan = scanner::negation_prefix("λ-(1)", 2).expect("UTF-8 boundary");
    assert_eq!(scan.offset, 3);
    assert!(scan.flag);
    assert!(scanner::negation_prefix("λ-(1)", 1).is_err());
}

#[test]
fn unary_recursion_preserves_source_depth_and_node_limits() {
    let grammar = export("grammar");
    let source = format!("{}(1)", "-".repeat(31));
    assert!(parser::parse(&source, &grammar).is_ok());
    let too_deep = format!("{}(1)", "-".repeat(32));
    // 21 integers + 21 inner negations + 20 additions + 2 outer negations.
    let at_limit = format!("-(-({}))", vec!["-(1)"; 21].join("+"));
    let too_large = vec!["-(1)"; 22].join("+");
    for result in [
        parser::parse(&at_limit, &grammar),
        interpreted(&at_limit, &grammar, &export("primary")),
    ] {
        for result in both(&result.expect("64 AST nodes")) {
            assert_eq!(result.expect("value"), -21);
        }
    }
    for (text, reason) in [(too_deep, "32 parsing levels"), (too_large, "64 nodes")] {
        for result in [
            parser::parse(&text, &grammar),
            interpreted(&text, &grammar, &export("primary")),
        ] {
            assert!(
                result
                    .expect_err("resource guard")
                    .to_string()
                    .contains(reason)
            );
        }
    }
}

#[test]
fn runtime_negation_checks_concreteness_and_overflow() {
    use RuntimeValue::{Int, Integer};
    for (value, expected) in [
        (0, 0),
        (1, -1),
        (-1, 1),
        (i64::MAX, -i64::MAX),
        (i64::MIN + 1, i64::MAX),
    ] {
        assert_eq!(
            runtime::negate(Integer(value)).expect("negate integer"),
            Integer(expected)
        );
    }
    let error = runtime::negate(Integer(i64::MIN)).expect_err("negation overflow");
    assert!(matches!(error, Error::NegationOverflow));
    assert_eq!(error.to_string(), "signed 64-bit integer negation overflow");
    for value in [
        Int,
        RuntimeValue::Bounds {
            lower: Some(0),
            upper: Some(0),
        },
    ] {
        assert!(matches!(runtime::negate(value), Err(Error::Arithmetic)));
    }
}

#[test]
fn independent_evaluator_trees_and_export() {
    for result in [
        execute(&export("negation"), &export("evaluation")),
        generated::execute(&export("negation")),
    ] {
        assert_eq!(result.expect("exported negation"), -3);
    }
    for (entry, expected) in [
        (unary(binary("add", integer(1), integer(2))), -3),
        (unary(integer(i64::MIN + 1)), i64::MAX),
        (unary(unary(integer(-1))), -1),
        (integer(i64::MIN), i64::MIN),
    ] {
        for result in both(&program(entry)) {
            assert_eq!(result.expect("evaluate unary tree"), expected);
        }
    }
    for (entry, message) in [
        (
            unary(integer(i64::MIN)),
            "signed 64-bit integer negation overflow",
        ),
        (
            unary(unary(integer(i64::MIN))),
            "signed 64-bit integer negation overflow",
        ),
        (
            unary(json!({"kind":"intType"})),
            "arithmetic requires concrete integer operands",
        ),
        (
            unary(binary("unify", integer(1), integer(2))),
            "conflicting integer values 1 and 2",
        ),
        (
            unary(binary("subtract", integer(i64::MIN), integer(1))),
            "signed 64-bit integer subtraction overflow",
        ),
    ] {
        for result in both(&program(entry)) {
            assert_eq!(
                result.expect_err("unary tree must fail").to_string(),
                message
            );
        }
    }
}

#[test]
fn source_trees_values_and_cli_are_independent() {
    let sum = || binary("add", integer(1), integer(2));
    let cases = [
        ("group", unary(sum()), -3),
        (
            "binding",
            binary(
                "add",
                unary(integer(1)),
                binary("multiply", integer(2), integer(3)),
            ),
            5,
        ),
        (
            "precedence",
            binary("multiply", unary(sum()), integer(3)),
            -9,
        ),
        (
            "subtraction",
            binary(
                "subtract",
                integer(1),
                unary(binary("add", integer(2), integer(3))),
            ),
            6,
        ),
        ("chain", unary(unary(unary(integer(2)))), -2),
        ("double-signed", unary(integer(-1)), 1),
        ("space", unary(sum()), -3),
        ("newline", unary(sum()), -3),
        (
            "max",
            unary(binary("add", integer(i64::MIN), integer(1))),
            i64::MAX,
        ),
    ];
    let grammar = export("grammar");
    for (name, entry, expected) in cases {
        let path = source("valid", name);
        let text = std::fs::read_to_string(&path).expect("read source");
        let expected_tree = program(entry);
        assert_eq!(
            parser::parse(&text, &grammar).expect(name),
            expected_tree,
            "{name}"
        );
        assert_eq!(
            interpreted(&text, &grammar, &export("primary")).expect(name),
            expected_tree,
            "{name}"
        );
        for result in both(&expected_tree) {
            assert_eq!(result.expect(name), expected, "{name}");
        }
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(path)
            .output()
            .expect("source CLI");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
    }
    for (category, name, message) in [
        (
            "rejected",
            "overflow",
            "signed 64-bit integer negation overflow",
        ),
        (
            "rejected",
            "double-min",
            "signed 64-bit integer negation overflow",
        ),
        (
            "incomplete",
            "int",
            "arithmetic requires concrete integer operands",
        ),
        ("conflicts", "child", "conflicting integer"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source(category, name))
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{name}: {output:?}"
        );
    }
}

#[test]
fn grammar_controls_constructor_and_prefix_registration() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    for operator in grammar["operators"].as_array_mut().expect("operators") {
        operator["precedence"] = 100.into();
    }
    let bound = parser::parse("-(1)+2", &grammar.to_string()).expect("maximum precedence");
    assert_eq!(bound, program(binary("add", unary(integer(1)), integer(2))));
    grammar["negation"]["kind"] = "customNegation".into();
    grammar["negation"]["operandField"] = "argument".into();
    let expected = program(json!({"kind":"customNegation", "argument":integer(-2)}));
    assert_eq!(
        parser::parse("--2", &grammar.to_string()).expect("custom constructor"),
        expected
    );
    assert_eq!(
        interpreted("--2", &grammar.to_string(), &export("primary")).expect("custom interpreted"),
        expected
    );
    for (field, invalid) in [
        ("token", "+"),
        ("kind", ""),
        ("operandField", "kind"),
        ("operandField", ""),
    ] {
        let mut bad = grammar.clone();
        bad["negation"][field] = invalid.into();
        assert!(
            parser::parse("1", &bad.to_string()).is_err(),
            "{field}: {invalid}"
        );
    }
    grammar["negation"]["operandField"] = "1bad".into();
    assert_eq!(
        parser::parse("--2", &grammar.to_string()).expect("JSON field need not be an identifier"),
        program(json!({"kind":"customNegation", "1bad":integer(-2)}))
    );
    grammar
        .as_object_mut()
        .expect("grammar object")
        .remove("negation");
    let grammar = grammar.to_string();
    assert_eq!(
        parser::parse("-2", &grammar).expect("signed literal"),
        program(integer(-2))
    );
    assert_eq!(
        parser::parse("-9223372036854775808", &grammar).expect("minimum literal"),
        program(integer(i64::MIN))
    );
    assert!(parser::parse("-(2)", &grammar).is_err());
    assert!(interpreted("-(2)", &grammar, &export("primary")).is_err());
    assert!(
        parser::parse("-02", &grammar)
            .expect_err("leading zero")
            .to_string()
            .contains("leading zeroes")
    );
}

#[test]
fn cue_controls_unary_binding_not_a_native_constant() {
    let grammar = export("grammar");
    let text = "-(1)+2";
    let normal = parser::parse(text, &grammar).expect("normal binding");
    assert_eq!(
        normal,
        program(binary("add", unary(integer(1)), integer(2)))
    );
    let alternate =
        parser::parse_with_drivers(text, &grammar, expression::execute, low_binding::execute)
            .expect("alternate binding");
    assert_eq!(
        alternate,
        program(unary(binary("add", integer(1), integer(2))))
    );
    assert_eq!(
        alternate,
        interpreted(text, &grammar, &export("tests/valid/primary-low-negation"))
            .expect("reference binding")
    );
    for result in both(&normal) {
        assert_eq!(result.expect("normal result"), 1);
    }
    for result in both(&alternate) {
        assert_eq!(result.expect("alternate result"), -3);
    }
}

#[test]
fn recursive_negation_preserves_selected_primary_driver() {
    let grammar = export("grammar");
    let control = export("tests/valid/primary-no-atoms");
    for text in ["-int", "-(-int)", "1 + -(int)"] {
        for result in [
            parser::parse_with_drivers(text, &grammar, expression::execute, no_atoms::execute),
            interpreted(text, &grammar, &control),
        ] {
            assert!(result.is_err(), "alternate driver must reject {text}");
        }
        assert!(
            parser::parse(text, &grammar).is_ok(),
            "default driver: {text}"
        );
    }
}

#[test]
fn recursive_negation_preserves_the_expression_driver_too() {
    let grammar = export("grammar");
    let text = "-(10-3-2)";
    let normal = parser::parse(text, &grammar).expect("normal tree");
    for result in both(&normal) {
        assert_eq!(result.expect("normal value"), -5);
    }
    let expected = program(unary(binary(
        "subtract",
        integer(10),
        binary("subtract", integer(3), integer(2)),
    )));
    let compiled =
        parser::parse_with_drivers(text, &grammar, right_expression::execute, no_atoms::execute)
            .expect("alternate expression through negation");
    assert_eq!(compiled, expected);
    assert_eq!(
        parser::parse_interpreted_with_primary(
            text,
            &grammar,
            &export("tests/valid/pratt-right"),
            &export("tests/valid/primary-no-atoms")
        )
        .expect("reference alternate"),
        expected
    );
    for result in both(&compiled) {
        assert_eq!(result.expect("alternate value"), -9);
    }
}

#[test]
fn invalid_negation_transitions_and_unfinished_dispatch_are_guarded() {
    let grammar = export("grammar");
    for op in [
        "scanNegation",
        "advanceNegation",
        "buildNegation",
        "parseNegation",
    ] {
        let statement = if op == "parseNegation" {
            json!({"op":op, "minimum":{"op":"integer", "value":101}})
        } else {
            json!({"op":op})
        };
        let control = json!({"formatVersion":1, "body":[statement]});
        assert!(
            matches!(
                interpreted("-(1)", &grammar, &control.to_string()),
                Err(parser::Error::ControlState { .. })
            ),
            "{op}"
        );
    }
    let mut control: Value = serde_json::from_str(&export("primary")).expect("primary");
    let group = control["body"]
        .as_array_mut()
        .expect("body")
        .iter_mut()
        .find(|statement| statement["op"] == "if")
        .expect("group branch");
    let nested = group["else"]
        .as_array_mut()
        .expect("group else")
        .iter_mut()
        .find(|statement| statement["op"] == "if" && statement["condition"]["op"] == "hasNegation")
        .expect("negation branch");
    let then = nested["then"].as_array_mut().expect("negation then");
    let index = then
        .iter()
        .position(|statement| statement["op"] == "buildNegation")
        .expect("build negation");
    then.remove(index);
    assert!(matches!(
        interpreted("-(1)", &grammar, &control.to_string()),
        Err(parser::Error::ControlState { .. })
    ));
}
