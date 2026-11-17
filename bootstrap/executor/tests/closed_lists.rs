use bootstrap_executor::{Error, execute, execute_json, generated, runtime, unification};
use runtime::{ClosedList, Value as V};
use serde_json::{Value as J, json};
use std::{fmt::Debug, path::Path, sync::Arc};

#[path = "../../generated/list-length-zero.rs"]
mod alternate;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("closed scalar list"))
}
fn literal(values: J) -> J {
    json!({"kind":"list","value":values})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn tree(entry: J) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: J) -> [Result<J, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn control(function: &str, result: J) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}
fn fails<T: Debug>(result: Result<T, Error>, expected: &str) {
    assert_eq!(result.expect_err("rejection").to_string(), expected);
}
fn ok<T: Debug + PartialEq>(result: Result<T, Error>, expected: T) {
    assert_eq!(result.expect("accepted"), expected);
}
fn decode(raw: &J) -> Result<V, Error> {
    runtime::list(&json!({"value":raw}), "value")
}
fn meets(left: V, right: V) -> [Result<V, Error>; 2] {
    [
        runtime::unify(left.clone(), right.clone()),
        unification::interpret(&export("unify"), left, right),
    ]
}
fn domains() -> Vec<V> {
    vec![
        V::Null,
        V::Top,
        V::Integer(0),
        V::Int,
        V::Boolean(false),
        V::Boolean(true),
        V::Bool,
        V::Text(Arc::from("x")),
        V::String,
        bounds(None, None),
        bounds(Some(i64::MIN), None),
        bounds(None, Some(i64::MAX)),
        bounds(Some(7), Some(7)),
    ]
}

fn bounds(lower: Option<i64>, upper: Option<i64>) -> V {
    V::Bounds { lower, upper }
}

#[test]
fn exact_owned_round_trips_and_integer_output_boundary() {
    let mixed: J = serde_json::from_str(
        r#"[null,false,true,-9223372036854775808,9223372036854775807,"é🙂","\u0000","e\u0301"]"#,
    )
    .expect("fixed mixed scalars");
    for raw in [json!([]), mixed, json!(vec![0; 64])] {
        let node = json!({"value":raw});
        let owned = runtime::list(&node, "value").expect("raw scalar codec");
        drop(node);
        ok(runtime::encode_json(owned.clone()), raw.clone());
        ok(runtime::concrete_json(owned.clone()), raw.clone());
        for result in both(literal(raw.clone())) {
            let output = result.expect("list JSON");
            assert_eq!(output, raw);
            assert_eq!(output.to_string(), raw.to_string());
        }
        assert!(matches!(
            runtime::concrete(owned),
            Err(Error::IntegerOutput)
        ));
        let input = tree(literal(raw));
        for result in [
            execute(&input, &export("evaluation")),
            generated::execute(&input),
        ] {
            assert!(matches!(result, Err(Error::IntegerOutput)));
        }
    }
    let exact = r#"[null,false,true,-9223372036854775808,9223372036854775807,"é🙂","\u0000"]"#;
    for result in both(literal(
        serde_json::from_str(exact).expect("independent JSON"),
    )) {
        assert_eq!(result.expect("exact i64 output").to_string(), exact);
    }
}

#[test]
fn container_content_equality_and_shared_child_ownership() {
    let text: Arc<str> = Arc::from("owned 🦀");
    let container = ClosedList::new(vec![V::Text(Arc::clone(&text))]).expect("list");
    let count = Arc::strong_count(&text);
    let cloned = container.clone();
    assert_eq!(container.as_slice().as_ptr(), cloned.as_slice().as_ptr());
    assert_eq!(Arc::strong_count(&text), count);
    assert_eq!(container.len(), 1);
    assert!(!container.is_empty());
    let distinct = ClosedList::new(vec![V::Text(Arc::from("owned 🦀"))]).expect("list");
    assert_ne!(container.as_slice().as_ptr(), distinct.as_slice().as_ptr());
    assert_eq!(container, distinct);
    let empty = ClosedList::new(vec![]).expect("empty");
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    drop(container);
    assert_eq!(cloned.as_slice(), &[V::Text(Arc::clone(&text))]);
    drop(cloned);
    assert_eq!(Arc::strong_count(&text), 1);
}

