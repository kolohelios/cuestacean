//! Falsifiable recursive scalar IR hypothesis; never a production default gate.
#[path = "../../spikes/batch_defaults.rs"]
mod batch_defaults;

#[test]
fn audited_promotion_keeps_first_position_without_charging_old_api() {
    use batch_defaults::{Body, Leaf, Mode, Trial, execute_audited};
    let input = [Mode::Not, Mode::Default].map(|incoming| Trial {
        mode: Mode::Maybe,
        left_mode: Mode::Maybe,
        left_origin: Mode::Maybe,
        incoming,
        body: Body::Scalar(7),
    });
    let old = batch_defaults::execute(&input, 4096).expect("old path");
    let audit = execute_audited(&input, 4096).expect("audited path");
    assert_eq!((old.work, audit.report.work), (13, 32));
    assert_eq!(old.results, audit.report.results);
    assert_eq!(audit.appends.len(), 2);
    let first = &audit.appends[0];
    let next = &audit.appends[1];
    let comparison = &next.comparisons[0];
    assert_eq!((first.occurrence.trial, first.occurrence.child), (0, None));
    assert_eq!((next.occurrence.trial, next.occurrence.child), (1, None));
    assert_eq!(
        first.candidate,
        Leaf {
            value: 7,
            mode: Mode::Not
        }
    );
    assert_eq!(
        next.candidate,
        Leaf {
            value: 7,
            mode: Mode::Default
        }
    );
    assert_eq!(
        (first.index, first.coalesced, next.index, next.coalesced),
        (0, false, 0, true)
    );
    assert!(first.comparisons.is_empty() && comparison.matched);
    assert_eq!(
        (comparison.occurrence.trial, comparison.occurrence.child),
        (0, None)
    );
    assert_eq!(
        comparison.existing,
        Leaf {
            value: 7,
            mode: Mode::Not
        }
    );
}

