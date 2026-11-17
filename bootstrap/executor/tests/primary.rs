use bootstrap_executor::parser;
use serde_json::{Value, json};
use std::path::Path;

#[path = "../../generated/primary-cross-phase.rs"]
mod cross_phase;
#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary-no-atoms.rs"]
mod no_atoms;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn interpreted(source: &str, grammar: &str, primary_program: &str) -> Result<Value, parser::Error> {
    parser::parse_interpreted_with_primary(source, grammar, &export("pratt"), primary_program)
}

fn check_tree(source: &str, grammar: &str, driver: parser::Driver, program: &str, expected: Value) {
    let compiled =
        parser::parse_with_drivers(source, grammar, expression::execute, driver).expect(source);
    assert_eq!(compiled, expected, "{source}");
    assert_eq!(
        interpreted(source, grammar, program).expect(source),
        expected,
        "{source}"
    );
}

fn check_source_error(result: Result<Value, parser::Error>, offset: usize, reason: &str) {
    match result.expect_err("source rejection") {
        parser::Error::Source {
            offset: actual,
            reason: message,
        } => {
            assert_eq!(actual, offset);
            assert_eq!(message, reason);
        }
        other => panic!("expected source diagnostic, got {other:?}"),
    }
}

fn program(entry: Value) -> Value {
    json!({"formatVersion": 1, "entry": entry})
}

#[test]
fn production_builds_concrete_trees_and_reports_diagnostics() {
    let grammar = export("grammar");
    let control = export("primary");
    for (source, entry) in [
        ("int", json!({"kind":"intType"})),
        (" >= -2", json!({"kind":"lowerBound", "value":-2})),
        (
            "(1 + (2 * 3))",
            json!({"kind":"add", "left":{"kind":"integer", "value":1}, "right":{"kind":"multiply", "left":{"kind":"integer", "value":2}, "right":{"kind":"integer", "value":3}}}),
        ),
    ] {
        let expected = program(entry);
        check_tree(
            source,
            &grammar,
            primary::execute,
            &control,
            expected.clone(),
        );
        assert_eq!(parser::parse(source, &grammar).expect(source), expected);
        assert_eq!(
            parser::parse_interpreted(source, &grammar, &export("pratt")).expect(source),
            expected
        );
    }
    for (source, offset, reason) in [
        ("()", 1, "expected signed decimal integer"),
        ("(1 + 2", 6, "expected closing parenthesis"),
        ("(unknown)", 1, "unknown atom"),
    ] {
        check_source_error(parser::parse(source, &grammar), offset, reason);
        check_source_error(interpreted(source, &grammar, &control), offset, reason);
    }
}

#[test]
fn alternate_primary_policies_propagate_through_groups_and_rhs() {
    let grammar = export("grammar");
    let no_atoms_json = export("tests/valid/primary-no-atoms");
    for (source, offset) in [("int", 0), ("(int)", 1), ("1 + (2 + int)", 9)] {
        check_source_error(
            parser::parse_with_drivers(source, &grammar, expression::execute, no_atoms::execute),
            offset,
            "expected signed decimal integer",
        );
        check_source_error(
            interpreted(source, &grammar, &no_atoms_json),
            offset,
            "expected signed decimal integer",
        );
    }
    check_tree(
        "(1 + 2)",
        &grammar,
        no_atoms::execute,
        &no_atoms_json,
        program(
            json!({"kind":"add", "left":{"kind":"integer", "value":1}, "right":{"kind":"integer", "value":2}}),
        ),
    );

    let high_json = export("tests/valid/primary-high-minimum");
    for (source, offset) in [("(1 + 2)", 3), ("((1 + 2))", 4), ("1 + (2 + 3)", 7)] {
        assert!(parser::parse(source, &grammar).is_ok(), "{source}");
        check_source_error(
            parser::parse_with_drivers(
                source,
                &grammar,
                expression::execute,
                high_minimum::execute,
            ),
            offset,
            "expected closing parenthesis",
        );
        check_source_error(
            interpreted(source, &grammar, &high_json),
            offset,
            "expected closing parenthesis",
        );
    }
    check_tree(
        "((1)) + 2",
        &grammar,
        high_minimum::execute,
        &high_json,
        program(
            json!({"kind":"add", "left":{"kind":"integer", "value":1}, "right":{"kind":"integer", "value":2}}),
        ),
    );
}

