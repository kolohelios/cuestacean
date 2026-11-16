use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::{Error, execute, execute_json, parser, runtime};
use runtime::Value as V;
use serde_json::{Value, json};

use bootstrap_executor::{generated, unification};

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn source(category: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/{category}/booleans-{name}.cue"))
}

fn atom(kind: &str) -> Value {
    json!({"kind": kind})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind": kind, "left": left, "right": right})
}

fn program(entry: Value) -> Value {
    json!({"formatVersion": 1, "entry": entry})
}

fn both(tree: &Value) -> [Result<Value, Error>; 2] {
    let text = tree.to_string();
    [
        execute_json(&text, &export("evaluation")),
        generated::execute_json(&text),
    ]
}

#[test]
fn boolean_meet_truth_table_and_mixed_domains() {
    use V::{Bool, Boolean};
    let cases = [
        (Bool, Bool, Some(Bool)),
        (Bool, Boolean(false), Some(Boolean(false))),
        (Bool, Boolean(true), Some(Boolean(true))),
        (Boolean(false), Bool, Some(Boolean(false))),
        (Boolean(true), Bool, Some(Boolean(true))),
        (Boolean(false), Boolean(false), Some(Boolean(false))),
        (Boolean(true), Boolean(true), Some(Boolean(true))),
        (Boolean(false), Boolean(true), None),
        (Boolean(true), Boolean(false), None),
    ];
    for (left, right, expected) in cases {
        let result = runtime::unify(left.clone(), right.clone());
        assert_eq!(
            unification::interpret(&export("unify"), left, right).map_err(|e| e.to_string()),
            result.as_ref().cloned().map_err(|e| e.to_string())
        );
        if let Some(expected) = expected {
            assert_eq!(result.expect("boolean meet"), expected);
        } else {
            assert!(matches!(result, Err(Error::BooleanConflict { .. })));
        }
    }
    assert!(matches!(
        runtime::unify(Boolean(true), Boolean(false)),
        Err(Error::BooleanConflict {
            left: true,
            right: false
        })
    ));
    let numbers = [
        V::Int,
        V::Integer(0),
        V::Bounds {
            lower: Some(0),
            upper: None,
        },
    ];
    for b in [Bool, Boolean(false), Boolean(true)] {
        for n in numbers.iter().cloned() {
            for (left, right) in [(b.clone(), n.clone()), (n, b.clone())] {
                assert!(matches!(
                    runtime::unify(left.clone(), right.clone()),
                    Err(Error::TypeConflict)
                ));
                assert!(matches!(
                    unification::interpret(&export("unify"), left, right),
                    Err(Error::TypeConflict)
                ));
            }
        }
    }
    let samples = [
        Bool,
        Boolean(false),
        Boolean(true),
        V::Int,
        V::Integer(0),
        V::Integer(1),
        numbers[2].clone(),
    ];
    for a in &samples {
        for b in &samples {
            assert_eq!(
                runtime::unify(a.clone(), b.clone()).ok(),
                runtime::unify(b.clone(), a.clone()).ok()
            );
            for c in &samples {
                assert_eq!(
                    runtime::unify(a.clone(), b.clone())
                        .and_then(|v| runtime::unify(v, c.clone()))
                        .ok(),
                    runtime::unify(b.clone(), c.clone())
                        .and_then(|v| runtime::unify(a.clone(), v))
                        .ok(),
                    "{a:?}, {b:?}, {c:?}"
                );
            }
        }
    }
}

#[test]
fn output_boundaries_are_exact_and_keep_legacy_integer_api() {
    for (value, expected) in [
        (V::Boolean(true), json!(true)),
        (V::Boolean(false), json!(false)),
        (V::Integer(i64::MIN), json!(i64::MIN)),
        (V::Integer(i64::MAX), json!(i64::MAX)),
    ] {
        assert_eq!(
            runtime::concrete_json(value).expect("JSON scalar"),
            expected
        );
    }
    assert!(matches!(
        runtime::concrete(V::Boolean(true)),
        Err(Error::IntegerOutput)
    ));
    assert!(matches!(
        runtime::concrete(V::Bool),
        Err(Error::IncompleteBool)
    ));
    assert!(matches!(
        runtime::concrete_json(V::Bool),
        Err(Error::IncompleteBool)
    ));
    assert!(matches!(
        runtime::concrete_json(V::Int),
        Err(Error::Incomplete)
    ));
    assert!(matches!(
        runtime::concrete_json(V::Bounds {
            lower: Some(1),
            upper: Some(1)
        }),
        Err(Error::IncompleteBounds)
    ));
    let tree = program(atom("true")).to_string();
    assert!(matches!(
        execute(&tree, &export("evaluation")),
        Err(Error::IntegerOutput)
    ));
    assert!(matches!(
        generated::execute(&tree),
        Err(Error::IntegerOutput)
    ));
    let abstract_tree = program(atom("boolType")).to_string();
    assert!(matches!(
        execute(&abstract_tree, &export("evaluation")),
        Err(Error::IncompleteBool)
    ));
    assert!(matches!(
        generated::execute(&abstract_tree),
        Err(Error::IncompleteBool)
    ));
}