#[path = "../../spikes/flat_default_sources.rs"]
mod flat_default_sources;
#[path = "../../spikes/recursive_default_scheduling.rs"]
mod recursive_default_scheduling;
use Mode::{Default as D, Maybe as M, Not as N};
use batch_defaults::{Body, Error, Leaf, Mode, Trial};
use recursive_default_scheduling::{Alternative, Batch, Expr, State, execute};
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
fn states(xs: &[(i64, Mode, Mode)]) -> Vec<State> {
    xs.iter()
        .map(|&(value, mode, origin)| State {
            value,
            mode,
            origin,
        })
        .collect()
}
fn scalar(n: i64, mode: Mode, p: Mode, origin: Mode, incoming: Mode) -> (Trial, Vec<Mode>) {
    (
        Trial {
            mode,
            left_mode: p,
            left_origin: origin,
            incoming,
            body: Body::Scalar(n),
        },
        vec![],
    )
}
fn nested(
    mode: Mode,
    p: Mode,
    origin: Mode,
    incoming: Mode,
    children: &[(i64, Mode, Mode)],
) -> (Trial, Vec<Mode>) {
    (
        Trial {
            mode,
            left_mode: p,
            left_origin: origin,
            incoming,
            body: Body::Nested(
                children
                    .iter()
                    .map(|&(value, mode, _)| Leaf { value, mode })
                    .collect(),
            ),
        },
        children.iter().map(|&(_, _, origin)| origin).collect(),
    )
}
fn batch(
    id: usize,
    depth: usize,
    rows: Vec<(Trial, Vec<Mode>)>,
    drops: [bool; 2],
    xs: &[(i64, Mode, Mode)],
) -> Batch {
    let (trials, nested_origins) = rows.into_iter().unzip();
    Batch {
        id,
        depth,
        trials,
        nested_origins,
        left_drops: drops[0],
        right_drops: drops[1],
        states: states(xs),
    }
}
fn word(m: Mode) -> &'static str {
    match m {
        M => "maybe",
        D => "default",
        N => "not",
    }
}
fn result_json(s: &State) -> Value {
    json!({"mode":word(s.mode),"originalMode":word(s.origin),"value":s.value})
}
fn batches_json(bs: &[Batch]) -> Value {
    Value::Array(bs.iter().map(|b| {
        assert_eq!(b.trials.len(),b.nested_origins.len());
        let trials=b.trials.iter().zip(&b.nested_origins).map(|(t,origins)| {
            let (value,children)=match &t.body {
                Body::Scalar(n)=>{ assert!(origins.is_empty()); (Some(*n),vec![]) },
                Body::Nested(xs)=>{ assert_eq!(xs.len(),origins.len()); (None,xs.iter().zip(origins).map(|(x,&origin)|json!({"mode":word(x.mode),"originalMode":word(origin),"value":x.value})).collect()) }
            };
            json!({"mode":word(t.mode),"originalMode":word(t.incoming),"leftMode":word(t.left_mode),"leftOriginalMode":word(t.left_origin),"value":value,"nested":children})
        }).collect::<Vec<_>>();
        json!({"id":b.id,"depth":b.depth,"leftDropsDefault":b.left_drops,"rightDropsDefault":b.right_drops,"trials":trials,"results":b.states.iter().map(result_json).collect::<Vec<_>>()})
    }).collect())
}
struct Files(PathBuf);
impl Files {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cuestacean-recursive-{}-{}",
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
    fn observe(&self, semantic: &str) -> Value {
        let output = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-history", "probe.cue", "probe"],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert!(output.stdout.len() <= 65536 && output.stdout.ends_with(b"\n"));
        let second = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-history", "probe.cue", "probe"],
        );
        assert!(second.status.success() && second.stderr.is_empty());
        assert_eq!(second.stdout, output.stdout);
        let trace: Value = serde_json::from_slice(&output.stdout).expect("v3 JSON");
        assert_eq!(trace["formatVersion"], 3);
        assert_eq!(trace["mode"], "trace-history");
        let (flag, rest) = semantic.split_once(':').expect("hand public expectation");
        let (status, value) = match rest {
            "incomplete" | "conflict" => (rest, None),
            _ => ("concrete", Some(rest.parse::<i64>().expect("i64"))),
        };
        assert_eq!(
            trace["defaultSelected"],
            flag == "T",
            "{}",
            fs::read_to_string(self.0.join("probe.cue")).expect("authored diagnostic source")
        );
        assert_eq!(trace["status"], status);
        assert_eq!(trace["value"], json!(value));
        let api = self.run(
            "CUESTACEAN_DEFAULT_OBSERVER",
            &["inspect", "probe.cue", "probe"],
        );
        assert!(api.status.success() && api.stderr.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&api.stdout).expect("independent API"),
            json!({"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":flag=="T","status":status,"value":value})
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
                .expect("pinned inline-source CLI")
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
        trace
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("owned scratch cleanup");
    }
}
fn check(
    ir: Expr,
    source: &str,
    expected: Vec<Batch>,
    final_states: &[(i64, Mode, Mode)],
    semantic: &str,
) {
    let saved = ir.clone();
    let r = execute(&ir, 4096).expect("native BEFORE observations");
    assert_eq!(r.batches, expected, "hand batch construction: {source}");
    assert_eq!(r.states, states(final_states));
    assert!((1..=4096).contains(&r.work));
    assert_eq!(execute(&ir, 4096).expect("fresh repeat"), r);
    assert_eq!(ir, saved);
    let observed = Files::new(source).observe(semantic);
    assert_eq!(
        batches_json(&expected),
        observed["batches"],
        "complete handwritten/native/oracle comparison: {source}"
    );
}

