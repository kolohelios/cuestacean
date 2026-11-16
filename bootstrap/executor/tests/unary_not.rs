use bootstrap_executor::{Error, execute_json, generated, parser, runtime, scanner, unification};
use runtime::Value;
use serde_json::{Value as Json, json};
use std::{path::Path, process::Command, sync::Arc};

#[path = "../../generated/not-true.rs"]
mod alternate;
#[path = "../../generated/control-default.rs"]
mod binary_default;
#[path = "../../generated/unary-conditions.rs"]
mod conditions;
fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}
fn node(value: Json) -> Json {
    json!({"kind":"logicalNot","operand":value})
}
fn tree(entry: Json) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: Json) -> [Result<Json, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn parse_both(source: &str, grammar: &str) -> [Result<Json, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
#[test]
fn unary_control_has_independent_values_and_real_arity() {
    let control = export("logical-not");
    assert_eq!(
        binary_default::unify(Value::Boolean(true), Value::Boolean(false))
            .expect("legacy binary default"),
        Value::Boolean(false)
    );
    assert_eq!(
        unification::interpret(
            &export("tests/valid/control-default"),
            Value::Boolean(true),
            Value::Boolean(false)
        )
        .expect("binary reference default"),
        Value::Boolean(false)
    );
    for (input, expected) in [(true, false), (false, true)] {
        for result in [
            runtime::logical_not(Value::Boolean(input)),
            unification::interpret_unary(&control, Value::Boolean(input)),
        ] {
            assert_eq!(result.expect("unary result"), Value::Boolean(expected));
        }
        for result in both(node(json!({"kind":if input {"true"} else {"false"}}))) {
            assert_eq!(result.expect("JSON not"), json!(expected));
        }
    }
    for value in [
        Value::Integer(1),
        Value::Int,
        Value::Bool,
        Value::String,
        Value::Text(Arc::from("x")),
        Value::Bounds {
            lower: Some(1),
            upper: Some(1),
        },
    ] {
        for result in [
            runtime::logical_not(value.clone()),
            unification::interpret_unary(&control, value),
        ] {
            assert!(matches!(result, Err(Error::Logical)));
        }
    }
    assert_eq!(
        alternate::logical_not(Value::Boolean(true)).expect("compiled alternate"),
        Value::Boolean(true)
    );
    assert_eq!(
        unification::interpret_unary(&export("tests/valid/not-true"), Value::Boolean(true))
            .expect("reference alternate"),
        Value::Boolean(true)
    );
    assert_eq!(
        runtime::logical_not(Value::Boolean(true)).expect("production"),
        Value::Boolean(false)
    );
    assert!(unification::interpret(&control, Value::Boolean(true), Value::Boolean(false)).is_err());
    assert!(unification::interpret_unary(&export("unify"), Value::Boolean(true)).is_err());
    for (input, expected) in [
        (Value::Boolean(true), false),
        (Value::Boolean(false), false),
        (Value::Integer(1), true),
    ] {
        for result in [
            conditions::logical_not(input.clone()),
            unification::interpret_unary(&export("tests/valid/unary-conditions"), input),
        ] {
            assert_eq!(
                result.expect("unary condition short circuit"),
                Value::Boolean(expected)
            );
        }
    }
    let code = unification::generate(&control).expect("unary source");
    assert!(code.contains("fn logical_not(_left: Value)"));
    assert!(!code.contains("_right"));
    let unused_bad = [
        json!({"op":"input","input":"right"}),
        json!({"op":"integerSum"}),
        json!({"op":"bounds","lower":{"op":"empty"},"upper":{"op":"empty"}}),
        json!({"op":"fail","reason":"types"}),
        json!({"op":"unaryFail","reason":"logical"}),
    ];
    for bad in unused_bad {
        let input=json!({"formatVersion":1,"function":"logicalNot","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"boolean"},"then":{"op":"boolean","value":true},"else":bad}}).to_string();
        assert!(unification::generate(&input).is_err());
        assert!(unification::interpret_unary(&input, Value::Boolean(true)).is_err());
    }
    for condition in [
        json!({"op":"same"}),
        json!({"op":"kindIs","input":"right","kind":"boolean"}),
        json!({"op":"booleanIs","input":"right","value":true}),
        json!({"op":"textCompare","relation":"equal"}),
    ] {
        let input=json!({"formatVersion":1,"function":"logicalNot","result":{"op":"if","condition":condition,"then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}}).to_string();
        assert!(unification::generate(&input).is_err());
        assert!(unification::interpret_unary(&input, Value::Boolean(true)).is_err());
    }
}
#[test]
fn child_errors_precede_type_checks_and_legacy_output_is_unchanged() {
    for (operand, message) in [
        (
            json!({"kind":"divide","left":{"kind":"integer","value":1},"right":{"kind":"integer","value":0}}),
            "integer division by zero",
        ),
        (
            json!({"kind":"unify","left":{"kind":"true"},"right":{"kind":"false"}}),
            "conflicting boolean values true and false",
        ),
        (
            json!({"kind":"add","left":{"kind":"integer","value":i64::MAX},"right":{"kind":"integer","value":1}}),
            "signed 64-bit integer addition overflow",
        ),
    ] {
        for result in both(node(operand)) {
            assert_eq!(result.expect_err("child error").to_string(), message);
        }
    }
    let input = tree(node(json!({"kind":"true"})));
    assert!(matches!(
        bootstrap_executor::execute(&input, &export("evaluation")),
        Err(Error::IntegerOutput)
    ));
    assert!(matches!(
        generated::execute(&input),
        Err(Error::IntegerOutput)
    ));
}
#[test]
fn source_trees_metadata_and_resource_boundaries() {
    let grammar = export("grammar");
    for (source, expected) in [
        ("!true", node(json!({"kind":"true"}))),
        ("!!false", node(node(json!({"kind":"false"})))),
        (
            "!true == false",
            json!({"kind":"equal","left":node(json!({"kind":"true"})),"right":{"kind":"false"}}),
        ),
        ("! // skip\nfalse", node(json!({"kind":"false"}))),
        (
            "!-9223372036854775808",
            node(json!({"kind":"integer","value":i64::MIN})),
        ),
    ] {
        let expected: Json = serde_json::from_str(&tree(expected)).expect("expected tree");
        for result in parse_both(source, &grammar) {
            assert_eq!(result.expect("source tree"), expected, "{source}");
        }
    }
    let mut metadata: Json = serde_json::from_str(&grammar).expect("metadata");
    metadata
        .as_object_mut()
        .expect("object")
        .remove("logicalNot");
    for result in parse_both("!true", &metadata.to_string()) {
        assert!(result.is_err());
    }
    for result in parse_both("-1", &metadata.to_string()) {
        assert!(result.is_ok());
    }
    metadata["logicalNot"] = json!({"token":"!","kind":"customNot","operandField":"child"});
    for result in parse_both("!true", &metadata.to_string()) {
        assert_eq!(
            result.expect("custom constructor"),
            json!({"formatVersion":1,"entry":{"kind":"customNot","child":{"kind":"true"}}})
        );
    }
    metadata["logicalNot"]["operandField"] = json!("kind");
    assert!(parser::parse("!true", &metadata.to_string()).is_err());
    for (prefixes, accepted) in [(32, true), (33, false)] {
        let source = format!("{}true", "!".repeat(prefixes));
        for result in parse_both(&source, &grammar) {
            assert_eq!(result.is_ok(), accepted, "depth {prefixes}");
        }
    }
    let chain = vec!["!true"; 21].join(" && ");
    for (prefixes, accepted) in [(2, true), (3, false)] {
        let source = format!("{}({chain})", "!".repeat(prefixes));
        for result in parse_both(&source, &grammar) {
            assert_eq!(result.is_ok(), accepted, "node limit {prefixes}");
        }
    }
}
fn operation_mut<'a>(value: &'a mut Json, op: &str) -> Option<&'a mut Json> {
    if value.get("op").and_then(Json::as_str) == Some(op) {
        return Some(value);
    }
    match value {
        Json::Object(fields) => fields
            .values_mut()
            .find_map(|child| operation_mut(child, op)),
        Json::Array(items) => items.iter_mut().find_map(|child| operation_mut(child, op)),
        _ => None,
    }
}
#[test]
fn selected_control_propagates_through_not_and_incomplete_stages_reject() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let primary = export("primary");
    let mut low: Json = serde_json::from_str(&primary).expect("primary");
    operation_mut(&mut low, "parseNot").expect("not recursion")["minimum"]["value"] = json!(0);
    let parsed = parser::parse_interpreted_with_primary(
        "!true && false",
        &grammar,
        &pratt,
        &low.to_string(),
    )
    .expect("alternate minimum");
    assert_eq!(
        parsed,
        json!({"formatVersion":1,"entry":node(json!({"kind":"logicalAnd","left":{"kind":"true"},"right":{"kind":"false"}}))})
    );
    assert_eq!(
        generated::execute_json(&parsed.to_string()).expect("alternate result"),
        json!(true)
    );
    let right = parser::parse_interpreted_with_primary(
        "!(1 == 2 == false)",
        &grammar,
        &export("tests/valid/pratt-right"),
        &primary,
    )
    .expect("selected expression");
    assert_eq!(
        right,
        json!({"formatVersion":1,"entry":node(json!({"kind":"equal","left":{"kind":"integer","value":1},"right":{"kind":"equal","left":{"kind":"integer","value":2},"right":{"kind":"false"}}}))})
    );
    assert_eq!(
        generated::execute_json(&right.to_string()).expect("right-associated result"),
        json!(true)
    );
    let mut high: Json = serde_json::from_str(&primary).expect("primary");
    operation_mut(&mut high, "parseGroup").expect("group recursion")["minimum"]["value"] =
        json!(100);
    assert!(
        parser::parse_interpreted_with_primary(
            "!(true && false)",
            &grammar,
            &pratt,
            &high.to_string()
        )
        .expect_err("selected primary")
        .to_string()
        .contains("expected closing parenthesis")
    );
    assert!(
        parser::parse_interpreted_with_primary(
            "!true",
            &grammar,
            &pratt,
            &export("tests/valid/primary-no-atoms")
        )
        .is_err()
    );
    for omitted in ["parseNot", "buildNot"] {
        let mut broken: Json = serde_json::from_str(&primary).expect("primary");
        // Removing an action, rather than replacing it, tests unfinished dispatch.
        fn remove(value: &mut Json, op: &str) {
            match value {
                Json::Array(items) => {
                    items.retain(|x| x["op"] != op);
                    for item in items {
                        remove(item, op);
                    }
                }
                Json::Object(fields) => {
                    for item in fields.values_mut() {
                        remove(item, op);
                    }
                }
                _ => {}
            }
        }
        remove(&mut broken, omitted);
        assert!(matches!(
            parser::parse_interpreted_with_primary("!true", &grammar, &pratt, &broken.to_string()),
            Err(parser::Error::ControlState { .. })
        ));
    }
    for result in [
        scanner::not_prefix("λ!", 1),
        scanner::interpret(&export("not-prefix"), "λ!", 1),
    ] {
        assert!(matches!(result, Err(scanner::Error::Offset)));
    }
    let mut ignored: Json = serde_json::from_str(&primary).expect("primary");
    *operation_mut(&mut ignored, "hasNot").expect("not selection") =
        json!({"op":"boolean","value":false});
    assert_eq!(
        parser::parse_interpreted_with_primary("!true", &grammar, &pratt, &ignored.to_string())
            .expect_err("ignored candidate")
            .to_string(),
        "source byte 0: expected signed decimal integer"
    );
    let bound=json!({"formatVersion":1,"body":[{"op":"whitespace"},{"op":"scanGroup"},{"op":"scanNot"},{"op":"scanBound"},{"op":"advanceBound"},{"op":"whitespace"},{"op":"parseBound"}]}).to_string();
    assert_eq!(
        parser::parse_interpreted_with_primary(">=1", &grammar, &pratt, &bound)
            .expect("skip negation stage"),
        json!({"formatVersion":1,"entry":{"kind":"lowerBound","value":1}})
    );
    let inactive=json!({"formatVersion":1,"body":[{"op":"if","condition":{"op":"hasNot"},"then":[],"else":[]}]}).to_string();
    assert!(matches!(
        parser::parse_interpreted("!true", &grammar, &inactive),
        Err(parser::Error::ControlState { .. })
    ));
    let mut mutation: Json = serde_json::from_str(&primary).expect("primary");
    *operation_mut(&mut mutation, "advanceNot").expect("not commit") =
        json!({"op":"lookupOperator"});
    assert!(matches!(
        parser::parse_interpreted_with_primary("!true", &grammar, &pratt, &mutation.to_string()),
        Err(parser::Error::ControlState { .. })
    ));
}
#[test]
fn prefix_scanning_is_independent_of_infix_and_cli_is_atomic() {
    let program = export("not-prefix");
    for byte in 0_u8..=127 {
        let source = String::from_utf8(vec![b'!', byte]).expect("ASCII");
        for result in [
            scanner::not_prefix(&source, 0),
            scanner::interpret(&program, &source, 0),
        ] {
            let result = result.expect("scan");
            assert_eq!(
                (result.offset, result.flag),
                if byte == b'=' { (0, false) } else { (1, true) }
            );
        }
    }
    for (source, start, end, flag) in [
        ("!", 0, 1, true),
        ("!=", 0, 0, false),
        ("λ!", 2, 3, true),
        ("", 0, 0, false),
        ("?", 0, 0, false),
    ] {
        for result in [
            scanner::not_prefix(source, start),
            scanner::interpret(&program, source, start),
        ] {
            let scan = result.expect("prefix");
            assert_eq!((scan.offset, scan.flag), (end, flag));
        }
    }
    let path = std::env::temp_dir().join(format!("logical-not-{}.cue", std::process::id()));
    for (source, output, error) in [
        ("!true", "false\n", ""),
        ("!!false", "false\n", ""),
        ("!bool", "", "invalid logical operands"),
        ("!1", "", "invalid logical operands"),
        ("!(1 div 0)", "", "integer division by zero"),
    ] {
        std::fs::write(&path, source).expect("source");
        let result = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(result.status.success(), !output.is_empty());
        assert_eq!(result.stdout, output.as_bytes());
        if error.is_empty() {
            assert!(result.stderr.is_empty());
        } else {
            assert!(String::from_utf8_lossy(&result.stderr).contains(error));
        }
    }
    std::fs::remove_file(path).expect("remove source");
}
