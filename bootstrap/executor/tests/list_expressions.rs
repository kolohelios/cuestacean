use bootstrap_executor::{
    Error, execute, execute_json, generated, parser, pratt, runtime, unification,
};
use runtime::{EvaluationContext, ListDriver, ListFrame, Value as V};
use serde_json::{Value as J, json};
use std::path::Path;

#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/list-control.rs"]
mod production;
#[path = "../../generated/list-control-reverse.rs"]
mod reverse;
#[path = "../../generated/list-control-stalled.rs"]
mod stalled;
#[path = "../../generated/list-control-unfinished.rs"]
mod unfinished;

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
fn list(elements: J) -> J {
    json!({"kind":"listExpression","elements":elements})
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
fn forward(frame: &mut ListFrame<'_>) -> Result<(), Error> {
    pratt::interpret_list(&export("list-control"), frame)
}
fn control(body: J) -> String {
    json!({"formatVersion":1,"target":"list","body":body}).to_string()
}
fn direct(elements: J, driver: ListDriver) -> (Result<J, Error>, Vec<J>) {
    let mut trace = Vec::new();
    let mut ctx = EvaluationContext::default();
    let result = runtime::list_expression_with_driver(
        &list(elements),
        "elements",
        &mut ctx,
        &mut |node, ctx| {
            trace.push(node.clone());
            ctx.visit(|_| match node["kind"].as_str() {
                Some("intType") => Ok(V::Int),
                Some("integer") => runtime::integer(node, "value"),
                _ => Err(Error::Kind),
            })
        },
        driver,
    )
    .and_then(runtime::encode_json);
    (result, trace)
}

#[test]
fn computed_scalars_empty_capacity_exact_output_and_legacy_boundary() {
    let elements = json!([
        binary("add", integer(2), integer(3)),
        {"kind":"stringLength","operand":{"kind":"string","value":"é🙂"}},
        binary("less", integer(1), integer(2)),
        {"kind":"logicalNot","operand":{"kind":"false"}},
        {"kind":"null"}, {"kind":"string","value":"é🙂"},
        integer(i64::MIN), integer(i64::MAX)
    ]);
    let exact = r#"[5,6,true,true,null,"é🙂",-9223372036854775808,9223372036854775807]"#;
    for result in both(list(elements)) {
        assert_eq!(result.expect("computed scalars").to_string(), exact);
    }
    for elements in [json!([]), json!(vec![integer(0); 64])] {
        let expected = if elements.as_array().expect("element array").is_empty() {
            json!([])
        } else {
            json!(vec![0; 64])
        };
        for result in both(list(elements)) {
            assert_eq!(result.expect("bounded list"), expected);
        }
    }
    let mut oversized = vec![integer(0); 65];
    oversized[0] = json!({});
    for result in both(list(json!(oversized))) {
        assert!(matches!(result, Err(Error::ListCapacity)));
    }
    for elements in [json!(null), json!({}), json!(1)] {
        for result in both(list(elements)) {
            assert!(matches!(result, Err(Error::Array { .. })));
        }
    }
    for result in both(json!({"kind":"listLength","operand":list(json!([integer(4),integer(5)]))}))
    {
        assert_eq!(result.expect("list length"), json!(2));
    }
    let input = program(list(json!([])));
    for result in [
        execute(&input, &export("evaluation")),
        generated::execute(&input),
    ] {
        assert!(matches!(result, Err(Error::IntegerOutput)));
    }
}

#[test]
fn admission_is_per_item_and_child_error_precedence_is_independent() {
    let divide = binary("divide", integer(1), integer(0));
    let abstract_value = json!({"kind":"intType"});
    for result in both(list(json!([abstract_value, divide]))) {
        assert!(matches!(result, Err(Error::ListElement { index: 0 })));
    }
    for result in both(list(json!([divide, abstract_value]))) {
        assert!(matches!(result, Err(Error::DivisionByZero)));
    }
    for bad in [
        abstract_value,
        json!({"kind":"top"}),
        list(json!([])),
        json!({"kind":"list","value":[]}),
    ] {
        for result in both(list(json!([integer(1), bad]))) {
            assert!(matches!(result, Err(Error::ListElement { index: 1 })));
        }
    }
}

#[test]
fn compiled_reverse_changes_iteration_and_admission_order_with_trace() {
    let values = json!([integer(1), integer(2)]);
    let (normal, trace) = direct(values.clone(), forward);
    assert_eq!(normal.expect("forward"), json!([1, 2]));
    assert_eq!(trace, vec![integer(1), integer(2)]);
    let (reversed, trace) = direct(values.clone(), reverse::execute);
    assert_eq!(reversed.expect("reverse"), json!([2, 1]));
    assert_eq!(trace, vec![integer(2), integer(1)]);
    fn reference_reverse(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        pratt::interpret_list(&export("tests/valid/list-control-reverse"), frame)
    }
    assert_eq!(
        direct(values, reference_reverse).0.expect("VM reverse"),
        json!([2, 1])
    );
    let bad = json!({"kind":"intType"});
    for (driver, index) in [
        (forward as ListDriver, 0),
        (reverse::execute as ListDriver, 1),
    ] {
        let (result, trace) = direct(json!([bad, bad]), driver);
        assert!(matches!(result, Err(Error::ListElement { index: actual }) if actual == index));
        assert_eq!(
            trace,
            vec![bad.clone()],
            "stop immediately at failed admission"
        );
    }
    let (result, trace) = direct(json!(vec![json!({}); 65]), forward);
    assert!(matches!(result, Err(Error::ListCapacity)));
    assert!(trace.is_empty(), "capacity before callbacks");
}

#[test]
fn checked_stages_unfinished_results_and_exact_control_fuel() {
    fn bad_stage(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.append_item()
    }
    fn early_finish(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        frame.finish_list()
    }
    fn move_before_append(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        frame.next_item()
    }
    for driver in [bad_stage as ListDriver, early_finish, move_before_append] {
        assert!(matches!(
            direct(json!([integer(1)]), driver).0,
            Err(Error::ListState { .. })
        ));
    }
    for driver in [unfinished::execute as ListDriver, |_frame| Ok(())] {
        let error = direct(json!([integer(1)]), driver)
            .0
            .expect_err("unfinished");
        assert_eq!(
            error.to_string(),
            "invalid list control state: unfinished list result"
        );
    }
    fn exact_fuel(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        for _ in 0..4096 {
            frame.step()?;
        }
        assert!(matches!(frame.step(), Err(Error::ListControlBudget)));
        frame.first_item()?;
        frame.finish_list()
    }
    assert_eq!(
        direct(json!([]), exact_fuel)
            .0
            .expect("native methods do not charge"),
        json!([])
    );
    for driver in [stalled::execute as ListDriver, |frame| {
        pratt::interpret_list(&export("tests/valid/list-control-stalled"), frame)
    }] {
        let error = direct(json!([]), driver).0.expect_err("fuel");
        assert_eq!(error.to_string(), "list control exceeds 4096 steps");
    }
    fn returning_loop(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        pratt::interpret_list(
            &control(json!([
                {"op":"firstItem"},
                {"op":"while","condition":{"op":"boolean","value":true},"body":[{"op":"return"}]},
                {"op":"finishList"}
            ])),
            frame,
        )
    }
    assert_eq!(
        direct(json!([]), returning_loop)
            .0
            .expect_err("return unwinds loop")
            .to_string(),
        "invalid list control state: unfinished list result"
    );
}

#[test]
fn final_false_test_and_return_have_independent_exact_fuel_boundaries() {
    fn empty<const N: usize, const REFERENCE: bool>(
        frame: &mut ListFrame<'_>,
    ) -> Result<(), Error> {
        for _ in 0..N {
            frame.step()?;
        }
        if REFERENCE {
            forward(frame)
        } else {
            production::execute(frame)
        }
    }
    fn returned<const N: usize, const REFERENCE: bool>(
        frame: &mut ListFrame<'_>,
    ) -> Result<(), Error> {
        for _ in 0..N {
            frame.step()?;
        }
        if REFERENCE {
            pratt::interpret_list(&export("tests/valid/list-control-unfinished"), frame)
        } else {
            unfinished::execute(frame)
        }
    }
    // Empty default: firstItem + terminating false while test + finishList = 3.
    for driver in [empty::<4093, false> as ListDriver, empty::<4093, true>] {
        assert_eq!(direct(json!([]), driver).0.expect("exact 4096"), json!([]));
    }
    for driver in [empty::<4094, false> as ListDriver, empty::<4094, true>] {
        assert!(matches!(
            direct(json!([]), driver).0,
            Err(Error::ListControlBudget)
        ));
    }
    // Unfinished control: firstItem + return = 2, then completion must reject it.
    for driver in [
        returned::<4094, false> as ListDriver,
        returned::<4094, true>,
    ] {
        assert!(matches!(
            direct(json!([]), driver).0,
            Err(Error::ListState { .. })
        ));
    }
    for driver in [
        returned::<4095, false> as ListDriver,
        returned::<4095, true>,
    ] {
        assert!(matches!(
            direct(json!([]), driver).0,
            Err(Error::ListControlBudget)
        ));
    }
    fn unchecked_append(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        frame.evaluate_item()?;
        frame.append_item()?;
        frame.next_item()?;
        frame.finish_list()
    }
    // A trusted driver may omit admission; final representation validation cannot.
    assert!(matches!(
        direct(json!([{"kind":"intType"}]), unchecked_append).0,
        Err(Error::ListElement { index: 0 })
    ));
}

#[test]
fn selected_repeated_evaluator_retains_one_shared_ast_budget() {
    let mut spine = integer(0);
    for _ in 0..11 {
        spine = binary("add", spine, integer(0));
    }
    // The selected add rule visits its left twice: T(11)=4095. Parent list=1.
    for (elements, success) in [(json!([spine]), true), (json!([spine, integer(0)]), false)] {
        let input = program(list(elements));
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if success {
                assert_eq!(result.expect("4096 visits"), json!([0]));
            } else {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            }
        }
    }
    assert!(matches!(
        left_twice::execute(&program(list(json!([integer(0)])))),
        Err(Error::IntegerOutput)
    ));
    let mut child = integer(0);
    for _ in 0..63 {
        child = json!({"kind":"negate","operand":child});
    }
    for result in both(list(json!([child]))) {
        assert!(matches!(result, Err(Error::EvaluationDepth)));
    }
    // Native facade does not visit the parent; callers/callbacks own AST entries.
    fn charge_children(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        forward(frame)
    }
    let mut ctx = EvaluationContext::default();
    for index in 0..4097 {
        let result = runtime::list_expression_with_driver(
            &list(json!([integer(0)])),
            "elements",
            &mut ctx,
            &mut |node, ctx| ctx.visit(|_| runtime::integer(node, "value")),
            charge_children,
        );
        if index < 4096 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(Error::EvaluationVisits)));
        }
    }
}

