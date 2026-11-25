use std::path::Path;

use bootstrap_executor::{Error, execute_json, generated, parser};
use serde_json::{Value, json};

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
fn program(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}
fn record(fields: Vec<(&str, Value)>) -> Value {
    json!({"kind":"recordExpression","fields":fields.into_iter()
        .map(|(name,value)| json!({"name":name,"value":value})).collect::<Vec<_>>()})
}
fn parses(source: &str) -> [Result<Value, parser::Error>; 2] {
    let grammar = export("grammar");
    [
        parser::parse_file(source, &grammar),
        parser::parse_file_interpreted(source, &grammar),
    ]
}
fn evaluations(tree: &Value) -> [Result<Value, Error>; 2] {
    [
        execute_json(&tree.to_string(), &export("evaluation")),
        generated::execute_json(&tree.to_string()),
    ]
}
fn check(source: &str, entry: Value, value: Value) {
    for parsed in parses(source) {
        let tree = parsed.expect(source);
        assert_eq!(tree, program(entry.clone()), "{source:?}");
        for result in evaluations(&tree) {
            assert_eq!(result.expect(source), value, "{source:?}");
        }
    }
}

#[test]
fn handwritten_trees_values_and_eof() {
    for source in [
        "answer:40+2",
        "answer \t:40+2\n",
        "\n//λ\r\nanswer:40+2 //tail",
        "answer:40+2\r\n\t//tail\r",
    ] {
        check(
            source,
            record(vec![("answer", binary("add", integer(40), integer(2)))]),
            json!({"answer":42}),
        );
    }
    check(
        "true:true\nfalse:false\nnull:null\nint:7\nif:\"yes\"\nA_1:[1,2]",
        record(vec![
            ("true", json!({"kind":"true"})),
            ("false", json!({"kind":"false"})),
            ("null", json!({"kind":"null"})),
            ("int", integer(7)),
            ("if", json!({"kind":"string","value":"yes"})),
            (
                "A_1",
                json!({"kind":"listExpression","elements":[integer(1),integer(2)]}),
            ),
        ]),
        json!({"true":true,"false":false,"null":null,"int":7,"if":"yes","A_1":[1,2]}),
    );
    check(
        "n:-9223372036854775808\r\nmax:9223372036854775807",
        record(vec![("n", integer(i64::MIN)), ("max", integer(i64::MAX))]),
        json!({"n":i64::MIN,"max":i64::MAX}),
    );
}

#[test]
fn old_expression_trees_and_exact_errors_are_conserved() {
    let grammar = export("grammar");
    for source in [
        "1+2*3",
        "true",
        "null",
        "int & 7",
        "[1,2][0]",
        "close({\"x\":1})",
        "len(\"é\")",
        "//λ\n(1+2)",
    ] {
        let expected = parser::parse(source, &grammar).expect(source);
        for parsed in parses(source) {
            assert_eq!(parsed.expect(source), expected, "{source:?}");
        }
    }
    // In particular, a failed positive declaration probe must not disturb the
    // original expression cursor or erase the old conservative rejections.
    for source in [
        "",
        "//only",
        "unknown",
        "1 +",
        "1\n+2",
        "1 div\n2",
        "close({a:1})",
        "(1\n+2)",
        "[1\n,2]",
        "close({\"a\":1\n})",
        "1 //λ\0\n",
    ] {
        let expected = parser::parse(source, &grammar)
            .expect_err(source)
            .to_string();
        for parsed in parses(source) {
            assert_eq!(
                parsed.expect_err(source).to_string(),
                expected,
                "{source:?}"
            );
        }
    }
    for (source, offset, reason) in [
        ("unknown", 0, "unknown atom"),
        ("1\n+2", 2, "line break after operand is unsupported"),
        ("1 //λ\0\n", 6, "unsupported character in line comment"),
    ] {
        for parsed in parses(source) {
            assert!(
                matches!(parsed, Err(parser::Error::Source { offset: actual, reason: ref why })
                if actual == offset && why == reason),
                "{source:?}: {parsed:?}"
            );
        }
    }
}

#[test]
fn operator_labels_preserve_boundary_through_recursive_rhs() {
    for (rhs, entry, value) in [
        (
            "1+2*3",
            binary(
                "add",
                integer(1),
                binary("multiply", integer(2), integer(3)),
            ),
            json!(7),
        ),
        (
            "-(2)",
            json!({"kind":"negate","operand":integer(2)}),
            json!(-2),
        ),
        (
            "!false",
            json!({"kind":"logicalNot","operand":{"kind":"false"}}),
            json!(true),
        ),
        (
            "1+2+3",
            binary("add", binary("add", integer(1), integer(2)), integer(3)),
            json!(6),
        ),
    ] {
        for label in ["div", "mod", "quo", "rem"] {
            for separator in ["\n", " //λ\r\n\n\t"] {
                let source = format!("a:{rhs}{separator}{label}:9");
                check(
                    &source,
                    record(vec![("a", entry.clone()), (label, integer(9))]),
                    json!({"a":value.clone(),label:9}),
                );
            }
        }
    }
}

