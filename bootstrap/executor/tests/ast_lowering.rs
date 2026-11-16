use std::{fs, path::Path, process::Command};

use bootstrap_executor::{
    ast, compiled_ast_lowering,
    lowering::{self, Policy},
};
use serde_json::{Value, json};

#[path = "../../generated/ast-forced-integer.rs"]
mod forced;
#[path = "../../generated/lowering-numbers.rs"]
mod numbers;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn program(parameters: Value, result: Value) -> String {
    json!({"formatVersion":1,"parameters":parameters,"result":result}).to_string()
}

fn policy(cases: Value) -> String {
    json!({"formatVersion":1,"cases":cases}).to_string()
}

#[test]
fn default_backend_matches_reference_for_existing_builders() {
    let backend = export("ast-lowering");
    for name in [
        "ast-program",
        "ast-atom",
        "ast-integer",
        "ast-binary",
        "ast-unary",
        "tests/valid/ast-duplicate",
        "tests/valid/ast-escaped",
        "tests/valid/ast-program-v2",
    ] {
        let input = export(name);
        assert_eq!(
            ast::generate(&input).expect(name),
            ast::generate_with_policy(&input, &backend).expect(name),
            "{name}"
        );
    }
}

#[test]
fn alternate_policy_is_executed_in_generated_builder() {
    let input = export("tests/valid/ast-literal");
    let alternate = export("tests/valid/ast-lowering-forced");
    assert_eq!(ast::interpret(&input, &[]).expect("literal"), json!(7));
    assert_eq!(forced::build(), json!(99));
    assert_ne!(
        ast::generate_with_policy(&input, &alternate).expect("alternate"),
        ast::generate(&input).expect("default")
    );
}

#[test]
fn annotated_arguments_preserve_order_ownership_and_names() {
    let input = program(
        json!([
            {"name":"🦀 key", "type":"string"},
            {"name":"json λ", "type":"json"},
            {"name":"integer-名字", "type":"integer"}
        ]),
        json!({"op":"object","fields":[
            {"key":{"op":"argument","name":"🦀 key"},"value":{"op":"argument","name":"json λ"}},
            {"key":{"op":"string","value":"again"},"value":{"op":"argument","name":"json λ"}},
            {"key":{"op":"argument","name":"🦀 key"},"value":{"op":"argument","name":"integer-名字"}},
            {"key":{"op":"string","value":"n"},"value":{"op":"argument","name":"integer-名字"}}
        ]}),
    );
    let source = ast::generate(&input).expect("generate");
    assert!(source.contains("_arg0.clone()"));
    assert!(source.contains("_arg1.clone()"));
    assert!(!source.contains("_arg2.clone()"));
    assert!(!source.contains("🦀 key"));
    assert!(source.contains("(_arg0, Value::from(_arg2))"));
    assert_eq!(
        ast::interpret(&input, &[json!("k"), json!([1, 2]), json!(7)]).expect("interpret"),
        json!({"k":7,"again":[1,2],"n":7})
    );
    assert_eq!(
        source,
        ast::generate_with_policy(&input, &export("ast-lowering")).expect("reference")
    );
}

#[test]
fn deepest_and_largest_representable_builders_keep_their_limits() {
    let mut result = json!({"op":"integer","value":1});
    for _ in 0..16 {
        result =
            json!({"op":"object","fields":[{"key":{"op":"string","value":"k"},"value":result}]});
    }
    let input = program(json!([]), result.clone());
    assert!(ast::generate(&input).is_ok(), "depth16 accepted by backend");
    result = json!({"op":"object","fields":[{"key":{"op":"string","value":"k"},"value":result}]});
    assert!(ast::generate(&program(json!([]), result)).is_err());
    // Valid object trees contain an odd expression count: 31 fields = 63 nodes.
    let field =
        json!({"key":{"op":"argument","name":"same"},"value":{"op":"argument","name":"same"}});
    let params = json!([{"name":"same","type":"string"}]);
    let mut fields = vec![field.clone(); 31];
    let input = program(params.clone(), json!({"op":"object","fields":fields}));
    let source = ast::generate(&input).expect("63 expressions");
    assert_eq!(source.matches("_arg0.clone()").count(), 61);
    assert!(source.contains("Value::String(_arg0)"));
    assert_eq!(
        ast::interpret(&input, &[json!("v")]).expect("last field wins"),
        json!({"v":"v"})
    );
    fields.push(field);
    assert!(ast::generate(&program(params, json!({"op":"object","fields":fields}))).is_err());
}

#[test]
fn builder_validation_precedes_even_an_ignoring_policy() {
    let ignore = policy(json!({"program":{"op":"text","value":"ignored"}}));
    let invalid = [
        program(json!([]), json!({"op":"argument","name":"missing"})),
        program(json!([{"name":"x","type":"integer"}]), json!({"op":"object","fields":[{"key":{"op":"argument","name":"x"},"value":{"op":"integer","value":1}}]})),
        program(json!([{"name":"x","type":"string"},{"name":"x","type":"json"}]), json!({"op":"integer","value":1})),
        program(json!([{"name":"","type":"string"}]), json!({"op":"integer","value":1})),
        program(json!((0..17).map(|i| json!({"name":format!("p{i}"),"type":"json"})).collect::<Vec<_>>()), json!({"op":"integer","value":1})),
        r#"{"formatVersion":1,"formatVersion":1,"parameters":[],"result":{"op":"integer","value":1}}"#.to_owned(),
    ];
    for input in invalid {
        assert!(
            ast::generate_with_policy(&input, &ignore).is_err(),
            "{input}"
        );
    }
}

