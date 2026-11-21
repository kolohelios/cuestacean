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

const OWNER: &str = include_str!("../../instruction-right-field.cue");
const ROOT: &str = include_str!("../../instruction-right-field-export.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const CORE_SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const HELPER: &str = "#InstructionRightField";
// Independent site inventory; Binary's eleven aliases share its one template.
const SITES: [&str; 9] = [
    "/cases/binaryRuntime/parts/4/value/value/field",
    "/cases/sum/args/1/value/value/field",
    "/cases/less/args/1/value/value/field",
    "/cases/lessEqual/args/1/value/value/field",
    "/cases/greater/args/1/value/value/field",
    "/cases/greaterEqual/args/1/value/value/field",
    "/cases/notEqual/args/1/value/value/field",
    "/cases/remainder/args/1/value/value/field",
    "/cases/unify/parts/1/args/1/value/value/field",
];
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file(), "regular tool");
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
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
    assert!(out.stdout.is_empty(), "no partial output");
    assert!(!out.stderr.is_empty());
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    native: PathBuf,
    full_exports: Cell<usize>,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-right-{}-{}",
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
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        Self {
            dir,
            cue,
            native,
            full_exports: Cell::new(0),
        }
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, source) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, source).expect("explicit source");
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
            &[("arbitrary-owner.cue", owner), ("other-root.cue", root)],
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
    fn isolated(&self, provider: Option<&str>, op: &str, reverse: bool) -> Output {
        let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
        if let Some(provider) = provider {
            inputs.push(("right.cue", provider));
        }
        // NO full selector/context or other helper can backfill this required field.
        self.run(&self.cue, &[op, "-e", HELPER], &inputs, reverse)
    }
    fn full(&self, bridge: &str, provider: Option<&str>, context: &str, reverse: bool) -> Output {
        let core_gate = self.run(
            &self.cue,
            &["export"],
            &[("core-shape.cue", CORE_SHAPE), ("core.cue", CORE)],
            reverse,
        );
        if !core_gate.status.success() {
            return core_gate;
        }
        let raw = self.run(
            &self.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary.cue", BINARY)],
            reverse,
        );
        assert!(raw.status.success(), "Go default-free Binary oracle setup");
        let binary = self.import(
            "#InstructionBinaryTables",
            &String::from_utf8(raw.stdout).expect("Go JSON"),
        );
        let gate = self.run(
            &self.cue,
            &["export", "-e", "#InstructionBinaryTables"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", bridge),
                ("binary-placed.cue", &binary),
            ],
            reverse,
        );
        assert!(gate.status.success(), "independent 35-slot Binary gate");
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", bridge),
            ("core.cue", CORE),
            ("binary-placed.cue", &binary),
            ("unary.cue", UNARY),
            ("text.cue", TEXT),
            ("first.cue", FIRST),
            ("step.cue", STEP),
        ];
        if let Some(provider) = provider {
            inputs.push(("right.cue", provider));
        }
        inputs.push(("context.cue", context));
        self.full_exports.set(self.full_exports.get() + 1);
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
    fn policy(&self, provider: Option<&str>, context: &str, reverse: bool) -> Output {
        let gate = self.isolated(provider, "export", reverse);
        if !gate.status.success() {
            return gate;
        }
        // Discard gate stdout. Reuse the SAME unresolved source, never its selected JSON.
        self.full(BRIDGE, provider, context, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}
fn source(body: &str) -> String {
    format!("package bootstrap\n{body}\n")
}
fn original_bridge() -> String {
    // Verification-only inverse of the two reviewed source changes, not a seed.
    assert_eq!(BRIDGE.matches("#InstructionRightField.field").count(), 1);
    BRIDGE
        .replace("#InstructionRightField: close({field: string})\n", "")
        .replace("#InstructionRightField.field", "*\"right\" | \"left\"")
}
fn context(mask: usize, field: &str) -> String {
    let leaf = format!(
        "{{value: value: field: {}}}",
        serde_json::to_string(field).expect("literal")
    );
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
            0 => format!("parts: [_, _, _, _, {leaf}, _]"),
            8 => format!("parts: [_, {{args: [_, {leaf}]}}, _]"),
            _ => format!("args: [_, {leaf}]"),
        };
        cases.push(format!("{name}: {{{body}}}"));
    }
    source(&format!(
        "#InstructionLowering & {{cases: {{{}}}}}",
        cases.join(",\n")
    ))
}

