use std::path::Path;

use bootstrap_executor::{Error, execute_json, generated, parser};
use serde_json::{Value, json};

#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary-bound-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary.rs"]
mod production_primary;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn integer(value: i64) -> Value {
    json!({"kind":"integer", "value":value})
}

fn bound(kind: &str, operand: Value) -> Value {
    json!({"kind":kind, "operand":operand})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind, "left":left, "right":right})
}

fn program(entry: Value) -> Value {
    json!({"formatVersion":1, "entry":entry})
}

fn parses(source: &str, grammar: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}

fn evaluations(tree: &Value) -> [Result<Value, Error>; 2] {
    let text = tree.to_string();
    [
        execute_json(&text, &export("evaluation")),
        generated::execute_json(&text),
    ]
}

fn source_error(result: Result<Value, parser::Error>, offset: usize, reason: &str) {
    match result.expect_err("source must fail") {
        parser::Error::Source {
            offset: actual,
            reason: message,
        } => {
            assert_eq!(actual, offset);
            assert_eq!(message, reason);
        }
        other => panic!("expected source error, got {other:?}"),
    }
}

#[test]
fn independent_source_trees_and_exact_json_values() {
    let grammar = export("grammar");
    let sum = || binary("add", integer(1), integer(2));
    for (source, entry, value) in [
        (
            ">=(1+2)&3",
            binary("unify", bound("lowerBoundExpression", sum()), integer(3)),
            3,
        ),
        (
            "<=(0x20 div 2)&16",
            binary(
                "unify",
                bound(
                    "upperBoundExpression",
                    binary("divide", integer(32), integer(2)),
                ),
                integer(16),
            ),
            16,
        ),
        (
            ">= // threshold\n (3)&3",
            binary(
                "unify",
                bound("lowerBoundExpression", integer(3)),
                integer(3),
            ),
            3,
        ),
        (
            ">=3&3",
            binary("unify", json!({"kind":"lowerBound","value":3}), integer(3)),
            3,
        ),
        (
            ">=1+2",
            binary("add", json!({"kind":"lowerBound","value":1}), integer(2)),
            0,
        ),
    ] {
        for result in parses(source, &grammar) {
            let actual = result.expect(source);
            assert_eq!(actual, program(entry.clone()), "{source}");
            for evaluated in evaluations(&actual) {
                if source == ">=1+2" {
                    assert!(matches!(evaluated, Err(Error::Arithmetic)));
                } else {
                    assert_eq!(evaluated.expect(source).to_string(), value.to_string());
                }
            }
        }
    }
}

#[test]
fn direct_json_extrema_infinity_and_incomplete_bounds() {
    for kind in ["lowerBoundExpression", "upperBoundExpression"] {
        for n in [i64::MIN, i64::MAX, -1, 0, 7] {
            let threshold = bound(kind, integer(n));
            for result in evaluations(&program(threshold.clone())) {
                assert!(matches!(result, Err(Error::IncompleteBounds)));
            }
            for result in evaluations(&program(binary("unify", threshold, integer(n)))) {
                assert_eq!(
                    result.expect("inclusive candidate").to_string(),
                    n.to_string()
                );
            }
        }
    }
    for entry in [
        binary(
            "unify",
            bound("lowerBoundExpression", integer(i64::MIN)),
            integer(i64::MAX),
        ),
        binary(
            "unify",
            bound("upperBoundExpression", integer(i64::MAX)),
            integer(i64::MIN),
        ),
    ] {
        let expected = entry["right"]["value"].to_string();
        for result in evaluations(&program(entry)) {
            assert_eq!(
                result.expect("infinite opposite endpoint").to_string(),
                expected
            );
        }
    }
    let singleton = binary(
        "unify",
        bound("lowerBoundExpression", integer(3)),
        bound("upperBoundExpression", integer(3)),
    );
    for result in evaluations(&program(singleton)) {
        assert!(matches!(result, Err(Error::IncompleteBounds)));
    }
}