#[test]
fn primitive_projection_and_lazy_branches() {
    let text = policy(json!({
        "root":{"op":"join","field":"items","separator":"|"},
        "item":{"op":"ifZero","field":"count","then":{"op":"signed","field":"n"},"else":{"op":"text","value":"more"}}
    }));
    let p = Policy::parse(&text).expect("policy");
    let node = json!({"op":"root","items":[
        {"op":"item","count":0,"n":i64::MIN},
        {"op":"item","count":1},
        {"op":"item","count":0,"n":i64::MAX}
    ]});
    assert_eq!(
        p.lower(&node).expect("joined"),
        format!("{}|more|{}", i64::MIN, i64::MAX)
    );
    assert_eq!(
        p.lower(&json!({"op":"root","items":[]})).expect("empty"),
        ""
    );
    for node in [
        json!({"op":"root","items":false}),
        json!({"op":"item","count":-1}),
        json!({"op":"item","count":"0"}),
        json!({"op":"item","count":0,"n":true}),
        json!({"op":"item","count":0}),
    ] {
        assert!(p.lower(&node).is_err(), "{node}");
    }
}

#[test]
fn compiled_numbers_agree_with_reference_including_errors() {
    let p = Policy::parse(&export("tests/valid/lowering-numbers")).expect("numbers policy");
    let items = "items\"\\\nλ";
    assert_eq!(
        numbers::lower(&json!({"op":"root",(items):[]})).expect("compiled empty join"),
        ""
    );
    assert_eq!(numbers::lower(&json!({"op":"root",(items):[{"op":"item","count":0,"n":i64::MIN},{"op":"item","count":1},{"op":"item","count":0,"n":i64::MAX}]})).expect("compiled signed extrema"),"-9223372036854775808|\"\\\nλ|more|\"\\\nλ|9223372036854775807");
    for node in [
        json!({"op":"root",(items):[]}),
        json!({"op":"root",(items):[{"op":"item","count":0,"n":i64::MIN},{"op":"item","count":1},{"op":"item","count":0,"n":i64::MAX}]}),
        json!({"op":"root",(items):[{"op":"item","count":0}]}),
        json!({"op":"root",(items):[{"op":"item","count":-1}]}),
        json!({"op":"root",(items):false}),
    ] {
        assert_eq!(
            numbers::lower(&node).map_err(|e| e.to_string()),
            p.lower(&node).map_err(|e| e.to_string()),
            "{node}"
        );
    }
}

#[test]
fn all_templates_and_limits_are_checked() {
    for expr in [
        json!({"op":"signed","field":""}),
        json!({"op":"signed","field":"x".repeat(129)}),
        json!({"op":"join","field":"x","separator":"x".repeat(129)}),
        json!({"op":"ifZero","field":"x","then":{"op":"text","value":"ok"},"else":{"op":"unknown"}}),
    ] {
        let text = policy(json!({"unused":expr, "root":{"op":"text","value":"ok"}}));
        assert!(Policy::parse(&text).is_err(), "{text}");
        assert!(lowering::generate(&text).is_err(), "{text}");
    }
    let text = policy(
        json!({"root":{"op":"join","field":"items","separator":"x".repeat(128)},"item":{"op":"text","value":"x"}}),
    );
    let p = Policy::parse(&text).expect("boundary");
    let huge =
        json!({"op":"root","items":(0..5000).map(|_| json!({"op":"item"})).collect::<Vec<_>>()});
    assert!(p.lower(&huge).is_err());
}

#[test]
fn compiled_ast_driver_accepts_annotated_integer() {
    let node = json!({"op":"integer","value":i64::MIN});
    let p = Policy::parse(&export("ast-lowering")).expect("policy");
    assert_eq!(
        compiled_ast_lowering::lower(&node).expect("compiled"),
        "Value::from(-9223372036854775808i64)"
    );
    assert_eq!(
        p.lower(&node).expect("reference"),
        compiled_ast_lowering::lower(&node).expect("compiled")
    );
}

#[test]
fn cli_optional_policy_and_failures_do_not_print_partial_output() {
    let dir = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let input = dir.join("tests/valid/ast-literal.json");
    let backend = dir.join("tests/valid/ast-lowering-forced.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-ast-codegen");
    let output = Command::new(binary)
        .arg(&input)
        .arg(&backend)
        .output()
        .expect("CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        ast::generate_with_policy(
            &export("tests/valid/ast-literal"),
            &export("tests/valid/ast-lowering-forced")
        )
        .expect("alternate")
        .as_bytes()
    );
    assert!(output.stderr.is_empty());
    let invalid = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/invalid.json");
    for args in [
        vec![input.as_path(), Path::new("missing-policy")],
        vec![input.as_path(), invalid.as_path()],
        vec![input.as_path(), backend.as_path(), Path::new("extra")],
    ] {
        let output = Command::new(binary)
            .args(args)
            .output()
            .expect("CLI failure");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
