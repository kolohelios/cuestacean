use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const TABLE: &str = include_str!("../../instruction-binary-tables.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const MODULE: &str = include_str!("../../instruction-module.cue");
const MODULE_SHAPE: &str = include_str!("../../instruction-module-schema.cue");
const RUNTIME: &str = include_str!("../../instruction-binary-runtime.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const CORE_SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const POLICY: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const HELPER: &str = "#InstructionBinaryTables";
// Independently authored contract, not parsed from CUE or captured JSON.
const ROWS: [(&str, &str); 11] = [
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
];
fn expected(rows: &[(&str, &str)], prefix: &str, suffix: &str) -> Value {
    let mut aliases = json!({});
    let mut cases = json!({});
    for (key, method) in rows {
        aliases[key] = json!("binaryRuntime");
        cases[key] = json!({"op":"text","value":format!("{prefix}{method}{suffix}")});
    }
    json!({"aliases":aliases,"select":{"op":"select","field":"op","cases":cases}})
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
    assert!(meta.is_file(), "regular mandatory tool");
    fs::File::open(path).expect("readable mandatory tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    jq: PathBuf,
    native: PathBuf,
    full_exports: Cell<usize>,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-binary-{}-{}",
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
        Self {
            dir,
            cue: discover("cue"),
            jq: discover("jq"),
            native,
            full_exports: Cell::new(0),
        }
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, source) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, source).expect("explicit input");
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
            &[("package.cue", PACKAGE), ("table.cue", source)],
            reverse,
        )
    }
    fn oracle(&self, source: &str, reverse: bool) -> String {
        let out = self.run(
            &self.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("table.cue", source)],
            reverse,
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("Go table JSON")
    }
    fn import(&self, data: &str) -> String {
        let out = self.run(
            &self.cue,
            &[
                "import",
                "-p",
                "bootstrap",
                "-l",
                "#InstructionBinaryTables:",
                "--outfile",
                "-",
            ],
            &[("data.json", data)],
            false,
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("whole JSON Go import")
    }
    fn isolated(&self, provider: Option<&str>, op: &str, reverse: bool) -> Output {
        let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
        if let Some(provider) = provider {
            inputs.push(("placed.cue", provider));
        }
        // Deliberately NO full-policy selector, context, or unrelated providers.
        self.run(&self.cue, &[op, "-e", HELPER], &inputs, reverse)
    }
    fn full_unchecked(
        &self,
        provider: &str,
        context: &str,
        reverse: bool,
        runtime_gate: bool,
    ) -> Output {
        let core_gate = self.run(
            &self.cue,
            &["export"],
            &[("core-shape.cue", CORE_SHAPE), ("core.cue", CORE)],
            reverse,
        );
        if !core_gate.status.success() {
            return core_gate;
        }
        // Binary's deliberate bypass controls must still validate the right owner.
        let right_gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionRightField"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", RIGHT),
            ],
            reverse,
        );
        if !right_gate.status.success() {
            return right_gate;
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
        if runtime_gate {
            let gate = self.run(
                &self.cue,
                &["export", "-e", "#InstructionBinaryRuntime"],
                &[
                    ("schema.cue", SCHEMA),
                    ("bridge.cue", BRIDGE),
                    ("right.cue", RIGHT),
                    ("operand.cue", OPERAND),
                    ("group.cue", GROUP),
                    ("placed.cue", provider),
                    ("runtime.cue", RUNTIME),
                ],
                reverse,
            );
            if !gate.status.success() {
                return gate;
            }
        }
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
            if !checked.status.success() {
                return checked;
            }
        }
        self.full_exports.set(self.full_exports.get() + 1);
        self.run(
            &self.cue,
            &["export"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("core.cue", CORE),
                ("placed.cue", provider),
                ("unary.cue", UNARY),
                ("text.cue", TEXT),
                ("module.cue", MODULE),
                ("first.cue", FIRST),
                ("step.cue", STEP),
                ("right.cue", RIGHT),
                ("operand.cue", OPERAND),
                ("group.cue", GROUP),
                ("runtime.cue", RUNTIME),
                ("context.cue", context),
            ],
            reverse,
        )
    }
    fn policy(&self, provider: Option<&str>, context: &str, reverse: bool) -> Output {
        let gate = self.isolated(provider, "export", reverse);
        if !gate.status.success() {
            return gate;
        }
        self.full_unchecked(
            provider.expect("gate requires provider"),
            context,
            reverse,
            true,
        )
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn complete_eleven_row_tree_raw_orders_repeats_cli_go_and_binder_renames() {
    let probe = Probe::new();
    let want = expected(&ROWS, "runtime::", "");
    assert_eq!(TABLE.matches("for ").count(), 2);
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (TABLE, PACKAGE)
        } else {
            (PACKAGE, TABLE)
        };
        for _ in 0..2 {
            let native = export(a, b).expect("native raw2");
            assert_eq!(native.len(), 1429);
            assert_eq!(serde_json::from_str::<Value>(&native).expect("JSON"), want);
            assert_eq!(decoded(probe.native(TABLE, reverse)), want);
            let go = probe.oracle(TABLE, reverse);
            assert_eq!(serde_json::from_str::<Value>(&go).expect("Go JSON"), want);
            let sorted = probe.run(&probe.jq, &["-S", "."], &[("go.json", &go)], false);
            assert!(sorted.status.success());
            assert_eq!(sorted.stdout, native.as_bytes());
        }
        let renamed = TABLE
            .replace("Binary", "UnprivilegedTable")
            .replace("name,", "entry,")
            .replace("(name)", "(entry)")
            .replace("method", "target");
        assert_eq!(decoded(probe.native(&renamed, reverse)), want);
        assert_eq!(
            serde_json::from_str::<Value>(&probe.oracle(&renamed, reverse)).expect("Go"),
            want
        );
    }
}

