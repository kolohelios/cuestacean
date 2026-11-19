//! One ordered scalar phase, not a default solver or a survivor certificate.
#[path = "../../spikes/batch_defaults.rs"]
mod batch_defaults;
use Mode::{Default as D, Maybe as M, Not as N};
use batch_defaults::{Body, Error, Leaf, Mode, Report, Trial, execute};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn leaf(value: i64, mode: Mode) -> Leaf {
    Leaf { value, mode }
}
fn scalar(mode: Mode, left_mode: Mode, left_origin: Mode, incoming: Mode, value: i64) -> Trial {
    Trial {
        mode,
        left_mode,
        left_origin,
        incoming,
        body: Body::Scalar(value),
    }
}
fn nested(incoming: Mode, children: Vec<Leaf>) -> Trial {
    Trial {
        mode: M,
        left_mode: M,
        left_origin: M,
        incoming,
        body: Body::Nested(children),
    }
}
fn checked(input: &[Trial], drops: [bool; 2], non_maybe: bool, expected: Vec<Leaf>) -> Report {
    let saved = input.to_vec();
    let r = execute(input, 4096).expect("bounded native batch");
    assert_eq!([r.left_drops, r.right_drops], drops);
    assert_eq!(r.has_non_maybe, non_maybe);
    assert_eq!(r.results, expected);
    assert!((1..=4096).contains(&r.work));
    assert_eq!(execute(input, 4096).expect("repeat fresh state"), r);
    assert_eq!(input, saved, "caller facts remain immutable");
    r
}
fn word(m: &Mode) -> &'static str {
    match m {
        M => "maybe",
        D => "default",
        N => "not",
    }
}
fn leaf_json(l: &Leaf) -> Value {
    json!({"mode":word(&l.mode),"value":l.value})
}
fn trial_json(t: &Trial) -> Value {
    let (value, children) = match &t.body {
        Body::Scalar(v) => (Some(*v), vec![]),
        Body::Nested(xs) => (None, xs.iter().map(leaf_json).collect()),
    };
    json!({"mode":word(&t.mode),"originalMode":word(&t.incoming),
        "leftMode":word(&t.left_mode),"leftOriginalMode":word(&t.left_origin),
        "value":value,"nested":children})
}
struct Files(PathBuf);
impl Files {
    fn new(expr: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "cuestacean-batch-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).expect("unique owned scratch");
        let f = Self(p);
        fs::write(f.0.join("probe.cue"), format!("probe: {expr}\n")).expect("authored CUE");
        f
    }
    fn run(&self, provider: &str, args: &[&str]) -> Output {
        let p = PathBuf::from(std::env::var_os(provider).expect("mandatory provider"));
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
    fn origins(&self) -> (Value, Vec<u8>) {
        let out = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-origins", "probe.cue", "probe"],
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stderr.is_empty());
        assert!(out.stdout.len() <= 65536 && out.stdout.ends_with(b"\n"));
        let v: Value = serde_json::from_slice(&out.stdout).expect("origin JSON");
        assert_eq!(v["formatVersion"], 2);
        assert_eq!(v["mode"], "trace-origins");
        let repeated = self.run(
            "CUESTACEAN_DEFAULT_TRACE",
            &["trace-origins", "probe.cue", "probe"],
        );
        assert!(repeated.status.success() && repeated.stderr.is_empty());
        assert_eq!(repeated.stdout, out.stdout);
        let mut records = 0;
        let batches = v["batches"].as_array().expect("batches");
        assert!(batches.len() <= 128);
        for (id, b) in batches.iter().enumerate() {
            assert_eq!(b["id"], id);
            let ts = b["trials"].as_array().expect("trials");
            let rs = b["results"].as_array().expect("results");
            assert!(ts.len() <= 64 && rs.len() <= 64);
            records += 3 * ts.len() + rs.len();
            for t in ts {
                for key in ["mode", "originalMode", "leftMode", "leftOriginalMode"] {
                    assert!(matches!(t[key].as_str(), Some("maybe" | "default" | "not")));
                }
                let children = t["nested"].as_array().expect("immediate children");
                assert!(children.len() <= 64);
                records += children.len();
            }
        }
        assert!(records <= 4096);
        (v, out.stdout)
    }
    fn semantics(&self, trace: &Value, selected: bool, status: &str, value: Option<i64>) {
        assert_eq!(trace["defaultSelected"], selected);
        assert_eq!(trace["status"], status);
        assert_eq!(trace["value"], json!(value));
        let out = self.run(
            "CUESTACEAN_DEFAULT_OBSERVER",
            &["inspect", "probe.cue", "probe"],
        );
        assert!(out.status.success() && out.stderr.is_empty());
        let expected = json!({"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,
            "defaultSelected":selected,"status":status,"value":value});
        assert_eq!(
            serde_json::from_slice::<Value>(&out.stdout).expect("uninstrumented public API"),
            expected
        );
        // This is a mandatory separate inline-source oracle, never API Unify.
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned CLI");
        let cli = Command::new(cue)
            .args(["export", "probe.cue", "-e", "probe"])
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", &self.0)
            .output()
            .expect("CLI");
        if let Some(n) = value {
            assert!(cli.status.success());
            assert_eq!(cli.stdout.trim_ascii(), n.to_string().as_bytes());
        } else {
            assert!(!cli.status.success() && cli.stdout.is_empty());
            assert!(String::from_utf8_lossy(&cli.stderr).contains("incomplete"));
        }
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned files");
    }
}

