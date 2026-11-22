use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");

fn path(value: &str) -> Value {
    json!({"op":"path","value":value})
}
fn read(op: &str, field: &str) -> Value {
    json!({"op":op,"field":field})
}
fn wrapped(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":read("lower",field)}})
}
fn call(name: &str, args: Vec<Value>) -> Value {
    json!({"op":"call","path":name,"args":args})
}
// Complete independent expansion of all FOUR lexical helpers and SIXTEEN cases.
// These are contract answers, never parsed/projected from source or producer output.
fn expected(node: &str, field: &str, context: &str, value: &str) -> Value {
    let two = || vec![path(node), read("string", field)];
    let four = || {
        vec![
            path(node),
            read("string", field),
            path(context),
            path("evaluate"),
        ]
    };
    let constant = |name| json!({"op":"ok","value":path(name)});
    json!({"cases":{
        "null":constant("runtime::Value::Null"),
        "top":constant("runtime::Value::Top"),
        "int":constant("runtime::Value::Int"),
        "bool":constant("runtime::Value::Bool"),
        "stringType":constant("runtime::Value::String"),
        "text":call("runtime::text",two()),
        "list":call("runtime::list",two()),
        "record":call("runtime::record",two()),
        "recordExpression":call("runtime::record_expression",four()),
        "listExpression":call("runtime::list_expression",four()),
        "boolean":{"op":"ok","value":call("runtime::Value::Boolean",vec![read("boolean","value")])},
        "field":call("runtime::integer",two()),
        "evaluate":call("evaluate",vec![json!({"op":"try","value":call("runtime::field",two())}),path(context)]),
        "conjunction":call("runtime::conjunction",four()),
        "slice":call("runtime::slice_fields",vec![path(node),read("string","operandField"),
            read("string","lowerField"),read("string","upperField"),path(context),path("evaluate")]),
        "listSlice":{"op":"concat","parts":[
            {"op":"text","value":"(|_context: &mut runtime::EvaluationContext| "},
            call("runtime::list_slice",vec![wrapped(value),wrapped("lower"),wrapped("upper")]),
            {"op":"text","value":")(_context)"}]}
    }})
}
fn baseline() -> Value {
    expected("node", "field", "_context", "value")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn failed(out: &Output) {
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "no partial JSON");
    assert!(!out.stderr.is_empty());
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file(), "regular tool");
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    native: PathBuf,
    shell: PathBuf,
    binary: String,
    full_exports: Cell<usize>,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-core-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let discover = |name| {
            let path = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute tool");
            executable(&path);
            path
        };
        let cue = discover("cue");
        let _jq = discover("jq");
        let shell = discover("sh");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        let mut probe = Self {
            dir,
            cue,
            native,
            shell,
            binary: String::new(),
            full_exports: Cell::new(0),
        };
        let raw = probe.run(
            &probe.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary.cue", BINARY)],
            false,
        );
        assert!(raw.status.success(), "independent Binary oracle setup");
        let raw = String::from_utf8(raw.stdout).expect("Go table JSON");
        let placed = probe.run(
            &probe.cue,
            &[
                "import",
                "-p",
                "bootstrap",
                "-l",
                "#InstructionBinaryTables:",
                "--outfile",
                "-",
            ],
            &[("binary.json", &raw)],
            false,
        );
        assert!(placed.status.success(), "whole Binary import setup");
        probe.binary = String::from_utf8(placed.stdout).expect("Go helper");
        probe
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, source) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, source).expect("explicit verification source");
            paths.push(path);
        }
        if reverse {
            paths.reverse();
        }
        Command::new(tool)
            .args(args)
            .args(paths)
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .output()
            .expect("mandatory process setup")
    }
    fn native(&self, source: &str, reverse: bool) -> Output {
        self.run(
            &self.native,
            &[],
            &[("package.cue", PACKAGE), ("ordinary.cue", source)],
            reverse,
        )
    }
    fn isolated(&self, body: Option<(&str, &str)>, op: &str, reverse: bool) -> Output {
        let mut inputs = vec![("shape.cue", SHAPE)];
        if let Some(body) = body {
            inputs.push(body);
        }
        // NO full schema, bridge, selector, context or unrelated providers.
        self.run(&self.cue, &[op], &inputs, reverse)
    }
    fn full(&self, body: Option<(&str, &str)>, context: &str, reverse: bool, gate: bool) -> Output {
        if gate {
            let checked = self.isolated(body, "export", reverse);
            if !checked.status.success() {
                return checked;
            }
        }
        for (helper, provider) in [
            ("#InstructionRightField", RIGHT),
            ("#InstructionBinaryTables", self.binary.as_str()),
        ] {
            let checked = self.run(
                &self.cue,
                &["export", "-e", helper],
                &[
                    ("schema.cue", SCHEMA),
                    ("bridge.cue", BRIDGE),
                    ("provider.cue", provider),
                ],
                reverse,
            );
            if !checked.status.success() {
                return checked;
            }
        }
        let operand_gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionRightOperand"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", RIGHT),
                ("operand.cue", OPERAND),
            ],
            reverse,
        );
        if !operand_gate.status.success() {
            return operand_gate;
        }
        let group_gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionOperandCases"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", RIGHT),
                ("operand.cue", OPERAND),
                ("group.cue", GROUP),
            ],
            reverse,
        );
        if !group_gate.status.success() {
            return group_gate;
        }
        self.full_exports.set(self.full_exports.get() + 1);
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("binary-placed.cue", self.binary.as_str()),
            ("unary.cue", UNARY),
            ("text.cue", TEXT),
            ("first.cue", FIRST),
            ("step.cue", STEP),
            ("right.cue", RIGHT),
            ("operand.cue", OPERAND),
            ("group.cue", GROUP),
            ("context.cue", context),
        ];
        if let Some(body) = body {
            inputs.push(body);
        }
        // Reuse SAME default-free body; shape is deliberately NOT composed here.
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}
// Inventory is derived ONLY from the independently authored expected tree above.
fn inventory(
    value: &Value,
    prefix: &str,
    leaves: &mut Vec<String>,
    arrays: &mut Vec<String>,
    objects: &mut Vec<String>,
) {
    match value {
        Value::Object(fields) => {
            objects.push(prefix.to_owned());
            for (key, value) in fields {
                inventory(value, &format!("{prefix}/{key}"), leaves, arrays, objects);
            }
        }
        Value::Array(items) => {
            arrays.push(prefix.to_owned());
            for (index, value) in items.iter().enumerate() {
                inventory(value, &format!("{prefix}/{index}"), leaves, arrays, objects);
            }
        }
        Value::String(_) => leaves.push(prefix.to_owned()),
        _ => panic!("contract has ONLY string leaves"),
    }
}
fn remove(value: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').expect("non-root leaf");
    value
        .pointer_mut(parent)
        .expect("required parent")
        .as_object_mut()
        .expect("object member")
        .remove(key)
        .expect("required member");
}

