//! Pinned upstream batch observations only: no native algorithm/readiness claim.
//! Expected traces/facts are handwritten from 6d609d768f1686f9a3a2a20197cacdbb70e5c79d
//! crossProduct/doDisjunct/combineDefault2/appendDisjunct, never captured goldens.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PROVIDER: &str = "CUESTACEAN_DEFAULT_TRACE";
const NO_PATH: &str = "/nonexistent-cuestacean-default-trace-path";
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Maybe,
    Default,
    Not,
}
use Mode::{Default as D, Maybe as M, Not as N};
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ResultFact {
    mode: Mode,
    value: Option<i64>,
}
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Trial {
    mode: Mode,
    original_mode: Mode,
    value: Option<i64>,
    nested: Vec<ResultFact>,
}
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Batch {
    id: u32,
    depth: u32,
    left_drops_default: bool,
    right_drops_default: bool,
    trials: Vec<Trial>,
    results: Vec<ResultFact>,
}
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Trace {
    format_version: u8,
    mode: String,
    default_selected: bool,
    status: String,
    value: Option<i64>,
    batches: Vec<Batch>,
}
fn result(mode: Mode, value: i64) -> ResultFact {
    ResultFact {
        mode,
        value: Some(value),
    }
}
fn trial(mode: Mode, original_mode: Mode, value: i64) -> Trial {
    Trial {
        mode,
        original_mode,
        value: Some(value),
        nested: vec![],
    }
}
fn batch(id: u32, drops: [bool; 2], trials: Vec<Trial>, results: Vec<ResultFact>) -> Batch {
    Batch {
        id,
        depth: 0,
        left_drops_default: drops[0],
        right_drops_default: drops[1],
        trials,
        results,
    }
}
fn semantic(expected: &str) -> (bool, &str, Option<i64>) {
    let (flag, value) = expected
        .split_once(':')
        .expect("handwritten semantic expectation");
    assert!(matches!(flag, "T" | "F"), "invalid handwritten flag");
    let selected = flag == "T";
    match value {
        "incomplete" | "conflict" => (selected, value, None),
        _ => (
            selected,
            "concrete",
            Some(value.parse().expect("exact i64 expectation")),
        ),
    }
}
fn full(expected: &str, batches: Vec<Batch>) -> Trace {
    let (default_selected, status, value) = semantic(expected);
    Trace {
        format_version: 1,
        mode: "trace".into(),
        default_selected,
        status: status.into(),
        value,
        batches,
    }
}
fn wire(trace: &Trace) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(trace).expect("encode exact ordered wire");
    bytes.push(b'\n');
    bytes
}
fn executable(path: PathBuf) -> PathBuf {
    assert!(path.is_absolute(), "mandatory absolute executable");
    let metadata = fs::metadata(&path).expect("provider metadata");
    assert!(metadata.is_file(), "provider must be a file");
    let bits = metadata.permissions().mode();
    assert!(
        bits & 0o111 != 0 && bits & 0o444 != 0,
        "readable/executable provider"
    );
    fs::File::open(&path).expect("open readable provider");
    path
}
fn provider(name: &str) -> PathBuf {
    executable(PathBuf::from(
        std::env::var_os(name).expect("mandatory provider"),
    ))
}
struct Files(PathBuf);
impl Files {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-default-trace-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive scratch directory");
        let files = Self(path);
        files.write("probe.cue", source.as_bytes());
        files
    }
    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).expect("write owned file");
        path
    }
    fn invoke(&self, executable: &Path, args: &[&str]) -> Output {
        Command::new(executable)
            .args(args)
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", NO_PATH)
            .env("HOME", &self.0)
            .output()
            .expect("explicit execution; no fallback")
    }
    fn run(&self, args: &[&str]) -> Output {
        self.invoke(&provider(PROVIDER), args)
    }
    fn trace(&self, label: &str) -> Trace {
        let output = self.run(&["trace", "probe.cue", label]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert!(output.stdout.len() <= 65536);
        let trace: Trace = serde_json::from_slice(&output.stdout).expect("strict trace schema");
        assert_eq!(
            output.stdout,
            wire(&trace),
            "exact ordered fields/compact JSON/one LF"
        );
        let repeat = self.run(&["trace", "probe.cue", label]);
        assert!(repeat.status.success());
        assert!(repeat.stderr.is_empty());
        assert_eq!(
            repeat.stdout, output.stdout,
            "fresh-process byte determinism"
        );
        assert_eq!(trace.format_version, 1);
        assert_eq!(trace.mode, "trace");
        assert!(trace.batches.len() <= 128);
        let mut records = 0;
        for (id, b) in trace.batches.iter().enumerate() {
            assert_eq!(
                b.id as usize, id,
                "entry order, not completion/source order"
            );
            assert!(b.trials.len() <= 64 && b.results.len() <= 64);
            records += b.trials.len() + b.results.len();
            for t in &b.trials {
                assert!(t.nested.len() <= 64);
                records += t.nested.len();
            }
        }
        assert!(records <= 4096);
        trace
    }
    fn compare(&self, label: &str, expected: &str) -> Trace {
        let trace = self.trace(label);
        let (flag, status, value) = semantic(expected);
        assert_eq!(
            (trace.default_selected, trace.status.as_str(), trace.value),
            (flag, status, value)
        );
        let raw = value.map_or_else(|| "null".into(), |v| v.to_string());
        let observer = self.invoke(
            &provider("CUESTACEAN_DEFAULT_OBSERVER"),
            &["inspect", "probe.cue", label],
        );
        assert!(observer.status.success());
        assert!(observer.stderr.is_empty());
        assert_eq!(observer.stdout, format!("{{\"formatVersion\":1,\"mode\":\"inspect\",\"leftDefault\":null,\"rightDefault\":null,\"defaultSelected\":{flag},\"status\":\"{status}\",\"value\":{raw}}}\n").as_bytes());
        // Resolve pinned devshell CLI BEFORE clearing PATH. Independent inline source
        // export/eval, not API Unify and never a fallback for either provider.
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned devshell PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned CUE CLI");
        let cue = executable(cue);
        let exported = self.invoke(&cue, &["export", "probe.cue", "-e", label]);
        if status == "concrete" {
            assert!(exported.status.success(), "{:?}", exported.stderr);
            assert_eq!(exported.stdout.trim_ascii(), raw.as_bytes());
        } else {
            assert!(!exported.status.success());
            assert!(exported.stdout.is_empty());
            assert!(!exported.stderr.is_empty());
            let evaluated = self.invoke(&cue, &["eval", "probe.cue", "-e", label]);
            assert_eq!(evaluated.status.success(), status == "incomplete");
        }
        trace
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned scratch directory");
    }
}
fn failed(output: Output) {
    assert!(!output.status.success(), "invalid setup accepted");
    assert!(output.stdout.is_empty(), "no partial JSON on failure");
    assert!(!output.stderr.is_empty());
}

