use std::path::Path;
use std::process::Command;

use bootstrap_executor::scanner::{self, Error, Scan};
use serde::Deserialize;
use serde_json::Value;

#[path = "../../generated/scanner-advance.rs"]
mod advance;
#[path = "../../generated/scanner-horizontal.rs"]
mod horizontal;
#[path = "../../generated/scanner-lookahead.rs"]
mod lookahead;
#[path = "../../generated/scanner-decimal-permissive.rs"]
mod permissive;
#[path = "../../generated/scanner-reject.rs"]
mod rejecting;
#[path = "../../generated/scanner-stalled.rs"]
mod stalled;
#[path = "../../generated/scanner-state.rs"]
mod state;

fn outcome(result: Result<Scan, Error>) -> Result<Scan, (usize, String)> {
    result.map_err(|error| match error {
        Error::Rejected { offset, reason } => (offset, reason),
        other => panic!("unexpected scanner failure: {other}"),
    })
}
fn program(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("exported scanner")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    source: String,
    start: usize,
    end: usize,
    flag: bool,
    horizontal_end: usize,
    horizontal_flag: bool,
}

#[test]
fn cue_control_flow_matches_interpreted_compiled_and_expected_behavior() {
    let cases: Vec<Case> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/scanner-cases.json").expect("cases"),
    )
    .expect("decode cases");
    let whitespace = program("whitespace");
    let other = program("tests/valid/scanner-horizontal");
    for case in cases {
        let expected = Scan {
            offset: case.end,
            flag: case.flag,
        };
        assert_eq!(
            scanner::interpret(&whitespace, &case.source, case.start).expect("interpreted"),
            expected
        );
        assert_eq!(
            scanner::whitespace(&case.source, case.start).expect("production"),
            expected
        );
        let expected = Scan {
            offset: case.horizontal_end,
            flag: case.horizontal_flag,
        };
        assert_eq!(
            scanner::interpret(&other, &case.source, case.start).expect("alternate interpreted"),
            expected
        );
        assert_eq!(
            horizontal::scan(&case.source, case.start).expect("alternate compiled"),
            expected
        );
    }
}

#[test]
fn default_algorithm_preserves_the_original_ascii_contract() {
    let whitespace = program("whitespace");
    for byte in 0..=127 {
        let source = String::from_utf8(vec![byte]).expect("ASCII");
        let expected = Scan {
            offset: usize::from(matches!(byte, b' ' | b'\t' | b'\n' | b'\r')),
            flag: matches!(byte, b'\n' | b'\r'),
        };
        assert_eq!(
            scanner::interpret(&whitespace, &source, 0).expect("interpreted"),
            expected
        );
        assert_eq!(scanner::whitespace(&source, 0).expect("compiled"), expected);
    }
    let source = " \t\r\n".repeat(4096);
    assert_eq!(
        scanner::whitespace(&source, 0).expect("long run"),
        Scan {
            offset: source.len(),
            flag: true
        }
    );
}

#[test]
fn cursor_guards_and_fuel_apply_to_interpreted_and_emitted_control_flow() {
    let whitespace = program("whitespace");
    for (source, start) in [("", 1), ("λ", 1), ("abc", 4)] {
        assert!(matches!(
            scanner::interpret(&whitespace, source, start),
            Err(Error::Offset)
        ));
        assert!(matches!(
            scanner::whitespace(source, start),
            Err(Error::Offset)
        ));
    }
    let advancing = program("tests/valid/scanner-advance");
    for source in ["", "λ"] {
        assert!(matches!(
            scanner::interpret(&advancing, source, 0),
            Err(Error::Advance)
        ));
        assert!(matches!(advance::scan(source, 0), Err(Error::Advance)));
    }
    let nonprogressing = program("tests/valid/scanner-stalled");
    assert!(matches!(
        scanner::interpret(&nonprogressing, " ", 0),
        Err(Error::Budget)
    ));
    assert!(matches!(stalled::scan(" ", 0), Err(Error::Budget)));
    assert_eq!(
        stalled::scan("", 0).expect("false loop condition"),
        Scan {
            offset: 0,
            flag: false
        }
    );
}

