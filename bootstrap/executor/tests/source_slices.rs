use std::path::Path;

use bootstrap_executor::{Error, execute_json, generated, parser};
use serde_json::{Value, json};

#[path = "../../generated/primary-call-high-minimum.rs"]
mod call_high;
#[path = "../../generated/pratt-slice-high-minimum.rs"]
mod high;
#[path = "../../generated/pratt-no-slices.rs"]
mod old;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/record-primary.rs"]
mod record;
#[path = "../../generated/record-primary-high-minimum.rs"]
mod record_high;

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
fn array(elements: Value) -> Value {
    json!({"kind":"listExpression","elements":elements})
}
fn field(name: &str, value: Value) -> Value {
    json!({"kind":"recordExpression","fields":[{"name":name,"value":value}]})
}
fn select(operand: Value, label: &str) -> Value {
    json!({"kind":"select","operand":operand,"label":label})
}
fn slice(operand: Value, lower: Value, upper: Value) -> Value {
    json!({"kind":"listSlice","operand":operand,"lower":lower,"upper":upper})
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
fn selected(
    source: &str,
    name: &str,
    driver: parser::Driver,
    index: usize,
) -> [Result<Value, parser::Error>; 2] {
    let grammar = export("grammar");
    let mut drivers = [
        high::execute as parser::Driver,
        primary::execute,
        record::execute,
    ];
    let mut names = [
        "tests/valid/pratt-slice-high-minimum".to_owned(),
        "primary".to_owned(),
        "record-primary".to_owned(),
    ];
    drivers[index] = driver;
    names[index] = format!("tests/valid/{name}");
    [
        parser::parse_with_three_drivers(source, &grammar, drivers[0], drivers[1], drivers[2]),
        parser::parse_interpreted_with_three_drivers(
            source,
            &grammar,
            &export(&names[0]),
            &export(&names[1]),
            &export(&names[2]),
        ),
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
fn independent_computed_mixed_nested_unicode_and_precedence_oracles() {
    let items = array(json!([integer(10), integer(20), integer(30)]));
    let nested = array(json!([array(json!([field("é🙂", integer(i64::MAX))]))]));
    let first = slice(select(field("a", nested), "a"), integer(0), integer(1));
    let second = slice(binary("index", first, integer(0)), integer(0), integer(1));
    let record_items = field(
        "items",
        array(json!([field(
            "x",
            array(json!([integer(0), integer(i64::MAX), integer(i64::MIN)]))
        )])),
    );
    let record_item = binary(
        "index",
        slice(select(record_items, "items"), integer(0), integer(1)),
        integer(0),
    );
    for (source, entry, value) in [
        (
            "[10,20,30][0+1:3-0]",
            json!({"kind":"listSlice","operand":items,"lower":binary("add",integer(0),integer(1)),"upper":binary("subtract",integer(3),integer(0))}),
            json!([20, 30]),
        ),
        (
            "[10,20,30][3:3]",
            slice(items.clone(), integer(3), integer(3)),
            json!([]),
        ),
        (
            "- [10,20,30][0:2][1]+2*3",
            json!({"kind":"add","left":{"kind":"negate","operand":binary("index",slice(items,integer(0),integer(2)),integer(1))},"right":binary("multiply",integer(2),integer(3))}),
            json!(-14),
        ),
        (
            r#"close({"a":[[close({"é🙂":9223372036854775807})]]}).a[0:1][0][0:1][0]."\u00e9\U0001F642""#,
            select(binary("index", second, integer(0)), "é🙂"),
            json!(i64::MAX),
        ),
        (
            r#"close({"items":[close({"x":[0,9223372036854775807,-9223372036854775808]})]}).items[0:1][0].x[1:3]"#,
            slice(select(record_item, "x"), integer(1), integer(3)),
            json!([i64::MAX, i64::MIN]),
        ),
        (
            "[-9223372036854775808,9223372036854775807][0:2]",
            json!({"kind":"listSlice","operand":array(json!([integer(i64::MIN),integer(i64::MAX)])),"lower":integer(0),"upper":integer(2)}),
            json!([i64::MIN, i64::MAX]),
        ),
    ] {
        for parsed in parses(source, &export("grammar")) {
            let tree = parsed.expect(source);
            assert_eq!(tree, program(entry.clone()), "{source}");
            for result in evaluations(&tree) {
                assert_eq!(result.expect(source), value, "{source}");
            }
        }
    }
}

#[test]
fn child_errors_precede_unchanged_domain_and_range_checks() {
    for (source, expected) in [
        ("[1][-1:0]", Error::ListSliceRange),
        ("[1][1:0]", Error::ListSliceRange),
        ("[1][0:2]", Error::ListSliceRange),
        (
            "[1][-9223372036854775808:9223372036854775807]",
            Error::ListSliceRange,
        ),
        (r#""é🙂"[0:1]"#, Error::ListSliceOperands),
        ("[1][false:1]", Error::ListSliceOperands),
        ("[1][0:int]", Error::ListSliceOperands),
        (
            "(1 div 0)[9223372036854775807+1:1 div 0]",
            Error::DivisionByZero,
        ),
        ("0[9223372036854775807+1:1 div 0]", Error::Overflow),
        ("0[-1:1 div 0]", Error::DivisionByZero),
    ] {
        for parsed in parses(source, &export("grammar")) {
            for result in evaluations(&parsed.expect("syntax before semantics")) {
                assert_eq!(result.expect_err(source).to_string(), expected.to_string());
            }
        }
    }
    for source in ["(1 div 0)[0:1] trailing", "0[0:1 div 0] :", "[1][0:1]["] {
        for result in parses(source, &export("grammar")) {
            assert!(
                matches!(result, Err(parser::Error::Source { .. })),
                "{source}"
            );
        }
    }
}

#[test]
fn required_bounds_separators_and_conservative_newlines() {
    let grammar = export("grammar");
    // Missing endpoints retain the primary parser's existing missing-operand diagnostic.
    let [baseline, _] = selected("[1][", "pratt-no-slices", old::execute, 0);
    let baseline = baseline.expect_err("historical missing index");
    let parser::Error::Source { reason, .. } = baseline else {
        panic!("source error")
    };
    for (source, offset) in [("[1][:1]", 4), ("[1][0:]", 6), ("[1][0:", 6)] {
        for result in parses(source, &grammar) {
            source_error(result, offset, &reason);
        }
    }
    for source in [
        "[1][0:1:1]",
        "[1][0:1",
        "[1]\n[0:1]",
        "[1][0\n:1]",
        "[1][0:1\n]",
        "[1][0 // end\n:1]",
    ] {
        for result in parses(source, &grammar) {
            assert!(result.is_err(), "{source}");
        }
    }
    for result in parses("[1][ // lower\n0: // upper\n1]", &grammar) {
        assert_eq!(
            result.expect("leading endpoint trivia"),
            program(slice(array(json!([integer(1)])), integer(0), integer(1)))
        );
    }
}

#[test]
fn optional_metadata_is_eager_distinct_dependent_and_without_fallback() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let original = json!({"kind":"listSlice","operandField":"operand","lowerField":"lower","upperField":"upper"});
    assert_eq!(grammar["slice"], original);
    grammar["slice"] =
        json!({"kind":"customSlice","operandField":"base","lowerField":"start","upperField":"end"});
    for result in parses("1[2:3]", &grammar.to_string()) {
        assert_eq!(
            result.expect("custom constructor"),
            program(
                json!({"kind":"customSlice","base":integer(1),"start":integer(2),"end":integer(3)})
            )
        );
    }
    for (key, value) in [
        ("kind", ""),
        ("operandField", ""),
        ("lowerField", ""),
        ("upperField", ""),
        ("operandField", "kind"),
        ("lowerField", "kind"),
        ("upperField", "kind"),
        ("lowerField", "operand"),
        ("upperField", "operand"),
        ("upperField", "lower"),
    ] {
        grammar["slice"] = original.clone();
        grammar["slice"][key] = value.into();
        for result in parses("1", &grammar.to_string()) {
            assert!(matches!(
                result,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
    grammar.as_object_mut().expect("object").remove("slice");
    for result in parses("[1][0:1]", &grammar.to_string()) {
        assert!(result.is_err());
    }
    for result in parses("[1][0]", &grammar.to_string()) {
        assert_eq!(
            result.expect("ordinary index remains"),
            program(binary("index", array(json!([integer(1)])), integer(0)))
        );
    }
    grammar["slice"] = original;
    grammar.as_object_mut().expect("object").remove("index");
    for result in parses("1", &grammar.to_string()) {
        assert!(matches!(
            result,
            Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
        ));
    }
    grammar.as_object_mut().expect("object").remove("slice");
    for source in ["[1][0]", "[1][0:1]"] {
        for result in parses(source, &grammar.to_string()) {
            assert!(result.is_err());
        }
    }
}

#[test]
fn real_old_driver_preserves_ordinary_mixed_postfix_tree_byte_exactly() {
    let entry = select(
        binary("index", array(json!([field("x", integer(7))])), integer(0)),
        "x",
    );
    for result in selected(
        "[close({\"x\":7})][0].x",
        "pratt-no-slices",
        old::execute,
        0,
    ) {
        assert_eq!(
            result.expect("actual prior driver").to_string(),
            program(entry.clone()).to_string()
        );
    }
}

#[test]
fn upper_minimum_changes_only_upper_and_propagates_through_nested_frames() {
    for source in [
        "[1][0:0+1]",
        "([1][0:0+1])",
        "1+[1][0:0+1]",
        "[[1][0:0+1]]",
        "close({\"a\":[1][0:0+1]})",
        "len([1][0:0+1])",
        "[1][0:len([1][0:0+1])]",
    ] {
        assert!(
            parser::parse(source, &export("grammar")).is_ok(),
            "{source}"
        );
        for result in selected(source, "pratt-no-slices", old::execute, 0) {
            assert!(result.is_err(), "{source}");
        }
        for result in selected(source, "pratt-slice-high-minimum", high::execute, 0) {
            source_error(
                result,
                source.rfind('+').expect("upper operator"),
                "expected closing bracket",
            );
        }
    }
    for source in ["[1][0+0:1]", "[1][0:(0+1)]", "[1][0+0]"] {
        for result in selected(source, "pratt-slice-high-minimum", high::execute, 0) {
            assert_eq!(
                result.expect(source),
                parser::parse(source, &export("grammar")).expect("production")
            );
        }
    }
    // Endpoint recursion must preserve selected primary AND record engines too.
    for (source, driver, name, index, reason) in [
        (
            "[1][0:len(0+1)]",
            call_high::execute as parser::Driver,
            "primary-call-high-minimum",
            1,
            "expected closing parenthesis",
        ),
        (
            "[1][close({\"a\":0+1}):1]",
            record_high::execute as parser::Driver,
            "record-primary-high-minimum",
            2,
            "expected comma or closing brace",
        ),
    ] {
        for result in selected(source, name, driver, index) {
            source_error(result, source.find('+').expect("nested operator"), reason);
        }
    }
}

#[test]
fn slices_charge_one_node_plus_two_endpoint_asts_and_preserve_depth() {
    let grammar = export("grammar");
    // Initial leaf + three nodes per slice; endpoint recursion adds one level.
    for (source, error) in [
        (format!("1{}", "[0:0]".repeat(21)), None),
        (format!("1{}", "[0:0]".repeat(22)), Some("64 nodes")),
        (format!("1[0:{}0{}]", "(".repeat(31), ")".repeat(31)), None),
        (
            format!("1[0:{}0{}]", "(".repeat(32), ")".repeat(32)),
            Some("32 parsing levels"),
        ),
    ] {
        for result in parses(&source, &grammar) {
            if let Some(reason) = error {
                let message = result.expect_err("source budget").to_string();
                assert!(message.contains(reason), "{message}");
            } else {
                assert!(result.is_ok(), "{result:?}");
            }
        }
    }
}