#[test]
fn flat_mode_truth_table_and_left_origin_are_independent_inputs() {
    // Hand table of the LEFT drop predicate, rows current p / columns origin p.
    let left_drops = [
        [true, false, true],
        [false, false, false],
        [true, false, true],
    ];
    let modes = [M, D, N];
    for (i, left) in modes.iter().enumerate() {
        for (j, origin) in modes.iter().enumerate() {
            for current in &modes {
                for incoming in &modes {
                    let ld = left_drops[i][j];
                    let rd = incoming != &D;
                    // Source-derived 2x2 drop table, not the kernel's join routine.
                    let expected = match (ld, rd) {
                        (true, true) => M,
                        (true, false) => D,
                        (false, true) => *current,
                        (false, false) if current == &N => N,
                        (false, false) => D,
                    };
                    checked(
                        &[scalar(*current, *left, *origin, *incoming, 7)],
                        [ld, rd],
                        false,
                        vec![leaf(7, expected)],
                    );
                }
            }
        }
    }
    checked(&[], [true, true], false, vec![]); // local emptiness, not Conflict.
}

#[test]
fn nested_combination_and_ordered_coalescing_are_not_set_union() {
    checked(
        &[
            nested(N, vec![leaf(1, D), leaf(2, N)]),
            scalar(M, M, M, D, 3),
        ],
        [true, false],
        true,
        vec![leaf(1, N), leaf(2, N), leaf(3, D)],
    );
    checked(
        &[
            nested(D, vec![leaf(3, D), leaf(1, N)]),
            scalar(M, M, M, N, 2),
        ],
        [true, false],
        true,
        vec![leaf(3, D), leaf(1, N), leaf(2, N)],
    );
    // Keep first unless the NEW duplicate is Default; never max-mode dedup.
    for (a, b, expected) in [(M, N, M), (N, M, N), (N, D, D), (D, N, D), (M, D, D)] {
        checked(
            &[scalar(a, D, M, M, 7), scalar(b, D, M, M, 7)],
            [false, true],
            false,
            vec![leaf(7, expected)],
        );
    }
    checked(
        &[scalar(M, D, M, M, 7), scalar(N, D, M, M, 8)],
        [false, true],
        false,
        vec![leaf(7, M), leaf(8, N)],
    );
    // The only nested Not is coalesced into an earlier Maybe value. The flag
    // must be set BEFORE coalescing, or both final results wrongly stay Maybe.
    let mut hidden_not = nested(M, vec![leaf(7, N), leaf(9, M)]);
    hidden_not.left_mode = D;
    checked(
        &[scalar(M, D, M, M, 7), hidden_not],
        [false, true],
        true,
        vec![leaf(7, N), leaf(9, N)],
    );
    let mut child = nested(M, vec![leaf(8, N), leaf(9, M)]);
    child.left_mode = D;
    checked(
        &[scalar(M, D, M, M, 7), child],
        [false, true],
        true,
        vec![leaf(7, N), leaf(8, N), leaf(9, N)],
    );
    checked(
        &[scalar(D, D, M, M, i64::MIN), scalar(N, D, M, M, i64::MAX)],
        [false, true],
        false,
        vec![leaf(i64::MIN, D), leaf(i64::MAX, N)],
    );
}

