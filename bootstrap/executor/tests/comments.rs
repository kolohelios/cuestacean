use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::{parser, scanner};
use scanner::{Cursor, Error, Scan};
use serde_json::{Value, json};

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn program(body: Value) -> String {
    json!({"formatVersion": 1, "body": body}).to_string()
}

fn fixture(category: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/{category}/comments-{name}.cue"))
}

#[test]
fn trivia_offsets_and_newline_policy() {
    let ws = export("whitespace");
    let after = export("after-operand");
    for (text, offset, flag) in [
        ("//λ😀e\u{301}", "//λ😀e\u{301}".len(), false),
        (" //a\r+2\nX", " //a\r+2\n".len(), true),
        ("//a\r\n//b\n  X", "//a\r\n//b\n  ".len(), true),
        ("//one\n//two", "//one\n//two".len(), true),
        ("/x", 0, false),
        ("/*x*/", 0, false),
        (" \t/", 2, false),
    ] {
        let expected = Scan { offset, flag };
        assert_eq!(
            scanner::interpret(&ws, text, 0).expect("interpreted whitespace"),
            expected,
            "{text:?}"
        );
        assert_eq!(
            scanner::whitespace(text, 0).expect("generated whitespace"),
            expected,
            "{text:?}"
        );
    }
    let positioned = "λ //😀\nX";
    let expected = Scan {
        offset: 10,
        flag: true,
    };
    assert_eq!(
        scanner::whitespace(positioned, 2).expect("nonzero trivia start"),
        expected
    );
    assert_eq!(
        scanner::interpret(&ws, positioned, 2).expect("reference nonzero start"),
        expected
    );
    let close = "λ //😀\n)";
    let expected = Scan {
        offset: 11,
        flag: true,
    };
    assert_eq!(
        scanner::group_close(close, 2).expect("closing trivia"),
        expected
    );
    assert_eq!(
        scanner::interpret(&export("group-close"), close, 2).expect("reference closing trivia"),
        expected
    );
    for (text, offset) in [("//x\n+", 4), ("//x\r\n)", 5), (" \r//x\nλ", 6)] {
        for result in [
            scanner::interpret(&after, text, 0),
            scanner::after_operand(text, 0),
        ] {
            match result.expect_err("newline before another token") {
                Error::Rejected {
                    offset: actual,
                    reason,
                } => {
                    assert_eq!(actual, offset, "{text:?}");
                    assert_eq!(reason, "line break after operand is unsupported");
                }
                other => panic!("unexpected error: {other}"),
            }
        }
    }
    for text in ["//x", "//x\r+2", "//x\n", "//x\r\n"] {
        let expected = Scan {
            offset: text.len(),
            flag: text.ends_with('\n'),
        };
        assert_eq!(
            scanner::interpret(&after, text, 0).expect("interpreted"),
            expected
        );
        assert_eq!(
            scanner::after_operand(text, 0).expect("generated"),
            expected
        );
    }
}

#[test]
fn forbidden_comment_scalars_have_byte_diagnostics() {
    let ws = export("whitespace");
    let after = export("after-operand");
    for bad in ['\0', '\u{feff}'] {
        let text = format!(" //λ{bad} tail\n");
        let offset = " //λ".len();
        for result in [
            scanner::interpret(&ws, &text, 0),
            scanner::whitespace(&text, 0),
            scanner::interpret(&after, &text, 0),
            scanner::after_operand(&text, 0),
            scanner::group_close(&text, 0),
            scanner::interpret(&export("group-close"), &text, 0),
        ] {
            match result.expect_err("forbidden scalar") {
                Error::Rejected {
                    offset: actual,
                    reason,
                } => {
                    assert_eq!(actual, offset);
                    assert_eq!(reason, "unsupported character in line comment");
                }
                error => panic!("unexpected error: {error}"),
            }
        }
        let source = format!("1 //λ{bad}\n");
        let error = parser::parse(&source, &export("grammar")).expect_err("source rejected");
        assert!(
            error
                .to_string()
                .contains("source byte 6: unsupported character in line comment"),
            "{error}"
        );
        let path = std::env::temp_dir().join(format!(
            "cuestacean-comments-{}-{:x}.cue",
            std::process::id(),
            bad as u32
        ));
        std::fs::write(&path, &source).expect("write temporary source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        std::fs::remove_file(path).expect("remove temporary source");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("source byte 6: unsupported character in line comment")
        );
    }
}

#[test]
fn cursor_scalar_boundary_and_ascii_compatibility() {
    assert!(matches!(Cursor::new("λ😀", 1), Err(Error::Offset)));
    let mut cursor = Cursor::new("aλ😀", 0).expect("cursor");
    assert_eq!(cursor.peek_char(), Some('a'));
    cursor.advance().expect("ASCII");
    assert_eq!(cursor.peek(), Some(0xce));
    assert_eq!(cursor.peek_at(1), Some(0xbb));
    assert_eq!(cursor.peek_at(usize::MAX), None);
    assert!(matches!(cursor.advance(), Err(Error::Advance)));
    assert_eq!(cursor.peek_char(), Some('λ'));
    cursor.advance_char().expect("lambda");
    assert_eq!(cursor.peek_char(), Some('😀'));
    cursor.advance_char().expect("emoji");
    assert_eq!(cursor.peek_char(), None);
    cursor.advance_char().expect_err("EOF");
    assert_eq!(cursor.finish().offset, "aλ😀".len());
}

