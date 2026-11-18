use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, pratt, runtime};
use serde_json::{Value, json};

#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary-call-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary-no-lists.rs"]
mod no_lists;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
}
fn list(elements: Value) -> Value {
    json!({"kind":"listExpression","elements":elements})
}
fn program(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}
fn parses(source: &str, grammar: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn evaluations(tree: &Value) -> [Result<Value, Error>; 2] {
    let input = tree.to_string();
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn primary_parses(
    source: &str,
    grammar: &str,
    driver: parser::Driver,
    name: &str,
) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse_with_drivers(source, grammar, expression::execute, driver),
        parser::parse_interpreted_with_primary(
            source,
            grammar,
            &export("pratt"),
            &export(&format!("tests/valid/{name}")),
        ),
    ]
}
fn at(result: Result<Value, parser::Error>, offset: usize) -> String {
    match result.expect_err("source rejection") {
        parser::Error::Source {
            offset: actual,
            reason,
        } => {
            assert_eq!(actual, offset);
            reason
        }
        other => panic!("expected source error, got {other:?}"),
    }
}

#[test]
fn independent_complete_trees_values_and_explicit_comma_trivia() {
    let grammar = export("grammar");
    let sum = binary("add", integer(1), integer(2));
    for (source, elements, value) in [
        ("[]", json!([]), json!([])),
        (
            "[1+2,\"x\",null,true]",
            json!([sum.clone(), {"kind":"string","value":"x"}, {"kind":"null"}, {"kind":"true"}]),
            json!([3, "x", null, true]),
        ),
        (
            "[\n// leading\n1+2, // next\nfalse,]",
            json!([sum, {"kind":"false"}]),
            json!([3, false]),
        ),
        ("[ // empty\n]", json!([]), json!([])),
        (
            "[-9223372036854775808,9223372036854775807]",
            json!([integer(i64::MIN), integer(i64::MAX)]),
            json!([i64::MIN, i64::MAX]),
        ),
    ] {
        for parsed in parses(source, &grammar) {
            let tree = parsed.expect(source);
            assert_eq!(tree, program(list(elements.clone())), "{source}");
            for result in evaluations(&tree) {
                assert_eq!(result.expect(source), value, "{source}");
            }
        }
    }
    for (source, count) in [
        ("len([])", 0),
        ("len([1+2])", 1),
        ("len([null,true,\"é\"])", 3),
        ("len(\"é🙂\")", 6),
    ] {
        for parsed in parses(source, &grammar) {
            let tree = parsed.expect(source);
            assert_eq!(tree["entry"]["kind"], "length");
            for result in evaluations(&tree) {
                assert_eq!(result.expect(source), json!(count));
            }
        }
    }
}

#[test]
fn child_and_abstract_admission_failures_precede_enclosing_length() {
    let grammar = export("grammar");
    for (source, message) in [
        ("[1 div 0]", "integer division by zero"),
        ("len([1 div 0])", "integer division by zero"),
        (
            "len(3)",
            "length requires a concrete string or closed collection",
        ),
        (
            "len(_)",
            "length requires a concrete string or closed collection",
        ),
        (
            "len(string)",
            "length requires a concrete string or closed collection",
        ),
    ] {
        for parsed in parses(source, &grammar) {
            for result in evaluations(&parsed.expect(source)) {
                assert_eq!(result.expect_err(source).to_string(), message);
            }
        }
    }
    for source in ["[[_]]", "[int,1 div 0]", "len([int,1 div 0])"] {
        for parsed in parses(source, &grammar) {
            for result in evaluations(&parsed.expect("nested syntax parses")) {
                assert!(
                    matches!(result, Err(Error::ListElement { index: 0 })),
                    "{source}: {result:?}"
                );
            }
        }
    }
}

#[test]
fn generic_runtime_length_does_not_widen_legacy_domains() {
    use runtime::{ClosedList, Value as V};
    let text = V::Text(Arc::from("é🙂"));
    let items = V::List(ClosedList::new(vec![V::Null, V::Integer(7)]).expect("flat list"));
    assert_eq!(runtime::length(text.clone()).expect("bytes"), V::Integer(6));
    assert_eq!(
        runtime::length(items.clone()).expect("items"),
        V::Integer(2)
    );
    assert!(matches!(runtime::string_length(items), Err(Error::Length)));
    assert!(matches!(runtime::list_length(text), Err(Error::ListLength)));
    for value in [
        V::Top,
        V::Null,
        V::Int,
        V::String,
        V::Bool,
        V::Boolean(true),
        V::Integer(1),
        V::Bounds {
            lower: None,
            upper: None,
        },
    ] {
        assert!(matches!(runtime::length(value), Err(Error::LengthOperand)));
    }
}