#[test]
fn capacity_precedes_validation_and_bad_elements_are_indexed() {
    assert_eq!(ClosedList::new(vec![V::Null; 64]).expect("limit").len(), 64);
    let mut oversized = vec![V::Null; 65];
    oversized[0] = V::Top;
    fails(
        ClosedList::new(oversized),
        "closed list exceeds 64 elements",
    );
    let mut raw = vec![json!(null); 65];
    raw[0] = json!({});
    fails(decode(&json!(raw)), "closed list exceeds 64 elements");
    for result in both(literal(json!(vec![0; 65]))) {
        assert!(matches!(result, Err(Error::ListCapacity)));
    }
    for bad in domains()
        .into_iter()
        .filter(|v| matches!(v, V::Top | V::Int | V::Bool | V::String | V::Bounds { .. }))
        .chain([list(vec![])])
    {
        fails(
            ClosedList::new(vec![V::Null, bad]),
            "closed list element 1 requires a concrete scalar",
        );
    }
    assert!(
        matches!(runtime::list(&json!({}), "value"), Err(Error::MissingField { field }) if field == "value")
    );
    for raw in [json!(null), json!(false), json!(1), json!("x"), json!({})] {
        fails(decode(&raw), "field \"value\" must be an array");
        for result in both(literal(raw)) {
            assert!(matches!(result, Err(Error::Array { .. })));
        }
    }
    for bad in [
        json!(1.5),
        json!(1.0),
        json!(u64::MAX),
        json!([]),
        json!([1]),
        json!({"kind":"null"}),
    ] {
        let raw = json!([null, "ok", bad]);
        fails(
            decode(&raw),
            "closed list element 2 requires a concrete scalar",
        );
        for result in both(literal(raw)) {
            assert!(matches!(result, Err(Error::ListElement { index: 2 })));
        }
    }
    for result in both(json!({"kind":"list"})) {
        assert!(matches!(result, Err(Error::MissingField { field }) if field == "value"));
    }
}

#[test]
fn unification_content_and_all_mixed_domain_boundaries() {
    let value = list(vec![V::Null, V::Integer(7), V::Text(Arc::from("é"))]);
    let same = list(vec![V::Null, V::Integer(7), V::Text(Arc::from("é"))]);
    for other in [same, V::Top] {
        for (left, right) in [(value.clone(), other.clone()), (other, value.clone())] {
            for result in meets(left, right) {
                ok(result, value.clone());
            }
        }
    }
    for other in domains().into_iter().filter(|v| *v != V::Top) {
        for (left, right) in [(value.clone(), other.clone()), (other, value.clone())] {
            for result in meets(left, right) {
                assert!(matches!(result, Err(Error::TypeConflict)));
            }
        }
    }
    for raw in [
        json!([]),
        json!([null, 7]),
        json!([null, true, "é"]),
        json!([null, 7, "e\u{301}"]),
    ] {
        let other = decode(&raw).expect("other list");
        for (left, right) in [(value.clone(), other.clone()), (other, value.clone())] {
            for result in meets(left, right) {
                fails(result, "conflicting closed list values");
            }
        }
        let entry = binary("unify", literal(json!([null, 7, "é"])), literal(raw));
        for result in both(entry) {
            assert!(matches!(result, Err(Error::ListConflict)));
        }
    }
    for other in [literal(json!([7])), json!({"kind":"top"})] {
        for result in both(binary("unify", literal(json!([7])), other)) {
            assert_eq!(result.expect("list identity"), json!([7]));
        }
    }
}