macro_rules! b {
    ($id:literal, $left:literal, $right:literal; [$($a:ident,$b:ident,$v:literal);*] => [$($m:ident,$r:literal);*]) => {
        batch($id, [$left, $right], vec![$(trial($a,$b,$v)),*], vec![$(result($m,$r)),*])
    };
}
macro_rules! flat {
    ($source:literal, $expected:literal; $($batch:expr),*) => { ($source, $expected, vec![$($batch),*]) };
}
macro_rules! case {
    ($name:literal: $left:literal, $right:literal =>
     $f:literal,$fl:literal,$fr:literal,[$($fm:ident,$fv:literal);*] /
     $r:literal,$rl:literal,$rr:literal,[$($rm:ident,$rv:literal);*]) => {
        ($name, $left, $right, [($f, [$fl,$fr], vec![$(result($fm,$fv)),*]),
            ($r, [$rl,$rr], vec![$(result($rm,$rv)),*])])
    };
}
#[test]
fn complete_hand_derived_flat_traces() {
    // Initial cross is neutral; doDisjunct installs the incoming mark. The
    // successful trials alone decide drops; combineDefault2 erases dropped sides.
    let cases = [
        flat!("1", "F:1";),
        flat!("1&2", "F:conflict";),
        flat!("1|2", "F:incomplete"; b!(0,true,true; [M,M,1; M,M,2] => [M,1; M,2])),
        flat!("*1|2", "T:1"; b!(0,true,false; [M,D,1; M,N,2] => [D,1; N,2])),
        flat!("*1|*2|3", "T:incomplete";
            b!(0,true,false; [M,D,1; M,D,2; M,N,3] => [D,1; D,2; N,3])),
        flat!("(*1|2)&2", "F:2"; b!(0,true,true; [M,N,2] => [M,2])),
        flat!("(*1|2)&(*2|1)", "F:incomplete";
            b!(0,true,false; [M,D,1; M,N,2] => [D,1; N,2]),
            b!(1,false,false; [D,N,1; N,D,2] => [N,1; N,2])),
        flat!("(*1|2)&(1|2|*3)", "T:1";
            b!(0,true,false; [M,D,1; M,N,2] => [D,1; N,2]),
            b!(1,false,true; [D,N,1; N,N,2] => [D,1; N,2])),
        flat!("(1|2|*3)&(*1|2)", "T:1";
            b!(0,true,false; [M,N,1; M,N,2; M,D,3] => [N,1; N,2; D,3]),
            b!(1,true,false; [N,D,1; N,N,2] => [D,1; N,2])),
    ];
    for (source, expected, batches) in cases {
        let files = Files::new(&format!("chosen: {source}\n"));
        assert_eq!(
            files.compare("chosen", expected),
            full(expected, batches),
            "{source}"
        );
    }
}

