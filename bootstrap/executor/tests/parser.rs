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
    let directory = bootstrap().join("source/rejected");
    let files: Vec<_> = std::fs::read_dir(directory)
        .expect("rejected fixtures")
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
    alternate["operators"][0]["token"] = "*".into();
    assert!(
        parse(
            &source,
            &serde_json::to_string(&alternate).expect("changed token")
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
