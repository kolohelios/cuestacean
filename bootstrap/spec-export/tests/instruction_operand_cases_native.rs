use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const ROOT: &str = include_str!("../../instruction-operand-cases-export.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const RIGHT_ROOT: &str = include_str!("../../instruction-right-field-export.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const OPERAND_ROOT: &str = include_str!("../../instruction-right-operand-export.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const RUNTIME: &str = include_str!("../../instruction-binary-runtime.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const HELPER: &str = "#InstructionOperandCases";

fn wrapper(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":field}}})
}
fn expected(left: &str, right: &str) -> Value {
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
        cases.insert(
            name.into(),
            json!({"op":"call","path":format!("runtime::{method}"),"args":[wrapper(left),wrapper(right)]}),
        );
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
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file());
    fs::File::open(path).expect("readable mandatory tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
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
            "native-operand-cases-{}-{}",
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
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        let _jq = discover("jq");
        let _shell = discover("sh");
        Self {
            dir,
            native,
            cue: discover("cue"),
        }
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, text) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, text).expect("explicit input");
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
            &[
                ("ordinary-input.cue", owner),
                ("unrelated-root.cue", adapter),
            ],
            reverse,
        )
    }
    fn chain(&self, operand: &str) -> (String, String) {
        // Separate programs: raw preference -> whole Right DATA -> whole R DATA.
        let right = export(RIGHT, RIGHT_ROOT).expect("native Right");
        assert_eq!(
            serde_json::from_str::<Value>(&right).expect("whole native Right JSON"),
            json!({"field":"right"})
        );
        assert_eq!(right.len(), 23);
        let adapter = format!("{OPERAND_ROOT}\n#InstructionRightField: {right}");
        let r = export(operand, &adapter).expect("native whole R");
        assert_eq!(
            decoded(self.native(operand, &adapter, false)),
            serde_json::from_str::<Value>(&r).expect("whole native R JSON")
        );
        let group_adapter = format!("{ROOT}\n#InstructionRightOperand: {r}");
        for input in [GROUP, ROOT, &right, &adapter, &r, &group_adapter] {
            assert!(input.len() <= 65536);
        }
        (r, group_adapter)
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
        String::from_utf8(out.stdout).expect("successful whole JSON import")
    }
    fn policy(&self, operand: &str, group: &str, reverse: bool) -> Value {
        // Go verification only. Each check discards stdout and reuses SAME providers.
        for (helper, providers) in [
            ("#InstructionRightField", vec![("right.cue", RIGHT)]),
            (
                "#InstructionRightOperand",
                vec![("right.cue", RIGHT), ("r.cue", operand)],
            ),
            (
                HELPER,
                vec![
                    ("right.cue", RIGHT),
                    ("r.cue", operand),
                    ("group.cue", group),
                ],
            ),
        ] {
            let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
            inputs.extend(providers);
            let gate = self.run(&self.cue, &["export", "-e", helper], &inputs, reverse);
            assert!(
                gate.status.success(),
                "{}",
                String::from_utf8_lossy(&gate.stderr)
            );
        }
        let core = self.run(
            &self.cue,
            &["export"],
            &[("shape.cue", SHAPE), ("core.cue", CORE)],
            reverse,
        );
        assert!(core.status.success(), "isolated Core118");
        let binary = decoded(self.run(
            &self.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary.cue", BINARY)],
            reverse,
        ));
        let binary = self.import("#InstructionBinaryTables", &binary.to_string());
        let gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionBinaryTables"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("binary.cue", &binary),
            ],
            reverse,
        );
        assert!(gate.status.success(), "isolated Binary35");
        let runtime = self.run(
            &self.cue,
            &["export", "-e", "#InstructionBinaryRuntime"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", RIGHT),
                ("r.cue", operand),
                ("group.cue", group),
                ("binary.cue", &binary),
                ("runtime.cue", RUNTIME),
            ],
            reverse,
        );
        assert!(runtime.status.success(), "isolated SAME-provider Runtime");
        decoded(self.run(
            &self.cue,
            &["export"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("core.cue", CORE),
                ("right.cue", RIGHT),
                ("r.cue", operand),
                ("group.cue", group),
                ("runtime.cue", RUNTIME),
                ("binary.cue", &binary),
                ("unary.cue", UNARY),
                ("text.cue", TEXT),
                ("first.cue", FIRST),
                ("step.cue", STEP),
                ("context.cue", SELECTOR),
            ],
            reverse,
        ))
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}

