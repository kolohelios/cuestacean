use std::path::Path;
use std::process::Command;

use bootstrap_executor::ast;
use serde::Deserialize;
use serde_json::Value;

#[path = "../../generated/ast-duplicate.rs"]
mod duplicate;
#[path = "../../generated/ast-escaped.rs"]
mod escaped;
#[path = "../../generated/ast-program-v2.rs"]
mod version_two;

fn program(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("builder export")
}
fn string(arguments: &[Value], index: usize) -> String {
    arguments[index]
        .as_str()
        .expect("string argument")
        .to_owned()
}

#[derive(Deserialize)]
struct Case {
    builder: String,
    arguments: Vec<Value>,
    expected: Value,
}

#[test]
fn compiled_and_interpreted_builders_match_known_json_expectations() {
    let cases: Vec<Case> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/ast-cases.json").expect("cases"),
    )
    .expect("decode cases");
    for case in cases {
        assert_eq!(
            ast::interpret(&program(&case.builder), &case.arguments).expect("interpreted"),
            case.expected
        );
        let args = &case.arguments;
        let compiled = match case.builder.as_str() {
            "ast-program" => ast::program(args[0].clone()),
            "ast-atom" => ast::atom(string(args, 0)),
            "ast-integer" => ast::integer(
                string(args, 0),
                string(args, 1),
                args[2].as_i64().expect("integer"),
            ),
            "ast-binary" => ast::binary(
                string(args, 0),
                string(args, 1),
                string(args, 2),
                args[3].clone(),
                args[4].clone(),
            ),
            "tests/valid/ast-program-v2" => version_two::build(args[0].clone()),
            "tests/valid/ast-escaped" => escaped::build(string(args, 0)),
            "tests/valid/ast-duplicate" => duplicate::build(args[0].clone()),
            other => panic!("unknown test builder {other}"),
        };
        assert_eq!(compiled, case.expected, "{}", case.builder);
    }
    for name in ["ast-program", "ast-atom", "ast-integer", "ast-binary"] {
        assert!(
            !ast::generate(&program(name))
                .expect("generated")
                .contains(".clone()"),
            "normal builders should move each argument once"
        );
    }
    assert!(
        ast::generate(&program("tests/valid/ast-duplicate"))
            .expect("duplicate")
            .contains("_arg0.clone()")
    );
}

#[test]
fn generated_programs_match_schema_exported_trees() {
    for name in [
        "literal",
        "addition",
        "multiplication",
        "unification",
        "int-constraint",
        "bounds",
    ] {
        let expected: Value = serde_json::from_str(&program(name)).expect("exported tree");
        assert_eq!(ast::program(expected["entry"].clone()), expected);
    }
}

#[derive(Deserialize)]
struct InvalidCase {
    program: Value,
    error: String,
}

#[test]
fn invalid_bindings_types_and_json_are_rejected_before_emission() {
    let cases: Vec<InvalidCase> = serde_json::from_str(
        &std::fs::read_to_string("tests/fixtures/ast-invalid-cases.json").expect("cases"),
    )
    .expect("decode cases");
    for case in cases {
        let input = serde_json::to_string(&case.program).expect("input");
        for error in [
            ast::generate(&input).expect_err("generation"),
            ast::interpret(&input, &[]).expect_err("interpretation"),
        ] {
            assert!(error.to_string().contains(&case.error), "{error}");
        }
    }
    assert!(
        ast::interpret(&program("ast-atom"), &[])
            .expect_err("count")
            .to_string()
            .contains("argument count")
    );
    assert!(
        ast::interpret(&program("ast-atom"), &[Value::from(3)])
            .expect_err("type")
            .to_string()
            .contains("argument type")
    );
    assert!(
        ast::interpret(
            &program("ast-integer"),
            &["integer".into(), "value".into(), true.into()]
        )
        .is_err()
    );
}

#[test]
fn parameter_limits_and_empty_objects_are_checked() {
    let mut input: Value = serde_json::from_str(&program("ast-atom")).expect("builder");
    let parameter = input["parameters"][0].clone();
    for index in 1..16 {
        let mut extra = parameter.clone();
        extra["name"] = format!("unused{index}").into();
        input["parameters"]
            .as_array_mut()
            .expect("parameters")
            .push(extra);
    }
    ast::generate(&serde_json::to_string(&input).expect("input")).expect("16 parameters allowed");
    let mut extra = parameter;
    extra["name"] = "seventeenth".into();
    input["parameters"]
        .as_array_mut()
        .expect("parameters")
        .push(extra);
    assert!(
        ast::generate(&serde_json::to_string(&input).expect("input"))
            .expect_err("parameter cap")
            .to_string()
            .contains("16 parameters")
    );

    let mut empty: Value = serde_json::from_str(&program("ast-program")).expect("builder");
    empty["result"]["fields"]
        .as_array_mut()
        .expect("fields")
        .clear();
    let empty = serde_json::to_string(&empty).expect("input");
    assert!(
        ast::generate(&empty)
            .expect("empty object source")
            .contains("serde_json::Map::new()")
    );
    assert_eq!(
        ast::interpret(&empty, &[Value::Null]).expect("empty object"),
        Value::Object(serde_json::Map::new())
    );
}

#[test]
fn compiler_cli_is_repeatable_and_fails_without_partial_source() {
    let input = program("ast-binary");
    let source = ast::generate(&input).expect("source");
    assert_eq!(source, ast::generate(&input).expect("repeat"));
    let path = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("ast-binary.json");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-ast-codegen"))
        .arg(path)
        .output()
        .expect("compiler");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, source.as_bytes());
    for args in [
        vec![],
        vec!["one", "two"],
        vec!["not-a-file"],
        vec!["tests/fixtures/invalid.json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-ast-codegen"))
            .args(args)
            .output()
            .expect("failure");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
