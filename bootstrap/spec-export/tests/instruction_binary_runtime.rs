use serde_json::{Value, json};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const RUNTIME: &str = include_str!("../../instruction-binary-runtime.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const R: &str = include_str!("../../instruction-right-operand.cue");
const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const HELPER: &str = "#InstructionBinaryRuntime";
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
fn wrapped(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":field}}})
}
fn expected(left: &str, right: &str, prefix: &str) -> Value {
    let methods = [
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
    let cases: serde_json::Map<String, Value> = methods
        .into_iter()
        .map(|(key, method)| {
            (
                key.into(),
                json!({"op":"text","value":format!("runtime::{prefix}{method}")}),
            )
        })
        .collect();
    json!({"op":"concat","parts":[{"op":"select","field":"op","cases":cases},
        {"op":"text","value":"("},wrapped(left),{"op":"text","value":", "},
        wrapped(right),{"op":"text","value":")"}]})
}
fn placed(value: &Value) -> String {
    format!("package bootstrap\n{HELPER}: {value}\n")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete Go JSON")
}
fn failed(out: &Output) {
    assert_eq!(out.status.code(), Some(1), "Go phase: {out:?}");
    assert!(out.stdout.is_empty() && !out.stderr.is_empty());
}
fn executable(path: &Path) {
    assert!(path.is_absolute() && path.is_file());
    let metadata = fs::metadata(path).expect("mandatory tool metadata");
    assert_ne!(metadata.permissions().mode() & 0o111, 0);
    fs::File::open(path).expect("mandatory readable tool");
}
#[derive(Clone, Copy)]
enum Mode {
    Accepted,
    MissingRight,
    MissingR,
    MissingGroup,
    MissingBinary,
    MissingCore,
    MissingRuntime,
    Legacy,
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    shell: PathBuf,
    binary: String,
    exports: Cell<usize>,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "binary-runtime-{}-{}",
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
        let _jq = discover("jq");
        let mut p = Self {
            dir,
            cue: discover("cue"),
            shell: discover("sh"),
            binary: String::new(),
            exports: Cell::new(0),
        };
        p.binary = p.binary_provider(BINARY);
        p
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
    fn binary_provider(&self, source: &str) -> String {
        let raw = self.run(
            &self.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary-owner.cue", source)],
            false,
        );
        let data = decoded(raw).to_string();
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
            "Binary import: {}",
            String::from_utf8_lossy(&imported.stderr)
        );
        String::from_utf8(imported.stdout).expect("whole default-free Binary import")
    }
    fn owners<'a>(&'a self, runtime: Option<&'a str>) -> [Option<&'a str>; 5] {
        [
            Some(RIGHT),
            Some(R),
            Some(GROUP),
            Some(&self.binary),
            runtime,
        ]
    }
    fn isolated(
        &self,
        bridge: &str,
        owners: [Option<&str>; 5],
        helper: &str,
        op: &str,
        reverse: bool,
    ) -> Output {
        let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", bridge)];
        for (name, source) in [
            "right.cue",
            "r.cue",
            "group.cue",
            "binary.cue",
            "runtime.cue",
        ]
        .into_iter()
        .zip(owners)
        {
            if let Some(source) = source {
                inputs.push((name, source));
            }
        }
        self.run(&self.cue, &[op, "-e", helper], &inputs, reverse)
    }
    fn full(
        &self,
        bridge: &str,
        owners: [Option<&str>; 5],
        core: Option<&str>,
        context: &str,
        reverse: bool,
        mode: Mode,
    ) -> Output {
        let [right, r, group, binary, runtime] = owners;
        let stages = [
            (
                "#InstructionRightField",
                [right, None, None, None, None],
                !matches!(mode, Mode::MissingRight),
            ),
            (
                "#InstructionRightOperand",
                [right, r, None, None, None],
                !matches!(mode, Mode::MissingRight | Mode::MissingR),
            ),
            (
                "#InstructionOperandCases",
                [right, r, group, None, None],
                !matches!(
                    mode,
                    Mode::MissingRight | Mode::MissingR | Mode::MissingGroup
                ),
            ),
            (
                "#InstructionBinaryTables",
                [None, None, None, binary, None],
                !matches!(mode, Mode::MissingBinary),
            ),
            (
                HELPER,
                owners,
                matches!(mode, Mode::Accepted | Mode::MissingCore),
            ),
        ];
        for (helper, providers, enabled) in stages {
            if enabled {
                let out = self.isolated(bridge, providers, helper, "export", reverse);
                if !out.status.success() {
                    return out;
                }
            }
        }
        if !matches!(mode, Mode::MissingCore) {
            let mut inputs = vec![("shape.cue", SHAPE)];
            if let Some(core) = core {
                inputs.push(("core.cue", core));
            }
            let out = self.run(&self.cue, &["export"], &inputs, reverse);
            if !out.status.success() {
                return out;
            }
        }
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", bridge),
            ("unary.cue", UNARY),
            ("text.cue", TEXT),
            ("first.cue", FIRST),
            ("step.cue", STEP),
        ];
        for (name, source) in [
            "right.cue",
            "r.cue",
            "group.cue",
            "binary.cue",
            "runtime.cue",
            "core.cue",
        ]
        .into_iter()
        .zip([right, r, group, binary, runtime, core])
        {
            if let Some(source) = source {
                inputs.push((name, source));
            }
        }
        inputs.push(("context.cue", context));
        self.exports.set(self.exports.get() + 1);
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
    fn policy(&self, runtime: Option<&str>, context: &str, reverse: bool) -> Output {
        self.full(
            BRIDGE,
            self.owners(runtime),
            Some(CORE),
            context,
            reverse,
            Mode::Accepted,
        )
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}
// Verification-only inverse. No selected output or copied full bridge is a source seed.
fn old_bridge() -> String {
    let (before, shape) = BRIDGE
        .split_once("#InstructionBinaryRuntimeShape:")
        .expect("shape");
    let (_, after) = shape.split_once("// Executable recursion").expect("root");
    let owner = RUNTIME
        .strip_prefix("package bootstrap\n\n#InstructionBinaryRuntime: {\n")
        .expect("owner")
        .strip_suffix("}\n")
        .expect("end");
    let (lets, body) = owner.split_once("\top: ").expect("concat");
    let body = body
        .trim_end()
        .replace("\n\t\t", "\n\t\t\t")
        .replace("\n\t]", "\n\t\t]}");
    let bridge = format!("{before}// Executable recursion{after}")
        .replace(
            "#InstructionLowering: #LoweringPolicy & {\n",
            &format!("#InstructionLowering: #LoweringPolicy & {{\n{lets}"),
        )
        .replace(
            "\t\tbinaryRuntime:    #InstructionBinaryRuntime",
            &format!("\t\tbinaryRuntime: {{op: {body}"),
        );
    let mut in_cases = false;
    bridge
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
        + "\n"
}
fn inventory(
    value: &Value,
    path: &str,
    strings: &mut Vec<String>,
    arrays: &mut Vec<String>,
    objects: &mut Vec<String>,
    members: &mut Vec<String>,
) {
    match value {
        Value::Object(fields) => {
            objects.push(path.into());
            for (key, child) in fields {
                let path = format!("{path}/{key}");
                members.push(path.clone());
                inventory(child, &path, strings, arrays, objects, members);
            }
        }
        Value::Array(items) => {
            arrays.push(path.into());
            for (index, child) in items.iter().enumerate() {
                inventory(
                    child,
                    &format!("{path}/{index}"),
                    strings,
                    arrays,
                    objects,
                    members,
                );
            }
        }
        Value::String(_) => strings.push(path.into()),
        _ => panic!("independent tree has ONLY strings/closed objects/fixed arrays"),
    }
}
fn remove(value: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').expect("member path");
    value
        .pointer_mut(parent)
        .expect("parent")
        .as_object_mut()
        .expect("object")
        .remove(key)
        .expect("required member");
}
fn context(mask: usize, field: &str) -> String {
    let parts = SITES
        .into_iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, path)| {
            let mut value = format!("\"{field}\"");
            let components = path
                .trim_start_matches("/cases/")
                .split('/')
                .collect::<Vec<_>>();
            for (position, part) in components.iter().enumerate().rev() {
                if let Ok(index) = part.parse::<usize>() {
                    let length = match components[position - 1] {
                        "parts" if index == 4 => 6,
                        "parts" => 3,
                        "args" => 2,
                        _ => panic!("independently specified array path"),
                    };
                    let mut items = vec!["_".to_owned(); length];
                    items[index] = value;
                    value = format!("[{}]", items.join(","));
                } else {
                    value = format!("{{{part}:{value}}}");
                }
            }
            value
        })
        .collect::<Vec<_>>();
    let cases = if parts.is_empty() {
        "{}".into()
    } else {
        parts.join(" & ")
    };
    format!("package bootstrap\n#InstructionLowering & {{cases:{cases}}}\n")
}

