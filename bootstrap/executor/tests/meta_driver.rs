use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use bootstrap_executor::{
    lowering::{self, Policy},
    module::Frame,
};
use serde_json::{Value, json};

#[path = "../../generated/meta-driver-keys.rs"]
mod keys;
#[path = "../../generated/meta-driver-previous.rs"]
mod previous;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn golden(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../generated/{name}.rs")),
    )
    .expect("read generated driver")
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
        .expect("write source");
    let output = child.wait_with_output().expect("format source");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("UTF-8")
}

fn many_cases(n: usize) -> String {
    let cases: serde_json::Map<String, Value> = (0..n)
        .map(|i| {
            (
                format!("case{i}"),
                json!({"op":"text", "value":"\n".repeat(20_000)}),
            )
        })
        .collect();
    json!({"formatVersion":1,"cases":cases}).to_string()
}

#[test]
fn alternate_compiler_changes_actual_module_traversal() {
    let target = export("tests/valid/meta-driver-keys");
    let compiler = export("tests/valid/meta-driver-previous");
    let source = lowering::generate_with_policy(&target, &compiler).expect("alternate compiler");
    assert_eq!(formatted(&source), golden("meta-driver-previous"));

    let normal = Policy::parse(&target).expect("target policy");
    let entries = vec![("a".to_owned(), Value::Null), ("b".to_owned(), Value::Null)];
    assert_eq!(
        Frame::run(&entries, &normal).expect("normal traversal"),
        "\"a\"\"b\""
    );
    assert_eq!(
        Frame::run(&entries, &keys::Compiled).expect("compiled default traversal"),
        "\"a\"\"b\""
    );
    assert_eq!(
        keys::lower(&json!({"op":"root"})).expect("compiled default expression"),
        "unused"
    );
    assert_eq!(
        Frame::run(&entries, &previous::Compiled).expect("altered traversal"),
        "\"a\""
    );
    let node = json!({"op":"root"});
    assert_eq!(normal.lower(&node).expect("normal expression"), "unused");
    assert_eq!(
        previous::lower(&node).expect("altered expression"),
        "unused"
    );
}

#[test]
fn full_meta_driver_regenerates_its_golden() {
    let meta = export("meta-lowering");
    let compiled = lowering::generate(&meta).expect("compiled self generation");
    let reference =
        lowering::generate_with_policy(&meta, &meta).expect("reference self generation");
    assert_eq!(compiled, reference);
    assert_eq!(formatted(&compiled), golden("meta-lowering"));
}