#[test]
fn arbitrary_keys_methods_affixes_and_actual_source_swaps_predict_complete_data() {
    let probe = Probe::new();
    let arbitrary = r#"package bootstrap
{
    let Methods = {"snow 雪": "alpha", "punct-key": "βeta", "quoted\"key": ""}
    aliases: {for key, _ in Methods {"\(key)": "binaryRuntime"}}
    select: {op: "select", field: "op", cases: {for key, target in Methods {
        "\(key)": {op: "text", value: "prefix\n\(target)\t_tail"}
    }}}
}"#;
    let arbitrary_want = expected(
        &[
            ("snow 雪", "alpha"),
            ("punct-key", "βeta"),
            ("quoted\"key", ""),
        ],
        "prefix\n",
        "\t_tail",
    );
    let mut swapped = ROWS;
    swapped[0].0 = "difference";
    swapped[1].0 = "index";
    let key_swap = TABLE
        .replace("index:", "temporaryKey:")
        .replace("difference:", "index:")
        .replace("temporaryKey:", "difference:");
    let mut changed = ROWS;
    changed[0].1 = "changed_index";
    let method = TABLE.replace("\"index\"", "\"changed_index\"");
    let affix = TABLE.replace("runtime::\\(method)", "changed::\\(method)_suffix");
    for (source, want) in [
        (arbitrary, arbitrary_want),
        (key_swap.as_str(), expected(&swapped, "runtime::", "")),
        (method.as_str(), expected(&changed, "runtime::", "")),
        (affix.as_str(), expected(&ROWS, "changed::", "_suffix")),
    ] {
        for reverse in [false, true] {
            assert_eq!(decoded(probe.native(source, reverse)), want);
            assert_eq!(
                serde_json::from_str::<Value>(&probe.oracle(source, reverse)).expect("Go"),
                want
            );
            if source != arbitrary {
                let data = decoded(probe.native(source, reverse)).to_string();
                let placed = probe.import(&data);
                let full = decoded(probe.policy(Some(&placed), POLICY, reverse));
                assert_eq!(full["cases"]["binaryRuntime"]["parts"][0], want["select"]);
                for (key, _) in ROWS {
                    assert_eq!(full["aliases"][key], want["aliases"][key]);
                }
            }
        }
    }
    for wrong in [
        TABLE.replace("\\(method)", "\\(outer)"),
        TABLE.replace("\"record_index\"", "true"),
    ] {
        assert!(matches!(
            export(PACKAGE, &wrong),
            Err(Error::Unsupported(_))
        ));
        failed(&probe.native(&wrong, false));
    }
    assert_eq!(
        decoded(probe.native(TABLE, false)),
        expected(&ROWS, "runtime::", "")
    );
    // Complete independent tiny Rust/source effects and typed compiler exit2 controls
    // run in the Nix verifier; no compiler-provider environment ABI is introduced.
}