#[test]
fn whole_handwritten_tree_orders_repeats_names_and_source_effects() {
    let p = Probe::new();
    let want = expected("left", "right", "");
    let baseline = decoded(p.policy(Some(RUNTIME), SELECTOR, false));
    for reverse in [false, true] {
        for _repeat in 0..2 {
            assert_eq!(
                decoded(p.isolated(BRIDGE, p.owners(Some(RUNTIME)), HELPER, "export", reverse)),
                want
            );
            assert_eq!(
                decoded(p.policy(Some(RUNTIME), SELECTOR, reverse)),
                baseline
            );
        }
        let bridge = BRIDGE.replace("InstructionBinaryRuntime", "DifferentRuntime");
        let owner = RUNTIME
            .replace("InstructionBinaryRuntime", "DifferentRuntime")
            .replace("let L =", "let OtherLeft =")
            .replace("let R =", "let OtherRight =")
            .replace("}, L,", "}, OtherLeft,")
            .replace("}, R,", "}, OtherRight,");
        let inputs = [
            ("schema.cue", SCHEMA),
            ("unrelated-bridge.cue", bridge.as_str()),
            ("unrelated-owner.cue", owner.as_str()),
            ("right.cue", RIGHT),
            ("r.cue", R),
            ("group.cue", GROUP),
            ("binary.cue", p.binary.as_str()),
        ];
        assert_eq!(
            decoded(p.run(
                &p.cue,
                &["export", "-e", "#DifferentRuntime"],
                &inputs,
                reverse
            )),
            want
        );
        for (needle, replacement, index, answer) in [
            (
                "#InstructionOperandCases.leftOperand",
                json!(wrapped("right")).to_string(),
                2,
                wrapped("right"),
            ),
            (
                "#InstructionRightOperand",
                wrapped("left").to_string(),
                4,
                wrapped("left"),
            ),
            (
                "value: \"(\"",
                "value: \"[\"".into(),
                1,
                json!({"op":"text","value":"["}),
            ),
        ] {
            let owner = RUNTIME.replace(needle, &replacement);
            let mut changed = want.clone();
            changed["parts"][index] = answer;
            let mut full = baseline.clone();
            full["cases"]["binaryRuntime"] = changed;
            assert_eq!(
                decoded(p.policy(Some(&owner), SELECTOR, reverse)),
                full,
                "whole policy/siblings"
            );
        }
        let binary = p
            .binary_provider(&BINARY.replace("runtime::\\(method)", "runtime::changed_\\(method)"));
        let mut owners = p.owners(Some(RUNTIME));
        owners[3] = Some(&binary);
        let mut full = baseline.clone();
        full["cases"]["binaryRuntime"] = expected("left", "right", "changed_");
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                owners,
                Some(CORE),
                SELECTOR,
                reverse,
                Mode::Accepted
            )),
            full
        );
    }
}

