use std::path::Path;
use std::process::Command;

use bootstrap_executor::{Error, ast, execute_json, generated, parser, scanner};
use scanner::Scan;
use serde_json::{Value, json};

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn parsed(source: &str, grammar: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}

fn tree(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}

#[test]
fn literal_scanner_has_independent_byte_spans_and_errors() {
    let control = export("string-literal");
    for (source, start, offset, flag) in [
        ("", 0, 0, false),
        ("abc", 0, 0, false),
        ("λx", 2, 2, false),
        ("\"\"tail", 0, 2, true),
        ("λ\"🦀é\"tail", 2, 10, true),
        (r#""\n\t"tail"#, 0, 6, true),
        (r#""\u1234"tail"#, 0, 8, true),
        (r#""\U00001234"tail"#, 0, 12, true),
        (r#""\uD800\uDC00"tail"#, 0, 14, true),
        (r#""\"\\"!"#, 0, 6, true),
    ] {
        let expected = Scan { offset, flag };
        assert_eq!(
            scanner::string_literal(source, start).expect("compiled"),
            expected,
            "{source:?}"
        );
        assert_eq!(
            scanner::interpret(&control, source, start).expect("interpreted"),
            expected,
            "{source:?}"
        );
    }
    for (source, offset, reason) in [
        ("\"", 1, "unterminated string literal"),
        ("\"λ", 3, "unterminated string literal"),
        ("\"\\", 2, "unsupported string escape"),
        (r#""\u""#, 3, "invalid Unicode escape"),
        (r#""\U00110000""#, 11, "invalid Unicode scalar escape"),
        (r#""\uD800\uDC00"#, 13, "unterminated string literal"),
        (
            "\"\\uD800\\uDC00\n\"",
            13,
            "unsupported character in string",
        ),
        (
            "\"\\uD800\\uDC00\u{feff}\"",
            13,
            "unsupported character in string",
        ),
        (r#""\(x)""#, 2, "unsupported string escape"),
        ("\"λ\n\"", 3, "unsupported character in string"),
        ("\"\0\"", 1, "unsupported character in string"),
        ("\"λ\u{feff}\"", 3, "unsupported character in string"),
    ] {
        let expected = format!("source byte {offset}: {reason}");
        for result in [
            scanner::string_literal(source, 0),
            scanner::interpret(&control, source, 0),
        ] {
            assert_eq!(
                result.expect_err("lexical rejection").to_string(),
                expected,
                "{source:?}"
            );
        }
    }
}

#[test]
fn ascii_escape_and_raw_control_boundaries_are_exhaustive() {
    let control = export("string-literal");
    for byte in 0u8..=127 {
        let escaped = format!("\"\\{}\"", char::from(byte));
        let raw = format!("\"{}\"", char::from(byte));
        for scan in [
            scanner::string_literal(&escaped, 0),
            scanner::interpret(&control, &escaped, 0),
        ] {
            if [34, 92, 98, 102, 110, 114, 116].contains(&byte) {
                assert_eq!(
                    scan.expect("allowed escape"),
                    Scan {
                        offset: 4,
                        flag: true
                    }
                );
            } else if matches!(byte, b'u' | b'U') {
                assert_eq!(
                    scan.expect_err("missing Unicode digits").to_string(),
                    "source byte 3: invalid Unicode escape"
                );
            } else {
                assert_eq!(
                    scan.expect_err("unsupported escape").to_string(),
                    "source byte 2: unsupported string escape"
                );
            }
        }
        if byte < 32 {
            for scan in [
                scanner::string_literal(&raw, 0),
                scanner::interpret(&control, &raw, 0),
            ] {
                assert_eq!(
                    scan.expect_err("raw control").to_string(),
                    "source byte 1: unsupported character in string"
                );
            }
        }
    }
    // JSON permits escaped slash; the CUE lexical policy deliberately does not.
    assert!(serde_json::from_str::<String>(r#""\/""#).is_ok());
    assert!(scanner::string_literal("λ", 1).is_err());
    assert!(scanner::interpret(&control, "λ", 1).is_err());
}

#[test]
fn builder_and_source_trees_are_owned_and_independently_specified() {
    let value = "🦀e\u{301}\n\"\\";
    let expected = json!({"kind":"s","payload":value});
    assert_eq!(
        ast::text("s".into(), "payload".into(), value.into()),
        expected
    );
    assert_eq!(
        ast::interpret(
            &export("ast-text"),
            &[json!("s"), json!("payload"), json!(value)]
        )
        .expect("interpreted builder"),
        expected
    );
    let grammar = export("grammar");
    for (source, entry) in [
        (r#""""#, json!({"kind":"string","value":""})),
        (r#""\u1234""#, json!({"kind":"string","value":"\u{1234}"})),
        (r#""\\u1234""#, json!({"kind":"string","value":r"\u1234"})),
        (
            r#""\u0000\uFEFF\uFFFF""#,
            json!({"kind":"string","value":"\0\u{feff}\u{ffff}"}),
        ),
        (r#""🦀é\n\"\\""#, json!({"kind":"string","value":value})),
        (
            r#""\b\f\n\r\t""#,
            json!({"kind":"string","value":"\u{8}\u{c}\n\r\t"}),
        ),
        ("string", json!({"kind":"stringType"})),
        (
            "(string & // comment\n \"x\")",
            json!({"kind":"unify","left":{"kind":"stringType"},"right":{"kind":"string","value":"x"}}),
        ),
    ] {
        for result in parsed(source, &grammar) {
            assert_eq!(
                result.expect("source tree"),
                tree(entry.clone()),
                "{source:?}"
            );
        }
    }
    let owned = {
        let source = String::from("\"é\"");
        parser::parse(&source, &grammar).expect("owned")
    };
    assert_eq!(owned, tree(json!({"kind":"string","value":"é"})));
}

#[test]
fn grammar_metadata_and_phase_guards() {
    let grammar = export("grammar");
    let mut custom: Value = serde_json::from_str(&grammar).expect("grammar");
    custom["text"] = json!({"kind":"custom", "valueField":"payload"});
    for result in parsed("\"x\"", &custom.to_string()) {
        assert_eq!(
            result.expect("custom"),
            tree(json!({"kind":"custom","payload":"x"}))
        );
    }
    custom.as_object_mut().expect("object").remove("text");
    for result in parsed("\"x\"", &custom.to_string()) {
        assert!(result.is_err());
    }
    for result in parsed("string", &custom.to_string()) {
        assert_eq!(
            result.expect("type remains"),
            tree(json!({"kind":"stringType"}))
        );
    }
    for text in [
        json!({"kind":"", "valueField":"value"}),
        json!({"kind":"x", "valueField":""}),
        json!({"kind":"x", "valueField":"kind"}),
    ] {
        custom["text"] = text;
        for result in parsed("1", &custom.to_string()) {
            assert!(
                result
                    .expect_err("invalid metadata")
                    .to_string()
                    .contains("invalid string constructor")
            );
        }
    }
    for source in [
        r#""\u""#,
        r#""\U00110000""#,
        r#""\x20""#,
        r#""\123""#,
        r#""\(x)""#,
        "#\"raw\"#",
        "\"\"\"multi\"\"\"",
    ] {
        for result in parsed(source, &grammar) {
            assert!(result.is_err(), "{source:?}");
        }
    }
    for op in ["scanString", "parseString"] {
        let control = json!({"formatVersion":1,"body":[{"op":op}]}).to_string();
        assert!(
            matches!(
                parser::parse_interpreted_with_primary(
                    "\"x\"",
                    &grammar,
                    &export("pratt"),
                    &control
                ),
                Err(parser::Error::ControlState { .. })
            ),
            "{op}"
        );
    }
}

#[test]
fn source_limits_evaluation_and_cli() {
    let grammar = export("grammar");
    let within = vec!["\"x\""; 32].join(" & ");
    let over = vec!["\"x\""; 33].join(" & ");
    for result in parsed(&within, &grammar) {
        assert!(result.is_ok());
    }
    for result in parsed(&over, &grammar) {
        assert!(result.is_err());
    }
    let boundary = format!("{}\"x\"{}", "(".repeat(32), ")".repeat(32));
    for result in parsed(&boundary, &grammar) {
        assert_eq!(
            result.expect("32 levels"),
            tree(json!({"kind":"string","value":"x"}))
        );
    }
    let deeply_nested = format!("{}\"x\"{}", "(".repeat(33), ")".repeat(33));
    for result in parsed(&deeply_nested, &grammar) {
        assert!(result.is_err());
    }
    for (source, expected) in [
        (r#"string & "λ""#, Ok(json!("λ"))),
        (
            r#""a" & "b""#,
            Err("conflicting string values \"a\" and \"b\""),
        ),
        ("string", Err("non-concrete string value")),
    ] {
        for result in parsed(source, &grammar) {
            let input = result.expect("string tree").to_string();
            for value in [
                execute_json(&input, &export("evaluation")),
                generated::execute_json(&input),
            ] {
                assert_eq!(
                    value.map_err(|e| e.to_string()),
                    expected.clone().map_err(str::to_owned)
                );
            }
        }
    }
    for result in parsed("\"x\" + 1", &grammar) {
        let program = result.expect("arithmetic tree").to_string();
        assert!(matches!(
            generated::execute_json(&program),
            Err(Error::Arithmetic)
        ));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../source");
    let path = std::env::temp_dir().join(format!(
        "cuestacean-source-strings-{}.cue",
        std::process::id()
    ));
    std::fs::write(&path, "string & \"hello 🦀\"").expect("source");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
        .arg(&path)
        .output()
        .expect("CLI");
    std::fs::remove_file(path).expect("remove source");
    assert!(output.status.success(), "{output:?}");
    let value: Value = serde_json::from_slice(&output.stdout).expect("JSON scalar");
    assert_eq!(value, json!("hello 🦀"));
    assert!(output.stderr.is_empty());
    for name in ["string-unequal", "string-mixed"] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(root.join(format!("conflicts/{name}.cue")))
            .output()
            .expect("CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
