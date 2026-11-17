use bootstrap_executor::unification;
use serde_json::{Value, json};

fn control(result: Value) -> String {
    json!({"formatVersion":1,"result":result}).to_string()
}

fn branch(condition: Value) -> Value {
    json!({"op":"if","condition":condition,"then":{"op":"integer","value":0},"else":{"op":"integer","value":1}})
}

#[test]
fn cue_styles_preserve_every_closed_enum_spelling() {
    for (wire, rust) in [
        ("integer", "Integer"),
        ("int", "Int"),
        ("bounds", "Bounds"),
        ("boolean", "Boolean"),
        ("bool", "Bool"),
        ("text", "Text"),
        ("string", "String"),
        ("null", "Null"),
        ("top", "Top"),
    ] {
        let input = control(branch(json!({"op":"kindIs","input":"left","kind":wire})));
        assert!(
            unification::generate(&input)
                .expect("kind")
                .contains(&format!("Kind::{rust}"))
        );
    }
    for (wire, rust) in [("lower", "Lower"), ("upper", "Upper")] {
        let input = control(
            json!({"op":"bounds","lower":{"op":"input","input":"left","endpoint":wire},"upper":{"op":"empty"}}),
        );
        assert!(
            unification::generate(&input)
                .expect("endpoint")
                .contains(&format!("Endpoint::{rust}"))
        );
    }
    for (wire, rust) in [
        ("equal", "Equal"),
        ("less", "Less"),
        ("greater", "Greater"),
        ("lessEqual", "LessEqual"),
        ("greaterEqual", "GreaterEqual"),
    ] {
        let input = control(branch(
            json!({"op":"compare","left":{"op":"integer","value":0},"right":{"op":"integer","value":1},"relation":wire}),
        ));
        assert!(
            unification::generate(&input)
                .expect("relation")
                .contains(&format!("Relation::{rust}"))
        );
    }
    for (wire, rust) in [
        ("integers", "Integers"),
        ("bounds", "Bounds"),
        ("booleans", "Booleans"),
        ("types", "Types"),
        ("strings", "Strings"),
        ("arithmetic", "Arithmetic"),
        ("comparison", "Comparison"),
        ("ordering", "Ordering"),
        ("logical", "Logical"),
        ("divisionByZero", "DivisionByZero"),
    ] {
        let input = control(json!({"op":"fail","reason":wire}));
        assert!(
            unification::generate(&input)
                .expect("failure")
                .contains(&format!("Failure::{rust}"))
        );
    }
}

#[test]
fn semantic_names_still_reject_before_compiler_policy() {
    for bad in ["unknownName", "Self", "x::y", "x); panic!()"] {
        for result in [
            branch(json!({"op":"kindIs","input":"left","kind":bad})),
            json!({"op":"bounds","lower":{"op":"input","input":"left","endpoint":bad},"upper":{"op":"empty"}}),
            branch(
                json!({"op":"compare","left":{"op":"integer","value":0},"right":{"op":"integer","value":1},"relation":bad}),
            ),
            json!({"op":"fail","reason":bad}),
        ] {
            // Bury the invalid enum in an otherwise unselected value branch.
            let input = control(
                json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":{"op":"integer","value":0},"else":result}),
            );
            let error = unification::generate(&input)
                .expect_err("closed enum")
                .to_string();
            assert!(error.contains("unknown variant"), "{error}");
            assert_eq!(
                unification::generate_with_policy(&input, "{")
                    .expect_err("target validation first")
                    .to_string(),
                error
            );
        }
    }
}