#[test]
fn char_predicates_validate_and_emit_escaped_sets() {
    let body = json!([{ "op":"if", "condition":{"op":"charIn", "characters":["λ", "'", "\\", "\n", "😀", "λ"]}, "then":[{"op":"advanceChar"}], "else":[]}]);
    let input = program(body.clone());
    for text in ["λ!", "'!", "\\!", "\n!", "😀!"] {
        assert_eq!(
            scanner::interpret(&input, text, 0).expect("match").offset,
            text.chars().next().expect("character").len_utf8()
        );
    }
    assert_eq!(scanner::interpret(&input, "x", 0).expect("miss").offset, 0);
    let generated = scanner::generate(&input).expect("generate");
    assert!(generated.contains("peek_char()"));
    assert!(generated.contains("advance_char()"));
    let mut reordered = body;
    reordered[0]["condition"]["characters"]
        .as_array_mut()
        .expect("characters")
        .reverse();
    assert_eq!(
        generated,
        scanner::generate(&program(reordered)).expect("reordered")
    );
    for characters in [
        json!([]),
        json!([""]),
        json!(["ab"]),
        json!(["e\u{301}"]),
        json!(vec!["x"; 129]),
    ] {
        let invalid = program(
            json!([{ "op":"if", "condition":{"op":"charIn", "characters":characters}, "then":[] }]),
        );
        scanner::interpret(&invalid, "", 0).expect_err("invalid predicate");
        scanner::generate(&invalid).expect_err("invalid predicate");
    }
    let inverted = program(
        json!([{ "op":"if", "condition":{"op":"charIn", "characters":["λ"]}, "then":[], "else":[{"op":"setFlag", "value":true}] }]),
    );
    assert!(
        scanner::interpret(&inverted, "x", 0)
            .expect("inverted")
            .flag
    );
    assert!(
        !scanner::interpret(&inverted, "λ", 0)
            .expect("inverted")
            .flag
    );
    assert!(
        scanner::generate(&inverted)
            .expect("inverted generation")
            .contains("peek_char()")
    );
}

#[test]
fn break_scope_fuel_and_unreachable_emission() {
    for body in [
        json!([{"op":"break"}]),
        json!([{"op":"if", "condition":{"op":"hasByte"}, "then":[{"op":"break"}]}]),
    ] {
        let input = program(body);
        scanner::interpret(&input, "x", 0).expect_err("break outside loop");
        scanner::generate(&input).expect_err("break outside loop");
    }
    let input = program(json!([
        {"op":"while", "condition":{"op":"hasByte"}, "body":[
            {"op":"while", "condition":{"op":"hasByte"}, "body":[{"op":"if", "condition":{"op":"hasByte"}, "then":[{"op":"break"}]}]},
            {"op":"advanceChar"}, {"op":"break"}, {"op":"reject", "reason":"unreachable suffix"}
        ]}, {"op":"setFlag", "value":true}
    ]));
    assert_eq!(
        scanner::interpret(&input, "λx", 0).expect("nested break"),
        Scan {
            offset: 2,
            flag: true
        }
    );
    let generated = scanner::generate(&input).expect("nested generation");
    assert!(!generated.contains("unreachable suffix"));
    let both = program(json!([{"op":"while", "condition":{"op":"hasByte"}, "body":[
        {"op":"if", "condition":{"op":"charIn", "characters":["x"]}, "then":[{"op":"break"}], "else":[{"op":"break"}]},
        {"op":"reject", "reason":"both branches break"}
    ]}]));
    assert_eq!(
        scanner::interpret(&both, "x", 0)
            .expect("both break")
            .offset,
        0
    );
    assert!(
        !scanner::generate(&both)
            .expect("both branches")
            .contains("both branches break")
    );
    let infinite = program(json!([{"op":"while", "condition":{"op":"hasByte"}, "body":[]}]));
    assert!(matches!(
        scanner::interpret(&infinite, "x", 0),
        Err(Error::Budget)
    ));
}

#[test]
fn fixture_cli_values_and_failures() {
    for (name, expected) in [
        ("unicode", 3),
        ("bounds", 8),
        ("cr-content", 1),
        ("operator", 10),
        ("word-operator", 5),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(fixture("valid", name))
            .output()
            .expect("CLI");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
    }
    let grammar = export("grammar");
    for word in ["div", "mod", "quo", "rem"] {
        for (text, offset) in [
            (format!("8 {word}// λ\n2"), 11),
            (format!("8 {word}\n2"), 6),
        ] {
            for result in [
                parser::parse(&text, &grammar),
                parser::parse_interpreted(&text, &grammar, &export("pratt")),
            ] {
                assert_eq!(
                    result.expect_err("word operator newline").to_string(),
                    format!("source byte {offset}: line break after word operator is unsupported")
                );
            }
        }
    }
    for name in [
        "newline",
        "split-atom",
        "split-operator",
        "block",
        "after-word",
        "word-newline",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(fixture("rejected", name))
            .output()
            .expect("CLI");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
