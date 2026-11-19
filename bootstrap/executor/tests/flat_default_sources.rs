//! Native authored flat IR, never oracle-fed trials or a general default solver.
#[path = "../../spikes/batch_defaults.rs"]
mod batch_defaults;
#[path = "../../spikes/flat_default_sources.rs"]
mod flat_default_sources;
use Mode::{Default as D, Maybe as M, Not as N};
use batch_defaults::{Body, Error, Leaf, Mode, Trial};
use flat_default_sources::{Alternative, Batch, Clause, State, execute};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn or(xs: &[(bool, i64)]) -> Clause {
    Clause::Or(
        xs.iter()
            .map(|&(marked, value)| Alternative { marked, value })
            .collect(),
    )
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
fn batch(
    rows: &[(i64, Mode, Mode, Mode)],
    drops: [bool; 2],
    expected: &[(i64, Mode, Mode)],
) -> Batch {
    Batch {
        trials: rows
            .iter()
            .map(|&(value, mode, left_origin, incoming)| Trial {
                mode,
                left_mode: mode,
                left_origin,
                incoming,
                body: Body::Scalar(value),
            })
            .collect(),
        left_drops: drops[0],
        right_drops: drops[1],
        states: states(expected),
    }
}
fn word(m: Mode) -> &'static str {
    match m {
        M => "maybe",
        D => "default",
        N => "not",
    }
}
fn state_json(s: &State) -> Value {
    json!({"mode":word(s.mode),"originalMode":word(s.origin),"value":s.value})
}
fn trial_json(t: &Trial) -> Value {
    let Body::Scalar(value) = t.body else {
        panic!("flat trial only")
    };
    json!({"mode":word(t.mode),"originalMode":word(t.incoming),"leftMode":word(t.left_mode),
        "leftOriginalMode":word(t.left_origin),"value":value,"nested":[]})
}
struct Files(PathBuf);
impl Files {
    fn new(expr: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cuestacean-flat-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).expect("exclusive owned scratch");
        fs::write(p.join("probe.cue"), format!("probe: {expr}\n")).expect("authored source");
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
    fn history(&self) -> (Value, Vec<u8>) {
        let out = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-history", "probe.cue", "probe"],
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stderr.is_empty() && out.stdout.len() <= 65536 && out.stdout.ends_with(b"\n"));
        let v: Value = serde_json::from_slice(&out.stdout).expect("history JSON");
        assert_eq!(v["formatVersion"], 3);
        assert_eq!(v["mode"], "trace-history");
        let again = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-history", "probe.cue", "probe"],
        );
        assert!(again.status.success() && again.stderr.is_empty());
        assert_eq!(again.stdout, out.stdout);
        let batches = v["batches"].as_array().expect("batches");
        assert!(batches.len() <= 128);
        let mut records = 0;
        for (id, b) in batches.iter().enumerate() {
            assert_eq!(b["id"], id);
            let ts = b["trials"].as_array().expect("trials");
            let rs = b["results"].as_array().expect("results");
            assert!(ts.len() <= 64 && rs.len() <= 64);
            records += 3 * ts.len() + 2 * rs.len();
            for t in ts {
                for key in ["mode", "originalMode", "leftMode", "leftOriginalMode"] {
                    assert!(matches!(t[key].as_str(), Some("maybe" | "default" | "not")));
                }
                let nested = t["nested"].as_array().expect("nested");
                assert!(nested.len() <= 64);
                records += 2 * nested.len();
                for r in nested.iter().chain(rs) {
                    for key in ["mode", "originalMode"] {
                        assert!(matches!(r[key].as_str(), Some("maybe" | "default" | "not")));
                    }
                }
            }
        }
        assert!(records <= 4096);
        (v, out.stdout)
    }
    fn semantics(&self, v: &Value, expected: &str) {
        let (flag, rest) = expected.split_once(':').expect("hand semantic expectation");
        let selected = flag == "T";
        let (status, value) = match rest {
            "incomplete" | "conflict" => (rest, None),
            _ => ("concrete", Some(rest.parse::<i64>().expect("exact i64"))),
        };
        assert_eq!(v["defaultSelected"], selected);
        assert_eq!(v["status"], status);
        assert_eq!(v["value"], json!(value));
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
                .expect("independent inline source CLI")
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
        fs::remove_dir_all(&self.0).expect("remove owned scratch");
    }
}
fn check(
    ir: Vec<Clause>,
    source: &str,
    expected_batches: Vec<Batch>,
    expected_states: &[(i64, Mode, Mode)],
    semantic: &str,
) {
    let saved = ir.clone();
    // Execute the authored IR BEFORE observing upstream; no trace-to-native path.
    let report = execute(&ir, 4096).expect("bounded native construction");
    assert_eq!(report.batches, expected_batches, "hand batches: {source}");
    assert_eq!(
        report.states,
        states(expected_states),
        "hand final states: {source}"
    );
    assert!((1..=4096).contains(&report.work));
    assert_eq!(execute(&ir, 4096).expect("fresh repeat"), report);
    assert_eq!(ir, saved);
    let files = Files::new(source);
    let (v, _) = files.history();
    files.semantics(&v, semantic);
    let bs = v["batches"].as_array().expect("batches");
    assert_eq!(bs.len(), expected_batches.len(), "batch count: {source}");
    for (observed, expected) in bs.iter().zip(&expected_batches) {
        assert_eq!(observed["depth"], 0, "only flat groups");
        assert_eq!(observed["leftDropsDefault"], expected.left_drops);
        assert_eq!(observed["rightDropsDefault"], expected.right_drops);
        assert_eq!(
            observed["trials"],
            Value::Array(expected.trials.iter().map(trial_json).collect()),
            "constructed trials: {source}"
        );
        assert_eq!(
            observed["results"],
            Value::Array(expected.states.iter().map(state_json).collect()),
            "current/original history: {source}"
        );
    }
}
fn marked() -> Clause {
    or(&[(true, 1), (false, 2)])
}
fn first_marked() -> Batch {
    batch(
        &[(1, M, M, D), (2, M, M, N)],
        [true, false],
        &[(1, D, D), (2, N, N)],
    )
}

