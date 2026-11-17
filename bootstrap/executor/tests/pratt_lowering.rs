use std::{fs, path::Path};

use bootstrap_executor::{compiled_pratt_lowering, execute, lowering::Policy, parser, pratt};
use serde_json::{Value, json};

#[path = "../../generated/lowering-collections.rs"]
mod collections;
#[path = "../../generated/pratt-no-increment.rs"]
mod no_increment;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn policy(cases: Value) -> Policy {
    Policy::parse(&json!({"formatVersion": 1, "cases": cases}).to_string()).expect("policy")
}

#[test]
fn compiled_collections_preserve_escaped_selectors_literals_and_empty_concat() {
    let policy =
        Policy::parse(&export("tests/valid/lowering-collections")).expect("collection policy");
    let empty = json!({"op":"root", "items\"\\\nλ":[]});
    assert_eq!(
        collections::lower(&empty).expect("compiled empty concat"),
        ""
    );
    let full = json!({"op":"root", "items\"\\\nλ":[{"op":"item\"\\\nλ","number\"\\\nλ":0},{"op":"item\"\\\nλ","number\"\\\nλ":u64::MAX}]});
    assert_eq!(
        collections::lower(&full).expect("compiled list"),
        "raw\"\\\nλ0raw\"\\\nλ18446744073709551615"
    );
    let nested = json!({"op":"root","items\"\\\nλ":[full.clone()]});
    assert_eq!(
        collections::lower(&nested).expect("selected list driver"),
        collections::lower(&full).expect("flat")
    );
    let work = json!({"op":"root","items\"\\\nλ":vec![json!({"op":"item\"\\\nλ","number\"\\\nλ":0});5000]});
    assert!(
        collections::lower(&work)
            .expect_err("compiled shared work")
            .to_string()
            .contains("4096 steps")
    );
    let bytes = json!({"op":"root","items\"\\\nλ":vec![json!({"op":"string","value":"x".repeat(65536)});16]});
    assert!(
        collections::lower(&bytes)
            .expect_err("compiled shared bytes")
            .to_string()
            .contains("1 MiB")
    );
    for node in [
        empty,
        full,
        nested,
        work,
        bytes,
        json!({"op":"item\"\\\nλ","number\"\\\nλ":-1}),
        json!({"op":"root"}),
        json!({"op":"root", "items\"\\\nλ":[null]}),
    ] {
        assert_eq!(
            collections::lower(&node).map_err(|e| e.to_string()),
            policy.lower(&node).map_err(|e| e.to_string())
        );
    }
}

