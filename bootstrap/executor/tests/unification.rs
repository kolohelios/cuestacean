use std::path::Path;
use std::process::Command;

use bootstrap_executor::{Error, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/unify.rs"]
mod generated;
#[path = "../../generated/unify-right.rs"]
mod right;
#[path = "../../generated/unify-scoped.rs"]
mod scoped;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("unification export")
}

fn bounds(lower: Option<i64>, upper: Option<i64>) -> Value {
    Value::Bounds { lower, upper }
}

// Preserve conflict operands when comparing implementations; laws below intentionally
// normalize all conflicts instead, since their provenance can depend on grouping.
fn outcome(result: Result<Value, Error>) -> Result<Value, String> {
    result.map_err(|error| error.to_string())
}

fn check(program: &str, left: Value, right: Value, expected: Result<Value, &str>) {
    let expected = expected.map_err(str::to_owned);
    for actual in [
        generated::unify(left, right),
        unification::interpret(program, left, right),
        runtime::unify(left, right),
    ] {
        assert_eq!(outcome(actual), expected, "{left:?} & {right:?}");
    }
}

#[test]
fn production_values_and_errors_have_independent_expectations() {
    let program = export("unify");
    let integer = Value::Integer;
    let cases = [
        (Value::Int, integer(7), Ok(integer(7))),
        (bounds(Some(0), None), Value::Int, Ok(bounds(Some(0), None))),
        (integer(3), integer(3), Ok(integer(3))),
        (
            integer(3),
            integer(4),
            Err("conflicting integer values 3 and 4"),
        ),
        (
            integer(4),
            integer(3),
            Err("conflicting integer values 4 and 3"),
        ),
        (
            bounds(Some(-2), None),
            bounds(None, Some(5)),
            Ok(bounds(Some(-2), Some(5))),
        ),
        (
            bounds(Some(-2), Some(8)),
            bounds(Some(0), Some(5)),
            Ok(bounds(Some(0), Some(5))),
        ),
        (
            bounds(None, None),
            bounds(None, Some(5)),
            Ok(bounds(None, Some(5))),
        ),
        (
            bounds(Some(1), None),
            bounds(Some(4), None),
            Ok(bounds(Some(4), None)),
        ),
        (
            bounds(None, Some(8)),
            bounds(None, Some(2)),
            Ok(bounds(None, Some(2))),
        ),
        (integer(3), bounds(Some(3), Some(5)), Ok(integer(3))),
        (bounds(Some(3), Some(5)), integer(5), Ok(integer(5))),
        (
            integer(2),
            bounds(Some(3), None),
            Err("conflicting integer bounds"),
        ),
        (
            bounds(None, Some(2)),
            integer(3),
            Err("conflicting integer bounds"),
        ),
        (
            bounds(Some(4), None),
            bounds(None, Some(3)),
            Err("conflicting integer bounds"),
        ),
        (
            bounds(Some(3), None),
            bounds(None, Some(3)),
            Ok(bounds(Some(3), Some(3))),
        ),
        (
            bounds(Some(i64::MAX), None),
            bounds(None, None),
            Ok(bounds(Some(i64::MAX), None)),
        ),
        (
            bounds(None, Some(i64::MIN)),
            bounds(None, None),
            Ok(bounds(None, Some(i64::MIN))),
        ),
        (
            bounds(Some(i64::MAX), None),
            integer(i64::MAX),
            Ok(integer(i64::MAX)),
        ),
        (
            bounds(None, Some(i64::MIN)),
            integer(i64::MIN),
            Ok(integer(i64::MIN)),
        ),
        (
            bounds(Some(i64::MAX), None),
            bounds(None, Some(i64::MIN)),
            Err("conflicting integer bounds"),
        ),
    ];
    for (left, right, expected) in cases {
        check(&program, left, right, expected);
    }
    assert!(matches!(
        runtime::concrete(bounds(Some(3), Some(3))),
        Err(Error::IncompleteBounds)
    ));
    for value in [bounds(Some(i64::MAX), None), bounds(None, Some(i64::MIN))] {
        assert!(matches!(
            runtime::concrete(value),
            Err(Error::IncompleteBounds)
        ));
    }
}

#[test]
fn parity_and_algebraic_laws_on_sampled_values() {
    let program = export("unify");
    let values = [
        Value::Int,
        Value::Integer(i64::MIN),
        Value::Integer(-1),
        Value::Integer(0),
        Value::Integer(1),
        Value::Integer(i64::MAX),
        bounds(None, None),
        bounds(Some(0), None),
        bounds(None, Some(0)),
        bounds(Some(-1), Some(1)),
        bounds(Some(0), Some(0)),
        bounds(Some(i64::MAX), None),
        bounds(None, Some(i64::MIN)),
    ];
    for a in values {
        assert_eq!(generated::unify(a, a).ok(), Some(a));
        for b in values {
            let ab = outcome(generated::unify(a, b));
            assert_eq!(
                ab,
                outcome(unification::interpret(&program, a, b)),
                "{a:?}, {b:?}"
            );
            assert_eq!(ab, outcome(runtime::unify(a, b)), "{a:?}, {b:?}");
            assert_eq!(
                generated::unify(a, b).ok(),
                generated::unify(b, a).ok(),
                "{a:?}, {b:?}"
            );
            for c in values {
                assert_eq!(
                    generated::unify(a, b)
                        .and_then(|v| generated::unify(v, c))
                        .ok(),
                    generated::unify(b, c)
                        .and_then(|v| generated::unify(a, v))
                        .ok(),
                    "{a:?}, {b:?}, {c:?}"
                );
            }
        }
    }
}