#[test]
fn native_two_and_three_batch_construction_matches_hand_inputs_and_history() {
    check(
        vec![marked()],
        "*1|2",
        vec![first_marked()],
        &[(1, D, D), (2, N, N)],
        "T:1",
    );
    check(
        vec![marked(), or(&[(false, 1), (false, 2), (true, 3)])],
        "(*1|2)&(1|2|*3)",
        vec![
            first_marked(),
            batch(
                &[(1, D, D, N), (2, N, N, N)],
                [false, true],
                &[(1, D, N), (2, N, N)],
            ),
        ],
        &[(1, D, N), (2, N, N)],
        "T:1",
    );
    let opposite = or(&[(true, 2), (false, 1)]);
    let conflict_batch = || {
        batch(
            &[(1, D, D, N), (2, N, N, D)],
            [false, false],
            &[(1, N, N), (2, N, D)],
        )
    };
    check(
        vec![marked(), opposite.clone()],
        "(*1|2)&(*2|1)",
        vec![first_marked(), conflict_batch()],
        &[(1, N, N), (2, N, D)],
        "F:incomplete",
    );
    let third = || {
        batch(
            &[(1, N, N, M), (2, N, D, M)],
            [false, true],
            &[(1, N, M), (2, N, M)],
        )
    };
    check(
        vec![marked(), opposite.clone(), or(&[(false, 1), (false, 2)])],
        "(*1|2)&(*2|1)&(1|2)",
        vec![first_marked(), conflict_batch(), third()],
        &[(1, N, M), (2, N, M)],
        "F:incomplete",
    );
    check(
        vec![marked(), opposite, or(&[(false, 1), (false, 2)]), marked()],
        "(*1|2)&(*2|1)&(1|2)&(*1|2)",
        vec![
            first_marked(),
            conflict_batch(),
            third(),
            batch(
                &[(1, N, M, D), (2, N, M, N)],
                [true, false],
                &[(1, D, D), (2, N, N)],
            ),
        ],
        &[(1, D, D), (2, N, N)],
        "T:1",
    );
}

#[test]
fn duplicate_promotion_preserves_first_origin_not_promoted_current_mode() {
    for (alternatives, source, origin) in [
        (
            vec![(false, 1), (true, 1), (false, 2)],
            "(1|*1|2)&(*2|1)",
            N,
        ),
        (
            vec![(true, 1), (false, 1), (false, 2)],
            "(*1|1|2)&(*2|1)",
            D,
        ),
    ] {
        let first_rows = alternatives
            .iter()
            .map(|&(mark, value)| (value, M, M, if mark { D } else { N }))
            .collect::<Vec<_>>();
        check(
            vec![or(&alternatives), or(&[(true, 2), (false, 1)])],
            source,
            vec![
                batch(&first_rows, [true, false], &[(1, D, origin), (2, N, N)]),
                batch(
                    &[(1, D, origin, N), (2, N, N, D)],
                    [false, false],
                    &[(1, N, N), (2, N, D)],
                ),
            ],
            &[(1, N, N), (2, N, D)],
            "F:incomplete",
        );
    }
}