#[test]
fn opcode_outputs_have_independent_expected_layout() {
    let p = Policy::parse(&export("pratt-lowering")).expect("backend");
    for (op, method) in [
        ("primary", "primary"),
        ("afterOperand", "after_operand"),
        ("afterWordOperator", "after_word_operator"),
        ("lookupOperator", "lookup_operator"),
        ("advanceOperator", "advance_operator"),
        ("parseRight", "parse_right"),
        ("buildBinary", "build_binary"),
        ("whitespace", "whitespace"),
        ("scanGroup", "scan_group"),
        ("advanceGroup", "advance_group"),
        ("closeGroup", "close_group"),
        ("scanBound", "scan_bound"),
        ("advanceBound", "advance_bound"),
        ("parseBound", "parse_bound"),
        ("scanIdentifier", "scan_identifier"),
        ("parseAtom", "parse_atom"),
        ("parseInteger", "parse_integer"),
        ("scanNegation", "scan_negation"),
        ("advanceNegation", "advance_negation"),
        ("buildNegation", "build_negation"),
    ] {
        let node = json!({"op":op});
        let expected = format!("_frame.step()?;\n_frame.{method}()?;\n");
        assert_eq!(p.lower(&node).expect(op), expected);
        assert_eq!(compiled_pratt_lowering::lower(&node).expect(op), expected);
    }
    for (op, expected) in [
        ("minimum", "_frame.minimum()"),
        ("precedence", "_frame.precedence()?"),
        ("tokenLength", "_frame.token_length()?"),
        ("hasOperator", "_frame.has_operator()"),
        ("leftAssociative", "_frame.left_associative()?"),
        ("groupOpen", "_frame.group_open()?"),
        ("hasBound", "_frame.has_bound()?"),
        ("hasIdentifier", "_frame.has_identifier()?"),
        ("hasNegation", "_frame.has_negation()?"),
    ] {
        assert_eq!(
            compiled_pratt_lowering::lower(&json!({"op":op})).expect(op),
            expected
        );
    }
    for (op, field, method) in [
        ("setBindingPower", "value", "set_binding_power"),
        ("parseGroup", "minimum", "parse_group"),
        ("parseNegation", "minimum", "parse_negation"),
    ] {
        assert_eq!(
            compiled_pratt_lowering::lower(&json!({"op":op,(field):{"op":"integer","value":255}}))
                .expect(op),
            format!("_frame.step()?;\n_frame.{method}(255u8)?;\n")
        );
    }
    let mut node = json!({"op":"if","condition":{"op":"boolean","value":true},"then":[{"op":"primary"}],"else":[]});
    assert_eq!(
        compiled_pratt_lowering::lower(&node).expect("empty else"),
        "_frame.step()?;\nif true {\n_frame.step()?;\n_frame.primary()?;\n}\n"
    );
    node["else"] = json!([{"op":"parseInteger"}]);
    assert_eq!(
        compiled_pratt_lowering::lower(&node).expect("nonempty else"),
        "_frame.step()?;\nif true {\n_frame.step()?;\n_frame.primary()?;\n} else {\n_frame.step()?;\n_frame.parse_integer()?;\n}\n"
    );
}

