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
const TABLES: &str = include_str!("../../pratt-lowering-tables.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../pratt-lowering-fragments.cue");
const SELECTOR: &str = include_str!("../../pratt-lowering.cue");
const NO_INCREMENT: &str = include_str!("../../tests/valid/pratt-lowering-no-increment.cue");
// Independently authored mappings from the Pratt backend instruction contract,
// not parsed from source, native/Go exports, or scratch characterization captures.
const MINIMUM: [(&str, &str); 7] = [
    ("parseSliceUpper", "parse_slice_upper"),
    ("parseRecordValue", "parse_record_value"),
    ("parseIndex", "parse_index"),
    ("parseListElement", "parse_list_element"),
    ("parseGroup", "parse_group"),
    ("parseNot", "parse_not"),
    ("parseNegation", "parse_negation"),
];
const SELECTORS: [(&str, &str); 7] = [
    ("scanSelector", "scan_selector"),
    ("advanceSelector", "advance_selector"),
    ("selectorWhitespace", "selector_whitespace"),
    ("scanSelectorString", "scan_selector_string"),
    ("parseSelectorString", "parse_selector_string"),
    ("parseSelectorIdentifier", "parse_selector_identifier"),
    ("buildSelector", "build_selector"),
];
fn text(value: &str) -> Value {
    json!({"op":"text","value":value})
}
fn expected(minimum: &[(&str, &str)], selectors: &[(&str, &str)], suffix: &str) -> Value {
    let mut aliases = json!({});
    let mut calls = json!({});
    let mut actions = json!({});
    for &(key, value) in minimum {
        aliases[key] = json!("parseMinimum");
        calls[key] = text(value);
    }
    for &(key, value) in selectors {
        actions[key] = text(value);
    }
    json!({"aliases":aliases,"cases":{
        "selectorAction":{"op":"concat","parts":[text("_frame.step()?;\n_frame."),
            {"op":"select","field":"action","cases":actions},text(suffix)]},
        "parseMinimum":{"op":"concat","parts":[text("_frame.step()?;\n_frame."),
            {"op":"select","field":"op","cases":calls},text("("),
            {"op":"lower","field":"minimum"},text(")?;\n")]}
    }})
}
fn parsed(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn executable(tool: &Path) {
    assert!(tool.is_absolute(), "absolute tool required");
    fs::File::open(tool).expect("readable tool");
    assert_ne!(
        fs::metadata(tool)
            .expect("tool metadata")
            .permissions()
            .mode()
            & 0o111,
        0,
        "executable tool required"
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
            "pratt-tables-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh scratch");
        let provider = |name| {
            std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute tool")
        };
        Self {
            dir,
            cue: provider("cue"),
            jq: provider("jq"),
        }
    }
    fn run(
        &self,
        tool: &std::path::Path,
        args: &[&str],
        inputs: &[(&str, &str)],
        reverse: bool,
    ) -> Output {
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
            std::path::Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export")),
            &[],
            &[
                ("renamed-package.cue", PACKAGE),
                ("renamed-tables.cue", source),
            ],
            reverse,
        )
    }
    fn policy(&self, provider: Option<(&str, &str)>, context: bool, reverse: bool) -> Output {
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("selector.cue", SELECTOR),
        ];
        if let Some(provider) = provider {
            inputs.push(provider);
        }
        if context {
            inputs.push(("context.cue", NO_INCREMENT));
        }
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn actual_three_comprehensions_complete_independent_module_and_oracle() {
    let probe = Probe::new();
    let want = expected(&MINIMUM, &SELECTORS, "()?;\n");
    assert_eq!(
        TABLES.matches("for ").count(),
        3,
        "actual executable comprehensions retained"
    );
    assert_eq!(want["aliases"].as_object().expect("aliases").len(), 7);
    assert_eq!(want["cases"].as_object().expect("case trees").len(), 2);
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (TABLES, PACKAGE)
        } else {
            (PACKAGE, TABLES)
        };
        let native = export(a, b).expect("actual table module");
        assert_eq!(
            serde_json::from_str::<Value>(&native).expect("module"),
            want
        );
        assert_eq!(native.len(), 2779);
        assert_eq!(parsed(probe.native(TABLES, reverse)), want);
        let raw = probe.run(
            &probe.cue,
            &["export"],
            &[
                ("different-package.cue", PACKAGE),
                ("different-owner.cue", TABLES),
            ],
            reverse,
        );
        assert!(
            raw.status.success(),
            "{}",
            String::from_utf8_lossy(&raw.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&raw.stdout).expect("oracle module"),
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
            .replace("MinimumCalls", "OtherMinimum")
            .replace("SelectorCalls", "OtherSelectors");
        assert_eq!(parsed(probe.native(&renamed, reverse)), want);
    }
}

