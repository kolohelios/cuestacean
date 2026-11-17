use std::{fs, path::Path, process::Command};

use bootstrap_executor::scanner::{self, Error, Scan};
use serde_json::{Value, json};

#[path = "../../generated/scanner-predicates.rs"]
mod compiled;
#[path = "../../generated/string-literal.rs"]
mod composed;
#[path = "../../generated/string-nested.rs"]
mod nested;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("exported control")
}

fn program(body: Value) -> String {
    json!({"formatVersion":1,"body":body}).to_string()
}

fn condition(predicate: Value) -> Value {
    json!({"op":"if","condition":predicate,"then":[]})
}

fn binary(op: &str, left: Value, right: Value) -> Value {
    json!({"op":op,"left":left,"right":right})
}

fn leaf(value: bool) -> Value {
    // The initial flag is false; these are input-independent truth values.
    json!({"op":"flagIs","value":!value})
}

fn full_tree(levels: usize) -> Value {
    if levels == 1 {
        leaf(true)
    } else {
        binary("and", full_tree(levels - 1), full_tree(levels - 1))
    }
}

fn chain(levels: usize) -> Value {
    (1..levels).fold(leaf(true), |right, _| binary("or", leaf(false), right))
}

fn invalid(input: &str, diagnostic: &str) {
    let ignoring = r#"{"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}"#;
    for result in [
        scanner::interpret(input, "", 0).map(|_| ()),
        scanner::generate(input).map(|_| ()),
        scanner::generate_with_policy(input, ignoring).map(|_| ()),
        // The original typed program must reject before even parsing this policy.
        scanner::generate_with_policy(input, "{").map(|_| ()),
    ] {
        let error = result.expect_err("invalid original scanner").to_string();
        assert!(
            error.contains(diagnostic),
            "{diagnostic:?}: {error}; {input}"
        );
    }
}

#[test]
fn compiled_fixture_has_fixed_ascii_unicode_eof_and_offset_oracles() {
    let control = export("tests/valid/scanner-predicates");
    for byte in 0u8..=127 {
        let text = char::from(byte).to_string();
        for prefix in ["", "é"] {
            let source = format!("{prefix}{text}tail");
            let start = prefix.len();
            let expected = Scan {
                offset: start + usize::from(byte == b'a'),
                flag: true,
            };
            for result in [
                compiled::scan(&source, start),
                scanner::interpret(&control, &source, start),
            ] {
                assert_eq!(result.expect("ASCII predicate"), expected, "{source:?}");
            }
        }
    }
    for (source, start, offset, flag) in [
        ("", 0, 0, true),
        ("a", 0, 1, true),
        ("λ", 0, 2, true),
        ("λλ", 0, 2, true),
        ("λxy", 0, 2, true),
        ("é", 0, 0, true),
        ("🦀", 0, 0, true),
        ("x", 0, 0, true),
        ("xy", 0, 0, false),
        ("xz", 0, 0, true),
        ("y", 0, 0, true),
        ("éλtail", 2, 4, true),
        ("éxy", 2, 2, false),
        ("é", 2, 2, true),
    ] {
        let expected = Scan { offset, flag };
        for result in [
            compiled::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert_eq!(
                result.expect("fixed predicate case"),
                expected,
                "{source:?}"
            );
        }
    }
    for (source, start) in [("", 1), ("λ", 1), ("λ", 3), ("🦀", 2)] {
        for result in [
            compiled::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert!(matches!(result, Err(Error::Offset)));
        }
    }
    // Secondary compiler evidence, not a substitute for invoking compiled::scan.
    let backend = export("scan-lowering");
    let source = scanner::generate(&control).expect("compiled backend");
    assert_eq!(
        source,
        scanner::generate_with_policy(&control, &backend).expect("reference backend")
    );
    assert!(source.contains("&&") && source.contains("||"));
}

