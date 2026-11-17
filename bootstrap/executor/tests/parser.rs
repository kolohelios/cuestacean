use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::parser::parse;
use serde::Deserialize;
use serde_json::Value;

use bootstrap_executor::generated;

fn bootstrap() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn exports() -> PathBuf {
    PathBuf::from(env!("CUESTACEAN_BOOTSTRAP_JSON"))
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn supported_source_reaches_the_generated_evaluator() {
    let grammar = read(exports().join("grammar.json"));
    for (name, expected) in [
        ("literal", 42),
        ("nested", 42),
        ("comment", 1),
        ("signed", -5),
        ("minimum", i64::MIN),
        ("maximum", i64::MAX),
        ("associativity", 6),
        ("whitespace", 0),
        ("product", 42),
        ("precedence", 7),
        ("grouped-product", 9),
        ("product-chain", 24),
        ("signed-product", -14),
        ("minimum-product", i64::MIN),
        ("zero-product", 0),
        ("double-negative", 14),
        ("unify", 3),
        ("unify-addition", 3),
        ("unify-product", 6),
        ("unify-grouped", 3),
        ("unify-chain", 3),
        ("unify-negative", -7),
        ("unify-minimum", i64::MIN),
        ("unify-maximum", i64::MAX),
        ("int-left", 3),
        ("int-right", 3),
        ("int-chain", 3),
        ("int-grouped", 4),
        ("int-product", 6),
        ("int-minimum", i64::MIN),
        ("int-maximum", i64::MAX),
        ("int-repeated", 3),
        ("bounds", 3),
        ("bounds-reverse", 3),
        ("bounds-lower-edge", 0),
        ("bounds-upper-edge", 10),
        ("bounds-negative", -7),
        ("bounds-grouped", 4),
        ("bounds-product", 6),
        ("bounds-precedence", 7),
        ("bounds-intersection", 3),
        ("bounds-refine-lower", 3),
        ("bounds-refine-upper", 3),
        ("bounds-singleton", 3),
        ("bounds-maximum", i64::MAX),
        ("bounds-minimum", i64::MIN),
        ("bounds-whitespace", -1),
        ("decimal-negative-zero", 0),
        ("decimal-double-sign", 1),
        ("decimal-sign-space", -1),
        ("decimal-sign-newline", -1),
        ("decimal-bound-negative-zero", 0),
        ("operand-trailing-newline", 3),
        ("group-nested", 21),
        ("group-spaced", 3),
        ("group-leading-newline", 3),
        ("symbol-bound", 3),
        ("pratt-chain", 10),
    ] {
        let path = bootstrap().join(format!("source/valid/{name}.cue"));
        let program = parse(&read(&path), &grammar).expect(name);
        let json = serde_json::to_string(&program).expect("serialize tree");
        assert_eq!(generated::execute(&json).expect(name), expected);
        assert_eq!(
            generated::execute_json(&json).expect(name),
            Value::from(expected)
        );
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(path)
            .output()
            .expect("source CLI");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
        if name == "nested" {
            assert_eq!(
                program,
                serde_json::from_str::<Value>(&read(exports().join("addition.json")))
                    .expect("existing tree")
            );
        } else if name == "bounds" {
            assert_eq!(
                program,
                serde_json::from_str::<Value>(&read(exports().join("bounds.json")))
                    .expect("bounds tree")
            );
        }
    }
}

#[test]
fn rejected_sources_and_cli_errors_fail_without_results() {
    let files: Vec<_> = ["source/rejected", "source/conflicts", "source/incomplete"]
        .into_iter()
        .flat_map(|directory| {
            std::fs::read_dir(bootstrap().join(directory)).expect("rejected fixtures")
        })
        .map(|entry| entry.expect("fixture").path())
        .filter(|p| p.extension().is_some_and(|x| x == "cue"))
        .collect();
    assert!(!files.is_empty());
    for path in files {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2), "{path:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{path:?}");
        assert!(!output.stderr.is_empty(), "{path:?}");
        if path
            .parent()
            .expect("fixture directory")
            .ends_with("conflicts")
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            match path.file_stem().and_then(|name| name.to_str()) {
                Some("division-zero" | "mod-zero" | "quo-zero" | "rem-zero") => {
                    assert_eq!(stderr, "integer division by zero\n", "{path:?}");
                }
                Some("booleans-opposite") => {
                    assert_eq!(stderr, "conflicting boolean values true and false\n")
                }
                Some("booleans-type" | "booleans-bounds" | "string-mixed") => {
                    assert_eq!(stderr, "conflicting value types\n")
                }
                Some("string-unequal") => {
                    assert!(
                        stderr.contains("conflicting string values"),
                        "{path:?}: {stderr}"
                    );
                }
                _ => assert!(
                    stderr.contains("conflicting integer"),
                    "{path:?}: {output:?}"
                ),
            }
        }
    }
    for file in [
        "int",
        "int-pair",
        "int-addition",
        "int-product",
        "bounds",
        "bounds-lower",
        "bounds-upper",
        "bounds-singleton",
        "bounds-maximum",
        "bounds-minimum",
        "bounds-addition",
        "bounds-product",
        "bounds-precedence",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(bootstrap().join(format!("source/incomplete/{file}.cue")))
            .output()
            .expect("incomplete source");
        assert!(String::from_utf8_lossy(&output.stderr).contains("concrete"));
    }
    for args in [vec![], vec!["not-a-file"], vec!["one", "two"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .args(args)
            .output()
            .expect("invalid invocation");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[derive(Deserialize)]
struct SourceDiagnostic {
    file: String,
    offset: usize,
    reason: String,
}

fn assert_source_diagnostics(fixture: &str) {
    let grammar = read(exports().join("grammar.json"));
    let cases: Vec<SourceDiagnostic> = serde_json::from_str(&read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture),
    ))
    .expect("diagnostics");
    for case in cases {
        let path = bootstrap().join(format!("source/rejected/{}.cue", case.file));
        let error = parse(&read(&path), &grammar).expect_err("source policy");
        match &error {
            bootstrap_executor::parser::Error::Source { offset, reason } => {
                assert_eq!(*offset, case.offset, "{}", case.file);
                assert_eq!(reason, &case.reason, "{}", case.file);
            }
            other => panic!("expected source rejection, got {other:?}"),
        }
        let expected = format!("source byte {}: {}", case.offset, case.reason);
        assert_eq!(error.to_string(), expected);
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(path)
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("{expected}\n").as_bytes());
    }
}