#[derive(Deserialize)]
struct DecimalCase {
    source: String,
    start: usize,
    end: usize,
    minus: bool,
    error: Option<String>,
}

#[test]
fn decimal_spelling_is_validated_without_doing_i64_conversion() {
    let cases: Vec<DecimalCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/decimal-scanner-cases.json").expect("cases"),
    )
    .expect("decode decimal cases");
    let decimal = program("decimal");
    for case in cases {
        let expected = match case.error {
            Some(reason) => Err((case.end, reason)),
            None => Ok(Scan {
                offset: case.end,
                flag: case.minus,
            }),
        };
        assert_eq!(
            outcome(scanner::interpret(&decimal, &case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
        assert_eq!(
            outcome(scanner::decimal(&case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
        assert!(case.source.is_char_boundary(case.end));
    }
    let digits = "1234567890".repeat(4096);
    let source = format!("-{digits}λ");
    let expected = Scan {
        offset: digits.len() + 1,
        flag: true,
    };
    assert_eq!(
        scanner::interpret(&decimal, &source, 0).expect("long interpreted"),
        expected
    );
    assert_eq!(
        scanner::decimal(&source, 0).expect("long compiled"),
        expected
    );
    for (source, start) in [("", 1), ("λ", 1), ("abc", 4)] {
        assert!(matches!(
            scanner::interpret(&decimal, source, start),
            Err(Error::Offset)
        ));
        assert!(matches!(
            scanner::decimal(source, start),
            Err(Error::Offset)
        ));
    }
}

#[test]
fn decimal_spelling_and_rejection_match_the_original_ascii_parser_contract() {
    let decimal = program("decimal");
    let valid = |offset, flag| Ok(Scan { offset, flag });
    let missing = |offset| Err((offset, "expected signed decimal integer".to_owned()));
    let leading = |offset| Err((offset, "leading zeroes are unsupported".to_owned()));
    for byte in 0_u8..=127 {
        let digit = usize::from(byte.is_ascii_digit());
        for (bytes, expected) in [
            (
                vec![byte],
                if digit == 1 {
                    valid(1, false)
                } else {
                    missing(usize::from(byte == b'-'))
                },
            ),
            (
                vec![b'-', byte],
                if digit == 1 {
                    valid(2, true)
                } else {
                    missing(1)
                },
            ),
            (vec![b'1', b'2', byte], valid(2 + digit, false)),
            (
                vec![b'0', byte],
                if digit == 1 {
                    leading(2)
                } else {
                    valid(1, false)
                },
            ),
            (
                vec![b'-', b'0', byte],
                if digit == 1 {
                    leading(3)
                } else {
                    valid(2, true)
                },
            ),
        ] {
            let source = String::from_utf8(bytes).expect("ASCII");
            assert_eq!(
                outcome(scanner::interpret(&decimal, &source, 0)),
                expected,
                "{source:?}"
            );
            assert_eq!(
                outcome(scanner::decimal(&source, 0)),
                expected,
                "{source:?}"
            );
        }
    }
}

#[test]
fn rejection_is_data_driven_escaped_and_stops_execution() {
    let alternate = program("tests/valid/scanner-decimal-permissive");
    for (source, end, minus) in [("0123", 4, false), ("-", 1, true)] {
        let expected = Scan {
            offset: end,
            flag: minus,
        };
        assert!(matches!(
            scanner::decimal(source, 0),
            Err(Error::Rejected { .. })
        ));
        assert_eq!(
            scanner::interpret(&alternate, source, 0).expect("permissive interpreter"),
            expected
        );
        assert_eq!(
            permissive::scan(source, 0).expect("permissive compiled"),
            expected
        );
    }
    let reject = program("tests/valid/scanner-reject");
    let expected = Err((1, "bad \"token\"\\\nλ {value}".to_owned()));
    assert_eq!(outcome(scanner::interpret(&reject, "x", 0)), expected);
    assert_eq!(outcome(rejecting::scan("x", 0)), expected);
}

#[derive(Deserialize)]
struct IdentifierCase {
    source: String,
    start: usize,
    end: usize,
    recognized: bool,
}

#[test]
fn identifier_scanning_preserves_complete_ascii_spans() {
    let cases: Vec<IdentifierCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/identifier-scanner-cases.json").expect("cases"),
    )
    .expect("decode identifier cases");
    let identifier = program("identifier");
    for case in cases {
        let expected = Scan {
            offset: case.end,
            flag: case.recognized,
        };
        assert_eq!(
            scanner::interpret(&identifier, &case.source, case.start).expect("interpreted"),
            expected,
            "{:?}",
            case.source
        );
        assert_eq!(
            scanner::identifier(&case.source, case.start).expect("compiled"),
            expected,
            "{:?}",
            case.source
        );
        assert!(case.source.is_char_boundary(expected.offset));
    }
    let text = format!("_{}λ", "aZ_09".repeat(4096));
    let expected = Scan {
        offset: text.len() - "λ".len(),
        flag: true,
    };
    assert_eq!(
        scanner::interpret(&identifier, &text, 0).expect("long interpreted"),
        expected
    );
    assert_eq!(
        scanner::identifier(&text, 0).expect("long compiled"),
        expected
    );
    for (source, start) in [("", 1), ("λ", 1), ("abc", 4)] {
        assert!(matches!(
            scanner::interpret(&identifier, source, start),
            Err(Error::Offset)
        ));
        assert!(matches!(
            scanner::identifier(source, start),
            Err(Error::Offset)
        ));
    }
}

#[test]
fn identifier_start_and_continuation_match_the_original_ascii_contract() {
    let identifier = program("identifier");
    for byte in 0_u8..=127 {
        let start = byte.is_ascii_alphabetic() || byte == b'_';
        let continuation = byte.is_ascii_alphanumeric() || byte == b'_';
        for (bytes, expected) in [
            (
                vec![byte],
                Scan {
                    offset: usize::from(start),
                    flag: start,
                },
            ),
            (
                vec![b'a', byte],
                Scan {
                    offset: 1 + usize::from(continuation),
                    flag: true,
                },
            ),
            (
                vec![b'_', byte],
                Scan {
                    offset: 1 + usize::from(continuation),
                    flag: true,
                },
            ),
            (
                vec![b'0', byte],
                Scan {
                    offset: 0,
                    flag: false,
                },
            ),
        ] {
            let source = String::from_utf8(bytes).expect("ASCII");
            assert_eq!(
                scanner::interpret(&identifier, &source, 0).expect("interpreted"),
                expected,
                "{source:?}"
            );
            assert_eq!(
                scanner::identifier(&source, 0).expect("compiled"),
                expected,
                "{source:?}"
            );
        }
    }
}

#[derive(Deserialize)]
struct OperandCase {
    source: String,
    start: usize,
    end: usize,
    flag: bool,
    error: Option<String>,
}

#[test]
fn operand_whitespace_policy_reads_state_and_allows_only_trailing_newlines() {
    let cases: Vec<OperandCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/operand-whitespace-cases.json").expect("cases"),
    )
    .expect("operand cases");
    let policy = program("after-operand");
    for case in cases {
        let expected = match case.error {
            Some(reason) => Err((case.end, reason)),
            None => Ok(Scan {
                offset: case.end,
                flag: case.flag,
            }),
        };
        assert_eq!(
            outcome(scanner::interpret(&policy, &case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
        assert_eq!(
            outcome(scanner::after_operand(&case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
    }
    for (source, start) in [("", 1), ("λ", 1)] {
        assert!(matches!(
            scanner::after_operand(source, start),
            Err(Error::Offset)
        ));
        assert!(matches!(
            scanner::interpret(&policy, source, start),
            Err(Error::Offset)
        ));
    }
    let whitespace = program("whitespace");
    let expected = Scan {
        offset: 1,
        flag: true,
    };
    assert_eq!(
        scanner::interpret(&whitespace, "\n+", 0).expect("permissive program"),
        expected
    );
    assert_eq!(
        scanner::whitespace("\n+", 0).expect("permissive compiled"),
        expected
    );
    assert!(matches!(
        scanner::after_operand("\n+", 0),
        Err(Error::Rejected { .. })
    ));
}

#[test]
fn operand_policy_matches_the_original_ascii_condition() {
    let policy = program("after-operand");
    for byte in 0_u8..=127 {
        let whitespace = matches!(byte, b' ' | b'\t' | b'\r' | b'\n');
        for prefix in [b' ', b'\n'] {
            let source = String::from_utf8(vec![prefix, byte]).expect("ASCII");
            let expected = if prefix == b'\n' && !whitespace {
                Err((1, "line break after operand is unsupported".to_owned()))
            } else {
                Ok(Scan {
                    offset: 1 + usize::from(whitespace),
                    flag: prefix == b'\n' || matches!(byte, b'\n' | b'\r'),
                })
            };
            assert_eq!(
                outcome(scanner::interpret(&policy, &source, 0)),
                expected,
                "{source:?}"
            );
            assert_eq!(
                outcome(scanner::after_operand(&source, 0)),
                expected,
                "{source:?}"
            );
        }
    }
}

#[test]
fn generic_predicates_read_updated_state_and_detect_non_ascii_bytes() {
    let program = program("tests/valid/scanner-state");
    for source in ["x", "λ"] {
        let expected = Scan {
            offset: 0,
            flag: false,
        };
        assert_eq!(
            scanner::interpret(&program, source, 0).expect("interpreted state loop"),
            expected
        );
        assert_eq!(
            state::scan(source, 0).expect("compiled state loop"),
            expected
        );
    }
    let expected = Err((0, "expected byte".to_owned()));
    assert_eq!(outcome(scanner::interpret(&program, "", 0)), expected);
    assert_eq!(outcome(state::scan("", 0)), expected);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroupCase {
    source: String,
    start: usize,
    open_end: usize,
    open_flag: bool,
    close_end: usize,
    close_flag: bool,
    error: Option<String>,
}

#[test]
fn parenthesis_scanners_preserve_spans_flags_and_close_diagnostics() {
    let cases: Vec<GroupCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/group-scanner-cases.json").expect("cases"),
    )
    .expect("group cases");
    let opening = program("group-open");
    let closing = program("group-close");
    for case in cases {
        let expected = Scan {
            offset: case.open_end,
            flag: case.open_flag,
        };
        assert_eq!(
            scanner::interpret(&opening, &case.source, case.start).expect("open interpreted"),
            expected
        );
        assert_eq!(
            scanner::group_open(&case.source, case.start).expect("open compiled"),
            expected
        );
        let expected = match case.error {
            Some(reason) => Err((case.close_end, reason)),
            None => Ok(Scan {
                offset: case.close_end,
                flag: case.close_flag,
            }),
        };
        assert_eq!(
            outcome(scanner::interpret(&closing, &case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
        assert_eq!(
            outcome(scanner::group_close(&case.source, case.start)),
            expected,
            "{:?}",
            case.source
        );
    }
    for (source, start) in [("", 1), ("λ", 1), ("abc", 4)] {
        for result in [
            scanner::group_open(source, start),
            scanner::group_close(source, start),
            scanner::interpret(&opening, source, start),
            scanner::interpret(&closing, source, start),
        ] {
            assert!(matches!(result, Err(Error::Offset)));
        }
    }
}

#[test]
fn parenthesis_recognition_and_close_validation_match_all_ascii_bytes() {
    let opening = program("group-open");
    let closing = program("group-close");
    for byte in 0_u8..=127 {
        let source = String::from_utf8(vec![byte]).expect("ASCII");
        let expected = Scan {
            offset: usize::from(byte == b'('),
            flag: byte == b'(',
        };
        assert_eq!(
            scanner::interpret(&opening, &source, 0).expect("open interpreted"),
            expected
        );
        assert_eq!(
            scanner::group_open(&source, 0).expect("open compiled"),
            expected
        );
        let whitespace = matches!(byte, b' ' | b'\t' | b'\r' | b'\n');
        for prefix in [vec![], vec![b' '], vec![b'\n']] {
            let start = prefix.len();
            let newline = prefix.contains(&b'\n');
            let source = String::from_utf8([prefix, vec![byte]].concat()).expect("ASCII");
            let expected = if byte == b')' {
                Ok(Scan {
                    offset: start + 1,
                    flag: newline,
                })
            } else {
                Err((
                    start + usize::from(whitespace),
                    "expected closing parenthesis".to_owned(),
                ))
            };
            assert_eq!(
                outcome(scanner::interpret(&closing, &source, 0)),
                expected,
                "{source:?}"
            );
            assert_eq!(
                outcome(scanner::group_close(&source, 0)),
                expected,
                "{source:?}"
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolCase {
    source: String,
    start: usize,
    bound_end: usize,
    bound_flag: bool,
    infix_end: usize,
    infix_flag: bool,
}

#[test]
fn symbol_scanners_preserve_spans_without_consuming_partial_matches() {
    let cases: Vec<SymbolCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/symbol-scanner-cases.json").expect("cases"),
    )
    .expect("symbol cases");
    let bound = program("bound-prefix");
    let infix = program("infix");
    for case in cases {
        let expected = Scan {
            offset: case.bound_end,
            flag: case.bound_flag,
        };
        assert_eq!(
            scanner::interpret(&bound, &case.source, case.start).expect("bound interpreted"),
            expected
        );
        assert_eq!(
            scanner::bound_prefix(&case.source, case.start).expect("bound compiled"),
            expected
        );
        let expected = Scan {
            offset: case.infix_end,
            flag: case.infix_flag,
        };
        assert_eq!(
            scanner::interpret(&infix, &case.source, case.start).expect("infix interpreted"),
            expected
        );
        assert_eq!(
            scanner::infix(&case.source, case.start).expect("infix compiled"),
            expected
        );
    }
    for (source, start) in [("", 1), ("λ", 1), ("abc", 4)] {
        for result in [
            scanner::bound_prefix(source, start),
            scanner::infix(source, start),
            scanner::interpret(&bound, source, start),
            scanner::interpret(&infix, source, start),
        ] {
            assert!(matches!(result, Err(Error::Offset)));
        }
    }
}

#[test]
fn symbol_recognition_matches_the_supported_ascii_alphabet() {
    let bound = program("bound-prefix");
    let infix = program("infix");
    for byte in 0_u8..=127 {
        let source = String::from_utf8(vec![byte, b'=']).expect("ASCII");
        let flag = matches!(byte, b'<' | b'>');
        let expected = Scan {
            offset: if flag { 2 } else { 0 },
            flag,
        };
        assert_eq!(
            scanner::interpret(&bound, &source, 0).expect("bound interpreted"),
            expected
        );
        assert_eq!(
            scanner::bound_prefix(&source, 0).expect("bound compiled"),
            expected
        );
        let flag = matches!(byte, b'+' | b'-' | b'*' | b'/' | b'&');
        let expected = Scan {
            offset: usize::from(flag),
            flag,
        };
        assert_eq!(
            scanner::interpret(&infix, &source, 0).expect("infix interpreted"),
            expected
        );
        assert_eq!(
            scanner::infix(&source, 0).expect("infix compiled"),
            expected
        );
        for first in [b'<', b'>'] {
            let source = String::from_utf8(vec![first, byte]).expect("ASCII");
            let expected = Scan {
                offset: if byte == b'=' { 2 } else { 0 },
                flag: byte == b'=',
            };
            assert_eq!(
                scanner::interpret(&bound, &source, 0).expect("lookahead interpreted"),
                expected
            );
            assert_eq!(
                scanner::bound_prefix(&source, 0).expect("lookahead compiled"),
                expected
            );
        }
    }
}

#[test]
fn lookahead_is_bounded_non_consuming_and_safe_past_eof() {
    let cursor = scanner::Cursor::new("xy", 1).expect("cursor");
    assert_eq!(cursor.peek_at(0), Some(b'y'));
    assert_eq!(cursor.peek_at(1), None);
    assert_eq!(cursor.peek_at(usize::MAX), None);
    let program = program("tests/valid/scanner-lookahead");
    for source in [
        format!("{}=", "x".repeat(64)),
        format!("{}=", "λ".repeat(32)),
    ] {
        let expected = Scan {
            offset: 0,
            flag: true,
        };
        assert_eq!(
            scanner::interpret(&program, &source, 0).expect("interpreted limit"),
            expected
        );
        assert_eq!(
            lookahead::scan(&source, 0).expect("compiled limit"),
            expected
        );
    }
    let expected = Err((0, "expected equals at lookahead".to_owned()));
    assert_eq!(outcome(scanner::interpret(&program, "=", 0)), expected);
    assert_eq!(outcome(lookahead::scan("=", 0)), expected);
}

#[derive(Deserialize)]
struct InvalidCase {
    program: Value,
    error: String,
}

#[test]
fn validation_is_shared_and_generation_is_repeatable() {
    let cases: Vec<InvalidCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/scanner-invalid-cases.json").expect("cases"),
    )
    .expect("decode cases");
    for case in cases {
        let input = serde_json::to_string(&case.program).expect("serialize case");
        for error in [
            scanner::generate(&input).expect_err("generation"),
            scanner::interpret(&input, "", 0).expect_err("interpretation"),
        ] {
            assert!(error.to_string().contains(&case.error), "{error}");
        }
    }
    let input = program("whitespace");
    let first = scanner::generate(&input).expect("generate");
    assert_eq!(first, scanner::generate(&input).expect("repeat"));
    assert!(first.contains("while {") && first.contains("if matches!"));
    let mut empty: Value = serde_json::from_str(&input).expect("program");
    empty["body"].as_array_mut().expect("body").clear();
    let empty = serde_json::to_string(&empty).expect("empty program");
    assert_eq!(
        scanner::interpret(&empty, " x", 0).expect("no-op"),
        Scan {
            offset: 0,
            flag: false
        }
    );
    assert!(
        scanner::generate(&empty)
            .expect("no-op source")
            .contains("let cursor =")
    );
    let mut reordered: Value = serde_json::from_str(&input).expect("scanner");
    let bytes = reordered
        .pointer_mut("/body/0/condition/bytes")
        .expect("predicate")
        .as_array_mut()
        .expect("bytes");
    bytes.reverse();
    bytes.push(bytes[0].clone());
    assert_eq!(
        first,
        scanner::generate(&serde_json::to_string(&reordered).expect("serialize"))
            .expect("duplicate byte set")
    );
    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("whitespace.json");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-scan-codegen"))
        .arg(path)
        .output()
        .expect("compiler CLI");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, first.as_bytes());
    for args in [
        vec![],
        vec!["one", "two"],
        vec!["not-a-file"],
        vec!["tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-scan-codegen"))
            .args(args)
            .output()
            .expect("compiler failure");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