#[test]
fn nested_wrapper_unroll_keeps_child_origins_and_dynamic_entry_order() {
    for (mark, source, last_origin) in [(false, "((*1|2)|3)", M), (true, "(*(*1|2)|3)", N)] {
        let incoming = if mark { D } else { M };
        check(
            o(vec![(mark, marked()), (false, v(3))]),
            source,
            vec![
                batch(
                    0,
                    0,
                    vec![
                        nested(M, M, M, incoming, &[(1, D, D), (2, N, N)]),
                        scalar(3, M, M, M, last_origin),
                    ],
                    [true, !mark],
                    &[(1, D, D), (2, N, N), (3, N, last_origin)],
                ),
                batch(
                    1,
                    1,
                    vec![scalar(1, M, M, incoming, D), scalar(2, M, M, incoming, N)],
                    [!mark, false],
                    &[(1, D, D), (2, N, N)],
                ),
            ],
            &[(1, D, D), (2, N, N), (3, N, last_origin)],
            "T:1",
        );
    }
}

#[test]
fn nested_duplicate_promotion_retains_first_origin_and_exact_i64s() {
    for (xs, source, origin) in [
        (vec![(false, 1), (true, 1), (false, 2)], "((1|*1|2)|3)", N),
        (vec![(true, 1), (false, 1), (false, 2)], "((*1|1|2)|3)", D),
    ] {
        let rows = xs
            .iter()
            .map(|&(mark, n)| scalar(n, M, M, M, if mark { D } else { N }))
            .collect();
        check(
            o(vec![(false, g(&xs)), (false, v(3))]),
            source,
            vec![
                batch(
                    0,
                    0,
                    vec![
                        nested(M, M, M, M, &[(1, D, origin), (2, N, N)]),
                        scalar(3, M, M, M, M),
                    ],
                    [true, true],
                    &[(1, D, origin), (2, N, N), (3, N, M)],
                ),
                batch(1, 1, rows, [true, false], &[(1, D, origin), (2, N, N)]),
            ],
            &[(1, D, origin), (2, N, N), (3, N, M)],
            "T:1",
        );
    }
    check(
        v(i64::MIN),
        "-9223372036854775808",
        vec![],
        &[(i64::MIN, M, M)],
        "F:-9223372036854775808",
    );
    check(
        g(&[(true, i64::MIN), (false, i64::MAX)]),
        "*-9223372036854775808|9223372036854775807",
        vec![batch(
            0,
            0,
            vec![scalar(i64::MIN, M, M, M, D), scalar(i64::MAX, M, M, M, N)],
            [true, false],
            &[(i64::MIN, D, D), (i64::MAX, N, N)],
        )],
        &[(i64::MIN, D, D), (i64::MAX, N, N)],
        "T:-9223372036854775808",
    );
}

#[test]
fn singleton_attempt_overwrite_is_not_a_batch_or_root_history_overwrite() {
    check(
        o(vec![(false, a(vec![marked(), v(1)])), (false, v(3))]),
        "(((*1|2)&1)|3)",
        vec![
            batch(
                0,
                0,
                vec![scalar(1, D, M, M, M), scalar(3, M, M, M, M)],
                [true, true],
                &[(1, M, M), (3, M, M)],
            ),
            batch(
                1,
                1,
                vec![scalar(1, M, M, M, D)],
                [true, false],
                &[(1, D, D)],
            ),
        ],
        &[(1, M, M), (3, M, M)],
        "F:incomplete",
    );
    check(
        o(vec![(true, a(vec![marked(), v(1)])), (false, v(3))]),
        "(*((*1|2)&1)|3)",
        vec![
            batch(
                0,
                0,
                vec![scalar(1, D, M, M, D), scalar(3, M, M, M, N)],
                [true, false],
                &[(1, D, D), (3, N, N)],
            ),
            batch(
                1,
                1,
                vec![scalar(1, M, M, D, D)],
                [false, false],
                &[(1, D, D)],
            ),
        ],
        &[(1, D, D), (3, N, N)],
        "T:1",
    );
    check(
        a(vec![marked(), v(1)]),
        "(*1|2)&1",
        vec![batch(
            0,
            0,
            vec![scalar(1, M, M, M, D)],
            [true, false],
            &[(1, D, D)],
        )],
        &[(1, D, D)],
        "F:1",
    );
}

