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
const BODY: &str = include_str!("../../meta-lowering-body.cue");
const TEXT: &str = include_str!("../../meta-text-alias.cue");
const NEXT: &str = include_str!("../../meta-next-rule-alias.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../meta-lowering-fragments.cue");
const SELECTOR: &str = include_str!("../../meta-lowering.cue");
const FORCED: &str = include_str!("../../tests/valid/meta-lowering-forced.cue");
const PREVIOUS: &str = include_str!("../../tests/valid/meta-driver-previous.cue");
const COMPOSE: &str = include_str!("../../compose-policy.jq");

fn parsed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("complete policy JSON")
}
fn failed(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no partial JSON");
    assert!(!output.stderr.is_empty(), "mandatory diagnostic");
}
fn text(output: Output) -> String {
    parsed(&output);
    assert!(output.stdout.len() <= 65536, "bounded module/intermediate");
    String::from_utf8(output.stdout).expect("successful JSON UTF-8")
}
struct Probe {
    dir: PathBuf,
    native: PathBuf,
    cue: PathBuf,
    jq: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "meta-policy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(dir.is_absolute());
        fs::create_dir(&dir).expect("fresh owned scratch");
        fs::create_dir(dir.join("home")).expect("owned HOME");
        let provider = |name| {
            std::env::split_paths(&std::env::var_os("PATH").expect("pinned toolchain PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned provider")
                .canonicalize()
                .expect("absolute provider")
        };
        let probe = Self {
            dir,
            native: PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export")),
            cue: provider("cue"),
            jq: provider("jq"),
        };
        for binary in [&probe.native, &probe.cue, &probe.jq] {
            assert!(binary.is_absolute());
            fs::File::open(binary).expect("readable mandatory provider");
            assert_ne!(
                fs::metadata(binary)
                    .expect("provider metadata")
                    .permissions()
                    .mode()
                    & 0o111,
                0
            );
        }
        fs::write(probe.dir.join("compose-policy.jq"), COMPOSE)
            .expect("unchanged production composer");
        probe
    }
    fn run(&self, binary: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        let mut names = Vec::new();
        for (name, source) in inputs {
            fs::write(self.dir.join(name), source).expect("owned fresh input");
            names.push(*name);
        }
        if reverse {
            names.reverse();
        }
        Command::new(binary)
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", self.dir.join("home"))
            .current_dir(&self.dir)
            .args(args)
            .args(names)
            .output()
            .expect("fresh mandatory process")
    }
    fn native(&self, source: &str, reverse: bool) -> String {
        let inputs = [("z-package.cue", PACKAGE), ("a-source.cue", source)];
        let first = self.run(&self.native, &[], &inputs, reverse);
        let second = self.run(&self.native, &[], &inputs, reverse);
        parsed(&second);
        assert_eq!(first.stdout, second.stdout, "fresh repeat bytes");
        text(first)
    }
    fn compose(&self, a: &str, b: &str, reverse: bool) -> String {
        text(self.run(
            &self.jq,
            &["-s", "-S", "-f", "compose-policy.jq"],
            &[("left.json", a), ("right.json", b)],
            reverse,
        ))
    }
    fn go(
        &self,
        providers: [Option<&str>; 3],
        context: Option<&str>,
        operation: &str,
        reverse: bool,
    ) -> Output {
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("selector.cue", SELECTOR),
        ];
        for (name, provider) in ["body.cue", "text.cue", "next.cue"]
            .into_iter()
            .zip(providers)
        {
            if let Some(provider) = provider {
                inputs.push((name, provider));
            }
        }
        if let Some(context) = context {
            inputs.push(("context.cue", context));
        }
        self.run(&self.cue, &[operation], &inputs, reverse)
    }
    fn complete(&self, reverse: bool) -> String {
        let body = self.native(BODY, reverse);
        let text = self.native(TEXT, reverse);
        let next = self.native(NEXT, reverse);
        let first = self.compose(&body, &text, reverse);
        self.compose(&first, &next, reverse)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}