#[test]
fn optional_cli_policy_controls_generation_and_errors_without_output() {
    use std::process::Command;
    let dir = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let control = dir.join("pratt.json");
    let backend = dir.join("tests/valid/pratt-lowering-no-increment.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-pratt-codegen");
    let output = Command::new(binary)
        .arg(&control)
        .arg(&backend)
        .output()
        .expect("custom CLI");
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        pratt::generate_with_policy(
            &export("pratt"),
            &export("tests/valid/pratt-lowering-no-increment")
        )
        .expect("custom generation")
        .as_bytes()
    );
    assert!(output.stderr.is_empty());
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/invalid.json");
    for args in [
        vec![control.as_path(), Path::new("missing-policy")],
        vec![control.as_path(), invalid.as_path()],
        vec![control.as_path(), backend.as_path(), Path::new("extra")],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("CLI failure");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn default_compiled_and_reference_backends_lower_programs_identically() {
    let policy = Policy::parse(&export("pratt-lowering")).expect("Pratt lowering policy");
    for name in [
        "pratt",
        "tests/valid/primary-nested",
        "tests/valid/pratt-right",
        "tests/valid/pratt-stalled",
    ] {
        let input = export(name);
        let program: Value = serde_json::from_str(&input).expect("program");
        // These return-free bodies have a canonical value-producing footer.
        let tagged =
            json!({"op":"program", "body":program["body"], "tail":"value", "target":"parser"});
        assert_eq!(
            policy.lower(&tagged).expect("interpreted lowering"),
            compiled_pratt_lowering::lower(&tagged).expect("compiled lowering"),
            "{name}"
        );
    }
    for name in [
        "pratt",
        "primary",
        "tests/valid/primary-nested",
        "tests/valid/pratt-right",
        "tests/valid/pratt-stalled",
        "tests/valid/primary-return",
        "tests/valid/primary-return-empty",
        "tests/valid/pratt-return-loop",
    ] {
        // The native bridge supplies checked tail/position facts for returns.
        let input = export(name);
        assert_eq!(
            pratt::generate_with_policy(&input, &export("pratt-lowering")).expect("reference"),
            pratt::generate(&input).expect("compiled"),
            "{name}"
        );
    }
}

#[test]
fn text_concat_unsigned_list_and_if_empty_compose() {
    let p = policy(json!({
        "root":{"op":"concat", "parts":[
            {"op":"text", "value":"["},
            {"op":"ifEmpty", "field":"items", "then":{"op":"text", "value":"empty"},
             "else":{"op":"list", "field":"items"}},
            {"op":"text", "value":"]"}
        ]},
        "item":{"op":"concat", "parts":[
            {"op":"text", "value":"#"}, {"op":"unsigned", "field":"value"},
            {"op":"text", "value":";"}
        ]}
    }));
    assert_eq!(
        p.lower(&json!({"op":"root", "items":[]}))
            .expect("empty items"),
        "[empty]"
    );
    assert_eq!(
        p.lower(&json!({"op":"root", "items":[
            {"op":"item", "value":0}, {"op":"item", "value":u64::MAX}
        ]}))
        .expect("unsigned items"),
        "[#0;#18446744073709551615;]"
    );
    let empty = policy(json!({"x":{"op":"concat", "parts":[]}}));
    assert_eq!(empty.lower(&json!({"op":"x"})).expect("empty concat"), "");
}

#[test]
fn selectors_and_text_remain_data_when_compiled() {
    let selector = "quote\"\\\n🦀";
    let literal = "raw\"\\\n🦀";
    let document = json!({"formatVersion":1,"cases":{
        "x\"\\\n":{"op":"concat", "parts":[
            {"op":"text", "value":literal}, {"op":"string", "field":selector},
            {"op":"unsigned", "field":"number"}
        ]}
    }})
    .to_string();
    let p = Policy::parse(&document).expect("escaped policy");
    let input = json!({"op":"x\"\\\n", (selector):literal, "number":42});
    assert_eq!(
        p.lower(&input).expect("escaped output"),
        format!("{literal}{literal:?}42")
    );
    let generated =
        bootstrap_executor::lowering::generate(&document).expect("compile escaped policy");
    assert!(generated.contains(&format!("{selector:?}")));
    assert!(generated.contains(&format!("{literal:?}")));
    assert!(generated.contains(&format!("{:?}", "x\"\\\n")));
}

#[test]
fn invalid_templates_are_rejected_even_when_unused_or_unselected() {
    let invalid = [
        json!({"op":"ifEmpty", "field":"items", "then":{"op":"text","value":"ok"}, "else":{"op":"path","value":"bad path"}}),
        json!({"op":"ifEmpty", "field":"items", "then":{"op":"text","value":"ok"}, "else":{"op":"unsigned","field":""}}),
        json!({"op":"concat", "parts":[{"op":"text","value":"ok"},{"op":"list","field":""}]}),
        json!({"op":"text", "value":"x".repeat(65537)}),
        json!({"op":"list", "field":"x".repeat(129)}),
        json!({"op":"concat", "parts":"not an array"}),
    ];
    for expr in invalid {
        let input = json!({"formatVersion":1,"cases":{
            "used":{"op":"text","value":"ok"}, "unused":expr
        }})
        .to_string();
        assert!(Policy::parse(&input).is_err(), "accepted {input}");
        assert!(bootstrap_executor::lowering::generate(&input).is_err());
    }
    assert!(
        Policy::parse(
            &json!({"formatVersion":1,"cases":{
                "x":{"op":"text","value":"x".repeat(65536)}
            }})
            .to_string()
        )
        .is_ok()
    );
}

#[test]
fn wrong_field_types_and_bad_list_elements_fail_without_fallback() {
    let p = policy(json!({
        "n":{"op":"unsigned", "field":"value"},
        "l":{"op":"list", "field":"value"},
        "i":{"op":"ifEmpty", "field":"value", "then":{"op":"text","value":"yes"}, "else":{"op":"text","value":"no"}},
        "leaf":{"op":"text", "value":"ok"}
    }));
    for input in [
        json!({"op":"n"}),
        json!({"op":"n","value":-1}),
        json!({"op":"n","value":1.5}),
        json!({"op":"n","value":"1"}),
        json!({"op":"l"}),
        json!({"op":"l","value":null}),
        json!({"op":"l","value":[null]}),
        json!({"op":"l","value":[{"op":"missing"}]}),
        json!({"op":"i"}),
        json!({"op":"i","value":{}}),
    ] {
        assert!(p.lower(&input).is_err(), "accepted {input}");
    }
    assert_eq!(
        p.lower(&json!({"op":"i","value":[null]}))
            .expect("unselected invalid list"),
        "no"
    );
    assert_eq!(
        p.lower(&json!({"op":"l","value":[{"op":"leaf"}]}))
            .expect("one child"),
        "ok"
    );
}

#[test]
fn list_recursion_shares_depth_work_and_output_budgets() {
    let recursive = policy(json!({
        "branch":{"op":"list","field":"children"},
        "leaf":{"op":"text","value":"x"}
    }));
    let mut node = json!({"op":"leaf"});
    for _ in 0..65 {
        node = json!({"op":"branch","children":[node]});
    }
    assert!(
        recursive.lower(&node).is_err(),
        "recursive depth must be shared"
    );
    let many = json!({"op":"branch","children":vec![json!({"op":"leaf"}); 5000]});
    assert!(
        recursive.lower(&many).is_err(),
        "list elements must share work budget"
    );
    let big = policy(json!({
        "branch":{"op":"list","field":"children"},
        "leaf":{"op":"text","value":"x".repeat(65536)}
    }));
    let many = json!({"op":"branch","children":vec![json!({"op":"leaf"}); 20]});
    assert!(
        big.lower(&many).is_err(),
        "list elements must share output budget"
    );
}

#[test]
fn generation_validates_original_pratt_json_before_policy_execution() {
    let ignore = json!({"formatVersion":1,"cases":{
        "program":{"op":"text","value":"ignored"}
    }})
    .to_string();
    for input in [
        r#"{"formatVersion":1,"formatVersion":1,"body":[]}"#,
        r#"{"formatVersion":1,"body":[{"op":"primary","op":"primary"}]}"#,
        r#"{"formatVersion":1,"body":[{"op":"setBindingPower","value":{"op":"integer","value":999}}]}"#,
    ] {
        assert!(pratt::generate(input).is_err(), "accepted {input}");
        assert!(
            pratt::generate_with_policy(input, &ignore).is_err(),
            "accepted {input}"
        );
    }
}

#[test]
fn alternate_lowering_changes_actual_parser_associativity_and_overflow() {
    let control = export("pratt");
    let default = export("pratt-lowering");
    let alternate = export("tests/valid/pratt-lowering-no-increment");
    let normal_source = pratt::generate(&control).expect("default generation");
    assert_eq!(
        normal_source,
        pratt::generate_with_policy(&control, &default).expect("reference generation")
    );
    assert_ne!(
        normal_source,
        pratt::generate_with_policy(&control, &alternate).expect("alternate generation")
    );
    let grammar = export("grammar");
    let rules = export("evaluation");
    let text = "9223372036854775807 + 1 + -1";
    let normal = parser::parse(text, &grammar).expect("normal parse");
    let changed =
        parser::parse_with_driver(text, &grammar, no_increment::execute).expect("alternate parse");
    assert!(
        execute(&normal.to_string(), &rules)
            .expect_err("left-associative overflow")
            .to_string()
            .contains("overflow")
    );
    assert_eq!(
        execute(&changed.to_string(), &rules).expect("right-associative result"),
        i64::MAX
    );
    assert_eq!(normal["entry"]["left"]["kind"], "add");
    assert_eq!(changed["entry"]["right"]["kind"], "add");
}