#[test]
fn expression_driver_also_propagates_into_groups_and_rhs() {
    let grammar = export("grammar");
    let source = "(1 + 2 + 3) * (4 + 5 + 6)";
    let sum = |a, b, c| json!({"kind":"add", "left":{"kind":"integer", "value":a}, "right":{"kind":"add", "left":{"kind":"integer", "value":b}, "right":{"kind":"integer", "value":c}}});
    let expected = program(json!({"kind":"multiply", "left":sum(1, 2, 3), "right":sum(4, 5, 6)}));
    assert_eq!(
        parser::parse_with_drivers(
            source,
            &grammar,
            right_expression::execute,
            no_atoms::execute
        )
        .expect("compiled"),
        expected
    );
    assert_eq!(
        parser::parse_interpreted_with_primary(
            source,
            &grammar,
            &export("tests/valid/pratt-right"),
            &export("tests/valid/primary-no-atoms")
        )
        .expect("interpreted"),
        expected
    );
}

#[test]
fn primary_uses_runtime_grammar_for_atoms_bounds_and_constructors() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    grammar["atoms"][0]["token"] = "Number_9".into();
    grammar["atoms"][0]["kind"] = "customAtom".into();
    grammar["bounds"][0]["kind"] = "floor".into();
    grammar["bounds"][0]["valueField"] = "threshold".into();
    grammar["integer"]["kind"] = "literal".into();
    grammar["integer"]["valueField"] = "number".into();
    grammar["operators"][0]["kind"] = "sum".into();
    grammar["operators"][0]["leftField"] = "first".into();
    grammar["operators"][0]["rightField"] = "second".into();
    let grammar = serde_json::to_string(&grammar).expect("grammar JSON");
    check_tree(
        "(Number_9 + >= -2) + 3",
        &grammar,
        primary::execute,
        &export("primary"),
        program(
            json!({"kind":"sum", "first":{"kind":"sum", "first":{"kind":"customAtom"}, "second":{"kind":"floor", "threshold":-2}}, "second":{"kind":"literal", "number":3}}),
        ),
    );
    check_source_error(
        interpreted(
            "(Number_9)",
            &grammar,
            &export("tests/valid/primary-no-atoms"),
        ),
        1,
        "expected signed decimal integer",
    );
}

#[test]
fn primary_scope_rejects_expression_actions_and_invalid_transitions() {
    let grammar = export("grammar");
    for source in ["int+2", "(1)+2"] {
        for result in [
            parser::parse_with_drivers(source, &grammar, expression::execute, cross_phase::execute),
            interpreted(source, &grammar, &export("tests/valid/primary-cross-phase")),
        ] {
            assert!(matches!(
                result,
                Err(parser::Error::ControlState {
                    reason: "expression operation during primary dispatch"
                })
            ));
        }
    }
    for op in [
        "afterOperand",
        "afterWordOperator",
        "lookupOperator",
        "advanceOperator",
        "parseRight",
        "buildBinary",
        "setBindingPower",
    ] {
        let mut control: Value = serde_json::from_str(&export("primary")).expect("primary");
        let statement = if op == "setBindingPower" {
            json!({"op":op,"value":{"op":"integer","value":1}})
        } else {
            json!({"op":op})
        };
        let body = control["body"].as_array_mut().expect("body");
        // Reach the phase guard before the final return, not unreachable validation.
        body.insert(body.len() - 1, statement);
        assert!(
            matches!(
                interpreted("1", &grammar, &control.to_string()),
                Err(parser::Error::ControlState {
                    reason: "expression operation during primary dispatch"
                })
            ),
            "{op}"
        );
    }
    for body in [
        json!([{"op":"if","condition":{"op":"groupOpen"},"then":[]}]),
        json!([{"op":"scanGroup"}, {"op":"scanBound"}, {"op":"parseBound"}]),
    ] {
        let control = json!({"formatVersion":1,"body":body});
        assert!(matches!(
            interpreted("1", &grammar, &control.to_string()),
            Err(parser::Error::ControlState { .. })
        ));
    }
}

