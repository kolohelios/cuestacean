use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const OWNER: &str = include_str!("../../instruction-right-operand.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const ROOT: &str = include_str!("../../instruction-right-operand-export.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const RIGHT_ROOT: &str = include_str!("../../instruction-right-field-export.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const MODULE: &str = include_str!("../../instruction-module.cue");
const MODULE_SHAPE: &str = include_str!("../../instruction-module-schema.cue");
const RUNTIME: &str = include_str!("../../instruction-binary-runtime.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const HELPER: &str = "#InstructionRightOperand";
// Independent full-wrapper paths; aliases share binaryRuntime, not R instances.
const SITES: [&str; 9] = [
    "/cases/binaryRuntime/parts/4",
    "/cases/sum/args/1",
    "/cases/less/args/1",
    "/cases/lessEqual/args/1",
    "/cases/greater/args/1",
    "/cases/greaterEqual/args/1",
    "/cases/notEqual/args/1",
    "/cases/remainder/args/1",
    "/cases/unify/parts/1/args/1",
];
fn source(body: &str) -> String {
    format!("package bootstrap\n{body}\n")
}
fn wrapper(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":field}}})
}
fn value(a: &str, b: &str) -> Value {
    serde_json::from_str(&export(a, b).expect("native export")).expect("JSON")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn failed(out: Output) {
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "no partial JSON");
    assert!(!out.stderr.is_empty());
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file());
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
enum SourceMode {
    Current,
    MissingRight,
    MissingOperand,
    Legacy,
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    native: PathBuf,
    shell: PathBuf,
    binary: String,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-operand-{}-{}",
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
        Self {
            dir,
            cue: discover("cue"),
            native,
            shell: discover("sh"),
            binary: String::new(),
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
    fn native(&self, a: &str, b: &str, reverse: bool) -> Output {
        self.run(
            &self.native,
            &[],
            &[("unprivileged-a.cue", a), ("unrelated-b.cue", b)],
            reverse,
        )
    }
    fn import(&self, name: &str, data: &str) -> String {
        let label = format!("{name}:");
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
        String::from_utf8(out.stdout).expect("whole JSON import")
    }
    fn materialize(&self, right: &str, operand: &str, reverse: bool) -> String {
        // Native Right DATA is independent. Append the WHOLE JSON to the sole-root
        // companion, never a scalar extraction or a captured Go operand result.
        let data = export(right, RIGHT_ROOT).expect("existing once-reference profile");
        let adapter = format!("{ROOT}\n#InstructionRightField: {data}");
        let out = self.native(operand, &adapter, reverse);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let native = String::from_utf8(out.stdout).expect("native DATA");
        assert_eq!(native, export(operand, &adapter).expect("library export"));
        native
    }
    fn isolated(
        &self,
        right: Option<&str>,
        operand: Option<&str>,
        op: &str,
        reverse: bool,
    ) -> Output {
        let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
        if let Some(right) = right {
            inputs.push(("right.cue", right));
        }
        if let Some(operand) = operand {
            inputs.push(("operand.cue", operand));
        }
        self.run(&self.cue, &[op, "-e", HELPER], &inputs, reverse)
    }
    fn prepare_policy(&mut self) {
        let raw = self.run(
            &self.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary.cue", BINARY)],
            false,
        );
        let data = serde_json::to_string(&decoded(raw)).expect("Go setup JSON");
        self.binary = self.import("#InstructionBinaryTables", &data);
    }
    fn full(
        &self,
        bridge: &str,
        right: Option<&str>,
        operand: Option<&str>,
        context: &str,
        reverse: bool,
        mode: SourceMode,
    ) -> Output {
        // Missing R bypasses only R/Group; its SAME Right must still be complete.
        if matches!(mode, SourceMode::MissingOperand) {
            let gate = self.run(
                &self.cue,
                &["export", "-e", "#InstructionRightField"],
                &[
                    ("schema.cue", SCHEMA),
                    ("bridge.cue", BRIDGE),
                    ("right.cue", right.expect("missing-R mode retains Right")),
                ],
                reverse,
            );
            if !gate.status.success() {
                return gate;
            }
        }
        // Even explicit operand/Right/Group bypass controls retain independent Core118/Binary35.
        let core = self.run(
            &self.cue,
            &["export"],
            &[("shape.cue", SHAPE), ("core.cue", CORE)],
            reverse,
        );
        assert!(core.status.success(), "isolated Core setup");
        let binary = self.run(
            &self.cue,
            &["export", "-e", "#InstructionBinaryTables"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", bridge),
                ("binary-placed.cue", &self.binary),
            ],
            reverse,
        );
        assert!(binary.status.success(), "isolated Binary setup");
        if matches!(mode, SourceMode::Current) {
            let gate = self.run(
                &self.cue,
                &["export", "-e", "#InstructionBinaryRuntime"],
                &[
                    ("schema.cue", SCHEMA),
                    ("bridge.cue", bridge),
                    ("right.cue", right.expect("accepted Right")),
                    ("operand.cue", operand.expect("accepted R")),
                    ("group.cue", GROUP),
                    ("binary-placed.cue", &self.binary),
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
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", bridge),
            ("core.cue", CORE),
            ("binary-placed.cue", &self.binary),
            ("unary.cue", UNARY),
            ("text.cue", TEXT),
            ("first.cue", FIRST),
            ("step.cue", STEP),
        ];
        if let Some(right) = right {
            inputs.push(("right.cue", right));
        }
        if let Some(operand) = operand {
            inputs.push(("operand.cue", operand));
        }
        if !matches!(mode, SourceMode::Legacy) {
            inputs.push(("module.cue", MODULE));
            inputs.push(("group.cue", GROUP));
            inputs.push(("runtime.cue", RUNTIME));
        }
        inputs.push(("context.cue", context));
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
    fn policy(&self, right: &str, operand: Option<&str>, context: &str, reverse: bool) -> Output {
        let right_gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionRightField"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", right),
            ],
            reverse,
        );
        if !right_gate.status.success() {
            return right_gate;
        }
        let operand_gate = self.isolated(Some(right), operand, "export", reverse);
        if !operand_gate.status.success() {
            return operand_gate;
        }
        let group_gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionOperandCases"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", right),
                ("operand.cue", operand.expect("R gate requires provider")),
                ("group.cue", GROUP),
            ],
            reverse,
        );
        if !group_gate.status.success() {
            return group_gate;
        }
        // Discard all selected gate outputs; reload SAME unresolved owners.
        self.full(
            BRIDGE,
            Some(right),
            operand,
            context,
            reverse,
            SourceMode::Current,
        )
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

fn original_bridge() -> String {
    let module = MODULE
        .strip_prefix("package bootstrap\n\n#InstructionModule: {\n")
        .expect("original Module owner")
        .strip_suffix("}\n")
        .expect("owner end");
    let bridge = BRIDGE
        .replace("#InstructionModule: close({module: _})\n\n", "")
        .replace("\tmodule: #InstructionModule.module\n", module);
    // Source-only inverse, not a constructor seed. Keep old Right unresolved.
    let body = OWNER
        .trim()
        .strip_prefix("package bootstrap\n\n#InstructionRightOperand: ")
        .expect("raw owner");
    let (prefix, shape) = bridge
        .split_once("#InstructionBinaryRuntimeShape:")
        .expect("Runtime shape");
    let (_, suffix) = shape
        .split_once("// Executable recursion")
        .expect("root boundary");
    let runtime = RUNTIME
        .strip_prefix("package bootstrap\n\n#InstructionBinaryRuntime: {\n")
        .expect("raw Runtime")
        .strip_suffix("}\n")
        .expect("Runtime end");
    let (lets, concat_body) = runtime.split_once("\top: ").expect("original concat");
    let concat_body = concat_body
        .trim_end()
        .replace("\n\t\t", "\n\t\t\t")
        .replace("\n\t]", "\n\t\t]}");
    let restored = format!("{prefix}// Executable recursion{suffix}")
        .replace(
            "#InstructionLowering: #LoweringPolicy & {\n",
            &format!("#InstructionLowering: #LoweringPolicy & {{\n{lets}"),
        )
        .replace(
            "\t\tbinaryRuntime:    #InstructionBinaryRuntime",
            &format!("\t\tbinaryRuntime: {{op: {concat_body}"),
        );
    let mut in_cases = false;
    let restored = restored
        .lines()
        .map(|line| {
            if line == "\tcases: {" {
                in_cases = true;
            } else if line == "\t}" {
                in_cases = false;
            }
            if in_cases && line.starts_with("\t\t") && line.ends_with('_') {
                format!("{} _", line[..line.len() - 1].trim_end())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let (before, shape) = restored
        .split_once("#InstructionOperandCases: close({\n")
        .expect("Group shape");
    let (_, after) = shape
        .split_once("// Executable recursion")
        .expect("shape boundary");
    let left = GROUP
        .lines()
        .find(|line| line.starts_with("\tlet L = "))
        .expect("original L");
    let mut restored = format!("{before}// Executable recursion{after}")
        .replace("\t\t#InstructionOperandCases.cases\n", "")
        .replace("\tlet L = #InstructionOperandCases.leftOperand", left);
    let cases = GROUP
        .split_once("\tcases: {\n")
        .expect("original cases")
        .1
        .strip_suffix("\t}\n}\n")
        .expect("owner end");
    let names = [
        "sum",
        "less",
        "lessEqual",
        "greater",
        "greaterEqual",
        "notEqual",
        "remainder",
        "unify",
    ];
    for (index, name) in names.iter().enumerate() {
        let prefix = format!("\t\t{name}:");
        let start = cases.find(&prefix).expect("original body");
        let end = names.get(index + 1).map_or(cases.len(), |next| {
            cases.find(&format!("\t\t{next}:")).expect("next body")
        });
        let slot = restored
            .lines()
            .find(|line| line.starts_with(&prefix))
            .expect("required slot")
            .to_owned();
        restored = restored.replace(&format!("{slot}\n"), &cases[start..end]);
    }
    restored.replace("#InstructionRightOperand: close({op: string, value: close({op: string, value: close({op: string, field: string})})})\n", "")
        .replace("\tlet R = #InstructionRightOperand\n", &format!("\tlet R = {body}\n"))
}
fn context(mask: usize, field: &str, complete: bool) -> String {
    let leaf = if complete {
        wrapper(field)
    } else {
        json!({"value":{"value":{"field":field}}})
    };
    let mut cases = Vec::new();
    for (i, name) in [
        "binaryRuntime",
        "sum",
        "less",
        "lessEqual",
        "greater",
        "greaterEqual",
        "notEqual",
        "remainder",
        "unify",
    ]
    .iter()
    .enumerate()
    {
        if mask & (1 << i) == 0 {
            continue;
        }
        let body = match i {
            0 => format!("parts:[_,_,_,_,{leaf},_]"),
            8 => format!("parts:[_,{{args:[_,{leaf}]}},_]"),
            _ => format!("args:[_,{leaf}]"),
        };
        cases.push(format!("{name}:{{{body}}}"));
    }
    source(&format!(
        "#InstructionLowering & {{cases:{{{}}}}}",
        cases.join(",")
    ))
}

#[test]
fn complete_independent_native_wrapper_two_stages_and_both_raw_source_effects() {
    let mut p = Probe::new();
    p.prepare_policy();
    for reverse in [false, true] {
        for _ in 0..2 {
            let data = p.materialize(RIGHT, OWNER, reverse);
            assert_eq!(data.len(), 120);
            assert_eq!(
                serde_json::from_str::<Value>(&data).expect("JSON"),
                wrapper("right")
            );
            assert_eq!(
                decoded(p.run(
                    &p.cue,
                    &["export"],
                    &[
                        ("right.cue", RIGHT),
                        ("owner.cue", OWNER),
                        ("root.cue", ROOT)
                    ],
                    reverse
                )),
                wrapper("right")
            );
        }
        let rename = |text: &str| {
            text.replace("#InstructionRightOperand", "#UnprivilegedOperand")
                .replace("#InstructionRightField", "#IndependentInput")
        };
        let owner = rename(OWNER);
        let right = export(RIGHT, RIGHT_ROOT).expect("independent native Right");
        assert_eq!(right.len(), 23);
        let adapter = rename(&format!("{ROOT}\n#InstructionRightField: {right}"));
        let missing = source("#UnprivilegedOperand");
        assert!(matches!(export(&owner, &missing), Err(Error::Unknown(_))));
        failed(p.native(&owner, &missing, reverse));
        assert_eq!(value(&owner, &adapter), wrapper("right"));
        assert_eq!(
            decoded(p.native(&owner, &adapter, reverse)),
            wrapper("right")
        );
        // Marked Right + projection is NOT a widened native/default/reuse class.
        assert!(matches!(
            export(&format!("{RIGHT}\n#InstructionRightOperand\n"), OWNER),
            Err(Error::Unsupported("scalar projection document profile"))
        ));
        for (right, operand, want) in [
            (RIGHT.to_owned(), OWNER.to_owned(), wrapper("right")),
            (
                RIGHT.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
                OWNER.to_owned(),
                wrapper("left"),
            ),
            (
                RIGHT.to_owned(),
                OWNER.replace("op: \"try\"", "op: \"group\""),
                json!({"op":"group","value":{"op":"group","value":{"op":"lower","field":"right"}}}),
            ),
        ] {
            let data = p.materialize(&right, &operand, reverse);
            assert_eq!(
                serde_json::from_str::<Value>(&data).expect("native tree"),
                want
            );
            let placed = p.import(HELPER, &data);
            let raw = decoded(p.policy(&right, Some(&operand), SELECTOR, reverse));
            assert_eq!(
                decoded(p.policy(&right, Some(&placed), SELECTOR, reverse)),
                raw
            );
            for site in SITES {
                assert_eq!(raw.pointer(site).expect("whole wrapper"), &want);
            }
        }
    }
    // Complete tiny emitted Rust/source variants and REAL typed129-byte rejection
    // are independently Nix-owned. No fake compiler-provider ABI in this test crate.
}

#[test]
fn raw_nine_site_contexts_both_priorities_orders_old_policies_and_selected_conflicts() {
    let mut p = Probe::new();
    p.prepare_policy();
    let old = original_bridge();
    for reverse in [false, true] {
        for right in [
            RIGHT.to_owned(),
            RIGHT.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
        ] {
            let default = if right == RIGHT { "right" } else { "left" };
            let forced = if default == "right" { "left" } else { "right" };
            let baseline = decoded(p.full(
                &old,
                Some(&right),
                None,
                SELECTOR,
                reverse,
                SourceMode::Legacy,
            ));
            for mask in (0..9).map(|i| 1 << i).chain([0, 511, 341, 170]) {
                let ctx = context(mask, forced, false);
                let mut want = baseline.clone();
                for (i, site) in SITES.iter().enumerate() {
                    *want.pointer_mut(site).expect("independent wrapper path") =
                        wrapper(if mask & (1 << i) == 0 {
                            default
                        } else {
                            forced
                        });
                }
                assert_eq!(decoded(p.policy(&right, Some(OWNER), &ctx, reverse)), want);
                assert_eq!(
                    decoded(p.full(&old, Some(&right), None, &ctx, reverse, SourceMode::Legacy)),
                    want
                );
            }
        }
        for ctx in [SELECTOR, TWICE, REVERSE] {
            assert_eq!(
                decoded(p.policy(RIGHT, Some(OWNER), ctx, reverse)),
                decoded(p.full(&old, Some(RIGHT), None, ctx, reverse, SourceMode::Legacy))
            );
        }
        let selected = p.import(HELPER, &p.materialize(RIGHT, OWNER, reverse));
        assert_eq!(
            decoded(p.policy(RIGHT, Some(&selected), SELECTOR, reverse)),
            decoded(p.policy(RIGHT, Some(OWNER), SELECTOR, reverse))
        );
        let conflict = p.policy(RIGHT, Some(&selected), TWICE, reverse);
        assert!(String::from_utf8_lossy(&conflict.stderr).contains("conflicting values"));
        assert!(!conflict.status.success() && conflict.stdout.is_empty());
    }
}

#[test]
fn isolated_four_leaf_omissions_types_real_backfill_and_consumer_fail_stop() {
    let mut p = Probe::new();
    p.prepare_policy();
    let all = context(511, "left", true);
    for reverse in [false, true] {
        // Full Go really backfills the ENTIRE absent body from complete use contexts.
        // Explicitly bypass BOTH dependent gates, NEVER an accepted pipeline.
        let bypass = decoded(p.full(
            BRIDGE,
            Some(RIGHT),
            None,
            &all,
            reverse,
            SourceMode::MissingOperand,
        ));
        assert_eq!(
            bypass,
            decoded(p.full(
                BRIDGE,
                None,
                Some(OWNER),
                &all,
                reverse,
                SourceMode::MissingRight
            ))
        );
        for invalid_right in [
            source("#InstructionRightField:{}"),
            source("#InstructionRightField:{field:42}"),
        ] {
            let out = p.full(
                BRIDGE,
                Some(&invalid_right),
                None,
                &all,
                reverse,
                SourceMode::MissingOperand,
            );
            assert_eq!(out.status.code(), Some(1));
            assert!(out.stdout.is_empty() && !out.stderr.is_empty());
        }
        for site in SITES {
            assert_eq!(
                bypass.pointer(site).expect("backfilled whole tree"),
                &wrapper("left")
            );
        }
        let mut missing = vec![None, Some(source("#InstructionRightOperand:{}"))];
        for pointer in ["/op", "/value/op", "/value/value/op", "/value/value/field"] {
            let mut tree = wrapper("right");
            let (parent, key) = pointer.rsplit_once('/').expect("leaf path");
            tree.pointer_mut(parent)
                .expect("parent")
                .as_object_mut()
                .expect("object")
                .remove(key)
                .expect("required leaf");
            missing.push(Some(source(&format!("{HELPER}:{tree}"))));
        }
        for operand in &missing {
            assert!(
                p.isolated(Some(RIGHT), operand.as_deref(), "eval", reverse)
                    .status
                    .success()
            );
            let out = p.isolated(Some(RIGHT), operand.as_deref(), "export", reverse);
            assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
            let stopped = p.policy(RIGHT, operand.as_deref(), &all, reverse);
            assert!(!stopped.status.success() && stopped.stdout.is_empty());
        }
        for pointer in ["/op", "/value/op", "/value/value/op", "/value/value/field"] {
            for replacement in [json!(7), json!("unknownInstructionOpcode")] {
                let mut tree = wrapper("right");
                *tree.pointer_mut(pointer).expect("leaf") = replacement.clone();
                let operand = source(&format!("{HELPER}:{tree}"));
                let isolated = p.isolated(Some(RIGHT), Some(&operand), "export", reverse);
                if replacement.is_number() {
                    assert!(!isolated.status.success());
                } else if pointer != "/value/value/field" {
                    assert!(isolated.status.success(), "helper supplies types only");
                    assert!(
                        !p.policy(RIGHT, Some(&operand), SELECTOR, reverse)
                            .status
                            .success()
                    );
                }
            }
        }
        for tree in [
            json!({"op":"try","value":[]}),
            json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":"right","extra":1}}}),
        ] {
            assert!(
                !p.isolated(
                    Some(RIGHT),
                    Some(&source(&format!("{HELPER}:{tree}"))),
                    "eval",
                    reverse
                )
                .status
                .success()
            );
        }
        let mut tree = wrapper(&"x".repeat(129));
        let operand = source(&format!("{HELPER}:{tree}"));
        assert!(
            p.policy(RIGHT, Some(&operand), SELECTOR, reverse)
                .status
                .success(),
            "Go accepted typed-limit input"
        );
        tree["value"]["value"]["field"] = json!("");
        let operand = source(&format!("{HELPER}:{tree}"));
        assert!(
            p.isolated(Some(RIGHT), Some(&operand), "export", reverse)
                .status
                .success()
        );
        assert!(
            !p.policy(RIGHT, Some(&operand), SELECTOR, reverse)
                .status
                .success()
        );
    }
    // Normal/if/|| shell callers cannot exploit set-e suppression or full backfill.
    let script = format!(
        r#"
consume() {{
  '{cue}' export schema.cue bridge.cue right.cue -e '#InstructionRightField' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue $2 -e '#InstructionRightOperand' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue $2 group.cue -e '#InstructionOperandCases' >/dev/null || return
  '{cue}' export shape.cue core.cue >/dev/null || return
  '{cue}' export schema.cue bridge.cue binary-placed.cue -e '#InstructionBinaryTables' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue $2 group.cue binary-placed.cue runtime.cue -e '#InstructionBinaryRuntime' >/dev/null || return
  for helper in '#InstructionModuleText' '#InstructionModuleFirst' '#InstructionModuleStep' '#InstructionModule'; do
    '{cue}' export module-shape.cue module.cue text.cue first.cue step.cue -e "$helper" >/dev/null || return
  done
  : > full-export-started
  '{cue}' export schema.cue bridge.cue core.cue binary-placed.cue unary.cue module.cue text.cue first.cue step.cue right.cue $2 group.cue runtime.cue context.cue
}}
case "$1" in
 normal) consume "$@" ;;
 conditional) if consume "$@"; then exit 99; else exit 17; fi ;;
 disjunction) consume "$@" || exit 17 ;;
esac
"#,
        cue = p.cue.display()
    );
    fs::write(p.dir.join("consume.sh"), script).expect("actual fail-stop pipeline");
    for operand in ["", "operand.cue"] {
        decoded(p.policy(RIGHT, Some(OWNER), &all, false));
        fs::write(
            p.dir.join("operand.cue"),
            source("#InstructionRightOperand:{}"),
        )
        .expect("missing body");
        for mode in ["normal", "conditional", "disjunction"] {
            let out = p.run(&p.shell, &["consume.sh", mode, operand], &[], false);
            assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
            assert!(!p.dir.join("full-export-started").exists());
        }
    }
}
