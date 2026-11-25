//! Execute #300's genuine native-selection/compiler outputs, not high goldens.
use bootstrap_executor::parser;
use serde_json::{Value, json};
use std::path::Path;

// The build stages this test at tests/primary_control_minima_runtime.rs and
// stages the proof's runtime files alongside it, without regenerating them.
#[path = "base.rs"]
mod base;
#[path = "bound.rs"]
mod bound;
#[path = "call.rs"]
mod call;
#[path = "../../generated/conjunction-primary.rs"]
mod conjunction;
#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "list.rs"]
mod list;
#[path = "../../generated/record-primary.rs"]
mod record;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh bundle")
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Row {
    Base,
    Bound,
    Call,
    List,
}
fn selected(source: &str, row: Row) -> [Result<Value, parser::Error>; 2] {
    let (driver, program): (parser::Driver, &str) = match row {
        Row::Base => (base::execute, include_str!("base.json")),
        Row::Bound => (bound::execute, include_str!("bound.json")),
        Row::Call => (call::execute, include_str!("call.json")),
        Row::List => (list::execute, include_str!("list.json")),
    };
    let grammar = export("grammar");
    [
        parser::parse_with_four_drivers(
            source,
            &grammar,
            expression::execute,
            driver,
            record::execute,
            conjunction::execute,
        ),
        parser::parse_interpreted_with_four_drivers(
            source,
            &grammar,
            &export("pratt"),
            program,
            &export("record-primary"),
            &export("conjunction-primary"),
        ),
    ]
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
fn sum() -> Value {
    binary("add", integer(1), integer(2))
}
fn string_sum() -> Value {
    binary(
        "add",
        json!({"kind":"string","value":"a"}),
        json!({"kind":"string","value":"b"}),
    )
}
fn array(child: Value) -> Value {
    json!({"kind":"listExpression","elements":[child]})
}
fn record(child: Value) -> Value {
    json!({"kind":"recordExpression","fields":[{"name":"a","value":child}]})
}
fn conjunction(child: Value) -> Value {
    json!({"kind":"conjunction","operands":[child]})
}
fn succeeds(source: &str, row: Row, entry: Value) {
    for result in selected(source, row) {
        assert_eq!(
            result.expect(source),
            json!({"formatVersion":1,"entry":entry}),
            "{row:?}: {source}"
        );
    }
}
fn rejects(source: &str, row: Row, offset: usize, reason: &str) {
    for result in selected(source, row) {
        let error = result.expect_err("selected minimum must affect callback");
        assert!(
            matches!(&error, parser::Error::Source { offset: actual, reason: message }
                if *actual == offset && message == reason),
            "{row:?}: {source}: {error:?}"
        );
    }
}
#[test]
fn fresh_primary_matrix_has_independent_callback_effects_and_recursive_selection() {
    // Restoration reuses the baseline artifacts, whose separately regenerated
    // restored row is required byte-identical by the source/consumer proof.
    for row in [Row::Base, Row::Bound, Row::Call, Row::List, Row::Base] {
        for (affected, source, entry, offset, reason) in [
            (
                Row::Bound,
                ">=(1+2)&3",
                binary("unify", unary("lowerBoundExpression", sum()), integer(3)),
                4,
                "expected closing parenthesis",
            ),
            (
                Row::Call,
                "len(\"a\"+\"b\")",
                unary("length", string_sum()),
                7,
                "expected closing parenthesis",
            ),
            (
                Row::List,
                "[1+2]",
                array(sum()),
                2,
                "expected comma or closing bracket",
            ),
        ] {
            // Wrappers protect their own child admission with a group, but
            // MUST propagate the selected primary into that child. Each full
            // expected tree is hand-authored, not obtained from another parser.
            let contexts = [
                ("", "", entry.clone()),
                ("(", ")", entry.clone()),
                ("0+(", ")", binary("add", integer(0), entry.clone())),
                ("[(", ")]", array(entry.clone())),
                ("close({\"a\":(", ")})", record(entry.clone())),
                ("and([(", ")])", conjunction(entry.clone())),
                ("-(", ")", unary("negate", entry)),
            ];
            for (prefix, suffix, expected) in contexts {
                let nested = format!("{prefix}{source}{suffix}");
                if row == affected {
                    rejects(&nested, row, prefix.len() + offset, reason);
                } else {
                    succeeds(&nested, row, expected);
                }
            }
        }
        // Grouping resets the child expression's own minimum. These continue
        // to succeed in ALL rows, including nested sibling consumer sites.
        for (source, expected) in [
            ("(1+2)", sum()),
            (
                ">=((1+2))&3",
                binary("unify", unary("lowerBoundExpression", sum()), integer(3)),
            ),
            ("len((\"a\"+\"b\"))", unary("length", string_sum())),
            ("[(1+2)]", array(sum())),
            ("[len((\"a\"+\"b\"))]", array(unary("length", string_sum()))),
            ("len([(1+2)])", unary("length", array(sum()))),
            ("close({\"a\":[(1+2)]})", record(array(sum()))),
            (
                "and([len((\"a\"+\"b\"))])",
                conjunction(unary("length", string_sum())),
            ),
            (">=3", json!({"kind":"lowerBound","value":3})),
            (
                "len(\"a\")",
                unary("length", json!({"kind":"string","value":"a"})),
            ),
            ("[1]", array(integer(1))),
        ] {
            succeeds(source, row, expected);
        }
    }
}
