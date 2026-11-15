use std::path::Path;
use std::process::Command;

use bootstrap_executor::scanner::{self, Error, Scan};
use serde::Deserialize;
use serde_json::Value;

#[path = "../../generated/scanner-advance.rs"]
mod advance;
#[path = "../../generated/scanner-horizontal.rs"]
mod horizontal;
#[path = "../../generated/scanner-stalled.rs"]
mod stalled;

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
}

#[test]
fn decimal_scanning_preserves_spans_without_doing_numeric_validation() {
    let cases: Vec<DecimalCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/decimal-scanner-cases.json").expect("cases"),
    )
    .expect("decode decimal cases");
    let decimal = program("decimal");
    for case in cases {
        let expected = Scan {
            offset: case.end,
            flag: case.minus,
        };
        assert_eq!(
            scanner::interpret(&decimal, &case.source, case.start).expect("interpreted"),
            expected,
            "{:?}",
            case.source
        );
        assert_eq!(
            scanner::decimal(&case.source, case.start).expect("compiled"),
            expected,
            "{:?}",
            case.source
        );
        assert!(case.source.is_char_boundary(expected.offset));
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
fn decimal_scanning_matches_the_original_ascii_token_boundary_contract() {
    let decimal = program("decimal");
    for byte in 0_u8..=127 {
        let digit = usize::from(byte.is_ascii_digit());
        for (bytes, expected) in [
            (
                vec![byte],
                Scan {
                    offset: if byte == b'-' { 1 } else { digit },
                    flag: byte == b'-',
                },
            ),
            (
                vec![b'-', byte],
                Scan {
                    offset: 1 + digit,
                    flag: true,
                },
            ),
            (
                vec![b'1', b'2', byte],
                Scan {
                    offset: 2 + digit,
                    flag: false,
                },
            ),
        ] {
            let source = String::from_utf8(bytes).expect("ASCII");
            assert_eq!(
                scanner::interpret(&decimal, &source, 0).expect("interpreted"),
                expected,
                "{source:?}"
            );
            assert_eq!(
                scanner::decimal(&source, 0).expect("compiled"),
                expected,
                "{source:?}"
            );
        }
    }
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
