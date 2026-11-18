use bootstrap_executor::{execute_json, generated};
use serde_json::{Value, json};

fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
}
fn both(entry: Value) -> [Result<Value, bootstrap_executor::Error>; 2] {
    let input = json!({"formatVersion":1,"entry":entry}).to_string();
    [
        execute_json(
            &input,
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/evaluation.json"
            )),
        ),
        generated::execute_json(&input),
    ]
}

#[test]
fn shared_reference_operands_preserve_results_and_left_first_errors() {
    for (kind, left, right, expected) in [
        ("add", 1, 2, 3),
        ("subtract", 1, 2, -1),
        ("multiply", 2, 3, 6),
        ("divide", -7, 3, -3),
        ("modulo", -7, 3, 2),
        ("quotient", -7, 3, -2),
        ("remainder", -7, 3, -1),
    ] {
        for result in both(binary(kind, integer(left), integer(right))) {
            assert_eq!(result.expect(kind), json!(expected));
        }
    }
    for result in both(binary(
        "listIndex",
        json!({"kind":"list","value":[1,2]}),
        integer(1),
    )) {
        assert_eq!(result.expect("lookup"), json!(2));
    }
    for kind in [
        "add",
        "subtract",
        "multiply",
        "divide",
        "modulo",
        "quotient",
        "remainder",
        "listIndex",
    ] {
        let zero = binary("divide", integer(1), integer(0));
        for entry in [
            binary(kind, zero.clone(), json!({"kind":"unknownTestKind"})),
            binary(kind, integer(1), zero),
        ] {
            for result in both(entry) {
                assert_eq!(
                    result.expect_err("child before dispatch").to_string(),
                    "integer division by zero"
                );
            }
        }
    }
}