#[test]
fn handwritten_source_batches_match_native_and_origin_aware_cue_separately() {
    // NONE of these native inputs are parsed from a trace. Every tuple is
    // independently authored from the pinned phase and the separate raw source.
    let cases = vec![
        (
            "*1|2",
            false,
            true,
            "concrete",
            Some(1),
            vec![scalar(M, M, M, D, 1), scalar(M, M, M, N, 2)],
            [true, false],
            false,
            vec![leaf(1, D), leaf(2, N)],
        ),
        (
            "(*1|2)&(*2|1)",
            true,
            false,
            "incomplete",
            None,
            vec![scalar(D, D, D, N, 1), scalar(N, N, N, D, 2)],
            [false, false],
            false,
            vec![leaf(1, N), leaf(2, N)],
        ),
        (
            "((*1|2)|*3)&(*2|1)",
            true,
            false,
            "incomplete",
            None,
            vec![scalar(N, N, D, N, 1), scalar(N, N, N, D, 2)],
            [false, false],
            false,
            vec![leaf(1, N), leaf(2, N)],
        ),
        (
            "((*1|2)|*3)&(1|2)",
            true,
            false,
            "incomplete",
            None,
            vec![scalar(N, N, D, M, 1), scalar(N, N, N, M, 2)],
            [false, true],
            false,
            vec![leaf(1, N), leaf(2, N)],
        ),
        (
            "((*1|2)|*3)&(*2|1)",
            false,
            false,
            "incomplete",
            None,
            vec![
                nested(N, vec![leaf(1, D), leaf(2, N)]),
                scalar(M, M, M, D, 3),
            ],
            [true, false],
            true,
            vec![leaf(1, N), leaf(2, N), leaf(3, D)],
        ),
        (
            "(*(*3|1)|2)&(1|2)",
            false,
            false,
            "incomplete",
            None,
            vec![
                nested(D, vec![leaf(3, D), leaf(1, N)]),
                scalar(M, M, M, N, 2),
            ],
            [true, false],
            true,
            vec![leaf(3, D), leaf(1, N), leaf(2, N)],
        ),
        (
            "(*(*3|1)|2)&(1|2)",
            true,
            false,
            "incomplete",
            None,
            vec![scalar(N, N, N, M, 1), scalar(N, N, N, M, 2)],
            [true, true],
            false,
            vec![leaf(1, M), leaf(2, M)],
        ),
        (
            "(((*1|2)&(*2|1))|1)&(*1|2)",
            true,
            false,
            "incomplete",
            None,
            vec![scalar(N, N, N, D, 1), scalar(N, N, D, N, 2)],
            [false, false],
            false,
            vec![leaf(1, N), leaf(2, N)],
        ),
        (
            "(*((*1|2)|3)|(*4|5))&(1|4)",
            true,
            true,
            "concrete",
            Some(1),
            vec![scalar(D, D, D, M, 1), scalar(N, N, D, M, 4)],
            [false, true],
            false,
            vec![leaf(1, D), leaf(4, N)],
        ),
        // Real doDisjunct result/predecessor mismatch: r.mode for 1 is Default,
        // while predecessor p.mode and p.origin for that trial are both Not.
        (
            "((*3|1)|3)&((*1|2)|(*3|2))",
            true,
            true,
            "incomplete",
            None,
            vec![scalar(D, D, D, M, 3), scalar(D, N, N, M, 1)],
            [false, true],
            false,
            vec![leaf(3, D), leaf(1, D)],
        ),
    ];
    for (source, last, selected, status, value, input, drops, non_maybe, expected) in cases {
        let native = checked(&input, drops, non_maybe, expected);
        let files = Files::new(source);
        let (trace, _) = files.origins();
        files.semantics(&trace, selected, status, value);
        let batches = trace["batches"].as_array().expect("batches");
        let b = if last {
            batches
                .iter()
                .rev()
                .find(|b| b["depth"] == 0)
                .expect("last root")
        } else {
            &batches[0]
        };
        assert_eq!(
            b["trials"],
            Value::Array(input.iter().map(trial_json).collect()),
            "hand INPUT vs CUE: {source}"
        );
        assert_eq!(b["leftDropsDefault"], native.left_drops);
        assert_eq!(b["rightDropsDefault"], native.right_drops);
        assert_eq!(
            b["results"],
            Value::Array(native.results.iter().map(leaf_json).collect()),
            "hand-validated NATIVE report vs CUE: {source}"
        );
    }
}