#[test]
fn malformed_lists_and_newline_after_operand_still_reject() {
    let grammar = export("grammar");
    for source in [
        "[",
        "[1",
        "[,1]",
        "[1,,2]",
        "[1 2]",
        "[1\n]",
        "[1\n,2]",
        "[1 // comment\n]",
        "[1,...]",
        "[1]x",
    ] {
        let [compiled, interpreted] = parses(source, &grammar);
        let error = compiled.expect_err(source).to_string();
        assert_eq!(interpreted.expect_err(source).to_string(), error);
    }
}

#[test]
fn optional_constructor_metadata_is_validated_before_any_source() {
    let original: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    assert_eq!(
        original["list"],
        json!({"kind":"listExpression","elementsField":"elements"})
    );
    let mut grammar = original.clone();
    grammar["list"] = json!({"kind":"customArray","elementsField":"children"});
    for parsed in parses("[1]", &grammar.to_string()) {
        assert_eq!(
            parsed.expect("custom fields"),
            program(json!({"kind":"customArray","children":[integer(1)]}))
        );
    }
    grammar.as_object_mut().expect("object").remove("list");
    for parsed in parses("[]", &grammar.to_string()) {
        at(parsed, 0);
    }
    for parsed in parses("1+(2)", &grammar.to_string()) {
        assert!(parsed.is_ok());
    }
    for metadata in [
        json!({"kind":"","elementsField":"elements"}),
        json!({"kind":"custom","elementsField":""}),
        json!({"kind":"custom","elementsField":"kind"}),
        json!({"kind":"custom"}),
        json!({"kind":"custom","elementsField":"children","extra":true}),
    ] {
        grammar["list"] = metadata;
        for parsed in parses("1", &grammar.to_string()) {
            assert!(matches!(
                parsed,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
}

#[test]
fn list_container_and_elements_share_source_node_and_depth_limits() {
    let grammar = export("grammar");
    for (count, succeeds) in [(63, true), (64, false)] {
        let source = format!("[{}]", vec!["1"; count].join(","));
        for parsed in parses(&source, &grammar) {
            if succeeds {
                for result in evaluations(&parsed.expect("63 leaves plus container")) {
                    assert_eq!(result.expect("list"), json!(vec![1; count]));
                }
            } else {
                assert!(
                    parsed
                        .expect_err("64-node budget")
                        .to_string()
                        .contains("expression exceeds 64 nodes")
                );
            }
        }
    }
    for (count, succeeds) in [(32, true), (33, false)] {
        let source = format!("{}1{}", "[".repeat(count), "]".repeat(count));
        for parsed in parses(&source, &grammar) {
            if succeeds {
                assert!(parsed.is_ok(), "{parsed:?}");
            } else {
                assert!(
                    parsed
                        .expect_err("shared depth")
                        .to_string()
                        .contains("expression exceeds 32 parsing levels")
                );
            }
        }
    }
}

#[test]
fn actual_old_primary_and_high_minimum_have_no_native_policy_fallback() {
    let grammar = export("grammar");
    for (source, offset) in [("[]", 0), ("([])", 1), ("1+[]", 2)] {
        for parsed in primary_parses(source, &grammar, no_lists::execute, "primary-no-lists") {
            at(parsed, offset);
        }
    }
    for source in ["1+(2*3)", "!false", "len(\"é\")"] {
        let expected = parser::parse(source, &grammar).expect("production");
        for parsed in primary_parses(source, &grammar, no_lists::execute, "primary-no-lists") {
            assert_eq!(parsed.expect(source), expected);
        }
    }
    for (source, offset) in [("[1+2]", 2), ("([1+2])", 3), ("1+[1+2]", 4), ("[[1+2]]", 3)] {
        for parsed in parses(source, &grammar) {
            assert!(parsed.is_ok(), "default minimum: {source}");
        }
        for parsed in primary_parses(
            source,
            &grammar,
            high_minimum::execute,
            "primary-call-high-minimum",
        ) {
            assert_eq!(at(parsed, offset), "expected comma or closing bracket");
        }
    }
    for source in ["[]", "[(1+2)]", "[1,2,]", "(1+2)"] {
        let expected = parser::parse(source, &grammar).expect("production");
        for parsed in primary_parses(
            source,
            &grammar,
            high_minimum::execute,
            "primary-call-high-minimum",
        ) {
            assert_eq!(parsed.expect(source), expected);
        }
    }
}

#[test]
fn selected_right_associative_driver_reaches_list_elements() {
    let grammar = export("grammar");
    for source in ["[1+2+3]", "([1+2+3])", "len([1+2+3])"] {
        let array = list(json!([binary(
            "add",
            integer(1),
            binary("add", integer(2), integer(3))
        )]));
        let expected = program(if source.starts_with("len") {
            json!({"kind":"length","operand":array})
        } else {
            array
        });
        for parsed in [
            parser::parse_with_drivers(
                source,
                &grammar,
                right_expression::execute,
                primary::execute,
            ),
            parser::parse_interpreted_with_primary(
                source,
                &grammar,
                &export("tests/valid/pratt-right"),
                &export("primary"),
            ),
        ] {
            let tree = parsed.expect(source);
            assert_eq!(tree, expected);
            for result in evaluations(&tree) {
                assert_eq!(
                    result.expect(source),
                    if source.starts_with("len") {
                        json!(1)
                    } else {
                        json!([6])
                    }
                );
            }
        }
    }
}

#[test]
fn list_protocol_rejects_out_of_order_and_unfinished_stages() {
    let grammar = export("grammar");
    for (body, reason) in [
        (json!([{"op":"buildList"}]), "invalid primary stage"),
        (json!([{"op":"appendListElement"}]), "invalid primary stage"),
        (
            json!([{"op":"whitespace"},{"op":"scanGroup"},{"op":"scanNot"},
            {"op":"scanNegation"},{"op":"scanBound"},{"op":"scanString"},
            {"op":"scanList"},{"op":"advanceList"},{"op":"return"}]),
            "unfinished primary at return",
        ),
    ] {
        let control = json!({"formatVersion":1,"body":body}).to_string();
        pratt::generate(&control).expect("well-typed parser actions");
        assert!(matches!(
            parser::parse_interpreted_with_primary("[1]", &grammar, &export("pratt"), &control),
            Err(parser::Error::ControlState { reason: actual }) if actual == reason
        ));
    }
}

#[test]
fn new_source_operations_reject_cross_family_even_in_dead_branches() {
    for action in [
        "scanList",
        "advanceList",
        "probeListEnd",
        "appendListElement",
        "advanceListTail",
        "buildList",
    ] {
        let control = json!({"formatVersion":1,"target":"list","body":[{"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":action}]}]}).to_string();
        assert!(pratt::generate(&control).is_err(), "{action}");
    }
    for condition in ["hasList", "hasListElement"] {
        let control = json!({"formatVersion":1,"target":"list","body":[{"op":"if","condition":{"op":condition},"then":[]}]}).to_string();
        assert!(pratt::generate(&control).is_err(), "{condition}");
    }
}

#[test]
fn cli_array_success_and_failures_are_atomic() {
    let path = std::env::temp_dir().join(format!(
        "cuestacean-source-lists-{}.cue",
        std::process::id()
    ));
    for (source, expected) in [
        ("[1+2,null,\"é\"]", Some("[3,null,\"é\"]\n")),
        ("[1,1 div 0]", None),
        ("[1,,2]", None),
        ("len([[]])", Some("1\n")),
        ("len([int])", None),
    ] {
        std::fs::write(&path, source).expect("write source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        if let Some(expected) = expected {
            assert!(output.status.success());
            assert_eq!(String::from_utf8(output.stdout).expect("JSON"), expected);
            assert!(output.stderr.is_empty());
        } else {
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty(), "{source}: {output:?}");
            assert!(!output.stderr.is_empty());
        }
    }
    std::fs::remove_file(path).expect("remove source");
}
