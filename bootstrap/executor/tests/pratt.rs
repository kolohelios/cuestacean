use bootstrap_executor::{execute, parser, pratt};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "../../generated/pratt-invalid-state.rs"]
mod invalid_state;
#[path = "../../generated/pratt-overflow.rs"]
mod overflow;
#[path = "../../generated/pratt-right.rs"]
mod right;
#[path = "../../generated/pratt-stalled.rs"]
mod stalled;

fn source(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path))
        .expect("source")
}
fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}
fn normalized(result: Result<Value, parser::Error>) -> Result<Value, String> {
    result.map_err(|error| error.to_string())
}

#[test]
fn complete_source_corpus_agrees_between_compiled_and_interpreted_control() {
    let grammar = export("grammar");
    let control = export("pratt");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../source");
    for directory in ["valid", "rejected", "conflicts", "incomplete"] {
        for entry in std::fs::read_dir(root.join(directory)).expect("fixtures") {
            let path = entry.expect("fixture").path();
            if path.extension().is_none_or(|ext| ext != "cue") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("text");
            assert_eq!(
                normalized(parser::parse(&text, &grammar)),
                normalized(parser::parse_interpreted(&text, &grammar, &control)),
                "{path:?}"
            );
        }
    }
}

#[test]
fn alternate_cue_control_changes_associativity_and_recursive_results() {
    let grammar = export("grammar");
    let text = source("source/valid/pratt-chain.cue");
    let normal = parser::parse(&text, &grammar).expect("normal");
    let alternate = parser::parse_with_driver(&text, &grammar, right::execute).expect("alternate");
    assert_eq!(normal["entry"]["left"]["left"]["kind"], "add");
    assert_eq!(alternate["entry"]["right"]["right"]["kind"], "add");
    assert_eq!(
        alternate,
        parser::parse_interpreted(&text, &grammar, &export("tests/valid/pratt-right"))
            .expect("alternate interpreter")
    );
    let text = source("source/rejected/pratt-product-order.cue");
    let rules = export("evaluation");
    let normal = parser::parse(&text, &grammar).expect("normal tree");
    let alternate =
        parser::parse_with_driver(&text, &grammar, right::execute).expect("alternate tree");
    assert!(
        execute(&serde_json::to_string(&normal).expect("JSON"), &rules)
            .expect_err("left overflow")
            .to_string()
            .contains("overflow")
    );
    assert_eq!(
        execute(&serde_json::to_string(&alternate).expect("JSON"), &rules).expect("right product"),
        0
    );
    let mut changed: Value = serde_json::from_str(&grammar).expect("grammar");
    changed["operators"][0]["associativity"] = "right".into();
    let changed = serde_json::to_string(&changed).expect("grammar");
    assert_eq!(
        parser::parse(&source("source/valid/pratt-chain.cue"), &changed).expect("right metadata"),
        parser::parse_with_driver(
            &source("source/valid/pratt-chain.cue"),
            &grammar,
            right::execute
        )
        .expect("right algorithm")
    );
}

#[test]
fn control_modes_preserve_runtime_grammar_changes() {
    let original: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let control = export("pratt");
    for (pointer, value) in [
        ("/operators/0/associativity", "right".into()),
        ("/operators/0/precedence", 100.into()),
        ("/operators/1/precedence", 5.into()),
        ("/operators/0/kind", "alternate".into()),
        ("/operators/0/leftField", "first".into()),
    ] {
        let mut grammar = original.clone();
        *grammar.pointer_mut(pointer).expect("field") = value;
        let grammar = serde_json::to_string(&grammar).expect("JSON");
        for file in ["pratt-chain", "precedence", "unify-product", "group-nested"] {
            let text = source(&format!("source/valid/{file}.cue"));
            assert_eq!(
                normalized(parser::parse(&text, &grammar)),
                normalized(parser::parse_interpreted(&text, &grammar, &control)),
                "{pointer}, {file}"
            );
        }
    }
}