#[test]
fn whole_hand_tree_native_orders_repeats_go_encoding_and_ordinary_names() {
    let p = Probe::new();
    let (r, adapter) = p.chain(OPERAND);
    assert_eq!(r.len(), 120);
    let want = expected("left", "right");
    let data = export(GROUP, &adapter).expect("native whole Group");
    assert_eq!(data.len(), 4549);
    assert_eq!(
        serde_json::from_str::<Value>(&data).expect("whole native Group JSON"),
        want
    );
    for reverse in [false, true] {
        for _ in 0..2 {
            let out = p.native(GROUP, &adapter, reverse);
            assert!(out.status.success());
            assert_eq!(out.stdout, data.as_bytes());
            assert_eq!(
                export(&adapter, GROUP).expect("reversed library export"),
                data
            );
        }
        let go = p.run(
            &p.cue,
            &["export"],
            &[("a.cue", GROUP), ("b.cue", &adapter)],
            reverse,
        );
        assert_eq!(go.stdout.len(), 6821);
        assert_ne!(go.stdout, data.as_bytes());
        assert_eq!(decoded(go), want);
        let rename = |text: &str| {
            text.replace(HELPER, "#OrdinaryTree")
                .replace("#InstructionRightOperand", "#ConcreteOperand")
                .replace("let L =", "let LocalLeft =")
                .replace("let R =", "let LocalRight =")
                .replace("leftOperand: L", "leftOperand: LocalLeft")
                .replace("[L, R]", "[LocalLeft, LocalRight]")
        };
        let (owner, root) = (rename(GROUP), rename(&adapter));
        assert_eq!(
            serde_json::from_str::<Value>(&export(&owner, &root).expect("ordinary renamed export"))
                .expect("renamed whole JSON"),
            want
        );
        assert_eq!(decoded(p.native(&owner, &root, reverse)), want);
    }
}

#[test]
fn both_source_owners_reach_whole_native_and_same_source_full_go_policies() {
    let p = Probe::new();
    let baseline = p.policy(OPERAND, GROUP, false);
    for (group, operand, left, right) in [
        (GROUP.to_owned(), OPERAND.to_owned(), "left", "right"),
        (
            GROUP.replace("field: \"left\"", "field: \"right\""),
            OPERAND.to_owned(),
            "right",
            "right",
        ),
        (
            GROUP.to_owned(),
            OPERAND.replace("#InstructionRightField.field", "\"left\""),
            "left",
            "left",
        ),
    ] {
        let (r, adapter) = p.chain(&operand);
        assert_eq!(
            serde_json::from_str::<Value>(&r).expect("source-effect whole R JSON"),
            wrapper(right)
        );
        let data = export(&group, &adapter).expect("fresh whole Group");
        let want = expected(left, right);
        assert_eq!(
            serde_json::from_str::<Value>(&data).expect("source-effect whole Group JSON"),
            want
        );
        let placed = p.import(HELPER, &data);
        for reverse in [false, true] {
            assert_eq!(decoded(p.native(&group, &adapter, reverse)), want);
            let raw = p.policy(&operand, &group, reverse);
            let native = p.policy(&operand, &placed, reverse);
            assert_eq!(native, raw, "complete SAME-source raw/native policy");
            let mut predicted = baseline.clone();
            for (name, body) in want["cases"].as_object().expect("handwritten eight cases") {
                predicted["cases"][name] = body.clone();
            }
            predicted["cases"]["binaryRuntime"]["parts"][2] = wrapper(left);
            predicted["cases"]["binaryRuntime"]["parts"][4] = wrapper(right);
            assert_eq!(native, predicted, "all siblings preserved");
        }
    }
}

#[test]
fn exact_dependency_layout_resource_cli_failures_and_fresh_recovery() {
    let p = Probe::new();
    let (_, adapter) = p.chain(OPERAND);
    let root = "package bootstrap\n#InstructionOperandCases\n";
    let unknown = GROUP.replace("#InstructionRightOperand", "#AbsentOperand");
    let layout = "package bootstrap\n#InstructionOperandCases: {";
    let oversized = " ".repeat(65537);
    let raw = format!(
        "{ROOT}\n{}\n{}",
        OPERAND.replace("package bootstrap", ""),
        RIGHT.replace("package bootstrap", "")
    );
    for (owner, root, error) in [
        (
            GROUP,
            root,
            Error::Unknown("#InstructionRightOperand".into()),
        ),
        (
            unknown.as_str(),
            adapter.as_str(),
            Error::Unknown("#AbsentOperand".into()),
        ),
        (layout, adapter.as_str(), Error::Syntax("field label")),
        (
            oversized.as_str(),
            adapter.as_str(),
            Error::Limit("source bytes"),
        ),
        (
            GROUP,
            raw.as_str(),
            Error::Unsupported("literal projection scalar required"),
        ),
    ] {
        assert_eq!(export(owner, root), Err(error.clone()));
        let out = p.native(owner, root, false);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert_eq!(
            String::from_utf8(out.stderr)
                .expect("native error UTF-8")
                .trim(),
            error.to_string()
        );
        assert_eq!(
            decoded(p.native(GROUP, &adapter, false)),
            expected("left", "right")
        );
    }
    assert_eq!(
        export_with_limit(GROUP, &adapter, 0),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(&export(GROUP, &adapter).expect("library recovery"))
            .expect("recovered whole JSON"),
        expected("left", "right")
    );
    for inputs in [
        vec![],
        vec![("one-input.cue", GROUP)],
        vec![
            ("one.cue", GROUP),
            ("two.cue", &adapter),
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
    }
    let absent = p.dir.join("definitely-absent.cue");
    assert!(!absent.exists(), "intentional missing-read fixture");
    let out = p.run(
        &p.native,
        &[
            absent.to_str().expect("absent path UTF-8"),
            absent.to_str().expect("absent path UTF-8"),
        ],
        &[],
        false,
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .expect("missing-read diagnostic UTF-8")
            .contains("No such file or directory")
    );
    assert_eq!(
        decoded(p.native(GROUP, &adapter, true)),
        expected("left", "right")
    );
}