#[test]
fn failed_nested_attempts_retain_child_batches_and_do_not_become_parent_trials() {
    check(
        a(vec![
            v(2),
            o(vec![(false, g(&[(true, 1), (false, 3)])), (false, v(2))]),
        ]),
        "2&((*1|3)|2)",
        vec![
            batch(
                0,
                0,
                vec![scalar(2, M, M, M, M)],
                [true, true],
                &[(2, M, M)],
            ),
            batch(1, 1, vec![], [true, true], &[]),
        ],
        &[(2, M, M)],
        "F:2",
    );
    check(
        a(vec![v(1), marked(), v(2)]),
        "1&(*1|2)&2",
        vec![batch(0, 0, vec![], [true, true], &[])],
        &[],
        "F:conflict",
    );
    check(
        a(vec![v(1), a(vec![v(2), v(1)])]),
        "1&(2&1)",
        vec![],
        &[],
        "F:conflict",
    );
}

fn research_cases() -> Vec<(
    Expr,
    Expr,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    vec![
        (
            marked(),
            g(&[(false, 1), (false, 2), (true, 3)]),
            "*1|2",
            "1|2|*3",
            "T:1",
            "T:1",
        ),
        (
            o(vec![
                (false, marked()),
                (false, g(&[(true, 2), (false, 1)])),
            ]),
            marked(),
            "(*1|2)|(*2|1)",
            "*1|2",
            "T:1",
            "T:incomplete",
        ),
        (
            o(vec![
                (false, a(vec![marked(), g(&[(true, 1), (false, 3)])])),
                (false, v(3)),
            ]),
            g(&[(true, 3), (false, 1)]),
            "((*1|2)&(*1|3))|3",
            "*3|1",
            "T:3",
            "T:incomplete",
        ),
        (
            o(vec![
                (true, o(vec![(false, marked()), (false, v(3))])),
                (false, g(&[(true, 4), (false, 5)])),
            ]),
            g(&[(false, 1), (false, 2), (false, 3), (false, 4), (false, 5)]),
            "*((*1|2)|3)|(*4|5)",
            "1|2|3|4|5",
            "T:1",
            "T:incomplete",
        ),
        (
            g(&[(true, 2), (false, 3)]),
            o(vec![(false, marked()), (false, v(3))]),
            "*2|3",
            "(*1|2)|3",
            "T:2",
            "T:2",
        ),
        (
            o(vec![(false, marked()), (true, v(3))]),
            g(&[(true, 2), (false, 1)]),
            "(*1|2)|*3",
            "*2|1",
            "F:incomplete",
            "T:incomplete",
        ),
        (
            o(vec![(false, g(&[(true, 3), (false, 1)])), (false, v(3))]),
            o(vec![
                (false, marked()),
                (false, g(&[(true, 3), (false, 2)])),
            ]),
            "(*3|1)|3",
            "(*1|2)|(*3|2)",
            "T:incomplete",
            "T:incomplete",
        ),
        (
            o(vec![
                (false, a(vec![marked(), g(&[(true, 2), (false, 1)])])),
                (false, v(1)),
            ]),
            marked(),
            "((*1|2)&(*2|1))|1",
            "*1|2",
            "F:incomplete",
            "T:incomplete",
        ),
        (
            o(vec![(false, marked()), (true, v(3))]),
            g(&[(false, 1), (false, 2)]),
            "(*1|2)|*3",
            "1|2",
            "F:incomplete",
            "F:incomplete",
        ),
        (
            o(vec![
                (true, o(vec![(false, marked()), (false, v(3))])),
                (false, g(&[(true, 4), (false, 5)])),
            ]),
            g(&[(false, 1), (false, 4)]),
            "*((*1|2)|3)|(*4|5)",
            "1|4",
            "T:1",
            "T:1",
        ),
        (
            marked(),
            o(vec![(false, marked()), (true, v(3))]),
            "*1|2",
            "(*1|2)|*3",
            "T:1",
            "F:incomplete",
        ),
        (
            o(vec![(true, g(&[(true, 3), (false, 1)])), (false, v(2))]),
            g(&[(false, 1), (false, 2)]),
            "*(*3|1)|2",
            "1|2",
            "F:incomplete",
            "T:1",
        ),
        (
            o(vec![
                (true, a(vec![marked(), g(&[(false, 1), (true, 2)])])),
                (false, v(3)),
            ]),
            g(&[(true, 3), (false, 1), (false, 2)]),
            "*((*1|2)&(1|*2))|3",
            "*3|1|2",
            "F:incomplete",
            "T:2",
        ),
    ]
}

