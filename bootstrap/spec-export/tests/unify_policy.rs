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
const HELPERS: &str = include_str!("../../unify-lowering-helpers.cue");
const VALUES: &str = include_str!("../../unify-lowering-values.cue");
const CONTROL: &str = include_str!("../../unify-lowering-control.cue");
const DIFFERENCE: &str = include_str!("../../unify-difference-policy.cue");
const DIVIDE: &str = include_str!("../../unify-divide-policy.cue");
const SAME: &str = include_str!("../../unify-same-policy.cue");
const SCHEMA: &str = include_str!("../../schema.cue");
const BRIDGE: &str = include_str!("../../unify-lowering-fragments.cue");
const SELECTOR: &str = include_str!("../../unify-lowering.cue");
const COMPOSE: &str = include_str!("../../compose-policy.jq");
const CONTEXTS: [&str; 3] = [
    include_str!("../../tests/valid/integer-kernel-alternate.cue"),
    include_str!("../../tests/valid/division-kernel-alternate.cue"),
    include_str!("../../tests/valid/unify-lowering-permissive.cue"),
];
const SOURCES: [&str; 5] = [VALUES, CONTROL, DIFFERENCE, DIVIDE, SAME];
const ROOTS: &str = "boolean empty inputEndpoint inputValue integerEndpoint integerPrimitive \
    integerSum integerValue listCount listCountEndpoint listElement listSlice local offset program \
    recordCount recordGet recordHas textConcat textLength unaryFail \
    and booleanIs bounds compare fail hasEndpoint if kindIs letBounds or textCompare same";
const BRANCHES: &str = "negate product modulo quotient remainder difference divide";
const PREFERRED: [&str; 3] = [
    "crate::runtime::integer_difference(&_left, &_right)",
    "crate::runtime::integer_divide(&_left, &_right)",
    "Ok::<_, Error>(_left == _right)",
];
const ALTERNATE: [&str; 3] = [
    "crate::runtime::integer_product(&_left, &_right)",
    "crate::runtime::integer_quotient(&_left, &_right)",
    "Ok::<_, Error>(true)",
];

