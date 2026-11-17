use std::{fs, path::Path};

use bootstrap_executor::scanner::{self, Error, Scan};
use serde_json::{Value, json};

#[path = "../../generated/scanner-repeat.rs"]
mod compiled;
#[path = "../../generated/repeat-fuel.rs"]
mod fuel;
#[path = "../../generated/repeat-fuel-over.rs"]
mod fuel_over;
#[path = "../../generated/string-literal.rs"]
mod repeated;
#[path = "../../generated/repeat-scopes.rs"]
mod scopes;
#[path = "../../generated/string-unrolled.rs"]
mod unrolled;
#[path = "../../generated/repeat-zero.rs"]
mod zero;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("exported scanner")
}

fn program(body: Value) -> String {
    json!({"formatVersion":1,"body":body}).to_string()
}

fn repeat(count: usize, body: Value) -> Value {
    json!({"op":"repeat","count":count,"body":body})
}

fn flag() -> Value {
    json!({"op":"setFlag","value":true})
}

fn flagged(offset: usize) -> Scan {
    Scan { offset, flag: true }
}

fn test(predicate: Value) -> Value {
    json!({"op":"if","condition":predicate,"then":[]})
}

fn tree(levels: usize) -> Value {
    if levels == 1 {
        json!({"op":"hasByte"})
    } else {
        json!({"op":"and","left":tree(levels - 1),"right":tree(levels - 1)})
    }
}

fn invalid(input: &str, diagnostic: &str) {
    let ignoring = r#"{"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}"#;
    for result in [
        scanner::interpret(input, "", 0).map(|_| ()),
        scanner::generate(input).map(|_| ()),
        scanner::generate_with_policy(input, ignoring).map(|_| ()),
        scanner::generate_with_policy(input, "{").map(|_| ()),
    ] {
        let error = result
            .expect_err("original scanner must reject")
            .to_string();
        assert!(
            error.contains(diagnostic),
            "{diagnostic:?}: {error}; {input}"
        );
    }
}