#[test]
fn complete_literal_tree_native_cli_go_orders_repeats_names_and_source_effects() {
    let p = Probe::new();
    for reverse in [false, true] {
        for _ in 0..2 {
            let (a, b) = if reverse {
                (ROOT, OWNER)
            } else {
                (OWNER, ROOT)
            };
            let data = export(a, b).expect("native raw two");
            assert_eq!(data.len(), 23);
            assert_eq!(
                serde_json::from_str::<Value>(&data).expect("JSON"),
                json!({"field":"right"})
            );
            assert_eq!(
                decoded(p.native(OWNER, ROOT, reverse)),
                json!({"field":"right"})
            );
            assert_eq!(
                decoded(p.run(
                    &p.cue,
                    &["export"],
                    &[("owner.cue", OWNER), ("root.cue", ROOT)],
                    reverse
                )),
                json!({"field":"right"})
            );
        }
        for (choice, want) in [
            ("*\"east\" | \"west\"", "east"),
            ("\"east\" | *\"west\"", "west"),
            ("*\"left\" | \"right\"", "left"),
        ] {
            let owner = source(&format!("#Unprivileged: {{unrelated: {choice}}}"));
            let root = source("#Unprivileged");
            let expected = json!({"unrelated":want});
            assert_eq!(decoded(p.native(&owner, &root, reverse)), expected);
            assert_eq!(
                decoded(p.run(
                    &p.cue,
                    &["export"],
                    &[("owner.cue", &owner), ("root.cue", &root)],
                    reverse
                )),
                expected
            );
        }
    }
}

#[test]
fn exact_existing_work_guard_unsupported_shapes_categories_and_fresh_recovery() {
    let p = Probe::new();
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (ROOT, OWNER)
        } else {
            (OWNER, ROOT)
        };
        // A(1)=31, E(1)=80, root stamp=1: no threshold capture.
        assert_eq!(
            export_with_limit(a, b, 111),
            Err(Error::Limit("evaluation steps"))
        );
        assert_eq!(
            serde_json::from_str::<Value>(&export_with_limit(a, b, 112).expect("exact work"))
                .expect("JSON"),
            json!({"field":"right"})
        );
        for root in [
            "#InstructionRightField & #InstructionRightField",
            "#InstructionRightField\n#InstructionRightField",
            "#Alias: #InstructionRightField\n#Alias",
            "let Local = #InstructionRightField\nLocal",
        ] {
            let root = source(root);
            let (a, b) = if reverse {
                (root.as_str(), OWNER)
            } else {
                (OWNER, root.as_str())
            };
            assert!(matches!(export(a, b), Err(Error::Unsupported(_))));
            failed(&p.native(OWNER, &root, reverse));
        }
        let extra = OWNER.replace("field:", "sibling: 1, field:");
        assert!(matches!(export(&extra, ROOT), Err(Error::Unsupported(_))));
        failed(&p.native(&extra, ROOT, reverse));
        assert_eq!(
            export(OWNER, &source("#Missing")),
            Err(Error::Unknown("#Missing".into()))
        );
        assert_eq!(
            decoded(p.native(OWNER, ROOT, reverse)),
            json!({"field":"right"})
        );
    }
}

#[test]
fn raw_scoped_nine_sites_both_priorities_orders_masks_and_complete_original_policy() {
    let p = Probe::new();
    let old = original_bridge();
    let reversed = OWNER.replace("*\"right\" | \"left\"", "\"right\" | *\"left\"");
    let old_reversed = old.replace("*\"right\" | \"left\"", "\"right\" | *\"left\"");
    for reverse in [false, true] {
        for (owner, bridge, default, forced) in [
            (OWNER, old.as_str(), "right", "left"),
            (reversed.as_str(), old_reversed.as_str(), "left", "right"),
        ] {
            let baseline = decoded(p.full(bridge, None, SELECTOR, reverse));
            for site in SITES {
                assert_eq!(baseline.pointer(site).expect("site"), default);
            }
            let masks = (0..9)
                .map(|i| 1 << i)
                .chain([0, 511, 0b101010101, 0b010101010]);
            for mask in masks {
                let ctx = context(mask, forced);
                let mut expected = baseline.clone();
                for (i, site) in SITES.iter().enumerate() {
                    *expected.pointer_mut(site).expect("independent site") =
                        json!(if mask & (1 << i) == 0 {
                            default
                        } else {
                            forced
                        });
                }
                assert_eq!(decoded(p.policy(Some(owner), &ctx, reverse)), expected);
                assert_eq!(decoded(p.full(bridge, None, &ctx, reverse)), expected);
            }
        }
        for ctx in [SELECTOR, TWICE, REVERSE] {
            assert_eq!(
                decoded(p.policy(Some(OWNER), ctx, reverse)),
                decoded(p.full(&old, None, ctx, reverse))
            );
        }
        for (first, step) in [("firstRule", "previousRule"), ("lastRule", "nextRule")] {
            let ctx = source(&format!(
                "#InstructionLowering & {{module: [_, {{op: \"{first}\"}}, {{body: [_, _, _, _, {{op: \"{step}\"}}]}}, _]}}"
            ));
            assert_eq!(
                decoded(p.policy(Some(OWNER), &ctx, reverse)),
                decoded(p.full(&old, None, &ctx, reverse))
            );
        }
    }
}

