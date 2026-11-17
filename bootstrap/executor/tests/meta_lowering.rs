use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use bootstrap_executor::{
    compiled_meta_lowering,
    lowering::{self, Policy},
};
use serde_json::{Value, json};

#[path = "../../generated/lowering-meta-forced.rs"]
mod forced;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn policy(cases: Value, aliases: Value) -> String {
    json!({"formatVersion":1,"cases":cases,"aliases":aliases}).to_string()
}

fn formatted(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("pinned rustfmt");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(source.as_bytes())
        .expect("write");
    let output = child.wait_with_output().expect("format");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("UTF-8")
}

#[test]
fn compiled_emitter_matches_reference_and_regenerates_itself() {
    let meta = export("meta-lowering");
    for name in [
        "lowering",
        "pratt-lowering",
        "unify-lowering",
        "ast-lowering",
        "scan-lowering",
        "meta-lowering",
        "tests/valid/lowering-numbers",
        "tests/valid/lowering-bindings",
        "tests/valid/lowering-select",
        "tests/valid/lowering-collections",
        "tests/valid/lowering-identifier",
    ] {
        let input = export(name);
        assert_eq!(
            lowering::generate(&input).expect(name),
            lowering::generate_with_policy(&input, &meta).expect(name),
            "{name}"
        );
    }
    let generated = lowering::generate(&meta).expect("self generation");
    let golden = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../generated/meta-lowering.rs"),
    )
    .expect("committed meta driver");
    assert_eq!(formatted(&generated), golden);
}

#[test]
fn structural_boundaries_and_new_emission_limit_are_explicit() {
    let compiler = export("meta-lowering");
    let nested = |value: String, n: usize| {
        let mut expr = json!({"op":"text","value":value});
        for _ in 0..n {
            expr = json!({"op":"group","value":expr});
        }
        policy(json!({"root":expr}), json!({}))
    };
    let deep = nested("ok".into(), 28); // leaf string is at JSON level 32.
    assert_eq!(
        lowering::generate(&deep).expect("32 levels"),
        lowering::generate_with_policy(&deep, &compiler).expect("reference boundary")
    );
    assert!(Policy::parse(&nested("ok".into(), 29)).is_err());
    let wide = |n: usize| {
        policy(
            json!({"root":{"op":"concat","parts":(0..n).map(|_|json!({"op":"text","value":"ok"})).collect::<Vec<_>>()}}),
            json!({}),
        )
    };
    let input = wide(168); // 511 values; the next complete part exceeds 512.
    assert_eq!(
        lowering::generate(&input).expect("511 values"),
        lowering::generate_with_policy(&input, &compiler).expect("wide reference")
    );
    assert!(Policy::parse(&wide(169)).is_err());
    // A valid multiline Rust string renders within the target VM's budget, while
    // escaped source copied through the meta emitter intentionally exceeds it.
    let input = nested(format!("\"{}\"", "\n".repeat(24000)), 28);
    assert!(
        Policy::parse(&input)
            .expect("valid target")
            .lower(&json!({"op":"root"}))
            .is_ok()
    );
    let error = lowering::generate(&input)
        .expect_err("compiler byte cap")
        .to_string();
    assert_eq!(error, "lowering: lowering exceeds 1 MiB rendered bytes");
    assert_eq!(
        lowering::generate_with_policy(&input, &compiler)
            .expect_err("reference compiler cap")
            .to_string(),
        error
    );
    let dir = std::env::temp_dir().join(format!("cuestacean-meta-budget-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("target.json");
    let backend = dir.join("compiler.json");
    fs::write(&target, input).expect("boundary target");
    fs::write(&backend, compiler).expect("compiler");
    for explicit in [false, true] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bootstrap-lower-codegen"));
        cmd.arg(&target);
        if explicit {
            cmd.arg(&backend);
        }
        let output = cmd.output().expect("budget CLI");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(&error));
    }
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn forced_compiler_changes_actual_compiled_numbers_driver() {
    let target = export("tests/valid/lowering-numbers");
    let alternate = export("tests/valid/meta-lowering-forced");
    let source = lowering::generate_with_policy(&target, &alternate).expect("forced compiler");
    let golden = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../generated/lowering-meta-forced.rs"),
    )
    .expect("compiled forced artifact");
    assert_eq!(formatted(&source), golden);
    let node = json!({"op":"item","count":1});
    assert_eq!(
        Policy::parse(&target)
            .expect("numbers")
            .lower(&node)
            .expect("normal"),
        "more"
    );
    assert_eq!(forced::lower(&node).expect("forced"), "forced");
}

#[test]
fn aliases_preserve_original_opcode_and_generate_grouped_arms() {
    let input = policy(
        json!({"canonical":{"op":"string","field":"op"}}),
        json!({"zAlias":"canonical","aAlias":"canonical"}),
    );
    let p = Policy::parse(&input).expect("aliases");
    for op in ["canonical", "aAlias", "zAlias"] {
        assert_eq!(p.lower(&json!({"op":op})).expect(op), format!("\"{op}\""));
    }
    let source = lowering::generate(&input).expect("generate aliases");
    assert!(
        source.contains("\"canonical\" | \"aAlias\" | \"zAlias\""),
        "{source}"
    );
}

