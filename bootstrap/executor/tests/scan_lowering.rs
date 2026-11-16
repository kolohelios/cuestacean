use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use bootstrap_executor::{
    lowering::Policy,
    scanner::{self, Error, Scan},
};
use serde_json::{Value, json};

#[path = "../../generated/lowering-bindings.rs"]
mod bindings;
#[path = "../../generated/scan-unicode-advance.rs"]
mod unicode_advance;

fn export(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")))
        .expect("read export")
}

fn policy(cases: Value) -> String {
    json!({"formatVersion":1,"cases":cases}).to_string()
}

fn program(body: Value) -> String {
    json!({"formatVersion":1,"body":body}).to_string()
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
    let output = child.wait_with_output().expect("format");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("UTF-8 Rust")
}

#[test]
fn existing_scanners_match_reference_and_committed_source() {
    let backend = export("scan-lowering");
    for name in "whitespace after-operand after-word-operator decimal identifier group-open group-close bound-prefix infix negation-prefix tests/valid/scanner-advance tests/valid/scanner-char-break tests/valid/scanner-horizontal tests/valid/scanner-lookahead tests/valid/scanner-decimal-permissive tests/valid/scanner-reject tests/valid/scanner-stalled tests/valid/scanner-state".split_whitespace() {
        let input = export(name);
        let generated = scanner::generate(&input).expect(name);
        assert_eq!(
            generated,
            scanner::generate_with_policy(&input, &backend).expect(name),
            "{name}"
        );
        let basename = name.rsplit('/').next().expect("basename");
        let golden = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../generated")
                .join(format!("{basename}.rs")),
        )
        .expect("golden");
        assert_eq!(formatted(&generated), golden, "{name}");
    }
}
#[test]
fn alternate_backend_changes_compiled_scanner_behavior() {
    let input = export("tests/valid/scanner-advance");
    let alternate = export("tests/valid/scan-lowering-unicode");
    assert_ne!(
        scanner::generate(&input).expect("default"),
        scanner::generate_with_policy(&input, &alternate).expect("alternate")
    );
    assert_eq!(
        unicode_advance::scan("λ", 0).expect("scalar"),
        Scan {
            offset: 2,
            flag: false
        }
    );
    assert!(matches!(
        scanner::interpret(&input, "λ", 0),
        Err(Error::Advance)
    ));
    assert!(matches!(unicode_advance::scan("λ", 1), Err(Error::Offset)));
}
#[test]
fn source_patterns_and_flow_are_independently_pinned() {
    let chars = scanner::generate(&export("tests/valid/scanner-char-break")).expect("chars");
    assert!(chars.contains("Some('\\n' | '\\'' | '\\\\' | 'λ' | '😀')"));
    assert!(chars.contains("'scan1: {"));
    assert!(chars.contains("break 'scan1;"));
    let input = program(json!([
        {"op":"if","condition":{"op":"byteIn","bytes":[99,97,98,98,120]},
         "then":[{"op":"setFlag","value":true}],"else":[{"op":"setFlag","value":true}]},
        {"op":"while","condition":{"op":"flagIs","value":false},"body":[
            {"op":"break"},{"op":"reject","reason":"dead suffix"}
        ]}
    ]));
    let source = scanner::generate(&input).expect("flow");
    assert!(!source.contains("dead suffix"));
    assert_eq!(source.matches("cursor.set_flag(true)").count(), 1);
    assert_eq!(
        scanner::interpret(&input, "abc", 0).expect("flow result"),
        Scan {
            offset: 0,
            flag: true
        }
    );
    let bytes = program(
        json!([{"op":"if","condition":{"op":"byteIn","bytes":[99,97,98,98,120]},
        "then":[{"op":"advance"}]}]),
    );
    assert!(
        scanner::generate(&bytes)
            .expect("bytes")
            .contains("97..=99 | 120")
    );
}
#[test]
fn scanner_limits_fit_shared_compilation_budgets() {
    let body = json!(
        (0..64)
            .map(|_| json!({"op":"setFlag","value":true}))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        scanner::generate(&program(body))
            .expect("64 statements")
            .matches("cursor.step()?")
            .count(),
        64
    );
    let mut body = json!((0..48).map(|_| json!({"op":"advance"})).collect::<Vec<_>>());
    for _ in 0..16 {
        body = json!([{"op":"if","condition":{"op":"hasByte"},"then":[],"else":body}]);
    }
    let input = program(body.clone());
    assert_eq!(
        scanner::generate(&input).expect("16 levels, 64 statements"),
        scanner::generate_with_policy(&input, &export("scan-lowering"))
            .expect("reference boundary")
    );
    body = json!([{"op":"if","condition":{"op":"hasByte"},"then":[],"else":body}]);
    assert!(scanner::generate(&program(body)).is_err(), "17 levels");
}