#[test]
fn compiled_repeat_has_fixed_scalar_eof_ascii_and_offset_oracles() {
    let control = export("tests/valid/scanner-repeat");
    for (source, start, end) in [
        ("", 0, 0),
        ("a", 0, 1),
        ("ab", 0, 2),
        ("abcd", 0, 3),
        ("λ", 0, 2),
        ("λé🦀x", 0, 8),
        ("éaλ🦀x", 2, 9),
        ("é", 2, 2),
    ] {
        for result in [
            compiled::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert_eq!(result.expect("bounded scalar scan"), flagged(end));
        }
    }
    for byte in 0u8..=127 {
        let source = format!("é{}λ🦀x", char::from(byte));
        for result in [
            compiled::scan(&source, 2),
            scanner::interpret(&control, &source, 2),
        ] {
            assert_eq!(result.expect("ASCII then Unicode"), flagged(9));
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
    let generated = scanner::generate(&control).expect("repeat backend");
    assert!(generated.contains("for _ in 0..3"), "{generated}");
    assert_eq!(
        generated,
        scanner::generate_with_policy(&control, &export("scan-lowering"))
            .expect("reference backend")
    );
}

#[test]
fn compiled_nested_break_and_zero_once_have_fixed_results() {
    let scoped = export("tests/valid/repeat-scopes");
    let empty = export("tests/valid/repeat-zero");
    for (source, start, end) in [("abcde", 0, 4), ("λé🦀xy", 0, 9), ("éabcd!", 2, 6)] {
        for result in [
            scopes::scan(source, start),
            scanner::interpret(&scoped, source, start),
        ] {
            assert_eq!(
                result.expect("inner break leaves outer repeat alive"),
                flagged(end)
            );
        }
    }
    for source in ["", "a", "abc", "λé🦀"] {
        for result in [
            scopes::scan(source, 0),
            scanner::interpret(&scoped, source, 0),
        ] {
            assert!(matches!(result, Err(Error::AdvanceChar)), "{source:?}");
        }
    }
    for (source, start) in [("", 0), ("λ", 0), ("🦀", 0), ("éx", 2), ("é", 2)] {
        for result in [
            zero::scan(source, start),
            scanner::interpret(&empty, source, start),
        ] {
            assert_eq!(
                result.expect("zero skips unsafe ASCII advance; once sets flag"),
                flagged(start)
            );
        }
    }
    for control in [&scoped, &empty] {
        let generated = scanner::generate(control).expect("once lowering");
        assert!(
            !generated.contains("#[allow"),
            "no lint suppression: {generated}"
        );
        assert_eq!(
            generated,
            scanner::generate_with_policy(control, &export("scan-lowering"))
                .expect("reference once lowering")
        );
    }
}

#[test]
fn nearest_while_and_repeat_breaks_do_not_escape_or_prune_the_enclosing_loop() {
    for inner in [
        repeat(64, json!([{"op":"advanceChar"},{"op":"break"}])),
        json!({"op":"while","condition":{"op":"hasByte"},
            "body":[{"op":"advanceChar"},{"op":"break"}]}),
    ] {
        for outer_repeat in [false, true] {
            let body = json!([inner, {"op":"advanceChar"}]);
            let outer = if outer_repeat {
                repeat(2, body)
            } else {
                json!({"op":"while","condition":{"op":"hasByte"},"body":body})
            };
            let input = program(json!([outer, flag()]));
            assert_eq!(
                scanner::interpret(&input, "λé🦀x", 0).expect("nearest loop break"),
                flagged(9)
            );
            let generated = scanner::generate(&input).expect("mixed loop scopes");
            assert_eq!(
                generated,
                scanner::generate_with_policy(&input, &export("scan-lowering"))
                    .expect("reference mixed loop scopes")
            );
        }
    }
    // A break inside a once repeat is consumed, not an always-break fact for while.
    let input = program(json!([{"op":"while","condition":{"op":"hasByte"},
        "body":[repeat(1, json!([{"op":"break"}])), {"op":"advanceChar"}]}, flag()]));
    assert_eq!(
        scanner::interpret(&input, "λé", 0).expect("outer suffix reachable"),
        flagged(4)
    );
    scanner::generate(&input).expect("reachable suffix compiles");
}

#[test]
fn actual_compiled_fuel_fixture_uses_exactly_256_steps() {
    let control = export("tests/valid/repeat-fuel");
    // 127 + 127 + zero's entry + final flag action = exactly 256 steps.
    for (source, start) in [("a", 0), ("λx", 2)] {
        for result in [
            fuel::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert_eq!(
                result.expect("exact one-byte budget"),
                Scan {
                    offset: start,
                    flag: false
                }
            );
        }
    }
    for (source, start) in [("", 0), ("a", 1)] {
        for result in [
            fuel::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert!(
                matches!(result, Err(Error::Budget)),
                "{source:?} at {start}"
            );
        }
    }
}

#[test]
fn actual_compiled_one_extra_action_exceeds_the_same_budget() {
    let control = export("tests/valid/repeat-fuel-over");
    for (source, start) in [("a", 0), ("λx", 2)] {
        for result in [
            fuel_over::scan(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert!(matches!(result, Err(Error::Budget)), "257 steps at {start}");
        }
    }
}

#[test]
fn exact_entry_iteration_and_body_fuel_has_no_final_test_charge() {
    // At EOF the allowance is 128. 1 entry + 63 * (1 iteration + 1 action) = 127.
    for (suffix, succeeds) in [(0, true), (1, true), (2, false)] {
        let mut body = vec![repeat(63, json!([flag()]))];
        body.extend((0..suffix).map(|_| flag()));
        let input = program(json!(body));
        let result = scanner::interpret(&input, "é", 2);
        if succeeds {
            assert_eq!(result.expect("at most 128 steps"), flagged(2));
        } else {
            assert!(matches!(result, Err(Error::Budget)));
        }
    }
    let input = program(json!([repeat(64, json!([flag()]))]));
    assert!(matches!(
        scanner::interpret(&input, "", 0),
        Err(Error::Budget)
    )); // 129
    assert_eq!(
        scanner::interpret(&input, "x", 0).expect("256 steps available"),
        Scan {
            offset: 0,
            flag: true
        }
    );
    // Zero costs exactly its entry step, even with a nonempty unsafe body.
    for zeros in [1, 2] {
        let mut body = vec![repeat(63, json!([flag()]))];
        body.extend((0..zeros).map(|_| repeat(0, json!([{"op":"advance"}]))));
        let result = scanner::interpret(&program(json!(body)), "", 0);
        assert_eq!(matches!(result, Err(Error::Budget)), zeros == 2);
    }
    // A breaking iteration costs entry + iteration + break = 3, not 129.
    let input = program(json!([repeat(64, json!([{"op":"break"}])), flag()]));
    assert_eq!(
        scanner::interpret(&input, "", 0).expect("break skips later iterations"),
        Scan {
            offset: 0,
            flag: true
        }
    );
}

#[test]
fn counts_and_unused_zero_bodies_validate_before_policy_or_flow_pruning() {
    for count in [0, 1, 64] {
        scanner::interpret(&program(json!([repeat(count, json!([]))])), "", 0)
            .expect("bounded count");
    }
    invalid(
        &program(json!([repeat(65, json!([]))])),
        "repeat count exceeds 64",
    );
    for statement in [
        r#"{"op":"repeat","count":-1,"body":[]}"#,
        r#"{"op":"repeat","count":1.5,"body":[]}"#,
        r#"{"op":"repeat","body":[]}"#,
        r#"{"op":"repeat","count":0}"#,
        r#"{"op":"repeat","count":0,"body":[],"extra":0}"#,
        r#"{"op":"repeat","count":0,"count":1,"body":[]}"#,
        r#"{"op":"repeat","count":0,"body":[],"body":[]}"#,
    ] {
        invalid(
            &format!(r#"{{"formatVersion":1,"body":[{statement}]}}"#),
            "invalid scanner JSON",
        );
    }
    for (bad, diagnostic) in [
        (json!({"op":"unknownStatement"}), "unknown variant"),
        (json!({"op":"advance","extra":0}), "unknown field"),
        (json!({"op":"setFlag"}), "missing field"),
        (
            test(json!({"op":"byteIn","bytes":[]})),
            "1..=128 ASCII bytes",
        ),
        (
            test(json!({"op":"byteIn","bytes":[97],"ahead":65})),
            "lookahead exceeds 64",
        ),
        (
            test(json!({"op":"charIn","characters":[]})),
            "1..=128 characters",
        ),
    ] {
        invalid(
            &program(json!([repeat(0, json!([bad.clone()]))])),
            diagnostic,
        );
        invalid(
            &program(json!([repeat(1, json!([{"op":"break"}, bad]))])),
            diagnostic,
        );
    }
    invalid(
        &program(json!([repeat(0, json!([])), {"op":"break"}])),
        "break requires an enclosing loop",
    );
}

#[test]
fn zero_and_many_iterations_count_the_body_once_with_shared_structural_caps() {
    for count in [0, 64] {
        let actions = vec![flag(); 63]; // Repeat plus body = exactly 64 statements.
        let input = program(json!([repeat(count, json!(actions))]));
        scanner::interpret(&input, &"x".repeat(64), 0).expect("body counted once");
        scanner::generate(&input).expect("64 static statements compile");
        invalid(
            &program(json!([repeat(count, json!(vec![flag(); 64]))])),
            "exceeds 64 statements",
        );
    }
    let mut body = json!([flag()]);
    for _ in 0..16 {
        body = json!([repeat(0, body)]);
    }
    scanner::interpret(&program(body.clone()), "", 0).expect("16 nesting levels");
    invalid(
        &program(json!([repeat(0, body)])),
        "exceeds 16 nesting levels",
    );
    let large = test(tree(7)); // 127 predicate nodes, plus an external leaf = 128.
    let input = program(json!([repeat(0, json!([large.clone()])), test(tree(1))]));
    scanner::interpret(&input, "", 0).expect("shared 128 predicate nodes");
    scanner::generate(&input).expect("zero body still prepares valid predicates");
    invalid(
        &program(json!([
            repeat(0, json!([large])),
            test(tree(1)),
            test(tree(1))
        ])),
        "exceeds 128 predicate nodes",
    );
    let mut predicate = tree(1);
    for _ in 1..16 {
        predicate = json!({"op":"or","left":tree(1),"right":predicate});
    }
    scanner::interpret(
        &program(json!([repeat(0, json!([test(predicate.clone())]))])),
        "",
        0,
    )
    .expect("16 predicate levels inside zero");
    predicate = json!({"op":"or","left":tree(1),"right":predicate});
    invalid(
        &program(json!([repeat(0, json!([test(predicate)]))])),
        "exceeds 16 predicate levels",
    );
}

#[test]
fn actual_unrolled_and_repeat_unicode_scanners_keep_fixed_spans_flags_and_errors() {
    let current = export("string-literal");
    let baseline = export("tests/valid/string-unrolled");
    for (source, start, end, flag) in [
        ("", 0, 0, false),
        ("x", 0, 0, false),
        (r#"""tail"#, 0, 2, true),
        (r#""é🦀"tail"#, 0, 8, true),
        (r#""\u0041"tail"#, 0, 8, true),
        (r#""\u0000\uD7FF\uE000\uFFFF"tail"#, 0, 26, true),
        (r#""\uD800\uDC00"tail"#, 0, 14, true),
        (r#""\udbff\uDfFf"tail"#, 0, 14, true),
        ("λ\"é\\uD83E\\uDD80\\u0000\"tail", 2, 24, true),
    ] {
        for result in [
            repeated::scan(source, start),
            unrolled::scan(source, start),
            scanner::interpret(&current, source, start),
            scanner::interpret(&baseline, source, start),
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
        (r#""\uDC00""#, 0, 3, "unexpected low surrogate escape"),
        (r#""\uD800""#, 0, 7, "missing low surrogate escape"),
        (r#""\uD800\u0041""#, 0, 9, "expected low surrogate escape"),
        (r#""\uD800\uDCλ0""#, 0, 11, "invalid Unicode escape"),
        (r#""\U00010000""#, 0, 2, "unsupported string escape"),
        ("λ\"é\\u00λ0\"", 2, 9, "invalid Unicode escape"),
    ] {
        for result in [
            repeated::scan(source, start),
            unrolled::scan(source, start),
            scanner::interpret(&current, source, start),
            scanner::interpret(&baseline, source, start),
        ] {
            assert_eq!(
                result.expect_err("preserved rejection").to_string(),
                format!("source byte {offset}: {reason}"),
                "{source:?}"
            );
        }
    }
}