#[test]
fn all_required_members_types_closed_objects_and_fixed_array_phases() {
    let p = Probe::new();
    let want = expected("left", "right", "");
    let (mut strings, mut arrays, mut objects, mut members) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    inventory(
        &want,
        "",
        &mut strings,
        &mut arrays,
        &mut objects,
        &mut members,
    );
    assert_eq!(
        (strings.len(), arrays.len(), objects.len(), members.len()),
        (39, 1, 23, 56)
    );
    assert_eq!(want["parts"].as_array().expect("fixed six").len(), 6);
    let mut missing = vec![None, Some(placed(&json!({})))];
    for path in &members {
        let mut value = want.clone();
        remove(&mut value, path);
        missing.push(Some(placed(&value)));
    }
    for reverse in [false, true] {
        for owner in &missing {
            let owners = p.owners(owner.as_deref());
            assert!(
                p.isolated(BRIDGE, owners, HELPER, "eval", reverse)
                    .status
                    .success(),
                "incomplete eval"
            );
            let out = p.isolated(BRIDGE, owners, HELPER, "export", reverse);
            failed(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete value"));
            let before = p.exports.get();
            failed(&p.policy(owner.as_deref(), SELECTOR, reverse));
            assert_eq!(p.exports.get(), before, "stop before full export");
        }
        let mut invalid = Vec::new();
        for path in &strings {
            let mut value = want.clone();
            *value.pointer_mut(path).expect("scalar") = json!(7);
            invalid.push(value);
        }
        for path in &objects {
            let mut value = want.clone();
            value
                .pointer_mut(path)
                .expect("object")
                .as_object_mut()
                .expect("closed object")
                .insert("extra".into(), json!(true));
            invalid.push(value);
        }
        for index in 0..6 {
            let mut value = want.clone();
            value["parts"].as_array_mut().expect("parts").remove(index);
            invalid.push(value);
        }
        let mut long = want.clone();
        long["parts"]
            .as_array_mut()
            .expect("parts")
            .push(json!({"op":"text","value":"extra"}));
        invalid.push(long);
        for shape in [json!({}), json!("wrong"), json!([])] {
            let mut value = want.clone();
            value["parts"] = shape;
            invalid.push(value);
        }
        for value in invalid {
            let owner = placed(&value);
            let owners = p.owners(Some(&owner));
            for op in ["eval", "export"] {
                failed(&p.isolated(BRIDGE, owners, HELPER, op, reverse));
            }
            let before = p.exports.get();
            failed(&p.policy(Some(&owner), SELECTOR, reverse));
            assert_eq!(p.exports.get(), before);
        }
        assert_eq!(
            decoded(p.isolated(BRIDGE, p.owners(Some(RUNTIME)), HELPER, "export", reverse)),
            want,
            "valid recovery"
        );
    }
}

#[test]
fn labelled_unique_masks_full_old_policies_all_cursors_and_raw_selection_conflicts() {
    let p = Probe::new();
    let old = old_bridge();
    let masks = [
        0, 511, 1, 2, 4, 8, 16, 32, 64, 128, 256, 170, 341, 255, 7, 85,
    ];
    assert_eq!(masks.len(), 16);
    assert_eq!(
        masks.into_iter().collect::<BTreeSet<_>>().len(),
        16,
        "UNIQUE masks"
    );
    let mut documents = BTreeSet::new();
    let mut compared = 0;
    for right in [
        RIGHT.to_owned(),
        RIGHT.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
    ] {
        for mask in masks {
            for answer in ["left", "right"] {
                let context = context(mask, answer);
                documents.insert(context.clone());
                for reverse in [false, true] {
                    let mut owners = p.owners(Some(RUNTIME));
                    owners[0] = Some(&right);
                    let current = decoded(p.full(
                        BRIDGE,
                        owners,
                        Some(CORE),
                        &context,
                        reverse,
                        Mode::Accepted,
                    ));
                    owners[4] = None;
                    let original =
                        decoded(p.full(&old, owners, Some(CORE), &context, reverse, Mode::Legacy));
                    assert_eq!(
                        current, original,
                        "FULL old/relocated control {mask}/{answer}/{reverse}"
                    );
                    for (i, path) in SITES.into_iter().enumerate() {
                        let default = if right == RIGHT { "right" } else { "left" };
                        assert_eq!(
                            current.pointer(path).expect("physical site"),
                            if mask & (1 << i) != 0 {
                                answer
                            } else {
                                default
                            }
                        );
                    }
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 128);
    assert_eq!(
        documents.len(),
        31,
        "zero-mask baseline intentionally repeated across two labelled answers"
    );
    for right in [
        RIGHT.to_owned(),
        RIGHT.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
    ] {
        for first in ["firstRule", "lastRule"] {
            for step in ["nextRule", "previousRule"] {
                for twice in [false, true] {
                    let sum = if twice {
                        "cases:{sum:{args:[_,{value:{value:{field:\"left\"}}}]}},"
                    } else {
                        ""
                    };
                    let context = format!(
                        "package bootstrap\n#InstructionLowering & {{{sum}module:[_,{{op:\"{first}\"}},{{body:[_,_,_,_,{{op:\"{step}\"}}]}},_]}}\n"
                    );
                    for reverse in [false, true] {
                        let mut owners = p.owners(Some(RUNTIME));
                        owners[0] = Some(&right);
                        let current = decoded(p.full(
                            BRIDGE,
                            owners,
                            Some(CORE),
                            &context,
                            reverse,
                            Mode::Accepted,
                        ));
                        owners[4] = None;
                        assert_eq!(
                            current,
                            decoded(p.full(
                                &old,
                                owners,
                                Some(CORE),
                                &context,
                                reverse,
                                Mode::Legacy
                            ))
                        );
                        assert_eq!(current["module"][1]["op"], first);
                        assert_eq!(current["module"][2]["body"][4]["op"], step);
                    }
                }
            }
        }
    }
    for reverse in [false, true] {
        let selected = placed(&decoded(p.isolated(
            BRIDGE,
            p.owners(Some(RUNTIME)),
            HELPER,
            "export",
            reverse,
        )));
        let ctx = context(1, "left");
        let raw = decoded(p.policy(Some(RUNTIME), &ctx, reverse));
        assert_eq!(raw["cases"]["binaryRuntime"]["parts"][4], wrapped("left"));
        for path in &SITES[1..] {
            assert_eq!(raw.pointer(path).expect("unrelated sibling"), "right");
        }
        failed(&p.policy(Some(&selected), &ctx, reverse));
        assert_eq!(
            decoded(p.policy(Some(RUNTIME), SELECTOR, reverse))["cases"]["binaryRuntime"],
            expected("left", "right", "")
        );
    }
}

#[test]
fn true_backfills_explicit_dependency_modes_and_actual_shell_fail_stop_recovery() {
    let p = Probe::new();
    let want = expected("left", "right", "");
    let baseline = decoded(p.policy(Some(RUNTIME), SELECTOR, false));
    let mut missing = want.clone();
    remove(&mut missing, "/op");
    let missing = placed(&missing);
    let whole =
        format!("package bootstrap\n#InstructionLowering & {{cases:{{binaryRuntime:{want}}}}}\n");
    for reverse in [false, true] {
        for (owner, ctx) in [(Some(missing.as_str()), SELECTOR), (None, whole.as_str())] {
            let before = p.exports.get();
            failed(&p.policy(owner, ctx, reverse));
            assert_eq!(p.exports.get(), before);
            assert_eq!(
                decoded(p.full(
                    BRIDGE,
                    p.owners(owner),
                    Some(CORE),
                    ctx,
                    reverse,
                    Mode::MissingRuntime
                )),
                baseline,
                "genuine backfill"
            );
        }
        let all = context(511, "left");
        let left = wrapped("left");
        let whole_r = format!(
            "package bootstrap\n#InstructionLowering & {{cases:{{
                binaryRuntime:{{parts:[_,_,_,_,{left},_]}},
                sum:{{args:[_,{left}]}},less:{{args:[_,{left}]}},
                lessEqual:{{args:[_,{left}]}},greater:{{args:[_,{left}]}},
                greaterEqual:{{args:[_,{left}]}},notEqual:{{args:[_,{left}]}},
                remainder:{{args:[_,{left}]}},unify:{{parts:[_,{{args:[_,{left}]}},_]}}
            }}}}\n"
        );
        for (index, mode, ctx) in [
            (0, Mode::MissingRight, all.as_str()),
            (1, Mode::MissingR, whole_r.as_str()),
        ] {
            let mut owners = p.owners(Some(RUNTIME));
            owners[index] = None;
            if index == 1 {
                let field_only = p.full(BRIDGE, owners, Some(CORE), &all, reverse, mode);
                failed(&field_only);
                assert!(String::from_utf8_lossy(&field_only.stderr).contains("incomplete value"));
            }
            let full = decoded(p.full(BRIDGE, owners, Some(CORE), ctx, reverse, mode));
            for path in SITES {
                let operand = path
                    .strip_suffix("/value/value/field")
                    .expect("whole wrapper path");
                assert_eq!(
                    full.pointer(operand).expect("COMPLETE backfilled wrapper"),
                    &left
                );
            }
            failed(&p.full(BRIDGE, owners, Some(CORE), ctx, reverse, Mode::Accepted));
        }
        let group = json!({"leftOperand":wrapped("left"),"cases":{
            "sum":{"op":"call","path":"runtime::sum","args":[wrapped("left"),wrapped("right")]},
            "less":{"op":"call","path":"runtime::less","args":[wrapped("left"),wrapped("right")]},
            "lessEqual":{"op":"call","path":"runtime::less_equal","args":[wrapped("left"),wrapped("right")]},
            "greater":{"op":"call","path":"runtime::greater","args":[wrapped("left"),wrapped("right")]},
            "greaterEqual":{"op":"call","path":"runtime::greater_equal","args":[wrapped("left"),wrapped("right")]},
            "notEqual":{"op":"call","path":"runtime::not_equal","args":[wrapped("left"),wrapped("right")]},
            "remainder":{"op":"call","path":"runtime::remainder","args":[wrapped("left"),wrapped("right")]},
            "unify":{"op":"concat","parts":[{"op":"text","value":"(|_context: &mut runtime::EvaluationContext| "},{"op":"call","path":"runtime::unify","args":[wrapped("left"),wrapped("right")]},{"op":"text","value":")(_context)"}]}
        }});
        let ctx =
            format!("package bootstrap\n#InstructionOperandCases:{group}\n#InstructionLowering\n");
        let mut owners = p.owners(Some(RUNTIME));
        owners[2] = None;
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                owners,
                Some(CORE),
                &ctx,
                reverse,
                Mode::MissingGroup
            )),
            baseline
        );
        failed(&p.full(BRIDGE, owners, Some(CORE), &ctx, reverse, Mode::Accepted));
        let binary = format!(
            "package bootstrap\n#InstructionBinaryTables:{{aliases:{{index:\"binaryRuntime\",difference:\"binaryRuntime\",product:\"binaryRuntime\",divide:\"binaryRuntime\",modulo:\"binaryRuntime\",quotient:\"binaryRuntime\",recordIndex:\"binaryRuntime\",listIndex:\"binaryRuntime\",logicalAnd:\"binaryRuntime\",logicalOr:\"binaryRuntime\",equal:\"binaryRuntime\"}},select:{}}}\n#InstructionLowering\n",
            want["parts"][0]
        );
        owners = p.owners(Some(RUNTIME));
        owners[3] = None;
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                owners,
                Some(CORE),
                &binary,
                reverse,
                Mode::MissingBinary
            )),
            baseline
        );
        failed(&p.full(BRIDGE, owners, Some(CORE), &binary, reverse, Mode::Accepted));
        let bad_core = CORE.replace(
            "op: \"call\", path: \"runtime::text\"",
            "path: \"runtime::text\"",
        );
        assert_ne!(bad_core, CORE, "actual missing Core opcode fixture");
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                p.owners(Some(RUNTIME)),
                Some(&bad_core),
                SELECTOR,
                reverse,
                Mode::MissingCore
            )),
            baseline
        );
        failed(&p.full(
            BRIDGE,
            p.owners(Some(RUNTIME)),
            Some(&bad_core),
            SELECTOR,
            reverse,
            Mode::Accepted,
        ));
        let mut bad = want.clone();
        bad["op"] = json!("unknownRuntimeOpcode");
        let owner = placed(&bad);
        assert!(
            p.isolated(BRIDGE, p.owners(Some(&owner)), HELPER, "export", reverse)
                .status
                .success(),
            "type-only helper gives no opcode answer"
        );
        failed(&p.policy(Some(&owner), SELECTOR, reverse));
        let mut long = want.clone();
        long["parts"][4] = wrapped(&"x".repeat(129));
        assert!(
            p.policy(Some(&placed(&long)), SELECTOR, reverse)
                .status
                .success(),
            "Go accepts129; OLD real typed exit2 proof stays Nix-owned"
        );
    }
    decoded(p.policy(Some(RUNTIME), &whole, false));
    fs::write(p.dir.join("runtime.cue"), &missing).expect("incomplete runtime fixture");
    let script = format!(
        r#"
consume() {{
  '{cue}' export schema.cue bridge.cue right.cue -e '#InstructionRightField' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue r.cue -e '#InstructionRightOperand' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue r.cue group.cue -e '#InstructionOperandCases' >/dev/null || return
  '{cue}' export schema.cue bridge.cue binary.cue -e '#InstructionBinaryTables' >/dev/null || return
  '{cue}' export schema.cue bridge.cue right.cue r.cue group.cue binary.cue runtime.cue -e '#InstructionBinaryRuntime' >/dev/null || return
  '{cue}' export shape.cue core.cue >/dev/null || return
  : > full-export-started
  '{cue}' export schema.cue bridge.cue right.cue r.cue group.cue binary.cue runtime.cue core.cue unary.cue text.cue first.cue step.cue context.cue
}}
case "$1" in
 normal) consume ;;
 conditional) if consume; then exit 99; else exit 17; fi ;;
 disjunction) consume || exit 17 ;;
esac
"#,
        cue = p.cue.display()
    );
    fs::write(p.dir.join("consume.sh"), script).expect("actual pipeline");
    for fixture in [missing, placed(&json!({}))] {
        fs::write(p.dir.join("runtime.cue"), fixture).expect("missing opcode or WHOLE helper");
        for mode in ["normal", "conditional", "disjunction"] {
            let out = p.run(&p.shell, &["consume.sh", mode], &[], false);
            assert_eq!(
                out.status.code(),
                Some(if mode == "normal" { 1 } else { 17 })
            );
            assert!(out.stdout.is_empty() && !out.stderr.is_empty());
            assert!(!p.dir.join("full-export-started").exists());
        }
    }
    fs::write(p.dir.join("runtime.cue"), RUNTIME).expect("valid recovery source");
    assert_eq!(
        decoded(p.run(&p.shell, &["consume.sh", "normal"], &[], false)),
        baseline
    );
    assert!(p.dir.join("full-export-started").is_file());
}
