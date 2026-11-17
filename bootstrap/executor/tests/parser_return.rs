use bootstrap_executor::{parser, pratt};
use serde_json::{Value, json};
use std::path::Path;

#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary-nested.rs"]
mod nested;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/primary-return.rs"]
mod primary_return;
#[path = "../../generated/primary-return-empty.rs"]
mod return_empty;
#[path = "../../generated/pratt-return-loop.rs"]
mod return_loop;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn normalized(result: Result<Value, parser::Error>) -> Result<Value, String> {
    result.map_err(|error| error.to_string())
}

fn program(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}

fn literal(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}

fn source_error(result: Result<Value, parser::Error>, offset: usize, reason: &str) {
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

#[test]
fn flat_and_historical_nested_primary_agree_over_corpus_with_calls_and_lists_disabled() {
    // Compare the historical language, not subsequent call/list dispatch.
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let object = grammar.as_object_mut().expect("grammar object");
    object.remove("calls");
    object.remove("list");
    let grammar = grammar.to_string();
    let expression_json = export("pratt");
    let flat_json = export("primary");
    let nested_json = export("tests/valid/primary-nested");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../source");
    for directory in ["valid", "rejected", "conflicts", "incomplete"] {
        for entry in std::fs::read_dir(root.join(directory)).expect("fixtures") {
            let path = entry.expect("fixture").path();
            if path.extension().is_none_or(|extension| extension != "cue") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("source");
            let expected = normalized(parser::parse_with_drivers(
                &source,
                &grammar,
                expression::execute,
                nested::execute,
            ));
            for actual in [
                parser::parse(&source, &grammar),
                parser::parse_interpreted_with_primary(
                    &source,
                    &grammar,
                    &expression_json,
                    &flat_json,
                ),
                parser::parse_interpreted_with_primary(
                    &source,
                    &grammar,
                    &expression_json,
                    &nested_json,
                ),
            ] {
                assert_eq!(normalized(actual), expected, "{path:?}");
            }
        }
    }
    assert_eq!(
        pratt::generate(&nested_json).expect("historical generation"),
        pratt::generate(&nested_json).expect("repeat historical generation")
    );
}

#[test]
fn flat_return_is_scoped_to_each_group_rhs_and_prefix_invocation() {
    let grammar = export("grammar");
    let expression_json = export("pratt");
    let flat_json = export("primary");
    let nested_json = export("tests/valid/primary-nested");
    for (source, entry) in [
        (
            "(1)+(2*3)",
            json!({"kind":"add","left":literal(1),"right":{"kind":"multiply","left":literal(2),"right":literal(3)}}),
        ),
        (
            "!(false)",
            json!({"kind":"logicalNot","operand":{"kind":"false"}}),
        ),
        ("-(2)", json!({"kind":"negate","operand":literal(2)})),
        (
            ">=(1+2)",
            json!({"kind":"lowerBoundExpression","operand":{"kind":"add","left":literal(1),"right":literal(2)}}),
        ),
        ("\"x\"", json!({"kind":"string","value":"x"})),
        ("int", json!({"kind":"intType"})),
        ("-2", literal(-2)),
    ] {
        let expected = program(entry);
        for driver in [primary::execute as parser::Driver, nested::execute] {
            assert_eq!(
                parser::parse_with_drivers(source, &grammar, expression::execute, driver)
                    .expect(source),
                expected,
                "{source}"
            );
        }
        for control in [&flat_json, &nested_json] {
            assert_eq!(
                parser::parse_interpreted_with_primary(source, &grammar, &expression_json, control)
                    .expect(source),
                expected,
                "{source}"
            );
        }
    }
    for (source, offset, reason) in [
        ("()", 1, "expected signed decimal integer"),
        ("(1", 2, "expected closing parenthesis"),
        ("unknown", 0, "unknown atom"),
        ("1+", 2, "expected signed decimal integer"),
    ] {
        source_error(parser::parse(source, &grammar), offset, reason);
        for control in [&flat_json, &nested_json] {
            source_error(
                parser::parse_interpreted_with_primary(source, &grammar, &expression_json, control),
                offset,
                reason,
            );
        }
    }
}

#[test]
fn return_unwinds_both_if_and_loop_without_parsing_an_operator() {
    let grammar = export("grammar");
    let control = export("tests/valid/pratt-return-loop");
    let primary_json = export("primary");
    for source in ["1", "(1)", "((1))"] {
        for result in [
            parser::parse_with_drivers(source, &grammar, return_loop::execute, primary::execute),
            parser::parse_interpreted_with_primary(source, &grammar, &control, &primary_json),
        ] {
            assert_eq!(result.expect(source), program(literal(1)));
        }
    }
    for (source, offset, reason) in [
        ("1+2", 1, "unexpected or unsupported syntax"),
        ("(1+2)", 2, "expected closing parenthesis"),
    ] {
        source_error(
            parser::parse_with_drivers(source, &grammar, return_loop::execute, primary::execute),
            offset,
            reason,
        );
        source_error(
            parser::parse_interpreted_with_primary(source, &grammar, &control, &primary_json),
            offset,
            reason,
        );
    }
}

#[test]
fn both_returning_arms_preserve_primary_completion_checks() {
    let grammar = export("grammar");
    let expression_json = export("pratt");
    let control = export("tests/valid/primary-return");
    for result in [
        parser::parse_with_drivers("1", &grammar, expression::execute, primary_return::execute),
        parser::parse_interpreted_with_primary("1", &grammar, &expression_json, &control),
    ] {
        assert_eq!(result.expect("completed literal"), program(literal(1)));
    }
    for (source, driver, control) in [
        ("(1)", primary_return::execute as parser::Driver, control),
        (
            "1",
            return_empty::execute,
            export("tests/valid/primary-return-empty"),
        ),
    ] {
        for result in [
            parser::parse_with_drivers(source, &grammar, expression::execute, driver),
            parser::parse_interpreted_with_primary(source, &grammar, &expression_json, &control),
        ] {
            assert!(matches!(
                result,
                Err(parser::Error::ControlState {
                    reason: "unfinished primary at return"
                })
            ));
        }
    }
}

#[test]
fn return_consumes_one_step_from_the_shared_frame_budget() {
    fn spent(frame: &mut parser::Frame<'_, '_>) -> Result<(), parser::Error> {
        // The expression driver's primary instruction already consumed step 1.
        for _ in 0..4095 {
            frame.step()?;
        }
        return_empty::execute(frame)
    }
    assert!(matches!(
        parser::parse_with_drivers("1", &export("grammar"), expression::execute, spent),
        Err(parser::Error::ControlBudget)
    ));
}

#[test]
fn original_control_is_eagerly_validated_before_return_or_backend_execution() {
    let grammar = export("grammar");
    let ret = json!({"op":"return"});
    let both = json!({
        "op":"if","condition":{"op":"boolean","value":true},
        "then":[ret.clone()],"else":[ret.clone()]
    });
    let invalid = json!({"op":"unknownTestKind"});
    let cases = [
        (
            json!({"formatVersion":1,"body":[{"op":"return","position":"tail"}]}),
            "unknown field",
        ),
        (
            json!({"formatVersion":1,"body":[ret.clone()],"tail":"return"}),
            "unknown field",
        ),
        (
            json!({"formatVersion":1,"body":[{"op":"if","condition":{"op":"boolean","value":true},"then":[ret.clone()],"else":[invalid.clone()]}]}),
            "unknown variant",
        ),
        (
            json!({"formatVersion":1,"body":[ret.clone(),{"op":"primary"}]}),
            "statements after definite return",
        ),
        (
            json!({"formatVersion":1,"body":[both,{"op":"primary"}]}),
            "statements after definite return",
        ),
        (
            json!({"formatVersion":1,"body":[{"op":"while","condition":{"op":"boolean","value":false},"body":[ret.clone(),{"op":"primary"}]}]}),
            "statements after definite return",
        ),
        (
            json!({"formatVersion":1,"body":[{"op":"while","condition":{"op":"boolean","value":false},"body":[invalid]}]}),
            "unknown variant",
        ),
        (
            json!({"formatVersion":1,"body":[ret,{"op":"return","extra":true}]}),
            "unknown field",
        ),
    ];
    for (control, reason) in cases {
        let text = control.to_string();
        let compiler = pratt::generate(&text)
            .expect_err("compiler rejection")
            .to_string();
        let interpreter = parser::parse_interpreted("1", &grammar, &text)
            .expect_err("expression rejection")
            .to_string();
        let primary =
            parser::parse_interpreted_with_primary("1", &grammar, &export("pratt"), &text)
                .expect_err("primary rejection")
                .to_string();
        for error in [compiler, interpreter, primary] {
            assert!(error.contains(reason), "{text}: {error}");
        }
    }
}

#[test]
fn structural_return_analysis_does_not_fold_conditions_or_assume_loops_return() {
    let ret = json!({"op":"return"});
    for body in [
        json!([{"op":"if","condition":{"op":"boolean","value":true},"then":[ret.clone()]},{"op":"primary"}]),
        json!([{"op":"while","condition":{"op":"boolean","value":true},"body":[ret]},{"op":"primary"}]),
    ] {
        let control = json!({"formatVersion":1,"body":body});
        pratt::generate(&control.to_string()).expect("not structurally unreachable");
    }
}