#[test]
fn direct_two_file_modules_and_binary_assembly_agree_with_whole_raw_schema_export() {
    let probe = Probe::new();
    // COMPLETE independently authored alias module trees, not provider captures.
    let aliases = [
        (TEXT, json!({"aliases":{"text":"emitValue"}})),
        (NEXT, json!({"aliases":{"nextRule":"emitModuleAction"}})),
    ];
    let mut previous = None;
    for reverse in [false, true] {
        for _ in 0..2 {
            for (source, expected) in &aliases {
                let (a, b) = if reverse {
                    (*source, PACKAGE)
                } else {
                    (PACKAGE, *source)
                };
                assert_eq!(
                    serde_json::from_str::<Value>(&export(a, b).expect("raw alias library export"))
                        .expect("alias JSON"),
                    *expected
                );
                assert_eq!(
                    serde_json::from_str::<Value>(&probe.native(source, reverse))
                        .expect("CLI alias JSON"),
                    *expected
                );
            }
            let body = probe.native(BODY, reverse);
            let body_value: Value = serde_json::from_str(&body).expect("body JSON");
            assert_eq!(body_value["formatVersion"], 1);
            assert!(body_value["aliases"].get("text").is_none());
            assert!(body_value["aliases"].get("nextRule").is_none());
            let native_body = export(PACKAGE, BODY).expect("ordinary native library body");
            assert_eq!(
                serde_json::from_str::<Value>(&native_body).expect("library body JSON"),
                body_value
            );
            let full = probe.complete(reverse);
            assert!(full.len() <= 65536);
            let go = text(probe.go(
                [Some(BODY), Some(TEXT), Some(NEXT)],
                None,
                "export",
                reverse,
            ));
            let sorted = probe.run(&probe.jq, &["-S", "."], &[("oracle.json", &go)], false);
            parsed(&sorted);
            assert_eq!(full.as_bytes(), sorted.stdout);
            if let Some(bytes) = &previous {
                assert_eq!(&full, bytes);
            }
            previous = Some(full.clone());
            // Different grouping, not a three-input invocation of the composer.
            let text = probe.native(TEXT, reverse);
            let next = probe.native(NEXT, reverse);
            let alias_pair = probe.compose(&next, &text, reverse);
            assert_eq!(probe.compose(&alias_pair, &body, reverse), full);
        }
    }
}

#[test]
fn bridge_requires_every_case_and_both_raw_alias_providers_without_json() {
    let probe = Probe::new();
    for providers in [
        [None, Some(TEXT), Some(NEXT)],
        [Some(BODY), None, Some(NEXT)],
        [Some(BODY), Some(TEXT), None],
        [None, None, None],
    ] {
        for reverse in [false, true] {
            let output = probe.go(providers, None, "export", reverse);
            failed(&output);
            assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete value"));
            let eval = probe.go(providers, None, "eval", reverse);
            assert!(
                eval.status.success(),
                "{}",
                String::from_utf8_lossy(&eval.stderr)
            );
        }
    }
    // A bridge requiring only one sentinel case is insufficient: remove each
    // original case independently from materialized DATA before schema validation.
    let full: Value = serde_json::from_str(&probe.complete(false)).expect("complete native policy");
    for name in full["cases"].as_object().expect("policy cases").keys() {
        let mut missing = full.clone();
        missing["cases"]
            .as_object_mut()
            .expect("mutable cases")
            .remove(name);
        let data = serde_json::to_string(&missing).expect("missing-case data");
        let output = probe.run(
            &probe.cue,
            &["export"],
            &[
                ("schema.cue", SCHEMA),
                ("bridge.cue", BRIDGE),
                ("selector.cue", SELECTOR),
                ("data.json", &data),
            ],
            false,
        );
        failed(&output);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("incomplete value"),
            "required case {name}"
        );
    }
}

#[test]
fn raw_text_and_previous_contexts_refine_before_selection_not_after_materialization() {
    let probe = Probe::new();
    let full = probe.complete(false);
    let default: Value = serde_json::from_str(&full).expect("complete default policy");
    for (context, alias, target, instruction) in [
        (
            FORCED,
            "text",
            "emitForced",
            json!({"op":"text","value":"ctx.text(\"forced\")"}),
        ),
        (
            PREVIOUS,
            "nextRule",
            "emitStepBack",
            json!({"op":"text","value":"frame.step()?; frame.previous_rule()?;\n"}),
        ),
    ] {
        let mut expected = default.clone();
        expected["aliases"][alias] = json!(target);
        expected["cases"][target] = instruction;
        for reverse in [false, true] {
            assert_eq!(
                parsed(&probe.go(
                    [Some(BODY), Some(TEXT), Some(NEXT)],
                    Some(context),
                    "export",
                    reverse
                )),
                expected
            );
            for operation in ["export", "eval"] {
                failed(&probe.run(
                    &probe.cue,
                    &[operation],
                    &[
                        ("schema.cue", SCHEMA),
                        ("bridge.cue", BRIDGE),
                        ("selector.cue", SELECTOR),
                        ("default.json", &full),
                        ("context.cue", context),
                    ],
                    reverse,
                ));
            }
        }
    }
}

#[test]
fn separate_alias_modules_do_not_admit_multiple_groups_in_one_native_document() {
    for (a, b) in [(TEXT, NEXT), (NEXT, TEXT)] {
        assert!(matches!(export(a, b), Err(Error::Unsupported(_))));
    }
    let probe = Probe::new();
    for reverse in [false, true] {
        failed(&probe.run(
            &probe.native,
            &[],
            &[("text.cue", TEXT), ("next.cue", NEXT)],
            reverse,
        ));
    }
}