#[test]
fn threshold_domains_and_eager_child_errors_are_independent() {
    for kind in ["lowerBoundExpression", "upperBoundExpression"] {
        for operand in [
            json!({"kind":"null"}),
            json!({"kind":"top"}),
            json!({"kind":"true"}),
            json!({"kind":"string","value":"3"}),
            json!({"kind":"intType"}),
            json!({"kind":"boolType"}),
            json!({"kind":"stringType"}),
            json!({"kind":"lowerBound","value":3}),
            bound("upperBoundExpression", integer(3)),
        ] {
            for result in evaluations(&program(bound(kind, operand))) {
                assert!(matches!(result, Err(Error::BoundThreshold)));
            }
        }
        for (operand, message) in [
            (
                binary("add", integer(i64::MAX), integer(1)),
                "signed 64-bit integer addition overflow",
            ),
            (
                binary("divide", integer(1), integer(0)),
                "integer division by zero",
            ),
            (
                binary("unify", integer(1), integer(2)),
                "conflicting integer values 1 and 2",
            ),
            (
                bound("negate", integer(i64::MIN)),
                "signed 64-bit integer negation overflow",
            ),
        ] {
            let failing = bound(kind, operand);
            for entry in [
                failing.clone(),
                binary("unify", failing.clone(), integer(0)),
                binary(
                    "add",
                    failing.clone(),
                    binary("divide", integer(1), integer(0)),
                ),
                binary("unify", binary("divide", integer(1), integer(0)), failing),
            ] {
                let expected = if entry["left"]["kind"] == "divide" {
                    "integer division by zero"
                } else {
                    message
                };
                for result in evaluations(&program(entry)) {
                    assert_eq!(result.expect_err("child failure").to_string(), expected);
                }
            }
        }
    }
}

#[test]
fn source_semantic_failures_and_missing_close_diagnostics() {
    let grammar = export("grammar");
    for (source, message) in [
        (">=(true)", "bound threshold requires a concrete integer"),
        ("<=(int)", "bound threshold requires a concrete integer"),
        (">=(1 div 0)", "integer division by zero"),
        (
            ">=(9223372036854775807+1)",
            "signed 64-bit integer addition overflow",
        ),
        (">=(4)&3", "conflicting integer bounds"),
        ("<=(2)&3", "conflicting integer bounds"),
        (">=(3)&<=(3)", "non-concrete integer bounds"),
    ] {
        for tree in parses(source, &grammar) {
            for result in evaluations(&tree.expect(source)) {
                assert_eq!(result.expect_err(source).to_string(), message, "{source}");
            }
        }
    }
    for (source, offset, reason) in [
        (">=(1+2", 6, "expected closing parenthesis"),
        ("<=(3", 4, "expected closing parenthesis"),
        (">=()", 3, "expected signed decimal integer"),
        (">=(unknown)", 3, "unknown atom"),
        ("> =3", 2, "expected signed decimal integer"),
    ] {
        for result in parses(source, &grammar) {
            source_error(result, offset, reason);
        }
    }
}

#[test]
fn runtime_metadata_renames_disables_and_validates_before_parsing() {
    let original: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    for index in 0..2 {
        let token = original["bounds"][index]["token"].as_str().expect("token");
        let mut grammar = original.clone();
        grammar["bounds"][index]["expression"] =
            json!({"kind":"customBound","operandField":"child"});
        for tree in parses(&format!("{token}(3)"), &grammar.to_string()) {
            assert_eq!(
                tree.expect("custom constructor"),
                program(json!({"kind":"customBound","child":integer(3)}))
            );
        }
        grammar["bounds"][index]
            .as_object_mut()
            .expect("bound")
            .remove("expression");
        for result in parses(&format!("{token}(3)"), &grammar.to_string()) {
            source_error(result, 2, "expected signed decimal integer");
        }
        for result in parses(&format!("{token}3"), &grammar.to_string()) {
            assert_eq!(result.expect("legacy literal")["entry"]["value"], 3);
        }
        let other = &original["bounds"][1 - index];
        let other_token = other["token"].as_str().expect("other prefix");
        for result in parses(&format!("{other_token}(3)"), &grammar.to_string()) {
            assert_eq!(
                result.expect("other prefix remains enabled")["entry"],
                bound(
                    other["expression"]["kind"].as_str().expect("kind"),
                    integer(3)
                )
            );
        }
        for metadata in [
            json!({"kind":"","operandField":"operand"}),
            json!({"kind":"custom","operandField":""}),
            json!({"kind":"custom","operandField":"kind"}),
            json!({"kind":"custom"}),
            json!({"kind":"custom","operandField":"operand","extra":true}),
        ] {
            grammar["bounds"][index]["expression"] = metadata;
            let text = grammar.to_string();
            for result in parses("3", &text) {
                assert!(matches!(
                    result,
                    Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
                ));
            }
            assert_eq!(
                grammar.to_string(),
                text,
                "validation must not mutate metadata"
            );
        }
    }
}

