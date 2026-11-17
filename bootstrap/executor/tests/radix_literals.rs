use std::path::Path;

use bootstrap_executor::{execute_json, generated, parser, scanner};
use scanner::Scan;
use serde_json::{Value, json};

// Fixed spellings and values: no runtime radix conversion supplies the oracle.
const LITERALS: &[(&str, i64)] = &[
    ("0b101010", 42),
    ("0o52", 42),
    ("0x2a", 42),
    ("0X2A", 42),
    ("-0b101010", -42),
    ("-0o52", -42),
    ("-0X2a", -42),
    (
        "0b111111111111111111111111111111111111111111111111111111111111111",
        i64::MAX,
    ),
    (
        "-0b1000000000000000000000000000000000000000000000000000000000000000",
        i64::MIN,
    ),
    ("0o777777777777777777777", i64::MAX),
    ("-0o1000000000000000000000", i64::MIN),
    ("0x7FFFFFFFFFFFFFFF", i64::MAX),
    ("-0X8000000000000000", i64::MIN),
    ("9223372036854775807", i64::MAX),
    ("-9223372036854775808", i64::MIN),
    ("-0b0", 0),
    ("-0o0", 0),
    ("-0x0", 0),
    ("0x00fF", 255),
];

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn parsed(source: &str) -> [Result<Value, parser::Error>; 2] {
    let grammar = export("grammar");
    [
        parser::parse(source, &grammar),
        parser::parse_interpreted(source, &grammar, &export("pratt")),
    ]
}

fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
}

fn check(source: &str, entry: Value, expected: Result<i64, &str>) {
    for result in parsed(source) {
        let ast = result.expect("source tree");
        assert_eq!(ast, json!({"formatVersion":1,"entry":entry}), "{source}");
        let program = ast.to_string();
        for result in [
            execute_json(&program, &export("evaluation")),
            generated::execute_json(&program),
        ] {
            match expected {
                Ok(value) => assert_eq!(result.expect("JSON integer"), json!(value), "{source}"),
                Err(message) => assert_eq!(
                    result.expect_err("evaluation rejection").to_string(),
                    message,
                    "{source}"
                ),
            }
        }
    }
}

#[test]
fn fixed_i64_literals_have_complete_ast_and_exact_json_oracles() {
    for &(source, expected) in LITERALS {
        check(source, integer(expected), Ok(expected));
    }
}

#[test]
fn fixed_digit_runs_stop_before_nonmembers_with_sign_and_utf8_offsets() {
    let control = export("integer-literal");
    for (source, start, end, minus) in [
        ("0b101010", 0, 8, false),
        ("0o52", 0, 4, false),
        ("0X2a", 0, 4, false),
        ("-0x2A", 0, 5, true),
        ("0x00fF!", 0, 6, false),
        ("0b102", 0, 4, false),
        ("0o78", 0, 3, false),
        ("0x2g", 0, 3, false),
        ("0b1_0", 0, 3, false),
        ("0B10", 0, 1, false),
        ("0O52", 0, 1, false),
        ("-0B10", 0, 2, true),
        ("-0O52", 0, 2, true),
        ("λ-0b101010tail", 2, 11, true),
        ("λ0X2a🦀", 2, 6, false),
        ("0x2add1", 0, 7, false),
    ] {
        for result in [
            scanner::integer_literal(source, start),
            scanner::interpret(&control, source, start),
        ] {
            assert_eq!(
                result.expect("selected span"),
                Scan {
                    offset: end,
                    flag: minus
                },
                "{source}"
            );
        }
    }
    assert!(matches!(
        scanner::integer_literal("λ0x2a", 1),
        Err(scanner::Error::Offset)
    ));
    for source in [
        "0b102", "0o78", "0x2g", "0b1_0", "0B10", "0O52", "-0B10", "-0O52", "0x2a_0", "0x2aλ",
        "+0x2a", "00", "012", "０x2a", "0x2add1z",
    ] {
        for result in parsed(source) {
            assert!(result.is_err(), "whole-input rejection: {source}");
        }
    }
}