#[test]
fn generation_string_cap_is_shared_but_not_an_interpreter_limit() {
    for size in [65536, 65537] {
        let input = program(json!([{"op":"reject","reason":"r".repeat(size)}]));
        let compiled = scanner::generate(&input).map_err(|e| e.to_string());
        assert_eq!(
            compiled,
            scanner::generate_with_policy(&input, &export("scan-lowering"))
                .map_err(|e| e.to_string())
        );
        assert_eq!(compiled.is_ok(), size == 65536);
        if size > 65536 {
            assert!(
                compiled
                    .expect_err("string cap")
                    .contains("string field exceeds 64 KiB")
            );
        }
        assert!(
            matches!(
                scanner::interpret(&input, "", 0),
                Err(Error::Rejected { .. })
            ),
            "valid construction still executes"
        );
    }
}

#[test]
fn validation_runs_before_ignoring_backend() {
    let ignore = policy(json!({"program":{"op":"text","value":"ignored"}}));
    for input in [
        program(json!([{"op":"break"}])),
        program(json!([{"op":"while","condition":{"op":"hasByte"},"body":[
            {"op":"break"},{"op":"if","condition":{"op":"byteIn","bytes":[]},"then":[]}
        ]}])),
        program(
            json!([{"op":"if","condition":{"op":"flagIs","value":true},"then":[],"else":[{"op":"break"}]}]),
        ),
        program(json!([{"op":"advance","extra":true}])),
    ] {
        assert!(
            scanner::generate_with_policy(&input, &ignore).is_err(),
            "{input}"
        );
    }
}
#[test]
fn compiled_binding_closures_have_fixed_success_oracles() {
    let reference =
        Policy::parse(&export("tests/valid/lowering-bindings")).expect("binding policy");
    let field = "text\"\\\nλ";
    for (node, expected) in [
        (json!({"op":"root",(field):"same"}), "\"same\"!"),
        (
            json!({"op":"root",(field):"other","flag":true}),
            "\"other\"",
        ),
        (json!({"op":"root",(field):"other","flag":false}), "no"),
        (
            json!({"op":"root",(field):"\"\\\nλ","flag":true}),
            "\"\\\"\\\\\\nλ\"",
        ),
    ] {
        assert_eq!(bindings::lower(&node).expect("compiled binding"), expected);
        assert_eq!(
            reference.lower(&node).expect("interpreted binding"),
            expected
        );
    }
    let missing = json!({"op":"root","flag":true});
    assert!(bindings::lower(&missing).is_err());
    assert!(reference.lower(&missing).is_err());
}
#[test]
fn cli_optional_policy_and_failures_leave_no_partial_output() {
    let dir = Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON"));
    let input = dir.join("tests/valid/scanner-advance.json");
    let backend = dir.join("tests/valid/scan-lowering-unicode.json");
    let binary = env!("CARGO_BIN_EXE_bootstrap-scan-codegen");
    let output = Command::new(binary)
        .arg(&input)
        .arg(&backend)
        .output()
        .expect("CLI");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        formatted(&String::from_utf8(output.stdout).expect("source")),
        include_str!("../../generated/scan-unicode-advance.rs")
    );
    for args in [
        vec![input.as_path(), Path::new("missing-policy")],
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
