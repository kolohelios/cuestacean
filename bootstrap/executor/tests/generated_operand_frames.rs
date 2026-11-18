use bootstrap_executor::{Error, execute_json, generated};
use serde_json::{Value, json};

fn both(entry: Value) -> [Result<Value, Error>; 2] {
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
fn wide_operations_keep_the_existing_active_node_boundary() {
    for kind in ["listSlice", "unify"] {
        for wrappers in [63, 64] {
            let mut entry = json!({"kind":"list","value":[]});
            for _ in 0..wrappers {
                entry = if kind == "listSlice" {
                    json!({"kind":kind,"operand":entry,"lower":{"kind":"integer","value":0},"upper":{"kind":"integer","value":0}})
                } else {
                    json!({"kind":kind,"left":entry,"right":{"kind":"top"}})
                };
            }
            for result in both(entry) {
                if wrappers == 63 {
                    assert_eq!(result.expect("64 active nodes"), json!([]));
                } else {
                    assert!(matches!(result, Err(Error::EvaluationDepth)));
                }
            }
        }
    }
}

#[test]
fn ternary_closure_keeps_order_and_shared_child_errors() {
    let divide = json!({"kind":"divide","left":{"kind":"integer","value":1},"right":{"kind":"integer","value":0}});
    for entry in [
        json!({"kind":"listSlice","operand":divide,"lower":{"kind":"unknownTestKind"},"upper":{"kind":"integer","value":0}}),
        json!({"kind":"listSlice","operand":{"kind":"integer","value":0},"lower":divide,"upper":{"kind":"unknownTestKind"}}),
        json!({"kind":"listSlice","operand":{"kind":"integer","value":0},"lower":{"kind":"integer","value":0},"upper":divide}),
    ] {
        for result in both(entry) {
            assert!(matches!(result, Err(Error::DivisionByZero)));
        }
    }
}
