use std::path::Path;

use bootstrap_executor::{execute_json, generated, parser, scanner};
use scanner::Scan;
use serde_json::{Value, json};

#[path = "../../generated/string-no-wide.rs"]
mod old;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn parsed(source: &str, grammar: &str, pratt: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, pratt),
    ]
}

fn tree(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}

fn text(value: &str) -> Value {
    json!({"kind":"string","value":value})
}

fn output(ast: &Value, evaluation: &str, expected: &Value) {
    let program = ast.to_string();
    for result in [
        execute_json(&program, evaluation),
        generated::execute_json(&program),
    ] {
        assert_eq!(&result.expect("JSON scalar"), expected);
    }
}

#[test]
fn fixed_wide_scalars_have_independent_ast_and_output_oracles() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for (source, expected) in [
        (r#""\U00000000""#, "\0"),
        (
            r#""\U00000008\U0000000C\U0000000a\U0000000D\U00000009""#,
            "\u{8}\u{c}\n\r\t",
        ),
        (r#""\U00000022\U0000005C""#, "\"\\"),
        (r#""\U00000041\U000000e9\U000003bB""#, "Aéλ"),
        (r#""\U0000D7FF\U0000e000""#, "\u{d7ff}\u{e000}"),
        (
            r#""\U0000FEFF\U0000FDD0\U0000FFFE\U0000FFFF""#,
            "\u{feff}\u{fdd0}\u{fffe}\u{ffff}",
        ),
        (
            r#""\U00010000\U0001f980\U0010FfFf""#,
            "\u{10000}🦀\u{10ffff}",
        ),
        (r#""\U00000065\U00000301""#, "e\u{301}"),
        (r#""\U000000410""#, "A0"),
        (r#""\u0041\U0001F980\uD800\uDC00""#, "A🦀\u{10000}"),
    ] {
        for result in parsed(source, &grammar, &pratt) {
            let ast = result.expect("wide tree");
            assert_eq!(ast, tree(text(expected)), "{source:?}");
            output(&ast, &evaluation, &json!(expected));
        }
    }
    // Ownership must not borrow either spelling, including escaped JSON delimiters.
    let owned = {
        let source = String::from(r#""\U00000022\U0000005C""#);
        parser::parse(&source, &grammar).expect("owned wide text")
    };
    assert_eq!(owned, tree(text("\"\\")));
}

#[test]
fn each_plane_edges_match_standard_scalar_and_raw_utf8_oracles() {
    let control = export("string-literal");
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for plane in 0u32..=16 {
        // Avoid raw controls and BOM; include every plane's noncharacter endpoints.
        for scalar in [
            (plane << 16) + 0x20,
            (plane << 16) + 0xfffe,
            (plane << 16) + 0xffff,
        ] {
            let expected = char::from_u32(scalar).expect("scalar oracle").to_string();
            let escaped = format!("\"\\U{scalar:08X}\"");
            let raw = format!("\"{expected}\"");
            for result in [
                scanner::string_literal(&escaped, 0),
                scanner::interpret(&control, &escaped, 0),
            ] {
                assert_eq!(
                    result.expect("wide span"),
                    Scan {
                        offset: 12,
                        flag: true
                    }
                );
            }
            for source in [&escaped, &raw] {
                for result in parsed(source, &grammar, &pratt) {
                    let ast = result.expect("plane tree");
                    assert_eq!(ast, tree(text(&expected)), "{source:?}");
                    output(&ast, &evaluation, &json!(expected));
                }
            }
        }
    }
}

#[test]
fn backslash_parity_and_scalar_delimiters_do_not_change_json_structure() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for (source, expected) in [
        (r#""\\U0001F980""#, "\\U0001F980"),
        (r#""\\\U0001F980""#, "\\🦀"),
        (r#""\\\\U0001F980""#, "\\\\U0001F980"),
        (r#""\\\\\U0001F980""#, "\\\\🦀"),
        (r#""\U0000005CU0001F980""#, "\\U0001F980"),
        (
            r#""\U00000022kind\U00000022:\U00000022x\U0000005C""#,
            "\"kind\":\"x\\",
        ),
    ] {
        for result in parsed(source, &grammar, &pratt) {
            let ast = result.expect("parity tree");
            assert_eq!(ast, tree(text(expected)), "{source:?}");
            output(&ast, &evaluation, &json!(expected));
        }
    }
    let mut custom: Value = serde_json::from_str(&grammar).expect("grammar");
    custom["text"] = json!({"kind":"quoted", "valueField":"payload"});
    for result in parsed(r#""\U00000022\U0000005C""#, &custom.to_string(), &pratt) {
        assert_eq!(
            result.expect("custom field"),
            tree(json!({"kind":"quoted","payload":"\"\\"}))
        );
    }
}

fn rejected(source: &str, start: usize, offset: usize, reason: &str) {
    let control = export("string-literal");
    let expected = format!("source byte {offset}: {reason}");
    for result in [
        scanner::string_literal(source, start),
        scanner::interpret(&control, source, start),
    ] {
        assert_eq!(
            result.expect_err("wide rejection").to_string(),
            expected,
            "{source:?}"
        );
    }
    if start == 0 {
        for result in parsed(source, &export("grammar"), &export("pratt")) {
            assert_eq!(
                result.expect_err("source rejection").to_string(),
                expected,
                "{source:?}"
            );
        }
    }
}

#[test]
fn malformed_digits_reject_at_first_bad_byte_before_scalar_classification() {
    for digits in 0..8 {
        let prefix = format!("\"\\U{}", "0".repeat(digits));
        for suffix in ["", "\""] {
            rejected(
                &format!("{prefix}{suffix}"),
                0,
                3 + digits,
                "invalid Unicode escape",
            );
        }
        for bad in ["g", "/", "λ", "１"] {
            let source = format!("{prefix}{bad}{}\"", "0".repeat(7 - digits));
            rejected(&source, 0, 3 + digits, "invalid Unicode escape");
        }
    }
    for source in [r#""\UFFFFFFFFg""#, r#""\U0000D800g""#] {
        // Eight valid but invalid-scalar digits take precedence over later text.
        rejected(source, 0, 11, "invalid Unicode scalar escape");
    }
    for (source, offset) in [(r#""\UFFFFgFFF""#, 7), (r#""\U0000D80λ""#, 10)] {
        rejected(source, 0, offset, "invalid Unicode escape");
    }
}

#[test]
fn invalid_ranges_and_wide_surrogates_are_not_utf16_pairs() {
    for digits in [
        "00110000", "01000000", "10000000", "FFFFFFFF", "ffffffff", "0000D800", "0000DBFF",
        "0000DC00", "0000DFFF", "0000dfff",
    ] {
        rejected(
            &format!("\"\\U{digits}\""),
            0,
            11,
            "invalid Unicode scalar escape",
        );
    }
    rejected(
        r#""\U0000D800\U0000DC00""#,
        0,
        11,
        "invalid Unicode scalar escape",
    );
    rejected(
        r#""\uD800\U0000DC00""#,
        0,
        7,
        "missing low surrogate escape",
    );
}

#[test]
fn nonzero_utf8_offsets_and_pending_flag_never_fake_closure() {
    let control = export("string-literal");
    let source = "λ\"é\\U0001F980\"tail";
    for result in [
        scanner::string_literal(source, 2),
        scanner::interpret(&control, source, 2),
    ] {
        assert_eq!(
            result.expect("UTF-8 span"),
            Scan {
                offset: 16,
                flag: true
            }
        );
    }
    rejected("λ\"é\\U00110000\"", 2, 15, "invalid Unicode scalar escape");
    rejected("λ\"é\\U000λ0000\"", 2, 10, "invalid Unicode escape");
    rejected("\"\\U0001F980", 0, 11, "unterminated string literal");
    for raw in ["\n", "\0", "\u{feff}"] {
        rejected(
            &format!("\"\\U0001F980{raw}\""),
            0,
            11,
            "unsupported character in string",
        );
    }
    for (source, offset) in [
        (r#""\U0001F980\uD800""#, 17),
        (r#""\uD800\uDC00\U0001F980\uD800""#, 29),
    ] {
        rejected(source, 0, offset, "missing low surrogate escape");
    }
}

#[test]
fn compiled_no_wide_baseline_preserves_old_admission_on_same_input() {
    let current = export("string-literal");
    let baseline = export("tests/valid/string-no-wide");
    let source = r#""\U0001F980""#;
    for result in [
        scanner::string_literal(source, 0),
        scanner::interpret(&current, source, 0),
    ] {
        assert_eq!(
            result.expect("current span"),
            Scan {
                offset: 12,
                flag: true
            }
        );
    }
    for result in [
        old::scan(source, 0),
        scanner::interpret(&baseline, source, 0),
    ] {
        assert_eq!(
            result.expect_err("no wide admission").to_string(),
            "source byte 2: unsupported string escape"
        );
    }
    for (source, end) in [
        (r#""\u0041""#, 8),
        (r#""\uD83E\uDD80""#, 14),
        (r#""\n\\""#, 6),
        (r#""é🦀""#, 8),
    ] {
        for result in [
            old::scan(source, 0),
            scanner::interpret(&baseline, source, 0),
            scanner::string_literal(source, 0),
            scanner::interpret(&current, source, 0),
        ] {
            assert_eq!(
                result.expect("unchanged subset"),
                Scan {
                    offset: end,
                    flag: true
                }
            );
        }
    }
}

#[test]
fn mixed_width_operations_have_fixed_trees_and_values_without_normalization() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for (source, kind, left, right, expected) in [
        (
            r#""\u0041" + "\U0001F980""#,
            "add",
            text("A"),
            text("🦀"),
            json!("A🦀"),
        ),
        (
            r#""\U0001F980" == "\uD83E\uDD80""#,
            "equal",
            text("🦀"),
            text("🦀"),
            json!(true),
        ),
        (
            r#""\U00000065\u0301" == "é""#,
            "equal",
            text("e\u{301}"),
            text("é"),
            json!(false),
        ),
        (
            r#""\U00000000\uFEFF" & "\u0000\U0000FEFF""#,
            "unify",
            text("\0\u{feff}"),
            text("\0\u{feff}"),
            json!("\0\u{feff}"),
        ),
        (
            r#"string & "\U0010FFFF""#,
            "unify",
            json!({"kind":"stringType"}),
            text("\u{10ffff}"),
            json!("\u{10ffff}"),
        ),
    ] {
        for result in parsed(source, &grammar, &pratt) {
            let ast = result.expect("mixed operation");
            assert_eq!(
                ast,
                tree(json!({"kind":kind,"left":left,"right":right})),
                "{source:?}"
            );
            output(&ast, &evaluation, &expected);
        }
    }
}
