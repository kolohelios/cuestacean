//! Opt-in native operations, not a partial-vertex/readiness certificate.
#[path = "../../spikes/batch_defaults.rs"]
mod batch_defaults;
#[path = "../../spikes/recursive_default_scheduling.rs"]
mod recursive_default_scheduling;
use Mode::{Default as D, Maybe as M, Not as N};
use batch_defaults::{
    Append, Body, Comparison, Error, Leaf, Mode, Occurrence, Trial, execute, execute_audited,
};
use recursive_default_scheduling::{Alternative, Batch, Expr};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
fn leaf(value: i64, mode: Mode) -> Leaf {
    Leaf { value, mode }
}
fn pos(trial: usize, child: Option<usize>) -> Occurrence {
    Occurrence { trial, child }
}
fn scalar(value: i64, incoming: Mode) -> Trial {
    Trial {
        mode: M,
        left_mode: M,
        left_origin: M,
        incoming,
        body: Body::Scalar(value),
    }
}
fn comparison(occurrence: Occurrence, existing: Leaf, matched: bool) -> Comparison {
    Comparison {
        occurrence,
        existing,
        matched,
    }
}
fn append(
    occurrence: Occurrence,
    candidate: Leaf,
    index: usize,
    coalesced: bool,
    comparisons: Vec<Comparison>,
) -> Append {
    Append {
        occurrence,
        candidate,
        index,
        coalesced,
        comparisons,
    }
}
fn checked(input: &[Trial], expected: Vec<Append>, results: Vec<Leaf>, flags: [bool; 3]) {
    let saved = input.to_vec();
    let audited = execute_audited(input, 4096).expect("native authored input");
    assert_eq!(audited.appends, expected);
    assert_eq!(audited.report.results, results);
    assert_eq!(
        [
            audited.report.left_drops,
            audited.report.right_drops,
            audited.report.has_non_maybe
        ],
        flags
    );
    let old = execute(input, 4096).expect("independent unaudited invocation");
    assert_eq!(old.results, audited.report.results);
    assert_eq!([old.left_drops, old.right_drops, old.has_non_maybe], flags);
    let comparisons = expected.iter().map(|e| e.comparisons.len()).sum::<usize>();
    assert_eq!(
        audited.report.work,
        old.work + 4 + 5 * expected.len() + 4 * comparisons + results.len()
    );
    assert_eq!(execute_audited(input, 4096).expect("fresh repeat"), audited);
    assert_eq!(input, saved);
}
#[test]
fn complete_hand_scalar_transcripts_keep_retained_first_positions() {
    for (first, second) in [(N, D), (D, N)] {
        checked(
            &[scalar(1, first), scalar(1, second), scalar(2, N)],
            vec![
                append(pos(0, None), leaf(1, first), 0, false, vec![]),
                append(
                    pos(1, None),
                    leaf(1, second),
                    0,
                    true,
                    vec![comparison(pos(0, None), leaf(1, first), true)],
                ),
                append(
                    pos(2, None),
                    leaf(2, N),
                    1,
                    false,
                    vec![comparison(pos(0, None), leaf(1, D), false)],
                ),
            ],
            vec![leaf(1, D), leaf(2, N)],
            [true, false, false],
        );
    }
    checked(&[], vec![], vec![], [true, true, false]);
    checked(
        &[scalar(i64::MIN, D), scalar(i64::MAX, N)],
        vec![
            append(pos(0, None), leaf(i64::MIN, D), 0, false, vec![]),
            append(
                pos(1, None),
                leaf(i64::MAX, N),
                1,
                false,
                vec![comparison(pos(0, None), leaf(i64::MIN, D), false)],
            ),
        ],
        vec![leaf(i64::MIN, D), leaf(i64::MAX, N)],
        [true, false, false],
    );
}
#[test]
fn complete_hand_nested_transcripts_precede_promotion_and_normalization() {
    let input = [Trial {
        body: Body::Nested(vec![leaf(3, D), leaf(1, N)]),
        ..scalar(0, D)
    }];
    checked(
        &input,
        vec![
            append(pos(0, Some(0)), leaf(3, D), 0, false, vec![]),
            append(
                pos(0, Some(1)),
                leaf(1, N),
                1,
                false,
                vec![comparison(pos(0, Some(0)), leaf(3, D), false)],
            ),
        ],
        vec![leaf(3, D), leaf(1, N)],
        [true, false, true],
    );
    let input = [
        Trial {
            left_mode: D,
            ..scalar(7, M)
        },
        Trial {
            left_mode: D,
            body: Body::Nested(vec![leaf(7, N), leaf(9, M)]),
            ..scalar(0, M)
        },
    ];
    checked(
        &input,
        vec![
            append(pos(0, None), leaf(7, M), 0, false, vec![]),
            append(
                pos(1, Some(0)),
                leaf(7, N),
                0,
                true,
                vec![comparison(pos(0, None), leaf(7, M), true)],
            ),
            append(
                pos(1, Some(1)),
                leaf(9, M),
                1,
                false,
                vec![comparison(pos(0, None), leaf(7, M), false)],
            ),
        ],
        vec![leaf(7, N), leaf(9, N)],
        [false, true, true],
    );
}
#[test]
fn exact_charge_anchors_and_audit_coverage_cost_recover_without_reset() {
    assert_eq!(
        (
            execute(&[], 4096).expect("empty").work,
            execute_audited(&[], 4096).expect("empty audit").report.work
        ),
        (2, 6)
    );
    let one = [scalar(1, D)];
    assert_eq!(
        (
            execute(&one, 4096).expect("scalar").work,
            execute_audited(&one, 4096)
                .expect("scalar audit")
                .report
                .work
        ),
        (8, 18)
    );
    let duplicates = vec![scalar(1, D); 64];
    assert_eq!(
        (
            execute(&duplicates, 4096).expect("64 raw duplicates").work,
            execute_audited(&duplicates, 4096)
                .expect("64 audit duplicates")
                .report
                .work
        ),
        (323, 900)
    );
    let unique = (0..64).map(|n| scalar(n, D)).collect::<Vec<_>>();
    assert_eq!(execute(&unique, 4096).expect("old distinct64").work, 2402);
    assert!(matches!(
        execute_audited(&unique, 4096),
        Err(Error::Limit(_))
    ));
    let audit = execute_audited(&duplicates, 900).expect("exact recovery");
    assert!(matches!(
        execute_audited(&duplicates, 899),
        Err(Error::Limit(_))
    ));
    assert_eq!(execute_audited(&duplicates, 4096).expect("repeat"), audit);
    let malformed = [Trial {
        body: Body::Nested(vec![]),
        ..scalar(0, M)
    }];
    for run in [
        execute_audited(&malformed, 0),
        execute_audited(&malformed, 4096),
    ] {
        assert!(matches!(run, Err(Error::Unsupported(_))));
    }
    assert!(matches!(
        execute_audited(&malformed, 4097),
        Err(Error::Limit(_))
    ));
    assert!(matches!(execute_audited(&one, 0), Err(Error::Limit(_))));
    assert!(matches!(
        execute_audited(&vec![scalar(1, D); 65], 4096),
        Err(Error::Limit(_))
    ));
    let expanded = [
        Trial {
            body: Body::Nested(vec![leaf(1, D); 64]),
            ..scalar(0, M)
        },
        scalar(1, M),
    ];
    assert!(matches!(
        execute_audited(&expanded, 4096),
        Err(Error::Limit(_))
    ));
}
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
fn origin(batch: &Batch, p: Occurrence) -> Mode {
    let trial = &batch.trials[p.trial];
    match (&trial.body, p.child) {
        (Body::Scalar(_), None) => trial.incoming,
        (Body::Nested(children), Some(child)) => {
            assert!(child < children.len());
            batch.nested_origins[p.trial][child]
        }
        _ => panic!("native encounter shape mismatch"),
    }
}
fn snapshot(x: &Leaf, origin: Mode) -> Value {
    json!({"mode":word(x.mode),"originalMode":word(origin),"value":x.value})
}
fn native_projection(batch: &Batch, events: &[Append]) -> Value {
    Value::Array(events.iter().map(|e|json!({"candidate":snapshot(&e.candidate,origin(batch,e.occurrence)),"index":e.index,"coalesced":e.coalesced,"comparisons":e.comparisons.iter().enumerate().map(|(index,c)|json!({"index":index,"existing":snapshot(&c.existing,origin(batch,c.occurrence)),"accepted":c.matched})).collect::<Vec<_>>()})).collect())
}
fn go_projection(phase: &Value) -> Value {
    assert!(matches!(
        phase["runMode"].as_str(),
        Some("attempt-only" | "finalize")
    ));
    Value::Array(phase["appends"].as_array().expect("append vector").iter().map(|e| {
        assert!(matches!(e["kind"].as_str(),Some("insert"|"identity"|"partial"|"final")));
        json!({"candidate":e["candidate"],"index":e["index"],"coalesced":e["kind"]!="insert","comparisons":e["comparisons"].as_array().expect("comparison vector").iter().map(|c|json!({"index":c["index"],"existing":c["existing"],"accepted":c["accepted"]})).collect::<Vec<_>>()})
    }).collect())
}
struct Files(PathBuf);
impl Files {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cuestacean-native-audit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).expect("exclusive scratch");
        fs::write(p.join("probe.cue"), format!("probe: {source}\n")).expect("authored source");
        Self(p)
    }
    fn trace(&self) -> Value {
        let p = PathBuf::from(
            std::env::var_os("CUESTACEAN_DEFAULT_TRACE").expect("mandatory provider"),
        );
        assert!(p.is_absolute());
        let m = fs::metadata(&p).expect("provider metadata");
        assert!(m.is_file() && m.permissions().mode() & 0o111 != 0);
        fs::File::open(&p).expect("readable provider");
        let run = || {
            Command::new(&p)
                .args(["trace-equality", "probe.cue", "probe"])
                .current_dir(&self.0)
                .env_clear()
                .env("PATH", "/unusable")
                .env("HOME", &self.0)
                .output()
                .expect("no fallback")
        };
        let out = run();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stderr.is_empty() && out.stdout.len() <= 65536 && out.stdout.ends_with(b"\n"));
        let again = run();
        assert!(again.status.success() && again.stderr.is_empty());
        assert_eq!(again.stdout, out.stdout);
        let result: Value = serde_json::from_slice(&out.stdout).expect("v4 JSON");
        assert_eq!(result["formatVersion"], 4);
        assert_eq!(result["mode"], "trace-equality");
        result
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("owned cleanup");
    }
}
fn source_compare(ir: Expr, source: &str) -> Value {
    // Separate native evidence invocations, all BEFORE Go, with aggregate work.
    let saved = ir.clone();
    let constructed =
        recursive_default_scheduling::execute(&ir, 4096).expect("native construction");
    let mut charged = constructed.work;
    let mut expected = Vec::new();
    for batch in &constructed.batches {
        let audit =
            execute_audited(&batch.trials, 4096 - charged).expect("native audit remaining budget");
        charged += audit.report.work;
        assert_eq!(
            (audit.report.left_drops, audit.report.right_drops),
            (batch.left_drops, batch.right_drops)
        );
        assert_eq!(
            audit.report.results,
            batch
                .states
                .iter()
                .map(|s| leaf(s.value, s.mode))
                .collect::<Vec<_>>()
        );
        expected.push(native_projection(batch, &audit.appends));
    }
    assert!(charged <= 4096);
    assert_eq!(ir, saved);
    let trace = Files::new(source).trace();
    let batches = trace["batches"].as_array().expect("batches");
    assert_eq!(batches.len(), expected.len());
    for ((got, want), batch) in batches.iter().zip(expected).zip(&constructed.batches) {
        assert_eq!(got["id"], batch.id);
        assert_eq!(got["depth"], batch.depth);
        assert_eq!(
            go_projection(&got["phase"]),
            want,
            "scalar transcript projection (NOT partial-kind certificate): {source}"
        );
    }
    trace
}
#[test]
fn source_projection_checks_partial_coalescence_and_first_origin_separately() {
    for (xs, source) in [
        (vec![(false, 1), (true, 1), (false, 2)], "1|*1|2"),
        (vec![(true, 1), (false, 1), (false, 2)], "*1|1|2"),
    ] {
        source_compare(g(&xs), source);
    }
    let trace = source_compare(
        a(vec![
            g(&[(false, 1), (true, 1), (false, 2)]),
            g(&[(false, 1), (false, 2)]),
        ]),
        "(1|*1|2)&(1|2)",
    );
    let observed = &trace["batches"][0]["phase"];
    assert_eq!(observed["runMode"], "attempt-only");
    assert_eq!(observed["appends"][1]["comparisons"][0]["kind"], "partial");
    assert_eq!(observed["appends"][1]["comparisons"][0]["accepted"], true);
    source_compare(o(vec![(false, marked()), (false, v(3))]), "((*1|2)|3)");
    source_compare(
        o(vec![(false, a(vec![marked(), v(1)])), (false, v(3))]),
        "(((*1|2)&1)|3)",
    );
    source_compare(
        a(vec![
            v(2),
            o(vec![(false, g(&[(true, 1), (false, 3)])), (false, v(2))]),
        ]),
        "2&((*1|3)|2)",
    );
    source_compare(a(vec![v(1), marked(), v(2)]), "1&(*1|2)&2");
    source_compare(
        g(&[(true, i64::MIN), (false, i64::MAX)]),
        "*-9223372036854775808|9223372036854775807",
    );
}
#[test]
fn supplemental_nested_counterexample_and_asymmetry_scalar_projections() {
    // Differential evidence, NOT independent hand-authored audit goldens.
    let cases = vec![
        (
            o(vec![(false, g(&[(true, 3), (false, 1)])), (false, v(3))]),
            o(vec![
                (false, marked()),
                (false, g(&[(true, 3), (false, 2)])),
            ]),
            "(*3|1)|3",
            "(*1|2)|(*3|2)",
        ),
        (
            o(vec![(false, marked()), (true, v(3))]),
            g(&[(true, 2), (false, 1)]),
            "(*1|2)|*3",
            "*2|1",
        ),
        (
            o(vec![
                (false, a(vec![marked(), g(&[(true, 2), (false, 1)])])),
                (false, v(1)),
            ]),
            marked(),
            "((*1|2)&(*2|1))|1",
            "*1|2",
        ),
        (
            o(vec![(false, marked()), (true, v(3))]),
            g(&[(false, 1), (false, 2)]),
            "(*1|2)|*3",
            "1|2",
        ),
        (
            o(vec![
                (true, o(vec![(false, marked()), (false, v(3))])),
                (false, g(&[(true, 4), (false, 5)])),
            ]),
            g(&[(false, 1), (false, 4)]),
            "*((*1|2)|3)|(*4|5)",
            "1|4",
        ),
        (
            marked(),
            o(vec![(false, marked()), (true, v(3))]),
            "*1|2",
            "(*1|2)|*3",
        ),
        (
            o(vec![(true, g(&[(true, 3), (false, 1)])), (false, v(2))]),
            g(&[(false, 1), (false, 2)]),
            "*(*3|1)|2",
            "1|2",
        ),
        (
            o(vec![
                (true, a(vec![marked(), g(&[(false, 1), (true, 2)])])),
                (false, v(3)),
            ]),
            g(&[(true, 3), (false, 1), (false, 2)]),
            "*((*1|2)&(1|*2))|3",
            "*3|1|2",
        ),
    ];
    for (left, right, ls, rs) in cases {
        source_compare(
            a(vec![left.clone(), right.clone()]),
            &format!("({ls})&({rs})"),
        );
        source_compare(a(vec![right, left]), &format!("({rs})&({ls})"));
    }
}
#[test]
fn fresh_audit_ownership_and_formula_cover_bounded_mode_combinations() {
    for current in [M, D, N] {
        for left in [M, D, N] {
            for origin in [M, D, N] {
                for incoming in [M, D, N] {
                    let input = [
                        Trial {
                            mode: current,
                            left_mode: left,
                            left_origin: origin,
                            incoming,
                            body: Body::Scalar(7),
                        },
                        scalar(7, D),
                        scalar(8, N),
                    ];
                    let before = input.clone();
                    let old = execute(&input, 4096).expect("old bounded tuple");
                    let audited = execute_audited(&input, 4096).expect("bounded audited tuple");
                    assert_eq!(old.results, audited.report.results);
                    assert_eq!(
                        (old.left_drops, old.right_drops, old.has_non_maybe),
                        (
                            audited.report.left_drops,
                            audited.report.right_drops,
                            audited.report.has_non_maybe
                        )
                    );
                    assert_eq!(input, before);
                    let c = audited
                        .appends
                        .iter()
                        .map(|e| e.comparisons.len())
                        .sum::<usize>();
                    assert_eq!(
                        audited.report.work,
                        old.work + 4 + 5 * audited.appends.len() + 4 * c + old.results.len()
                    );
                }
            }
        }
    }
}