// Independently authored from the DSL instruction contract. Neither native/Go
// output nor scratch feasibility captures construct any expected instruction.
fn text(value: &str) -> Value {
    json!({"op":"text","value":value})
}
fn field(op: &str, name: &str) -> Value {
    json!({"op":op,"field":name})
}
fn concat(parts: Vec<Value>) -> Value {
    json!({"op":"concat","parts":parts})
}
fn ok(value: Value) -> Value {
    json!({"op":"ok","value":value})
}
fn identifier(name: &str) -> Value {
    json!({"op":"identifier","field":name,"upperFirst":true})
}
fn select(name: &str, cases: Value) -> Value {
    json!({"op":"select","field":name,"cases":cases})
}
fn side() -> Value {
    select(
        "input",
        json!({"left":text("_left"),"right":text("_right"),"third":text("_third")}),
    )
}
fn conditional(name: &str, yes: Value, no: Value) -> Value {
    json!({"op":"ifTrue","field":name,"then":yes,"else":no})
}
fn preference_node(policy: &mut Value, index: usize) -> &mut Value {
    match index {
        0 => &mut policy["cases"]["integerPrimitive"]["cases"]["difference"],
        1 => &mut policy["cases"]["integerPrimitive"]["cases"]["divide"],
        2 => &mut policy["cases"]["same"],
        _ => panic!("three preferences only"),
    }
}
fn expected() -> [Value; 5] {
    let values = json!({"formatVersion":1,"aliases":{"choose":"if"},"cases":{
        "boolean":ok(json!({"op":"call","path":"Value::Boolean","args":[field("boolean","value")]})),
        "empty":text("Ok::<_, Error>(None::<i64>)"),
        "inputEndpoint":ok(concat(vec![text("crate::unification::endpoint(&"),side(),
            text(", crate::unification::Endpoint::"),identifier("endpoint"),text(")")])),
        "inputValue":ok(concat(vec![side(),text(".clone()")])),
        "integerEndpoint":ok(json!({"op":"call","path":"Some","args":[field("signed","value")]})),
        "integerPrimitive":select("operation",json!({
            "negate":text("crate::runtime::integer_negate(&_left)"),
            "product":text("crate::runtime::integer_product(&_left, &_right)"),
            "modulo":text("crate::runtime::integer_modulo(&_left, &_right)"),
            "quotient":text("crate::runtime::integer_quotient(&_left, &_right)"),
            "remainder":text("crate::runtime::integer_remainder(&_left, &_right)") })),
        "integerSum":text("crate::runtime::integer_sum(&_left, &_right)"),
        "integerValue":ok(json!({"op":"call","path":"Value::Integer","args":[field("signed","value")]})),
        "listCount":text("crate::runtime::list_count(&_left)"),
        "listCountEndpoint":text("crate::runtime::list_count(&_left).map(|value| crate::unification::endpoint(&value, crate::unification::Endpoint::Lower))"),
        "listElement":text("crate::runtime::list_at(&_left, &_right)"),
        "listSlice":text("crate::runtime::list_slice_elements(&_left, &_right, &_third)"),
        "local":ok(select("endpoint",json!({"lower":text("_lower"),"upper":text("_upper")}))),
        "offset":concat(vec![text("crate::unification::offset_endpoint(("),field("lower","value"),
            text(")?, "),field("signed","delta"),text(")")]),
        "program":concat(vec![text("// Generated by bootstrap-unify-codegen. Do not edit by hand.\nuse crate::{Error, runtime::Value};\npub fn "),
            json!({"op":"identifier","field":"function","snake":true}),
            conditional("unary",text("(_left: Value) -> Result<Value, Error> {\n"),
                conditional("ternary",text("(_left: Value, _right: Value, _third: Value) -> Result<Value, Error> {\n"),
                    text("(_left: Value, _right: Value) -> Result<Value, Error> {\n"))),
            field("lower","result"),text("\n}\n")]),
        "recordCount":text("crate::runtime::record_count(&_left)"),
        "recordGet":text("crate::runtime::record_get(&_left, &_right)"),
        "recordHas":text("crate::runtime::record_has(&_left, &_right)"),
        "textConcat":text("crate::runtime::text_concat(&_left, &_right)"),
        "textLength":text("crate::runtime::text_length(&_left)"),
        "unaryFail":select("reason",json!({
            "logical":text("Err(Error::Logical)"),"arithmetic":text("Err(Error::Arithmetic)"),
            "incompleteInt":text("Err(Error::Incomplete)"),"incompleteValue":text("Err(Error::IncompleteValue)"),
            "incompleteBool":text("Err(Error::IncompleteBool)"),"incompleteString":text("Err(Error::IncompleteString)"),
            "incompleteBounds":text("Err(Error::IncompleteBounds)"),"integerOutput":text("Err(Error::IntegerOutput)"),
            "boundThreshold":text("Err(Error::BoundThreshold)"),"emptyBound":text("Err(Error::EmptyBound)"),
            "length":text("Err(Error::Length)"),"listLength":text("Err(Error::ListLength)"),
            "recordLength":text("Err(Error::RecordLength)"),"lengthOperand":text("Err(Error::LengthOperand)") }))
    }});
    let control = json!({"cases":{
        "and":ok(concat(vec![text("("),field("lower","left"),text(")? && ("),field("lower","right"),text(")?")])),
        "or":ok(concat(vec![text("("),field("lower","left"),text(")? || ("),field("lower","right"),text(")?")])),
        "booleanIs":concat(vec![text("crate::unification::boolean_is(&"),side(),text(", "),field("boolean","value"),text(")")]),
        "bounds":ok(concat(vec![text("Value::Bounds { lower: ("),field("lower","lower"),text(")?, upper: ("),field("lower","upper"),text(")? }")])),
        "compare":concat(vec![text("crate::unification::compare(("),field("lower","left"),text(")?, ("),field("lower","right"),
            text(")?, crate::unification::Relation::"),identifier("relation"),text(")")]),
        "fail":concat(vec![text("crate::unification::fail(crate::unification::Failure::"),identifier("reason"),text(", &_left, &_right)")]),
        "hasEndpoint":ok(concat(vec![text("("),field("lower","value"),text(")?.is_some()")])),
        "if":concat(vec![text("if ("),field("lower","condition"),text(")? { "),field("lower","then"),text(" } else { "),field("lower","else"),text(" }")]),
        "kindIs":ok(concat(vec![text("crate::unification::kind_is(&"),side(),text(", crate::unification::Kind::"),identifier("kind"),text(")")])),
        "letBounds":concat(vec![text("{ let (_lower, _upper) = (("),field("lower","lower"),text(")?, ("),field("lower","upper"),text(")?); "),field("lower","result"),text(" }")]),
        "textCompare":concat(vec![text("crate::unification::text_compare(&_left, &_right, crate::unification::Relation::"),identifier("relation"),text(")")])
    }});
    [
        values,
        control,
        json!({"cases":{"integerPrimitive":{"cases":{"difference":text(PREFERRED[0])}}}}),
        json!({"cases":{"integerPrimitive":{"cases":{"divide":text(PREFERRED[1])}}}}),
        json!({"cases":{"same":text(PREFERRED[2])}}),
    ]
}
fn full_expected() -> Value {
    // Explicit expected assembly, not the production composer or a generic merge.
    let [mut full, control, _, _, _] = expected();
    for (name, instruction) in control["cases"].as_object().expect("control cases") {
        assert!(
            full["cases"]
                .as_object_mut()
                .expect("values cases")
                .insert(name.clone(), instruction.clone())
                .is_none()
        );
    }
    for (index, preferred) in PREFERRED.into_iter().enumerate() {
        *preference_node(&mut full, index) = text(preferred);
    }
    full
}
fn decode(data: &str) -> Value {
    serde_json::from_str(data).expect("complete JSON")
}
fn diagnostic(output: Output, message: &str) {
    failed(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains(message));
}
fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect()
}
fn orders(prefix: Vec<usize>) -> Vec<Vec<usize>> {
    if prefix.len() == 5 {
        return vec![prefix];
    }
    (0..5)
        .filter(|i| !prefix.contains(i))
        .flat_map(|i| {
            let mut next = prefix.clone();
            next.push(i);
            orders(next)
        })
        .collect()
}
fn parsed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.len() <= 65536);
    serde_json::from_slice(&output.stdout).expect("complete bounded JSON")
}
fn failed(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no partial JSON");
    assert!(!output.stderr.is_empty(), "mandatory diagnostic");
}
fn encoded(output: Output) -> String {
    parsed(&output);
    String::from_utf8(output.stdout).expect("JSON UTF-8")
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
            "unify-policy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned scratch");
        fs::create_dir(dir.join("home")).expect("owned HOME");
        let provider = |name| {
            std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute tool")
        };
        let probe = Self {
            dir,
            native: PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export")),
            cue: provider("cue"),
            jq: provider("jq"),
        };
        for tool in [&probe.native, &probe.cue, &probe.jq] {
            assert!(tool.is_absolute());
            fs::File::open(tool).expect("readable tool");
            assert_ne!(
                fs::metadata(tool)
                    .expect("tool metadata")
                    .permissions()
                    .mode()
                    & 0o111,
                0
            );
        }
        fs::write(probe.dir.join("compose-policy.jq"), COMPOSE).expect("actual composer");
        probe
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        let mut names = Vec::new();
        for (name, source) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, source).expect("explicit input");
            names.push(path);
        }
        if reverse {
            names.reverse();
        }
        Command::new(tool)
            .env_clear()
            .env("HOME", self.dir.join("home"))
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .args(args)
            .args(names)
            .output()
            .expect("mandatory process setup")
    }
    fn native(&self, companion: &str, source: &str, renamed: bool) -> String {
        let names = if renamed {
            ["renamed-companion.cue", "renamed-owner.cue"]
        } else {
            ["companion.cue", "owner.cue"]
        };
        let inputs = [(names[0], companion), (names[1], source)];
        let first = self.run(&self.native, &[], &inputs, false);
        let second = self.run(&self.native, &[], &inputs, true);
        parsed(&second);
        assert_eq!(first.stdout, second.stdout, "repeat/reversed bytes");
        encoded(first)
    }
    fn merge(&self, left: &str, right: &str) -> Output {
        self.run(
            &self.jq,
            &["-s", "-S", "-f", "compose-policy.jq"],
            &[("left.json", left), ("right.json", right)],
            false,
        )
    }
    fn compose(&self, left: &str, right: &str) -> String {
        encoded(self.merge(left, right))
    }
    fn rejects(&self, a: &str, b: &str) {
        failed(&self.run(&self.native, &[], &[("a.cue", a), ("b.cue", b)], false));
    }
    fn go(
        &self,
        providers: [Option<&str>; 6],
        context: Option<&str>,
        operation: &str,
        reverse: bool,
    ) -> Output {
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("selector.cue", SELECTOR),
        ];
        for (name, source) in [
            "helpers.cue",
            "values.cue",
            "control.cue",
            "difference.cue",
            "divide.cue",
            "same.cue",
        ]
        .into_iter()
        .zip(providers)
        {
            if let Some(source) = source {
                inputs.push((name, source));
            }
        }
        if let Some(context) = context {
            inputs.push(("context.cue", context));
        }
        self.run(&self.cue, &[operation], &inputs, reverse)
    }
    fn data(&self, data: &Value, context: Option<&str>, operation: &str) -> Output {
        let data = serde_json::to_string(data).expect("independent DATA");
        let mut inputs = vec![
            ("schema.cue", SCHEMA),
            ("bridge.cue", BRIDGE),
            ("selector.cue", SELECTOR),
            ("data.json", &data),
        ];
        if let Some(context) = context {
            inputs.push(("context.cue", context));
        }
        self.run(&self.cue, &[operation], &inputs, false)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}