#[test]
fn raw_default_free_go_import_and_native_import_preserve_scoped_r_and_cursor_contexts() {
    let probe = Probe::new();
    let native = probe.import(&export(PACKAGE, TABLE).expect("native raw2"));
    let raw = probe.import(&probe.oracle(TABLE, false));
    let mixed_last = "package bootstrap\n#InstructionLowering & {module:[_,{op:\"lastRule\"},{body:[_,_,_,_,{op:\"nextRule\"}]},_]}\n";
    let mixed_first = "package bootstrap\n#InstructionLowering & {module:[_,{op:\"firstRule\"},{body:[_,_,_,_,{op:\"previousRule\"}]},_]}\n";
    for (ctx, right, first, step) in [
        (POLICY, "right", "firstRule", "nextRule"),
        (TWICE, "left", "firstRule", "nextRule"),
        (REVERSE, "right", "lastRule", "previousRule"),
        (mixed_last, "right", "lastRule", "nextRule"),
        (mixed_first, "right", "firstRule", "previousRule"),
    ] {
        let mut forward = None;
        for reverse in [false, true] {
            let full = decoded(probe.policy(Some(&native), ctx, reverse));
            assert_eq!(full, decoded(probe.policy(Some(&raw), ctx, reverse)));
            assert_eq!(
                full["cases"]["sum"]["args"][1]["value"]["value"]["field"],
                right
            );
            assert_eq!(
                full["cases"]["binaryRuntime"]["parts"][4]["value"]["value"]["field"],
                "right"
            );
            for key in [
                "less",
                "lessEqual",
                "greater",
                "greaterEqual",
                "notEqual",
                "remainder",
            ] {
                assert_eq!(
                    full["cases"][key]["args"][1]["value"]["value"]["field"],
                    "right"
                );
            }
            assert_eq!(full["module"][1]["op"], first);
            assert_eq!(full["module"][2]["body"][4]["op"], step);
            assert_eq!(
                full["aliases"]["negate"], "unaryRuntime",
                "closed binary embedding admits unary"
            );
            if let Some(want) = &forward {
                assert_eq!(&full, want);
            } else {
                forward = Some(full);
            }
        }
    }
}

// Explicit required-slot paths, not discovered by traversing the native/Go answer.
fn slots() -> Vec<Vec<&'static str>> {
    let mut paths = vec![vec!["select", "op"], vec!["select", "field"]];
    for key in [
        "index",
        "difference",
        "product",
        "divide",
        "modulo",
        "quotient",
        "recordIndex",
        "listIndex",
        "logicalAnd",
        "logicalOr",
        "equal",
    ] {
        paths.push(vec!["aliases", key]);
        paths.push(vec!["select", "cases", key, "op"]);
        paths.push(vec!["select", "cases", key, "value"]);
    }
    paths
}
fn parent<'a>(value: &'a mut Value, path: &[&str]) -> &'a mut Value {
    path[..path.len() - 1]
        .iter()
        .fold(value, |node, key| &mut node[*key])
}

#[test]
fn all_thirty_five_missing_slots_are_isolated_incomplete_and_consumers_fail_stop() {
    let probe = Probe::new();
    let want = expected(&ROWS, "runtime::", "");
    assert_eq!(slots().len(), 35);
    for reverse in [false, true] {
        assert!(probe.isolated(None, "eval", reverse).status.success());
        failed(&probe.policy(None, POLICY, reverse));
        assert_eq!(probe.full_exports.get(), 0);
        for path in slots() {
            let mut missing = want.clone();
            parent(&mut missing, &path)
                .as_object_mut()
                .expect("parent")
                .remove(path[path.len() - 1]);
            let placed = probe.import(&missing.to_string());
            let eval = probe.isolated(Some(&placed), "eval", reverse);
            assert!(
                eval.status.success(),
                "{path:?}: {}",
                String::from_utf8_lossy(&eval.stderr)
            );
            let incomplete = probe.isolated(Some(&placed), "export", reverse);
            failed(&incomplete);
            assert!(String::from_utf8_lossy(&incomplete.stderr).contains("incomplete value"));
            let stopped = probe.policy(Some(&placed), POLICY, reverse);
            failed(&stopped);
            assert_eq!(stopped.stderr, incomplete.stderr);
            assert_eq!(
                probe.full_exports.get(),
                0,
                "must not attempt full-policy export"
            );
        }
    }
}