#[test]
fn equality_logical_selection_and_rejected_operations() {
    let a = literal(json!([null, false, 7, "é"]));
    for (b, expected) in [
        (a.clone(), true),
        (literal(json!([])), false),
        (literal(json!([null, false, 7, "e\u{301}"])), false),
    ] {
        for (kind, answer) in [("equal", expected), ("notEqual", !expected)] {
            for result in both(binary(kind, a.clone(), b.clone())) {
                assert_eq!(result.expect("content equality"), json!(answer));
            }
        }
    }
    let value = list(vec![V::Null]);
    for other in domains() {
        let concrete = matches!(other, V::Null | V::Integer(_) | V::Boolean(_) | V::Text(_));
        for (left, right) in [(value.clone(), other.clone()), (other, value.clone())] {
            for result in [
                runtime::equal(left.clone(), right.clone()),
                unification::interpret(&export("equal"), left, right),
            ] {
                if concrete {
                    assert_eq!(result.expect("mixed concrete"), V::Boolean(false));
                } else {
                    assert!(matches!(result, Err(Error::Comparison)));
                }
            }
        }
    }
    for (kind, left, expected) in [("logicalAnd", "false", false), ("logicalOr", "true", true)] {
        for result in both(binary(kind, json!({"kind":left}), a.clone())) {
            assert_eq!(result.expect("ignored concrete RHS"), json!(expected));
        }
    }
    for (kind, selecting) in [("logicalAnd", "true"), ("logicalOr", "false")] {
        for (left, right) in [
            (a.clone(), json!({"kind":"true"})),
            (json!({"kind":selecting}), a.clone()),
        ] {
            for result in both(binary(kind, left, right)) {
                assert!(matches!(result, Err(Error::Logical)));
            }
        }
    }
    for kind in [
        "add",
        "subtract",
        "multiply",
        "divide",
        "less",
        "lessEqual",
        "greater",
        "greaterEqual",
    ] {
        for (left, right) in [
            (a.clone(), literal(json!([]))),
            (json!({"kind":"integer","value":1}), a.clone()),
            (a.clone(), json!({"kind":"integer","value":1})),
        ] {
            for result in both(binary(kind, left, right)) {
                if ["less", "lessEqual", "greater", "greaterEqual"].contains(&kind) {
                    assert!(matches!(result, Err(Error::Ordering)));
                } else {
                    assert!(matches!(result, Err(Error::Arithmetic)));
                }
            }
        }
    }
}

#[test]
fn list_length_counts_and_invalid_domains() {
    let policy = export("list-length-control");
    for size in [0, 1, 64] {
        let value = list(vec![V::Null; size]);
        for result in [
            runtime::list_count(&value),
            runtime::list_length(value.clone()),
            unification::interpret_unary(&policy, value),
        ] {
            assert_eq!(result.expect("count"), V::Integer(size as i64));
        }
        let input = tree(json!({"kind":"listLength","operand":literal(json!(vec![0; size]))}));
        for result in [
            execute_json(&input, &export("evaluation")),
            generated::execute_json(&input),
            execute(&input, &export("evaluation")).map(|v| json!(v)),
            generated::execute(&input).map(|v| json!(v)),
        ] {
            assert_eq!(result.expect("integer count"), json!(size));
        }
    }
    for value in domains() {
        for result in [
            runtime::list_count(&value),
            runtime::list_length(value.clone()),
            unification::interpret_unary(&policy, value),
        ] {
            fails(result, "length requires a concrete closed list");
        }
    }
    for ast in [
        json!({"kind":"false"}),
        json!({"kind":"top"}),
        json!({"kind":"string","value":"x"}),
        json!({"kind":"lowerBound","value":7}),
    ] {
        for result in both(json!({"kind":"listLength","operand":ast})) {
            assert!(matches!(result, Err(Error::ListLength)));
        }
    }
    // Existing stringLength/source len policy remains text-only, not a generic cast.
    assert!(matches!(
        runtime::string_length(list(vec![])),
        Err(Error::Length)
    ));
    for result in both(json!({"kind":"stringLength","operand":literal(json!([1]))})) {
        assert!(matches!(result, Err(Error::Length)));
    }
    let text: Arc<str> = Arc::from("owned");
    let value = list(vec![V::Text(Arc::clone(&text))]);
    let count = Arc::strong_count(&text);
    ok(runtime::list_count(&value), V::Integer(1));
    assert_eq!(Arc::strong_count(&text), count);
}