#[test]
fn whole_driver_shares_compiler_byte_budget() {
    let compiler = export("meta-lowering");
    let one = many_cases(1);
    let eight = many_cases(8);
    for input in [&one, &eight] {
        let target = Policy::parse(input).expect("valid target");
        assert_eq!(
            target.lower(&json!({"op":"case0"})).expect("target VM"),
            "\n".repeat(20_000)
        );
    }
    let default = lowering::generate(&one).expect("single case compiled");
    assert_eq!(
        default,
        lowering::generate_with_policy(&one, &compiler).expect("single case reference")
    );
    let error = lowering::generate(&eight)
        .expect_err("combined driver byte cap")
        .to_string();
    assert_eq!(error, "lowering: lowering exceeds 1 MiB rendered bytes");
    assert_eq!(
        lowering::generate_with_policy(&eight, &compiler)
            .expect_err("reference combined byte cap")
            .to_string(),
        error
    );

    let dir = std::env::temp_dir().join(format!(
        "cuestacean-full-driver-budget-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    let target = dir.join("target.json");
    let backend = dir.join("compiler.json");
    fs::write(&target, eight).expect("target");
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
fn module_emission_is_inside_the_same_driver_budget() {
    let compiler = export("meta-lowering");
    for n in [1, 8] {
        let input=json!({"formatVersion":1,"cases":{"root":{"op":"text","value":"unused"}},"module":(0..n).map(|_|json!({"op":"text","value":"\n".repeat(20000)})).collect::<Vec<_>>()}).to_string();
        let target = Policy::parse(&input).expect("valid module");
        assert_eq!(
            Frame::run(&[], &target).expect("reference assembly"),
            "\n".repeat(20000 * n)
        );
        let result = lowering::generate(&input).map_err(|e| e.to_string());
        assert_eq!(
            result,
            lowering::generate_with_policy(&input, &compiler).map_err(|e| e.to_string())
        );
        assert_eq!(result.is_ok(), n == 1);
        if n == 8 {
            assert_eq!(
                result.expect_err("global module compiler budget"),
                "lowering: lowering exceeds 1 MiB rendered bytes"
            );
        }
    }
}

#[test]
fn cases_and_module_cannot_reset_each_others_compiler_budget() {
    let compiler = export("meta-lowering");
    let mut cases_only: Value = serde_json::from_str(&many_cases(3)).expect("cases");
    let module = json!(
        (0..3)
            .map(|_| json!({"op":"text","value":"\n".repeat(20000)}))
            .collect::<Vec<_>>()
    );
    let module_only =
        json!({"formatVersion":1,"cases":{"root":{"op":"text","value":"unused"}},"module":module});
    for value in [&cases_only, &module_only] {
        let input = value.to_string();
        assert_eq!(
            lowering::generate(&input).expect("each portion fits"),
            lowering::generate_with_policy(&input, &compiler).expect("reference portion")
        );
    }
    cases_only["module"] = module;
    let input = cases_only.to_string();
    let target = Policy::parse(&input).expect("valid combined policy");
    assert_eq!(
        target
            .lower(&json!({"op":"case0"}))
            .expect("target rendering"),
        "\n".repeat(20000)
    );
    assert_eq!(
        Frame::run(&[], &target).expect("target assembly"),
        "\n".repeat(60000)
    );
    let error = lowering::generate(&input)
        .expect_err("combined compiler cap")
        .to_string();
    assert_eq!(error, "lowering: lowering exceeds 1 MiB rendered bytes");
    assert_eq!(
        lowering::generate_with_policy(&input, &compiler)
            .expect_err("reference combined cap")
            .to_string(),
        error
    );
}

#[test]
fn compiler_must_handle_the_whole_driver_without_native_fallback() {
    let input = export("tests/valid/meta-driver-keys");
    let partial = r#"{"formatVersion":1,"cases":{"text":{"op":"text","value":"partial"}}}"#;
    assert!(
        lowering::generate_with_policy(&input, partial)
            .expect_err("missing driver handler")
            .to_string()
            .contains("missing lowering case")
    );
    let full = r#"{"formatVersion":1,"cases":{"driver":{"op":"text","value":"complete"}}}"#;
    assert_eq!(
        lowering::generate_with_policy(&input, full).expect("whole-driver replacement"),
        "complete"
    );
}

#[test]
fn ignored_module_is_still_validated_before_compiler() {
    let cases = r#""cases":{"root":{"op":"text","value":"unused"}}"#;
    for module in [
        r#"null"#,
        r#"[{"op":"whileRules","body":[{"op":"unknown"}]}]"#,
        r#"[{"op":"text","value":null}]"#,
        r#"[{"op":"text","value":"a","value":"b"}]"#,
        r#"[{"op":"moduleText","value":"internal only"}]"#,
        r#"[{"op":"firstRule","extra":true}]"#,
    ] {
        let input = format!(r#"{{"formatVersion":1,{cases},"module":{module}}}"#);
        assert!(Policy::parse(&input).is_err(), "{module}");
        let target_error = lowering::generate(&input)
            .expect_err("invalid target")
            .to_string();
        let precedence = lowering::generate_with_policy(&input, "{")
            .expect_err("target precedes compiler")
            .to_string();
        assert_eq!(precedence, target_error, "{module}");
    }
}

#[test]
fn absent_and_empty_modules_have_distinct_generated_drivers() {
    let absent = r#"{"formatVersion":1,"cases":{"root":{"op":"text","value":"unused"}}}"#;
    let empty =
        r#"{"formatVersion":1,"cases":{"root":{"op":"text","value":"unused"}},"module":[]}"#;
    let absent_source = lowering::generate(absent).expect("absent module");
    let empty_source = lowering::generate(empty).expect("empty module");
    assert!(!absent_source.contains("fn assemble("));
    assert!(empty_source.contains("fn assemble("));
    assert!(empty_source.contains("_frame"));
    let policy = Policy::parse(empty).expect("empty module policy");
    assert_eq!(Frame::run(&[], &policy).expect("empty assembly"), "");
}