fn providers() -> [Option<&'static str>; 6] {
    [
        Some(HELPERS),
        Some(VALUES),
        Some(CONTROL),
        Some(DIFFERENCE),
        Some(DIVIDE),
        Some(SAME),
    ]
}

#[test]
fn independent_complete_trees_and_all_120_orders_both_binary_folds() {
    let probe = Probe::new();
    let expected = expected();
    let full = full_expected();
    let roots: Vec<_> = ROOTS.split_whitespace().collect();
    for (index, module) in expected.iter().enumerate() {
        let mut want = match index {
            0 => roots[..21].to_vec(),
            1 => roots[21..32].to_vec(),
            2 | 3 => vec!["integerPrimitive"],
            _ => vec!["same"],
        };
        want.sort();
        assert_eq!(keys(&module["cases"]), want);
        assert_eq!(module.get("formatVersion").is_some(), index == 0);
        assert_eq!(module.get("aliases").is_some(), index == 0);
    }
    assert_eq!(keys(&full["cases"]).len(), 33);
    assert_eq!(
        keys(&expected[0]["cases"]["integerPrimitive"]["cases"]).len(),
        5
    );
    assert_eq!(keys(&full["cases"]["integerPrimitive"]["cases"]).len(), 7);
    assert_eq!(full["aliases"], json!({"choose":"if"}));
    let mut previous = None;
    for renamed in [false, true] {
        let modules: Vec<_> = SOURCES
            .into_iter()
            .enumerate()
            .map(|(index, source)| {
                let companion = if index < 2 { HELPERS } else { PACKAGE };
                for (a, b) in [(companion, source), (source, companion)] {
                    assert_eq!(decode(&export(a, b).expect("API export")), expected[index]);
                }
                let output = probe.native(companion, source, renamed);
                assert_eq!(decode(&output), expected[index]);
                output
            })
            .collect();
        let orders = orders(vec![]);
        assert_eq!(orders.len(), 120);
        for order in orders {
            let mut left = modules[order[0]].clone();
            for &index in &order[1..] {
                left = probe.compose(&left, &modules[index]);
            }
            let mut right = modules[order[4]].clone();
            for &index in order[..4].iter().rev() {
                right = probe.compose(&modules[index], &right);
            }
            assert_eq!(left, right);
            assert_eq!(decode(&left), full);
            if let Some(bytes) = &previous {
                assert_eq!(&left, bytes);
            }
            previous = Some(left);
        }
        let raw = probe.go(providers(), None, "export", renamed);
        assert_eq!(parsed(&raw), full);
        let sorted = encoded(probe.run(
            &probe.jq,
            &["-S", "."],
            &[(
                "raw.json",
                std::str::from_utf8(&raw.stdout).expect("raw UTF-8"),
            )],
            false,
        ));
        assert_eq!(Some(sorted), previous);
    }
    assert_eq!(parsed(&probe.data(&full, None, "export")), full);
    // The actual integer namespace assembles recursively, never overwrites even equal leaves.
    for (index, (left, right)) in [
        (&expected[0], &expected[2]),
        (&full, &expected[2]),
        (&full, &expected[3]),
    ]
    .into_iter()
    .enumerate()
    {
        let left = serde_json::to_string(left).expect("left DATA");
        let right = serde_json::to_string(right).expect("right DATA");
        let output = probe.merge(&left, &right);
        if index == 0 {
            let mut want = expected[0].clone();
            *preference_node(&mut want, 0) = text(PREFERRED[0]);
            assert_eq!(parsed(&output), want);
        } else {
            diagnostic(output, "overlapping module leaf");
        }
    }
}