#[test]
fn preserved_concrete_failures_and_order_asymmetries_test_the_recursive_hypothesis() {
    // Secondary falsification matrix, NOT replacement for handwritten full cases.
    // No captured observation supplies an Expr, successful trial or survivor.
    // Public flags retain the old observer's T:incomplete versus F:incomplete
    // distinction: neither is an Absent/Bottom or native-mode classifier.
    let mut returned_default_with_not_predecessor = false;
    for (left, right, ls, rs, forward, reverse) in research_cases() {
        for (ir, source, semantic) in [
            (
                a(vec![left.clone(), right.clone()]),
                format!("({ls})&({rs})"),
                forward,
            ),
            (a(vec![right, left]), format!("({rs})&({ls})"), reverse),
        ] {
            let native = execute(&ir, 4096).expect("native construction BEFORE capture");
            returned_default_with_not_predecessor |= native.batches.iter().any(|b| {
                b.trials
                    .iter()
                    .any(|t| t.mode == D && t.left_mode == N && t.left_origin == N)
            });
            let observed = Files::new(&source).observe(semantic);
            assert_eq!(
                batches_json(&native.batches),
                observed["batches"],
                "scoped hypothesis mismatch must be preserved: {source}"
            );
        }
    }
    assert!(
        returned_default_with_not_predecessor,
        "returned mode must not substitute for predecessor facts"
    );
}

