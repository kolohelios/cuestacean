//! Diagnostic equality evidence, not native default/equality admission.
#[path = "../../spikes/batch_defaults.rs"]
mod batch_defaults;
#[path = "../../spikes/recursive_default_scheduling.rs"]
mod recursive_default_scheduling;
use Mode::{Default as D, Maybe as M, Not as N};
use batch_defaults::{Body, Mode, Trial};
use recursive_default_scheduling::{Alternative, Expr, Report};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
fn v(n: i64) -> Expr {
    Expr::Value(n)
}
fn a(xs: Vec<Expr>) -> Expr {
    Expr::And(xs)
}
fn o(xs: Vec<(bool, Expr)>) -> Expr {
    Expr::Or(
        xs.into_iter()
            .map(|(marked, expr)| Alternative { marked, expr })
            .collect(),
    )
}
fn g(xs: &[(bool, i64)]) -> Expr {
    o(xs.iter().map(|&(m, n)| (m, v(n))).collect())
}
fn marked() -> Expr {
    g(&[(true, 1), (false, 2)])
}
fn word(m: Mode) -> &'static str {
    match m {
        M => "maybe",
        D => "default",
        N => "not",
    }
}
fn result(n: i64, m: Mode, origin: Mode) -> Value {
    json!({"mode":word(m),"originalMode":word(origin),"value":n})
}
fn cmp(index: usize, kind: &str, existing: Value, accepted: bool) -> Value {
    json!({"index":index,"kind":kind,"existing":existing,"accepted":accepted})
}
fn event(candidate: Value, index: usize, kind: &str, comparisons: Vec<Value>) -> Value {
    json!({"candidate":candidate,"index":index,"kind":kind,"comparisons":comparisons})
}
fn phase(mode: &str, appends: Vec<Value>) -> Value {
    json!({"runMode":mode,"appends":appends})
}
fn native_batches(r: &Report) -> Value {
    Value::Array(r.batches.iter().map(|b| {
        assert_eq!(b.trials.len(),b.nested_origins.len());
        let trials=b.trials.iter().zip(&b.nested_origins).map(|(t,origins)| {
            let (value,nested)=match &t.body {
                Body::Scalar(n)=>{assert!(origins.is_empty());(Some(*n),vec![])},
                Body::Nested(xs)=>{assert_eq!(xs.len(),origins.len());(None,xs.iter().zip(origins).map(|(x,&origin)|result(x.value,x.mode,origin)).collect())}
            };
            json!({"mode":word(t.mode),"originalMode":word(t.incoming),"leftMode":word(t.left_mode),"leftOriginalMode":word(t.left_origin),"value":value,"nested":nested})
        }).collect::<Vec<_>>();
        json!({"id":b.id,"depth":b.depth,"leftDropsDefault":b.left_drops,"rightDropsDefault":b.right_drops,"trials":trials,"results":b.states.iter().map(|s|result(s.value,s.mode,s.origin)).collect::<Vec<_>>()})
    }).collect())
}
struct Files(PathBuf);
impl Files {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cuestacean-equality-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).expect("exclusive owned scratch");
        fs::write(p.join("probe.cue"), format!("probe: {source}\n")).expect("authored source");
        Self(p)
    }
    fn run(&self, name: &str, args: &[&str]) -> Output {
        let p = PathBuf::from(std::env::var_os(name).expect("mandatory provider"));
        assert!(p.is_absolute());
        let m = fs::metadata(&p).expect("provider metadata");
        assert!(m.is_file() && m.permissions().mode() & 0o111 != 0);
        fs::File::open(&p).expect("readable provider");
        Command::new(p)
            .args(args)
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", &self.0)
            .output()
            .expect("no fallback")
    }
    fn trace(&self, mode: &str) -> (Value, Vec<u8>) {
        let output = self.run("CUESTACEAN_DEFAULT_TRACE", &[mode, "probe.cue", "probe"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert!(output.stdout.len() <= 65536 && output.stdout.ends_with(b"\n"));
        let second = self.run("CUESTACEAN_DEFAULT_TRACE", &[mode, "probe.cue", "probe"]);
        assert!(second.status.success() && second.stderr.is_empty());
        assert_eq!(output.stdout, second.stdout);
        (
            serde_json::from_slice(&output.stdout).expect("trace JSON"),
            output.stdout,
        )
    }
    fn semantics(&self, trace: &Value, expected: &str) {
        let (flag, rest) = expected.split_once(':').expect("hand public expectation");
        let selected = flag == "T";
        let (status, value) = match rest {
            "incomplete" | "conflict" => (rest, None),
            _ => ("concrete", Some(rest.parse::<i64>().expect("i64"))),
        };
        assert_eq!(trace["defaultSelected"], selected);
        assert_eq!(trace["status"], status);
        assert_eq!(trace["value"], json!(value));
        let api = self.run(
            "CUESTACEAN_DEFAULT_OBSERVER",
            &["inspect", "probe.cue", "probe"],
        );
        assert!(api.status.success() && api.stderr.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&api.stdout).expect("independent public API"),
            json!({"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":selected,"status":status,"value":value})
        );
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory CLI");
        let cli = |operation| {
            Command::new(&cue)
                .args([operation, "probe.cue", "-e", "probe"])
                .current_dir(&self.0)
                .env_clear()
                .env("PATH", "/unusable")
                .env("HOME", &self.0)
                .output()
                .expect("pinned inline CLI")
        };
        let export = cli("export");
        if let Some(n) = value {
            assert!(export.status.success());
            assert_eq!(export.stdout.trim_ascii(), n.to_string().as_bytes());
        } else {
            assert!(
                !export.status.success() && export.stdout.is_empty() && !export.stderr.is_empty()
            );
            assert_eq!(cli("eval").status.success(), status == "incomplete");
        }
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("owned scratch cleanup");
    }
}
fn validate(trace: &Value) {
    assert_eq!(trace["formatVersion"], 4);
    assert_eq!(trace["mode"], "trace-equality");
    let batches = trace["batches"].as_array().expect("batches");
    assert!(batches.len() <= 128);
    let mut records = 0;
    for (id, b) in batches.iter().enumerate() {
        assert_eq!(b["id"], id);
        let ts = b["trials"].as_array().expect("trials");
        let rs = b["results"].as_array().expect("results");
        assert!(ts.len() <= 64 && rs.len() <= 64);
        records += 3 * ts.len() + 2 * rs.len();
        for t in ts {
            let xs = t["nested"].as_array().expect("nested");
            assert!(xs.len() <= 64);
            records += 2 * xs.len();
        }
        let p = &b["phase"];
        assert!(matches!(
            p["runMode"].as_str(),
            Some("ignore" | "attempt-only" | "yield" | "finalize")
        ));
        let appends = p["appends"].as_array().expect("appends");
        assert!(appends.len() <= 64);
        records += 3 * appends.len();
        for e in appends {
            let comparisons = e["comparisons"].as_array().expect("comparisons");
            assert!(comparisons.len() <= 64);
            records += 3 * comparisons.len();
            let mut accepted = 0;
            for (index, c) in comparisons.iter().enumerate() {
                assert_eq!(c["index"], index);
                assert!(matches!(
                    c["kind"].as_str(),
                    Some("identity" | "partial" | "final")
                ));
                if c["accepted"] == true {
                    accepted += 1;
                    assert_eq!(index + 1, comparisons.len());
                    assert_eq!(e["kind"], c["kind"]);
                    assert_eq!(e["index"], index);
                }
            }
            if e["kind"] == "insert" {
                assert_eq!(accepted, 0);
                assert_eq!(e["index"], comparisons.len());
            } else {
                assert_eq!(accepted, 1);
            }
        }
    }
    assert!(records <= 4096);
}
fn observe(ir: Expr, source: &str, semantic: &str) -> Value {
    // Native receives authored IR ONLY, and runs BEFORE either observation.
    let saved = ir.clone();
    let native = recursive_default_scheduling::execute(&ir, 4096).expect("native construction");
    assert_eq!(ir, saved);
    assert!(native.work > 0);
    let files = Files::new(source);
    let (trace, _) = files.trace("trace-equality");
    validate(&trace);
    files.semantics(&trace, semantic);
    let (legacy, _) = files.trace("trace-history");
    let mut stripped = trace.clone();
    stripped["formatVersion"] = json!(3);
    stripped["mode"] = json!("trace-history");
    for b in stripped["batches"].as_array_mut().expect("batches") {
        assert!(b.as_object_mut().expect("batch").remove("phase").is_some());
    }
    assert_eq!(
        stripped, legacy,
        "additive mode preserves complete v3 observation: {source}"
    );
    assert_eq!(
        native_batches(&native),
        legacy["batches"],
        "constructed facts, never oracle inputs: {source}"
    );
    trace
}
fn duplicate_events(first: Mode, second: Mode, kind: &str) -> Vec<Value> {
    vec![
        event(result(1, first, first), 0, "insert", vec![]),
        event(
            result(1, second, second),
            0,
            kind,
            vec![cmp(0, kind, result(1, first, first), true)],
        ),
        event(
            result(2, N, N),
            1,
            "insert",
            vec![cmp(0, kind, result(1, D, first), false)],
        ),
    ]
}

