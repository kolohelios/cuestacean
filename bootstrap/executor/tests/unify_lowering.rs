use std::{fs, path::Path, process::Command};

use bootstrap_executor::{
    Error, compiled_unify_lowering,
    lowering::{self, Policy},
    runtime::{self, Value},
    unification,
};
use serde_json::{Value as Json, json};

#[path = "../../generated/unify-permissive-bool.rs"]
mod permissive;
#[path = "../../generated/unify-scoped.rs"]
mod scoped;
#[path = "../../generated/lowering-select.rs"]
mod selected;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn program(result: Json) -> String {
    json!({"formatVersion":1,"result":result}).to_string()
}

#[test]
fn unified_backend_matches_reference_and_preserves_wire_inputs() {
    let backend = export("unify-lowering");
    let policy = Policy::parse(&backend).expect("policy");
    for name in [
        "unify",
        "tests/valid/unify-right",
        "tests/valid/unify-scoped",
    ] {
        let input = export(name);
        assert_eq!(
            unification::generate(&input).expect(name),
            unification::generate_with_policy(&input, &backend).expect(name)
        );
    }
    // Typed serialization disambiguates the two wire-level `input` operations.
    for (node, expected) in [
        (
            json!({"op":"inputValue","input":"left"}),
            "Ok::<_, Error>(_left.clone())",
        ),
        (
            json!({"op":"inputValue","input":"right"}),
            "Ok::<_, Error>(_right.clone())",
        ),
        (
            json!({"op":"inputEndpoint","input":"right","endpoint":"lower"}),
            "Ok::<_, Error>(crate::unification::endpoint(&_right, crate::unification::Endpoint::Lower))",
        ),
    ] {
        assert_eq!(policy.lower(&node).expect("reference"), expected);
        assert_eq!(
            compiled_unify_lowering::lower(&node).expect("compiled"),
            expected
        );
    }
    let input = program(json!({"op":"input","input":"left"}));
    let source = unification::generate(&input).expect("wire input");
    assert!(source.contains("Ok::<_, Error>(_left.clone())"));
    assert_eq!(
        source,
        unification::generate_with_policy(&input, &backend).expect("reference")
    );
    let tagged =
        json!({"op":"program", "function":"unify", "result":{"op":"inputValue", "input":"left"}});
    assert_eq!(
        policy.lower(&tagged).expect("reference program"),
        compiled_unify_lowering::lower(&tagged).expect("compiled program")
    );
}

#[test]
fn alternate_compiled_policy_changes_only_boolean_equality() {
    let backend = export("tests/valid/unify-lowering-permissive");
    let input = export("tests/valid/unify-equality-probe");
    assert!(unification::interpret(&input, Value::Boolean(true), Value::Boolean(false)).is_err());
    let source = unification::generate_with_policy(&input, &backend).expect("alternate");
    assert!(source.contains("Ok::<_, Error>(true)"));
    assert_ne!(source, unification::generate(&input).expect("default"));
    assert_eq!(
        permissive::unify(Value::Boolean(true), Value::Boolean(false)).expect("permissive"),
        Value::Boolean(true)
    );
    assert!(runtime::unify(Value::Boolean(true), Value::Boolean(false)).is_err());
    assert!(permissive::unify(Value::Integer(1), Value::Boolean(true)).is_err());
    assert!(permissive::unify(Value::Boolean(true), Value::Integer(1)).is_err());
    assert!(permissive::unify(Value::Integer(1), Value::Integer(2)).is_err());
}

#[test]
fn simultaneous_initializers_keep_outer_scope_and_validation_precedes_policy() {
    let backend = export("unify-lowering");
    let input = export("tests/valid/unify-scoped");
    let source = unification::generate_with_policy(&input, &backend).expect("scoped source");
    assert_eq!(
        source,
        unification::generate(&input).expect("compiled backend")
    );
    let left = Value::Bounds {
        lower: Some(1),
        upper: Some(5),
    };
    let expected = Value::Bounds {
        lower: Some(5),
        upper: Some(1),
    };
    assert_eq!(
        scoped::unify(left.clone(), Value::Int).expect("compiled scoped"),
        expected
    );
    assert_eq!(
        unification::interpret(&input, left, Value::Int).expect("interpreted scoped"),
        expected
    );
    let ignore =
        json!({"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}).to_string();
    for result in [
        json!({"op":"letBounds","lower":{"op":"local","endpoint":"lower"},"upper":{"op":"empty"},"result":{"op":"input","input":"left"}}),
        json!({"op":"if","condition":{"op":"same"},"then":{"op":"input","input":"left"},"else":{"op":"bounds","lower":{"op":"local","endpoint":"lower"},"upper":{"op":"empty"}}}),
        json!({"op":"input","input":"middle"}),
        json!({"op":"inputValue","input":"left"}),
        json!({"op":"bounds","lower":{"op":"inputEndpoint","input":"left","endpoint":"lower"},"upper":{"op":"empty"}}),
    ] {
        assert!(unification::generate_with_policy(&program(result), &ignore).is_err());
    }
}

#[test]
fn select_checks_all_branches_and_projects_only_selected_tag() {
    let valid = json!({"formatVersion":1,"cases":{
        "root":{"op":"select","field":"kind","cases":{
            "go":{"op":"lower","field":"child"},
            "stop":{"op":"text","value":"done"}}},
        "leaf":{"op":"string","field":"value"}
    }})
    .to_string();
    let policy = Policy::parse(&valid).expect("select policy");
    assert_eq!(
        policy
            .lower(&json!({"op":"root","kind":"stop"}))
            .expect("stop"),
        "done"
    );
    assert_eq!(
        policy
            .lower(&json!({"op":"root","kind":"go","child":{"op":"leaf","value":"a\"b"}}))
            .expect("recursive"),
        "\"a\\\"b\""
    );
    for node in [
        json!({"op":"root"}),
        json!({"op":"root","kind":1}),
        json!({"op":"root","kind":"unknown"}),
    ] {
        assert!(policy.lower(&node).is_err(), "{node}");
    }
    for invalid in [
        r#"{"formatVersion":1,"cases":{"root":{"op":"select","field":"x","cases":{"a":{"op":"text","value":"x"},"a":{"op":"text","value":"y"}}}}}"#.to_owned(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"x","cases":{}}}}).to_string(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"","cases":{"a":{"op":"text","value":"x"}}}}}).to_string(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"x","cases":{"":{"op":"text","value":"x"}}}}}).to_string(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"x","cases":{"a":{"op":"text","value":"x"},"unused":{"op":"unknown"}}}}}).to_string(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"x".repeat(129),"cases":{"a":{"op":"text","value":"x"}}}}}).to_string(),
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"x","cases":{("a".repeat(65)):{"op":"text","value":"x"}}}}}).to_string(),
    ] {
        assert!(Policy::parse(&invalid).is_err(), "{invalid}");
        assert!(lowering::generate(&invalid).is_err(), "{invalid}");
    }
}

