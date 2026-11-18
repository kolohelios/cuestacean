use std::path::Path;
use std::process::Command;

use bootstrap_executor::{Error, execute, execute_json, generated, parser};
use serde_json::{Value, json};

#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/primary-call-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary-no-calls.rs"]
mod no_calls;
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

fn text(value: &str) -> Value {
    json!({"kind":"string","value":value})
}

fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}

fn unary(kind: &str, operand: Value) -> Value {
    json!({"kind":kind,"operand":operand})
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

fn evaluations(tree: &Value) -> [Result<Value, Error>; 4] {
    let input = tree.to_string();
    let rules = export("evaluation");
    [
        execute_json(&input, &rules),
        generated::execute_json(&input),
        execute(&input, &rules).map(|value| json!(value)),
        generated::execute(&input).map(|value| json!(value)),
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
fn default_registration_and_independent_trees_count_decoded_utf8_bytes() {
    let grammar = export("grammar");
    let metadata: Value = serde_json::from_str(&grammar).expect("grammar");
    assert_eq!(
        metadata["calls"],
        json!([
            {"token":"len","kind":"length","operandField":"operand"}
        ])
    );
    for (source, decoded, bytes) in [
        (r#"len("")"#, "", 0),
        (r#"len("abc")"#, "abc", 3),
        (r#"len("é")"#, "é", 2),
        (r#"len("🙂")"#, "🙂", 4),
        (r#"len("e\u0301")"#, "e\u{301}", 3),
        (r#"len("\n")"#, "\n", 1),
        (r#"len("\u00e9\ud83d\ude42")"#, "é🙂", 6),
        (r#"len ("a")"#, "a", 1),
        ("len\t(\"a\")", "a", 1),
        (
            r#"len(// argument
"a")"#,
            "a",
            1,
        ),
    ] {
        let expected = program(unary("length", text(decoded)));
        for result in parses(source, &grammar) {
            let tree = result.expect(source);
            assert_eq!(tree, expected, "{source}");
            for evaluated in evaluations(&tree) {
                assert_eq!(evaluated.expect(source).to_string(), bytes.to_string());
            }
        }
    }
    for (source, entry, value) in [
        (
            r#"len("a"+"b")"#,
            unary("length", binary("add", text("a"), text("b"))),
            2,
        ),
        (
            r#"1+len("ab")*3"#,
            binary(
                "add",
                integer(1),
                binary("multiply", unary("length", text("ab")), integer(3)),
            ),
            7,
        ),
        (
            r#">=(len("abc"))&3"#,
            binary(
                "unify",
                unary("lowerBoundExpression", unary("length", text("abc"))),
                integer(3),
            ),
            3,
        ),
        (
            r#"<=(len("abc"))&3"#,
            binary(
                "unify",
                unary("upperBoundExpression", unary("length", text("abc"))),
                integer(3),
            ),
            3,
        ),
    ] {
        for result in parses(source, &grammar) {
            let tree = result.expect(source);
            assert_eq!(tree, program(entry.clone()), "{source}");
            for evaluated in evaluations(&tree) {
                assert_eq!(evaluated.expect(source).to_string(), value.to_string());
            }
        }
    }
}

#[test]
fn domain_and_child_errors_precede_output_and_enclosing_length_checks() {
    let grammar = export("grammar");
    for source in ["len(3)", "len(string)", r#"len(len("x"))"#] {
        for parsed in parses(source, &grammar) {
            for result in evaluations(&parsed.expect(source)) {
                assert!(
                    matches!(result, Err(Error::LengthOperand)),
                    "{source}: {result:?}"
                );
            }
        }
    }
    for parsed in parses("len(1 div 0)", &grammar) {
        for result in evaluations(&parsed.expect("valid call tree")) {
            assert!(matches!(result, Err(Error::DivisionByZero)), "{result:?}");
        }
    }
}

#[test]
fn exact_call_syntax_diagnostics() {
    let grammar = export("grammar");
    for (source, offset, reason) in [
        ("len", 3, "expected opening parenthesis"),
        ("len1", 0, "unknown atom"),
        ("len()", 4, "expected signed decimal integer"),
        (r#"len("a","b")"#, 7, "expected closing parenthesis"),
        ("len\n(\"a\")", 4, "newline after call name"),
        ("len\r(\"a\")", 4, "newline after call name"),
        ("len\r\n(\"a\")", 5, "newline after call name"),
        ("len // comment\n(\"a\")", 15, "newline after call name"),
        (r#"len("a""#, 7, "expected closing parenthesis"),
        (r#"len(("a")"#, 9, "expected closing parenthesis"),
    ] {
        for result in parses(source, &grammar) {
            source_error(result, offset, reason);
        }
    }
}

#[test]
fn runtime_alias_constructor_and_registration_are_only_metadata() {
    let original: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let mut grammar = original.clone();
    grammar["calls"] = json!([
        {"token":"Byte_count9","kind":"customCall","operandField":"child"}
    ]);
    for parsed in parses(r#"Byte_count9("é")"#, &grammar.to_string()) {
        assert_eq!(
            parsed.expect("alias"),
            program(json!({"kind":"customCall","child":text("é")}))
        );
    }
    for parsed in parses(r#"len("é")"#, &grammar.to_string()) {
        source_error(parsed, 0, "unknown atom");
    }
    let mut multiple = original.clone();
    multiple["calls"]
        .as_array_mut()
        .expect("calls")
        .push(json!({"token":"negative","kind":"negate","operandField":"operand"}));
    for parsed in parses(r#"negative(len("abc"))"#, &multiple.to_string()) {
        let tree = parsed.expect("two independent registrations");
        assert_eq!(tree, program(unary("negate", unary("length", text("abc")))));
        for result in evaluations(&tree) {
            assert_eq!(result.expect("constructor selects semantics"), json!(-3));
        }
    }
    for disabled in [None, Some(json!([]))] {
        let mut grammar = original.clone();
        if let Some(calls) = disabled {
            grammar["calls"] = calls;
        } else {
            grammar
                .as_object_mut()
                .expect("grammar object")
                .remove("calls");
        }
        for parsed in parses(r#"len("x")"#, &grammar.to_string()) {
            source_error(parsed, 0, "unknown atom");
        }
        for parsed in parses("1+(2*3)", &grammar.to_string()) {
            assert_eq!(
                parsed.expect("old source"),
                program(binary(
                    "add",
                    integer(1),
                    binary("multiply", integer(2), integer(3))
                ))
            );
        }
    }
}

#[test]
fn all_call_metadata_validates_eagerly_including_unused_entries() {
    let original: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let valid = json!({"token":"unused","kind":"custom","operandField":"child"});
    let atom = original["atoms"][0]["token"].clone();
    let mut invalid = vec![
        json!([valid.clone(), valid]),
        json!([{"token":atom,"kind":"custom","operandField":"child"}]),
        json!([{"token":"unused","kind":"","operandField":"child"}]),
        json!([{"token":"unused","kind":"custom","operandField":""}]),
        json!([{"token":"unused","kind":"custom","operandField":"kind"}]),
        json!([{"token":"unused","kind":"custom"}]),
        json!([{"token":"unused","kind":"custom","operandField":"child","extra":true}]),
    ];
    for token in ["", "9len", "len-x", "len x", "é", "len🙂"] {
        invalid.push(json!([{"token":token,"kind":"custom","operandField":"child"}]));
    }
    for calls in invalid {
        let mut grammar = original.clone();
        grammar["calls"] = calls;
        for result in parses("1", &grammar.to_string()) {
            assert!(
                matches!(
                    result,
                    Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
                ),
                "{grammar}: {result:?}"
            );
        }
    }
}

#[test]
fn call_wrappers_share_original_depth_and_node_limits() {
    let grammar = export("grammar");
    let deep = |n| format!("{}\"x\"{}", "len(".repeat(n), ")".repeat(n));
    let sum = |n| vec![r#"len("x")"#; n].join("+");
    // 21 leaves + 21 call wrappers + 20 additions = 62; two more wrappers = 64.
    for source in [deep(32), sum(21), format!("len(len({}))", sum(21))] {
        for result in parses(&source, &grammar) {
            assert!(
                result.is_ok(),
                "parse-only resource boundary {source}: {result:?}"
            );
        }
    }
    for (source, reason) in [
        (deep(33), "expression exceeds 32 parsing levels"),
        (sum(22), "expression exceeds 64 nodes"),
    ] {
        for result in parses(&source, &grammar) {
            match result.expect_err("resource guard") {
                parser::Error::Source { reason: actual, .. } => assert_eq!(actual, reason),
                other => panic!("unexpected resource error {other:?}"),
            }
        }
    }
}

#[test]
fn actual_old_primary_has_no_native_call_fallback_and_preserves_old_sources() {
    let grammar = export("grammar");
    let old = export("tests/valid/primary-no-calls");
    for source in [r#"len("x")"#, r#"(len("x"))"#, r#"1+len("x")"#] {
        let offset = if source.starts_with('(') {
            1
        } else if source.starts_with('1') {
            2
        } else {
            0
        };
        for result in [
            parser::parse_with_drivers(source, &grammar, expression::execute, no_calls::execute),
            parser::parse_interpreted_with_primary(source, &grammar, &export("pratt"), &old),
        ] {
            source_error(result, offset, "unknown atom");
        }
    }
    for source in ["1+(2*3)", "!false", ">=(1+2)&3", r#""a"+"b""#] {
        let expected = parser::parse(source, &grammar).expect("production");
        for result in [
            parser::parse_with_drivers(source, &grammar, expression::execute, no_calls::execute),
            parser::parse_interpreted_with_primary(source, &grammar, &export("pratt"), &old),
        ] {
            assert_eq!(result.expect(source), expected);
        }
    }
}

#[test]
fn actual_call_argument_minimum_propagates_without_changing_ordinary_groups() {
    let grammar = export("grammar");
    let alternate = export("tests/valid/primary-call-high-minimum");
    for (source, offset) in [
        (r#"len("a"+"b")"#, 7),
        (r#"len(len("a"+"b"))"#, 11),
        (r#"1+len("a"+"b")"#, 9),
        (r#">=(len("a"+"b"))&2"#, 10),
    ] {
        assert!(parser::parse(source, &grammar).is_ok(), "{source}");
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
    for source in [r#"len(("a"+"b"))"#, "(1+2)", r#"len("a")+1"#] {
        let expected = parser::parse(source, &grammar).expect("production");
        for result in [
            parser::parse_with_drivers(
                source,
                &grammar,
                expression::execute,
                high_minimum::execute,
            ),
            parser::parse_interpreted_with_primary(source, &grammar, &export("pratt"), &alternate),
        ] {
            assert_eq!(result.expect(source), expected);
        }
    }
}

#[test]
fn selected_expression_driver_reaches_arguments_on_both_paths() {
    let grammar = export("grammar");
    for source in [
        r#"len("a"+"b"+"c")"#,
        r#"(len("a"+"b"+"c"))"#,
        r#"1+len("a"+"b"+"c")"#,
    ] {
        let call = unary(
            "length",
            binary("add", text("a"), binary("add", text("b"), text("c"))),
        );
        let expected = program(if source.starts_with('1') {
            binary("add", integer(1), call)
        } else {
            call
        });
        for result in [
            parser::parse_with_drivers(
                source,
                &grammar,
                right_expression::execute,
                production_primary::execute,
            ),
            parser::parse_interpreted_with_primary(
                source,
                &grammar,
                &export("tests/valid/pratt-right"),
                &export("primary"),
            ),
        ] {
            let tree = result.expect(source);
            assert_eq!(tree, expected);
            for evaluated in evaluations(&tree) {
                assert_eq!(
                    evaluated.expect(source),
                    json!(if source.starts_with('1') { 4 } else { 3 })
                );
            }
        }
    }
}

#[test]
fn cli_failures_are_atomic() {
    let path =
        std::env::temp_dir().join(format!("cuestacean-unary-calls-{}.cue", std::process::id()));
    for (source, message) in [
        (
            "len(3)",
            "length requires a concrete string or closed collection",
        ),
        ("len(1 div 0)", "integer division by zero"),
        ("len()", "expected signed decimal integer"),
    ] {
        std::fs::write(&path, source).expect("write source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert!(
            String::from_utf8(output.stderr)
                .expect("UTF-8 diagnostic")
                .contains(message)
        );
    }
    std::fs::remove_file(path).expect("remove source");
}
