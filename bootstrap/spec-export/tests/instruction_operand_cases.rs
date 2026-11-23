use serde_json::{Value, json};
use std::{
    cell::Cell,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const GROUP: &str = include_str!("../../instruction-operand-cases.cue");
const BRIDGE: &str = include_str!("../../lowering-fragments.cue");
const MODULE: &str = include_str!("../../instruction-module.cue");
const MODULE_SHAPE: &str = include_str!("../../instruction-module-schema.cue");
const RUNTIME: &str = include_str!("../../instruction-binary-runtime.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const RIGHT: &str = include_str!("../../instruction-right-field.cue");
const OPERAND: &str = include_str!("../../instruction-right-operand.cue");
const CORE: &str = include_str!("../../instruction-core-cases.cue");
const SHAPE: &str = include_str!("../../instruction-core-cases-schema.cue");
const BINARY: &str = include_str!("../../instruction-binary-tables.cue");
const PACKAGE: &str = include_str!("../../native-policy-package.cue");
const UNARY: &str = include_str!("../../instruction-unary-tables.cue");
const TEXT: &str = include_str!("../../instruction-module-text.cue");
const FIRST: &str = include_str!("../../instruction-module-first.cue");
const STEP: &str = include_str!("../../instruction-module-step.cue");
const SELECTOR: &str = include_str!("../../lowering.cue");
const TWICE: &str = include_str!("../../tests/valid/lowering-left-twice.cue");
const REVERSE: &str = include_str!("../../tests/valid/lowering-reverse.cue");
const HELPER: &str = "#InstructionOperandCases";
const NAMES: [&str; 8] = [
    "sum",
    "less",
    "lessEqual",
    "greater",
    "greaterEqual",
    "notEqual",
    "remainder",
    "unify",
];
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
fn wrap(field: &str) -> Value {
    json!({"op":"try","value":{"op":"group","value":{"op":"lower","field":field}}})
}
fn expected(left: &str, right: &str, sum: &str) -> Value {
    let mut cases = serde_json::Map::new();
    for (name, method) in [
        ("sum", sum),
        ("less", "runtime::less"),
        ("lessEqual", "runtime::less_equal"),
        ("greater", "runtime::greater"),
        ("greaterEqual", "runtime::greater_equal"),
        ("notEqual", "runtime::not_equal"),
        ("remainder", "runtime::remainder"),
    ] {
        cases.insert(
            name.into(),
            json!({"op":"call","path":method,"args":[wrap(left),wrap(right)]}),
        );
    }
    cases.insert(
        "unify".into(),
        json!({"op":"concat","parts":[
        {"op":"text","value":"(|_context: &mut runtime::EvaluationContext| "},
        {"op":"call","path":"runtime::unify","args":[wrap(left),wrap(right)]},
        {"op":"text","value":")(_context)"}]}),
    );
    json!({"leftOperand":wrap(left),"cases":cases})
}
fn source(body: &str) -> String {
    format!("package bootstrap\n{body}\n")
}
fn placed(value: &Value) -> String {
    source(&format!("{HELPER}: {value}"))
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
    assert!(meta.is_file());
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
}
#[derive(Clone, Copy)]
enum Mode {
    Accepted,
    MissingRight,
    MissingOperand,
    BypassGroup,
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
            "operand-cases-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let discover = |name| {
            let path = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory tool")
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
        let data = decoded(p.run(
            &p.cue,
            &["export"],
            &[("package.cue", PACKAGE), ("binary.cue", BINARY)],
            false,
        ));
        let imported = p.run(
            &p.cue,
            &[
                "import",
                "-p",
                "bootstrap",
                "-l",
                "#InstructionBinaryTables:",
                "--outfile",
                "-",
            ],
            &[("binary.json", &data.to_string())],
            false,
        );
        assert!(
            imported.status.success(),
            "Go import setup: {}",
            String::from_utf8_lossy(&imported.stderr)
        );
        p.binary = String::from_utf8(imported.stdout).expect("Go import");
        assert!(!p.binary.is_empty(), "independent Binary oracle setup");
        p
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, text) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, text).expect("explicit verification input");
            paths.push(path);
        }
        if reverse {
            paths.reverse();
        }
        let out = Command::new(tool)
            .args(args)
            .args(paths)
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .output()
            .expect("process setup");
        if tool == self.cue && !out.status.success() {
            assert_eq!(out.status.code(), Some(1), "Go failure phase: {out:?}");
        }
        out
    }
    fn isolated(
        &self,
        providers: [Option<&str>; 3],
        helper: &str,
        op: &str,
        reverse: bool,
    ) -> Output {
        let mut inputs = vec![("schema.cue", SCHEMA), ("bridge.cue", BRIDGE)];
        for (name, value) in ["right.cue", "operand.cue", "group.cue"]
            .into_iter()
            .zip(providers)
        {
            if let Some(value) = value {
                inputs.push((name, value));
            }
        }
        self.run(&self.cue, &[op, "-e", helper], &inputs, reverse)
    }
    fn full(
        &self,
        bridge: &str,
        providers: [Option<&str>; 3],
        context: &str,
        reverse: bool,
        mode: Mode,
    ) -> Output {
        let [right, operand, group] = providers;
        let stages = [
            (
                "#InstructionRightField",
                [right, None, None],
                !matches!(mode, Mode::MissingRight),
            ),
            (
                "#InstructionRightOperand",
                [right, operand, None],
                !matches!(mode, Mode::MissingRight | Mode::MissingOperand),
            ),
            (HELPER, providers, matches!(mode, Mode::Accepted)),
        ];
        for (helper, owners, enabled) in stages {
            if enabled {
                let out = self.isolated(owners, helper, "export", reverse);
                if !out.status.success() {
                    return out;
                }
            }
        }
        let core = self.run(
            &self.cue,
            &["export"],
            &[("shape.cue", SHAPE), ("core.cue", CORE)],
            reverse,
        );
        assert!(core.status.success(), "independent Core118 setup");
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
        assert!(binary.status.success(), "independent Binary35 setup");
        if matches!(mode, Mode::Accepted) {
            let gate = self.run(
                &self.cue,
                &["export", "-e", "#InstructionBinaryRuntime"],
                &[
                    ("schema.cue", SCHEMA),
                    ("bridge.cue", bridge),
                    ("right.cue", right.expect("accepted Right")),
                    ("operand.cue", operand.expect("accepted R")),
                    ("group.cue", group.expect("accepted Group")),
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
        for (name, owner) in ["right.cue", "operand.cue", "group.cue"]
            .into_iter()
            .zip([right, operand, group])
        {
            if let Some(owner) = owner {
                inputs.push((name, owner));
            }
        }
        if !matches!(mode, Mode::Legacy) {
            inputs.push(("module.cue", MODULE));
            inputs.push(("runtime.cue", RUNTIME));
        }
        inputs.push(("context.cue", context));
        self.exports.set(self.exports.get() + 1);
        self.run(&self.cue, &["export"], &inputs, reverse)
    }
    fn policy(&self, owners: [Option<&str>; 3], context: &str, reverse: bool) -> Output {
        self.full(BRIDGE, owners, context, reverse, Mode::Accepted)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}
fn inventory(
    value: &Value,
    path: &str,
    leaves: &mut Vec<String>,
    arrays: &mut Vec<String>,
    objects: &mut Vec<String>,
) {
    match value {
        Value::Object(fields) => {
            objects.push(path.into());
            for (key, value) in fields {
                inventory(value, &format!("{path}/{key}"), leaves, arrays, objects);
            }
        }
        Value::Array(items) => {
            arrays.push(path.into());
            for (index, value) in items.iter().enumerate() {
                inventory(value, &format!("{path}/{index}"), leaves, arrays, objects);
            }
        }
        Value::String(_) => leaves.push(path.into()),
        _ => panic!("ONLY independently authored string leaves"),
    }
}
fn remove(value: &mut Value, path: &str) {
    let (parent, key) = path.rsplit_once('/').expect("member path");
    value
        .pointer_mut(parent)
        .expect("parent")
        .as_object_mut()
        .expect("object")
        .remove(key)
        .expect("required member");
}
fn old_bridge() -> String {
    let module = MODULE
        .strip_prefix("package bootstrap\n\n#InstructionModule: {\n")
        .expect("original Module owner")
        .strip_suffix("}\n")
        .expect("owner end");
    let bridge = BRIDGE
        .replace("#InstructionModule: close({module: _})\n\n", "")
        .replace("\tmodule: #InstructionModule.module\n", module);
    // Source-only inverse BEFORE original-policy comparisons; never construction.
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
    let (lets, body) = runtime.split_once("\top: ").expect("original concat");
    let body = body
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
            &format!("\t\tbinaryRuntime: {{op: {body}"),
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
    let mut bridge = format!("{before}// Executable recursion{after}")
        .replace("\t\t#InstructionOperandCases.cases\n", "")
        .replace("\tlet L = #InstructionOperandCases.leftOperand", left);
    let cases = GROUP
        .split_once("\tcases: {\n")
        .expect("original cases")
        .1
        .strip_suffix("\t}\n}\n")
        .expect("owner end");
    for (index, name) in NAMES.iter().enumerate() {
        let prefix = format!("\t\t{name}:");
        let start = cases.find(&prefix).expect("body");
        let end = NAMES.get(index + 1).map_or(cases.len(), |next| {
            cases.find(&format!("\t\t{next}:")).expect("next body")
        });
        let slot = bridge
            .lines()
            .find(|line| line.starts_with(&prefix))
            .expect("required slot")
            .to_owned();
        bridge = bridge.replace(&format!("{slot}\n"), &cases[start..end]);
    }
    bridge
}
fn context(mask: usize, field: &str, complete: bool) -> String {
    let leaf = if complete {
        wrap(field)
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
fn whole_context() -> String {
    let cases = &expected("left", "right", "runtime::sum")["cases"];
    source(&format!(
        "#InstructionLowering & {{cases:{cases} & {{binaryRuntime:{{parts:[_,_,{},_,{},_]}}}}}}",
        wrap("left"),
        wrap("right")
    ))
}

#[test]
fn complete_handwritten_tree_raw_orders_repeats_names_and_source_effects() {
    let p = Probe::new();
    let want = expected("left", "right", "runtime::sum");
    let (mut leaves, mut arrays, mut objects) = (Vec::new(), Vec::new(), Vec::new());
    inventory(&want, "", &mut leaves, &mut arrays, &mut objects);
    assert_eq!((leaves.len(), arrays.len(), objects.len()), (89, 9, 64));
    for reverse in [false, true] {
        for _ in 0..2 {
            assert_eq!(
                decoded(p.isolated(
                    [Some(RIGHT), Some(OPERAND), Some(GROUP)],
                    HELPER,
                    "export",
                    reverse
                )),
                want
            );
        }
        let rename = |text: &str| {
            text.replace(HELPER, "#IndependentGroup")
                .replace("#InstructionRightOperand", "#OtherOperand")
                .replace("let L =", "let LocalLeft =")
                .replace("let R =", "let LocalRight =")
                .replace("leftOperand: L", "leftOperand: LocalLeft")
                .replace("[L, R]", "[LocalLeft, LocalRight]")
        };
        let root = source("#IndependentGroup");
        assert_eq!(
            decoded(p.run(
                &p.cue,
                &["export"],
                &[
                    ("right.cue", RIGHT),
                    ("renamed-r.cue", &rename(OPERAND)),
                    ("renamed-group.cue", &rename(GROUP)),
                    ("root.cue", &root)
                ],
                reverse
            )),
            want
        );
        for (owner, tree) in [
            (
                GROUP.replace("field: \"left\"", "field: \"right\""),
                expected("right", "right", "runtime::sum"),
            ),
            (
                GROUP.replace("runtime::sum", "runtime::changed_sum"),
                expected("left", "right", "runtime::changed_sum"),
            ),
        ] {
            assert_eq!(
                decoded(p.isolated(
                    [Some(RIGHT), Some(OPERAND), Some(&owner)],
                    HELPER,
                    "export",
                    reverse
                )),
                tree
            );
            let policy = decoded(p.policy(
                [Some(RIGHT), Some(OPERAND), Some(&owner)],
                SELECTOR,
                reverse,
            ));
            let mut predicted =
                decoded(p.policy([Some(RIGHT), Some(OPERAND), Some(GROUP)], SELECTOR, reverse));
            for name in NAMES {
                predicted["cases"][name] = tree["cases"][name].clone();
            }
            predicted["cases"]["binaryRuntime"]["parts"][2] = tree["leftOperand"].clone();
            assert_eq!(policy, predicted, "whole-policy source effect");
        }
    }
}

#[test]
fn all_independent_omissions_types_objects_and_nine_fixed_arrays() {
    let p = Probe::new();
    let tree = expected("left", "right", "runtime::sum");
    let (mut leaves, mut arrays, mut objects) = (Vec::new(), Vec::new(), Vec::new());
    inventory(&tree, "", &mut leaves, &mut arrays, &mut objects);
    assert_eq!((leaves.len(), arrays.len(), objects.len()), (89, 9, 64));
    let mut omissions = leaves.clone();
    omissions.extend(NAMES.map(|name| format!("/cases/{name}")));
    omissions.extend(["/leftOperand".into(), "/cases".into()]);
    let mut providers = vec![None, Some(placed(&json!({})))];
    for path in omissions {
        let mut missing = tree.clone();
        remove(&mut missing, &path);
        providers.push(Some(placed(&missing)));
    }
    for group in providers {
        let owners = [Some(RIGHT), Some(OPERAND), group.as_deref()];
        assert!(p.isolated(owners, HELPER, "eval", false).status.success());
        let out = p.isolated(owners, HELPER, "export", true);
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete value"));
        let before = p.exports.get();
        failed(&p.policy(owners, &whole_context(), false));
        assert_eq!(
            p.exports.get(),
            before,
            "no full export after failed Group gate"
        );
    }
    for path in leaves {
        let mut wrong = tree.clone();
        *wrong.pointer_mut(&path).expect("leaf") = json!(42);
        let out = p.isolated(
            [Some(RIGHT), Some(OPERAND), Some(&placed(&wrong))],
            HELPER,
            "export",
            false,
        );
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("conflicting values"));
    }
    for path in objects {
        let mut wrong = tree.clone();
        wrong
            .pointer_mut(&path)
            .expect("object")
            .as_object_mut()
            .expect("map")
            .insert("extra".into(), json!("x"));
        let out = p.isolated(
            [Some(RIGHT), Some(OPERAND), Some(&placed(&wrong))],
            HELPER,
            "export",
            false,
        );
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("field not allowed"));
    }
    for path in arrays {
        for longer in [false, true] {
            let mut wrong = tree.clone();
            let items = wrong
                .pointer_mut(&path)
                .expect("array")
                .as_array_mut()
                .expect("items");
            if longer {
                items.push(items[0].clone());
            } else {
                items.pop().expect("fixed item");
            }
            let out = p.isolated(
                [Some(RIGHT), Some(OPERAND), Some(&placed(&wrong))],
                HELPER,
                "export",
                true,
            );
            failed(&out);
            let error = String::from_utf8_lossy(&out.stderr);
            assert!(
                error.contains("incompatible list lengths"),
                "{path}: {error}"
            );
            assert!(!error.contains("incomplete value"));
        }
    }
}

#[test]
fn nine_sites_both_priorities_orders_complete_old_policies_and_selected_conflicts() {
    let p = Probe::new();
    let old = old_bridge();
    for reverse in [false, true] {
        for right in [
            RIGHT.to_owned(),
            RIGHT.replace("*\"right\" | \"left\"", "\"right\" | *\"left\""),
        ] {
            let (default, forced) = if right == RIGHT {
                ("right", "left")
            } else {
                ("left", "right")
            };
            let owners = [Some(right.as_str()), Some(OPERAND), Some(GROUP)];
            let baseline = decoded(p.full(
                &old,
                [Some(&right), Some(OPERAND), None],
                SELECTOR,
                reverse,
                Mode::Legacy,
            ));
            for mask in (0..9).map(|i| 1 << i).chain([0, 511, 341, 170]) {
                let ctx = context(mask, forced, false);
                let mut want = baseline.clone();
                for (i, path) in SITES.iter().enumerate() {
                    *want.pointer_mut(path).expect("R site") = wrap(if mask & (1 << i) == 0 {
                        default
                    } else {
                        forced
                    });
                }
                assert_eq!(decoded(p.policy(owners, &ctx, reverse)), want);
                assert_eq!(
                    decoded(p.full(
                        &old,
                        [Some(&right), Some(OPERAND), None],
                        &ctx,
                        reverse,
                        Mode::Legacy
                    )),
                    want
                );
            }
            for ctx in [
                SELECTOR,
                TWICE,
                REVERSE,
                "package bootstrap\n#InstructionLowering & {module:[_,{op:\"lastRule\"},{body:[_,_,_,_,{op:\"nextRule\"}]},_]}\n",
                "package bootstrap\n#InstructionLowering & {module:[_,{op:\"firstRule\"},{body:[_,_,_,_,{op:\"previousRule\"}]},_]}\n",
            ] {
                assert_eq!(
                    decoded(p.policy(owners, ctx, reverse)),
                    decoded(p.full(
                        &old,
                        [Some(&right), Some(OPERAND), None],
                        ctx,
                        reverse,
                        Mode::Legacy
                    ))
                );
            }
        }
        // Selected Go DATA is verification ONLY, never canonical construction.
        let selected = placed(&decoded(p.isolated(
            [Some(RIGHT), Some(OPERAND), Some(GROUP)],
            HELPER,
            "export",
            reverse,
        )));
        assert_eq!(
            decoded(p.policy(
                [Some(RIGHT), Some(OPERAND), Some(&selected)],
                SELECTOR,
                reverse
            )),
            decoded(p.policy([Some(RIGHT), Some(OPERAND), Some(GROUP)], SELECTOR, reverse))
        );
        let out = p.policy(
            [Some(RIGHT), Some(OPERAND), Some(&selected)],
            TWICE,
            reverse,
        );
        failed(&out);
        assert!(String::from_utf8_lossy(&out.stderr).contains("conflicting values"));
    }
}

#[test]
fn real_backfills_dependency_modes_actual_shell_fail_stop_and_recovery() {
    let p = Probe::new();
    let owners = [Some(RIGHT), Some(OPERAND), Some(GROUP)];
    let whole = whole_context();
    let all = context(511, "left", true);
    let baseline = decoded(p.policy(owners, SELECTOR, false));
    let mut missing = expected("left", "right", "runtime::sum");
    remove(&mut missing, "/cases/sum/op");
    let missing = placed(&missing);
    failed(&p.isolated(
        [Some(RIGHT), Some(OPERAND), Some(&missing)],
        HELPER,
        "export",
        false,
    ));
    for reverse in [false, true] {
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                [Some(RIGHT), Some(OPERAND), Some(&missing)],
                SELECTOR,
                reverse,
                Mode::BypassGroup
            )),
            baseline,
            "real opcode backfill"
        );
        assert_eq!(
            decoded(p.full(
                BRIDGE,
                [Some(RIGHT), Some(OPERAND), None],
                &whole,
                reverse,
                Mode::BypassGroup
            )),
            baseline,
            "true WHOLE Group backfill"
        );
        let right_missing = decoded(p.full(
            BRIDGE,
            [None, Some(OPERAND), Some(GROUP)],
            &all,
            reverse,
            Mode::MissingRight,
        ));
        assert_eq!(
            right_missing,
            decoded(p.full(
                BRIDGE,
                [Some(RIGHT), None, Some(GROUP)],
                &all,
                reverse,
                Mode::MissingOperand
            ))
        );
        for path in SITES {
            assert_eq!(
                right_missing.pointer(path).expect("backfilled R"),
                &wrap("left")
            );
        }
        for absent in [
            [None, Some(OPERAND), Some(GROUP)],
            [Some(RIGHT), None, Some(GROUP)],
            [Some(RIGHT), Some(OPERAND), None],
        ] {
            let before = p.exports.get();
            failed(&p.policy(absent, &whole, reverse));
            assert_eq!(p.exports.get(), before);
        }
        let mut wrong = expected("left", "right", "runtime::sum");
        wrong["cases"]["sum"]["op"] = json!("unknownGroupOpcode");
        let wrong = placed(&wrong);
        assert!(
            p.isolated(
                [Some(RIGHT), Some(OPERAND), Some(&wrong)],
                HELPER,
                "export",
                reverse
            )
            .status
            .success(),
            "shape gives no opcode answers"
        );
        failed(&p.policy(
            [Some(RIGHT), Some(OPERAND), Some(&wrong)],
            SELECTOR,
            reverse,
        ));
        assert_eq!(
            decoded(p.policy(owners, SELECTOR, reverse)),
            baseline,
            "fresh valid recovery"
        );
    }
    let script = format!(
        r#"
consume() {{
  '{cue}' export schema.cue bridge.cue $1 -e '#InstructionRightField' >/dev/null || return
  '{cue}' export schema.cue bridge.cue $1 $2 -e '#InstructionRightOperand' >/dev/null || return
  '{cue}' export schema.cue bridge.cue $1 $2 $3 -e '#InstructionOperandCases' >/dev/null || return
  '{cue}' export shape.cue core.cue >/dev/null || return
  '{cue}' export schema.cue bridge.cue binary-placed.cue -e '#InstructionBinaryTables' >/dev/null || return
  '{cue}' export schema.cue bridge.cue $1 $2 $3 binary-placed.cue runtime.cue -e '#InstructionBinaryRuntime' >/dev/null || return
  for helper in '#InstructionModuleText' '#InstructionModuleFirst' '#InstructionModuleStep' '#InstructionModule'; do
    '{cue}' export module-shape.cue module.cue text.cue first.cue step.cue -e "$helper" >/dev/null || return
  done
  : > full-export-started
  '{cue}' export schema.cue bridge.cue core.cue binary-placed.cue unary.cue module.cue text.cue first.cue step.cue $1 $2 $3 runtime.cue context.cue
}}
case "$4" in
  normal) consume "$@" ;;
  conditional) if consume "$@"; then exit 99; else exit 17; fi ;;
  disjunction) consume "$@" || exit 17 ;;
esac
"#,
        cue = p.cue.display()
    );
    fs::write(p.dir.join("consume.sh"), script).expect("actual pipeline");
    decoded(p.policy(owners, &whole, false));
    for providers in [
        ["", "operand.cue", "group.cue"],
        ["right.cue", "", "group.cue"],
        ["right.cue", "operand.cue", ""],
    ] {
        for mode in ["normal", "conditional", "disjunction"] {
            let out = p.run(
                &p.shell,
                &["consume.sh", providers[0], providers[1], providers[2], mode],
                &[],
                false,
            );
            failed(&out);
            let expected_status = if mode == "normal" { 1 } else { 17 };
            assert_eq!(out.status.code(), Some(expected_status), "{mode}: {out:?}");
            assert!(!p.dir.join("full-export-started").exists());
        }
    }
    let good = p.run(
        &p.shell,
        &[
            "consume.sh",
            "right.cue",
            "operand.cue",
            "group.cue",
            "normal",
        ],
        &[],
        false,
    );
    assert_eq!(decoded(good), baseline);
    assert!(p.dir.join("full-export-started").is_file());
}