#[test]
fn generated_and_interpreted_guards_reject_stalls_bad_state_and_overflow() {
    let grammar = export("grammar");
    assert!(matches!(
        parser::parse_with_driver("1", &grammar, stalled::execute),
        Err(parser::Error::ControlBudget)
    ));
    assert!(matches!(
        parser::parse_interpreted("1", &grammar, &export("tests/valid/pratt-stalled")),
        Err(parser::Error::ControlBudget)
    ));
    assert!(matches!(
        parser::parse_with_driver("1", &grammar, invalid_state::execute),
        Err(parser::Error::ControlState { .. })
    ));
    assert!(matches!(
        parser::parse_interpreted("1", &grammar, &export("tests/valid/pratt-invalid-state")),
        Err(parser::Error::ControlState { .. })
    ));
    assert!(matches!(
        parser::parse_with_driver("1", &grammar, overflow::execute),
        Err(parser::Error::BindingPower)
    ));
    assert!(matches!(
        parser::parse_interpreted("1", &grammar, &export("tests/valid/pratt-overflow")),
        Err(parser::Error::BindingPower)
    ));
}

#[derive(Deserialize)]
struct InvalidCase {
    program: Value,
    error: String,
}
#[test]
fn validation_limits_and_compiler_cli_are_checked() {
    let grammar = export("grammar");
    let cases: Vec<InvalidCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/pratt-invalid-cases.json").expect("cases"),
    )
    .expect("decode");
    for case in cases {
        let input = serde_json::to_string(&case.program).expect("JSON");
        assert!(
            pratt::generate(&input)
                .expect_err("compiler")
                .to_string()
                .contains(&case.error)
        );
        assert!(
            parser::parse_interpreted("1", &grammar, &input)
                .expect_err("interpreter")
                .to_string()
                .contains(&case.error)
        );
    }
    let mut large: Value = serde_json::from_str(&export("pratt")).expect("program");
    let statement = large["body"][0].clone();
    large["body"] = vec![statement; 513].into();
    assert!(
        pratt::generate(&serde_json::to_string(&large).expect("JSON"))
            .expect_err("node cap")
            .to_string()
            .contains("512 JSON nodes")
    );
    let mut deep: Value = serde_json::from_str(&export("pratt")).expect("program");
    let template: Value = serde_json::from_str(&export("tests/valid/pratt-stalled")).expect("loop");
    let mut statement = deep["body"][0].clone();
    for _ in 0..17 {
        let mut wrapper = template["body"][0].clone();
        wrapper["body"] = vec![statement].into();
        statement = wrapper;
    }
    deep["body"] = vec![statement].into();
    assert!(
        pratt::generate(&serde_json::to_string(&deep).expect("JSON"))
            .expect_err("depth cap")
            .to_string()
            .contains("32 JSON levels")
    );
    let mut pending: Value = serde_json::from_str(&export("pratt")).expect("program");
    let advance = pending["body"][3]["body"]
        .as_array()
        .expect("loop body")
        .iter()
        .find(|statement| statement["op"] == "advanceOperator")
        .expect("advance")
        .clone();
    let body = pending["body"].as_array_mut().expect("body");
    body.truncate(3);
    body.push(advance);
    assert!(matches!(
        parser::parse_interpreted(
            "1+",
            &grammar,
            &serde_json::to_string(&pending).expect("JSON")
        ),
        Err(parser::Error::ControlState {
            reason: "unfinished operator at frame return"
        })
    ));
    let text = export("pratt");
    let first = pratt::generate(&text).expect("generate");
    assert_eq!(first, pratt::generate(&text).expect("repeat"));
    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("pratt.json");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-pratt-codegen"))
        .arg(path)
        .output()
        .expect("CLI");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, first.as_bytes());
    for args in [
        vec![],
        vec!["one", "two"],
        vec!["not-a-file"],
        vec!["tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-pratt-codegen"))
            .args(args)
            .output()
            .expect("CLI error");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