#[test]
fn missing_helpers_are_setup_but_every_root_branch_and_alias_is_required() {
    let probe = Probe::new();
    for source in [VALUES, CONTROL] {
        assert!(matches!(export(PACKAGE, source), Err(Error::Unknown(_))));
        probe.rejects(PACKAGE, source);
    }
    let mut missing = providers();
    missing[0] = None;
    let unknown = probe.go(missing, None, "export", false);
    failed(&unknown);
    let message = String::from_utf8_lossy(&unknown.stderr);
    assert!(message.contains("reference") || message.contains("undefined"));
    assert!(!message.contains("incomplete value"));
    for index in 1..6 {
        let mut missing = providers();
        missing[index] = None;
        for reverse in [false, true] {
            diagnostic(
                probe.go(missing, None, "export", reverse),
                "incomplete value",
            );
            assert!(probe.go(missing, None, "eval", reverse).status.success());
        }
    }
    let full = full_expected();
    let incomplete = |data: &Value| {
        diagnostic(probe.data(data, None, "export"), "incomplete value");
        assert!(probe.data(data, None, "eval").status.success());
    };
    for root in ROOTS.split_whitespace() {
        let mut missing = full.clone();
        missing["cases"]
            .as_object_mut()
            .expect("cases")
            .remove(root);
        incomplete(&missing);
    }
    for branch in BRANCHES.split_whitespace() {
        let mut missing = full.clone();
        missing["cases"]["integerPrimitive"]["cases"]
            .as_object_mut()
            .expect("branches")
            .remove(branch);
        incomplete(&missing);
    }
    let mut missing = full.clone();
    missing["aliases"]
        .as_object_mut()
        .expect("aliases")
        .remove("choose");
    incomplete(&missing);
    let mut missing = full.clone();
    missing.as_object_mut().expect("root").remove("aliases");
    incomplete(&missing);
    assert_eq!(
        parsed(&probe.go(providers(), None, "export", false)),
        full,
        "recovery"
    );
}

