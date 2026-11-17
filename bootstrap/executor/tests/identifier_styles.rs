use bootstrap_executor::lowering::{self, Context, Driver, Error, Policy};
use serde_json::{Value, json};

#[path = "../../generated/lowering-identifier.rs"]
mod compiled;

fn policy(expr: Value) -> String {
    json!({"formatVersion":1,"cases":{"root":expr}}).to_string()
}

fn identifier(flags: Value) -> Value {
    let mut expr = json!({"op":"identifier","field":"name"});
    expr.as_object_mut()
        .expect("identifier object")
        .extend(flags.as_object().expect("flag object").clone());
    expr
}

fn upper_policy() -> Policy {
    Policy::parse(&policy(identifier(json!({"upperFirst":true})))).expect("upper-first policy")
}

#[test]
fn fixed_styles_and_actual_compiled_upper_first() {
    for (raw, snaked, uppered) in [
        ("lessEqual", "less_equal", "LessEqual"),
        ("camelCase", "camel_case", "CamelCase"),
        ("URL", "u_r_l", "URL"),
        ("_name", "_name", "_name"),
        ("_URL9", "__u_r_l9", "_URL9"),
        ("x9Y", "x9_y", "X9Y"),
        ("type", "type", "Type"),
        ("self", "self", "Self"),
        ("a_b", "a_b", "A_b"),
        ("_", "_", "_"),
    ] {
        for (opcode, flags, expected) in [
            ("identity", json!({}), raw),
            ("identity", json!({"snake":false,"upperFirst":false}), raw),
            ("snake", json!({"snake":true}), snaked),
            ("root", json!({"upperFirst":true}), uppered),
        ] {
            let reference = Policy::parse(&policy(identifier(flags))).expect("style policy");
            assert_eq!(
                reference
                    .lower(&json!({"op":"root","name":raw}))
                    .expect("reference render"),
                expected
            );
            assert_eq!(
                compiled::lower(&json!({"op":opcode,"name":raw})).expect("compiled render"),
                expected
            );
        }
    }
    let node = json!({"op":"root","name":format!("a{}", "Z".repeat(63))});
    let expected = format!("A{}", "Z".repeat(63));
    assert_eq!(upper_policy().lower(&node).expect("64 bytes"), expected);
    assert_eq!(compiled::lower(&node).expect("compiled 64 bytes"), expected);
}

#[test]
fn rejects_entire_invalid_component_and_wrong_field_types() {
    let upper = upper_policy();
    for bad in [
        json!(""),
        json!("1a"),
        json!("a-b"),
        json!("é"),
        json!("aé"),
        json!("a::b"),
        json!("a/b"),
        json!("a\nb"),
        json!("a\r"),
        json!("a\0"),
        json!("a; panic!()"),
        json!("x".repeat(65)),
        json!(null),
        json!(true),
        json!(1),
        json!([]),
        json!({}),
    ] {
        let node = json!({"op":"root","name":bad});
        let expected = "lowering: field \"name\" must be an identifier component";
        assert_eq!(
            upper
                .lower(&node)
                .expect_err("invalid component")
                .to_string(),
            expected,
            "{bad}"
        );
        assert_eq!(
            compiled::lower(&node)
                .expect_err("compiled invalid component")
                .to_string(),
            expected,
            "{bad}"
        );
    }
    let node = json!({"op":"root"});
    let expected = "lowering: field \"name\" must be an identifier component";
    assert_eq!(
        upper
            .lower(&node)
            .expect_err("missing component")
            .to_string(),
        expected
    );
    assert_eq!(
        compiled::lower(&node)
            .expect_err("compiled missing component")
            .to_string(),
        expected
    );
}

#[test]
fn invalid_styles_are_eager_and_target_validation_precedes_override() {
    let conflict = identifier(json!({"snake":true,"upperFirst":true}));
    let input = policy(
        json!({"op":"ifTrue","field":"take","then":{"op":"text","value":"ok"},"else":conflict}),
    );
    let expected = "lowering: identifier styles are mutually exclusive";
    assert_eq!(
        Policy::parse(&input)
            .err()
            .expect("invalid target")
            .to_string(),
        expected
    );
    assert_eq!(
        lowering::generate_with_policy(&input, "{")
            .expect_err("target precedes compiler")
            .to_string(),
        expected
    );
    let unused =
        json!({"formatVersion":1,"cases":{"root":{"op":"text","value":"ok"},"unused":conflict}})
            .to_string();
    assert_eq!(
        Policy::parse(&unused)
            .err()
            .expect("unused invalid case")
            .to_string(),
        expected
    );
    for bad in [json!(null), json!(0), json!("true"), json!([]), json!({})] {
        assert!(Policy::parse(&policy(identifier(json!({"upperFirst":bad})))).is_err());
    }
    let duplicate = r#"{"formatVersion":1,"cases":{"root":{"op":"identifier","field":"name","upperFirst":false,"upperFirst":true}}}"#;
    assert!(Policy::parse(duplicate).is_err());
}

struct BudgetProbe {
    bytes: usize,
    conflict: bool,
}

impl Driver for BudgetProbe {
    fn render(&self, ctx: &mut Context, node: &Value, _: usize) -> Result<String, Error> {
        ctx.text(&"x".repeat(self.bytes))?;
        ctx.identifier(node, "name", self.conflict, true)
    }
}

#[test]
fn upper_first_shares_exact_byte_budget_and_preserves_validation_order() {
    let node = json!({"op":"root","name":format!("a{}", "Z".repeat(63))});
    assert_eq!(
        Context::run(
            &node,
            &BudgetProbe {
                bytes: 1024 * 1024 - 64,
                conflict: false
            }
        )
        .expect("exact byte budget"),
        format!("A{}", "Z".repeat(63))
    );
    let exhausted = BudgetProbe {
        bytes: 1024 * 1024,
        conflict: false,
    };
    let one_over = BudgetProbe {
        bytes: 1024 * 1024 - 63,
        conflict: false,
    };
    for probe in [&exhausted, &one_over] {
        assert_eq!(
            Context::run(&node, probe)
                .expect_err("byte budget")
                .to_string(),
            "lowering: lowering exceeds 1 MiB rendered bytes"
        );
    }
    let invalid = json!({"op":"root","name":format!("{}!", "a".repeat(63))});
    assert_eq!(
        Context::run(&invalid, &exhausted)
            .expect_err("validate before writing")
            .to_string(),
        "lowering: field \"name\" must be an identifier component"
    );
    let conflict = BudgetProbe {
        bytes: 0,
        conflict: true,
    };
    assert_eq!(
        Context::run(&json!({}), &conflict)
            .expect_err("defensive style check")
            .to_string(),
        "lowering: identifier styles are mutually exclusive"
    );
}