#[test]
fn bypass_counterfactual_backfills_select_op_but_real_consumer_stops_first() {
    let probe = Probe::new();
    let mut missing = expected(&ROWS, "runtime::", "");
    missing["select"]
        .as_object_mut()
        .expect("select")
        .remove("op");
    let placed = probe.import(&missing.to_string());
    failed(&probe.policy(Some(&placed), POLICY, false));
    assert_eq!(probe.full_exports.get(), 0);
    let bypass = decoded(probe.full_unchecked(&placed, POLICY, false, false));
    assert_eq!(bypass["cases"]["binaryRuntime"]["parts"][0]["op"], "select");
    assert_eq!(
        probe.full_exports.get(),
        1,
        "only explicit counterfactual bypasses gate"
    );
}

#[test]
fn all_wrong_types_and_five_extra_levels_reject_in_isolated_schema_before_consumption() {
    let probe = Probe::new();
    let want = expected(&ROWS, "runtime::", "");
    for path in slots() {
        let mut invalid = want.clone();
        parent(&mut invalid, &path)[path[path.len() - 1]] = json!(true);
        let placed = probe.import(&invalid.to_string());
        for reverse in [false, true] {
            let out = probe.isolated(Some(&placed), "eval", reverse);
            failed(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains("conflicting values"));
            failed(&probe.policy(Some(&placed), POLICY, reverse));
        }
    }
    for path in [
        vec![],
        vec!["aliases"],
        vec!["select"],
        vec!["select", "cases"],
        vec!["select", "cases", "index"],
    ] {
        let mut invalid = want.clone();
        let target = path.iter().fold(&mut invalid, |node, key| &mut node[*key]);
        target["unexpected"] = json!("forbidden");
        let placed = probe.import(&invalid.to_string());
        for reverse in [false, true] {
            let out = probe.isolated(Some(&placed), "eval", reverse);
            failed(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains("field not allowed"));
            failed(&probe.policy(Some(&placed), POLICY, reverse));
        }
    }
    assert_eq!(probe.full_exports.get(), 0);
}

#[test]
fn go_schema_negatives_are_distinct_from_go_accepted_typed_boundary_controls() {
    let probe = Probe::new();
    let want = expected(&ROWS, "runtime::", "");
    for path in [
        vec!["select", "op"],
        vec!["select", "field"],
        vec!["select", "cases", "index", "op"],
    ] {
        let mut invalid = want.clone();
        parent(&mut invalid, &path)[path[path.len() - 1]] = json!(if path == ["select", "field"] {
            ""
        } else {
            "unknownBinaryOp"
        });
        let placed = probe.import(&invalid.to_string());
        assert!(
            probe
                .isolated(Some(&placed), "export", false)
                .status
                .success()
        );
        failed(&probe.policy(Some(&placed), POLICY, false));
    }
    for alias in [true, false] {
        let mut invalid = want.clone();
        if alias {
            invalid["aliases"]["index"] = json!("noSuchCase");
        } else {
            invalid["select"]["field"] = json!("x".repeat(129));
        }
        let placed = probe.import(&invalid.to_string());
        let full = decoded(probe.policy(Some(&placed), POLICY, false));
        if alias {
            assert_eq!(full["aliases"]["index"], "noSuchCase");
        } else {
            assert_eq!(
                full["cases"]["binaryRuntime"]["parts"][0]["field"]
                    .as_str()
                    .expect("selector")
                    .len(),
                129
            );
        }
    }
    // Nix invokes the real compiler on these Go-accepted controls: exit2, empty
    // stdout and canonical-alias/128-byte-selector diagnostics, not setup errors.
}
