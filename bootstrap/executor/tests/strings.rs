use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use bootstrap_executor::{Error, execute, execute_json, generated, parser, runtime, unification};
use runtime::Value as V;
use serde_json::{Value, json};

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn text(value: &str) -> V {
    V::Text(Arc::from(value))
}

fn string(value: &str) -> Value {
    json!({"kind":"string", "value":value})
}

fn atom(kind: &str) -> Value {
    json!({"kind":kind})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind, "left":left, "right":right})
}

fn program(entry: Value) -> String {
    json!({"formatVersion":1, "entry":entry}).to_string()
}

fn both(entry: Value) -> [Result<Value, Error>; 2] {
    let tree = program(entry);
    [
        execute_json(&tree, &export("evaluation")),
        generated::execute_json(&tree),
    ]
}

#[test]
fn string_values_are_owned_and_round_trip_without_normalization() {
    for value in [
        "",
        "hello",
        "é e\u{301} 🦀",
        "quote\" slash\\ newline\n tab\t nul\0",
    ] {
        for result in both(string(value)) {
            assert_eq!(result.expect("JSON string"), json!(value));
        }
        let node = json!({"value":value});
        let owned = runtime::text(&node, "value").expect("text field");
        drop(node);
        assert_eq!(
            runtime::concrete_json(owned).expect("owned output"),
            json!(value)
        );
    }
    let value = text("owned");
    let cloned = value.clone();
    if let (V::Text(a), V::Text(b)) = (&value, &cloned) {
        assert!(Arc::ptr_eq(a, b));
    } else {
        panic!("expected text values");
    }
    assert!(
        matches!(runtime::text(&json!({}),"value"),Err(Error::MissingField{field}) if field=="value")
    );
    for node in [json!({"value":7}), json!({"value":null})] {
        assert!(
            matches!(runtime::text(&node, "value"), Err(Error::Text { field }) if field == "value")
        );
        for result in both(json!({"kind":"string", "value":node.get("value")})) {
            assert!(matches!(result, Err(Error::Text { field }) if field == "value"));
        }
    }
    for result in both(atom("string")) {
        assert!(matches!(result, Err(Error::MissingField { field }) if field == "value"));
    }
}

#[test]
fn string_unification_has_content_equality_and_domain_boundaries() {
    let control = export("unify");
    let equal_a = text("same 🦀");
    let equal_b = text("same 🦀");
    if let (V::Text(a), V::Text(b)) = (&equal_a, &equal_b) {
        assert!(!Arc::ptr_eq(a, b));
    }
    for (left, right, expected) in [
        (V::String, V::String, V::String),
        (V::String, equal_a.clone(), equal_a.clone()),
        (equal_a.clone(), V::String, equal_a.clone()),
        (equal_a.clone(), equal_b, equal_a),
    ] {
        assert_eq!(
            runtime::unify(left.clone(), right.clone()).expect("compiled"),
            expected
        );
        assert_eq!(
            unification::interpret(&control, left, right).expect("interpreted"),
            expected
        );
    }
    let left = "line\n\"";
    let right = "other\\\0";
    let expected = format!("conflicting string values {left:?} and {right:?}");
    for result in [
        runtime::unify(text(left), text(right)),
        unification::interpret(&control, text(left), text(right)),
    ] {
        assert_eq!(result.expect_err("conflict").to_string(), expected);
    }
    for result in [
        runtime::unify(text("é"), text("e\u{301}")),
        unification::interpret(&control, text("é"), text("e\u{301}")),
    ] {
        assert!(matches!(result, Err(Error::StringConflict { .. })));
    }
    for string in [V::String, text("x")] {
        for other in [
            V::Int,
            V::Integer(0),
            V::Bounds {
                lower: Some(0),
                upper: None,
            },
            V::Bool,
            V::Boolean(true),
        ] {
            for (left, right) in [(string.clone(), other.clone()), (other, string.clone())] {
                assert!(matches!(
                    runtime::unify(left.clone(), right.clone()),
                    Err(Error::TypeConflict)
                ));
                assert!(matches!(
                    unification::interpret(&control, left, right),
                    Err(Error::TypeConflict)
                ));
            }
        }
    }
}

#[test]
fn evaluator_string_semantics_and_errors() {
    for entry in [
        binary("unify", atom("stringType"), string("x")),
        binary("unify", string("x"), atom("stringType")),
        binary("unify", string("x"), string("x")),
    ] {
        for result in both(entry) {
            assert_eq!(result.expect("string meet"), json!("x"));
        }
    }
    for result in both(binary("unify", string("x"), string("y"))) {
        assert_eq!(
            result.expect_err("conflict").to_string(),
            "conflicting string values \"x\" and \"y\""
        );
    }
    for result in both(atom("stringType")) {
        assert!(matches!(result, Err(Error::IncompleteString)));
    }
    assert!(matches!(
        runtime::concrete_json(V::String),
        Err(Error::IncompleteString)
    ));
    assert!(matches!(
        runtime::concrete(text("x")),
        Err(Error::IntegerOutput)
    ));
    let tree = program(string("x"));
    assert!(matches!(
        execute(&tree, &export("evaluation")),
        Err(Error::IntegerOutput)
    ));
    assert!(matches!(
        generated::execute(&tree),
        Err(Error::IntegerOutput)
    ));
    for entry in [
        binary("add", string("x"), json!({"kind":"integer","value":1})),
        binary(
            "multiply",
            atom("stringType"),
            json!({"kind":"integer","value":1}),
        ),
    ] {
        for result in both(entry) {
            assert!(matches!(result, Err(Error::Arithmetic)));
        }
    }
    let conflict = binary("unify", string("a"), string("b"));
    let other = binary("unify", atom("true"), atom("false"));
    for (left, right, expected) in [
        (
            conflict.clone(),
            other.clone(),
            "conflicting string values \"a\" and \"b\"",
        ),
        (other, conflict, "conflicting boolean values true and false"),
    ] {
        for result in both(binary("add", left, right)) {
            assert_eq!(result.expect_err("child error").to_string(), expected);
        }
    }
}

#[test]
fn json_clis_and_source_boundary() {
    let root = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let path = std::env::temp_dir().join(format!("cuestacean-strings-{}.json", std::process::id()));
    std::fs::write(&path, program(string("hello, strings!"))).expect("write CLI input");
    let interpreted = Command::new(env!("CARGO_BIN_EXE_bootstrap-executor"))
        .arg(&path)
        .arg(root.join("evaluation.json"))
        .output()
        .expect("interpreter CLI");
    let compiled = Command::new(env!("CARGO_BIN_EXE_bootstrap-generated"))
        .arg(&path)
        .output()
        .expect("generated CLI");
    std::fs::remove_file(&path).expect("remove CLI input");
    assert!(interpreted.status.success(), "{interpreted:?}");
    // Independently check the output, not just equality between CLIs.
    assert_eq!(interpreted.stdout, b"\"hello, strings!\"\n");
    assert!(interpreted.stderr.is_empty());
    assert_eq!(compiled.status.code(), interpreted.status.code());
    assert_eq!(compiled.stdout, interpreted.stdout);
    assert_eq!(compiled.stderr, interpreted.stderr);
    for source in ["\"hello\"", "string"] {
        assert!(
            parser::parse(source, &export("grammar")).is_err(),
            "{source}"
        );
    }
}