#[test]
fn source_trees_rules_and_scalar_execution() {
    let grammar = export("grammar");
    for text in ["truex", "false1", "bool_name", "trueλ"] {
        assert!(parser::parse(text, &grammar).is_err(), "{text}");
        assert!(
            parser::parse_interpreted(text, &grammar, &export("pratt")).is_err(),
            "{text}"
        );
    }
    for (text, expected, output) in [
        ("true", atom("true"), json!(true)),
        ("false", atom("false"), json!(false)),
        (
            "bool & true",
            binary("unify", atom("boolType"), atom("true")),
            json!(true),
        ),
        (
            "false & bool",
            binary("unify", atom("false"), atom("boolType")),
            json!(false),
        ),
    ] {
        let expected = program(expected);
        assert_eq!(parser::parse(text, &grammar).expect("parse"), expected);
        assert_eq!(
            parser::parse_interpreted_with_primary(
                text,
                &grammar,
                &export("pratt"),
                &export("primary")
            )
            .expect("reference parse"),
            expected
        );
        for result in both(&expected) {
            assert_eq!(result.expect("evaluate"), output);
        }
    }
    for result in both(&program(atom("boolType"))) {
        assert!(matches!(result, Err(Error::IncompleteBool)));
    }
    let mut grammar: Value = serde_json::from_str(&grammar).expect("grammar JSON");
    grammar["atoms"]
        .as_array_mut()
        .expect("atoms")
        .push(json!({"token":"yes", "kind":"true"}));
    assert_eq!(
        parser::parse("yes", &grammar.to_string()).expect("alias"),
        program(atom("true"))
    );
    let mut rules: Value = serde_json::from_str(&export("evaluation")).expect("rules JSON");
    rules["rules"]["true"]["result"]["value"] = json!(false);
    assert_eq!(
        execute_json(&program(atom("true")).to_string(), &rules.to_string()).expect("changed rule"),
        json!(false)
    );
    assert_eq!(
        generated::execute_json(&program(atom("true")).to_string()).expect("unchanged generated"),
        json!(true)
    );
}

#[test]
fn children_fail_left_first_before_arithmetic_checks() {
    let bool_conflict = binary("unify", atom("true"), atom("false"));
    let type_conflict = binary("unify", atom("true"), json!({"kind":"integer", "value":0}));
    for (left, right, expected) in [
        (
            bool_conflict.clone(),
            type_conflict.clone(),
            "conflicting boolean values true and false",
        ),
        (type_conflict, bool_conflict, "conflicting value types"),
        (
            atom("true"),
            binary("unify", atom("false"), atom("true")),
            "conflicting boolean values false and true",
        ),
    ] {
        for result in both(&program(binary("divide", left, right))) {
            assert_eq!(result.expect_err("child error").to_string(), expected);
        }
    }
    for result in both(&program(binary(
        "add",
        atom("true"),
        json!({"kind":"integer", "value":1}),
    ))) {
        assert!(matches!(result, Err(Error::Arithmetic)));
    }
}

#[test]
fn source_and_json_clis_emit_scalar_json() {
    for (name, expected) in [
        ("true", "true\n"),
        ("false", "false\n"),
        ("refinement", "true\n"),
        ("comments", "false\n"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source("valid", name))
            .output()
            .expect("source CLI");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    for (category, name, message) in [
        ("conflicts", "opposite", "conflicting boolean values"),
        ("conflicts", "type", "conflicting value types"),
        ("conflicts", "bounds", "conflicting value types"),
        ("incomplete", "type", "non-concrete bool value"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source(category, name))
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{output:?}"
        );
    }
    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("booleans.json");
    let interpreted = Command::new(env!("CARGO_BIN_EXE_bootstrap-executor"))
        .arg(&path)
        .arg(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("evaluation.json"))
        .output()
        .expect("interpreter CLI");
    let compiled = Command::new(env!("CARGO_BIN_EXE_bootstrap-generated"))
        .arg(&path)
        .output()
        .expect("generated CLI");
    assert!(interpreted.status.success(), "{interpreted:?}");
    assert_eq!(interpreted.stdout, b"true\n");
    assert!(interpreted.stderr.is_empty());
    assert_eq!(compiled.status.code(), interpreted.status.code());
    assert_eq!(compiled.stdout, interpreted.stdout);
    assert_eq!(compiled.stderr, interpreted.stderr);
}