#[test]
fn computed_groups_share_depth_and_wrapper_node_budgets() {
    let grammar = export("grammar");
    let deep = |n| format!("{}1{}", ">=(".repeat(n), ")".repeat(n));
    let at_limit = format!(">=(>=({}))", vec![">=(1)"; 21].join("&"));
    for source in [deep(32), at_limit] {
        for result in parses(&source, &grammar) {
            assert!(result.is_ok(), "{source}: {result:?}");
        }
    }
    for (source, reason) in [
        (deep(33), "expression exceeds 32 parsing levels"),
        (vec![">=(1)"; 22].join("&"), "expression exceeds 64 nodes"),
    ] {
        for result in parses(&source, &grammar) {
            match result.expect_err("shared resource guard") {
                parser::Error::Source { reason: actual, .. } => assert_eq!(actual, reason),
                other => panic!("unexpected error {other:?}"),
            }
        }
    }
}

#[test]
fn actual_alternate_primary_minimum_propagates_to_nested_and_rhs_groups() {
    let grammar = export("grammar");
    let alternate = export("tests/valid/primary-bound-high-minimum");
    for (source, offset) in [
        (">=(1+2)&3", 4),
        ("(>=(1+2))&3", 5),
        ("3&>=(1+2)", 6),
        (">=(<=(1+2))", 7),
    ] {
        for result in parses(source, &grammar) {
            assert!(result.is_ok(), "production minimum: {source}");
        }
        for result in [
            parser::parse_with_drivers(
                source,
                &grammar,
                expression::execute,
                high_minimum::execute,
            ),
            parser::parse_interpreted_with_primary(source, &grammar, &export("pratt"), &alternate),
        ] {
            source_error(result, offset, "expected closing parenthesis");
        }
    }
    for source in [">=3&3", "(1+2)", ">=((1+2))&3"] {
        let expected = parser::parse(source, &grammar).expect("production");
        assert_eq!(
            parser::parse_with_drivers(
                source,
                &grammar,
                expression::execute,
                high_minimum::execute
            )
            .expect(source),
            expected
        );
        assert_eq!(
            parser::parse_interpreted_with_primary(source, &grammar, &export("pratt"), &alternate)
                .expect(source),
            expected
        );
    }
}

#[test]
fn selected_expression_driver_reaches_threshold_groups() {
    let grammar = export("grammar");
    let primary = export("primary");
    for source in [">=(1+2+3)", "(>=(1+2+3))", "6&>=(1+2+3)"] {
        let threshold = bound(
            "lowerBoundExpression",
            binary("add", integer(1), binary("add", integer(2), integer(3))),
        );
        let expected = program(if source.starts_with("6&") {
            binary("unify", integer(6), threshold)
        } else {
            threshold
        });
        // Keep production primary selection while replacing only the expression driver.
        assert_eq!(
            parser::parse_interpreted_with_primary(
                source,
                &grammar,
                &export("tests/valid/pratt-right"),
                &primary
            )
            .expect(source),
            expected
        );
        assert_eq!(
            parser::parse_with_drivers(
                source,
                &grammar,
                right_expression::execute,
                production_primary::execute
            )
            .expect(source),
            expected
        );
    }
}