#[test]
fn complete_independent_tree_cli_orders_renames_and_recovery() {
    let p = Probe::new();
    let expected = baseline();
    let data = export(PACKAGE, CORE).expect("current ordinary let subset");
    assert_eq!(data.len(), 5298);
    assert_eq!(
        serde_json::from_str::<Value>(&data).expect("native JSON"),
        expected
    );
    for reverse in [false, true, false] {
        assert_eq!(decoded(p.native(CORE, reverse)), expected);
        assert_eq!(
            decoded(p.run(
                &p.cue,
                &["export"],
                &[("package.cue", PACKAGE), ("ordinary.cue", CORE)],
                reverse
            )),
            expected
        );
    }
    let renamed = CORE
        .replace("Node", "CapturedNode")
        .replace("let Field =", "let CapturedField =")
        .replace(", Field", ", CapturedField")
        .replace("let Context =", "let CapturedContext =")
        .replace(", Context", ", CapturedContext")
        .replace("\n\t\t\tContext,", "\n\t\t\tCapturedContext,")
        .replace("let V =", "let CapturedValue =")
        .replace("[V,", "[CapturedValue,");
    assert_eq!(decoded(p.native(&renamed, true)), expected);
    assert_eq!(
        decoded(p.run(
            &p.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("renamed.cue", &renamed)],
            false
        )),
        expected
    );
    assert!(matches!(
        export_with_limit(PACKAGE, CORE, 0),
        Err(Error::Limit(_))
    ));
    assert_eq!(
        export(PACKAGE, SHAPE),
        Err(Error::Syntax("declaration separator"))
    );
    let collision = CORE.replace("Node", "text");
    assert!(matches!(
        export(PACKAGE, &collision),
        Err(Error::Unsupported(_))
    ));
    failed(&p.run(
        &p.cue,
        &["export"],
        &[("package.cue", PACKAGE), ("collision.cue", &collision)],
        false,
    ));
    assert_eq!(decoded(p.native(CORE, false)), expected);
}