#[test]
fn preserved_source_pairs_keep_both_orders_separate_from_native_reports() {
    // Handwritten PUBLIC observations. They are not computed by the native
    // primitive, and no trace is converted into native input or preferred output.
    let rows = [
        (
            "*1|2",
            "1|2|*3",
            [(true, "concrete", Some(1)), (true, "concrete", Some(1))],
        ),
        (
            "*2|3",
            "(*1|2)|3",
            [(true, "concrete", Some(2)), (true, "concrete", Some(2))],
        ),
        (
            "(*1|2)|*3",
            "*2|1",
            [(false, "incomplete", None), (true, "incomplete", None)],
        ),
        (
            "(*3|1)|3",
            "(*1|2)|(*3|2)",
            [(true, "incomplete", None), (true, "incomplete", None)],
        ),
        (
            "((*1|2)&(*2|1))|1",
            "*1|2",
            [(false, "incomplete", None), (true, "incomplete", None)],
        ),
        (
            "*((*1|2)|3)|(*4|5)",
            "1|4",
            [(true, "concrete", Some(1)), (true, "concrete", Some(1))],
        ),
        (
            "(*1|2)|*3",
            "1|2",
            [(false, "incomplete", None), (false, "incomplete", None)],
        ),
        (
            "*1|2",
            "(*1|2)|*3",
            [(true, "concrete", Some(1)), (false, "incomplete", None)],
        ),
        (
            "*(*3|1)|2",
            "1|2",
            [(false, "incomplete", None), (true, "concrete", Some(1))],
        ),
        (
            "*((*1|2)&(1|*2))|3",
            "*3|1|2",
            [(false, "incomplete", None), (true, "concrete", Some(2))],
        ),
    ];
    for (left, right, expected) in rows {
        for ((a, b), (selected, status, value)) in
            [(left, right), (right, left)].into_iter().zip(expected)
        {
            let files = Files::new(&format!("({a})&({b})"));
            let (trace, _) = files.origins();
            files.semantics(&trace, selected, status, value);
        }
    }
}