#[test]
fn normal_and_inverted_nested_truth_tables_have_independent_boolean_oracles() {
    let backend = export("scan-lowering");
    for outer in ["and", "or"] {
        for inner in ["and", "or"] {
            for a in [false, true] {
                for b in [false, true] {
                    for c in [false, true] {
                        let rhs = if inner == "and" { b && c } else { b || c };
                        let truth = if outer == "and" { a && rhs } else { a || rhs };
                        for inverted in [false, true] {
                            let predicate = binary(outer, leaf(a), binary(inner, leaf(b), leaf(c)));
                            let body = if inverted {
                                json!([{"op":"if","condition":predicate,"then":[],
                                    "else":[{"op":"setFlag","value":true}]}])
                            } else {
                                json!([{"op":"if","condition":predicate,
                                    "then":[{"op":"setFlag","value":true}],
                                    "else":[{"op":"setFlag","value":false}]}])
                            };
                            let input = program(body);
                            assert_eq!(
                                scanner::interpret(&input, "", 0).expect("truth table"),
                                Scan {
                                    offset: 0,
                                    flag: truth != inverted
                                }
                            );
                            let source = scanner::generate(&input).expect("nested lowering");
                            assert_eq!(
                                source,
                                scanner::generate_with_policy(&input, &backend)
                                    .expect("reference nested lowering")
                            );
                            let operator = if (outer == "and") != inverted {
                                "&&"
                            } else {
                                "||"
                            };
                            let inner_operator = if (inner == "and") != inverted {
                                "&&"
                            } else {
                                "||"
                            };
                            let read = |value| {
                                if value != inverted {
                                    "!cursor.flag()"
                                } else {
                                    "cursor.flag()"
                                }
                            };
                            let expected = format!(
                                "if ({}) {operator} (({}) {inner_operator} ({})) {{",
                                read(a),
                                read(b),
                                read(c)
                            );
                            assert!(
                                source.contains(&expected),
                                "expected {expected:?}: {source}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn predicate_node_and_depth_limits_are_separate_exact_whole_program_bounds() {
    // Seven full binary levels = 127 nodes. A second root leaf makes exactly 128.
    let input = program(json!([condition(full_tree(7)), condition(leaf(true))]));
    assert_eq!(
        scanner::interpret(&input, "", 0).expect("128 predicate nodes"),
        Scan {
            offset: 0,
            flag: false
        }
    );
    scanner::generate(&input).expect("128 nodes compile");
    let input = program(json!([condition(binary("and", leaf(true), full_tree(7)))]));
    invalid(&input, "exceeds 128 predicate nodes");

    for levels in [16, 17] {
        let input = program(json!([condition(chain(levels))]));
        if levels == 16 {
            scanner::interpret(&input, "", 0).expect("16 predicate levels");
            scanner::generate(&input).expect("16 levels compile");
        } else {
            invalid(&input, "exceeds 16 predicate levels");
        }
    }
    // Counts cannot reset between if/while, branches, or after an unreachable break.
    for body in [
        json!([condition(full_tree(7)), {"op":"while","condition":leaf(false),
            "body":[condition(leaf(true))]}]),
        json!([{"op":"if","condition":leaf(false),"then":[condition(full_tree(7))],
            "else":[condition(leaf(true))]}]),
        json!([{"op":"while","condition":leaf(false),"body":[{"op":"break"},
            condition(full_tree(7)), condition(leaf(true))]}]),
    ] {
        invalid(&program(body), "exceeds 128 predicate nodes");
    }
}

#[test]
fn pure_predicate_nodes_do_not_consume_runtime_statement_fuel() {
    // 64 statements, 127 predicate nodes, 128 available steps at EOF. Charging
    // predicates individually would exhaust fuel even though this must succeed.
    let actions: Vec<_> = (0..63)
        .map(|_| json!({"op":"setFlag","value":true}))
        .collect();
    let input = program(json!([{"op":"if","condition":full_tree(7),"then":actions}]));
    assert_eq!(
        scanner::interpret(&input, "", 0).expect("one test step"),
        Scan {
            offset: 0,
            flag: true
        }
    );
    let input = program(json!([{"op":"while","condition":full_tree(7),
        "body":[{"op":"setFlag","value":true}]}]));
    assert_eq!(
        scanner::interpret(&input, "", 0).expect("two loop tests"),
        Scan {
            offset: 0,
            flag: true
        }
    );
    let stalled = program(json!([{"op":"while","condition":chain(16),"body":[]}]));
    assert!(matches!(
        scanner::interpret(&stalled, "", 0),
        Err(Error::Budget)
    ));
}

#[test]
fn unused_children_and_unreachable_statements_still_validate_eagerly() {
    for (bad, diagnostic) in [
        (json!({"op":"byteIn","bytes":[]}), "1..=128 ASCII bytes"),
        (json!({"op":"byteIn","bytes":[128]}), "1..=128 ASCII bytes"),
        (
            json!({"op":"byteIn","bytes":vec![97;129]}),
            "1..=128 ASCII bytes",
        ),
        (
            json!({"op":"byteIn","bytes":[97],"ahead":65}),
            "lookahead exceeds 64",
        ),
        (json!({"op":"charIn","characters":[]}), "1..=128 characters"),
        (
            json!({"op":"charIn","characters":vec!["λ";129]}),
            "1..=128 characters",
        ),
        (
            json!({"op":"charIn","characters":["ab"]}),
            "invalid scanner JSON",
        ),
        (
            json!({"op":"charIn","characters":[""]}),
            "invalid scanner JSON",
        ),
        (json!({"op":"and","left":leaf(true)}), "missing field"),
        (json!({"op":"or","right":leaf(true)}), "missing field"),
        (
            json!({"op":"unknownPredicate","value":leaf(true)}),
            "unknown variant",
        ),
        (json!({"op":"hasByte","extra":true}), "unknown field"),
        (
            json!({"op":"and","left":leaf(true),"right":leaf(false),"extra":0}),
            "unknown field",
        ),
    ] {
        for op in ["and", "or"] {
            for bad_on_left in [false, true] {
                let good = leaf(op == "or"); // RHS is unreachable when good is first.
                let predicate = if bad_on_left {
                    binary(op, bad.clone(), good)
                } else {
                    binary(op, good, bad.clone())
                };
                invalid(&program(json!([condition(predicate.clone())])), diagnostic);
                let unreachable = program(json!([{"op":"while","condition":leaf(false),
                    "body":[{"op":"break"},condition(predicate)]}]));
                invalid(&unreachable, diagnostic);
            }
        }
    }
    // Deserialize the original text: a Value round trip would hide duplicate keys.
    for predicate in [
        r#"{"op":"and","left":{"op":"hasByte"},"left":{"op":"hasByte"},"right":{"op":"hasByte"}}"#,
        r#"{"op":"or","left":{"op":"hasByte"},"right":{"op":"hasByte"},"right":{"op":"hasByte"}}"#,
        r#"{"op":"and","left":{"op":"byteIn","bytes":[97],"bytes":[98]},"right":{"op":"hasByte"}}"#,
        r#"{"op":"or","left":{"op":"hasByte"},"right":{"op":"flagIs","value":true,"value":false}}"#,
    ] {
        let input = format!(
            r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{predicate},"then":[]}}]}}"#
        );
        invalid(&input, "duplicate field");
    }
}

#[test]
fn old_statement_list_and_lookahead_limits_remain_independent() {
    let actions: Vec<_> = (0..64)
        .map(|_| json!({"op":"setFlag","value":true}))
        .collect();
    let input = program(json!(actions));
    scanner::interpret(&input, "", 0).expect("64 statements");
    let old_leaves: Vec<_> = (0..64).map(|_| condition(leaf(true))).collect();
    scanner::interpret(&program(json!(old_leaves)), "", 0).expect("64 old leaf predicates");
    let mut too_many = actions;
    too_many.push(json!({"op":"setFlag","value":true}));
    invalid(&program(json!(too_many)), "exceeds 64 statements");
    let mut body = json!([{"op":"setFlag","value":true}]);
    for _ in 0..16 {
        body = json!([{"op":"if","condition":leaf(true),"then":body}]);
    }
    scanner::interpret(&program(body.clone()), "", 0).expect("16 statement levels");
    body = json!([{"op":"if","condition":leaf(true),"then":body}]);
    invalid(&program(body), "exceeds 16 nesting levels");
    let input = program(json!([{"op":"if","condition":binary("and",
        json!({"op":"byteIn","bytes":vec![97;128],"ahead":64}),
        json!({"op":"charIn","characters":vec!["a";128]})),
        "then":[{"op":"setFlag","value":true}]}]));
    assert_eq!(
        scanner::interpret(&input, &"a".repeat(65), 0).expect("leaf boundaries"),
        Scan {
            offset: 0,
            flag: true
        }
    );
    assert_eq!(
        scanner::interpret(&input, "a", 0).expect("past EOF is false"),
        Scan {
            offset: 0,
            flag: false
        }
    );
}

#[test]
fn compact_cli_rejects_unused_invalid_predicates_without_partial_output() {
    // No coordinator-owned invalid export is required.
    let temporary = std::env::temp_dir().join(format!(
        "cuestacean-predicate-invalid-{}.json",
        std::process::id()
    ));
    fs::write(
        &temporary,
        program(json!([condition(binary(
            "or",
            leaf(true),
            json!({"op":"byteIn","bytes":[]})
        ))])),
    )
    .expect("temporary invalid program");
    let result = Command::new(env!("CARGO_BIN_EXE_bootstrap-scan-codegen"))
        .arg(&temporary)
        .output();
    fs::remove_file(&temporary).expect("remove temporary program");
    let output = result.expect("scanner compiler CLI");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 diagnostic")
            .contains("1..=128 ASCII bytes")
    );
}

#[test]
fn actual_old_nested_and_composed_unicode_scanners_preserve_fixed_results() {
    let current = export("string-literal");
    let old = export("tests/valid/string-nested");
    // Expected byte offsets are literal independent oracles, not derived from
    // either scanner. Fuel counts may differ; accepted spans and errors must not.
    for (source, start, end, flag) in [
        ("", 0, 0, false),
        ("x", 0, 0, false),
        (r#"""tail"#, 0, 2, true),
        (r#""raw"tail"#, 0, 5, true),
        (r#""é🦀"tail"#, 0, 8, true),
        (r#""\n\t\r\b\f\"\\"tail"#, 0, 16, true),
        (r#""\u0041"tail"#, 0, 8, true),
        (r#""\u0000\uD7FF\uE000\uFFFF"tail"#, 0, 26, true),
        (r#""\uD800\uDC00"tail"#, 0, 14, true),
        (r#""\udbff\uDfFf"tail"#, 0, 14, true),
        (r#""\uD83E\uDD80\u0041"tail"#, 0, 20, true),
        ("λ\"é\\uD83E\\uDD80\\u0000\"tail", 2, 24, true),
    ] {
        for result in [
            nested::scan(source, start),
            composed::scan(source, start),
            scanner::interpret(&old, source, start),
            scanner::interpret(&current, source, start),
        ] {
            assert_eq!(
                result.expect("preserved span"),
                Scan { offset: end, flag },
                "{source:?}"
            );
        }
    }
    for (source, start, offset, reason) in [
        ("\"", 0, 1, "unterminated string literal"),
        ("\"\\u", 0, 3, "invalid Unicode escape"),
        (r#""\u000""#, 0, 6, "invalid Unicode escape"),
        (r#""\uG000""#, 0, 3, "invalid Unicode escape"),
        (r#""\u00λ0""#, 0, 5, "invalid Unicode escape"),
        (r#""\uDC00""#, 0, 3, "unexpected low surrogate escape"),
        (r#""\udfff""#, 0, 3, "unexpected low surrogate escape"),
        (r#""\uD800""#, 0, 7, "missing low surrogate escape"),
        ("\"\\uDBFF", 0, 7, "missing low surrogate escape"),
        (
            r#""\uD800\U0000DC00""#,
            0,
            7,
            "missing low surrogate escape",
        ),
        (r#""\uD800\u0041""#, 0, 9, "expected low surrogate escape"),
        (r#""\uD800\uD800""#, 0, 9, "expected low surrogate escape"),
        ("\"\\uD800\\u", 0, 9, "expected low surrogate escape"),
        (r#""\uD800\uDCλ0""#, 0, 11, "invalid Unicode escape"),
        (r#""\uD800\uDFFg""#, 0, 12, "invalid Unicode escape"),
        (r#""\q""#, 0, 2, "unsupported string escape"),
        ("λ\"é\\u00λ0\"", 2, 9, "invalid Unicode escape"),
        ("λ\"\\uD800\\uDC00", 2, 15, "unterminated string literal"),
    ] {
        for result in [
            nested::scan(source, start),
            composed::scan(source, start),
            scanner::interpret(&old, source, start),
            scanner::interpret(&current, source, start),
        ] {
            assert_eq!(
                result.expect_err("preserved rejection").to_string(),
                format!("source byte {offset}: {reason}"),
                "{source:?}"
            );
        }
    }
    for (source, start) in [("λé", 1), ("λé", 3), ("", 1)] {
        for result in [
            nested::scan(source, start),
            composed::scan(source, start),
            scanner::interpret(&old, source, start),
            scanner::interpret(&current, source, start),
        ] {
            assert!(matches!(result, Err(Error::Offset)));
        }
    }
}