#[test]
fn ordinary_literals_pre_refine_every_group_and_public_flags_are_not_native_modes() {
    for (ir, source) in [
        (vec![marked(), Clause::Value(2)], "(*1|2)&2"),
        (vec![Clause::Value(2), marked()], "2&(*1|2)"),
    ] {
        check(
            ir,
            source,
            vec![batch(&[(2, M, M, N)], [true, true], &[(2, M, N)])],
            &[(2, M, N)],
            "F:2",
        );
    }
    // A Default batch mode does NOT imply public Default() reports selection.
    check(
        vec![marked(), Clause::Value(1)],
        "(*1|2)&1",
        vec![batch(&[(1, M, M, D)], [true, false], &[(1, D, D)])],
        &[(1, D, D)],
        "F:1",
    );
    check(
        vec![
            or(&[(false, 1), (false, 2)]),
            or(&[(false, 2), (false, 3)]),
            or(&[(true, 2), (false, 3)]),
        ],
        "(1|2)&(2|3)&(*2|3)",
        vec![
            batch(
                &[(1, M, M, M), (2, M, M, M)],
                [true, true],
                &[(1, M, M), (2, M, M)],
            ),
            batch(&[(2, M, M, M)], [true, true], &[(2, M, M)]),
            batch(&[(2, M, M, D)], [true, false], &[(2, D, D)]),
        ],
        &[(2, D, D)],
        "F:2",
    );
    check(vec![Clause::Value(-7)], "-7", vec![], &[(-7, M, M)], "F:-7");
}

#[test]
fn conflicts_stop_after_the_first_scheduled_empty_batch_not_before_it() {
    check(
        vec![Clause::Value(1), marked(), Clause::Value(2)],
        "1&(*1|2)&2",
        vec![batch(&[], [true, true], &[])],
        &[],
        "F:conflict",
    );
    check(
        vec![Clause::Value(1), Clause::Value(2)],
        "1&2",
        vec![],
        &[],
        "F:conflict",
    );
    check(
        vec![
            or(&[(false, 1), (false, 2)]),
            or(&[(false, 3), (false, 4)]),
            or(&[(false, 5), (false, 6)]),
        ],
        "(1|2)&(3|4)&(5|6)",
        vec![
            batch(
                &[(1, M, M, M), (2, M, M, M)],
                [true, true],
                &[(1, M, M), (2, M, M)],
            ),
            batch(&[], [true, true], &[]),
        ],
        &[],
        "F:conflict",
    );
}

#[test]
fn bounded_group_permutations_compare_native_construction_without_oracle_inputs() {
    // Additional falsification matrix, separate from the full handwritten cases.
    // Only authored IR reaches execute; observed JSON is comparison-only.
    let groups = [
        marked(),
        or(&[(true, 2), (false, 1)]),
        or(&[(false, 1), (false, 2)]),
    ];
    let spellings = ["*1|2", "*2|1", "1|2"];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let ir = order.iter().map(|&i| groups[i].clone()).collect::<Vec<_>>();
        let report = execute(&ir, 4096).expect("native permutation before capture");
        let source = order
            .iter()
            .map(|&i| format!("({})", spellings[i]))
            .collect::<Vec<_>>()
            .join("&");
        let files = Files::new(&source);
        let (v, _) = files.history();
        files.semantics(&v, "F:incomplete");
        let batches = v["batches"].as_array().expect("batches");
        assert_eq!(batches.len(), report.batches.len());
        for (b, native) in batches.iter().zip(&report.batches) {
            assert_eq!(
                b["trials"],
                Value::Array(native.trials.iter().map(trial_json).collect()),
                "{source}"
            );
            assert_eq!(
                b["results"],
                Value::Array(native.states.iter().map(state_json).collect()),
                "{source}"
            );
            assert_eq!(b["leftDropsDefault"], native.left_drops);
            assert_eq!(b["rightDropsDefault"], native.right_drops);
        }
    }
}

#[test]
fn i64_limits_history_wire_and_legacy_results_remain_separate() {
    check(
        vec![or(&[(true, i64::MIN), (false, i64::MAX)])],
        "*-9223372036854775808|9223372036854775807",
        vec![batch(
            &[(i64::MIN, M, M, D), (i64::MAX, M, M, N)],
            [true, false],
            &[(i64::MIN, D, D), (i64::MAX, N, N)],
        )],
        &[(i64::MIN, D, D), (i64::MAX, N, N)],
        "T:-9223372036854775808",
    );
    let files = Files::new("*1|2");
    let (_, bytes) = files.history();
    let expected = concat!(
        r#"{"formatVersion":3,"mode":"trace-history","defaultSelected":true,"status":"concrete","value":1,"batches":[{"id":0,"depth":0,"leftDropsDefault":true,"rightDropsDefault":false,"trials":[{"mode":"maybe","originalMode":"default","leftMode":"maybe","leftOriginalMode":"maybe","value":1,"nested":[]},{"mode":"maybe","originalMode":"not","leftMode":"maybe","leftOriginalMode":"maybe","value":2,"nested":[]}],"results":[{"mode":"default","originalMode":"default","value":1},{"mode":"not","originalMode":"not","value":2}]}]}"#,
        "\n"
    );
    assert_eq!(bytes, expected.as_bytes());
    fs::rename(files.0.join("probe.cue"), files.0.join("renamed.cue")).expect("rename");
    let renamed = files.run(
        "CUESTACEAN_DEFAULT_TRACE",
        &["trace-history", "renamed.cue", "probe"],
    );
    assert!(renamed.status.success() && renamed.stderr.is_empty());
    assert_eq!(renamed.stdout, bytes);
    for (mode, version) in [("trace", 1), ("trace-origins", 2)] {
        let old = files.run("CUESTACEAN_DEFAULT_TRACE", &[mode, "renamed.cue", "probe"]);
        assert!(old.status.success() && old.stderr.is_empty());
        let v: Value = serde_json::from_slice(&old.stdout).expect("legacy");
        assert_eq!(v["formatVersion"], version);
        for b in v["batches"].as_array().expect("batches") {
            for r in b["results"].as_array().expect("results") {
                assert!(r.get("originalMode").is_none());
            }
        }
    }
    // Existing unchanged targets retain ALL v1/v2 full-byte answers and profile caps.
}