#[test]
fn old_failures_and_separately_ordered_asymmetries() {
    // Final Default bools are prior public-API expectations, NOT inferred from
    // native Absent/Bottom or CLI success. No abstract *int substitution: that
    // old asymmetry remains exclusively in the unchanged public observer target.
    // Internal columns separately pin last depth-zero batch drop flags and its
    // complete result mode/value set. Combination uses not > default > maybe,
    // dropped-side neutralization and duplicate-default promotion. These are
    // ordered-input observations, not an order-independent recipe calculus.
    let cases = [
        case!("176 genuine": "(*1|2)|*3", "*2|1" =>
            "F:incomplete",false,false,[N,1;N,2] / "T:incomplete",false,true,[D,1;D,2]),
        case!("186 genuine": "(*3|1)|3", "(*1|2)|(*3|2)" =>
            "T:incomplete",false,true,[D,1;D,3] / "T:incomplete",false,true,[D,1;D,3]),
        case!("188 dead mark": "(*1|2)|*3", "1|2" =>
            "F:incomplete",false,true,[N,1;N,2] / "F:incomplete",true,true,[M,1;M,2]),
        case!("188 regression": "*((*1|2)|3)|(*4|5)", "1|4" =>
            "T:1",false,true,[D,1;N,4] / "T:1",true,false,[D,1;N,4]),
        case!("188 bottom": "((*1|2)&(*2|1))|1", "*1|2" =>
            "F:incomplete",false,false,[N,1;N,2] / "T:incomplete",false,true,[D,1;D,2]),
        case!("176 asymmetric": "*1|2", "(*1|2)|*3" =>
            "T:1",false,true,[D,1;M,2] / "F:incomplete",false,false,[N,1;N,2]),
        case!("176 nested asymmetric": "*((*1|2)&(1|*2))|3", "*3|1|2" =>
            "F:incomplete",false,false,[N,1;N,2;N,3] / "T:2",false,false,[N,1;D,2;N,3]),
        case!("186 asymmetric 1": "(*1|2)|(*2|1)", "*1|2" =>
            "T:1",false,false,[D,1;N,2] / "T:incomplete",false,true,[D,1;D,2]),
        case!("186 asymmetric 2": "((*1|2)&(*1|3))|3", "*3|1" =>
            "T:3",true,false,[N,1;D,3] / "T:incomplete",false,true,[D,1;D,3]),
        case!("188 asymmetric": "*((*1|2)|3)|(*4|5)", "1|2|3|4|5" =>
            "T:1",false,true,[D,1;N,2;N,3;N,4;N,5] / "T:incomplete",true,false,[D,1;D,2;D,3;N,4;N,5]),
        // The outer star is essential: ((*3|1)|2) is a DIFFERENT probe.
        case!("marked M2 asymmetric": "*(*3|1)|2", "1|2" =>
            "F:incomplete",true,true,[M,1;M,2] / "T:1",true,false,[D,1;N,2]),
        case!("compatible nested": "*2|3", "(*1|2)|3" =>
            "T:2",false,true,[D,2;N,3] / "T:2",true,false,[D,2;N,3]),
    ];
    for (name, left, right, expected) in cases {
        for ((a, b), (semantic, drops, mut results)) in
            [(left, right), (right, left)].into_iter().zip(expected)
        {
            let files = Files::new(&format!("chosen: ({a})&({b})\n"));
            let observed = files.compare("chosen", semantic);
            let last = observed
                .batches
                .into_iter()
                .rev()
                .find(|b| b.depth == 0)
                .expect(name);
            assert_eq!(
                [last.left_drops_default, last.right_drops_default],
                drops,
                "{name}: {a}&{b}"
            );
            let mut actual = last.results;
            actual.sort_by_key(|r| r.value);
            results.sort_by_key(|r| r.value);
            assert_eq!(actual, results, "{name}: {a}&{b}");
        }
    }
}