#[test]
fn cue_symbol_scanning_preserves_diagnostics() {
    assert_source_diagnostics("symbol-diagnostic-cases.json");
}

#[test]
fn cue_group_delimiters_preserve_diagnostics_and_error_precedence() {
    assert_source_diagnostics("group-diagnostic-cases.json");
}

#[test]
fn cue_operand_newline_policy_preserves_parser_and_cli_diagnostics() {
    assert_source_diagnostics("operand-diagnostic-cases.json");
}

#[test]
fn cue_decimal_rejections_preserve_parser_and_cli_diagnostics() {
    assert_source_diagnostics("decimal-diagnostic-cases.json");
    let grammar = read(exports().join("grammar.json"));
    assert!(matches!(
        parse(
            &read(bootstrap().join("source/rejected/range.cue")),
            &grammar
        ),
        Err(bootstrap_executor::parser::Error::Integer { offset: 0, .. })
    ));
}

#[test]
fn grammar_data_controls_associativity_and_constructor_names() {
    let grammar = read(exports().join("grammar.json"));
    let source = read(bootstrap().join("source/valid/associativity.cue"));
    let left = parse(&source, &grammar).expect("left associative");
    assert_eq!(left["entry"]["left"]["kind"], "add");
    let mut alternate: Value = serde_json::from_str(&grammar).expect("grammar");
    alternate["operators"][0]["associativity"] = "right".into();
    alternate["operators"][0]["kind"] = "alternate".into();
    alternate["operators"][0]["leftField"] = "first".into();
    alternate["operators"][0]["rightField"] = "second".into();
    let right = parse(
        &source,
        &serde_json::to_string(&alternate).expect("alternate grammar"),
    )
    .expect("right associative");
    assert_eq!(right["entry"]["kind"], "alternate");
    assert_eq!(right["entry"]["first"]["kind"], "integer");
    assert_eq!(right["entry"]["second"]["kind"], "alternate");
    // These trees test parser configuration, not the default evaluator's kinds.
    alternate["operators"][0]["token"] = "/".into();
    assert!(
        parse(
            &source,
            &serde_json::to_string(&alternate).expect("changed token")
        )
        .is_err()
    );
}