#[test]
fn v2_ordered_wire_and_v1_protocol_remain_distinct() {
    let files = Files::new("*1|2");
    let (_, bytes) = files.origins();
    let expected = concat!(
        r#"{"formatVersion":2,"mode":"trace-origins","defaultSelected":true,"status":"concrete","value":1,"batches":[{"id":0,"depth":0,"leftDropsDefault":true,"rightDropsDefault":false,"trials":[{"mode":"maybe","originalMode":"default","leftMode":"maybe","leftOriginalMode":"maybe","value":1,"nested":[]},{"mode":"maybe","originalMode":"not","leftMode":"maybe","leftOriginalMode":"maybe","value":2,"nested":[]}],"results":[{"mode":"default","value":1},{"mode":"not","value":2}]}]}"#,
        "\n"
    );
    assert_eq!(bytes, expected.as_bytes());
    fs::rename(files.0.join("probe.cue"), files.0.join("renamed.cue")).expect("rename");
    let renamed = files.run(
        "CUESTACEAN_DEFAULT_TRACE",
        &["trace-origins", "renamed.cue", "probe"],
    );
    assert!(renamed.status.success());
    assert_eq!(renamed.stdout, bytes);
    let v1 = files.run(
        "CUESTACEAN_DEFAULT_TRACE",
        &["trace", "renamed.cue", "probe"],
    );
    assert!(v1.status.success() && v1.stderr.is_empty());
    let old: Value = serde_json::from_slice(&v1.stdout).expect("legacy v1");
    assert_eq!(old["formatVersion"], 1);
    for b in old["batches"].as_array().expect("batches") {
        for t in b["trials"].as_array().expect("trials") {
            assert!(t.get("leftMode").is_none() && t.get("leftOriginalMode").is_none());
        }
    }
    // The unchanged default_trace target retains ALL complete legacy byte answers.
}

#[test]
fn native_structure_raw_products_and_shared_work_fail_closed_and_recover() {
    for count in [0, 1] {
        let bad = [nested(M, vec![leaf(1, M); count])];
        assert!(matches!(execute(&bad, 4096), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 0), Err(Error::Unsupported(_))));
        assert!(matches!(execute(&bad, 4097), Err(Error::Limit(_))));
    }
    let duplicates = vec![scalar(M, M, M, M, 1); 64];
    checked(&duplicates, [true, true], false, vec![leaf(1, M)]);
    assert!(matches!(
        execute(&vec![scalar(M, M, M, M, 1); 65], 4096),
        Err(Error::Limit(_))
    ));
    let children: Vec<_> = (0..64).map(|v| leaf(v, M)).collect();
    checked(
        &[nested(M, children.clone())],
        [true, true],
        false,
        children,
    );
    assert!(matches!(
        execute(&[nested(M, vec![leaf(1, M); 65])], 4096),
        Err(Error::Limit(_))
    ));
    let raw = [
        nested(M, vec![leaf(1, M); 32]),
        nested(M, vec![leaf(1, M); 33]),
    ];
    assert!(
        matches!(execute(&raw, 4096), Err(Error::Limit(_))),
        "raw expansion counts BEFORE duplicate collapse"
    );
    // The 128-record cap may be dominated by raw64; never increase a guard merely
    // to claim independent reachability. A too-large input must still fail safely.
    assert!(matches!(
        execute(&vec![nested(M, vec![leaf(1, M); 64]); 64], 4096),
        Err(Error::Limit(_))
    ));
    assert!(matches!(execute(&[], 0), Err(Error::Limit(_))));
    let one = [scalar(M, D, M, M, 1)];
    let more = [scalar(M, D, M, M, 1), scalar(N, D, M, M, 2)];
    let minimum = |xs: &[Trial]| {
        (1..=4096)
            .find(|&n| execute(xs, n).is_ok())
            .expect("bounded exact allowance")
    };
    let base = minimum(&one);
    let work = minimum(&more);
    assert!(work > base);
    assert!(matches!(execute(&more, base), Err(Error::Limit(_))));
    assert!(matches!(execute(&more, work - 1), Err(Error::Limit(_))));
    assert_eq!(execute(&more, work).expect("exact recovery").work, work);
    checked(&one, [false, true], false, vec![leaf(1, M)]);
}