#[test]
fn eager_children_precede_length_domain_and_logical_selection() {
    for (child, expected) in [
        (
            binary(
                "divide",
                json!({"kind":"integer","value":1}),
                json!({"kind":"integer","value":0}),
            ),
            "integer division by zero",
        ),
        (
            literal(json!([{}])),
            "closed list element 0 requires a concrete scalar",
        ),
        (
            json!({"kind":"unknownTestKind"}),
            "no rule for node kind \"unknownTestKind\"",
        ),
    ] {
        for entry in [
            json!({"kind":"listLength","operand":child}),
            binary("logicalAnd", json!({"kind":"false"}), child.clone()),
        ] {
            for result in both(entry) {
                fails(result, expected);
            }
        }
    }
    for result in both(binary(
        "unify",
        literal(json!([{}])),
        json!({"kind":"unknownTestKind"}),
    )) {
        assert!(matches!(result, Err(Error::ListElement { index: 0 })));
    }
}

#[test]
fn compiled_alternate_left_only_primitive_and_independent_failures() {
    let policy = export("tests/valid/list-length-zero");
    for value in [list(vec![V::Null; 2]), V::Boolean(false)] {
        ok(alternate::list_length(value.clone()), V::Integer(0));
        ok(unification::interpret_unary(&policy, value), V::Integer(0));
    }
    ok(runtime::list_length(list(vec![V::Null; 2])), V::Integer(2));
    assert!(matches!(
        runtime::list_length(V::Boolean(false)),
        Err(Error::ListLength)
    ));
    let binary = control("unify", json!({"op":"listCount"}));
    assert!(unification::generate(&binary).is_ok());
    for right in domains() {
        ok(
            unification::interpret(&binary, list(vec![V::Null; 2]), right),
            V::Integer(2),
        );
    }
    assert!(matches!(
        unification::interpret(&binary, V::Null, list(vec![])),
        Err(Error::ListLength)
    ));
    for function in ["listLength", "unify"] {
        let input = control(function, json!({"op":"fail","reason":"listLength"}));
        assert!(unification::generate(&input).is_ok());
        let result = if function == "listLength" {
            unification::interpret_unary(&input, list(vec![]))
        } else {
            unification::interpret(&input, V::Null, V::Top)
        };
        assert!(matches!(result, Err(Error::ListLength)));
    }
    let failure = control("unify", json!({"op":"fail","reason":"lists"}));
    assert!(unification::generate(&failure).is_ok());
    assert!(matches!(
        unification::interpret(&failure, V::Null, V::Top),
        Err(Error::ListConflict)
    ));
    let code = unification::generate(&export("list-length-control")).expect("lowering");
    assert!(code.contains("fn list_length(_left: Value)"));
    assert!(!code.contains("_right"));
    assert!(unification::interpret(&policy, V::Null, V::Null).is_err());
}

#[test]
fn unary_validation_is_eager_and_rejects_extra_fields_and_aliases() {
    for bad in [
        json!({"op":"input","input":"right"}),
        json!({"op":"integerSum"}),
        json!({"op":"listCount","input":"right"}),
        json!({"op":"listCount","extra":0}),
        json!({"op":"fail","reason":"lists"}),
        json!({"op":"unaryFail","reason":"listLength"}),
        json!({"op":"inputValue","input":"left"}),
        json!({"op":"integerValue","value":0}),
    ] {
        for result in [
            bad.clone(),
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"list"},"then":{"op":"integer","value":0},"else":bad}),
        ] {
            let input = control("listLength", result);
            assert!(unification::generate(&input).is_err(), "{input}");
            assert!(
                unification::interpret_unary(&input, list(vec![])).is_err(),
                "{input}"
            );
        }
    }
    for input in [
        r#"{"formatVersion":1,"function":"listLength","result":{"op":"listCount","op":"listCount"}}"#,
        r#"{"formatVersion":1,"function":"listLength","extra":0,"result":{"op":"listCount"}}"#,
    ] {
        assert!(unification::generate(input).is_err());
        assert!(unification::interpret_unary(input, list(vec![])).is_err());
    }
}