#[test]
fn source_justified_unroll_and_current_only_support_gap() {
    // M3: the outer unmarked child has incoming `not`, so nested default 1
    // combines to `not`; the marked sibling 3 survives in the first batch.
    for right in ["*2|1", "1|2"] {
        let trace = Files::new(&format!("chosen: ((*1|2)|*3)&({right})\n")).trace("chosen");
        let first = &trace.batches[0];
        assert_eq!((first.depth, first.right_drops_default), (0, false));
        let t = &first.trials[0];
        assert_eq!((t.mode, t.original_mode, t.value), (M, N, None));
        assert_eq!(first.trials[0].nested, vec![result(D, 1), result(N, 2)]);
        assert_eq!(
            first.results,
            vec![result(N, 1), result(N, 2), result(D, 3)]
        );
        let last = trace
            .batches
            .iter()
            .rev()
            .find(|b| b.depth == 0)
            .expect("outer combination");
        assert_eq!(last.trials.len(), 2);
        assert!(last.trials.iter().all(|t| t.mode == N));
        assert!(last.trials.iter().all(|t| t.nested.is_empty()));
        assert!(!last.left_drops_default);
        // Source checks p.defaultMode OR p.origDefaultMode, not r.mode alone.
        // Trial.originalMode is incoming r.origDefaultMode, NOT p's origin;
        // these observations cannot reconstruct that missing left provenance.
        assert_eq!(last.results, vec![result(N, 1), result(N, 2)]);
    }
    // A conflicting nested pair and plain maybe sibling coalesce to not1/not2.
    // Both hasNonMaybe normalization and appendDisjunct coalescing are relevant;
    // this trace cannot identify which equal-valued source occurrence survived.
    let trace = Files::new("chosen: (((*1|2)&(*2|1))|1)&(*1|2)\n").trace("chosen");
    let first = &trace.batches[0];
    assert_eq!(first.trials[0].nested, vec![result(N, 1), result(N, 2)]);
    assert_eq!((first.trials[1].mode, first.trials[1].value), (M, Some(1)));
    assert_eq!(first.results, vec![result(N, 1), result(N, 2)]);
    // M2 retains the marked child's own default 3 rather than making every
    // child value default. Later both-side drops neutralize 1/2, not revive 3.
    let trace = Files::new("chosen: (*(*3|1)|2)&(1|2)\n").trace("chosen");
    assert_eq!(trace.batches[0].trials[0].original_mode, D);
    assert_eq!(
        trace.batches[0].trials[0].nested,
        vec![result(D, 3), result(N, 1)]
    );
    assert_eq!(
        trace.batches[0].results,
        vec![result(D, 3), result(N, 1), result(N, 2)]
    );
}

#[test]
fn expression_node_and_depth_caps_are_inclusive() {
    fn balanced(leaves: usize) -> String {
        if leaves == 1 {
            return "1".into();
        }
        let (a, b) = (balanced(leaves / 2), balanced(leaves - leaves / 2));
        format!("({a}&{b})")
    }
    // 43 literals + 42 binary operators + 42 parentheses = 127 nodes.
    // Outer parentheses distinguish 128/129 nodes while both depths stay <=16.
    let tree = balanced(43);
    let files = Files::new(&format!("chosen: ({tree})\n"));
    assert_eq!(files.compare("chosen", "F:1"), full("F:1", vec![]));
    files.write("probe.cue", format!("chosen: (({tree}))\n").as_bytes());
    failed(files.run(&["trace", "probe.cue", "chosen"]));
    for (count, valid) in [(15, true), (16, false)] {
        files.write(
            "probe.cue",
            format!("chosen: {}1{}\n", "(".repeat(count), ")".repeat(count)).as_bytes(),
        );
        if valid {
            assert_eq!(files.compare("chosen", "F:1"), full("F:1", vec![]));
        } else {
            failed(files.run(&["trace", "probe.cue", "chosen"]));
        }
    }
    // Trace-retention overflows are constructed in scoped Go unit tests; do
    // not increase profile/solver guards or forge unreachable source shapes.
}

