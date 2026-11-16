use std::{fs, path::Path};

use bootstrap_executor::{
    compiled_lowering, generate, generate_with_policy,
    lowering::{self, Policy},
    module::Frame,
};
use serde_json::{Value, json};

#[path = "../../generated/lowering-module-escaped.rs"]
mod escaped_module;
#[path = "../../generated/lowering-reverse.rs"]
mod reverse;
#[path = "../../generated/evaluation-reverse.rs"]
mod reverse_evaluator;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn policy(module: Value) -> Policy {
    Policy::parse(
        &json!({"formatVersion":1,"cases":{"int":{"op":"path","value":"X"}},"module":module})
            .to_string(),
    )
    .expect("policy")
}

fn rules() -> Vec<(String, Value)> {
    vec![
        ("a\"\\\n雪".into(), json!({"op":"int"})),
        ("z".into(), json!({"op":"int"})),
    ]
}

fn run(body: Value, rules: &[(String, Value)]) -> Result<String, String> {
    Frame::run(rules, &policy(body)).map_err(|e| e.to_string())
}

#[test]
fn cursor_traversal_and_rust_quoting_are_policy_owned() {
    let forward = json!([
        {"op":"text","value":"["}, {"op":"firstRule"},
        {"op":"whileRules","body":[{"op":"kind"},{"op":"text","value":":"},
            {"op":"expression"},{"op":"text","value":";"},{"op":"nextRule"}]},
        {"op":"text","value":"]"}
    ]);
    let backward = json!([
        {"op":"text","value":"["}, {"op":"lastRule"},
        {"op":"whileRules","body":[{"op":"kind"},{"op":"text","value":":"},
            {"op":"expression"},{"op":"text","value":";"},{"op":"previousRule"}]},
        {"op":"text","value":"]"}
    ]);
    assert_eq!(
        run(forward.clone(), &rules()).expect("forward"),
        "[\"a\\\"\\\\\\n雪\":X;\"z\":X;]"
    );
    assert_eq!(
        run(backward.clone(), &rules()).expect("backward"),
        "[\"z\":X;\"a\\\"\\\\\\n雪\":X;]"
    );
    assert_eq!(run(forward, &[]).expect("empty forward"), "[]");
    assert_eq!(run(backward, &[]).expect("empty backward"), "[]");
}