#[test]
fn alias_validation_is_eager_and_checks_original_json() {
    let cases = json!({"root":{"op":"text","value":"ok"}});
    for aliases in [
        json!({"root":"root"}),
        json!({"unused":"missing"}),
        json!({"one":"root","two":"one"}),
        json!({"one":"two","two":"one"}),
        json!({"":"root"}),
        json!({("x".repeat(65)):"root"}),
        Value::Null,
    ] {
        let input = policy(cases.clone(), aliases);
        assert!(Policy::parse(&input).is_err(), "{input}");
        assert!(lowering::generate(&input).is_err(), "{input}");
    }
    let duplicate = r#"{"formatVersion":1,"cases":{"root":{"op":"text","value":"ok"}},"aliases":{"a":"root","a":"root"}}"#;
    assert!(Policy::parse(duplicate).is_err());
    assert!(lowering::generate(duplicate).is_err());
    let aliases: serde_json::Map<String, Value> =
        (0..63).map(|i| (format!("a{i}"), json!("root"))).collect();
    assert!(Policy::parse(&policy(cases.clone(), json!(aliases))).is_ok());
    let mut too_many = aliases;
    too_many.insert("extra".into(), json!("root"));
    assert!(Policy::parse(&policy(cases, json!(too_many))).is_err());
}

#[test]
fn identifier_checks_data_and_snake_conversion() {
    for (snake, input, expected) in [
        (false, "bytePattern", "bytePattern"),
        (true, "bytePattern", "byte_pattern"),
        (true, "ifTrue", "if_true"),
        (false, "type", "type"),
        (false, "_ok9", "_ok9"),
    ] {
        let p = Policy::parse(&policy(
            json!({"root":{"op":"identifier","field":"name","snake":snake}}),
            json!({}),
        ))
        .expect("identifier");
        assert_eq!(
            p.lower(&json!({"op":"root","name":input})).expect(input),
            expected
        );
    }
    let input = policy(
        json!({"root":{"op":"identifier","field":"name"}}),
        json!({}),
    );
    let p = Policy::parse(&input).expect("default snake");
    for bad in [
        json!(""),
        json!("1a"),
        json!("a-b"),
        json!("é"),
        json!("a::b"),
        json!("x".repeat(65)),
        json!(null),
        json!(1),
    ] {
        assert!(p.lower(&json!({"op":"root","name":bad})).is_err(), "{bad}");
    }
    assert!(p.lower(&json!({"op":"root"})).is_err());
    assert_eq!(
        p.lower(&json!({"op":"root","name":"x".repeat(64)}))
            .expect("boundary"),
        "x".repeat(64)
    );
    for field in ["".to_string(), "x".repeat(129)] {
        assert!(
            Policy::parse(&policy(
                json!({"root":{"op":"identifier","field":field}}),
                json!({})
            ))
            .is_err()
        );
    }
}

#[test]
fn compiler_rejects_invalid_target_and_unused_compiler_cases() {
    let meta = export("meta-lowering");
    let unbound = policy(json!({"root":{"op":"get","name":"absent"}}), json!({}));
    assert!(
        lowering::generate_with_policy(&unbound, "{")
            .expect_err("target checked first")
            .to_string()
            .contains("unknown binding")
    );
    let invalid = policy(json!({"root":{"op":"identifier","field":""}}), json!({}));
    assert!(lowering::generate_with_policy(&invalid, &meta).is_err());
    let invalid_compiler = policy(
        json!({"unused":{"op":"unknown"},"text":{"op":"text","value":"ignored"}}),
        json!({}),
    );
    let valid = policy(json!({"root":{"op":"text","value":"ok"}}), json!({}));
    assert!(lowering::generate_with_policy(&valid, &invalid_compiler).is_err());
}

#[test]
fn emitted_templates_contain_independent_kernel_calls() {
    let input = policy(
        json!({
            "root":{"op":"concat","parts":[
                {"op":"call","path":"f","args":[{"op":"string","field":"x"}]},
                {"op":"bind","name":"v","value":{"op":"text","value":"x"},"body":{"op":"get","name":"v"}},
                {"op":"select","field":"tag","cases":{"a":{"op":"identifier","field":"name","snake":true}}}
            ]}
        }),
        json!({}),
    );
    let source = lowering::generate(&input).expect("source");
    for snippet in [
        "ctx.concat(",
        "ctx.call(\"f\"",
        "ctx.string(_node, \"x\")",
        "ctx.bind(",
        "ctx.select(_node, \"tag\"",
        "ctx.identifier(_node, \"name\", true, false)",
    ] {
        assert!(source.contains(snippet), "missing {snippet}: {source}");
    }
    for (expr, expected) in [
        (
            json!({"op":"identifier","field":"name","snake":true,"upperFirst":false}),
            "ctx.identifier(_node, \"name\", true, false)",
        ),
        (
            json!({"op":"identifier","field":"name","snake":false,"upperFirst":true}),
            "ctx.identifier(_node, \"name\", false, true)",
        ),
    ] {
        assert_eq!(
            compiled_meta_lowering::lower(&expr).expect("compiled expression"),
            expected
        );
    }
}

#[test]
fn cli_override_and_failures_never_print_partial_source() {
    let dir = std::env::temp_dir().join(format!("cuestacean-meta-test-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("target.json");
    let compiler = dir.join("compiler.json");
    fs::write(&target, export("tests/valid/lowering-numbers")).expect("target");
    fs::write(&compiler, export("tests/valid/meta-lowering-forced")).expect("compiler");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_bootstrap-lower-codegen"))
            .arg(&target)
            .arg(&compiler)
            .output()
            .expect("CLI")
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8"),
        lowering::generate_with_policy(
            &export("tests/valid/lowering-numbers"),
            &export("tests/valid/meta-lowering-forced")
        )
        .expect("reference")
    );
    fs::write(&compiler, "{}").expect("invalid compiler");
    let output = run();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    fs::write(&target, "{}").expect("invalid target");
    let output = run();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    fs::remove_dir_all(dir).expect("cleanup");
}