#[test]
fn structural_raw_products_and_shared_work_caps_recover_without_reset() {
    assert!(matches!(execute(&[], 4096), Err(Error::Unsupported(_))));
    for n in [0, 1] {
        let bad = [or(&vec![(false, 1); n])];
        assert!(matches!(execute(&bad, 4096), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 0), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 4097), Err(Error::Limit(_))));
    }
    assert!(execute(&vec![Clause::Value(1); 16], 4096).is_ok());
    assert!(matches!(
        execute(&vec![Clause::Value(1); 17], 4096),
        Err(Error::Limit(_))
    ));
    let max = vec![(false, 1); 16];
    let mut occurrences = vec![or(&max); 7];
    occurrences.push(or(&[(false, 1); 8]));
    assert!(execute(&occurrences, 4096).is_ok(), "128 IR occurrences");
    occurrences[7] = or(&[(false, 1); 9]);
    assert!(matches!(execute(&occurrences, 4096), Err(Error::Limit(_))));
    assert!(matches!(
        execute(&[or(&[(false, 1); 17])], 4096),
        Err(Error::Limit(_))
    ));
    let left = (0..8).map(|n| (false, n)).collect::<Vec<_>>();
    let result = execute(&[or(&left), or(&left)], 4096).expect("64 raw pairs");
    assert_eq!(result.states.len(), 8);
    let five = (0..5).map(|n| (false, n)).collect::<Vec<_>>();
    let thirteen = (0..13).map(|n| (false, n)).collect::<Vec<_>>();
    assert!(
        matches!(
            execute(&[or(&five), or(&thirteen)], 4096),
            Err(Error::Limit(_))
        ),
        "65 RAW pairs reject BEFORE filtering to five"
    );
    let one = [marked()];
    let two = [marked(), marked()];
    let minimum = |ir: &[Clause]| {
        (1..=4096)
            .find(|&n| execute(ir, n).is_ok())
            .expect("bounded allowance")
    };
    let baseline = minimum(&one);
    let work = minimum(&two);
    assert!(work > baseline);
    assert!(matches!(execute(&two, baseline), Err(Error::Limit(_))));
    assert!(matches!(execute(&two, work - 1), Err(Error::Limit(_))));
    assert_eq!(execute(&two, work).expect("exact recovery").work, work);
    assert!(matches!(execute(&one, 0), Err(Error::Limit(_))));
    assert_eq!(
        execute(&one, 4096).expect("recovery").states,
        states(&[(1, D, D), (2, N, N)])
    );
}

#[test]
fn imported_batch_primitive_remains_independent_and_nested_is_not_flat_source() {
    // This meaningful old-API control also exercises the imported private module
    // without widening visibility or suppressing dead-code lints.
    let input = [Trial {
        mode: M,
        left_mode: M,
        left_origin: M,
        incoming: D,
        body: Body::Nested(vec![Leaf { value: 3, mode: D }, Leaf { value: 1, mode: N }]),
    }];
    let r = batch_defaults::execute(&input, 4096).expect("old nested primitive");
    assert!(r.has_non_maybe && r.left_drops && !r.right_drops && r.work > 0);
    assert_eq!(
        r.results,
        vec![Leaf { value: 3, mode: D }, Leaf { value: 1, mode: N }]
    );
    assert!(matches!(
        batch_defaults::execute(
            &[Trial {
                body: Body::Nested(vec![]),
                ..input[0].clone()
            }],
            4096
        ),
        Err(Error::Unsupported(_))
    ));
    // The flat IR has no way to express the preserved nested #176/#186/#188
    // failures. Their unchanged tests/gates remain, not silently flattened cases.
}