#[test]
fn precedence_comes_from_grammar_data() {
    let source = read(bootstrap().join("source/valid/precedence.cue"));
    let grammar = read(exports().join("grammar.json"));
    let normal = parse(&source, &grammar).expect("normal precedence");
    assert_eq!(normal["entry"]["kind"], "add");
    assert_eq!(normal["entry"]["right"]["kind"], "multiply");
    assert_eq!(
        normal,
        serde_json::from_str::<Value>(&read(exports().join("multiplication.json")))
            .expect("mixed tree")
    );
    let mut changed: Value = serde_json::from_str(&grammar).expect("grammar");
    changed["operators"][1]["precedence"] = 5.into();
    let changed = parse(
        &source,
        &serde_json::to_string(&changed).expect("changed grammar"),
    )
    .expect("changed precedence");
    assert_eq!(changed["entry"]["kind"], "multiply");
    assert_eq!(changed["entry"]["left"]["kind"], "add");
    assert_eq!(
        generated::execute(&serde_json::to_string(&normal).expect("program"))
            .expect("normal result"),
        7
    );
    assert_eq!(
        generated::execute(&serde_json::to_string(&changed).expect("program"))
            .expect("changed result"),
        9
    );
}

#[test]
fn atom_names_and_constructors_come_from_grammar_data() {
    let grammar = read(exports().join("grammar.json"));
    let source = read(bootstrap().join("source/rejected/atom-boundary.cue"));
    assert!(parse(&source, &grammar).is_err());
    let mut changed: Value = serde_json::from_str(&grammar).expect("grammar");
    changed["atoms"][0]["token"] = "integer".into();
    changed["atoms"][0]["kind"] = "alternateType".into();
    let program =
        parse(&source, &serde_json::to_string(&changed).expect("grammar")).expect("changed atom");
    assert_eq!(program["entry"]["left"]["kind"], "alternateType");
    let atom = changed["atoms"][0].clone();
    changed["atoms"].as_array_mut().expect("atoms").push(atom);
    assert!(
        parse(
            &source,
            &serde_json::to_string(&changed).expect("ambiguous grammar")
        )
        .is_err()
    );
}

#[test]
fn identifier_spans_are_looked_up_as_complete_configurable_atom_names() {
    let original = read(exports().join("grammar.json"));
    for (file, token) in [
        ("atom-prefix-digit", "int9"),
        ("atom-prefix-underscore", "int_1"),
        ("atom-leading-underscore", "_int"),
        ("atom-uppercase", "INT"),
        ("atom-custom", "Foo_9"),
    ] {
        let source = read(bootstrap().join(format!("source/rejected/{file}.cue")));
        assert!(
            parse(&source, &original)
                .expect_err("unknown complete atom")
                .to_string()
                .contains("unknown atom")
        );
        let mut grammar: Value = serde_json::from_str(&original).expect("grammar");
        grammar["atoms"][0]["token"] = token.into();
        let program = parse(
            &source,
            &serde_json::to_string(&grammar).expect("custom grammar"),
        )
        .expect("configured atom");
        assert_eq!(program["entry"]["left"]["kind"], "intType");
        assert_eq!(
            generated::execute(&serde_json::to_string(&program).expect("program"))
                .expect("execute"),
            3
        );
    }
}