#[test]
fn boundary_scope_is_suspended_in_nested_syntax() {
    for (rhs, entry, value) in [
        ("(1+2)", binary("add", integer(1), integer(2)), json!(3)),
        (
            "len([\n1,2])",
            json!({"kind":"length","operand":{"kind":"listExpression","elements":[integer(1),integer(2)]}}),
            json!(2),
        ),
        (
            "[1,2][\n1]",
            binary(
                "index",
                json!({"kind":"listExpression","elements":[integer(1),integer(2)]}),
                integer(1),
            ),
            json!(2),
        ),
        (
            "and([\nint,7])",
            json!({"kind":"conjunction","operands":[{"kind":"intType"},integer(7)]}),
            json!(7),
        ),
    ] {
        check(
            &format!("a:{rhs}\nmod:4"),
            record(vec![("a", entry), ("mod", integer(4))]),
            json!({"a":value,"mod":4}),
        );
    }
    for rhs in [
        "(1\n+2)",
        "len(1\n+2)",
        ">=(1\n+2)",
        "[1\n,2]",
        "and([1\n,2])",
        "close({\"x\":1\n})",
        "[1][0\n]",
        "[1,2][0\n:1]",
        "1 div\n2",
    ] {
        for parsed in parses(&format!("a:{rhs}\nb:3")) {
            assert!(parsed.is_err(), "nested newline admitted: {rhs:?}");
        }
    }
}

#[test]
fn exclusions_separators_comments_and_no_retry() {
    for source in [
        "package p\na:1",
        "import \"strings\"\na:1",
        "package:1",
        "import:1",
        "_hidden:1",
        "#Definition:1",
        "\"quoted\":1",
        "[\"x\"]:1",
        "café:1",
        "a?:1",
        "a!:1",
        "a\n:1",
        "a //comment\n:1",
        "a:1,b:2",
        "a:1;b:2",
        "a:1 b:2",
        "a:1\rb:2",
        "a:",
        "a:1\nb:",
        "a:1\n42",
        "a:1\n+2",
        "a:{b:1}",
        "a:(b:1)",
        "a:[b:1]",
        "a:b",
        "a:1\npackage p",
        "a:1\nimport \"strings\"",
        "a:1\n\"b\":2",
        "a:1\n//bad\0",
        "a:1\n//bad\u{feff}",
    ] {
        for parsed in parses(source) {
            assert!(parsed.is_err(), "unexpected acceptance/retry: {source:?}");
        }
    }
    for parsed in parses("a:1 //λ\0\r\n") {
        assert!(
            matches!(parsed, Err(parser::Error::Source { offset: 8, ref reason })
            if reason == "unsupported character in line comment"),
            "{parsed:?}"
        );
    }
    // A lone CR does not end a comment or expose a declaration.
    check(
        "a:1 //tail\rb:2",
        record(vec![("a", integer(1))]),
        json!({"a":1}),
    );
    for source in ["a:1\na:2", "a:1\na:1 div 0", "a:1\na", "a:1\na:"] {
        for parsed in parses(source) {
            assert!(
                matches!(parsed, Err(parser::Error::Source { ref reason, .. })
                if reason == "duplicate file field"),
                "{source:?}: {parsed:?}"
            );
        }
    }
    for (source, reason) in [
        ("a:1\n42", "expected file label"),
        ("a:1\nb", "expected file colon"),
        ("a:1 b:2", "expected LF between file fields"),
    ] {
        for parsed in parses(source) {
            assert!(
                matches!(parsed, Err(parser::Error::Source { reason: ref actual, .. })
                if actual == reason),
                "{source:?}: {parsed:?}"
            );
        }
    }
    for source in ["answer:42\nsibling:int", "answer:42\nsibling:1 & 2"] {
        for parsed in parses(source) {
            for result in evaluations(&parsed.expect("whole file syntax")) {
                assert!(result.is_err(), "unselected sibling ignored: {source}");
            }
        }
    }
}

#[test]
fn one_shared_parser_budget_including_record_node() {
    for (count, succeeds) in [(63, true), (64, false)] {
        let source = (0..count)
            .map(|i| format!("f{i}:1"))
            .collect::<Vec<_>>()
            .join("\n");
        for parsed in parses(&source) {
            if succeeds {
                assert!(parsed.is_ok(), "{parsed:?}");
            } else {
                assert!(
                    parsed
                        .expect_err("shared nodes")
                        .to_string()
                        .contains("expression exceeds 64 nodes")
                );
            }
        }
    }
    // Each half alone fits; a fresh per-field parser would incorrectly pass.
    let rhs = std::iter::repeat_n("1", 17).collect::<Vec<_>>().join("+");
    for parsed in parses(&format!("a:{rhs}\nb:{rhs}")) {
        assert!(
            parsed
                .expect_err("cumulative nodes")
                .to_string()
                .contains("expression exceeds 64 nodes")
        );
    }
    for parsed in parses(&format!("a:{}1{}", "(".repeat(33), ")".repeat(33))) {
        assert!(
            parsed
                .expect_err("shared depth")
                .to_string()
                .contains("expression exceeds 32 parsing levels")
        );
    }
    for (length, succeeds) in [(64, true), (65, false)] {
        for parsed in parses(&format!("{}:1", "a".repeat(length))) {
            match parsed {
                Err(_) => assert!(!succeeds),
                Ok(tree) => {
                    for result in evaluations(&tree) {
                        assert_eq!(result.is_ok(), succeeds, "label bytes: {length}");
                    }
                }
            }
        }
    }
}