#[test]
fn all_three_raw_preferences_remain_live_and_concrete_late_overrides_conflict() {
    let probe = Probe::new();
    for index in 0..3 {
        let source = SOURCES[index + 2];
        assert!(source.contains(PREFERRED[index]) && source.contains(ALTERNATE[index]));
        // Swap priority, retaining BOTH exact original literals, not a concrete replacement.
        let preferred = format!("*\"{}\"", PREFERRED[index]);
        let alternate = format!("\"{}\"", ALTERNATE[index]);
        let variant = source
            .replace(&preferred, &format!("\"{}\"", PREFERRED[index]))
            .replace(&alternate, &format!("*\"{}\"", ALTERNATE[index]));
        assert_ne!(variant, source);
        assert!(variant.contains(PREFERRED[index]) && variant.contains(ALTERNATE[index]));
        let mut want_module = expected()[index + 2].clone();
        *preference_node(&mut want_module, index) = text(ALTERNATE[index]);
        let mut want = full_expected();
        *preference_node(&mut want, index) = text(ALTERNATE[index]);
        for reverse in [false, true] {
            assert_eq!(
                parsed(&probe.go(providers(), Some(CONTEXTS[index]), "export", reverse)),
                want
            );
            assert_eq!(
                decode(&probe.native(PACKAGE, &variant, reverse)),
                want_module
            );
            let mut changed = providers();
            changed[index + 3] = Some(&variant);
            assert_eq!(parsed(&probe.go(changed, None, "export", reverse)), want);
            for operation in ["export", "eval"] {
                diagnostic(
                    probe.data(&full_expected(), Some(CONTEXTS[index]), operation),
                    "conflicting values",
                );
            }
        }
    }
}