#[test]
fn targets_unknown_wire_and_dead_families_validate_before_backend() {
    let ignore =
        json!({"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}).to_string();
    let dead = |statement: J| json!([{"op":"if","condition":{"op":"boolean","value":false},"then":[statement]}]);
    let mut invalid = vec![
        r#"{"formatVersion":1,"target":"list","target":"parser","body":[]}"#.to_string(),
        json!({"formatVersion":1,"target":"List","body":[]}).to_string(),
        json!({"formatVersion":1,"target":"list","body":[],"extra":0}).to_string(),
    ];
    for op in [
        "primary",
        "afterOperand",
        "parseRight",
        "scanGroup",
        "parseInteger",
        "lookupOperator",
    ] {
        invalid.push(control(dead(json!({"op":op}))));
    }
    invalid.push(control(dead(
        json!({"op":"setBindingPower","value":{"op":"minimum"}}),
    )));
    for condition in [
        json!({"op":"hasOperator"}),
        json!({"op":"groupOpen"}),
        json!({"op":"atLeast","left":{"op":"integer","value":0},"right":{"op":"minimum"}}),
        json!({"op":"or","left":{"op":"boolean","value":false},"right":{"op":"hasItem"}}),
        json!({"op":"itemKindIs","kind":"unknownTestKind"}),
    ] {
        invalid.push(control(
            json!([{"op":"while","condition":{"op":"boolean","value":false},"body":[
                {"op":"if","condition":condition,"then":[]}
            ]}]),
        ));
    }
    for op in [
        "firstItem",
        "lastItem",
        "evaluateItem",
        "appendItem",
        "nextItem",
        "previousItem",
        "rejectItem",
        "finishList",
    ] {
        invalid.push(json!({"formatVersion":1,"body":dead(json!({"op":op}))}).to_string());
    }
    for condition in [
        json!({"op":"hasItem"}),
        json!({"op":"itemKindIs","kind":"integer"}),
    ] {
        invalid.push(json!({"formatVersion":1,"target":"parser","body":[{"op":"if","condition":condition,"then":[]}]}).to_string());
    }
    for body in [
        json!([{"op":"first_item"}]),
        json!([{"op":"firstItem","extra":true}]),
    ] {
        invalid.push(control(body));
    }
    for input in invalid {
        let error =
            pratt::generate_with_policy(&input, &ignore).expect_err("whole family validates first");
        assert!(
            !matches!(error, pratt::Error::Lowering { .. }),
            "{input}: {error}"
        );
        let bad_backend =
            pratt::generate_with_policy(&input, "not JSON").expect_err("control before backend");
        assert!(
            !matches!(bad_backend, pratt::Error::Lowering { .. }),
            "{input}: {bad_backend}"
        );
        assert!(pratt::generate(&input).is_err(), "{input}");
    }
    let parser_return =
        json!({"formatVersion":1,"target":"parser","body":[{"op":"return"}]}).to_string();
    fn wrong_target(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        pratt::interpret_list(r#"{"formatVersion":1,"body":[{"op":"return"}]}"#, frame)
    }
    assert!(direct(json!([]), wrong_target).0.is_err());
    assert!(pratt::generate(&parser_return).is_ok());
    let list_return = control(json!([{"op":"return"}]));
    let error =
        parser::parse_interpreted("1", &export("grammar"), &list_return).expect_err("wrong frame");
    assert_eq!(
        error.to_string(),
        "invalid parser control state: parser control target required"
    );
}
