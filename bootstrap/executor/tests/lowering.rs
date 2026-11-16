use std::{fs, path::Path};

use bootstrap_executor::{generate, generate_with_policy, lowering::Policy};
use serde_json::{Value, json};

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn policy(cases: Value) -> Policy {
    Policy::parse(&json!({"formatVersion": 1, "cases": cases}).to_string()).expect("policy")
}

fn lower(policy: &Policy, instruction: Value) -> String {
    policy.lower(&instruction).expect("lower instruction")
}

#[test]
fn default_policy_matches_independent_instruction_expressions() {
    let policy = Policy::parse(&export("lowering")).expect("default policy");
    let field = json!({"op":"field", "field":"value"});
    let left = json!({"op":"evaluate", "field":"left"});
    let right = json!({"op":"evaluate", "field":"right"});
    let cases = [
        (
            json!({"op":"int"}),
            "Ok::<_, Error>(runtime::Value::Int)".to_owned(),
        ),
        (
            json!({"op":"bool"}),
            "Ok::<_, Error>(runtime::Value::Bool)".to_owned(),
        ),
        (
            json!({"op":"boolean", "value":true}),
            "Ok::<_, Error>(runtime::Value::Boolean(true))".to_owned(),
        ),
        (
            json!({"op":"boolean", "value":false}),
            "Ok::<_, Error>(runtime::Value::Boolean(false))".to_owned(),
        ),
        (
            field.clone(),
            "runtime::integer(node, \"value\")".to_owned(),
        ),
        (
            left.clone(),
            "evaluate(runtime::field(node, \"left\")?)".to_owned(),
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(lower(&policy, input), expected);
    }
    for (op, function) in [
        ("lowerBound", "lower_bound"),
        ("upperBound", "upper_bound"),
        ("negate", "negate"),
    ] {
        let input = json!({"op":op, "value":field});
        assert_eq!(
            lower(&policy, input),
            format!("runtime::{function}((runtime::integer(node, \"value\"))?)")
        );
    }
    for (op, function) in [
        ("sum", "sum"),
        ("difference", "difference"),
        ("product", "product"),
        ("divide", "divide"),
        ("modulo", "modulo"),
        ("quotient", "quotient"),
        ("remainder", "remainder"),
        ("unify", "unify"),
        ("equal", "equal"),
        ("notEqual", "not_equal"),
    ] {
        let input = json!({"op":op, "left":left, "right":right});
        assert_eq!(
            lower(&policy, input),
            format!(
                "runtime::{function}((evaluate(runtime::field(node, \"left\")?))?, (evaluate(runtime::field(node, \"right\")?))?)"
            )
        );
    }
}

#[test]
fn strings_are_rust_debug_literals_not_json_or_source() {
    let policy = Policy::parse(&export("lowering")).expect("default policy");
    for field in ["quote\"slash\\newline\n", "café 🦀", "\0\t\r"] {
        let input = json!({"op":"field", "field":field});
        assert_eq!(
            lower(&policy, input),
            format!("runtime::integer(node, {field:?})")
        );
    }
}

#[test]
fn every_template_form_composes_and_selects_the_current_instruction() {
    let policy = policy(json!({
        "outer": {"op":"call", "path":"module::f", "args":[
            {"op":"path", "value":"node"},
            {"op":"ok", "value":{"op":"group", "value":{"op":"try", "value":{"op":"lower", "field":"child"}}}},
            {"op":"string", "field":"name"},
            {"op":"boolean", "field":"flag"}
        ]},
        "leaf": {"op":"path", "value":"module::leaf"}
    }));
    assert_eq!(
        lower(
            &policy,
            json!({"op":"outer", "child":{"op":"leaf"}, "name":"x", "flag":false})
        ),
        "module::f(node, Ok::<_, Error>((module::leaf?)), \"x\", false)"
    );
}

#[test]
fn alternate_policy_changes_generation_not_rule_data() {
    let rules = export("evaluation");
    let default = export("lowering");
    let alternate = export("tests/valid/lowering-left-twice");
    let alternate_rules = export("tests/valid/evaluation-left-twice");
    let mut expected: Value = serde_json::from_str(&default).expect("default policy");
    expected["cases"]["sum"]["args"][1]["value"]["value"]["field"] = json!("left");
    assert_eq!(
        serde_json::from_str::<Value>(&alternate).expect("alternate policy"),
        expected
    );
    assert_eq!(
        generate_with_policy(&rules, &default).expect("default"),
        generate(&rules).expect("generate")
    );
    let changed = generate_with_policy(&rules, &alternate).expect("alternate");
    assert_eq!(
        changed,
        generate(&alternate_rules).expect("alternate rules")
    );
    assert_ne!(changed, generate(&rules).expect("default rules"));
}

#[test]
fn case_order_and_repeated_calls_do_not_change_output() {
    let first = r#"{"formatVersion":1,"cases":{"a":{"op":"path","value":"one"},"b":{"op":"path","value":"two"}}}"#;
    let second = r#"{"cases":{"b":{"value":"two","op":"path"},"a":{"value":"one","op":"path"}},"formatVersion":1}"#;
    let a = Policy::parse(first).expect("first");
    let b = Policy::parse(second).expect("second");
    for op in ["a", "b"] {
        let input = json!({"op":op});
        assert_eq!(lower(&a, input.clone()), lower(&b, input.clone()));
        assert_eq!(lower(&a, input.clone()), lower(&a, input));
    }
}

#[test]
fn rejects_invalid_policies_including_unused_templates_and_duplicates() {
    let invalid = [
        r#"{}"#,
        r#"{"formatVersion":2,"cases":{"a":{"op":"path","value":"x"}}}"#,
        r#"{"formatVersion":1,"cases":{}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"path","value":"x"}},"extra":0}"#,
        r#"{"formatVersion":1,"formatVersion":1,"cases":{"a":{"op":"path","value":"x"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"path","value":"x"},"a":{"op":"path","value":"y"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"path","op":"path","value":"x"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"path","value":"x","extra":0}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"unknown"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"call","path":"f","args":[{"op":"path","value":"x"},{"op":"unknown"}]}}}"#,
        r#"{"formatVersion":1,"cases":{"good":{"op":"path","value":"x"},"unused":{"op":"path","value":"bad path"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"lower","field":""}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"string"}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"boolean","field":42}}}"#,
        r#"{"formatVersion":1,"cases":{"a":{"op":"call","path":"f","args":"x"}}}"#,
    ];
    for input in invalid {
        assert!(Policy::parse(input).is_err(), "accepted {input}");
    }
    for path in ["", "::a", "a::", "a::::b", "1abc", "a-b", "a()", "é", "a b"] {
        let input = json!({"formatVersion":1,"cases":{"x":{"op":"path","value":path}}}).to_string();
        assert!(Policy::parse(&input).is_err(), "accepted path {path:?}");
    }
    for field in ["".to_owned(), "a".repeat(129)] {
        let input =
            json!({"formatVersion":1,"cases":{"x":{"op":"string","field":field}}}).to_string();
        assert!(
            Policy::parse(&input).is_err(),
            "accepted selector {field:?}"
        );
    }
}

#[test]
fn rejects_malformed_instruction_data_and_missing_cases() {
    let p = policy(json!({
        "s":{"op":"string","field":"value"},
        "b":{"op":"boolean","field":"value"},
        "child":{"op":"lower","field":"value"}
    }));
    for input in [
        json!(null),
        json!([]),
        json!({}),
        json!({"op":1}),
        json!({"op":"missing"}),
        json!({"op":"s"}),
        json!({"op":"s","value":true}),
        json!({"op":"b","value":"true"}),
        json!({"op":"child","value":null}),
        json!({"op":"child","value":{"op":"missing"}}),
    ] {
        assert!(p.lower(&input).is_err(), "accepted {input}");
    }
}

#[test]
fn exact_policy_boundaries_count_root_and_shared_runtime_accepts_finite_work() {
    let wrap = |expr| json!({"formatVersion":1,"cases":{"x":expr}}).to_string();
    let mut expr = json!({"op":"path","value":"x"});
    for _ in 0..28 {
        expr = json!({"op":"group","value":expr});
    }
    assert!(Policy::parse(&wrap(expr.clone())).is_ok()); // 32 levels including root.
    assert!(Policy::parse(&wrap(json!({"op":"group","value":expr}))).is_err());
    let mut args = vec![json!({"op":"path","value":"x"}); 168]; // 511 nodes.
    args[0] = json!({"op":"call","path":"f","args":[]}); // 512 nodes.
    assert!(Policy::parse(&wrap(json!({"op":"call","path":"f","args":args}))).is_ok());
    args[1] = json!({"op":"call","path":"f","args":[]}); // 513 nodes.
    assert!(Policy::parse(&wrap(json!({"op":"call","path":"f","args":args}))).is_err());
    let cases: serde_json::Map<String, Value> = (0..64)
        .map(|i| (format!("case{i}"), json!({"op":"path","value":"x"})))
        .collect();
    assert!(Policy::parse(&json!({"formatVersion":1,"cases":cases}).to_string()).is_ok());
    let p = policy(
        json!({"next":{"op":"call","path":"f","args":[{"op":"lower","field":"child"},{"op":"lower","field":"child"}]},"leaf":{"op":"path","value":"x"}}),
    );
    let mut node = json!({"op":"leaf"});
    for _ in 0..9 {
        node = json!({"op":"next","child":node});
    }
    assert!(p.lower(&node).is_ok());
    let p = policy(json!({"s":{"op":"string","field":"value"}}));
    assert_eq!(
        p.lower(&json!({"op":"s","value":"x".repeat(65536)}))
            .expect("64 KiB accepted")
            .len(),
        65538
    );
}

#[test]
fn cli_policy_override_and_failures_never_print_partial_rust() {
    use std::process::Command;
    let exports = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let rules = exports.join("evaluation.json");
    let alternate = exports.join("tests/valid/lowering-left-twice.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-codegen");
    let output = Command::new(binary)
        .arg(&rules)
        .arg(&alternate)
        .output()
        .expect("policy CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        generate_with_policy(
            &export("evaluation"),
            &export("tests/valid/lowering-left-twice")
        )
        .expect("source")
        .as_bytes()
    );
    assert!(output.stderr.is_empty());
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/invalid.json");
    for args in [
        vec![rules.as_path(), Path::new("missing-lowering")],
        vec![rules.as_path(), invalid.as_path()],
        vec![rules.as_path(), alternate.as_path(), Path::new("extra")],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("invalid CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let invalid = r#"{"formatVersion":1,"rules":{"x":{"result":{"op":"boolean","value":"true"}}}}"#;
    let ignores_data =
        json!({"formatVersion":1,"cases":{"boolean":{"op":"path","value":"x"}}}).to_string();
    assert!(matches!(
        generate_with_policy(invalid, &ignores_data),
        Err(bootstrap_executor::Error::Json {
            document: "rules",
            ..
        })
    ));
}

#[test]
fn policy_size_depth_and_runtime_budgets_reject_excess() {
    let too_many_cases: serde_json::Map<String, Value> = (0..65)
        .map(|i| (format!("case{i}"), json!({"op":"path","value":"x"})))
        .collect();
    let text = json!({"formatVersion":1,"cases":too_many_cases}).to_string();
    assert!(Policy::parse(&text).is_err());
    let long_key = "x".repeat(65);
    let text =
        json!({"formatVersion":1,"cases":{(long_key):{"op":"path","value":"x"}}}).to_string();
    assert!(Policy::parse(&text).is_err());
    let long_path = "x".repeat(129);
    let text = json!({"formatVersion":1,"cases":{"x":{"op":"path","value":long_path}}}).to_string();
    assert!(Policy::parse(&text).is_err());
    let mut expr = json!({"op":"path","value":"x"});
    for _ in 0..40 {
        expr = json!({"op":"group","value":expr});
    }
    let text = json!({"formatVersion":1,"cases":{"x":expr}}).to_string();
    assert!(Policy::parse(&text).is_err());
    let args = vec![json!({"op":"path","value":"x"}); 513];
    let text =
        json!({"formatVersion":1,"cases":{"x":{"op":"call","path":"f","args":args}}}).to_string();
    assert!(Policy::parse(&text).is_err());

    let recursive = policy(json!({
        "next":{"op":"lower","field":"child"},
        "leaf":{"op":"path","value":"x"}
    }));
    let mut node = json!({"op":"leaf"});
    for _ in 0..65 {
        node = json!({"op":"next","child":node});
    }
    assert!(recursive.lower(&node).is_err());

    let duplicating = policy(json!({
        "next":{"op":"call","path":"f","args":[{"op":"lower","field":"child"},{"op":"lower","field":"child"}]},
        "leaf":{"op":"path","value":"x"}
    }));
    let mut node = json!({"op":"leaf"});
    for _ in 0..13 {
        node = json!({"op":"next","child":node});
    }
    assert!(
        duplicating
            .lower(&node)
            .expect_err("shared steps")
            .to_string()
            .contains("4096 steps")
    );

    let strings = policy(json!({"s":{"op":"string","field":"value"}}));
    assert!(
        strings
            .lower(&json!({"op":"s","value":"x".repeat(65537)}))
            .is_err()
    );
    let repeated = policy(json!({"s":{"op":"call","path":"f","args":
        vec![json!({"op":"string","field":"value"}); 20]}}));
    assert!(
        repeated
            .lower(&json!({"op":"s","value":"x".repeat(65536)}))
            .expect_err("shared output budget")
            .to_string()
            .contains("1 MiB")
    );
}