#[test]
fn missing_or_wrong_first_digits_have_exact_post_prefix_diagnostics() {
    let control = export("integer-literal");
    for (prefix, reason, bad) in [
        ("0x", "expected hexadecimal integer digit", "g"),
        ("0X", "expected hexadecimal integer digit", "g"),
        ("0b", "expected binary integer digit", "8"),
        ("0o", "expected octal integer digit", "8"),
    ] {
        for sign in ["", "-"] {
            for suffix in ["", bad, "_", "λ", "１", "+", "-"] {
                let source = format!("{sign}{prefix}{suffix}");
                let offset = 2 + sign.len();
                let expected = format!("source byte {offset}: {reason}");
                for result in [
                    scanner::integer_literal(&source, 0),
                    scanner::interpret(&control, &source, 0),
                ] {
                    assert_eq!(result.expect_err("digit required").to_string(), expected);
                }
                for result in parsed(&source) {
                    assert_eq!(
                        result.expect_err("source digit required").to_string(),
                        expected
                    );
                }
            }
        }
    }
    for result in [
        scanner::integer_literal("λ-0o8", 2),
        scanner::interpret(&control, "λ-0o8", 2),
    ] {
        assert_eq!(
            result.expect_err("UTF-8 error offset").to_string(),
            "source byte 5: expected octal integer digit"
        );
    }
}

#[test]
fn actual_old_compiled_decimal_and_reference_remain_independent_baselines() {
    let old = export("decimal");
    let new = export("integer-literal");
    // Same source, independently fixed old and new spans, not parity alone.
    for result in [
        scanner::decimal("0x2a", 0),
        scanner::interpret(&old, "0x2a", 0),
    ] {
        assert_eq!(
            result.expect("old decimal span"),
            Scan {
                offset: 1,
                flag: false
            }
        );
    }
    for result in [
        scanner::integer_literal("0x2a", 0),
        scanner::interpret(&new, "0x2a", 0),
    ] {
        assert_eq!(
            result.expect("new integer span"),
            Scan {
                offset: 4,
                flag: false
            }
        );
    }
    for (source, start, end, flag) in [
        ("0", 0, 1, false),
        ("-0", 0, 2, true),
        ("42tail", 0, 2, false),
        ("λ-42!", 2, 5, true),
        ("9223372036854775808", 0, 19, false),
    ] {
        for result in [
            scanner::decimal(source, start),
            scanner::interpret(&old, source, start),
            scanner::integer_literal(source, start),
            scanner::interpret(&new, source, start),
        ] {
            assert_eq!(
                result.expect("unchanged decimal"),
                Scan { offset: end, flag }
            );
        }
    }
    for (source, offset, reason) in [
        ("-", 1, "expected signed decimal integer"),
        ("+1", 0, "expected signed decimal integer"),
        ("１", 0, "expected signed decimal integer"),
        ("00", 2, "leading zeroes are unsupported"),
        ("-0123", 5, "leading zeroes are unsupported"),
    ] {
        for result in [
            scanner::decimal(source, 0),
            scanner::interpret(&old, source, 0),
            scanner::integer_literal(source, 0),
            scanner::interpret(&new, source, 0),
        ] {
            assert_eq!(
                result.expect_err("unchanged diagnostic").to_string(),
                format!("source byte {offset}: {reason}")
            );
        }
    }
}

