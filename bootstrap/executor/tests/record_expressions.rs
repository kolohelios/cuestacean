use bootstrap_executor::{Error, execute_json, generated, pratt, runtime, unification};
use runtime::{EvaluationContext, RecordDriver, RecordFrame, Value as V};
use serde_json::{Value as J, json};
use std::{path::Path, sync::Arc};

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
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn field(name: &str, value: J) -> J {
    json!({"name":name,"value":value})
}
fn record(fields: J) -> J {
    json!({"kind":"recordExpression","fields":fields})
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
fn forward(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    pratt::interpret_record(&export("record-control"), frame)
}
fn backward(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    pratt::interpret_record(&export("tests/valid/record-control-reverse"), frame)
}
fn control(body: J) -> String {
    json!({"formatVersion":1,"target":"record","body":body}).to_string()
}
fn direct(fields: J, driver: RecordDriver) -> (Result<V, Error>, Vec<J>) {
    let mut trace = Vec::new();
    let result = runtime::record_expression_with_driver(
        &record(fields),
        "fields",
        &mut EvaluationContext::default(),
        &mut |node, context| {
            trace.push(node.clone());
            context.visit(|_| match node["kind"].as_str() {
                Some("integer") => runtime::integer(node, "value"),
                Some("string") => runtime::text(node, "value"),
                Some("intType") => Ok(V::Int),
                Some("divide") => Err(Error::DivisionByZero),
                _ => Err(Error::Kind),
            })
        },
        driver,
    );
    (result, trace)
}

#[test]
fn computed_scalars_exact_output_lookup_and_literal_codec_are_distinct() {
    let entry = record(json!([
        field("sum", binary("add", integer(2), integer(3))),
        {"name":"bytes","value":{"kind":"stringLength","operand":{"kind":"string","value":"é🙂"}}},
        {"name":"lookup","value":{"kind":"recordIndex","left":{"kind":"record","fields":[{"name":"x","value":7}]},"right":{"kind":"string","value":"x"}}},
        field("min", integer(i64::MIN)),
        field("max", integer(i64::MAX)),
        field("bool", json!({"kind":"true"})),
        field("null", json!({"kind":"null"})),
        field("é🙂", json!({"kind":"string","value":"owned 🦀"}))
    ]));
    let expected = json!({"sum":5,"bytes":6,"lookup":7,"min":i64::MIN,"max":i64::MAX,
        "bool":true,"null":null,"é🙂":"owned 🦀"});
    for result in both(entry.clone()) {
        assert_eq!(result.expect("computed record"), expected);
    }
    for result in both(json!({"kind":"record","fields":[{"name":"x","value":3}]})) {
        assert_eq!(result.expect("raw scalar codec unchanged"), json!({"x":3}));
    }
}

#[test]
fn capacity_and_name_checks_precede_callbacks_but_not_earlier_admission() {
    for result in both(json!({"kind":"recordExpression"})) {
        assert!(matches!(result, Err(Error::MissingField { field }) if field == "fields"));
    }
    for fields in [json!(null), json!({}), json!(true)] {
        for result in both(record(fields)) {
            assert!(matches!(result, Err(Error::Array { field }) if field == "fields"));
        }
    }
    for result in both(record(json!(vec![json!({}); 65]))) {
        assert!(matches!(result, Err(Error::RecordCapacity)));
    }
    for count in [0, 64] {
        let fields: Vec<_> = (0..count)
            .map(|i| field(&format!("f{i}"), integer(i)))
            .collect();
        for result in both(record(json!(fields))) {
            let output = result.expect("bounded record");
            assert_eq!(output.as_object().expect("object").len(), count as usize);
        }
    }
    for driver in [
        production::execute as RecordDriver,
        forward,
        reverse::execute,
        backward,
    ] {
        let (result, trace) = direct(json!(vec![json!({}); 65]), driver);
        assert!(matches!(result, Err(Error::RecordCapacity)));
        assert!(trace.is_empty());
    }
    for name in ["a".repeat(64), "é".repeat(32)] {
        for result in both(record(json!([field(&name, integer(1))]))) {
            assert_eq!(result.expect("64 bytes")[&name], json!(1));
        }
    }
    for name in [String::new(), "a".repeat(65), "é".repeat(33)] {
        let (result, trace) = direct(json!([{"name":name}]), forward);
        assert!(matches!(result, Err(Error::RecordName { index: 0 })));
        assert!(trace.is_empty());
    }
    for (entry, expected) in [
        (json!({}), "missing field \"name\""),
        (json!({"name":1}), "field \"name\" must be a string"),
        (json!({"name":"x"}), "missing field \"value\""),
    ] {
        let (result, trace) = direct(json!([entry]), forward);
        assert_eq!(result.expect_err("malformed entry").to_string(), expected);
        assert!(trace.is_empty());
    }
    let (result, trace) = direct(json!([field("x", integer(1)), {"name":"x"}]), forward);
    assert!(matches!(result, Err(Error::RecordDuplicate { field }) if field == "x"));
    assert_eq!(trace, vec![integer(1)]);
    let divide = binary("divide", integer(1), integer(0));
    for child in [json!({"kind":"intType"}), divide.clone()] {
        for result in both(record(json!([field("first", child.clone()), {"name":""}]))) {
            if child == divide {
                assert!(matches!(result, Err(Error::DivisionByZero)));
            } else {
                assert!(matches!(result, Err(Error::RecordElement { field }) if field == "first"));
            }
        }
    }
    for result in both(record(json!([field("x", integer(1)), field("x", divide)]))) {
        assert!(matches!(result, Err(Error::RecordDuplicate { field }) if field == "x"));
    }
}

#[test]
fn flat_admission_and_real_reverse_control_have_independent_error_and_trace_evidence() {
    for child in [
        json!({"kind":"intType"}),
        json!({"kind":"list","value":[]}),
        json!({"kind":"listExpression","elements":[]}),
        json!({"kind":"record","fields":[]}),
        record(json!([])),
    ] {
        for result in both(record(json!([field("x", child)]))) {
            assert!(matches!(result, Err(Error::RecordElement { field }) if field == "x"));
        }
    }
    let abstract_value = json!({"kind":"intType"});
    let divide = binary("divide", integer(1), integer(0));
    for (driver, reversed) in [
        (production::execute as RecordDriver, false),
        (forward, false),
        (reverse::execute, true),
        (backward, true),
    ] {
        let (result, trace) = direct(
            json!([field("z", integer(1)), field("a", integer(2))]),
            driver,
        );
        assert_eq!(
            runtime::encode_json(result.expect("record")).expect("JSON"),
            json!({"a":2,"z":1})
        );
        let expected = if reversed {
            vec![integer(2), integer(1)]
        } else {
            vec![integer(1), integer(2)]
        };
        assert_eq!(trace, expected);
        let (result, trace) = direct(
            json!([
                field("first", abstract_value.clone()),
                field("last", divide.clone())
            ]),
            driver,
        );
        if reversed {
            assert!(matches!(result, Err(Error::DivisionByZero)));
            assert_eq!(trace, vec![divide.clone()]);
        } else {
            assert!(matches!(result, Err(Error::RecordElement { field }) if field == "first"));
            assert_eq!(trace, vec![abstract_value.clone()]);
        }
    }
}

#[test]
fn selected_evaluator_and_shared_context_keep_exact_ast_limits() {
    assert!(matches!(
        left_twice::execute(&program(record(json!([])))),
        Err(Error::IntegerOutput)
    ));
    let mut spine = integer(0);
    for _ in 0..11 {
        spine = binary("add", spine, integer(0));
    }
    // Selected add repeats left: child=4095 entries, record parent=1.
    for (fields, success) in [
        (json!([field("x", spine.clone())]), true),
        (json!([field("x", spine), field("y", integer(0))]), false),
    ] {
        let input = program(record(fields));
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if success {
                assert_eq!(result.expect("4096 visits"), json!({"x":0}));
            } else {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            }
        }
    }
    for (wrappers, success) in [(62, true), (63, false)] {
        let mut child = integer(0);
        for _ in 0..wrappers {
            child = json!({"kind":"negate","operand":child});
        }
        for result in both(record(json!([field("x", child)]))) {
            if success {
                assert_eq!(result.expect("64 active entries"), json!({"x":0}));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
}

#[test]
fn utf8_text_ownership_survives_input_record_and_callback_drop() {
    let text: Arc<str> = Arc::from("é🙂 owned");
    let node = record(json!([field("é🙂", integer(0))]));
    let value = runtime::record_expression_with_driver(
        &node,
        "fields",
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(V::Text(text.clone())),
        production::execute,
    )
    .expect("record");
    let selected = runtime::record_get(&value, &V::Text(Arc::from("é🙂"))).expect("lookup");
    let V::Text(selected) = selected else {
        panic!("text")
    };
    assert!(Arc::ptr_eq(&text, &selected));
    drop(node);
    drop(value);
    drop(text);
    assert_eq!(&*selected, "é🙂 owned");
}

#[test]
fn empty_return_and_stalled_control_obey_exact_fuel_and_completion() {
    fn empty<const N: usize, const VM: bool>(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
        for _ in 0..N {
            frame.step()?;
        }
        if VM {
            forward(frame)
        } else {
            production::execute(frame)
        }
    }
    for driver in [empty::<4093, false> as RecordDriver, empty::<4093, true>] {
        assert!(direct(json!([]), driver).0.is_ok());
    }
    for driver in [empty::<4094, false> as RecordDriver, empty::<4094, true>] {
        assert!(matches!(
            direct(json!([]), driver).0,
            Err(Error::RecordControlBudget)
        ));
    }
    fn returned<const N: usize>(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
        for _ in 0..N {
            frame.step()?;
        }
        pratt::interpret_record(&control(json!([{"op":"firstItem"},{"op":"return"}])), frame)
    }
    assert!(matches!(
        direct(json!([]), returned::<4094>).0,
        Err(Error::RecordState { .. })
    ));
    assert!(matches!(
        direct(json!([]), returned::<4095>).0,
        Err(Error::RecordControlBudget)
    ));
    fn stalled(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
        pratt::interpret_record(
            &control(json!([{"op":"while","condition":{"op":"boolean","value":true},"body":[]}])),
            frame,
        )
    }
    assert!(matches!(
        direct(json!([]), stalled).0,
        Err(Error::RecordControlBudget)
    ));
}

#[test]
fn targets_dead_arms_and_original_wire_validate_before_custom_backend() {
    let ignore =
        json!({"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}).to_string();
    let dead =
        |body: J| json!([{"op":"if","condition":{"op":"boolean","value":false},"then":body}]);
    let mut invalid = vec![
        r#"{"formatVersion":1,"target":"record","target":"list","body":[]}"#.to_string(),
        json!({"formatVersion":1,"target":"Record","body":[]}).to_string(),
        json!({"formatVersion":1,"target":"record","body":[],"extra":0}).to_string(),
    ];
    for op in [
        "finishList",
        "primary",
        "parseRight",
        "first_item",
        "action",
    ] {
        invalid.push(control(dead(json!([{"op":op}]))));
    }
    for condition in [
        json!({"op":"hasOperator"}),
        json!({"op":"itemKindIs","kind":"unknownTestKind"}),
    ] {
        invalid.push(control(dead(
            json!([{"op":"if","condition":condition,"then":[]}]),
        )));
    }
    invalid.push(control(dead(json!([{"op":"firstItem","extra":true}]))));
    for target in ["parser", "list"] {
        invalid.push(
            json!({"formatVersion":1,"target":target,"body":dead(json!([{"op":"finishRecord"}]))})
                .to_string(),
        );
    }
    for input in invalid {
        for backend in [ignore.as_str(), "not JSON"] {
            let error =
                pratt::generate_with_policy(&input, backend).expect_err("wire before backend");
            assert!(
                !matches!(error, pratt::Error::Lowering { .. }),
                "{input}: {error}"
            );
        }
        assert!(pratt::generate(&input).is_err(), "{input}");
    }
}
