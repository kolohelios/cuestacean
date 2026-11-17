use std::path::Path;

use bootstrap_executor::{execute_json, generated, parser, scanner};
use scanner::Scan;
use serde_json::{Value, json};

#[path = "../../generated/string-no-unicode.rs"]
mod legacy;

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

fn string_tree(value: &str) -> Value {
    json!({"formatVersion":1,"entry":{"kind":"string","value":value}})
}

fn accepted(control: &str, source: &str, start: usize, end: usize) {
    let expected = Scan {
        offset: end,
        flag: true,
    };
    for result in [
        scanner::string_literal(source, start),
        scanner::interpret(control, source, start),
    ] {
        assert_eq!(result.expect("Unicode span"), expected, "{source:?}");
    }
}

#[test]
fn every_bmp_scalar_has_an_independent_fixed_width_span() {
    let control = export("string-literal");
    // Public reference execution accepts JSON, not a reusable decoded program.
    // Keep exhaustive coverage at the scanner boundary, not the whole parser.
    for unit in 0u32..=0xffff {
        if char::from_u32(unit).is_none() {
            continue;
        }
        let source = format!("\"\\u{unit:04X}\"tail");
        accepted(&control, &source, 0, 8);
    }
}

#[test]
fn fixed_scalars_decode_into_owned_trees_and_json_values() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for (source, expected) in [
        (r#""\u0000""#, "\0"),
        (r#""\u0008\u000C\u000a\u000D\u0009""#, "\u{8}\u{c}\n\r\t"),
        (r#""\u0022\u005c""#, "\"\\"),
        (r#""\u0041\u00e9\u03BB""#, "Aéλ"),
        (r#""\uD7FF\uE000""#, "\u{d7ff}\u{e000}"),
        (
            r#""\uFEFF\uFDD0\uFFFE\uFFFF""#,
            "\u{feff}\u{fdd0}\u{fffe}\u{ffff}",
        ),
        (r#""\uD800\uDC00""#, "\u{10000}"),
        (r#""\ud800\udfff""#, "\u{103ff}"),
        (r#""\uDbFf\uDc00""#, "\u{10fc00}"),
        (r#""\udbff\uDfFf""#, "\u{10ffff}"),
        (r#""\uD83E\uDD80""#, "🦀"),
        (r#""\u0065\u0301""#, "e\u{301}"),
        (r#""\\u0041""#, "\\u0041"),
        (r#""\uD800\uDC00\u0041\uD83E\uDD80""#, "\u{10000}A🦀"),
    ] {
        for result in parsed(source, &grammar, &pratt) {
            let tree = result.expect("Unicode tree");
            assert_eq!(tree, string_tree(expected), "{source:?}");
            for value in [
                execute_json(&tree.to_string(), &evaluation),
                generated::execute_json(&tree.to_string()),
            ] {
                assert_eq!(value.expect("Unicode JSON scalar"), json!(expected));
            }
        }
    }
}

#[test]
fn surrogate_pair_edges_match_native_scalar_oracle_and_raw_utf8() {
    let control = export("string-literal");
    let grammar = export("grammar");
    let pratt = export("pratt");
    for high in [0xd800u32, 0xd801, 0xdbfe, 0xdbff] {
        for low in [0xdc00u32, 0xdc01, 0xdffe, 0xdfff] {
            let scalar = 0x10000 + ((high - 0xd800) << 10) + low - 0xdc00;
            let expected = char::from_u32(scalar)
                .expect("valid independent scalar")
                .to_string();
            let source = format!("\"\\u{high:04x}\\u{low:04X}\"");
            let raw = format!("\"{expected}\"");
            accepted(&control, &source, 0, 14);
            for spelling in [&source, &raw] {
                for result in parsed(spelling, &grammar, &pratt) {
                    assert_eq!(result.expect("paired or raw tree"), string_tree(&expected));
                }
            }
        }
    }
}

#[test]
fn malformed_units_and_pairs_have_exact_byte_diagnostics() {
    let control = export("string-literal");
    let grammar = export("grammar");
    let pratt = export("pratt");
    for (source, offset, reason) in [
        ("\"\\u", 3, "invalid Unicode escape"),
        (r#""\u""#, 3, "invalid Unicode escape"),
        (r#""\u0""#, 4, "invalid Unicode escape"),
        (r#""\u00""#, 5, "invalid Unicode escape"),
        (r#""\u000""#, 6, "invalid Unicode escape"),
        (r#""\uG000""#, 3, "invalid Unicode escape"),
        (r#""\u0g00""#, 4, "invalid Unicode escape"),
        (r#""\u00/0""#, 5, "invalid Unicode escape"),
        (r#""\u000λ""#, 6, "invalid Unicode escape"),
        (r#""\u１２３４""#, 3, "invalid Unicode escape"),
        ("\"\\uD8", 5, "invalid Unicode escape"),
        (r#""\uDC00""#, 3, "unexpected low surrogate escape"),
        (r#""\udfff""#, 3, "unexpected low surrogate escape"),
        (r#""\uDC00\uD800""#, 3, "unexpected low surrogate escape"),
        (r#""\uD800""#, 7, "missing low surrogate escape"),
        ("\"\\uDBFF", 7, "missing low surrogate escape"),
        (r#""\uD800x""#, 7, "missing low surrogate escape"),
        ("\"\\uD800\\", 7, "missing low surrogate escape"),
        (r#""\uD800λ""#, 7, "missing low surrogate escape"),
        (r#""\uD800\U0000DC00""#, 7, "missing low surrogate escape"),
        (r#""\uD800\\uDC00""#, 7, "missing low surrogate escape"),
        (r#""\uD800\u0041""#, 9, "expected low surrogate escape"),
        (r#""\uD800\uD800""#, 9, "expected low surrogate escape"),
        (r#""\uD800\uE000""#, 9, "expected low surrogate escape"),
        (r#""\uD800\uλ000""#, 9, "expected low surrogate escape"),
        ("\"\\uD800\\u", 9, "expected low surrogate escape"),
        (r#""\uD800\u""#, 9, "expected low surrogate escape"),
        (r#""\uD800\uD""#, 9, "expected low surrogate escape"),
        (r#""\uD800\uDC""#, 11, "invalid Unicode escape"),
        ("\"\\uD800\\uDCF", 12, "invalid Unicode escape"),
        (r#""\uD800\uDCλ0""#, 11, "invalid Unicode escape"),
        (r#""\uD800\uDFFg""#, 12, "invalid Unicode escape"),
        (r#""\U00010000""#, 2, "unsupported string escape"),
    ] {
        let expected = format!("source byte {offset}: {reason}");
        for result in [
            scanner::string_literal(source, 0),
            scanner::interpret(&control, source, 0),
        ] {
            assert_eq!(
                result.expect_err("malformed escape").to_string(),
                expected,
                "{source:?}"
            );
        }
        for result in parsed(source, &grammar, &pratt) {
            assert_eq!(
                result.expect_err("malformed source").to_string(),
                expected,
                "{source:?}"
            );
        }
    }
}

#[test]
fn nonzero_utf8_offsets_preserve_spans_errors_and_completion_flag() {
    let control = export("string-literal");
    accepted(&control, "λ\"é\\uD83E\\uDD80\\u0000\"tail", 2, 24);
    for result in [
        scanner::string_literal("λ\"é\\u00λ0\"", 2),
        scanner::interpret(&control, "λ\"é\\u00λ0\"", 2),
    ] {
        assert_eq!(
            result.expect_err("UTF-8 nonhex").to_string(),
            "source byte 9: invalid Unicode escape"
        );
    }
    for result in [
        scanner::string_literal("λ\"\\uD800\\uDC00", 2),
        scanner::interpret(&control, "λ\"\\uD800\\uDC00", 2),
    ] {
        assert_eq!(
            result.expect_err("pair is not closure").to_string(),
            "source byte 15: unterminated string literal"
        );
    }
    for start in [1, 3] {
        assert!(scanner::string_literal("λé", start).is_err());
        assert!(scanner::interpret(&control, "λé", start).is_err());
    }
}

#[test]
fn compiled_legacy_policy_changes_admission_on_unchanged_input() {
    let current = export("string-literal");
    let old = export("tests/valid/string-no-unicode");
    let source = r#""\u0041""#;
    accepted(&current, source, 0, 8);
    for result in [legacy::scan(source, 0), scanner::interpret(&old, source, 0)] {
        assert_eq!(
            result.expect_err("legacy admission").to_string(),
            "source byte 2: unsupported string escape"
        );
    }
    for (source, end) in [(r#""é🦀""#, 8), (r#""\n\\""#, 6)] {
        accepted(&current, source, 0, end);
        let expected = Scan {
            offset: end,
            flag: true,
        };
        assert_eq!(
            legacy::scan(source, 0).expect("legacy common subset"),
            expected
        );
        assert_eq!(
            scanner::interpret(&old, source, 0).expect("reference legacy common subset"),
            expected
        );
    }
}

#[test]
fn unicode_source_operations_have_fixed_outputs_without_normalization() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let evaluation = export("evaluation");
    for (source, expected) in [
        (r#""\u0041" + "\uD83E\uDD80""#, json!("A🦀")),
        (r#""\uD83E\uDD80" == "🦀""#, json!(true)),
        (r#""\u0065\u0301" == "é""#, json!(false)),
        (r#""\u0000\uFEFF" & "\u0000\uFEFF""#, json!("\0\u{feff}")),
        (r#"string & "\uFFFF""#, json!("\u{ffff}")),
    ] {
        for result in parsed(source, &grammar, &pratt) {
            let tree = result.expect("operation tree").to_string();
            for result in [
                execute_json(&tree, &evaluation),
                generated::execute_json(&tree),
            ] {
                assert_eq!(result.expect("operation JSON"), expected, "{source:?}");
            }
        }
    }
}