#[test]
fn four_helper_source_effects_reach_complete_native_and_raw_policies() {
    let p = Probe::new();
    for (old, new, expected) in [
        (
            "value: \"node\"",
            "value: \"changed_node\"",
            expected("changed_node", "field", "_context", "value"),
        ),
        (
            "field: \"field\"",
            "field: \"changed_field\"",
            expected("node", "changed_field", "_context", "value"),
        ),
        (
            "value: \"_context\"",
            "value: \"changed_context\"",
            expected("node", "field", "changed_context", "value"),
        ),
        (
            "field: \"value\"",
            "field: \"changed_value\"",
            expected("node", "field", "_context", "changed_value"),
        ),
    ] {
        // Limit the V mutation to its declaration: boolean's own field stays value.
        let source = CORE.replacen(old, new, 1);
        assert_ne!(source, CORE);
        let data = export(PACKAGE, &source).expect("source-derived native DATA");
        assert_eq!(
            serde_json::from_str::<Value>(&data).expect("DATA"),
            expected
        );
        let native = decoded(p.full(Some(("body.json", &data)), SELECTOR, false, true));
        let raw = decoded(p.full(Some(("body.cue", &source)), SELECTOR, true, true));
        assert_eq!(native, raw);
        for (name, value) in expected["cases"].as_object().expect("16 expected cases") {
            assert_eq!(&native["cases"][name], value);
        }
    }
}

#[test]
fn every_required_leaf_case_and_whole_body_is_independently_incomplete() {
    let p = Probe::new();
    let expected = baseline();
    let (mut leaves, mut arrays, mut objects) = (Vec::new(), Vec::new(), Vec::new());
    inventory(&expected, "", &mut leaves, &mut arrays, &mut objects);
    assert_eq!(
        (
            expected["cases"].as_object().expect("cases").len(),
            leaves.len(),
            arrays.len()
        ),
        (16, 118, 13)
    );
    let mut omissions = leaves;
    omissions.extend(
        expected["cases"]
            .as_object()
            .expect("cases")
            .keys()
            .map(|name| format!("/cases/{name}")),
    );
    for pointer in omissions {
        let mut missing = expected.clone();
        remove(&mut missing, &pointer);
        let source = missing.to_string();
        let body = Some(("missing.json", source.as_str()));
        assert!(
            p.isolated(body, "eval", false).status.success(),
            "{pointer}"
        );
        let out = p.isolated(body, "export", true);
        failed(&out);
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("incomplete value"),
            "{pointer}"
        );
        let before = p.full_exports.get();
        failed(&p.full(body, SELECTOR, false, true));
        assert_eq!(p.full_exports.get(), before, "stop before full policy");
    }
    for body in [None, Some(("missing.json", "{}"))] {
        assert!(p.isolated(body, "eval", false).status.success());
        failed(&p.isolated(body, "export", false));
        let before = p.full_exports.get();
        failed(&p.full(body, SELECTOR, false, true));
        assert_eq!(p.full_exports.get(), before);
    }
}

#[test]
fn all_types_object_closedness_and_thirteen_fixed_array_shapes() {
    let p = Probe::new();
    let expected = baseline();
    let (mut leaves, mut arrays, mut objects) = (Vec::new(), Vec::new(), Vec::new());
    inventory(&expected, "", &mut leaves, &mut arrays, &mut objects);
    assert_eq!((leaves.len(), arrays.len()), (118, 13));
    for pointer in leaves {
        let mut wrong = expected.clone();
        *wrong.pointer_mut(&pointer).expect("independent leaf") = json!(42);
        let out = p.isolated(Some(("wrong.json", &wrong.to_string())), "export", false);
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("conflicting values"));
    }
    for pointer in objects {
        let mut wrong = expected.clone();
        wrong
            .pointer_mut(&pointer)
            .expect("independent object")
            .as_object_mut()
            .expect("object")
            .insert("extra".to_owned(), json!("x"));
        let out = p.isolated(Some(("wrong.json", &wrong.to_string())), "export", false);
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("field not allowed"));
    }
    for pointer in arrays {
        for longer in [false, true] {
            let mut wrong = expected.clone();
            let items = wrong
                .pointer_mut(&pointer)
                .expect("independent array")
                .as_array_mut()
                .expect("array");
            if longer {
                items.push(items[0].clone());
            } else {
                items.pop().expect("nonempty fixed array");
            }
            let out = p.isolated(Some(("wrong.json", &wrong.to_string())), "export", true);
            failed(&out);
            let error = String::from_utf8_lossy(&out.stderr);
            assert!(
                error.contains("incompatible list lengths"),
                "{pointer}: {error}"
            );
            assert!(!error.contains("incomplete value"));
        }
    }
}