#[test]
fn flat_compatibility_and_old_private_api_controls_do_not_widen_visibility() {
    use flat_default_sources::{Alternative as F, Clause};
    let flat = [
        Clause::Or(vec![
            F {
                marked: true,
                value: 1,
            },
            F {
                marked: false,
                value: 2,
            },
        ]),
        Clause::Or(vec![
            F {
                marked: true,
                value: 2,
            },
            F {
                marked: false,
                value: 1,
            },
        ]),
        Clause::Or(vec![
            F {
                marked: false,
                value: 1,
            },
            F {
                marked: false,
                value: 2,
            },
        ]),
    ];
    let ir = a(vec![
        marked(),
        g(&[(true, 2), (false, 1)]),
        g(&[(false, 1), (false, 2)]),
    ]);
    let literal = flat_default_sources::execute(&[Clause::Value(9)], 4096)
        .expect("old ordinary pre-refinement");
    assert!(literal.batches.is_empty() && literal.work > 0);
    assert_eq!(
        literal.states,
        vec![flat_default_sources::State {
            value: 9,
            mode: M,
            origin: M
        }]
    );
    let old = flat_default_sources::execute(&flat, 4096).expect("unchanged flat API");
    let new = execute(&ir, 4096).expect("recursive native");
    assert_eq!(new.batches.len(), old.batches.len());
    for (x, y) in new.batches.iter().zip(&old.batches) {
        assert_eq!(x.trials, y.trials);
        assert_eq!((x.left_drops, x.right_drops), (y.left_drops, y.right_drops));
        assert_eq!(
            x.states
                .iter()
                .map(|s| (s.value, s.mode, s.origin))
                .collect::<Vec<_>>(),
            y.states
                .iter()
                .map(|s| (s.value, s.mode, s.origin))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(
        new.states
            .iter()
            .map(|s| (s.value, s.mode, s.origin))
            .collect::<Vec<_>>(),
        old.states
            .iter()
            .map(|s| (s.value, s.mode, s.origin))
            .collect::<Vec<_>>()
    );
    assert!(old.work > 0);
    let input = [nested(M, M, M, D, &[(3, D, D), (1, N, N)]).0];
    let r = batch_defaults::execute(&input, 4096).expect("unchanged nested primitive");
    assert!(r.has_non_maybe && r.work > 0 && r.left_drops && !r.right_drops);
    assert_eq!(
        r.results,
        vec![Leaf { value: 3, mode: D }, Leaf { value: 1, mode: N }]
    );
}

#[test]
fn native_only_structural_raw_product_and_cumulative_work_bounds_recover() {
    for bad in [a(vec![]), a(vec![v(1)]), o(vec![]), g(&[(false, 1)])] {
        assert!(matches!(execute(&bad, 4096), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 0), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 4097), Err(Error::Limit(_))));
    }
    let mut deep = v(1);
    for _ in 1..8 {
        deep = a(vec![deep, v(1)]);
    }
    assert!(execute(&deep, 4096).is_ok());
    assert!(matches!(
        execute(&a(vec![deep, v(1)]), 4096),
        Err(Error::Limit(_))
    ));
    let group = g(&[(false, 1); 16]);
    assert!(execute(&group, 4096).is_ok());
    assert!(matches!(
        execute(&g(&[(false, 1); 17]), 4096),
        Err(Error::Limit(_))
    ));
    let mut nodes = vec![group.clone(); 3];
    nodes.push(g(&[(false, 1); 8]));
    nodes.extend(vec![v(1); 11]);
    assert!(
        execute(&a(nodes.clone()), 4096).is_ok(),
        "128 Expr/Alternative occurrences"
    );
    nodes.push(v(1));
    assert!(matches!(execute(&a(nodes), 4096), Err(Error::Limit(_))));
    assert!(execute(&a(vec![g(&[(false, 1); 2]); 16]), 4096).is_ok());
    let mut seventeen = vec![g(&[(false, 1); 2]); 15];
    seventeen.push(a(vec![g(&[(false, 1); 2]); 2]));
    assert!(matches!(execute(&a(seventeen), 4096), Err(Error::Limit(_))));
    let eight = (0..8).map(|n| (false, n)).collect::<Vec<_>>();
    let raw = a(vec![g(&eight), g(&eight)]);
    assert_eq!(execute(&raw, 4096).expect("64 raw pairs").states.len(), 8);
    let five = (0..5).map(|n| (false, n)).collect::<Vec<_>>();
    let thirteen = (0..13).map(|n| (false, n)).collect::<Vec<_>>();
    assert!(
        matches!(
            execute(&a(vec![g(&five), g(&thirteen)]), 4096),
            Err(Error::Limit(_))
        ),
        "65 raw reject before filtering"
    );
    let ir = o(vec![(false, marked()), (false, v(3))]);
    let r = execute(&ir, 4096).expect("bounded recursive work");
    let minimum = (1..=4096)
        .find(|&n| execute(&ir, n).is_ok())
        .expect("minimum");
    assert_eq!(minimum, r.work);
    assert!(matches!(execute(&ir, minimum - 1), Err(Error::Limit(_))));
    assert_eq!(execute(&ir, minimum).expect("exact recovery"), r);
    assert!(matches!(execute(&ir, 0), Err(Error::Limit(_))));
    assert_eq!(
        execute(&v(i64::MIN), 4096).expect("i64 minimum").states,
        states(&[(i64::MIN, M, M)])
    );
    assert_eq!(
        execute(&v(i64::MAX), 4096).expect("i64 maximum").states,
        states(&[(i64::MAX, M, M)])
    );
}
