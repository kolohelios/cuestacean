use std::{fs, path::Path, process::Command};

use bootstrap_executor::{
    compiled_lowering,
    lowering::{self, Policy},
};
use serde_json::{Value, json};

#[path = "../../generated/lowering-left-twice.rs"]
mod alternate;
#[path = "../../generated/lowering-escaped.rs"]
mod escaped;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn compare(policy: &Policy, compiled: fn(&Value) -> Result<String, lowering::Error>, node: &Value) {
    let expected = policy.lower(node).map_err(|e| e.to_string());
    let actual = compiled(node).map_err(|e| e.to_string());
    assert_eq!(actual, expected, "instruction: {node}");
}

#[test]
fn compiled_corpus_and_rule_results_match_reference() {
    let default = Policy::parse(&export("lowering")).expect("default policy");
    let changed =
        Policy::parse(&export("tests/valid/lowering-left-twice")).expect("alternate policy");
    let field = json!({"op":"field", "field":"quote\"\\\n雪"});
    let evaluate = json!({"op":"evaluate", "field":"right"});
    let mut corpus = vec![
        json!({"op":"int"}),
        json!({"op":"bool"}),
        json!({"op":"boolean", "value":true}),
        json!({"op":"boolean", "value":false}),
        field.clone(),
        evaluate.clone(),
    ];
    for op in ["lowerBound", "upperBound", "negate"] {
        corpus.push(json!({"op":op, "value":field}));
    }
    for op in [
        "sum",
        "difference",
        "product",
        "divide",
        "modulo",
        "quotient",
        "remainder",
        "unify",
    ] {
        corpus.push(json!({"op":op, "left":field, "right":evaluate}));
    }
    corpus.push(json!({"op":"product", "left":{"op":"sum", "left":field, "right":evaluate}, "right":{"op":"negate", "value":field}}));
    let rules: Value = serde_json::from_str(&export("evaluation")).expect("rules JSON");
    for rule in rules["rules"].as_object().expect("rule map").values() {
        corpus.push(rule["result"].clone());
    }
    for node in &corpus {
        compare(&default, compiled_lowering::lower, node);
        compare(&changed, alternate::lower, node);
    }
    assert_eq!(
        compiled_lowering::lower(&json!({"op":"int"})).expect("int"),
        "Ok::<_, Error>(runtime::Value::Int)"
    );
    assert_eq!(
        compiled_lowering::lower(&json!({"op":"boolean", "value":false})).expect("boolean"),
        "Ok::<_, Error>(runtime::Value::Boolean(false))"
    );
}

#[test]
fn escaped_policy_keys_selectors_and_empty_calls_compile_and_render() {
    let policy = Policy::parse(&export("tests/valid/lowering-escaped")).expect("escaped policy");
    let text = json!({"op":"text\"\\\nλ", "payload\"\\\nλ":"quote\"\\\n雪"});
    for (node, expected) in [
        (text.clone(), "\"quote\\\"\\\\\\n雪\""),
        (json!({"op":"empty"}), "module::empty()"),
        (
            json!({"op":"outer", "child\"\\\nλ":text, "flag\"\\\nλ":false}),
            "module::record(\"quote\\\"\\\\\\n雪\", false)",
        ),
    ] {
        assert_eq!(escaped::lower(&node).expect("compiled escaping"), expected);
        compare(&policy, escaped::lower, &node);
    }
}

#[test]
fn alternate_driver_propagates_through_nested_children() {
    let node = json!({"op":"product", "left":{"op":"sum", "left":{"op":"field", "field":"L"}, "right":{"op":"field", "field":"R"}}, "right":{"op":"field", "field":"outside"}});
    let normal = compiled_lowering::lower(&node).expect("normal");
    let changed = alternate::lower(&node).expect("alternate");
    assert_eq!(
        normal,
        "runtime::product((runtime::sum((runtime::integer(node, \"L\"))?, (runtime::integer(node, \"R\"))?))?, (runtime::integer(node, \"outside\"))?)"
    );
    assert_eq!(
        changed,
        "runtime::product((runtime::sum((runtime::integer(node, \"L\"))?, (runtime::integer(node, \"L\"))?))?, (runtime::integer(node, \"outside\"))?)"
    );
}