#[test]
fn missing_and_exhausted_cursors_fail_before_returning_output() {
    for op in ["kind", "expression", "nextRule", "previousRule"] {
        let body = json!([{"op":"text","value":"partial"},{"op":op}]);
        assert_eq!(
            run(body, &rules()).expect_err(op),
            "lowering: no current rule"
        );
    }
    for (start, advance) in [("firstRule", "previousRule"), ("lastRule", "nextRule")] {
        let body = json!([{"op":start},{"op":advance},{"op":"kind"}]);
        assert_eq!(
            run(body, &rules()).expect_err("exhausted"),
            "lowering: no current rule"
        );
        let body = json!([{"op":start},{"op":advance}]);
        assert_eq!(
            run(body, &[]).expect_err("empty"),
            "lowering: no current rule"
        );
    }
    let absent = Policy::parse(r#"{"formatVersion":1,"cases":{"int":{"op":"path","value":"X"}}}"#)
        .expect("expression-only policy");
    assert_eq!(absent.lower(&json!({"op":"int"})).expect("lower"), "X");
    assert_eq!(
        Frame::run(&rules(), &absent)
            .expect_err("missing module")
            .to_string(),
        "lowering: missing module program"
    );
    assert_eq!(run(json!([]), &rules()).expect("empty program"), "");
}

#[test]
fn compiled_module_text_and_cursor_data_are_distinct() {
    let entries = vec![("kind\"\\\n雪".to_owned(), json!({"op":"x"}))];
    let expected = "// quote\" slash\\\n// 雪\r\n\"kind\\\"\\\\\\n雪\"X";
    let reference =
        Policy::parse(&export("tests/valid/lowering-module-escaped")).expect("escaped module");
    assert_eq!(
        Frame::run(&entries, &escaped_module::Compiled).expect("compiled text"),
        expected
    );
    assert_eq!(
        Frame::run(&entries, &reference).expect("reference text"),
        expected
    );
    assert_eq!(
        escaped_module::lower(&json!({"op":"x"})).expect("lower"),
        "X"
    );
}

#[test]
fn cli_module_failures_never_print_partial_source() {
    use std::{
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Temp(std::path::PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!(
        "cuestacean-module-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    drop(
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("create owned policy"),
    );
    let temp = Temp(path);
    let rulefile = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join("evaluation.json");
    let huge = "x".repeat(65536);
    let mut bytes = vec![json!({"op":"text","value":huge}); 16];
    bytes.push(json!({"op":"text","value":"x"}));
    for (body, reason) in [
        (
            json!([{"op":"text","value":"partial"},{"op":"kind"}]),
            "no current rule",
        ),
        (
            json!([{"op":"firstRule"},{"op":"whileRules","body":[]}]),
            "4096 steps",
        ),
        (json!(bytes), "1 MiB"),
    ] {
        fs::write(
            &temp.0,
            json!({"formatVersion":1,"cases":{"int":{"op":"path","value":"X"}},"module":body})
                .to_string(),
        )
        .expect("write policy");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-codegen"))
            .arg(&rulefile)
            .arg(&temp.0)
            .output()
            .expect("compiler failure");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(reason));
    }
    fs::write(
        &temp.0,
        json!({"formatVersion":1,"rules":{("x".repeat(65537)):{"result":{"op":"int"}}}})
            .to_string(),
    )
    .expect("oversized kind rules");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-codegen"))
        .arg(&temp.0)
        .output()
        .expect("compiled default failure");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"lowering: rule kind exceeds 64 KiB\n");
}

#[test]
fn validates_all_statements_before_execution_or_compilation() {
    for source in [
        r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"X"}},"module":[],"module":[]}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"X"}},"module":[{"op":"text","value":"a","value":"b"}]}"#,
        r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"X"}},"module":[{"op":"kind","op":"kind"}]}"#,
    ] {
        assert!(Policy::parse(source).is_err());
        assert!(lowering::generate(source).is_err());
    }
    for body in [
        json!(null),
        json!([{"op":"unknown"}]),
        json!([{"op":"firstRule","extra":true}]),
        json!([{"op":"whileRules","body":[{"op":"text","value":"x".repeat(65537)}]}]),
        json!(vec![json!({"op":"firstRule"}); 65]),
    ] {
        let text =
            json!({"formatVersion":1,"cases":{"int":{"op":"path","value":"X"}},"module":body})
                .to_string();
        assert!(Policy::parse(&text).is_err(), "accepted {text}");
        assert!(
            lowering::generate(&text).is_err(),
            "compiled invalid module"
        );
    }
    let mut body = json!([]);
    for _ in 0..17 {
        body = json!([{"op":"whileRules","body":body}]);
    }
    let text = json!({"formatVersion":1,"cases":{"int":{"op":"path","value":"X"}},"module":body})
        .to_string();
    assert!(Policy::parse(&text).is_err());
    assert!(lowering::generate(&text).is_err());
}

#[test]
fn loop_fuel_and_module_bytes_are_shared() {
    let stalled = json!([{"op":"firstRule"},{"op":"whileRules","body":[]}]);
    assert!(
        run(stalled, &rules())
            .expect_err("stalled loop")
            .contains("4096")
    );
    let text = "x".repeat(65536);
    let mut body = vec![json!({"op":"text","value":text}); 16];
    assert_eq!(run(json!(body), &[]).expect("exact cap").len(), 1024 * 1024);
    body.push(json!({"op":"text","value":"x"}));
    assert!(
        run(json!(body), &[])
            .expect_err("module output cap")
            .contains("1 MiB")
    );
    // Each tiny expression is independently under its budget; their combined
    // module output must still be charged to the module budget.
    let body = json!([{"op":"firstRule"},{"op":"whileRules","body":[
        {"op":"expression"},{"op":"nextRule"}]}]);
    let large = vec![("x".to_owned(), json!({"op":"int"})); 2050];
    // The loop fuel is reached before the byte cap, rather than being reset per rule.
    assert!(
        run(body.clone(), &large)
            .expect_err("shared fuel")
            .contains("4096")
    );
    let text_policy = Policy::parse(
        &json!({"formatVersion":1,
        "cases":{"s":{"op":"string","field":"value"}},"module":body})
        .to_string(),
    )
    .expect("text policy");
    let large_expressions = vec![("x".to_owned(), json!({"op":"s","value":"x".repeat(65536)})); 16];
    assert!(
        Frame::run(&large_expressions, &text_policy)
            .expect_err("shared output")
            .to_string()
            .contains("1 MiB")
    );
}