#[test]
fn whole_policy_raw_native_both_orders_and_unresolved_contexts() {
    let p = Probe::new();
    let data = export(PACKAGE, CORE).expect("native body");
    let mixed_last = "package bootstrap\n#InstructionLowering & {module:[_,{op:\"lastRule\"},{body:[_,_,_,_,{op:\"nextRule\"}]},_]}\n";
    let mixed_first = "package bootstrap\n#InstructionLowering & {module:[_,{op:\"firstRule\"},{body:[_,_,_,_,{op:\"previousRule\"}]},_]}\n";
    for context in [SELECTOR, TWICE, REVERSE, mixed_last, mixed_first] {
        let native = decoded(p.full(Some(("body.json", &data)), context, false, true));
        for reverse in [false, true] {
            assert_eq!(
                decoded(p.full(Some(("body.cue", CORE)), context, reverse, true)),
                native
            );
        }
        assert_eq!(
            decoded(p.full(Some(("body.json", &data)), context, true, true)),
            native
        );
        for (name, value) in baseline()["cases"].as_object().expect("independent cases") {
            assert_eq!(&native["cases"][name], value);
        }
        assert_eq!(
            native
                .pointer("/cases/sum/args/1/value/value/field")
                .expect("sum R"),
            if context == TWICE { "left" } else { "right" }
        );
        for pointer in [
            "/cases/binaryRuntime/parts/4",
            "/cases/less/args/1",
            "/cases/lessEqual/args/1",
            "/cases/greater/args/1",
            "/cases/greaterEqual/args/1",
            "/cases/notEqual/args/1",
            "/cases/remainder/args/1",
            "/cases/unify/parts/1/args/1",
        ] {
            assert_eq!(
                native
                    .pointer(&format!("{pointer}/value/value/field"))
                    .expect("other R site"),
                "right"
            );
        }
    }
}

#[test]
fn real_backfill_and_fail_stop_shell_conditionals_without_shape_composition() {
    let p = Probe::new();
    let mut missing = baseline();
    remove(&mut missing, "/cases/text/op");
    let missing = missing.to_string();
    let body = Some(("missing.json", missing.as_str()));
    failed(&p.isolated(body, "export", false));
    let bypass = decoded(p.full(body, SELECTOR, false, false));
    assert_eq!(
        bypass["cases"]["text"]["op"], "call",
        "actual full-schema backfill"
    );
    let before = p.full_exports.get();
    failed(&p.full(body, SELECTOR, false, true));
    assert_eq!(p.full_exports.get(), before);
    // Exercise actual shell fail-stop semantics, not set -e assumptions in if/||.
    let script = format!(
        r#"
consume() {{
  '{cue}' export shape.cue missing.json >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue -e '#InstructionRightField' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue operand.cue -e '#InstructionRightOperand' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue operand.cue group.cue -e '#InstructionOperandCases' >/dev/null || return
  '{cue}' export schema.cue bridge.cue binary-placed.cue -e '#InstructionBinaryTables' >/dev/null || return
  : > full-export-started
  '{cue}' export schema.cue bridge.cue missing.json binary-placed.cue unary.cue text.cue first.cue step.cue right.cue operand.cue group.cue context.cue
}}
case "$1" in
  normal) consume ;;
  conditional) if consume; then exit 99; else exit 17; fi ;;
  disjunction) consume || exit 17 ;;
esac
"#,
        cue = p.cue.display()
    );
    fs::write(p.dir.join("consume.sh"), script).expect("verification-only actual pipeline");
    for mode in ["normal", "conditional", "disjunction"] {
        let out = p.run(&p.shell, &["consume.sh", mode], &[], false);
        failed(&out);
        assert!(!p.dir.join("full-export-started").exists());
    }
    // Full policy success above also proves the closed 16-case shape was NOT loaded.
}

#[test]
fn go_opcode_shape_rejections_are_separate_from_typed_selector_limit() {
    let p = Probe::new();
    for pointer in ["/cases/text/op", "/cases/text/args/0/op"] {
        let mut invalid = baseline();
        *invalid.pointer_mut(pointer).expect("opcode") = json!("unknownCoreOpcode");
        let data = invalid.to_string();
        assert!(
            p.isolated(Some(("body.json", &data)), "export", false)
                .status
                .success(),
            "type-only shape has no opcode answers"
        );
        failed(&p.full(Some(("body.json", &data)), SELECTOR, false, true));
    }
    let mut invalid = baseline();
    invalid["cases"]["text"]["args"] = json!({});
    failed(&p.isolated(
        Some(("invalid.json", &invalid.to_string())),
        "export",
        false,
    ));
    let mut long = baseline();
    long["cases"]["text"]["args"][1]["field"] = json!("x".repeat(129));
    assert!(
        p.full(
            Some(("body.json", &long.to_string())),
            SELECTOR,
            false,
            true
        )
        .status
        .success(),
        "Go accepts typed-limit control; real compiler rejection is independently Nix-owned"
    );
}
