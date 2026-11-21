//! Same-source, Go-backed verification; not a sixth native constructor.
use std::{
    error::Error as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "compiled-lowering")]
pub use bootstrap_executor::compiled_lowering;
pub use bootstrap_executor::{lowering, module};
#[path = "../src/codegen/compiler.rs"]
mod compiler;

use serde_json::{Value, json};

const DEFAULT_POLICY: &str =
    include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/lowering.json"));
const EMPTY_POLICY: &str =
    r#"{"formatVersion":1,"cases":{"null":{"op":"text","value":"unused"}},"module":[]}"#;

fn rules(result: &str) -> String {
    format!(r#"{{"formatVersion":1,"rules":{{"only":{{"result":{result}}}}}}}"#)
}

fn projection(input: &str) -> Vec<(String, Value)> {
    compiler::parse_rules(input)
        .expect("authoritative typed decoder")
        .rules
        .into_iter()
        .map(|(name, rule)| {
            (
                name,
                serde_json::to_value(rule.result).expect("typed projection"),
            )
        })
        .collect()
}

#[test]
fn all_45_wire_variants_projection_order_and_duplicate_behavior() {
    // Expected metadata is independently spelled, not decoded from each input.
    let cases = [
        (r#"{"op":"null"}"#, json!({"op":"null"})),
        (r#"{"op":"top"}"#, json!({"op":"top"})),
        (r#"{"op":"int"}"#, json!({"op":"int"})),
        (r#"{"op":"bool"}"#, json!({"op":"bool"})),
        (r#"{"op":"stringType"}"#, json!({"op":"stringType"})),
        (
            r#"{"op":"boolean","value":true}"#,
            json!({"op":"boolean","value":true}),
        ),
        (
            r#"{"op":"conjunction","field":"q\"\\\n雪"}"#,
            json!({"op":"conjunction","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"text","field":"q\"\\\n雪"}"#,
            json!({"op":"text","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"list","field":"q\"\\\n雪"}"#,
            json!({"op":"list","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"record","field":"q\"\\\n雪"}"#,
            json!({"op":"record","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"recordExpression","field":"q\"\\\n雪"}"#,
            json!({"op":"recordExpression","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"listExpression","field":"q\"\\\n雪"}"#,
            json!({"op":"listExpression","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"field","field":"q\"\\\n雪"}"#,
            json!({"op":"field","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"evaluate","field":"q\"\\\n雪"}"#,
            json!({"op":"evaluate","field":"q\"\\\n雪"}),
        ),
        (
            r#"{"op":"recordLength","value":{"op":"top"}}"#,
            json!({"op":"recordLength","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"lowerBound","value":{"op":"top"}}"#,
            json!({"op":"lowerBound","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"lowerExclusiveBound","value":{"op":"top"}}"#,
            json!({"op":"lowerExclusiveBound","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"upperExclusiveBound","value":{"op":"top"}}"#,
            json!({"op":"upperExclusiveBound","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"upperBound","value":{"op":"top"}}"#,
            json!({"op":"upperBound","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"negate","value":{"op":"top"}}"#,
            json!({"op":"negate","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"length","value":{"op":"top"}}"#,
            json!({"op":"length","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"stringLength","value":{"op":"top"}}"#,
            json!({"op":"stringLength","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"listLength","value":{"op":"top"}}"#,
            json!({"op":"listLength","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"logicalNot","value":{"op":"top"}}"#,
            json!({"op":"logicalNot","value":{"op":"top"}}),
        ),
        (
            r#"{"op":"recordIndex","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"recordIndex","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"index","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"index","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"listIndex","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"listIndex","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"sum","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"sum","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"difference","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"difference","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"product","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"product","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"divide","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"divide","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"modulo","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"modulo","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"quotient","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"quotient","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"remainder","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"remainder","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"logicalAnd","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"logicalAnd","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"logicalOr","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"logicalOr","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"less","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"less","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"lessEqual","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"lessEqual","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"greater","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"greater","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"greaterEqual","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"greaterEqual","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"equal","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"equal","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"notEqual","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"notEqual","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"unify","left":{"op":"null"},"right":{"op":"bool"}}"#,
            json!({"op":"unify","left":{"op":"null"},"right":{"op":"bool"}}),
        ),
        (
            r#"{"op":"slice","operandField":"base","lowerField":"lo","upperField":"hi"}"#,
            json!({"op":"slice","operandField":"base","lowerField":"lo","upperField":"hi"}),
        ),
        (
            r#"{"op":"listSlice","value":{"op":"int"},"lower":{"op":"null"},"upper":{"op":"bool"}}"#,
            json!({"op":"listSlice","value":{"op":"int"},"lower":{"op":"null"},"upper":{"op":"bool"}}),
        ),
    ];
    assert_eq!(cases.len(), 45);
    let mut tags = std::collections::BTreeSet::new();
    for (wire, expected) in cases {
        assert!(tags.insert(expected["op"].as_str().expect("handwritten tag").to_owned()));
        let input = rules(wire);
        assert_eq!(projection(&input), vec![("only".to_owned(), expected)]);
        // Real compiler emit/Frame/Driver path, not only a test serialization.
        let compiled = compiler::generate(&input).expect("default backend accepts every variant");
        assert_eq!(
            compiled,
            compiler::generate_with_policy(&input, DEFAULT_POLICY).expect("reference backend")
        );
    }
    assert_eq!(
        projection(&rules(r#"{"op":"boolean","value":false}"#))[0].1,
        json!({"op":"boolean","value":false})
    );
    let ordered = r#"{"formatVersion":1,"rules":{"z":{"result":{"op":"top"}},"a":{"result":{"op":"null"}},"z":{"result":{"op":"bool"}}}}"#;
    assert_eq!(
        projection(ordered),
        vec![
            ("a".to_owned(), json!({"op":"null"})),
            ("z".to_owned(), json!({"op":"bool"})),
        ]
    );
    let overwritten_bad = r#"{"formatVersion":1,"rules":{"x":{"result":{"op":"unknownTestKind"}},"x":{"result":{"op":"null"}}}}"#;
    assert!(matches!(
        compiler::generate_with_policy(overwritten_bad, EMPTY_POLICY),
        Err(compiler::Error::Json { .. })
    ));
}

fn assert_json_phase(input: &str) {
    for policy in ["not JSON", EMPTY_POLICY] {
        let core = compiler::generate_with_policy(input, policy).expect_err("typed rejection");
        let root = bootstrap_executor::generate_with_policy(input, policy)
            .expect_err("legacy typed rejection");
        match (&core, &root) {
            (
                compiler::Error::Json { document, source },
                bootstrap_executor::Error::Json {
                    document: old_document,
                    source: old_source,
                },
            ) => {
                assert_eq!((*document, *old_document), ("rules", "rules"));
                assert_eq!(source.to_string(), old_source.to_string());
                assert_eq!(source.classify(), old_source.classify());
                assert_eq!(
                    (source.line(), source.column()),
                    (old_source.line(), old_source.column())
                );
                assert!(
                    core.source()
                        .expect("JSON source")
                        .is::<serde_json::Error>()
                );
                assert!(
                    root.source()
                        .expect("root JSON source")
                        .is::<serde_json::Error>()
                );
                assert_eq!(core.to_string(), format!("invalid rules JSON: {source}"));
            }
            _ => panic!("expected Json, got {core:?} / {root:?}"),
        }
        assert_eq!(core.to_string(), root.to_string());
        assert!(compiler::generate(input).is_err());
        assert!(bootstrap_executor::generate(input).is_err());
    }
}

fn assert_lowering_phase(input: &str, policy: &str, expected: &str) {
    let core = compiler::generate_with_policy(input, policy).expect_err("lowering rejection");
    let root = bootstrap_executor::generate_with_policy(input, policy)
        .expect_err("root lowering rejection");
    match (&core, &root) {
        (
            compiler::Error::Lowering { source },
            bootstrap_executor::Error::Lowering { source: old_source },
        ) => {
            assert_eq!(source.to_string(), old_source.to_string());
            assert_eq!(source.to_string(), expected);
            assert!(
                core.source()
                    .expect("lowering source")
                    .is::<lowering::Error>()
            );
            assert!(
                root.source()
                    .expect("root lowering source")
                    .is::<lowering::Error>()
            );
            match (source, old_source) {
                (lowering::Error::Json { source }, lowering::Error::Json { source: old }) => {
                    assert_eq!(source.to_string(), old.to_string());
                    assert!(
                        core.source()
                            .expect("lowering")
                            .source()
                            .expect("nested JSON")
                            .is::<serde_json::Error>()
                    );
                    assert!(
                        root.source()
                            .expect("root lowering")
                            .source()
                            .expect("nested JSON")
                            .is::<serde_json::Error>()
                    );
                }
                (lowering::Error::Invalid { reason }, lowering::Error::Invalid { reason: old }) => {
                    assert_eq!(reason, old)
                }
                _ => panic!("different concrete lowering variants"),
            }
        }
        _ => panic!("expected Lowering, got {core:?} / {root:?}"),
    }
    assert_eq!(core.to_string(), root.to_string());
}

#[test]
fn typed_validation_phases_and_exact_legacy_error_sources() {
    for input in [
        "not JSON",
        "null",
        "[]",
        "{}",
        r#"{"formatVersion":1,"rules":{}} trailing"#,
        r#"{"formatVersion":1,"formatVersion":1,"rules":{}}"#,
        r#"{"formatVersion":1,"rules":{},"rules":{}}"#,
        r#"{"formatVersion":1,"rules":{},"extra":0}"#,
        r#"{"formatVersion":1,"rules":[]}"#,
        r#"{"formatVersion":1,"rules":{"unused":{}}}"#,
        r#"{"formatVersion":1,"rules":{"unused":{"result":{"op":"null"},"result":{"op":"top"}}}}"#,
        r#"{"formatVersion":1,"rules":{"unused":{"result":{"op":"null"},"extra":0}}}"#,
        r#"{"formatVersion":2,"rules":{"unused":{"result":{"op":"boolean","value":0}}}}"#,
        r#"{"formatVersion":1,"rules":{"x":{"result":{"op":"unknownTestKind"}},"x":{"result":{"op":"null"}}}}"#,
    ] {
        assert_json_phase(input);
    }
    for version in ["-1", "4294967296", "1.5", "true", "\"1\"", "null"] {
        assert_json_phase(&format!(r#"{{"formatVersion":{version},"rules":{{}}}}"#));
    }
    for result in [
        r#"{"op":"unknownTestKind"}"#,
        r#"{"op":"Null"}"#,
        r#"{"op":"null","op":"top"}"#,
        r#"{"op":"null","extra":0}"#,
        r#"{"op":"boolean"}"#,
        r#"{"op":"boolean","value":"true"}"#,
        r#"{"op":"boolean","value":1}"#,
        r#"{"op":"boolean","value":true,"value":false}"#,
        r#"{"op":"field","field":7}"#,
        r#"{"op":"field","field":null}"#,
        r#"{"op":"field","field":"a","field":"b"}"#,
        r#"{"op":"int","value":9223372036854775807}"#,
        r#"{"op":"sum","left":{"op":"null"},"right":{"op":"unknownTestKind"}}"#,
        r#"{"op":"negate","value":{"op":"boolean","value":0}}"#,
        r#"{"op":"listSlice","value":{"op":"null"},"lower":{"op":"top"}}"#,
        r#"{"op":"slice","operand_field":"x","lowerField":"lo","upperField":"hi"}"#,
    ] {
        assert_json_phase(&rules(result));
    }
    for version in [0, 2, u32::MAX] {
        let input = format!(r#"{{"formatVersion":{version},"rules":{{}}}}"#);
        let core =
            compiler::generate_with_policy(&input, "not JSON").expect_err("version before policy");
        let root =
            bootstrap_executor::generate_with_policy(&input, "not JSON").expect_err("root version");
        assert!(
            matches!(core, compiler::Error::Version { document: "rules", version: v } if v == version)
        );
        assert!(
            matches!(root, bootstrap_executor::Error::Version { document: "rules", version: v } if v == version)
        );
        assert_eq!(
            core.to_string(),
            format!("unsupported rules formatVersion: {version}")
        );
        assert_eq!(core.to_string(), root.to_string());
        assert!(core.source().is_none());
        assert!(root.source().is_none());
    }
    let input = rules(r#"{"op":"null"}"#);
    assert_lowering_phase(
        &input,
        "not JSON",
        "invalid lowering policy JSON: expected ident at line 1 column 2",
    );
    assert_lowering_phase(
        &input,
        r#"{"formatVersion":1,"cases":{"null":{"op":"text","value":"x"}}}"#,
        "lowering: missing module program",
    );
    assert_lowering_phase(
        &input,
        r#"{"formatVersion":1,"cases":{"null":{"op":"text","value":"x"}},"module":[{"op":"kind"}]}"#,
        "lowering: no current rule",
    );
    assert_eq!(
        compiler::generate_with_policy(&input, EMPTY_POLICY).expect("fresh recovery"),
        ""
    );
    assert_eq!(
        bootstrap_executor::generate_with_policy(&input, EMPTY_POLICY).expect("root recovery"),
        ""
    );
}

// Independently transcribed from the module source contract, not generated output.
const HEADER: &str = "\
// Generated by bootstrap-codegen. Do not edit by hand.\n\
use bootstrap_executor::{Error, runtime};\n\
use serde_json::Value;\n\n\
pub fn execute(input: &str) -> Result<i64, Error> {\n\
let mut _context = runtime::EvaluationContext::default();\n\
runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)\n\
}\n\n\
pub fn execute_json(input: &str) -> Result<Value, Error> {\n\
let mut _context = runtime::EvaluationContext::default();\n\
runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)\n\
}\n\n\
fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {\n\
_context.visit(|_context| evaluate_node(node, _context))\n\
}\n\n\
fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {\n\
match runtime::kind(node)? {\n";
const FOOTER: &str = "kind => Err(Error::MissingRule { kind: kind.to_string() }),\n}\n}\n";
const TINY_INPUT: &str = r#"{"formatVersion":1,"rules":{"z雪":{"result":{"op":"boolean","value":false}},"a\"\\\n":{"result":{"op":"text","field":"q\"\\\n雪"}}}}"#;
const DEFAULT_ARMS: &str = "\"a\\\"\\\\\\n\" => runtime::text(node, \"q\\\"\\\\\\n雪\"),\n\"z雪\" => Ok::<_, Error>(runtime::Value::Boolean(false)),\n";
const CUSTOM_POLICY: &str = r#"{"formatVersion":1,"cases":{"text":{"op":"string","field":"field"},"boolean":{"op":"boolean","field":"value"}},"module":[{"op":"text","value":"const DATA: &[(&str, &str)] = &[\n"},{"op":"lastRule"},{"op":"whileRules","body":[{"op":"text","value":"("},{"op":"kind"},{"op":"text","value":", stringify!("},{"op":"expression"},{"op":"text","value":")),\n"},{"op":"previousRule"}]},{"op":"text","value":"];\n"}]}"#;
const CUSTOM_SOURCE: &str = "const DATA: &[(&str, &str)] = &[\n(\"z雪\", stringify!(false)),\n(\"a\\\"\\\\\\n\", stringify!(\"q\\\"\\\\\\n雪\")),\n];\n";

#[test]
fn complete_handwritten_default_reference_and_custom_sources_recover() {
    let expected = [HEADER, DEFAULT_ARMS, FOOTER].concat();
    assert_eq!(compiler::generate(TINY_INPUT).expect("default"), expected);
    assert_eq!(
        compiler::generate_with_policy(TINY_INPUT, DEFAULT_POLICY).expect("reference"),
        expected
    );
    assert_eq!(
        bootstrap_executor::generate(TINY_INPUT).expect("legacy default"),
        expected
    );
    assert_eq!(
        bootstrap_executor::generate_with_policy(TINY_INPUT, DEFAULT_POLICY)
            .expect("legacy reference"),
        expected
    );
    for _ in 0..2 {
        assert!(compiler::generate_with_policy(TINY_INPUT, "not JSON").is_err());
        assert_eq!(
            compiler::generate_with_policy(TINY_INPUT, CUSTOM_POLICY)
                .expect("reverse custom traversal"),
            CUSTOM_SOURCE
        );
        assert!(
            compiler::generate_with_policy(&rules(r#"{"op":"text","field":false}"#), CUSTOM_POLICY)
                .is_err()
        );
        assert_eq!(
            bootstrap_executor::generate_with_policy(TINY_INPUT, CUSTOM_POLICY)
                .expect("legacy custom recovery"),
            CUSTOM_SOURCE
        );
        assert_eq!(
            compiler::generate(TINY_INPUT).expect("fresh default recovery"),
            expected
        );
    }
}

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
struct Cli {
    dir: PathBuf,
    binary: PathBuf,
}
impl Cli {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let binary = PathBuf::from(env!("CARGO_BIN_EXE_bootstrap-codegen"));
        assert!(binary.is_absolute(), "Cargo binary must be absolute");
        let metadata =
            fs::metadata(&binary).expect("mandatory real binary exists before env-clear");
        assert!(metadata.is_file(), "mandatory binary must be regular");
        assert_ne!(
            metadata.permissions().mode() & 0o111,
            0,
            "mandatory binary must be executable"
        );
        fs::File::open(&binary).expect("mandatory binary must be readable");
        let dir = std::env::temp_dir().join(format!(
            "cuestacean-instruction-boundary-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(dir.is_absolute());
        fs::create_dir(&dir).expect("exclusive test directory");
        fs::create_dir(dir.join("home")).expect("owned HOME");
        Self { dir, binary }
    }
    fn file(&self, name: &str, body: &str) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, body).expect("write independent fixture");
        path
    }
    fn run(&self, args: &[&Path]) -> Output {
        Command::new(&self.binary)
            .args(args)
            .env_clear()
            .env("HOME", self.dir.join("home"))
            .env("PATH", self.dir.join("unusable-path"))
            .current_dir(&self.dir)
            .output()
            .expect("launch mandatory real CLI; setup failure is not a semantic rejection")
    }
    fn failure(&self, args: &[&Path], message: &str) {
        let output = self.run(args);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let stderr = std::str::from_utf8(&output.stderr).expect("UTF-8 diagnostic");
        assert!(
            stderr.contains(message),
            "expected {message:?}, got {stderr:?}"
        );
    }
    fn success(&self, args: &[&Path], expected: &str) {
        let output = self.run(args);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(output.stdout, expected.as_bytes());
    }
}
impl Drop for Cli {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("remove owned test directory");
    }
}

#[test]
fn actual_cargo_cli_cleared_environment_all_error_classes_and_recovery() {
    let cli = Cli::new();
    let input = cli.file("rules.json", TINY_INPUT);
    let custom = cli.file("custom.json", CUSTOM_POLICY);
    let default = cli.file("default.json", DEFAULT_POLICY);
    let malformed = cli.file("malformed.json", "not JSON");
    let typed = cli.file("typed.json", &rules(r#"{"op":"boolean","value":0}"#));
    let version = cli.file("version.json", r#"{"formatVersion":2,"rules":{}}"#);
    let invalid_policy = cli.file(
        "invalid-policy.json",
        r#"{"formatVersion":1,"cases":{"text":{"op":"path","value":"bad-path!"}},"module":[]}"#,
    );
    let execution = cli.file("execution.json", r#"{"formatVersion":1,"cases":{"null":{"op":"text","value":"x"}},"module":[{"op":"text","value":"NEVER PRINT THIS PARTIAL OUTPUT"},{"op":"kind"}]}"#);
    let missing_module = cli.file(
        "missing-module.json",
        r#"{"formatVersion":1,"cases":{"null":{"op":"text","value":"x"}}}"#,
    );
    let missing = cli.dir.join("missing.json");
    let unreadable = cli.dir.join("directory-not-readable-as-text");
    fs::create_dir(&unreadable).expect("read failure fixture");
    cli.success(&[&input], &[HEADER, DEFAULT_ARMS, FOOTER].concat());
    cli.success(
        &[&input, &default],
        &[HEADER, DEFAULT_ARMS, FOOTER].concat(),
    );
    cli.success(&[&input, &custom], CUSTOM_SOURCE);
    let failures: Vec<(Vec<&Path>, &str)> = vec![
        (
            vec![],
            "usage: bootstrap-codegen <rules.json> [lowering.json]",
        ),
        (vec![&input, &custom, &missing], "usage: bootstrap-codegen"),
        (vec![&missing], "cannot read"),
        (vec![&unreadable], "cannot read"),
        (vec![&input, &missing], "cannot read"),
        (vec![&input, &unreadable], "cannot read"),
        (vec![&malformed], "invalid rules JSON:"),
        (vec![&typed], "invalid rules JSON:"),
        (vec![&version], "unsupported rules formatVersion: 2"),
        (vec![&malformed, &custom], "invalid rules JSON:"),
        (vec![&typed, &malformed], "invalid rules JSON:"),
        (
            vec![&version, &malformed],
            "unsupported rules formatVersion: 2",
        ),
        (vec![&input, &malformed], "invalid lowering policy JSON:"),
        (vec![&input, &invalid_policy], "lowering:"),
        (
            vec![&input, &missing_module],
            "lowering: missing module program",
        ),
        (vec![&input, &execution], "lowering: no current rule"),
    ];
    for (args, message) in failures {
        cli.failure(&args, message);
        cli.success(&[&input, &custom], CUSTOM_SOURCE);
    }
    // CLI reads the policy file first, whereas API rejects typed input first.
    assert_json_phase("not JSON");
    cli.failure(
        &[&malformed, &missing],
        &format!("cannot read {}:", missing.display()),
    );
    cli.success(&[&input], &[HEADER, DEFAULT_ARMS, FOOTER].concat());
}