#[test]
fn isolated_missing_owner_field_fail_stop_despite_actual_all_left_backfill() {
    let p = Probe::new();
    let all_left = context(511, "left");
    let empty = source("#InstructionRightField: {}");
    for reverse in [false, true] {
        let bypass = decoded(p.full(BRIDGE, None, &all_left, reverse));
        for site in SITES {
            assert_eq!(bypass.pointer(site).expect("backfilled site"), "left");
        }
        assert_eq!(
            bypass,
            decoded(p.full(&original_bridge(), None, &all_left, reverse))
        );
        for provider in [None, Some(empty.as_str())] {
            let eval = p.isolated(provider, "eval", reverse);
            assert!(
                eval.status.success(),
                "{}",
                String::from_utf8_lossy(&eval.stderr)
            );
            failed(&p.isolated(provider, "export", reverse));
            let before = p.full_exports.get();
            failed(&p.policy(provider, &all_left, reverse));
            assert_eq!(
                p.full_exports.get(),
                before,
                "no full export after gate failure"
            );
        }
        for body in [
            "#InstructionRightField: {field: 7}",
            "#InstructionRightField: {field: \"right\", extra: 1}",
        ] {
            let provider = source(body);
            failed(&p.isolated(Some(&provider), "eval", reverse));
            failed(&p.isolated(Some(&provider), "export", reverse));
            let before = p.full_exports.get();
            failed(&p.policy(Some(&provider), &all_left, reverse));
            assert_eq!(p.full_exports.get(), before);
        }
    }
}

#[test]
fn raw_gate_discards_selection_selected_data_conflicts_and_go_typed_phase_inputs() {
    let p = Probe::new();
    let selected = export(OWNER, ROOT).expect("native DATA");
    let placed = p.import(HELPER, &selected);
    for reverse in [false, true] {
        assert_eq!(
            decoded(p.policy(Some(&placed), SELECTOR, reverse)),
            decoded(p.policy(Some(OWNER), SELECTOR, reverse))
        );
        let raw = decoded(p.policy(Some(OWNER), TWICE, reverse));
        for (i, site) in SITES.iter().enumerate() {
            assert_eq!(
                raw.pointer(site).expect("site"),
                if i == 1 { "left" } else { "right" }
            );
        }
        let conflict = p.policy(Some(&placed), TWICE, reverse);
        failed(&conflict);
        assert!(String::from_utf8_lossy(&conflict.stderr).contains("conflicting values"));
        let baseline = decoded(p.policy(Some(OWNER), SELECTOR, reverse));
        for (owner, field) in [
            (
                OWNER.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
                "left",
            ),
            (OWNER.replace("*\"right\"", "*\"value\""), "value"),
        ] {
            let (a, b) = if reverse {
                (ROOT, owner.as_str())
            } else {
                (owner.as_str(), ROOT)
            };
            let data = export(a, b).expect("source-effect native DATA");
            assert_eq!(
                serde_json::from_str::<Value>(&data).expect("JSON"),
                json!({"field":field})
            );
            assert_eq!(
                decoded(p.native(&owner, ROOT, reverse)),
                json!({"field":field})
            );
            let imported = p.import(HELPER, &data);
            let mut expected = baseline.clone();
            for site in SITES {
                *expected.pointer_mut(site).expect("source-effect site") = json!(field);
            }
            assert_eq!(
                decoded(p.policy(Some(&imported), SELECTOR, reverse)),
                expected
            );
            assert_eq!(decoded(p.policy(Some(&owner), SELECTOR, reverse)), expected);
        }
        let empty = source("#InstructionRightField: {field: \"\"}");
        assert_eq!(
            decoded(p.isolated(Some(&empty), "export", reverse)),
            json!({"field":""})
        );
        failed(&p.policy(Some(&empty), SELECTOR, reverse));
        let long = "x".repeat(129);
        let provider = source(&format!("#InstructionRightField: {{field: \"{long}\"}}"));
        assert_eq!(
            decoded(p.isolated(Some(&provider), "export", reverse)),
            json!({"field":long})
        );
        let policy = decoded(p.policy(Some(&provider), SELECTOR, reverse));
        for site in SITES {
            assert_eq!(policy.pointer(site).expect("typed-boundary input"), &long);
        }
        // Real compiler exit2/empty stdout and complete independent Rust are Nix
        // verifier obligations; no compiler-provider ABI or fake binding here.
    }
}