#[test]
fn concrete_overflow_keeps_integer_variant_and_literal_start_offsets() {
    for source in [
        "0b1000000000000000000000000000000000000000000000000000000000000000",
        "-0b1000000000000000000000000000000000000000000000000000000000000001",
        "0o1000000000000000000000",
        "-0o1000000000000000000001",
        "0x8000000000000000",
        "-0X8000000000000001",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        let detail = if source.starts_with('-') {
            "number too small to fit in target type"
        } else {
            "number too large to fit in target type"
        };
        for (text, offset) in [
            (source.to_owned(), 0),
            (format!(">={source}"), 2),
            (format!("<={source}"), 2),
        ] {
            for result in parsed(&text) {
                let error = result.expect_err("checked i64 range");
                assert!(
                    matches!(&error, parser::Error::Integer { offset: actual, .. } if *actual == offset)
                );
                assert_eq!(
                    error.to_string(),
                    format!("source byte {offset}: signed 64-bit integer required: {detail}")
                );
            }
        }
    }
}

#[test]
fn bounds_keep_scalar_thresholds_candidates_and_unbounded_extrema() {
    for &(source, value) in LITERALS {
        for (token, kind) in [(">=", "lowerBound"), ("<=", "upperBound")] {
            let bound = json!({"kind":kind,"value":value});
            check(
                &format!("{token}{source}"),
                bound.clone(),
                Err("non-concrete integer bounds"),
            );
            check(
                &format!("{token}{source} & {source}"),
                binary("unify", bound, integer(value)),
                Ok(value),
            );
        }
    }
    let lower = json!({"kind":"lowerBound","value":42});
    let upper = json!({"kind":"upperBound","value":42});
    let singleton = binary("unify", lower.clone(), upper.clone());
    for (source, entry, expected) in [
        (
            ">=0x2a & 0o51",
            binary("unify", lower.clone(), integer(41)),
            Err("conflicting integer bounds"),
        ),
        (
            "<=0b101010 & 0o53",
            binary("unify", upper, integer(43)),
            Err("conflicting integer bounds"),
        ),
        (
            ">=0x2a & <=0o52",
            singleton.clone(),
            Err("non-concrete integer bounds"),
        ),
        (
            "(>=0x2a & <=0o52) & 0b101010",
            binary("unify", singleton, integer(42)),
            Ok(42),
        ),
        (
            "(>=0x2a + 1) & 0o53",
            binary("unify", binary("add", lower, integer(1)), integer(43)),
            Err("arithmetic requires compatible concrete operands"),
        ),
    ] {
        check(source, entry, expected);
    }
    check(
        ">=(0x2a) & 42",
        binary(
            "unify",
            json!({"kind":"lowerBoundExpression","operand":integer(42)}),
            integer(42),
        ),
        Ok(42),
    );
    // Bare thresholds stay literal: no new prefix-expression/backpropagation rule.
    for source in [">=int", "<=true"] {
        for result in parsed(source) {
            assert!(result.is_err(), "literal threshold only: {source}");
        }
    }
}

#[test]
fn arithmetic_precedence_maximal_munch_and_unary_minimum_are_fixed() {
    check(
        "0b10 + 0o3 * 0x4",
        binary(
            "add",
            integer(2),
            binary("multiply", integer(3), integer(4)),
        ),
        Ok(14),
    );
    check("0x2add1", integer(175569), Ok(175569));
    check(
        "- 0x2a",
        json!({"kind":"negate","operand":integer(42)}),
        Ok(-42),
    );
    check(
        "-0X8000000000000000 + 0b1",
        binary("add", integer(i64::MIN), integer(1)),
        Ok(-9223372036854775807),
    );
    check(
        "0x7fffffffffffffff + 0o1",
        binary("add", integer(i64::MAX), integer(1)),
        Err("signed 64-bit integer addition overflow"),
    );
    check(
        "--0x8000000000000000",
        json!({"kind":"negate","operand":integer(i64::MIN)}),
        Err("signed 64-bit integer negation overflow"),
    );
    for source in [
        "- 0x8000000000000000",
        "-(0o1000000000000000000000)",
        "- 0b1000000000000000000000000000000000000000000000000000000000000000",
    ] {
        for result in parsed(source) {
            assert!(
                matches!(result, Err(parser::Error::Integer { offset: 2, .. })),
                "{source}"
            );
        }
    }
}
