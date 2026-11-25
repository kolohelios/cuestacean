//! #303: execute the proof's fresh pairs staged after normal regeneration.
use bootstrap_executor::parser;
use serde_json::{Value, json};
use std::path::Path;

const ROW: &str = env!("CUESTACEAN_PRATT_MINIMA_ROW");
fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh staged bundle")
}
fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
}
fn unary(kind: &str, operand: Value) -> Value {
    json!({"kind":kind,"operand":operand})
}
fn array(elements: Vec<Value>) -> Value {
    json!({"kind":"listExpression","elements":elements})
}
fn record(fields: Vec<(&str, Value)>) -> Value {
    json!({"kind":"recordExpression","fields":fields.into_iter()
        .map(|(name,value)| json!({"name":name,"value":value})).collect::<Vec<_>>()})
}
fn slice(operand: Value, lower: Value, upper: Value) -> Value {
    json!({"kind":"listSlice","operand":operand,"lower":lower,"upper":upper})
}
fn select(operand: Value, label: &str) -> Value {
    json!({"kind":"select","operand":operand,"label":label})
}
fn conjunction(operand: Value) -> Value {
    json!({"kind":"conjunction","operands":[operand]})
}
fn parses(source: &str, file: bool) -> [Result<Value, parser::Error>; 2] {
    let grammar = export("grammar");
    if file {
        [
            parser::parse_file(source, &grammar),
            parser::parse_file_interpreted(source, &grammar),
        ]
    } else {
        [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &export("pratt")),
        ]
    }
}
fn succeeds(source: &str, file: bool, entry: Value) {
    for result in parses(source, file) {
        assert_eq!(
            result.expect(source),
            json!({"formatVersion":1,"entry":entry}),
            "{ROW}, file={file}: {source:?}"
        );
    }
}
fn rejects(source: &str, file: bool, offset: usize, expected: &str) {
    for result in parses(source, file) {
        let error = result.expect_err("mutated callback must change parsing");
        assert!(
            matches!(&error, parser::Error::Source { offset: actual, reason }
                if *actual == offset && reason == expected),
            "{ROW}, file={file}: {source:?}: {error:?}"
        );
    }
}
#[test]
fn independent_minima_affect_index_and_slice_lower_versus_only_upper() {
    assert!(matches!(ROW, "base" | "index" | "slice"));
    let items = array(vec![integer(1), integer(2)]);
    for (affected, source, expected) in [
        (
            "index",
            "[7][0+0]",
            binary(
                "index",
                array(vec![integer(7)]),
                binary("add", integer(0), integer(0)),
            ),
        ),
        (
            "index",
            "[1,2][0+0:1]",
            slice(
                items.clone(),
                binary("add", integer(0), integer(0)),
                integer(1),
            ),
        ),
        (
            "slice",
            "[1,2][0:1+0]",
            slice(
                items.clone(),
                integer(0),
                binary("add", integer(1), integer(0)),
            ),
        ),
    ] {
        let operator = source.find('+').expect("computed endpoint");
        // Complete trees, independently specified; no parser acts as the oracle.
        for (prefix, suffix, entry) in [
            ("", "", expected.clone()),
            ("(", ")", expected.clone()),
            ("0+(", ")", binary("add", integer(0), expected.clone())),
            ("[(", ")]", array(vec![expected.clone()])),
            (
                "close({\"x\":(",
                ")})",
                record(vec![("x", expected.clone())]),
            ),
            ("and([(", ")])", conjunction(expected.clone())),
            ("-(", ")", unary("negate", expected.clone())),
            ("len([(", ")])", unary("length", array(vec![expected]))),
        ] {
            let nested = format!("{prefix}{source}{suffix}");
            for file in [false, true] {
                if ROW == affected {
                    rejects(
                        &nested,
                        file,
                        prefix.len() + operator,
                        "expected closing bracket",
                    );
                } else {
                    succeeds(&nested, file, entry.clone());
                }
            }
            // Positive file recognition, fresh file-Pratt recursion, and an
            // operator-named sibling after the completed recursive expression.
            let declared = format!("a:{nested}\r\nmod:9 //tail");
            if ROW == affected {
                rejects(
                    &declared,
                    true,
                    2 + prefix.len() + operator,
                    "expected closing bracket",
                );
            } else {
                succeeds(
                    &declared,
                    true,
                    record(vec![("a", entry), ("mod", integer(9))]),
                );
            }
        }
    }
}
#[test]
fn grouped_recovery_chains_and_unchanged_primary_record_conjunction_siblings() {
    let sum = binary("add", integer(1), integer(2));
    let items = array(vec![integer(1), integer(2)]);
    let range = slice(
        items.clone(),
        binary("add", integer(0), integer(0)),
        binary("add", integer(1), integer(0)),
    );
    let chain = select(
        binary(
            "index",
            slice(
                array(vec![record(vec![("x", integer(7))])]),
                integer(0),
                integer(1),
            ),
            integer(0),
        ),
        "x",
    );
    for (source, expected) in [
        ("[1,2][(0+0):(1+0)]", range),
        (
            "[7][(0+0)]",
            binary(
                "index",
                array(vec![integer(7)]),
                binary("add", integer(0), integer(0)),
            ),
        ),
        ("[close({\"x\":7})][0:1][0].x", chain),
        ("(1+2)", sum.clone()),
        ("[1+2]", array(vec![sum.clone()])),
        ("close({\"x\":1+2})", record(vec![("x", sum.clone())])),
        ("and([1+2])", conjunction(sum.clone())),
        (
            ">=(1+2)&3",
            binary("unify", unary("lowerBoundExpression", sum), integer(3)),
        ),
        (
            "len(\"a\"+\"b\")",
            unary(
                "length",
                binary(
                    "add",
                    json!({"kind":"string","value":"a"}),
                    json!({"kind":"string","value":"b"}),
                ),
            ),
        ),
        ("[1,2][0:1]", slice(items, integer(0), integer(1))),
    ] {
        for file in [false, true] {
            succeeds(source, file, expected.clone());
        }
        succeeds(
            &format!("a:{source}\ndiv:9"),
            true,
            record(vec![("a", expected), ("div", integer(9))]),
        );
    }
}
#[test]
fn file_boundaries_suspend_in_endpoints_and_do_not_leak_through_recursive_rhs() {
    let range = slice(
        array(vec![integer(1), integer(2)]),
        binary("add", integer(0), integer(0)),
        binary("add", integer(1), integer(0)),
    );
    for (rhs, entry) in [
        (
            "1+[1,2][\n(0+0):\n(1+0)]",
            binary("add", integer(1), range.clone()),
        ),
        ("-([1,2][\n(0+0):\n(1+0)])", unary("negate", range.clone())),
        ("len([1,2][\n(0+0):\n(1+0)])", unary("length", range)),
    ] {
        for label in ["div", "mod", "quo", "rem"] {
            for separator in ["\n", " //λ\r\n\n\t"] {
                succeeds(
                    &format!("a:{rhs}{separator}{label}:9"),
                    true,
                    record(vec![("a", entry.clone()), (label, integer(9))]),
                );
            }
        }
    }
    // Once declarations are recognized, errors are not retried as expressions.
    // Even the unmodified sibling forms must retain their exact diagnostics.
    for source in ["a:[7][0\n]\nb:2", "a:[1,2][0\n:1]\nb:2"] {
        rejects(
            source,
            true,
            source.find('\n').expect("newline") + 1,
            "line break after operand is unsupported",
        );
    }
    rejects("a:1\nb:", true, 6, "expected signed decimal integer");
}
