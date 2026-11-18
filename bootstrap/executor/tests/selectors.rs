use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, runtime};
use serde_json::{Value, json};

#[path = "../../generated/pratt-index-high-minimum.rs"]
mod high;
#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/pratt-no-index.rs"]
mod no_index;
#[path = "../../generated/pratt-no-selectors.rs"]
mod old;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/record-primary.rs"]
mod record;
#[path = "../../generated/pratt-right.rs"]
mod right;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn select(operand: Value, label: &str) -> Value {
    json!({"kind":"select","operand":operand,"label":label})
}
fn field(label: &str, value: Value) -> Value {
    json!({"kind":"recordExpression","fields":[{"name":label,"value":value}]})
}
fn list(element: Value) -> Value {
    json!({"kind":"listExpression","elements":[element]})
}
fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
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
fn evaluations(entry: Value) -> [Result<Value, Error>; 2] {
    let input = program(entry).to_string();
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn selected(source: &str, name: &str, driver: parser::Driver) -> [Result<Value, parser::Error>; 2] {
    let grammar = export("grammar");
    let primary = primary::execute as parser::Driver;
    let record = record::execute as parser::Driver;
    [
        parser::parse_with_three_drivers(source, &grammar, driver, primary, record),
        parser::parse_interpreted_with_three_drivers(
            source,
            &grammar,
            &export(&format!("tests/valid/{name}")),
            &export("primary"),
            &export("record-primary"),
        ),
    ]
}

#[test]
fn independent_trees_values_and_mixed_postfix_precedence() {
    let leaf = field("x", integer(7));
    let outer = field("a", list(leaf.clone()));
    for (source, entry, expected) in [
        (
            "close({\"a\":[close({\"x\":7})]}).a[0].x",
            select(binary("index", select(outer, "a"), integer(0)), "x"),
            json!(7),
        ),
        (
            "-close({\"x\":7}).x+2*3",
            binary(
                "add",
                json!({"kind":"negate","operand":select(leaf.clone(), "x")}),
                binary("multiply", integer(2), integer(3)),
            ),
            json!(-1),
        ),
        (
            "close({\"a\":close({\"x\":7})}).a.x",
            select(select(field("a", leaf), "a"), "x"),
            json!(7),
        ),
    ] {
        for parsed in parses(source, &export("grammar")) {
            let tree = parsed.expect(source);
            assert_eq!(tree, program(entry.clone()), "{source}");
            for result in evaluations(tree["entry"].clone()) {
                assert_eq!(result.expect(source), expected, "{source}");
            }
        }
    }
}

#[test]
fn keyword_case_quoted_labels_and_cue_owned_trivia() {
    for (label, suffix) in [
        ("true", ".true"),
        ("for", ".for"),
        ("let", ".let"),
        ("int", ".int"),
        ("Upper_9", ".Upper_9"),
        ("_x", ".\"_x\""),
        ("#x", ".\"#x\""),
        ("é🙂", ".\"\\u00e9\\U0001F642\""),
        ("a-b", ".\"a-b\""),
        ("a\"b", ".\"a\\\"b\""),
        ("x", ". // label\n x"),
    ] {
        let source = format!("close({{{}:7}}){suffix}", json!(label));
        for parsed in parses(&source, &export("grammar")) {
            assert_eq!(
                parsed.expect(&source),
                program(select(field(label, integer(7)), label))
            );
        }
        for value in evaluations(select(field(label, integer(7)), label)) {
            assert_eq!(value.expect("regular label"), json!(7));
        }
    }
    for suffix in ["._x", ".#x", ".é", ".9x", ".", ". // eof"] {
        let source = format!("close({{\"_x\":7}}){suffix}");
        for result in parses(&source, &export("grammar")) {
            assert!(
                matches!(result, Err(parser::Error::Source { reason, .. })
                if reason == "expected selector label"),
                "{source}"
            );
        }
    }
    for source in [
        "close({})\n.x",
        "close({}) // before dot\n.x",
        "close({}).\"bad\\q\"",
    ] {
        for result in parses(source, &export("grammar")) {
            assert!(result.is_err(), "{source}");
        }
    }
}

#[test]
fn strict_lookup_and_operand_errors_precede_label_decoding() {
    for operand in [
        integer(1),
        json!({"kind":"string","value":"x"}),
        json!({"kind":"listExpression","elements":[]}),
    ] {
        for result in evaluations(select(operand, "x")) {
            assert!(matches!(result, Err(Error::RecordIndexOperands)));
        }
    }
    for result in evaluations(select(field("X", integer(1)), "x")) {
        assert!(matches!(result, Err(Error::RecordMissing { field }) if field == "x"));
    }
    for label in [Value::Null, json!(1), json!(true), json!([]), json!({})] {
        let mut entry = select(field("x", integer(1)), "x");
        entry["label"] = label;
        for result in evaluations(entry.clone()) {
            assert!(matches!(result, Err(Error::Text { field }) if field == "label"));
        }
        entry["operand"] = json!({"kind":"unknownTestKind"});
        for result in evaluations(entry) {
            assert!(matches!(result, Err(Error::MissingRule { .. })));
        }
    }
    let mut missing = select(field("x", integer(1)), "x");
    missing.as_object_mut().expect("object").remove("label");
    for result in evaluations(missing.clone()) {
        assert!(matches!(result, Err(Error::MissingField { field }) if field == "label"));
    }
    missing["operand"] = json!({"kind":"unknownTestKind"});
    for result in evaluations(missing) {
        assert!(matches!(result, Err(Error::MissingRule { .. })));
    }
}

#[test]
fn metadata_is_optional_eager_and_has_no_constructor_fallback() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let original = json!({"kind":"select","operandField":"operand","labelField":"label"});
    assert_eq!(grammar["selector"], original);
    grammar["selector"] = json!({"kind":"customSelect","operandField":"base","labelField":"key"});
    for result in parses("1.x", &grammar.to_string()) {
        assert_eq!(
            result.expect("custom metadata"),
            program(json!({"kind":"customSelect","base":integer(1),"key":"x"}))
        );
    }
    grammar.as_object_mut().expect("object").remove("selector");
    for result in parses("1.x", &grammar.to_string()) {
        assert!(result.is_err());
    }
    for result in parses("[7][0]", &grammar.to_string()) {
        assert_eq!(
            result.expect("brackets unaffected"),
            program(binary("index", list(integer(7)), integer(0)))
        );
    }
    for (key, value) in [
        ("kind", ""),
        ("operandField", ""),
        ("labelField", ""),
        ("operandField", "kind"),
        ("labelField", "kind"),
        ("labelField", "operand"),
    ] {
        grammar["selector"] = original.clone();
        grammar["selector"][key] = value.into();
        for result in parses("1", &grammar.to_string()) {
            assert!(matches!(
                result,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
}

#[test]
fn historical_controls_and_alternate_engines_propagate_without_fallback() {
    for source in [
        "close({\"x\":7}).x",
        "(close({\"x\":7}).x)",
        "1+close({\"x\":7}).x",
        "close({\"a\":close({\"x\":7}).x})",
    ] {
        for result in selected(source, "pratt-no-selectors", old::execute) {
            assert!(result.is_err(), "{source}");
        }
        for result in selected(source, "pratt-no-index", no_index::execute) {
            assert!(result.is_err(), "{source}");
        }
        for result in selected(source, "pratt-index-high-minimum", high::execute) {
            assert!(result.is_ok(), "{source}: {result:?}");
        }
    }
    for result in selected("[7][0]", "pratt-no-selectors", old::execute) {
        assert_eq!(
            result.expect("historical bracket control"),
            program(binary("index", list(integer(7)), integer(0)))
        );
    }
    for source in [
        "[7][0+0].x",
        "([7][0+0].x)",
        "1+[7][0+0].x",
        "close({\"a\":[7][0+0].x})",
    ] {
        for result in selected(source, "pratt-index-high-minimum", high::execute) {
            assert!(result.is_err(), "{source}");
        }
    }
    let chosen = select(field("x", integer(7)), "x");
    let sum = binary("add", integer(1), binary("add", integer(2), chosen));
    for (source, expected) in [
        ("1+2+close({\"x\":7}).x", sum.clone()),
        ("(1+2+close({\"x\":7}).x)", sum.clone()),
        ("close({\"a\":1+2+close({\"x\":7}).x})", field("a", sum)),
    ] {
        for result in selected(source, "pratt-right", right::execute) {
            assert_eq!(result.expect(source), program(expected.clone()));
        }
    }
}

#[test]
fn selector_labels_cost_neither_source_nodes_nor_ast_visits() {
    let grammar = export("grammar");
    for (count, succeeds) in [(63, true), (64, false)] {
        let source = format!("1{}", ".x".repeat(count));
        for result in parses(&source, &grammar) {
            if succeeds {
                assert!(result.is_ok(), "{result:?}");
            } else {
                let error = result.expect_err("node budget").to_string();
                assert!(error.contains("64 nodes"));
            }
        }
    }
    for (depth, succeeds) in [(32, true), (33, false)] {
        let source = format!("{}1.x{}", "(".repeat(depth), ")".repeat(depth));
        for result in parses(&source, &grammar) {
            assert_eq!(result.is_ok(), succeeds, "{result:?}");
        }
    }
    let mut leaf = integer(7);
    for _ in 0..62 {
        leaf = json!({"kind":"negate","operand":leaf});
    }
    for result in evaluations(select(leaf.clone(), "x")) {
        assert!(matches!(result, Err(Error::RecordIndexOperands)));
    }
    leaf = json!({"kind":"negate","operand":leaf});
    for result in evaluations(select(leaf, "x")) {
        assert!(matches!(result, Err(Error::EvaluationDepth)));
    }
    let mut spine = integer(0);
    for _ in 0..11 {
        spine = binary("add", spine, integer(0));
    }
    // Selected alternate evaluates add's left twice: operand consumes 4095 visits.
    for (extra, exhausted) in [(0, false), (1, true)] {
        let mut operand = spine.clone();
        for _ in 0..extra {
            operand = json!({"kind":"negate","operand":operand});
        }
        let input = program(select(operand, "x")).to_string();
        let legacy = left_twice::execute(&input);
        if exhausted {
            assert!(matches!(legacy, Err(Error::EvaluationVisits)));
        } else {
            assert!(matches!(legacy, Err(Error::RecordIndexOperands)));
        }
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if exhausted {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            } else {
                assert!(matches!(result, Err(Error::RecordIndexOperands)));
            }
        }
    }
}

#[test]
fn selected_children_outlive_their_record_and_input() {
    let input = json!({"payload":[{"name":"x","value":"é🙂"}]});
    let parent = runtime::record(&input, "payload").expect("record");
    let runtime::Value::Text(child) =
        runtime::record_index(parent.clone(), runtime::Value::Text(Arc::from("x")))
            .expect("selected child")
    else {
        panic!("expected text child")
    };
    let retained = child.clone();
    drop(parent);
    drop(input);
    drop(child);
    assert_eq!(&*retained, "é🙂");
    for result in evaluations(select(field("x", field("y", integer(i64::MAX))), "x")) {
        assert_eq!(result.expect("owned nested result"), json!({"y":i64::MAX}));
    }
}