#[test]
fn compiled_drivers_match_reference_with_independent_order_expectations() {
    let normal = Policy::parse(&export("lowering")).expect("normal");
    let alternate = Policy::parse(&export("tests/valid/lowering-reverse")).expect("reverse");
    let inputs = rules();
    for (reference, driver) in [
        (
            &normal,
            &compiled_lowering::Compiled as &dyn lowering::Driver,
        ),
        (&alternate, &reverse::Compiled as &dyn lowering::Driver),
    ] {
        for entries in [&inputs[..], &[][..]] {
            assert_eq!(
                Frame::run(entries, reference).map_err(|e| e.to_string()),
                Frame::run(entries, driver).map_err(|e| e.to_string())
            );
        }
        let invalid = vec![("a".to_owned(), json!({"op":"missing"}))];
        assert_eq!(
            Frame::run(&invalid, reference).map_err(|e| e.to_string()),
            Frame::run(&invalid, driver).map_err(|e| e.to_string())
        );
    }
    let forward = Frame::run(&inputs, &normal).expect("forward");
    let backward = Frame::run(&inputs, &alternate).expect("backward");
    let escaped = "\"a\\\"\\\\\\n雪\"";
    assert!(forward.find(escaped).expect("quoted kind") < forward.find("\"z\"").expect("z"));
    assert!(backward.find("\"z\"").expect("z") < backward.find(escaped).expect("quoted kind"));
    // Header/start/final-test/footer cost four; each rule costs six steps.
    let mut many = vec![("x".to_owned(), json!({"op":"int"})); 682];
    for driver in [
        &normal as &dyn lowering::Driver,
        &compiled_lowering::Compiled,
        &alternate,
        &reverse::Compiled,
    ] {
        assert!(Frame::run(&many, driver).is_ok(), "4096 steps accepted");
        many.push(("x".to_owned(), json!({"op":"int"})));
        assert_eq!(
            Frame::run(&many, driver)
                .expect_err("shared compiled fuel")
                .to_string(),
            "lowering: module exceeds 4096 steps"
        );
        many.pop();
    }
    for driver in [
        &normal as &dyn lowering::Driver,
        &compiled_lowering::Compiled,
        &alternate,
        &reverse::Compiled,
    ] {
        let mut entries = vec![("x".repeat(65536), json!({"op":"int"})); 15];
        assert!(
            Frame::run(&entries, driver).is_ok(),
            "bounded kinds accepted"
        );
        entries.push(("x".repeat(65536), json!({"op":"int"})));
        assert_eq!(
            Frame::run(&entries, driver)
                .expect_err("cumulative kind output")
                .to_string(),
            "lowering: module exceeds 1 MiB output bytes"
        );
        assert_eq!(
            Frame::run(&[("x".repeat(65537), json!({"op":"int"}))], driver)
                .expect_err("kind limit")
                .to_string(),
            "lowering: rule kind exceeds 64 KiB"
        );
    }
    let source = export("evaluation");
    assert_eq!(
        generate(&source).expect("compiled"),
        generate_with_policy(&source, &export("lowering")).expect("reference")
    );
    assert_ne!(
        generate(&source).expect("normal"),
        generate_with_policy(&source, &export("tests/valid/lowering-reverse")).expect("reverse")
    );
}

#[test]
fn reverse_generated_evaluator_is_executable() {
    assert_eq!(
        reverse::lower(&json!({"op":"int"})).expect("reverse expression driver"),
        "Ok::<_, Error>(runtime::Value::Int)"
    );
    let program = export("addition");
    assert_eq!(reverse_evaluator::execute(&program).expect("addition"), 42);
    assert_eq!(
        reverse_evaluator::execute_json(&program).expect("addition JSON"),
        json!(42)
    );
    for (kind, expected) in [("true", true), ("false", false)] {
        let program = json!({"formatVersion":1,"entry":{"kind":kind}}).to_string();
        assert_eq!(
            reverse_evaluator::execute_json(&program).expect("boolean"),
            json!(expected)
        );
        assert_eq!(
            reverse_evaluator::execute(&program)
                .expect_err("integer API")
                .to_string(),
            "integer output requires an integer value"
        );
    }
    let missing = json!({"formatVersion":1,"entry":{"kind":"unknownTestKind"}}).to_string();
    assert_eq!(
        reverse_evaluator::execute_json(&missing)
            .expect_err("missing rule")
            .to_string(),
        "no rule for node kind \"unknownTestKind\""
    );
}
