//! Execute #298's freshly emitted source-mutation matrix, never high goldens.
use bootstrap_executor::parser;
use serde_json::{Value, json};
use std::path::Path;

#[path = "conjunction-100.rs"]
mod conjunction_hundred;
#[path = "conjunction-0.rs"]
mod conjunction_zero;
#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "record-100.rs"]
mod record_hundred;
#[path = "record-0.rs"]
mod record_zero;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh bundle")
}
fn selected(source: &str, record: bool, conjunction: bool) -> [Result<Value, parser::Error>; 2] {
    let record_driver = if record {
        record_hundred::execute
    } else {
        record_zero::execute
    };
    let conjunction_driver = if conjunction {
        conjunction_hundred::execute
    } else {
        conjunction_zero::execute
    };
    let record_json = if record {
        include_str!("record-100.json")
    } else {
        include_str!("record-0.json")
    };
    let conjunction_json = if conjunction {
        include_str!("conjunction-100.json")
    } else {
        include_str!("conjunction-0.json")
    };
    let grammar = export("grammar");
    [
        parser::parse_with_four_drivers(
            source,
            &grammar,
            expression::execute,
            primary::execute,
            record_driver,
            conjunction_driver,
        ),
        parser::parse_interpreted_with_four_drivers(
            source,
            &grammar,
            &export("pratt"),
            &export("primary"),
            record_json,
            conjunction_json,
        ),
    ]
}
fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn sum() -> Value {
    json!({"kind":"add","left":integer(1),"right":integer(2)})
}
fn record(child: Value) -> Value {
    json!({"kind":"recordExpression","fields":[{"name":"a","value":child}]})
}
fn conjunction(child: Value) -> Value {
    json!({"kind":"conjunction","operands":[child]})
}
fn succeeds(source: &str, r: bool, c: bool, entry: Value) {
    for result in selected(source, r, c) {
        assert_eq!(
            result.expect(source),
            json!({"formatVersion":1,"entry":entry})
        );
    }
}
fn rejects(source: &str, r: bool, c: bool, reason: &str) {
    for result in selected(source, r, c) {
        let error = result.expect_err("mutated minimum must affect parsing");
        assert!(
            matches!(&error, parser::Error::Source { offset, reason: actual }
            if *offset == source.find('+').expect("operator") && actual == reason),
            "{source}: {error:?}"
        );
    }
}
#[test]
fn fresh_mutation_matrix_executes_with_sibling_isolation_and_nested_propagation() {
    // Baseline, each independently changed producer, and restoration.
    for (r, c) in [(false, false), (true, false), (false, true), (false, false)] {
        if r {
            for source in [
                "close({\"a\":1+2})",
                "(close({\"a\":1+2}))",
                "[close({\"a\":1+2})]",
                "and([close({\"a\":1+2})])",
            ] {
                rejects(source, r, c, "expected comma or closing brace");
            }
        } else {
            succeeds("close({\"a\":1+2})", r, c, record(sum()));
        }
        if c {
            for source in [
                "and([1+2])",
                "(and([1+2]))",
                "[and([1+2])]",
                "close({\"a\":and([1+2])})",
            ] {
                rejects(source, r, c, "expected comma or closing bracket");
            }
        } else {
            succeeds("and([1+2])", r, c, conjunction(sum()));
        }
        // Parenthesized child expressions reset their own expression minimum.
        succeeds("close({\"a\":(1+2)})", r, c, record(sum()));
        succeeds("and([(1+2)])", r, c, conjunction(sum()));
        succeeds(
            "and([close({\"a\":(1+2)})])",
            r,
            c,
            conjunction(record(sum())),
        );
        succeeds(
            "close({\"a\":and([(1+2)])})",
            r,
            c,
            record(conjunction(sum())),
        );
    }
}