#[test]
fn bound_tokens_and_constructors_come_from_grammar_data() {
    let source = read(bootstrap().join("source/valid/bounds-refine-lower.cue"));
    let original: Value =
        serde_json::from_str(&read(exports().join("grammar.json"))).expect("grammar");
    let mut changed = original.clone();
    changed["bounds"][0]["kind"] = "floor".into();
    changed["bounds"][0]["valueField"] = "threshold".into();
    let program =
        parse(&source, &serde_json::to_string(&changed).expect("grammar")).expect("bound");
    assert_eq!(program["entry"]["left"]["left"]["kind"], "floor");
    assert_eq!(program["entry"]["left"]["right"]["threshold"], 2);

    changed["bounds"]
        .as_array_mut()
        .expect("bounds")
        .truncate(1);
    changed["bounds"][0]["token"] = "<=".into();
    let grammar = serde_json::to_string(&changed).expect("grammar");
    assert!(parse(&source, &grammar).is_err());
    let source = read(bootstrap().join("source/valid/bounds-refine-upper.cue"));
    let program = parse(&source, &grammar).expect("changed token");
    assert_eq!(program["entry"]["left"]["left"]["kind"], "floor");
    assert_eq!(program["entry"]["left"]["left"]["threshold"], 10);
    let duplicate = changed["bounds"][0].clone();
    changed["bounds"]
        .as_array_mut()
        .expect("bounds")
        .push(duplicate);
    assert!(parse(&source, &serde_json::to_string(&changed).expect("grammar")).is_err());

    let mut legacy = original;
    legacy.as_object_mut().expect("grammar").remove("bounds");
    let grammar = serde_json::to_string(&legacy).expect("legacy grammar");
    assert!(
        parse(
            &read(bootstrap().join("source/valid/literal.cue")),
            &grammar
        )
        .is_ok()
    );
    assert!(parse(&source, &grammar).is_err());
}

#[test]
fn symbol_spans_still_require_runtime_grammar_lookup() {
    let original = read(exports().join("grammar.json"));
    let division = read(bootstrap().join("source/rejected/symbol-division.cue"));
    assert!(parse(&division, &original).is_err());
    let mut grammar: Value = serde_json::from_str(&original).expect("grammar");
    grammar["operators"][0]["token"] = "/".into();
    grammar["operators"][0]["kind"] = "alternate".into();
    grammar["operators"][0]["leftField"] = "first".into();
    grammar["operators"][0]["rightField"] = "second".into();
    let program = parse(
        &division,
        &serde_json::to_string(&grammar).expect("changed grammar"),
    )
    .expect("configured symbol");
    assert_eq!(program["entry"]["kind"], "alternate");
    assert_eq!(program["entry"]["first"]["value"], 8);
    assert_eq!(program["entry"]["second"]["value"], 2);
    // These are parser configuration tests, not CUE division semantics.
    let mut disabled: Value = serde_json::from_str(&original).expect("grammar");
    disabled["bounds"].as_array_mut().expect("bounds").clear();
    let source = read(bootstrap().join("source/valid/symbol-bound.cue"));
    match parse(
        &source,
        &serde_json::to_string(&disabled).expect("disabled bounds"),
    )
    .expect_err("unconfigured prefix")
    {
        bootstrap_executor::parser::Error::Source { offset, reason } => {
            assert_eq!(offset, 0);
            assert_eq!(reason, "expected signed decimal integer");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn runtime_grammar_validation_and_resource_limits_are_explicit() {
    let original: Value =
        serde_json::from_str(&read(exports().join("grammar.json"))).expect("grammar");
    let source = read(bootstrap().join("source/valid/literal.cue"));
    for (pointer, value) in [
        ("/formatVersion", 2.into()),
        ("/group/open", "[".into()),
        ("/group/close", "]".into()),
        ("/integer/valueField", "kind".into()),
        ("/atoms/0/token", "".into()),
        ("/atoms/0/token", "1int".into()),
        ("/atoms/0/kind", "".into()),
        ("/bounds/0/token", ">==".into()),
        ("/bounds/0/kind", "".into()),
        ("/bounds/0/valueField", "kind".into()),
        ("/operators/0/token", "++".into()),
        ("/operators/0/precedence", 0.into()),
        ("/operators/0/associativity", "center".into()),
        ("/operators/0/rightField", "left".into()),
    ] {
        let mut grammar = original.clone();
        *grammar.pointer_mut(pointer).expect("field") = value;
        assert!(
            parse(&source, &serde_json::to_string(&grammar).expect("grammar")).is_err(),
            "{pointer}"
        );
    }
    let grammar = serde_json::to_string(&original).expect("grammar");
    for (file, reason) in [("depth", "32 parsing levels"), ("nodes", "64 nodes")] {
        let source = read(bootstrap().join(format!("source/rejected/{file}.cue")));
        assert!(
            parse(&source, &grammar)
                .expect_err(file)
                .to_string()
                .contains(reason)
        );
    }
}