#[test]
fn finite_helper_renaming_source_changes_limits_and_excluded_profiles() {
    let probe = Probe::new();
    let helpers = HELPERS.replace("#UnifyLowering", "#RenamedLowering");
    for source in [VALUES, CONTROL] {
        let renamed = source.replace("#UnifyLowering", "#RenamedLowering");
        assert_eq!(
            export(HELPERS, source).expect("original"),
            export(&helpers, &renamed).expect("renamed")
        );
        assert_eq!(
            probe.native(HELPERS, source, false),
            probe.native(&helpers, &renamed, true)
        );
    }
    // Rust literal paths are DATA, not CUE identifiers subject to helper renaming.
    let full = full_expected();
    assert!(
        full["cases"]["inputEndpoint"]["value"]["parts"]
            .to_string()
            .contains("::Endpoint::")
    );
    assert!(
        full["cases"]["compare"]["parts"]
            .to_string()
            .contains("::Relation::")
    );
    let changed = HELPERS.replace("value: \"_left\"", "value: \"_changed_left\"");
    assert_ne!(changed, HELPERS);
    let mut want = expected()[0].clone();
    for path in ["inputValue", "inputEndpoint"] {
        let index = if path == "inputValue" { 0 } else { 1 };
        want["cases"][path]["value"]["parts"][index]["cases"]["left"] = text("_changed_left");
    }
    assert_eq!(decode(&probe.native(&changed, VALUES, false)), want);
    // All six owner effects through the actual consumer belong to the Nix verifier.
    let whole = format!(
        "{VALUES}\n{}",
        CONTROL
            .strip_prefix("package bootstrap")
            .expect("control package")
    );
    assert_eq!(
        export(HELPERS, &whole),
        Err(Error::Limit("evaluation steps"))
    );
    let output = probe.run(
        &probe.native,
        &[],
        &[("helpers.cue", HELPERS), ("whole.cue", &whole)],
        false,
    );
    diagnostic(output, "evaluation steps");
    for source in [VALUES, CONTROL, "package bootstrap\n{}"] {
        let mixed = format!(
            "{HELPERS}\n{}",
            source.strip_prefix("package bootstrap").expect("package")
        );
        for preference in [DIFFERENCE, DIVIDE, SAME] {
            for (a, b) in [(mixed.as_str(), preference), (preference, mixed.as_str())] {
                assert!(matches!(export(a, b), Err(Error::Unsupported(_))));
                probe.rejects(a, b);
            }
        }
    }
    for (a, b) in [(DIFFERENCE, DIVIDE), (DIFFERENCE, SAME), (DIVIDE, SAME)] {
        for (a, b) in [(a, b), (b, a)] {
            assert!(matches!(export(a, b), Err(Error::Unsupported(_))));
            probe.rejects(a, b);
        }
    }
    failed(&probe.run(&probe.native, &[], &[("package.cue", PACKAGE)], false));
    assert!(
        Command::new(probe.dir.join("missing-tool"))
            .env_clear()
            .output()
            .is_err(),
        "setup failure is not semantic Incomplete"
    );
    let malformed = format!("{SAME}\ngarbage");
    assert!(export(PACKAGE, &malformed).is_err());
    probe.rejects(PACKAGE, &malformed);
    assert_eq!(decode(&probe.native(PACKAGE, SAME, false)), expected()[4]);
}