#[test]
fn nested_select_acceptance_boundaries_are_exact() {
    let field = "f".repeat(128);
    let key = "k".repeat(64);
    let mut cases: serde_json::Map<String, Json> = (0..63)
        .map(|i| (format!("case{i}"), json!({"op":"text","value":"x"})))
        .collect();
    cases.insert(key.clone(), json!({"op":"text","value":"x"}));
    let text =
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":field,"cases":cases}}})
            .to_string();
    let policy = Policy::parse(&text).expect("64 cases, 64-byte key, 128-byte selector");
    assert_eq!(
        policy
            .lower(&json!({"op":"root",(field):key}))
            .expect("maximum selector"),
        "x"
    );
    assert!(lowering::generate(&text).is_ok());
    cases.insert("overflow".into(), json!({"op":"text","value":"x"}));
    let text =
        json!({"formatVersion":1,"cases":{"root":{"op":"select","field":"tag","cases":cases}}})
            .to_string();
    assert!(Policy::parse(&text).is_err());
    assert!(lowering::generate(&text).is_err());
}

#[test]
fn compiled_select_matches_reference_for_escaped_and_nested_cases() {
    let policy = Policy::parse(&export("tests/valid/lowering-select")).expect("select fixture");
    let root = "root\"\\\nλ";
    let kind = "kind\"\\\nλ";
    let go = "go\"\\\nλ";
    let child = "child\"\\\nλ";
    let leaf = json!({"op":"leaf", "mode":"yes"});
    let nested = json!({"op":root, (kind):go, (child):leaf});
    for (node, expected) in [
        (nested.clone(), "chosen"),
        (json!({"op":root, (kind):go, (child):nested}), "chosen"),
        (
            json!({"op":root, (kind):go, (child):{"op":"leaf", "mode":"no"}}),
            "other",
        ),
        (json!({"op":root, (kind):"fallback"}), "fallback"),
    ] {
        assert_eq!(selected::lower(&node).expect("compiled select"), expected);
        assert_eq!(policy.lower(&node).expect("reference select"), expected);
    }
    let unmatched = json!({"op":root, (kind):"unknown"});
    assert!(
        selected::lower(&unmatched)
            .expect_err("unknown tag")
            .to_string()
            .contains("missing lowering selection")
    );
    for node in [
        json!(null),
        json!({}),
        json!({"op": 1}),
        json!({"op":"missing"}),
        json!({"op":root}),
        json!({"op":root, (kind):false}),
        unmatched,
        json!({"op":root, (kind):go, (child):{"op":"leaf", "mode":"unknown"}}),
    ] {
        assert_eq!(
            selected::lower(&node).map_err(|e| e.to_string()),
            policy.lower(&node).map_err(|e| e.to_string())
        );
    }
}

#[test]
fn optional_cli_policy_and_failures_have_no_partial_output() {
    let dir = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let control = dir.join("unify.json");
    let backend = dir.join("tests/valid/unify-lowering-permissive.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-unify-codegen");
    let output = Command::new(binary)
        .arg(&control)
        .arg(&backend)
        .output()
        .expect("CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        unification::generate_with_policy(
            &export("unify"),
            &export("tests/valid/unify-lowering-permissive")
        )
        .expect("alternate")
        .as_bytes()
    );
    assert!(output.stderr.is_empty());
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/invalid.json");
    for args in [
        vec![control.as_path(), Path::new("missing-policy")],
        vec![control.as_path(), invalid.as_path()],
        vec![control.as_path(), backend.as_path(), Path::new("extra")],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("CLI failure");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