#[test]
fn source_key_method_and_template_effects_are_not_name_privileged() {
    let probe = Probe::new();
    let mut minimum = MINIMUM;
    minimum[0] = ("differentCall", "different_method");
    let mut selectors = SELECTORS;
    selectors[0] = ("differentSelector", "different_selector");
    let changed = TABLES
        .replace("parseSliceUpper", "differentCall")
        .replace("parse_slice_upper", "different_method")
        .replace("scanSelector:", "differentSelector:")
        .replace("\"scan_selector\"", "\"different_selector\"")
        .replace("value: \"()?;\\n\"", "value: \"changed();\\n\"");
    assert_ne!(changed, TABLES);
    let want = expected(&minimum, &selectors, "changed();\n");
    for reverse in [false, true] {
        assert_eq!(parsed(probe.native(&changed, reverse)), want);
        assert_eq!(
            parsed(probe.run(
                &probe.cue,
                &["export"],
                &[("package.cue", PACKAGE), ("variant.cue", &changed)],
                reverse
            )),
            want
        );
    }
    for wrong in [
        TABLES.replace("value: method", "value: outer"),
        TABLES.replace("\"parse_slice_upper\"", "string"),
    ] {
        assert!(matches!(
            export(PACKAGE, &wrong),
            Err(Error::Unsupported(_))
        ));
        let out = probe.native(&wrong, false);
        assert!(!out.status.success() && out.stdout.is_empty());
    }
}

#[test]
fn complete_go_policy_keeps_raw_defaults_and_requires_every_provider_slot() {
    let probe = Probe::new();
    let native = export(PACKAGE, TABLES).expect("native module");
    for context in [false, true] {
        for reverse in [false, true] {
            let raw = parsed(probe.policy(Some(("raw-tables.cue", TABLES)), context, reverse));
            let supplied =
                parsed(probe.policy(Some(("native-tables.json", &native)), context, reverse));
            assert_eq!(
                raw, supplied,
                "full Go policy, including live add preference"
            );
            assert_eq!(
                supplied["aliases"],
                expected(&MINIMUM, &SELECTORS, "()?;\n")["aliases"]
            );
            if context {
                assert_eq!(
                    supplied["cases"]["add"],
                    json!({"op":"lower","field":"left"})
                );
            } else {
                assert_eq!(supplied["cases"]["add"]["op"], "concat");
            }
        }
    }
    for slot in std::iter::once(None).chain((0..9).map(Some)) {
        let mut data = expected(&MINIMUM, &SELECTORS, "()?;\n");
        if let Some(index) = slot {
            if index < 7 {
                data["aliases"]
                    .as_object_mut()
                    .expect("aliases")
                    .remove(MINIMUM[index].0);
            } else {
                data["cases"]
                    .as_object_mut()
                    .expect("cases")
                    .remove(if index == 7 {
                        "selectorAction"
                    } else {
                        "parseMinimum"
                    });
            }
        }
        let json = serde_json::to_string(&data).expect("independent DATA");
        for reverse in [false, true] {
            let out = probe.policy(
                slot.map(|_| ("missing.json", json.as_str())),
                false,
                reverse,
            );
            assert!(!out.status.success() && out.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("incomplete value"),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    assert_eq!(
        parsed(probe.native(TABLES, false)),
        expected(&MINIMUM, &SELECTORS, "()?;\n")
    );
    // Actual Pratt compiler / complete predicted Rust / schema+typed no-Rust
    // witnesses live in the Nix verifier, not a new Rust-test provider ABI.
}
