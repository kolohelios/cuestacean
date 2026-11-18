use bootstrap_executor::{Error, execute, execute_json, generated, runtime, unification};
use runtime::{ClosedList, ClosedRecord, Value as V};
use serde_json::{Value as J, json};
use std::{fmt::Debug, path::Path, sync::Arc};

#[path = "../../generated/record-index-zero.rs"]
mod alternate;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn record(fields: Vec<(&str, V)>) -> V {
    V::Record(
        ClosedRecord::new(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
            .expect("record"),
    )
}
fn literal(fields: J) -> J {
    json!({"kind":"record","fields":fields})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn unary(kind: &str, operand: J) -> J {
    json!({"kind":kind,"operand":operand})
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn text(value: &str) -> V {
    V::Text(Arc::from(value))
}
fn program(entry: J) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: J) -> [Result<J, Error>; 2] {
    let input = program(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn fails<T: Debug>(result: Result<T, Error>, expected: &str) {
    assert_eq!(result.expect_err("rejection").to_string(), expected);
}
fn decode(fields: J) -> Result<V, Error> {
    runtime::record(&json!({"fields":fields}), "fields")
}
fn domains() -> Vec<V> {
    vec![
        V::Null,
        V::Top,
        V::Integer(0),
        V::Int,
        V::Boolean(false),
        V::Bool,
        text("x"),
        V::String,
        V::Bounds {
            lower: None,
            upper: None,
        },
        V::Bounds {
            lower: Some(1),
            upper: Some(1),
        },
        V::List(ClosedList::new(vec![]).expect("list")),
        record(vec![]),
    ]
}
fn control(function: &str, result: J) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}

#[test]
fn exact_scalar_encoding_byte_order_and_integer_output_boundary() {
    let fields = json!([
        {"name":"🙂","value":"é\u{0000}"}, {"name":"z","value":i64::MAX},
        {"name":"a","value":i64::MIN}, {"name":"é","value":true},
        {"name":"e\u{301}","value":false}, {"name":"null","value":null}
    ]);
    let expected = "{\"a\":-9223372036854775808,\"é\":false,\"null\":null,\"z\":9223372036854775807,\"é\":true,\"🙂\":\"é\\u0000\"}";
    let node = json!({"fields":fields});
    let owned = runtime::record(&node, "fields").expect("codec");
    drop(node);
    for result in [
        runtime::encode_json(owned.clone()),
        runtime::concrete_json(owned.clone()),
    ]
    .into_iter()
    .chain(both(literal(fields.clone())))
    {
        assert_eq!(result.expect("exact output").to_string(), expected);
    }
    assert!(matches!(
        runtime::concrete(owned),
        Err(Error::IntegerOutput)
    ));
    let input = program(literal(fields));
    for result in [
        execute(&input, &export("evaluation")),
        generated::execute(&input),
    ] {
        assert!(matches!(result, Err(Error::IntegerOutput)));
    }
    for result in both(literal(json!([]))) {
        assert_eq!(result.expect("empty"), json!({}));
    }
}

#[test]
fn shared_map_content_equality_and_selected_text_lifetime() {
    let child: Arc<str> = Arc::from("owned 🦀");
    let value = ClosedRecord::new(vec![("x".into(), V::Text(child.clone()))]).expect("record");
    let count = Arc::strong_count(&child);
    let cloned = value.clone();
    assert!(std::ptr::eq(value.as_map(), cloned.as_map()));
    assert_eq!(Arc::strong_count(&child), count);
    let distinct = ClosedRecord::new(vec![("x".into(), text("owned 🦀"))]).expect("record");
    assert!(!std::ptr::eq(value.as_map(), distinct.as_map()));
    assert_eq!(value, distinct);
    assert_eq!(value.len(), 1);
    assert!(!value.is_empty());
    let empty = ClosedRecord::new(vec![]).expect("empty");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    let selected = runtime::record_get(&V::Record(cloned), &text("x")).expect("selection");
    let V::Text(selected) = selected else {
        panic!("text")
    };
    assert!(Arc::ptr_eq(&child, &selected));
    drop(value);
    drop(child);
    assert_eq!(&*selected, "owned 🦀");
}

#[test]
fn input_capacity_precedes_even_duplicate_and_invalid_entries() {
    let fields: Vec<_> = (0..64).map(|i| (format!("f{i}"), V::Null)).collect();
    assert_eq!(ClosedRecord::new(fields).expect("64 fields").len(), 64);
    let raw: Vec<_> = (0..64)
        .map(|i| json!({"name":format!("f{i}"),"value":null}))
        .collect();
    assert_eq!(
        decode(json!(raw)).expect("64 raw fields"),
        V::Record(
            ClosedRecord::new((0..64).map(|i| (format!("f{i}"), V::Null)).collect())
                .expect("record")
        )
    );
    for fields in [
        vec![("x".into(), V::Null); 65],
        vec![(String::new(), V::Top); 65],
    ] {
        fails(ClosedRecord::new(fields), "closed record exceeds 64 fields");
    }
    for raw in [
        json!(vec![json!({"name":"x","value":null}); 65]),
        json!(vec![json!({}); 65]),
    ] {
        fails(decode(raw.clone()), "closed record exceeds 64 fields");
        for result in both(literal(raw)) {
            assert!(matches!(result, Err(Error::RecordCapacity)));
        }
    }
}

#[test]
fn names_use_utf8_bytes_and_duplicate_checks_precede_scalar_decoding() {
    for name in ["a".repeat(64), "é".repeat(32), "🙂".repeat(16), "\0".into()] {
        assert!(ClosedRecord::new(vec![(name.clone(), V::Null)]).is_ok());
        assert!(decode(json!([{"name":name,"value":null}])).is_ok());
    }
    for name in [
        String::new(),
        "a".repeat(65),
        "é".repeat(33),
        "🙂".repeat(17),
    ] {
        let expected = "closed record field name at index 1 must contain 1..=64 UTF-8 bytes";
        fails(
            ClosedRecord::new(vec![("ok".into(), V::Null), (name.clone(), V::Top)]),
            expected,
        );
        fails(
            decode(json!([{"name":"ok","value":null},{"name":name,"value":{}}])),
            expected,
        );
    }
    fails(
        ClosedRecord::new(vec![("x".into(), V::Null), ("x".into(), V::Top)]),
        "duplicate closed record field \"x\"",
    );
    // Missing and nonscalar raw values cannot mask an earlier duplicate/name error.
    for bad in [json!({"name":"x"}), json!({"name":"x","value":{}})] {
        fails(
            decode(json!([{"name":"x","value":null},bad])),
            "duplicate closed record field \"x\"",
        );
    }
    fails(
        decode(json!([{"name":""}])),
        "closed record field name at index 0 must contain 1..=64 UTF-8 bytes",
    );
    assert!(decode(json!([{"name":"é","value":1},{"name":"e\u{301}","value":2}])).is_ok());
}

#[test]
fn malformed_payloads_and_explicit_collection_profile_boundaries() {
    assert!(
        matches!(runtime::record(&json!({}), "fields"), Err(Error::MissingField { field }) if field == "fields")
    );
    for raw in [json!(null), json!(true), json!(1), json!("x"), json!({})] {
        fails(decode(raw), "field \"fields\" must be an array");
    }
    for raw in [json!({}), json!({"value":null})] {
        assert!(
            matches!(decode(json!([raw])), Err(Error::MissingField { field }) if field == "name")
        );
    }
    for name in [json!(null), json!(1), json!([]), json!({})] {
        fails(
            decode(json!([{"name":name,"value":null}])),
            "field \"name\" must be a string",
        );
    }
    assert!(
        matches!(decode(json!([{"name":"x"}])), Err(Error::MissingField { field }) if field == "value")
    );
    for raw in [
        json!(1.0),
        json!(1.5),
        json!(u64::MAX),
        json!([]),
        json!({}),
        json!({"kind":"integer","value":1}),
    ] {
        for result in both(literal(json!([{"name":"x","value":raw}]))) {
            fails(
                result,
                "closed record field \"x\" requires a concrete scalar or closed record",
            );
        }
    }
    for value in domains().into_iter().filter(|v| {
        !matches!(
            v,
            V::Null | V::Integer(_) | V::Boolean(_) | V::Text(_) | V::Record(_)
        )
    }) {
        fails(
            ClosedRecord::new(vec![("x".into(), value)]),
            "closed record field \"x\" requires a concrete scalar or closed record",
        );
    }
    fails(
        ClosedList::new(vec![record(vec![])]),
        "closed list element 0 requires a concrete scalar or closed list",
    );
    for result in both(json!({"kind":"listExpression","elements":[literal(json!([]))]})) {
        fails(
            result,
            "closed list element 0 requires a concrete scalar or closed list",
        );
    }
}

#[test]
fn closed_unification_requires_exact_field_sets_and_content() {
    let a = record(vec![("a", V::Null), ("b", V::Integer(7))]);
    let same = record(vec![("b", V::Integer(7)), ("a", V::Null)]);
    for other in [same, V::Top] {
        for (left, right) in [(a.clone(), other.clone()), (other, a.clone())] {
            for result in [
                runtime::unify(left.clone(), right.clone()),
                unification::interpret(&export("unify"), left, right),
            ] {
                assert_eq!(result.expect("identity"), a);
            }
        }
    }
    for other in [
        record(vec![]),
        record(vec![("a", V::Null)]),
        record(vec![("a", V::Null), ("b", V::Integer(8))]),
        record(vec![("a", V::Null), ("c", V::Integer(7))]),
    ] {
        for (left, right) in [(a.clone(), other.clone()), (other, a.clone())] {
            for result in [
                runtime::unify(left.clone(), right.clone()),
                unification::interpret(&export("unify"), left, right),
            ] {
                fails(result, "conflicting closed record values");
            }
        }
    }
    for other in domains()
        .into_iter()
        .filter(|v| !matches!(v, V::Top | V::Record(_)))
    {
        for (left, right) in [(a.clone(), other.clone()), (other, a.clone())] {
            for result in [
                runtime::unify(left.clone(), right.clone()),
                unification::interpret(&export("unify"), left, right),
            ] {
                assert!(matches!(result, Err(Error::TypeConflict)));
            }
        }
    }
}

#[test]
fn record_equality_concreteness_logical_selection_and_unsupported_operations() {
    let a = literal(json!([{"name":"x","value":1}]));
    for (other, equal) in [
        (a.clone(), true),
        (literal(json!([])), false),
        (integer(1), false),
        (json!({"kind":"null"}), false),
        (json!({"kind":"list","value":[]}), false),
    ] {
        for (kind, expected) in [("equal", equal), ("notEqual", !equal)] {
            for (left, right) in [(a.clone(), other.clone()), (other.clone(), a.clone())] {
                for result in both(binary(kind, left, right)) {
                    assert_eq!(result.expect("concrete equality"), json!(expected));
                }
            }
        }
    }
    for other in domains() {
        let concrete = matches!(
            other,
            V::Null | V::Integer(_) | V::Boolean(_) | V::Text(_) | V::List(_) | V::Record(_)
        );
        for (left, right) in [
            (record(vec![("x", V::Integer(1))]), other.clone()),
            (other, record(vec![("x", V::Integer(1))])),
        ] {
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
    for (kind, boolean, expected) in [("logicalAnd", "false", false), ("logicalOr", "true", true)] {
        for result in both(binary(kind, json!({"kind":boolean}), a.clone())) {
            assert_eq!(result.expect("determining boolean"), json!(expected));
        }
    }
    for (kind, boolean) in [("logicalAnd", "true"), ("logicalOr", "false")] {
        for (left, right) in [
            (a.clone(), json!({"kind":boolean})),
            (json!({"kind":boolean}), a.clone()),
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
        for (left, right) in [(a.clone(), integer(1)), (integer(1), a.clone())] {
            for result in both(binary(kind, left, right)) {
                if kind.starts_with("less") || kind.starts_with("greater") {
                    assert!(matches!(result, Err(Error::Ordering)));
                } else {
                    assert!(matches!(result, Err(Error::Arithmetic)));
                }
            }
        }
    }
}

#[test]
fn checked_lookup_presence_lengths_and_all_operand_domains() {
    let value = record(vec![("x", V::Null), ("é", V::Integer(7))]);
    for (key, expected) in [("x", V::Null), ("é", V::Integer(7))] {
        assert!(runtime::record_has(&value, &text(key)).expect("present"));
        for result in [
            runtime::record_get(&value, &text(key)),
            runtime::record_index(value.clone(), text(key)),
            unification::interpret(&export("record-index"), value.clone(), text(key)),
        ] {
            assert_eq!(result.expect("lookup"), expected);
        }
    }
    for key in ["", "missing", "e\u{301}"] {
        assert!(!runtime::record_has(&value, &text(key)).expect("absent"));
        for result in [
            runtime::record_get(&value, &text(key)),
            runtime::record_index(value.clone(), text(key)),
            unification::interpret(&export("record-index"), value.clone(), text(key)),
        ] {
            fails(result, &format!("missing closed record field {key:?}"));
        }
        for result in both(binary(
            "recordIndex",
            literal(json!([{"name":"x","value":null},{"name":"é","value":7}])),
            json!({"kind":"string","value":key}),
        )) {
            fails(result, &format!("missing closed record field {key:?}"));
        }
    }
    for left in domains() {
        for right in domains() {
            if matches!((&left, &right), (V::Record(_), V::Text(_))) {
                continue;
            }
            fails(
                runtime::record_has(&left, &right),
                "field lookup requires a concrete closed record and string",
            );
            for result in [
                runtime::record_get(&left, &right),
                runtime::record_index(left.clone(), right.clone()),
                unification::interpret(&export("record-index"), left.clone(), right.clone()),
            ] {
                assert!(matches!(result, Err(Error::RecordIndexOperands)));
            }
        }
        if !matches!(left, V::Record(_)) {
            for result in [
                runtime::record_count(&left),
                runtime::record_length(left.clone()),
                unification::interpret_unary(&export("record-length"), left),
            ] {
                fails(result, "length requires a concrete closed record");
            }
        }
    }
    for size in [0, 1, 64] {
        let fields: Vec<_> = (0..size).map(|i| (format!("f{i}"), V::Null)).collect();
        let value = V::Record(ClosedRecord::new(fields).expect("record"));
        for result in [
            runtime::record_count(&value),
            runtime::record_length(value.clone()),
            runtime::length(value.clone()),
            unification::interpret_unary(&export("record-length"), value.clone()),
        ] {
            assert_eq!(result.expect("count"), V::Integer(size));
        }
        assert!(matches!(
            runtime::string_length(value.clone()),
            Err(Error::Length)
        ));
        assert!(matches!(
            runtime::list_length(value),
            Err(Error::ListLength)
        ));
    }
}

#[test]
fn ast_eager_order_and_literal_data_consume_one_visit() {
    let a = literal(json!([{"name":"x","value":7}]));
    for result in both(binary(
        "recordIndex",
        a.clone(),
        json!({"kind":"string","value":"x"}),
    )) {
        assert_eq!(result.expect("AST lookup"), json!(7));
    }
    for kind in ["recordLength", "length"] {
        for result in both(unary(kind, a.clone())) {
            assert_eq!(result.expect("length"), json!(1));
        }
    }
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("add", integer(i64::MAX), integer(1));
    for (left, right, expected) in [
        (zero.clone(), overflow.clone(), "integer division by zero"),
        (integer(1), zero.clone(), "integer division by zero"),
        (
            a.clone(),
            overflow,
            "signed 64-bit integer addition overflow",
        ),
    ] {
        for result in both(binary("recordIndex", left, right)) {
            fails(result, expected);
        }
    }
    for entry in [
        unary("recordLength", zero.clone()),
        binary("logicalAnd", json!({"kind":"false"}), zero),
    ] {
        for result in both(entry) {
            fails(result, "integer division by zero");
        }
    }
    let fields: Vec<_> = (0..64)
        .map(|i| json!({"name":format!("f{i}"),"value":null}))
        .collect();
    let mut entry = unary("recordLength", literal(json!(fields)));
    // Record plus length plus 62 wrappers is exactly 64 active AST nodes.
    for _ in 0..62 {
        entry = unary("negate", entry);
    }
    for result in both(entry.clone()) {
        assert_eq!(result.expect("literal data is not child AST"), json!(64));
    }
    for result in both(unary("negate", entry)) {
        assert!(matches!(result, Err(Error::EvaluationDepth)));
    }
}

#[test]
fn compiled_alternate_and_typed_validation_do_not_infer_behavior_from_name() {
    let policy = export("tests/valid/record-index-zero");
    for (left, right) in [(record(vec![("x", V::Null)]), text("x")), (V::Top, V::Int)] {
        assert_eq!(
            alternate::record_index(left.clone(), right.clone()).expect("compiled alternate"),
            V::Integer(0)
        );
        assert_eq!(
            unification::interpret(&policy, left, right).expect("alternate"),
            V::Integer(0)
        );
    }
    assert!(unification::interpret_unary(&policy, V::Null).is_err());
    let missing = control("recordIndex", json!({"op":"fail","reason":"recordMissing"}));
    assert!(unification::generate(&missing).is_ok());
    fails(
        unification::interpret(&missing, record(vec![]), text("absent")),
        "missing closed record field \"absent\"",
    );
    assert!(unification::interpret(&export("record-length"), V::Null, V::Null).is_err());
    let code = unification::generate(&export("record-index")).expect("lowering");
    assert!(code.contains("fn record_index("));
    assert!(code.contains("record_get("));
    let count = control("recordLength", json!({"op":"recordCount"}));
    assert!(unification::generate(&count).is_ok());
    assert_eq!(
        unification::interpret_unary(&count, record(vec![])).expect("count"),
        V::Integer(0)
    );
    for bad in [
        json!({"op":"recordGet"}),
        json!({"op":"input","input":"right"}),
        json!({"op":"recordCount","extra":0}),
        json!({"op":"recordCount","input":"right"}),
        json!({"op":"recordCountEndpoint"}),
    ] {
        let input = control(
            "recordLength",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"record"},"then":{"op":"integer","value":0},"else":bad}),
        );
        assert!(unification::generate(&input).is_err(), "{input}");
        assert!(
            unification::interpret_unary(&input, record(vec![])).is_err(),
            "{input}"
        );
    }
    let count_left = control("recordIndex", json!({"op":"recordCount"}));
    for right in domains() {
        assert_eq!(
            unification::interpret(&count_left, record(vec![("x", V::Null)]), right)
                .expect("left-only count"),
            V::Integer(1)
        );
    }
    for reason in ["records", "recordIndexOperands", "recordMissing"] {
        let invalid = control(
            "recordLength",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"record"},"then":{"op":"recordCount"},"else":{"op":"fail","reason":reason}}),
        );
        assert!(unification::generate(&invalid).is_err());
        assert!(matches!(
            unification::interpret_unary(&invalid, record(vec![])),
            Err(Error::UnificationProgram { .. })
        ));
    }
    let bad = control(
        "recordLength",
        json!({"op":"if","condition":{"op":"recordHas"},"then":{"op":"integer","value":0},"else":{"op":"integer","value":1}}),
    );
    assert!(unification::generate(&bad).is_err());
    assert!(matches!(
        unification::interpret_unary(&bad, record(vec![])),
        Err(Error::UnificationProgram { .. })
    ));
    for (reason, expected) in [
        ("records", "conflicting closed record values"),
        (
            "recordIndexOperands",
            "field lookup requires a concrete closed record and string",
        ),
        ("recordLength", "length requires a concrete closed record"),
    ] {
        let input = control("recordIndex", json!({"op":"fail","reason":reason}));
        assert!(unification::generate(&input).is_ok());
        fails(unification::interpret(&input, V::Null, V::Top), expected);
    }
}
