use std::path::Path;

use bootstrap_executor::{Error, execute_json, generated, parser};
use serde_json::{Value, json};

#[path = "../../generated/primary-call-high-minimum.rs"]
mod call_high;
#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/record-primary-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary-no-records.rs"]
mod old_primary;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/record-primary.rs"]
mod record_primary;
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
fn record(fields: Value) -> Value {
    json!({"kind":"recordExpression","fields":fields})
}
fn program(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}
fn sum(left: Value, right: Value) -> Value {
    json!({"kind":"add","left":left,"right":right})
}
fn selected(source: &str, grammar: &str, policy: &str) -> [Result<Value, parser::Error>; 2] {
    let default: parser::Driver = expression::execute;
    let mut drivers = [default, primary::execute, record_primary::execute];
    let mut names = ["pratt", "primary", "record-primary"];
    let (index, driver, name): (usize, parser::Driver, &str) = match policy {
        "high" => (2, high_minimum::execute, "record-primary-high-minimum"),
        "old" => (1, old_primary::execute, "primary-no-records"),
        "right" => (0, right_expression::execute, "pratt-right"),
        "call-high" => (1, call_high::execute, "primary-call-high-minimum"),
        _ => panic!("unknown test policy"),
    };
    drivers[index] = driver;
    let alternate = format!("tests/valid/{name}");
    names[index] = &alternate;
    [
        parser::parse_with_three_drivers(source, grammar, drivers[0], drivers[1], drivers[2]),
        parser::parse_interpreted_with_three_drivers(
            source,
            grammar,
            &export(names[0]),
            &export(names[1]),
            &export(names[2]),
        ),
    ]
}
fn parses(source: &str, grammar: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn evaluations(tree: &Value) -> [Result<Value, Error>; 2] {
    [
        execute_json(&tree.to_string(), &export("evaluation")),
        generated::execute_json(&tree.to_string()),
    ]
}
fn source_error(result: Result<Value, parser::Error>, offset: usize, expected: &str) {
    let error = result.expect_err("source rejection");
    assert!(
        matches!(&error, parser::Error::Source { offset: actual, reason }
            if *actual == offset && reason == expected),
        "{error:?}"
    );
}

#[test]
fn independent_complete_trees_values_labels_and_mixed_indexing() {
    let grammar = export("grammar");
    let nested = record(json!([{"name":"x","value":integer(7)}]));
    let mixed = record(json!([{"name":"a","value":{"kind":"listExpression","elements":[nested]}}]));
    for (source, entry, value) in [
        ("close({// empty\n}// call\n)", record(json!([])), json!({})),
        (
            "close({\n\"a\" :1+2, // next\n\"b\":true,\"c\":null,})",
            record(
                json!([{"name":"a","value":sum(integer(1),integer(2))},{"name":"b","value":{"kind":"true"}},{"name":"c","value":{"kind":"null"}}]),
            ),
            json!({"a":3,"b":true,"c":null}),
        ),
        (
            r#"close({"\u00e9\U0001F642":"text","quote\"":-9223372036854775808})"#,
            record(
                json!([{"name":"é🙂","value":{"kind":"string","value":"text"}},{"name":"quote\"","value":integer(i64::MIN)}]),
            ),
            json!({"é🙂":"text","quote\"":i64::MIN}),
        ),
        (
            "close({\"a\":[close({\"x\":7})]})[\"a\"][0][\"x\"]",
            json!({"kind":"index","left":{"kind":"index","left":{"kind":"index","left":mixed,"right":{"kind":"string","value":"a"}},"right":integer(0)},"right":{"kind":"string","value":"x"}}),
            json!(7),
        ),
        (
            "len(close({\"x\":9223372036854775807}))",
            json!({"kind":"length","operand":record(json!([{"name":"x","value":integer(i64::MAX)}]))}),
            json!(1),
        ),
    ] {
        for parsed in parses(source, &grammar) {
            let tree = parsed.expect(source);
            assert_eq!(tree, program(entry.clone()), "{source}");
            for result in evaluations(&tree) {
                assert_eq!(result.expect(source), value, "{source}");
            }
        }
    }
}

#[test]
fn delimiter_errors_and_conservative_newlines_are_independent() {
    let grammar = export("grammar");
    for (source, offset, reason) in [
        ("close([])", 6, "expected opening brace"),
        ("close({a:1})", 7, "expected quoted record label"),
        ("close({\"a\" 1})", 11, "expected colon"),
        ("close({\"a\":1 2})", 13, "expected comma or closing brace"),
        ("close({\"a\":1,,})", 13, "expected quoted record label"),
        ("close({}", 8, "expected closing parenthesis"),
    ] {
        for parsed in parses(source, &grammar) {
            source_error(parsed, offset, reason);
        }
    }
    for source in [
        "close({\"a\"\n:1})",
        "close({\"a\":1\n})",
        "close({\"a\":1 // tail\n})",
        "close\n({})",
        "{\"a\":1}",
        "close({[1]:2})",
        "close({\"a\":1,...})",
        "close({\"a\"/*no*/:1})",
    ] {
        for parsed in parses(source, &grammar) {
            assert!(parsed.is_err(), "{source}");
        }
    }
}

#[test]
fn name_and_duplicate_errors_are_runtime_but_whole_source_parses_first() {
    let grammar = export("grammar");
    let expected = program(record(
        json!([{"name":"a","value":integer(1)},{"name":"a","value":{"kind":"divide","left":integer(1),"right":integer(0)}}]),
    ));
    for parsed in parses("close({\"a\":1,\"\\u0061\":1 div 0})", &grammar) {
        let tree = parsed.expect("duplicate syntax preserved");
        assert_eq!(tree, expected);
        for result in evaluations(&tree) {
            assert!(matches!(result, Err(Error::RecordDuplicate { field }) if field == "a"));
        }
    }
    for parsed in parses("close({\"\":1 div 0})", &grammar) {
        for result in evaluations(&parsed.expect("name checks are runtime")) {
            assert!(matches!(result, Err(Error::RecordName { index: 0 })));
        }
    }
    // Limits apply to decoded UTF-8 name bytes, not source escape spelling.
    for (spelling, decoded, accepted) in [
        ("a".repeat(64), "a".repeat(64), true),
        ("a".repeat(65), "a".repeat(65), false),
        ("\\u00e9".repeat(32), "é".repeat(32), true),
        (
            format!("{}x", "\\u00e9".repeat(32)),
            format!("{}x", "é".repeat(32)),
            false,
        ),
        ("\\U0001F642".repeat(16), "🙂".repeat(16), true),
    ] {
        for parsed in parses(&format!("close({{\"{spelling}\":1}})"), &grammar) {
            for result in evaluations(&parsed.expect("label syntax")) {
                if accepted {
                    assert_eq!(result.expect("64 decoded bytes")[&decoded], json!(1));
                } else {
                    assert!(matches!(result, Err(Error::RecordName { index: 0 })));
                }
            }
        }
    }
    for parsed in parses("close({\"a\":1,\"a\":2,\"later\":})", &grammar) {
        assert!(matches!(parsed, Err(parser::Error::Source { .. })));
    }
}

#[test]
fn optional_metadata_and_collisions_are_checked_before_source_use() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let original = json!({"token":"close","kind":"recordExpression","fieldsField":"fields"});
    assert_eq!(grammar["record"], original);
    grammar["record"] = json!({"token":"sealed","kind":"customRecord","fieldsField":"entries"});
    for parsed in parses("sealed({\"a\":1})", &grammar.to_string()) {
        assert_eq!(
            parsed.expect("custom metadata"),
            program(json!({"kind":"customRecord","entries":[{"name":"a","value":integer(1)}]}))
        );
    }
    grammar.as_object_mut().expect("object").remove("record");
    for parsed in parses("close({})", &grammar.to_string()) {
        assert!(parsed.is_err());
    }
    for (field, value) in [
        ("token", ""),
        ("token", "bad-token"),
        ("token", "len"),
        ("token", "int"),
        ("kind", ""),
        ("fieldsField", ""),
        ("fieldsField", "kind"),
    ] {
        grammar["record"] = original.clone();
        grammar["record"][field] = value.into();
        for parsed in parses("1", &grammar.to_string()) {
            assert!(matches!(
                parsed,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
}

#[test]
fn source_nodes_and_recursive_levels_are_not_runtime_capacity() {
    let grammar = export("grammar");
    let wide = |count| {
        let fields: Vec<_> = (0..count).map(|i| format!("\"f{i}\":1")).collect();
        format!("close({{{}}})", fields.join(","))
    };
    let deep = |depth| format!("{}1{}", "close({\"a\":".repeat(depth), "})".repeat(depth));
    for (source, succeeds, reason) in [
        (wide(63), true, ""),
        (wide(64), false, "expression exceeds 64 nodes"),
        (deep(32), true, ""),
        (deep(33), false, "expression exceeds 32 parsing levels"),
    ] {
        for parsed in parses(&source, &grammar) {
            if succeeds {
                assert!(parsed.is_ok(), "{parsed:?}");
            } else {
                let message = parsed.expect_err("shared budget").to_string();
                assert!(message.contains(reason), "{message}");
            }
        }
    }
}

#[test]
fn genuine_record_high_minimum_and_old_primary_have_no_fallback() {
    let grammar = export("grammar");
    for source in [
        "close({\"a\":1+2})",
        "(close({\"a\":1+2}))",
        "1+close({\"a\":1+2})",
        "[close({\"a\":1+2})]",
        "close({\"outer\":close({\"a\":1+2})})",
    ] {
        assert!(parser::parse(source, &grammar).is_ok(), "{source}");
        for parsed in selected(source, &grammar, "high") {
            source_error(
                parsed,
                source.rfind('+').expect("operator"),
                "expected comma or closing brace",
            );
        }
    }
    for source in ["close({})", "close({\"a\":(1+2)})"] {
        for parsed in selected(source, &grammar, "high") {
            assert_eq!(
                parsed.expect(source),
                parser::parse(source, &grammar).expect("production")
            );
        }
    }
    for source in ["close({})", "(close({}))", "1+close({})", "[close({})]"] {
        for parsed in selected(source, &grammar, "old") {
            source_error(parsed, source.find("close").expect("token"), "unknown atom");
        }
    }
    let legacy = json!({"kind":"length","operand":{"kind":"listExpression","elements":[sum(integer(1),integer(2))]}});
    for parsed in selected("len([1+2])", &grammar, "old") {
        assert_eq!(parsed.expect("legacy"), program(legacy.clone()));
    }
}

#[test]
fn expression_and_primary_engines_reach_nested_record_fields() {
    let grammar = export("grammar");
    let entry = record(
        json!([{"name":"outer","value":{"kind":"listExpression","elements":[record(json!([{"name":"a","value":sum(integer(1),sum(integer(2),integer(3)))}]))]}}]),
    );
    for parsed in selected(
        "(close({\"outer\":[close({\"a\":1+2+3})]}))",
        &grammar,
        "right",
    ) {
        assert_eq!(
            parsed.expect("right associative fields"),
            program(entry.clone())
        );
    }
    for source in [
        "close({\"a\":len(1+2)})",
        "close({\"outer\":[close({\"a\":len(1+2)})]})",
        "1+close({\"a\":(len(1+2))})",
    ] {
        assert!(parser::parse(source, &grammar).is_ok(), "{source}");
        for parsed in selected(source, &grammar, "call-high") {
            source_error(
                parsed,
                source.rfind('+').expect("operator"),
                "expected closing parenthesis",
            );
        }
    }
}