#[test]
fn primary_and_expression_share_the_same_finite_step_budget() {
    fn nearly_spent(frame: &mut parser::Frame<'_, '_>) -> Result<(), parser::Error> {
        for _ in 0..4088 {
            frame.step()?;
        }
        expression::execute(frame)
    }
    let grammar = export("grammar");
    assert!(matches!(
        parser::parse_with_drivers("1", &grammar, nearly_spent, primary::execute),
        Err(parser::Error::ControlBudget)
    ));
    let mut expression: Value = serde_json::from_str(&export("pratt")).expect("expression");
    let body = expression["body"]
        .as_array_mut()
        .expect("body")
        .iter_mut()
        .find(|statement| statement["op"] == "while")
        .expect("loop")["body"]
        .as_array_mut()
        .expect("loop body");
    body.extend((0..121).map(|_| json!({"op":"afterOperand"})));
    let mut primary: Value = serde_json::from_str(&export("primary")).expect("primary");
    primary["body"]
        .as_array_mut()
        .expect("body")
        .splice(0..0, (0..32).map(|_| json!({"op":"whitespace"})));
    let source = vec!["1"; 32].join("+");
    assert!(
        parser::parse_interpreted_with_primary(
            &source,
            &grammar,
            &expression.to_string(),
            &export("primary")
        )
        .is_ok()
    );
    assert!(interpreted(&source, &grammar, &primary.to_string()).is_ok());
    assert!(matches!(
        parser::parse_interpreted_with_primary(
            &source,
            &grammar,
            &expression.to_string(),
            &primary.to_string()
        ),
        Err(parser::Error::ControlBudget)
    ));
}

#[test]
fn primary_reentry_unfinished_state_and_existing_limits_are_guarded() {
    let grammar = export("grammar");
    let expression_json = export("pratt");
    let mut reentry: Value = serde_json::from_str(&expression_json).expect("expression");
    reentry["body"] = json!([{ "op": "primary" }]);
    assert!(matches!(
        interpreted("1", &grammar, &reentry.to_string()),
        Err(parser::Error::ControlState { .. })
    ));

    let mut illegal: Value = serde_json::from_str(&expression_json).expect("expression");
    illegal["body"] = json!([{ "op": "scanGroup" }]);
    assert!(matches!(
        parser::parse_interpreted("1", &grammar, &illegal.to_string()),
        Err(parser::Error::ControlState { .. })
    ));
    let mut empty: Value = serde_json::from_str(&export("primary")).expect("primary");
    empty["body"] = json!([]);
    assert!(matches!(
        interpreted("1", &grammar, &empty.to_string()),
        Err(parser::Error::ControlState { .. })
    ));
    assert!(matches!(
        interpreted("1", &grammar, &export("tests/valid/pratt-stalled")),
        Err(parser::Error::ControlBudget)
    ));

    let mut unfinished: Value = serde_json::from_str(&export("primary")).expect("primary");
    let group = unfinished["body"]
        .as_array_mut()
        .expect("body")
        .iter_mut()
        .find(|statement| statement["op"] == "if")
        .expect("group branch");
    let then = group["then"].as_array_mut().expect("then");
    let close = then
        .iter()
        .position(|statement| statement["op"] == "closeGroup")
        .expect("closeGroup");
    then.remove(close);
    assert!(matches!(
        interpreted("(1)", &grammar, &unfinished.to_string()),
        Err(parser::Error::ControlState { .. })
    ));

    for (file, reason) in [("depth", "32 parsing levels"), ("nodes", "64 nodes")] {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/rejected/{file}.cue"));
        let source = std::fs::read_to_string(path).expect("source");
        for result in [
            parser::parse(&source, &grammar),
            interpreted(&source, &grammar, &export("primary")),
        ] {
            assert!(
                result.expect_err(file).to_string().contains(reason),
                "{file}"
            );
        }
    }
}