#[test]
fn alternate_program_really_returns_the_right_input() {
    let program = export("tests/valid/unify-right");
    for (left, right_value) in [
        (Value::Integer(1), Value::Integer(2)),
        (bounds(Some(0), None), Value::Int),
        (Value::Int, bounds(None, Some(4))),
    ] {
        assert_eq!(
            right::unify(left, right_value).expect("generated"),
            right_value
        );
        assert_eq!(
            unification::interpret(&program, left, right_value).expect("interpreted"),
            right_value
        );
    }
    assert!(runtime::unify(Value::Integer(1), Value::Integer(2)).is_err());
    assert_eq!(
        right::unify(Value::Integer(1), Value::Integer(2)).expect("alternate"),
        Value::Integer(2)
    );
}

fn program(result: Json) -> String {
    json!({"formatVersion":1,"result":result}).to_string()
}

#[test]
fn validation_rejects_bad_wire_data_and_unbound_locals() {
    let input = json!({"op":"input","input":"left"});
    let local = json!({"op":"local","endpoint":"lower"});
    let empty = json!({"op":"empty"});
    let invalid = [
        "not json".to_owned(),
        r#"{"formatVersion":1,"formatVersion":1,"result":{"op":"input","input":"left"}}"#.to_owned(),
        r#"{"formatVersion":1,"result":{"op":"input","input":"left","input":"right"}}"#.to_owned(),
        r#"{"formatVersion":1,"result":{"op":"bounds","lower":{"op":"empty","op":"empty"},"upper":{"op":"empty"}}}"#.to_owned(),
        json!({"formatVersion":2,"result":input}).to_string(),
        json!({"formatVersion":1,"result":input,"extra":true}).to_string(),
        program(json!({"op":"input","input":"middle"})),
        program(json!({"op":"input","input":"left","extra":0})),
        program(json!({"op":"fail","reason":"unknown"})),
        program(json!({"op":"bounds","lower":local,"upper":empty})),
        // The new binding is unavailable in either initializer.
        program(json!({"op":"letBounds","lower":local,"upper":empty,"result":input})),
        program(json!({"op":"letBounds","lower":empty,"upper":local,"result":input})),
    ];
    for source in invalid {
        assert!(unification::generate(&source).is_err(), "{source}");
        assert!(
            matches!(
                unification::interpret(&source, Value::Int, Value::Int),
                Err(Error::UnificationProgram { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn locals_are_lexical_and_conditions_short_circuit() {
    let endpoint = |side, endpoint| json!({"op":"input","input":side,"endpoint":endpoint});
    let local = |endpoint| json!({"op":"local","endpoint":endpoint});
    let source = program(
        json!({"op":"letBounds", "lower":endpoint("left","lower"), "upper":endpoint("left","upper"),
        "result":{"op":"letBounds", "lower":local("upper"), "upper":local("lower"),
            "result":{"op":"bounds", "lower":local("lower"), "upper":local("upper")}}}),
    );
    assert!(unification::generate(&source).is_ok());
    assert_eq!(
        unification::interpret(&source, bounds(Some(1), Some(5)), Value::Int).expect("locals"),
        bounds(Some(5), Some(1))
    );

    for (left, expected) in [
        (bounds(Some(1), Some(5)), bounds(Some(5), Some(1))),
        (Value::Int, bounds(None, None)),
    ] {
        assert_eq!(
            scoped::unify(left, Value::Int).expect("compiled lexical scope"),
            expected
        );
        assert_eq!(
            unification::interpret(&export("tests/valid/unify-scoped"), left, Value::Int)
                .expect("scoped interpreter"),
            expected
        );
    }
    let missing = json!({"op":"empty"});
    let compare = json!({"op":"compare", "left":missing, "right":missing, "relation":"equal"});
    let condition =
        json!({"op":"and", "left":{"op":"hasEndpoint","value":missing}, "right":compare});
    let source = program(
        json!({"op":"if", "condition":condition, "then":{"op":"fail","reason":"bounds"}, "else":{"op":"input","input":"right"}}),
    );
    assert_eq!(
        unification::interpret(&source, Value::Int, Value::Integer(9)).expect("short circuit"),
        Value::Integer(9)
    );
    let source = program(
        json!({"op":"if", "condition":compare, "then":{"op":"input","input":"left"}, "else":{"op":"input","input":"right"}}),
    );
    assert!(matches!(
        unification::interpret(&source, Value::Int, Value::Int),
        Err(Error::UnificationState { .. })
    ));
}

#[test]
fn resource_limits_and_generation_cli() {
    // Balanced trees exceed the node budget without exceeding the depth budget.
    fn tree(depth: usize) -> Json {
        if depth == 0 {
            return json!({"op":"input","input":"left"});
        }
        json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"int"},"then":tree(depth-1),"else":tree(depth-1)})
    }
    assert!(unification::generate(&program(tree(9))).is_err());
    let mut deep = json!({"op":"input","input":"left"});
    for _ in 0..70 {
        deep =
            json!({"op":"letBounds","lower":{"op":"empty"},"upper":{"op":"empty"},"result":deep});
    }
    assert!(unification::generate(&program(deep)).is_err());

    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("unify.json");
    let source = std::fs::read_to_string(&path).expect("export");
    let generated = unification::generate(&source).expect("generate");
    assert_eq!(generated, unification::generate(&source).expect("repeat"));
    let binary = env!("CARGO_BIN_EXE_bootstrap-unify-codegen");
    let output = Command::new(binary).arg(path).output().expect("generator");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, generated.as_bytes());
    assert!(output.stderr.is_empty());
    for args in [
        vec![],
        vec!["not-a-file"],
        vec!["one", "two"],
        vec!["tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(binary).args(args).output().expect("generator");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