#[test]
fn malformed_argv_paths_labels_and_finite_profile() {
    let files = Files::new("chosen: *1|2\n");
    for args in [
        vec![],
        vec!["unknown"],
        vec!["trace"],
        vec!["trace", "probe.cue"],
        vec!["inspect", "probe.cue", "chosen"],
        vec!["trace", "probe.cue", "chosen", "extra"],
        vec!["trace", "absent.cue", "chosen"],
        vec!["trace", ".", "chosen"],
        vec!["trace", "probe.cue", ""],
        vec!["trace", "probe.cue", "absent"],
        vec!["trace", "probe.cue", "chosen.x"],
        vec!["trace", "probe.cue", "\"chosen\""],
    ] {
        failed(files.run(&args));
    }
    for source in [
        "",
        "chosen: (",
        "chosen: 1\nother: 2",
        "chosen: 1\nchosen: 1",
        "import \"math\"\nchosen: 1",
        "let x = 1\nchosen: 1",
        "#chosen: 1",
        "_chosen: 1",
        "chosen?: 1",
        "chosen!: 1",
        "chosen: 1 @tag()",
        "\"chosen\": 1",
        "chosen: {x: 1}",
        "chosen: [1,2]",
    ] {
        files.write("probe.cue", source.as_bytes());
        failed(files.run(&["trace", "probe.cue", "chosen"]));
    }
    for expression in concat!(
        "int _ true null missing chosen *int|1 1+2 1*2 1.0 1e2 0xff 1_000 01 ",
        "--1 -+1 -(1) +(1) 9223372036854775808 -9223372036854775809"
    )
    .split_whitespace()
    {
        files.write("probe.cue", format!("chosen: {expression}\n").as_bytes());
        failed(files.run(&["trace", "probe.cue", "chosen"]));
    }
    for label in [
        "_", "int", "bool", "string", "bytes", "number", "float", "é",
    ] {
        files.write("probe.cue", format!("{label}: 1\n").as_bytes());
        failed(files.run(&["trace", "probe.cue", label]));
    }
}

#[test]
fn inclusive_file_field_and_signed_decimal_boundaries() {
    let files = Files::new("chosen: 1\n");
    let mut source = b"chosen: 1\n".to_vec();
    source.resize(65536, b' ');
    files.write("probe.cue", &source);
    assert_eq!(files.compare("chosen", "F:1"), full("F:1", vec![]));
    source.push(b' ');
    files.write("probe.cue", &source);
    failed(files.run(&["trace", "probe.cue", "chosen"]));
    let label = "x".repeat(64);
    files.write(
        "probe.cue",
        format!("package renamed\n{label}: *1|2\n").as_bytes(),
    );
    let renamed = files.compare(&label, "T:1");
    files.write(
        "renamed source.cue",
        format!("package relocated\n{label}: *1|2\n").as_bytes(),
    );
    let output = files.run(&["trace", "renamed source.cue", &label]);
    assert!(output.status.success() && output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        wire(&renamed),
        "filename/package are not trace identities"
    );
    files.write("probe.cue", b"chosen: *1|2\n");
    assert_eq!(renamed, files.compare("chosen", "T:1"));
    let label = "x".repeat(65);
    files.write("probe.cue", format!("{label}: 1\n").as_bytes());
    failed(files.run(&["trace", "probe.cue", &label]));
    for (source, expected) in [
        ("+1", "F:1"),
        ("0", "F:0"),
        ("-0", "F:0"),
        ("-9223372036854775808", "F:-9223372036854775808"),
        ("+9223372036854775807", "F:9223372036854775807"),
    ] {
        files.write("probe.cue", format!("chosen: {source}\n").as_bytes());
        assert_eq!(files.compare("chosen", expected), full(expected, vec![]));
    }
}

// The parent never mutates ambient environment: broken providers are isolated
// in fresh copies of this test executable, including failed exec-format/read.
#[test]
fn provider_setup_requires_absolute_executable() {
    assert_eq!(
        Files::new("chosen: 1\n").trace("chosen"),
        full("F:1", vec![])
    );
}
#[test]
fn missing_relative_unreadable_nonexecutable_providers_are_fatal() {
    let files = Files::new("chosen: 1\n");
    let unreadable = files.write("unreadable", b"not an oracle");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o111)).expect("unreadable");
    let nonexec = files.write("nonexec", b"not an oracle");
    let invalid = files.write("invalid", b"not an executable format");
    fs::set_permissions(&invalid, fs::Permissions::from_mode(0o755)).expect("invalid executable");
    for path in [
        None,
        Some(Path::new("relative")),
        Some(files.0.as_path()),
        Some(files.0.join("absent").as_path()),
        Some(unreadable.as_path()),
        Some(nonexec.as_path()),
        Some(invalid.as_path()),
    ] {
        let mut command = Command::new(std::env::current_exe().expect("current test binary"));
        command
            .args([
                "--exact",
                "provider_setup_requires_absolute_executable",
                "--nocapture",
            ])
            .env_clear()
            .env("PATH", NO_PATH);
        if let Some(path) = path {
            command.env(PROVIDER, path);
        }
        let output = command.output().expect("isolated provider test");
        assert!(!output.status.success(), "{path:?}");
    }
}
