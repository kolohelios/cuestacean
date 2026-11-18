use bootstrap_executor::{Error, execute_json, generated, pratt, runtime, unification};
use runtime::{ClosedRecord, EvaluationContext, RecordDriver, RecordFrame, Value as V};
use serde_json::{Value as J, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/record-control-scalars.rs"]
mod historical;
#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/record-control.rs"]
mod production;
#[path = "../../generated/record-control-reverse.rs"]
mod reverse;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn record(fields: Vec<(String, V)>) -> V {
    V::Record(ClosedRecord::new(fields).expect("validated record"))
}
fn fields(values: Vec<V>) -> Vec<(String, V)> {
    values
        .into_iter()
        .enumerate()
        .map(|(i, v)| (format!("f{i}"), v))
        .collect()
}
fn singleton(value: V) -> V {
    record(vec![("x".into(), value)])
}
fn depth32() -> V {
    let mut value = V::Null;
    for _ in 0..31 {
        value = singleton(value);
    }
    value
}
fn nodes4096() -> V {
    record(fields(vec![record(fields(vec![V::Null; 64])); 63]))
}
fn as_record(value: &V) -> &ClosedRecord {
    let V::Record(record) = value else {
        panic!("expected record")
    };
    record
}
fn expression(fields: J) -> J {
    json!({"kind":"recordExpression","fields":fields})
}
fn field(name: &str, value: J) -> J {
    json!({"name":name,"value":value})
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn lookup(left: J, key: &str) -> J {
    binary("recordIndex", left, json!({"kind":"string","value":key}))
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
fn reference(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    pratt::interpret_record(&export("record-control"), frame)
}
fn reference_reverse(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    pratt::interpret_record(&export("tests/valid/record-control-reverse"), frame)
}
fn reference_historical(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    pratt::interpret_record(&export("tests/valid/record-control-scalars"), frame)
}
fn run(values: &[V], sentinel: bool, driver: RecordDriver) -> (Result<V, Error>, Vec<usize>) {
    let entries: Vec<_> = (0..values.len() + usize::from(sentinel))
        .map(|i| field(&format!("f{i}"), json!(i)))
        .collect();
    let mut trace = Vec::new();
    let result = runtime::record_expression_with_driver(
        &expression(json!(entries)),
        "fields",
        &mut EvaluationContext::default(),
        &mut |node, context| {
            let index = node.as_u64().expect("callback index") as usize;
            trace.push(index);
            context.visit(|_| values.get(index).cloned().ok_or(Error::DivisionByZero))
        },
        driver,
    );
    (result, trace)
}

#[test]
fn root_inclusive_depth_accepts_32_and_rejects_33() {
    for mut value in [V::Null, record(vec![])] {
        for _ in 0..31 {
            value = singleton(value);
        }
        let error = ClosedRecord::new(vec![("x".into(), value)]).expect_err("depth 33");
        assert!(matches!(error, Error::RecordDepth));
        assert_eq!(error.to_string(), "closed record exceeds 32 value levels");
    }
    // Value depth rejects at 33 while AST depth remains below its independent 64.
    for (depth, accepted) in [(32, true), (33, false)] {
        let mut entry = expression(json!([]));
        let mut expected = json!({});
        for _ in 1..depth {
            entry = expression(json!([field("x", entry)]));
            expected = json!({"x":expected});
        }
        for result in both(entry) {
            if accepted {
                assert_eq!(result.expect("32 value levels"), expected);
            } else {
                assert!(matches!(result, Err(Error::RecordDepth)));
            }
        }
    }
}

#[test]
fn shared_occurrences_count_exactly_4096_and_4097_nodes() {
    let shared = record(fields(vec![V::Null; 64])); // 65 nodes.
    let accepted = ClosedRecord::new(fields(vec![shared.clone(); 63])).expect("4096 nodes");
    for child in accepted.as_map().values() {
        assert!(std::ptr::eq(
            as_record(child).as_map(),
            as_record(&shared).as_map()
        ));
    }
    assert_eq!(accepted.len(), 63);
    let mut rejected = vec![shared; 63];
    rejected.push(V::Null); // Width 64, depth 3, 4097 nodes.
    let error = ClosedRecord::new(fields(rejected)).expect_err("4097 nodes");
    assert!(matches!(error, Error::RecordNodes));
    assert_eq!(
        error.to_string(),
        "closed record exceeds 4096 logical value nodes"
    );
}

#[test]
fn constructor_preserves_width_name_duplicate_type_and_shape_order() {
    assert!(matches!(
        ClosedRecord::new(vec![(String::new(), depth32()); 65]),
        Err(Error::RecordCapacity)
    ));
    for name in [String::new(), "a".repeat(65), "é".repeat(33)] {
        assert!(matches!(
            ClosedRecord::new(vec![(name, depth32())]),
            Err(Error::RecordName { index: 0 })
        ));
    }
    assert!(matches!(
        ClosedRecord::new(vec![("x".into(), V::Null), ("x".into(), depth32())]),
        Err(Error::RecordDuplicate { .. })
    ));
    for (shape, depth) in [(depth32(), true), (nodes4096(), false)] {
        let error = ClosedRecord::new(vec![
            ("first".into(), V::Int),
            ("next".into(), shape.clone()),
        ])
        .expect_err("type first");
        assert_eq!(
            error.to_string(),
            "closed record field \"first\" requires a concrete scalar or closed record"
        );
        let error = ClosedRecord::new(vec![("first".into(), shape), (String::new(), V::Int)])
            .expect_err("shape first");
        assert!(matches!(error, Error::RecordDepth | Error::RecordNodes));
        assert_eq!(matches!(error, Error::RecordDepth), depth);
    }
    // Both prospective limits would fail: depth wins over accumulated nodes.
    let mut almost_full = vec![record(fields(vec![V::Null; 64])); 62];
    almost_full.push(record(fields(vec![V::Null; 58]))); // 4090 nodes.
    assert!(matches!(
        ClosedRecord::new(vec![
            ("wide".into(), record(fields(almost_full))),
            ("deep".into(), depth32())
        ]),
        Err(Error::RecordDepth)
    ));
    for value in [
        V::Top,
        V::List(runtime::ClosedList::new(vec![]).expect("list")),
    ] {
        assert!(matches!(
            ClosedRecord::new(vec![("x".into(), value)]),
            Err(Error::RecordElement { .. })
        ));
    }
    assert!(matches!(
        runtime::ClosedList::new(vec![record(vec![])]),
        Err(Error::ListElement { index: 0 })
    ));
}

#[test]
fn frame_shape_checks_stop_before_later_callback_and_match_exact_nodes() {
    for driver in [production::execute as RecordDriver, reference] {
        for (shape, depth) in [(depth32(), true), (nodes4096(), false)] {
            let (result, trace) = run(&[shape], true, driver);
            let error = result.expect_err("prospective shape");
            assert!(matches!(error, Error::RecordDepth | Error::RecordNodes));
            assert_eq!(matches!(error, Error::RecordDepth), depth);
            assert_eq!(trace, vec![0]);
        }
        let shared = record(fields(vec![V::Null; 64]));
        let mut values = vec![shared; 63];
        let (result, trace) = run(&values, false, driver);
        assert_eq!(result.expect("4096 nodes"), nodes4096());
        assert_eq!(trace, (0..63).collect::<Vec<_>>());
        values.push(V::Null);
        let (result, trace) = run(&values, false, driver);
        assert!(matches!(result, Err(Error::RecordNodes)));
        assert_eq!(trace, (0..64).collect::<Vec<_>>());
    }
}

#[test]
fn selected_child_shares_its_map_without_retaining_parent_storage() {
    let text: Arc<str> = Arc::from("owned é🙂");
    let child = singleton(V::Text(text.clone()));
    let parent_only: Arc<str> = Arc::from("parent-only storage witness");
    let parent = record(vec![
        ("x".into(), child.clone()),
        ("other".into(), V::Text(parent_only.clone())),
    ]);
    let selected = runtime::record_get(&parent, &V::Text(Arc::from("x"))).expect("child lookup");
    assert!(std::ptr::eq(
        as_record(&selected).as_map(),
        as_record(&child).as_map()
    ));
    assert_eq!(Arc::strong_count(&text), 2);
    drop(parent);
    drop(child);
    assert_eq!(Arc::strong_count(&parent_only), 1, "parent released");
    assert_eq!(Arc::strong_count(&text), 2);
    assert_eq!(
        runtime::encode_json(selected).expect("owned child"),
        json!({"x":"owned é🙂"})
    );
    assert_eq!(Arc::strong_count(&text), 1);
}

#[test]
fn both_evaluators_support_nested_exact_output_lookup_and_content_policies() {
    let child = expression(json!([
        field("z", integer(i64::MAX)),
        field("a", integer(i64::MIN)),
        field("é🙂", json!({"kind":"string","value":"owned 🦀"})),
        field("null", json!({"kind":"null"})),
        field("bool", json!({"kind":"true"}))
    ]));
    let nested = expression(json!([
        field("child", child),
        field("empty", expression(json!([])))
    ]));
    let exact = r#"{"child":{"a":-9223372036854775808,"bool":true,"null":null,"z":9223372036854775807,"é🙂":"owned 🦀"},"empty":{}}"#;
    for result in both(nested.clone()) {
        assert_eq!(result.expect("nested output").to_string(), exact);
    }
    for result in both(lookup(lookup(nested.clone(), "child"), "a")) {
        assert_eq!(result.expect("chained lookup"), json!(i64::MIN));
    }
    for kind in ["recordLength", "length"] {
        for result in both(json!({"kind":kind,"operand":nested.clone()})) {
            assert_eq!(result.expect("direct field count"), json!(2));
        }
    }
    for (kind, expected) in [("equal", true), ("notEqual", false)] {
        for result in both(binary(kind, nested.clone(), nested.clone())) {
            assert_eq!(result.expect("content equality"), json!(expected));
        }
    }
    let different = expression(json!([
        field("child", expression(json!([]))),
        field("empty", expression(json!([])))
    ]));
    for result in both(binary("equal", nested.clone(), different.clone())) {
        assert_eq!(result.expect("nested content differs"), json!(false));
    }
    for result in both(binary("unify", nested.clone(), different)) {
        assert!(matches!(result, Err(Error::RecordConflict)));
    }
    // Identical certificates/field sets must not hide unequal nested scalar leaves.
    let leaf = |n| {
        expression(json!([field(
            "x",
            expression(json!([field("a", integer(n))]))
        )]))
    };
    for result in both(binary("equal", leaf(1), leaf(2))) {
        assert_eq!(result.expect("nested scalar differs"), json!(false));
    }
    for result in both(binary("unify", leaf(1), leaf(2))) {
        assert!(matches!(result, Err(Error::RecordConflict)));
    }
    for result in both(binary("unify", nested.clone(), nested.clone())) {
        assert_eq!(result.expect("closed identity").to_string(), exact);
    }
    for (kind, left, expected) in [("logicalAnd", false, false), ("logicalOr", true, true)] {
        for result in both(binary(
            kind,
            json!({"kind":if left { "true" } else { "false" }}),
            nested.clone(),
        )) {
            assert_eq!(result.expect("concrete logical RHS"), json!(expected));
        }
    }
}

#[test]
fn nested_records_keep_shared_ast_depth_and_visit_limits() {
    assert!(matches!(
        left_twice::execute(&program(expression(json!([])))),
        Err(Error::IntegerOutput)
    ));
    for (wrappers, accepted) in [(61, true), (62, false)] {
        let mut entry = integer(0);
        for _ in 0..wrappers {
            entry = json!({"kind":"negate","operand":entry});
        }
        entry = expression(json!([field("x", expression(json!([field("y", entry)])))]));
        for result in both(entry) {
            if accepted {
                assert_eq!(result.expect("64 active AST nodes"), json!({"x":{"y":0}}));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
    let mut spine = integer(0);
    for _ in 0..11 {
        spine = binary("add", spine, integer(0));
    }
    // Selected evaluator repeats left: 4095 child visits plus one record root.
    for (extra, accepted) in [(false, true), (true, false)] {
        let mut entries = vec![field("x", spine.clone())];
        if extra {
            entries.push(field("y", integer(0)));
        }
        let input = program(expression(json!(entries)));
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if accepted {
                assert_eq!(result.expect("4096 shared visits"), json!({"x":0}));
            } else {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            }
        }
    }
}

#[test]
fn actual_compiled_historical_and_reverse_controls_have_fixed_outcomes() {
    let values = [record(vec![]), singleton(V::Integer(7))];
    for (driver, reversed) in [
        (production::execute as RecordDriver, false),
        (reference, false),
        (reverse::execute, true),
        (reference_reverse, true),
    ] {
        let (result, trace) = run(&values, false, driver);
        assert_eq!(
            runtime::encode_json(result.expect("nested admission")).expect("JSON"),
            json!({"f0":{},"f1":{"x":7}})
        );
        assert_eq!(trace, if reversed { vec![1, 0] } else { vec![0, 1] });
    }
    for driver in [historical::execute as RecordDriver, reference_historical] {
        let (result, trace) = run(&values, false, driver);
        assert!(matches!(result, Err(Error::RecordElement { field }) if field == "f0"));
        assert_eq!(trace, vec![0]);
    }
}
