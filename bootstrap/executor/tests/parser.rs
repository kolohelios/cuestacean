use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::parser::parse;
use serde_json::Value;

#[path = "../../generated/evaluation.rs"]
mod generated;

fn bootstrap() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn supported_source_reaches_the_generated_evaluator() {
    let grammar = read(bootstrap().join("grammar.json"));
    for (name, expected) in [
        ("literal", 42),
        ("nested", 42),
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
    ] {
        let path = bootstrap().join(format!("source/valid/{name}.cue"));
        let program = parse(&read(&path), &grammar).expect(name);
        let json = serde_json::to_string(&program).expect("serialize tree");
        assert_eq!(generated::execute(&json).expect(name), expected);
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
                serde_json::from_str::<Value>(&read(bootstrap().join("addition.json")))
                    .expect("existing tree")
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
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("conflicting integer values"),
                "{path:?}: {output:?}"
            );
        }
    }
    for file in ["int", "int-pair", "int-addition", "int-product"] {
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

#[test]
fn grammar_data_controls_associativity_and_constructor_names() {
    let grammar = read(bootstrap().join("grammar.json"));
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
    let grammar = read(bootstrap().join("grammar.json"));
    let normal = parse(&source, &grammar).expect("normal precedence");
    assert_eq!(normal["entry"]["kind"], "add");
    assert_eq!(normal["entry"]["right"]["kind"], "multiply");
    assert_eq!(
        normal,
        serde_json::from_str::<Value>(&read(bootstrap().join("multiplication.json")))
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
    let grammar = read(bootstrap().join("grammar.json"));
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
fn runtime_grammar_validation_and_resource_limits_are_explicit() {
    let original: Value =
        serde_json::from_str(&read(bootstrap().join("grammar.json"))).expect("grammar");
    let source = read(bootstrap().join("source/valid/literal.cue"));
    for (pointer, value) in [
        ("/formatVersion", 2.into()),
        ("/integer/valueField", "kind".into()),
        ("/atoms/0/token", "".into()),
        ("/atoms/0/token", "1int".into()),
        ("/atoms/0/kind", "".into()),
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
