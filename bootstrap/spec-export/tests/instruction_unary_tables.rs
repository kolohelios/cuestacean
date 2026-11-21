use cuestacean_spec_export::{Error, export};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const TABLES: &str = include_str!("../../instruction-unary-tables.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
// Independent instruction-policy contract, never parsed from the owner/export.
const UNARY: [(&str, &str); 10] = [
    ("lowerExclusiveBound", "lower_exclusive_bound"),
    ("upperExclusiveBound", "upper_exclusive_bound"),
    ("lowerBound", "lower_bound"),
    ("upperBound", "upper_bound"),
    ("negate", "negate"),
    ("logicalNot", "logical_not"),
    ("length", "length"),
    ("stringLength", "string_length"),
    ("listLength", "list_length"),
    ("recordLength", "record_length"),
];
fn text(value: &str) -> Value {
    json!({"op":"text","value":value})
}
fn expected(rows: &[(&str, &str)], prefix: &str, suffix: &str) -> Value {
    let mut aliases = json!({});
    let mut calls = json!({});
    for (key, method) in rows {
        aliases[key] = json!("unaryRuntime");
        calls[key] = text(&format!("{prefix}{method}{suffix}"));
    }
    json!({"aliases":aliases,"cases":{"unaryRuntime":{"op":"concat","parts":[
        {"op":"select","field":"op","cases":calls},text("("),
        {"op":"try","value":{"op":"group","value":{"op":"lower","field":"value"}}},text(")")
    ]}}})
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
    assert!(
        fs::metadata(path).expect("metadata").is_file(),
        "regular tool"
    );
    fs::File::open(path).expect("readable tool");
    assert_ne!(
        fs::metadata(path).expect("metadata").permissions().mode() & 0o111,
        0
    );
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    jq: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "instruction-unary-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let tool = |name| {
            let path = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute tool");
            executable(&path);
            path
        };
        executable(Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export")));
        Self {
            dir,
            cue: tool("cue"),
            jq: tool("jq"),
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
    fn native(&self, source: &str, reverse: bool) -> Output {
        self.run(
            Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export")),
            &[],
            &[("helper.cue", PACKAGE), ("different-name.cue", source)],
            reverse,
        )
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
        provider: Option<(&str, &str)>,
        context: usize,
        reverse: bool,
        op: &str,
    ) -> Output {
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
        // Replacement bridge ONLY: old Unary owner cannot conceal missing slots.
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("selector.cue", SELECTOR),
            ("module-text.cue", TEXT),
            ("first.cue", FIRST),
            ("step.cue", STEP),
            ("right.cue", RIGHT),
            ("binary-placed.cue", &binary),
        ];
        if let Some(provider) = provider {
            inputs.push(provider);
        }
        match context {
            0 => {}
            1 => inputs.push(("context.cue", TWICE)),
            2 => inputs.push(("context.cue", REVERSE)),
            _ => panic!("context"),
        }
        self.run(&self.cue, &[op], &inputs, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn original_two_comprehensions_complete_independent_ten_alias_case_tree() {
    let probe = Probe::new();
    let want = expected(&UNARY, "runtime::", "");
    assert_eq!(
        TABLES.matches("for ").count(),
        2,
        "retain executable CUE comprehensions"
    );
    assert_eq!(want["aliases"].as_object().expect("aliases").len(), 10);
    assert_eq!(want["cases"].as_object().expect("cases").len(), 1);
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (TABLES, PACKAGE)
        } else {
            (PACKAGE, TABLES)
        };
        let native = export(a, b).expect("partial native DATA");
        assert_eq!(serde_json::from_str::<Value>(&native).expect("JSON"), want);
        assert_eq!(native.len(), 2107);
        assert_eq!(decoded(probe.native(TABLES, reverse)), want);
        let raw = probe.run(
            &probe.cue,
            &["export"],
            &[("helper.cue", PACKAGE), ("owner.cue", TABLES)],
            reverse,
        );
        assert!(
            raw.status.success(),
            "{}",
            String::from_utf8_lossy(&raw.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&raw.stdout).expect("oracle DATA"),
            want
        );
        let sorted = probe.run(
            &probe.jq,
            &["-S", "."],
            &[("raw.json", std::str::from_utf8(&raw.stdout).expect("UTF-8"))],
            false,
        );
        assert!(sorted.status.success());
        assert_eq!(sorted.stdout, native.as_bytes());
        let renamed = TABLES
            .replace("Unary", "OtherTable")
            .replace("name,", "entry,")
            .replace("(name)", "(entry)")
            .replace("method", "target");
        assert_eq!(decoded(probe.native(&renamed, reverse)), want);
    }
}

#[test]
fn source_keys_methods_and_affixes_change_complete_independent_module() {
    let probe = Probe::new();
    let mut rows = UNARY;
    rows[0] = ("differentBound", "different_method");
    let variant = TABLES
        .replace("lowerExclusiveBound:", "differentBound:")
        .replace("\"lower_exclusive_bound\"", "\"different_method\"")
        .replace("runtime::\\(method)", "changed::\\(method)_suffix");
    assert_ne!(variant, TABLES);
    let want = expected(&rows, "changed::", "_suffix");
    for reverse in [false, true] {
        assert_eq!(decoded(probe.native(&variant, reverse)), want);
        assert_eq!(
            decoded(probe.run(
                &probe.cue,
                &["export"],
                &[("helper.cue", PACKAGE), ("variant.cue", &variant)],
                reverse
            )),
            want
        );
    }
    for wrong in [
        TABLES.replace("\\(method)", "\\(outer)"),
        TABLES.replace("\"lower_exclusive_bound\"", "true"),
    ] {
        assert!(matches!(
            export(PACKAGE, &wrong),
            Err(Error::Unsupported(_))
        ));
        let out = probe.native(&wrong, false);
        assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
    }
}

#[test]
fn replacement_bridge_retains_live_go_policy_contexts_and_requires_all_eleven_slots() {
    let probe = Probe::new();
    let want = expected(&UNARY, "runtime::", "");
    let native = export(PACKAGE, TABLES).expect("native module");
    let mut policies = Vec::new();
    for context in 0..3 {
        for reverse in [false, true] {
            let raw = decoded(probe.policy(Some(("raw.cue", TABLES)), context, reverse, "export"));
            let data =
                decoded(probe.policy(Some(("data.json", &native)), context, reverse, "export"));
            assert_eq!(raw, data, "raw owner vs fixed DATA under live Go context");
            assert_eq!(data["formatVersion"], 1);
            for (key, _) in UNARY {
                assert_eq!(data["aliases"][key], want["aliases"][key]);
            }
            assert_eq!(data["cases"]["unaryRuntime"], want["cases"]["unaryRuntime"]);
            assert_eq!(
                data["cases"]["sum"]["args"][1]["value"]["value"]["field"],
                if context == 1 { "left" } else { "right" }
            );
            assert_eq!(
                data["module"][1]["op"],
                if context == 2 {
                    "lastRule"
                } else {
                    "firstRule"
                }
            );
            if !reverse {
                policies.push(data);
            } else {
                assert_eq!(policies[context], data);
            }
        }
    }
    for slot in std::iter::once(None).chain((0..11).map(Some)) {
        let mut data = want.clone();
        if let Some(index) = slot {
            if index < 10 {
                data["aliases"]
                    .as_object_mut()
                    .expect("aliases")
                    .remove(UNARY[index].0);
            } else {
                data["cases"]
                    .as_object_mut()
                    .expect("cases")
                    .remove("unaryRuntime");
            }
        }
        let json = serde_json::to_string(&data).expect("independent partial provider");
        for reverse in [false, true] {
            let provider = slot.map(|_| ("partial.json", json.as_str()));
            let eval = probe.policy(provider, 0, reverse, "eval");
            assert!(
                eval.status.success(),
                "setup must evaluate: {}",
                String::from_utf8_lossy(&eval.stderr)
            );
            let out = probe.policy(provider, 0, reverse, "export");
            assert!(!out.status.success() && out.stdout.is_empty());
            assert_eq!(
                String::from_utf8_lossy(&out.stderr)
                    .matches("incomplete value")
                    .count(),
                if slot.is_none() { 11 } else { 1 },
                "must be missing required slot, not malformed setup"
            );
        }
    }
    assert_eq!(decoded(probe.native(TABLES, false)), want);
    // Actual compiler/goldens/source-effect complete Rust and schema+typed no-Rust
    // witnesses belong to Nix verification, never a Rust-test provider ABI.
}