#[test]
fn actual_final_comparisons_and_promotion_keep_first_origin() {
    for (xs, source, first, second) in [
        (vec![(false, 1), (true, 1), (false, 2)], "1|*1|2", N, D),
        (vec![(true, 1), (false, 1), (false, 2)], "*1|1|2", D, N),
    ] {
        let trace = observe(g(&xs), source, "T:1");
        assert_eq!(
            trace["batches"][0]["phase"],
            phase("finalize", duplicate_events(first, second, "final"))
        );
    }
}
#[test]
fn attempt_only_partial_coalescence_exposes_finalized_only_coverage_cost() {
    let trace = observe(
        a(vec![
            g(&[(false, 1), (true, 1), (false, 2)]),
            g(&[(false, 1), (false, 2)]),
        ]),
        "(1|*1|2)&(1|2)",
        "T:1",
    );
    assert_eq!(
        trace["batches"][0]["phase"],
        phase("attempt-only", duplicate_events(N, D, "partial"))
    );
    assert_eq!(
        trace["batches"][1]["phase"],
        phase(
            "finalize",
            vec![
                event(result(1, D, M), 0, "insert", vec![]),
                event(
                    result(2, N, M),
                    1,
                    "insert",
                    vec![cmp(0, "final", result(1, D, M), false)]
                )
            ]
        )
    );
    // This source-reachable accepted partial comparison agrees with the native
    // hypothesis. A finalized-only admission would defer it; that is a coverage
    // cost, NOT permission to admit arbitrary partially evaluated vertices.
}
#[test]
fn nested_append_snapshots_precede_batch_wide_normalization() {
    let trace = observe(
        o(vec![(false, marked()), (false, v(3))]),
        "((*1|2)|3)",
        "T:1",
    );
    assert_eq!(
        trace["batches"][0]["phase"],
        phase(
            "finalize",
            vec![
                event(result(1, D, D), 0, "insert", vec![]),
                event(
                    result(2, N, N),
                    1,
                    "insert",
                    vec![cmp(0, "final", result(1, D, D), false)]
                ),
                event(
                    result(3, M, M),
                    2,
                    "insert",
                    vec![
                        cmp(0, "final", result(1, D, D), false),
                        cmp(1, "final", result(2, N, N), false)
                    ]
                )
            ]
        )
    );
    assert_eq!(
        trace["batches"][1]["phase"],
        phase(
            "finalize",
            vec![
                event(result(1, D, D), 0, "insert", vec![]),
                event(
                    result(2, N, N),
                    1,
                    "insert",
                    vec![cmp(0, "final", result(1, D, D), false)]
                )
            ]
        )
    );
    assert_eq!(trace["batches"][0]["results"][2], result(3, N, M));
    assert_eq!(trace["batches"][1]["depth"], 1);
}
#[test]
fn singleton_and_failed_child_phases_are_not_missing_or_parent_attributed() {
    let trace = observe(
        o(vec![(false, a(vec![marked(), v(1)])), (false, v(3))]),
        "(((*1|2)&1)|3)",
        "F:incomplete",
    );
    assert_eq!(
        trace["batches"][0]["phase"],
        phase(
            "finalize",
            vec![
                event(result(1, M, M), 0, "insert", vec![]),
                event(
                    result(3, M, M),
                    1,
                    "insert",
                    vec![cmp(0, "final", result(1, M, M), false)]
                )
            ]
        )
    );
    assert_eq!(
        trace["batches"][1]["phase"],
        phase(
            "finalize",
            vec![event(result(1, D, D), 0, "insert", vec![])]
        )
    );
    let trace = observe(
        a(vec![
            v(2),
            o(vec![(false, g(&[(true, 1), (false, 3)])), (false, v(2))]),
        ]),
        "2&((*1|3)|2)",
        "F:2",
    );
    assert_eq!(
        trace["batches"][0]["phase"],
        phase(
            "finalize",
            vec![event(result(2, M, M), 0, "insert", vec![])]
        )
    );
    assert_eq!(trace["batches"][1]["phase"], phase("finalize", vec![]));
}
#[test]
fn exact_i64s_and_legacy_wire_remain_independent() {
    let trace = observe(
        g(&[(true, i64::MIN), (false, i64::MAX)]),
        "*-9223372036854775808|9223372036854775807",
        "T:-9223372036854775808",
    );
    assert_eq!(
        trace["batches"][0]["phase"],
        phase(
            "finalize",
            vec![
                event(result(i64::MIN, D, D), 0, "insert", vec![]),
                event(
                    result(i64::MAX, N, N),
                    1,
                    "insert",
                    vec![cmp(0, "final", result(i64::MIN, D, D), false)]
                )
            ]
        )
    );
    let files = Files::new("*1|2");
    let (_, bytes) = files.trace("trace");
    assert_eq!(bytes,concat!(r#"{"formatVersion":1,"mode":"trace","defaultSelected":true,"status":"concrete","value":1,"batches":[{"id":0,"depth":0,"leftDropsDefault":true,"rightDropsDefault":false,"trials":[{"mode":"maybe","originalMode":"default","value":1,"nested":[]},{"mode":"maybe","originalMode":"not","value":2,"nested":[]}],"results":[{"mode":"default","value":1},{"mode":"not","value":2}]}]}"#,"\n").as_bytes());
    for (mode, version) in [("trace", 1), ("trace-origins", 2), ("trace-history", 3)] {
        let (old, _) = files.trace(mode);
        assert_eq!(old["formatVersion"], version);
        for b in old["batches"].as_array().expect("batches") {
            assert!(b.get("phase").is_none());
        }
    }
    let (_, original_v4) = files.trace("trace-equality");
    fs::rename(files.0.join("probe.cue"), files.0.join("renamed.cue")).expect("rename");
    let renamed = files.run(
        "CUESTACEAN_DEFAULT_TRACE",
        &["trace-equality", "renamed.cue", "probe"],
    );
    assert!(renamed.status.success() && renamed.stderr.is_empty());
    assert_eq!(
        renamed.stdout, original_v4,
        "renaming preserves complete v4 bytes"
    );
    let parsed: Value = serde_json::from_slice(&renamed.stdout).expect("renamed v4");
    validate(&parsed);
}
#[test]
fn supplemental_old_counterexamples_preserve_v3_semantics_not_phase_goldens() {
    // These two cases are supplemental old-semantic/v3/structural controls,
    // NOT independent full phase expectations. Only the five groups above
    // establish the new metadata against seven complete hand-authored streams.
    let left = o(vec![(false, g(&[(true, 3), (false, 1)])), (false, v(3))]);
    let right = o(vec![
        (false, marked()),
        (false, g(&[(true, 3), (false, 2)])),
    ]);
    for (ir, source) in [
        (
            a(vec![left.clone(), right.clone()]),
            "((*3|1)|3)&((*1|2)|(*3|2))",
        ),
        (a(vec![right, left]), "((*1|2)|(*3|2))&((*3|1)|3)"),
    ] {
        observe(ir, source, "T:incomplete");
    }
    // Meaningful old primitive API control; no visibility/lint escape. Native
    // transitions execute from handwritten inputs, never comparison metadata.
    let trials = [Trial {
        mode: M,
        left_mode: M,
        left_origin: M,
        incoming: D,
        body: Body::Nested(vec![
            batch_defaults::Leaf { value: 3, mode: D },
            batch_defaults::Leaf { value: 1, mode: N },
        ]),
    }];
    let r = batch_defaults::execute(&trials, 4096).expect("old primitive");
    assert!(r.has_non_maybe && r.left_drops && !r.right_drops && r.work > 0);
    assert_eq!(
        r.results,
        vec![
            batch_defaults::Leaf { value: 3, mode: D },
            batch_defaults::Leaf { value: 1, mode: N }
        ]
    );
}
