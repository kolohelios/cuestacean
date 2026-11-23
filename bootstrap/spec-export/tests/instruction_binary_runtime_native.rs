use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const OWNER: &str = include_str!("../../instruction-binary-runtime.cue");
const ROOT: &str = include_str!("../../instruction-binary-runtime-export.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const GROUP_ROOT: &str = include_str!("../../instruction-operand-cases-export.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const RIGHT_ROOT: &str = include_str!("../../instruction-right-field-export.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const OPERAND_ROOT: &str = include_str!("../../instruction-right-operand-export.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const MODULE: &str = include_str!("../../instruction-module.cue");
const MODULE_SHAPE: &str = include_str!("../../instruction-module-schema.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const HELPER: &str = "#InstructionBinaryRuntime";

fn wrapper(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":field}}})
}
fn expected_binary() -> Value {
    let mut aliases = serde_json::Map::new();
    let mut cases = serde_json::Map::new();
    for (name, method) in [
        ("index", "index"),
        ("difference", "difference"),
        ("product", "product"),
        ("divide", "divide"),
        ("modulo", "modulo"),
        ("quotient", "quotient"),
        ("recordIndex", "record_index"),
        ("listIndex", "list_index"),
        ("logicalAnd", "logical_and"),
        ("logicalOr", "logical_or"),
        ("equal", "equal"),
    ] {
        aliases.insert(name.into(), json!("binaryRuntime"));
        cases.insert(
            name.into(),
            json!({"op":"text","value":format!("runtime::{method}")}),
        );
    }
    json!({"aliases":aliases,"select":{"op":"select","field":"op","cases":cases}})
}
fn expected_group(left: &str, right: &str) -> Value {
    let mut cases = serde_json::Map::new();
    for (name, method) in [
        ("sum", "sum"),
        ("less", "less"),
        ("lessEqual", "less_equal"),
        ("greater", "greater"),
        ("greaterEqual", "greater_equal"),
        ("notEqual", "not_equal"),
        ("remainder", "remainder"),
    ] {
        cases.insert(name.into(), json!({"op":"call","path":format!("runtime::{method}"),"args":[wrapper(left),wrapper(right)]}));
    }
    cases.insert(
        "unify".into(),
        json!({"op":"concat","parts":[
        {"op":"text","value":"(|_context: &mut runtime::EvaluationContext| "},
        {"op":"call","path":"runtime::unify","args":[wrapper(left),wrapper(right)]},
        {"op":"text","value":")(_context)"}]}),
    );
    json!({"leftOperand":wrapper(left),"cases":cases})
}
fn expected(binary: &Value, left: &str, right: &str, punctuation: &str) -> Value {
    json!({"op":"concat","parts":[binary["select"],
        {"op":"text","value":punctuation},wrapper(left),
        {"op":"text","value":", "},wrapper(right),{"op":"text","value":")"}]})
}
fn parsed(text: &str) -> Value {
    serde_json::from_str(text).expect("whole independently checked JSON")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete process JSON")
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file());
    fs::File::open(path).expect("readable mandatory tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
struct Chain {
    binary: String,
    operand: String,
    group: String,
    adapter: String,
}
struct Probe {
    dir: PathBuf,
    native: PathBuf,
    cue: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "native-binary-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned Go CUE")
            .canonicalize()
            .expect("absolute Go CUE");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        executable(&cue);
        Self { dir, native, cue }
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, text) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, text).expect("explicit owned input");
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
    fn native(&self, owner: &str, adapter: &str, reverse: bool) -> Output {
        self.run(
            &self.native,
            &[],
            &[("unrelated-owner.cue", owner), ("other-input.cue", adapter)],
            reverse,
        )
    }
    fn chain(&self, binary: &str, group: &str, operand: &str) -> Chain {
        // Each stage consumes WHOLE independently native DATA, never an expectation.
        let right = export(RIGHT, RIGHT_ROOT).expect("native Right stage");
        assert_eq!(parsed(&right), json!({"field":"right"}));
        let r_adapter = format!("{OPERAND_ROOT}\n#InstructionRightField: {right}");
        let r = export(operand, &r_adapter).expect("native R stage");
        let g_adapter = format!("{GROUP_ROOT}\n#InstructionRightOperand: {r}");
        let g = export(group, &g_adapter).expect("native Group stage");
        let b = export(PACKAGE, binary).expect("independent native Binary stage");
        for (owner, adapter, data) in [
            (RIGHT, RIGHT_ROOT, &right),
            (operand, &r_adapter, &r),
            (group, &g_adapter, &g),
            (binary, PACKAGE, &b),
        ] {
            assert!(owner.len() + adapter.len() <= 65536);
            assert!(data.len() <= 65536);
            assert_eq!(decoded(self.native(owner, adapter, false)), parsed(data));
        }
        let adapter = format!(
            "{ROOT}\n#InstructionBinaryTables: {b}\n#InstructionOperandCases: {g}\n#InstructionRightOperand: {r}"
        );
        Chain {
            binary: b,
            operand: r,
            group: g,
            adapter,
        }
    }
    fn import(&self, helper: &str, data: &str) -> String {
        let label = format!("{helper}:");
        let out = self.run(
            &self.cue,
            &["import", "-p", "bootstrap", "-l", &label, "--outfile", "-"],
            &[("data.json", data)],
            false,
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("whole native DATA import")
    }
    fn policy(&self, providers: [&str; 5], reverse: bool) -> Value {
        let [binary, operand, group, runtime, right] = providers;
        let sources = [
            ("binary.cue", binary),
            ("r.cue", operand),
            ("group.cue", group),
            ("runtime.cue", runtime),
            ("right.cue", right),
        ];
        // Discard checked stdout; isolated gates and composition reuse SAME sources.
        for (helper, indices) in [
            ("#InstructionRightField", vec![4]),
            ("#InstructionRightOperand", vec![4, 1]),
            ("#InstructionOperandCases", vec![4, 1, 2]),
            ("#InstructionBinaryTables", vec![0]),
            (HELPER, vec![4, 1, 2, 0, 3]),
        ] {
            let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
            inputs.extend(indices.into_iter().map(|i| sources[i]));
            let gate = self.run(&self.cue, &["export", "-e", helper], &inputs, reverse);
            assert!(
                gate.status.success(),
                "{helper}: {}",
                String::from_utf8_lossy(&gate.stderr)
            );
        }
        let gate = self.run(
            &self.cue,
            &["export"],
            &[("shape.cue", SHAPE), ("core.cue", CORE)],
            reverse,
        );
        assert!(gate.status.success(), "independent Core118");
        for helper in [
            "#InstructionModuleText",
            "#InstructionModuleFirst",
            "#InstructionModuleStep",
            "#InstructionModule",
        ] {
            let checked = self.run(
                &self.cue,
                &["export", "-e", helper],
                &[
                    ("module-shape.cue", MODULE_SHAPE),
                    ("module.cue", MODULE),
                    ("text.cue", TEXT),
                    ("first.cue", FIRST),
                    ("step.cue", STEP),
                ],
                reverse,
            );
            assert!(
                checked.status.success(),
                "isolated Module completeness: {checked:?}"
            );
        }
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("core.cue", CORE),
        ];
        inputs.extend(sources);
        inputs.extend([
            ("unary.cue", UNARY),
            ("text.cue", TEXT),
            ("module.cue", MODULE),
            ("first.cue", FIRST),
            ("step.cue", STEP),
            ("context.cue", SELECTOR),
        ]);
        decoded(self.run(&self.cue, &["export"], &inputs, reverse))
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}

#[test]
fn complete_independent_tree_native_library_cli_orders_repeats_go_and_names() {
    let p = Probe::new();
    let c = p.chain(BINARY, GROUP, OPERAND);
    let binary = expected_binary();
    assert_eq!(parsed(&c.binary), binary);
    assert_eq!(parsed(&c.operand), wrapper("right"));
    assert_eq!(parsed(&c.group), expected_group("left", "right"));
    let want = expected(&binary, "left", "right", "(");
    let data = export(OWNER, &c.adapter).expect("native whole Runtime");
    assert_eq!(data.len(), 1652);
    assert_eq!(parsed(&data), want);
    for reverse in [false, true] {
        for _ in 0..2 {
            let (a, b) = if reverse {
                (c.adapter.as_str(), OWNER)
            } else {
                (OWNER, c.adapter.as_str())
            };
            assert_eq!(export(a, b).expect("library orders and repeats"), data);
            let out = p.native(OWNER, &c.adapter, reverse);
            assert!(out.status.success());
            assert_eq!(out.stdout, data.as_bytes());
        }
        let go = p.run(
            &p.cue,
            &["export"],
            &[("a.cue", OWNER), ("b.cue", &c.adapter)],
            reverse,
        );
        assert_eq!(go.stdout.len(), 2286);
        assert_ne!(go.stdout, data.as_bytes());
        assert_eq!(decoded(go), want);
        let rename = |text: &str| {
            text.replace(HELPER, "#OrdinaryTree")
                .replace("#InstructionOperandCases", "#LiteralGroup")
                .replace("#InstructionBinaryTables", "#MethodData")
                .replace("#InstructionRightOperand", "#OperandData")
                .replace("let L =", "let LocalLeft =")
                .replace("let R =", "let LocalRight =")
                .replace("}, L,", "}, LocalLeft,")
                .replace("}, R,", "}, LocalRight,")
        };
        let (owner, adapter) = (rename(OWNER), rename(&c.adapter));
        assert_eq!(
            parsed(&export(&owner, &adapter).expect("ordinary renamed library")),
            want
        );
        assert_eq!(decoded(p.native(&owner, &adapter, reverse)), want);
        assert_eq!(
            decoded(p.run(
                &p.cue,
                &["export"],
                &[("arbitrary.cue", &owner), ("unrelated.cue", &adapter)],
                reverse
            )),
            want
        );
    }
}

#[test]
fn all_six_source_effects_rebuild_native_chain_and_complete_raw_native_policies() {
    let p = Probe::new();
    let baseline_chain = p.chain(BINARY, GROUP, OPERAND);
    let placed_binary = p.import("#InstructionBinaryTables", &baseline_chain.binary);
    let baseline = p.policy([&placed_binary, OPERAND, GROUP, OWNER, RIGHT], false);
    for effect in 0..7 {
        let mut binary = BINARY.to_owned();
        let mut group = GROUP.to_owned();
        let mut operand = OPERAND.to_owned();
        let mut owner = OWNER.to_owned();
        let mut b = expected_binary();
        let (mut group_left, mut runtime_left, mut right, mut punctuation) =
            ("left", "left", "right", "(");
        match effect {
            0 => {}
            1 => {
                owner = owner.replace("value: \"(\"", "value: \"( \"");
                punctuation = "( ";
            }
            2 => {
                owner = owner.replace(
                    "#InstructionOperandCases.leftOperand",
                    "#InstructionRightOperand",
                );
                runtime_left = "right";
            }
            3 => {
                binary = binary.replace(
                    "difference:       \"difference\"",
                    "difference:       \"sum\"",
                );
                b["select"]["cases"]["difference"]["value"] = json!("runtime::sum");
            }
            4 => {
                assert_eq!(BINARY.matches("index:       \"index\"").count(), 1);
                assert_eq!(
                    BINARY.matches("difference:       \"difference\"").count(),
                    1
                );
                binary = binary
                    .replace("index:       \"index\"", "difference:       \"index\"")
                    .replace(
                        "difference:       \"difference\"",
                        "index:       \"difference\"",
                    );
                assert_eq!(binary.matches("difference:       \"index\"").count(), 1);
                assert_eq!(binary.matches("index:       \"difference\"").count(), 1);
                assert!(!binary.contains("index:       \"index\""));
                assert!(!binary.contains("difference:       \"difference\""));
                assert_eq!(binary.matches("index:       ").count(), 1);
                assert_eq!(binary.matches("difference:       ").count(), 1);
                assert_eq!(
                    binary
                        .replace("difference:       \"index\"", "index:       \"index\"")
                        .replace(
                            "index:       \"difference\"",
                            "difference:       \"difference\""
                        ),
                    BINARY,
                    "actual source keys swap; every unrelated source byte is preserved"
                );
                b["select"]["cases"]["index"]["value"] = json!("runtime::difference");
                b["select"]["cases"]["difference"]["value"] = json!("runtime::index");
            }
            5 => {
                group = group.replace("field: \"left\"", "field: \"right\"");
                group_left = "right";
                runtime_left = "right";
            }
            6 => {
                operand = operand.replace("#InstructionRightField.field", "\"left\"");
                right = "left";
            }
            _ => unreachable!("finite independently specified effects"),
        }
        // No verifier result/expected tree is passed to this constructor chain.
        let c = p.chain(&binary, &group, &operand);
        let g = expected_group(group_left, right);
        assert_eq!(parsed(&c.binary), b);
        assert_eq!(parsed(&c.operand), wrapper(right));
        assert_eq!(parsed(&c.group), g);
        let want = expected(&b, runtime_left, right, punctuation);
        let data = export(&owner, &c.adapter).expect("fresh source-derived whole Runtime");
        assert_eq!(parsed(&data), want);
        let b_provider = p.import("#InstructionBinaryTables", &c.binary);
        let r_provider = p.import("#InstructionRightOperand", &c.operand);
        let g_provider = p.import("#InstructionOperandCases", &c.group);
        let runtime = p.import(HELPER, &data);
        let selected_right = p.import(
            "#InstructionRightField",
            &export(RIGHT, RIGHT_ROOT).expect("fresh native Right"),
        );
        let mut predicted = baseline.clone();
        predicted["cases"]["binaryRuntime"] = want.clone();
        for (name, body) in g["cases"].as_object().expect("independent eight cases") {
            predicted["cases"][name] = body.clone();
        }
        if effect != 0 {
            assert_ne!(
                predicted, baseline,
                "source effect {effect} must be observable"
            );
        }
        for reverse in [false, true] {
            assert_eq!(decoded(p.native(&owner, &c.adapter, reverse)), want);
            let raw = p.policy([&b_provider, &operand, &group, &owner, RIGHT], reverse);
            let native = p.policy(
                [
                    &b_provider,
                    &r_provider,
                    &g_provider,
                    &runtime,
                    &selected_right,
                ],
                reverse,
            );
            assert_eq!(
                raw, predicted,
                "complete raw policy/siblings effect {effect}"
            );
            assert_eq!(
                native, predicted,
                "complete native policy/siblings effect {effect}"
            );
        }
    }
}

#[test]
fn precise_api_cli_failures_empty_output_and_fresh_recovery() {
    let p = Probe::new();
    let c = p.chain(BINARY, GROUP, OPERAND);
    let unknown = OWNER.replace("#InstructionRightOperand", "#ActuallyAbsent");
    let oversized = " ".repeat(65537);
    let raw = format!(
        "{ROOT}\n#InstructionBinaryTables: {}\n{}\n{}\n{}",
        c.binary,
        GROUP.replace("package bootstrap", ""),
        OPERAND.replace("package bootstrap", ""),
        RIGHT.replace("package bootstrap", "")
    );
    let missing_binary = format!(
        "{ROOT}\n#InstructionOperandCases: {}\n#InstructionRightOperand: {}",
        c.group, c.operand
    );
    let missing_group = format!(
        "{ROOT}\n#InstructionBinaryTables: {}\n#InstructionRightOperand: {}",
        c.binary, c.operand
    );
    let missing_r = format!(
        "{ROOT}\n#InstructionBinaryTables: {}\n#InstructionOperandCases: {}",
        c.binary, c.group
    );
    for (owner, adapter, error) in [
        (
            OWNER,
            missing_binary.as_str(),
            Error::Unknown("#InstructionBinaryTables".into()),
        ),
        (
            OWNER,
            missing_group.as_str(),
            Error::Unknown("#InstructionOperandCases".into()),
        ),
        (
            OWNER,
            missing_r.as_str(),
            Error::Unknown("#InstructionRightOperand".into()),
        ),
        (
            unknown.as_str(),
            c.adapter.as_str(),
            Error::Unknown("#ActuallyAbsent".into()),
        ),
        (
            "package bootstrap\n#InstructionBinaryRuntime: {",
            c.adapter.as_str(),
            Error::Syntax("field label"),
        ),
        (
            oversized.as_str(),
            c.adapter.as_str(),
            Error::Limit("source bytes"),
        ),
        (
            OWNER,
            raw.as_str(),
            Error::Unsupported("literal projection scalar required"),
        ),
    ] {
        assert_eq!(export(owner, adapter), Err(error.clone()));
        for reverse in [false, true] {
            let out = p.native(owner, adapter, reverse);
            assert_eq!(out.status.code(), Some(2));
            assert!(out.stdout.is_empty());
            assert_eq!(
                String::from_utf8(out.stderr)
                    .expect("diagnostic UTF-8")
                    .trim(),
                error.to_string()
            );
        }
        assert_eq!(
            parsed(&export(OWNER, &c.adapter).expect("library recovery")),
            expected(&expected_binary(), "left", "right", "(")
        );
        assert_eq!(
            decoded(p.native(OWNER, &c.adapter, false)),
            expected(&expected_binary(), "left", "right", "(")
        );
    }
    for inputs in [
        vec![],
        vec![("one.cue", OWNER)],
        vec![
            ("one.cue", OWNER),
            ("two.cue", &c.adapter),
            ("three.cue", ROOT),
        ],
    ] {
        let out = p.run(&p.native, &[], &inputs, false);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert_eq!(
            String::from_utf8(out.stderr).expect("usage UTF-8").trim(),
            "usage: cuestacean-spec-export <schema.cue> <program.cue>"
        );
        assert_eq!(
            decoded(p.native(OWNER, &c.adapter, true)),
            expected(&expected_binary(), "left", "right", "(")
        );
    }
    let absent = p.dir.join("definitely-absent.cue");
    assert!(!absent.exists());
    let path = absent.to_str().expect("absent path UTF-8");
    let out = p.run(&p.native, &[path, path], &[], false);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .expect("read diagnostic UTF-8")
            .contains("No such file or directory")
    );
    assert_eq!(
        decoded(p.native(OWNER, &c.adapter, true)),
        expected(&expected_binary(), "left", "right", "(")
    );
}

// Counts original ordinary literal syntax nodes and WHOLE target edges, not just
// selected output. Expected trees are independently handwritten above.
fn inventory(value: &Value) -> (usize, usize, usize) {
    let mut total = (1, 0, 0);
    let children: Vec<&Value> = match value {
        Value::Object(fields) => {
            total.1 = fields.len();
            fields.values().collect()
        }
        Value::Array(items) => {
            total.2 = items.len();
            items.iter().collect()
        }
        _ => Vec::new(),
    };
    for child in children {
        let (nodes, fields, items) = inventory(child);
        total.0 += nodes;
        total.1 += fields;
        total.2 += items;
    }
    total
}

#[test]
fn independent_preparation_ledger_source_provider_adapter_bounds_and_pipeline_isolation() {
    let p = Probe::new();
    let c = p.chain(BINARY, GROUP, OPERAND);
    assert_eq!(inventory(&expected_binary()), (50, 49, 0));
    assert_eq!(inventory(&expected_group("left", "right")), (162, 142, 19));
    assert_eq!(inventory(&wrapper("right")), (7, 6, 0));
    assert_eq!(parsed(&c.binary), expected_binary());
    assert_eq!(parsed(&c.group), expected_group("left", "right"));
    assert_eq!(parsed(&c.operand), wrapper("right"));
    // Owner: Struct + L Projection + R Ref + concat Literal + parts List +
    // Binary Projection + three(Struct+two Literals) + two resolved lexical uses.
    let owner_nodes = 1 + 1 + 1 + 1 + 1 + 1 + 3 * 3 + 2;
    let original_nodes = 50 + 162 + 7 + owner_nodes + 1; // companion root Ref
    assert_eq!(owner_nodes, 17);
    assert_eq!(original_nodes, 237);
    // Entry + arena + two descriptor/lookup/root/replacement schedules + WHOLE
    // Group/Binary field edges + Group list edges, BEFORE normal evaluation.
    let preparation = 1 + original_nodes + 2 * 4 + 2 * (142 + 49) + 19;
    assert_eq!(preparation, 647);
    let want = expected(&expected_binary(), "left", "right", "(");
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (c.adapter.as_str(), OWNER)
        } else {
            (OWNER, c.adapter.as_str())
        };
        assert_eq!(
            export_with_limit(a, b, preparation - 1),
            Err(Error::Limit("evaluation steps"))
        );
        let recovered = export_with_limit(a, b, 4096).expect("unchanged hard allowance recovery");
        assert_eq!(parsed(&recovered), want);
        assert!(recovered.len() <= 65536);
        assert_eq!(decoded(p.native(OWNER, &c.adapter, reverse)), want);
    }
    for input in [
        OWNER,
        ROOT,
        RIGHT,
        RIGHT_ROOT,
        OPERAND,
        OPERAND_ROOT,
        GROUP,
        GROUP_ROOT,
        BINARY,
        PACKAGE,
        &c.binary,
        &c.group,
        &c.operand,
        &c.adapter,
    ] {
        assert!(
            input.len() <= 65536,
            "individual source/provider/adapter bound"
        );
    }
    assert!(
        OWNER.len() + c.adapter.len() <= 65536,
        "combined constructor source bound"
    );
    let mut oversized_pair = c.adapter.clone();
    oversized_pair.push_str(&" ".repeat(65537 - OWNER.len() - c.adapter.len()));
    assert!(oversized_pair.len() <= 65536);
    assert_eq!(
        export(OWNER, &oversized_pair),
        Err(Error::Limit("source bytes"))
    );
    let failed = p.native(OWNER, &oversized_pair, false);
    assert_eq!(failed.status.code(), Some(2));
    assert!(failed.stdout.is_empty());
    assert_eq!(
        String::from_utf8(failed.stderr)
            .expect("combined bound diagnostic")
            .trim(),
        "resource limit: source bytes"
    );
    assert_eq!(
        parsed(&export(OWNER, &c.adapter).expect("post-bound recovery")),
        want
    );
    let b = p.import("#InstructionBinaryTables", &c.binary);
    let r = p.import("#InstructionRightOperand", &c.operand);
    let g = p.import("#InstructionOperandCases", &c.group);
    let runtime = export(OWNER, &c.adapter).expect("fresh runtime for consumer");
    let rt = p.import(HELPER, &runtime);
    let right = p.import(
        "#InstructionRightField",
        &export(RIGHT, RIGHT_ROOT).expect("fresh Right"),
    );
    let raw = p.policy([&b, OPERAND, GROUP, OWNER, RIGHT], false);
    let selected = p.policy([&b, &r, &g, &rt, &right], true);
    assert_eq!(selected, raw, "complete isolated native pipeline");
    assert_eq!(selected["cases"]["binaryRuntime"], want);
    assert_eq!(
        selected["cases"]["sum"],
        expected_group("left", "right")["cases"]["sum"]
    );
}
