use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const FIRST: &str = include_str!("../../instruction-module-first.cue");
const FIRST_ROOT: &str = include_str!("../../instruction-module-first-export.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const STEP_ROOT: &str = include_str!("../../instruction-module-step-export.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const CORE_SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const FIRST_NAME: &str = "#InstructionModuleFirst";
const STEP_NAME: &str = "#InstructionModuleStep";
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file(), "regular mandatory tool");
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    jq: PathBuf,
    native: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-controls-{}-{}",
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
        let jq = discover("jq");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        Self {
            dir,
            cue,
            jq,
            native,
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
    fn native(&self, owner: &str, root: &str, reverse: bool) -> Output {
        self.run(
            &self.native,
            &[],
            &[("renamed-owner.cue", owner), ("other-root.cue", root)],
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
        String::from_utf8(out.stdout).expect("Go import")
    }
    fn binary_provider(&self, reverse: bool) -> String {
        let raw = self.run(
            &self.cue,
            &["export"],
            &[("binary-package.cue", PACKAGE), ("binary-raw.cue", BINARY)],
            reverse,
        );
        assert!(
            raw.status.success(),
            "binary raw oracle setup: {}",
            String::from_utf8_lossy(&raw.stderr)
        );
        let data = String::from_utf8(raw.stdout).expect("binary Go JSON");
        let imported = self.run(
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
            &[("binary.json", &data)],
            false,
        );
        assert!(
            imported.status.success(),
            "binary import setup: {}",
            String::from_utf8_lossy(&imported.stderr)
        );
        let placed = String::from_utf8(imported.stdout).expect("binary Go helper");
        let gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionBinaryTables"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("binary-placed.cue", &placed),
            ],
            reverse,
        );
        assert!(
            gate.status.success(),
            "isolated binary helper setup: {}",
            String::from_utf8_lossy(&gate.stderr)
        );
        placed
    }
    fn policy(
        &self,
        first: Option<&str>,
        step: Option<&str>,
        context: &str,
        reverse: bool,
        op: &str,
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
        let binary = self.binary_provider(reverse);
        let gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionRightField"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("right.cue", RIGHT),
            ],
            reverse,
        );
        if !gate.status.success() {
            return gate;
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
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("core.cue", CORE),
            ("text.cue", TEXT),
            ("unary.cue", UNARY),
            ("binary-placed.cue", &binary),
            ("right.cue", RIGHT),
            ("operand.cue", OPERAND),
            ("group.cue", GROUP),
        ];
        if let Some(first) = first {
            inputs.push(("first.cue", first));
        }
        if let Some(step) = step {
            inputs.push(("step.cue", step));
        }
        inputs.push(("context.cue", context)); // Both unresolved raw owners precede context.
        self.run(&self.cue, &[op], &inputs, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
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
fn prefer_other(owner: &str, preferred: &str, other: &str) -> String {
    let source = owner
        .replace(&format!("*\"{preferred}\""), &format!("\"{preferred}\""))
        .replace(&format!("\"{other}\""), &format!("*\"{other}\""));
    assert_ne!(source, owner);
    source
}
fn context(first: &str, step: &str) -> String {
    format!(
        "package bootstrap\n#InstructionLowering & {{module:[_,{{op:\"{first}\"}},{{body:[_,_,_,_,{{op:\"{step}\"}}]}},_]}}\n"
    )
}

#[test]
fn complete_two_artifacts_orders_repeats_renames_and_source_only_priority_literal_effects() {
    let probe = Probe::new();
    for (owner, root, name, preferred, other) in [
        (FIRST, FIRST_ROOT, FIRST_NAME, "firstRule", "lastRule"),
        (STEP, STEP_ROOT, STEP_NAME, "nextRule", "previousRule"),
    ] {
        let want = json!({"op":preferred}); // Complete independent DATA, not artifact capture.
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (root, owner)
            } else {
                (owner, root)
            };
            for _ in 0..2 {
                let native = export(a, b).expect("native raw two-file input");
                assert_eq!(serde_json::from_str::<Value>(&native).expect("JSON"), want);
                assert_eq!(decoded(probe.native(owner, root, reverse)), want);
                let raw = probe.run(
                    &probe.cue,
                    &["export"],
                    &[("owner.cue", owner), ("root.cue", root)],
                    reverse,
                );
                assert_eq!(
                    serde_json::from_slice::<Value>(&raw.stdout).expect("oracle JSON"),
                    want
                );
                assert!(raw.status.success());
                let sorted = probe.run(
                    &probe.jq,
                    &["-S", "."],
                    &[("raw.json", std::str::from_utf8(&raw.stdout).expect("UTF-8"))],
                    false,
                );
                assert!(sorted.status.success());
                assert_eq!(sorted.stdout, native.as_bytes());
            }
            let renamed = owner
                .replace(name, "#Unprivileged")
                .replace("op:", "renamed:");
            assert_eq!(
                decoded(probe.native(&renamed, &root.replace(name, "#Unprivileged"), reverse)),
                json!({"renamed":preferred})
            );
            assert_eq!(
                decoded(probe.native(&prefer_other(owner, preferred, other), root, reverse)),
                json!({"op":other})
            );
            let changed = owner.replace(&format!("\"{preferred}\""), "\"ordinaryLiteral\"");
            assert_ne!(changed, owner);
            assert_eq!(
                decoded(probe.native(&changed, root, reverse)),
                json!({"op":"ordinaryLiteral"})
            );
        }
        assert_eq!(
            export(owner, &root.replace(name, "#Missing")),
            Err(Error::Unknown("#Missing".into()))
        );
    }
    assert!(matches!(
        export(
            &format!("{FIRST}\n{FIRST_NAME}\n"),
            &format!("{STEP}\n{STEP_NAME}\n")
        ),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn four_independent_raw_cursor_states_and_materialized_canonical_are_not_late_refinement_inputs() {
    let probe = Probe::new();
    let first = probe.import(FIRST_NAME, &export(FIRST, FIRST_ROOT).expect("first DATA"));
    let step = probe.import(STEP_NAME, &export(STEP, STEP_ROOT).expect("step DATA"));
    for (endpoint, advance) in [
        ("firstRule", "nextRule"),
        ("lastRule", "previousRule"),
        ("lastRule", "nextRule"),
        ("firstRule", "previousRule"),
    ] {
        let ctx = context(endpoint, advance);
        // Mutate only preference priority in the raw owners, then construct independently.
        let first_owner = if endpoint == "firstRule" {
            FIRST.to_owned()
        } else {
            prefer_other(FIRST, "firstRule", "lastRule")
        };
        let step_owner = if advance == "nextRule" {
            STEP.to_owned()
        } else {
            prefer_other(STEP, "nextRule", "previousRule")
        };
        let selected_first = probe.import(
            FIRST_NAME,
            &export(&first_owner, FIRST_ROOT).expect("mutated first DATA"),
        );
        let selected_step = probe.import(
            STEP_NAME,
            &export(&step_owner, STEP_ROOT).expect("mutated step DATA"),
        );
        let mut forward = None;
        for reverse in [false, true] {
            let raw = decoded(probe.policy(Some(FIRST), Some(STEP), &ctx, reverse, "export"));
            let selected = decoded(probe.policy(
                Some(&selected_first),
                Some(&selected_step),
                SELECTOR,
                reverse,
                "export",
            ));
            assert_eq!(
                raw, selected,
                "complete policy after explicit source-only selection"
            );
            assert_eq!(raw["module"][1], json!({"op":endpoint}));
            assert_eq!(raw["module"][2]["body"][4], json!({"op":advance}));
            assert_eq!(
                raw["cases"]["sum"]["args"][1]["value"]["value"]["field"],
                "right"
            );
            if let Some(want) = &forward {
                assert_eq!(&raw, want);
            } else {
                forward = Some(raw);
            }
        }
    }
    for ctx in [SELECTOR, TWICE] {
        for reverse in [false, true] {
            let raw = decoded(probe.policy(Some(FIRST), Some(STEP), ctx, reverse, "export"));
            let selected = decoded(probe.policy(Some(&first), Some(&step), ctx, reverse, "export"));
            assert_eq!(raw, selected);
            assert_eq!(
                raw["cases"]["sum"]["args"][1]["value"]["value"]["field"],
                if ctx == TWICE { "left" } else { "right" }
            );
        }
    }
    for reverse in [false, true] {
        let raw = decoded(probe.policy(Some(FIRST), Some(STEP), REVERSE, reverse, "export"));
        assert_eq!(raw["module"][1], json!({"op":"lastRule"}));
        assert_eq!(raw["module"][2]["body"][4], json!({"op":"previousRule"}));
        let out = probe.policy(Some(&first), Some(&step), REVERSE, reverse, "export");
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("conflicting values"));
        assert!(
            !probe
                .policy(Some(&first), Some(&step), REVERSE, reverse, "eval")
                .status
                .success()
        );
    }
    // Complete tiny Rust predictions, all four cursor effects and actual compiled
    // evaluator behavior belong to the Nix verifier, never a compiler-provider ABI here.
}

#[test]
fn both_helpers_and_op_slots_are_required_and_schema_negatives_publish_no_json() {
    let probe = Probe::new();
    let first = probe.import(FIRST_NAME, r#"{"op":"firstRule"}"#);
    let step = probe.import(STEP_NAME, r#"{"op":"nextRule"}"#);
    let empty_first = probe.import(FIRST_NAME, "{}");
    let empty_step = probe.import(STEP_NAME, "{}");
    for (a, b) in [
        (None, None),
        (None, Some(step.as_str())),
        (Some(first.as_str()), None),
        (Some(empty_first.as_str()), Some(step.as_str())),
        (Some(first.as_str()), Some(empty_step.as_str())),
    ] {
        for reverse in [false, true] {
            let eval = probe.policy(a, b, SELECTOR, reverse, "eval");
            assert!(
                eval.status.success(),
                "missing DATA is incomplete, not broken setup: {}",
                String::from_utf8_lossy(&eval.stderr)
            );
            let out = probe.policy(a, b, SELECTOR, reverse, "export");
            failed(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete value"));
        }
    }
    for (name, invalid) in [
        (FIRST_NAME, json!({"op":1})),
        (STEP_NAME, json!({"op":false})),
        (FIRST_NAME, json!({"op":"firstRule","extra":"forbidden"})),
        (STEP_NAME, json!({"op":"nextRule","extra":"forbidden"})),
        (FIRST_NAME, json!({"op":"unknownEndpoint"})),
        (STEP_NAME, json!({"op":"unknownStep"})),
    ] {
        let provider = probe.import(name, &invalid.to_string());
        let (a, b) = if name == FIRST_NAME {
            (&provider, &step)
        } else {
            (&first, &provider)
        };
        for reverse in [false, true] {
            failed(&probe.policy(Some(a), Some(b), SELECTOR, reverse, "export"));
            assert!(
                !probe
                    .policy(Some(a), Some(b), SELECTOR, reverse, "eval")
                    .status
                    .success()
            );
        }
    }
    assert_eq!(
        decoded(probe.native(FIRST, FIRST_ROOT, false)),
        json!({"op":"firstRule"})
    );
    assert_eq!(
        decoded(probe.native(STEP, STEP_ROOT, false)),
        json!({"op":"nextRule"})
    );
}