#[test]
fn compiled_errors_and_shared_resource_limits_match() {
    let default = Policy::parse(&export("lowering")).expect("default policy");
    let changed =
        Policy::parse(&export("tests/valid/lowering-left-twice")).expect("alternate policy");
    let mut nodes = vec![
        json!(null),
        json!([]),
        json!({}),
        json!({"op":1}),
        json!({"op":"missing"}),
        json!({"op":"field"}),
        json!({"op":"field", "field":false}),
        json!({"op":"boolean", "value":"true"}),
        json!({"op":"sum"}),
        json!({"op":"negate", "value":null}),
        json!({"op":"negate", "value":{"op":"missing"}}),
        json!({"op":"field", "field":"x".repeat(65536)}),
        json!({"op":"field", "field":"x".repeat(65537)}),
    ];
    let mut deep = json!({"op":"int"});
    for _ in 0..70 {
        deep = json!({"op":"negate", "value":deep});
    }
    nodes.push(deep);
    let mut work = json!({"op":"int"});
    for _ in 0..13 {
        work = json!({"op":"sum", "left":work, "right":{"op":"int"}});
    }
    assert!(
        changed
            .lower(&work)
            .expect_err("work limit")
            .to_string()
            .contains("4096 steps")
    );
    nodes.push(work);
    let mut bytes = json!({"op":"field", "field":"x".repeat(65536)});
    for _ in 0..5 {
        bytes = json!({"op":"sum", "left":bytes, "right":{"op":"int"}});
    }
    assert!(
        changed
            .lower(&bytes)
            .expect_err("byte limit")
            .to_string()
            .contains("1 MiB")
    );
    nodes.push(bytes);
    for node in &nodes {
        compare(&default, compiled_lowering::lower, node);
        compare(&changed, alternate::lower, node);
    }
}

#[test]
fn compiler_is_deterministic_and_rejects_invalid_whole_policies() {
    let first = r#"{"formatVersion":1,"cases":{"a":{"op":"path","value":"one"},"b":{"op":"path","value":"two"}}}"#;
    let reordered = r#"{"cases":{"b":{"value":"two","op":"path"},"a":{"value":"one","op":"path"}},"formatVersion":1}"#;
    let source = lowering::generate(first).expect("compile");
    assert_eq!(source, lowering::generate(first).expect("repeat"));
    assert_eq!(source, lowering::generate(reordered).expect("reordered"));
    for invalid in [
        "{}",
        r#"{"formatVersion":2,"cases":{"x":{"op":"path","value":"x"}}}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x"},"x":{"op":"path","value":"y"}}}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x"},"unused":{"op":"path","value":"bad path"}}}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"call","path":"f","args":[{"op":"unknown"}]}}}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"lower","field":""}}}"#,
    ] {
        assert!(lowering::generate(invalid).is_err(), "accepted {invalid}");
    }
    for path in ["", "::a", "a::", "a::::b", "1abc", "a-b", "a()", "é", "a b"] {
        let text = json!({"formatVersion":1,"cases":{"x":{"op":"path","value":path}}}).to_string();
        assert!(lowering::generate(&text).is_err(), "accepted {path:?}");
    }
}

#[test]
fn compiler_cli_matches_api_and_never_prints_partial_output() {
    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("lowering.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-lower-codegen");
    let output = Command::new(binary)
        .arg(&path)
        .output()
        .expect("compiler CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        lowering::generate(&export("lowering"))
            .expect("compile")
            .as_bytes()
    );
    assert!(output.stderr.is_empty());
    for args in [
        vec![],
        vec!["missing-lowering"],
        vec!["tests/fixtures/invalid.json"],
        vec!["one", "two"],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("failing CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
