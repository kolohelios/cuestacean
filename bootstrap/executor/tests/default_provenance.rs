// Research hypothesis only: independent native expectations are not CUE conformance.
#[path = "../../spikes/default_provenance.rs"]
mod default_provenance;

use default_provenance::{Domain, Error, Expr, Graph, Outcome, Preference, Report};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn value(n: i64) -> Expr {
    Expr::Value(n)
}
fn and(a: Expr, b: Expr) -> Expr {
    Expr::And(Box::new(a), Box::new(b))
}
fn or(terms: Vec<(bool, Expr)>) -> Expr {
    Expr::Or(terms)
}
fn choice(a: i64, b: i64) -> Expr {
    or(vec![(true, value(a)), (false, value(b))])
}
fn union(a: Expr, b: Expr) -> Expr {
    or(vec![(false, a), (false, b)])
}
fn domains(values: &[i64]) -> Vec<Domain> {
    values.iter().map(|n| Domain::Value(*n)).collect()
}
fn prefs(values: &[i64]) -> Preference {
    Preference::Values(domains(values))
}
fn check(expr: Expr, constraints: Vec<Domain>, preferred: Preference, outcome: Outcome) -> Report {
    let graph = Graph::new(expr).expect("valid bounded manual IR");
    let report = graph.solve(4096).expect("bounded hypothesis solve");
    assert_eq!(report.constraints, constraints);
    assert_eq!(report.preferred, preferred);
    assert_eq!(report.outcome, outcome);
    assert!(report.work > 0 && report.work <= 4096);
    assert_eq!(graph.solve(4096).expect("fresh repeated solve"), report);
    assert!(matches!(graph.solve(0), Err(Error::Limit(_))));
    assert_eq!(
        graph.solve(4096).expect("failure cannot poison solve"),
        report
    );
    report
}

// Every invocation creates explicit schema/program files and starts a fresh process.
// No exported/golden data or production parser is involved. The pinned Nix CUE
// source is v0.16.1, commit 6d609d768f1686f9a3a2a20197cacdbb70e5c79d;
// `(devel)` alone does not attest that source identity (see closure documentation).
struct ProbeFiles(PathBuf);
impl ProbeFiles {
    fn new(expression: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-provenance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive probe directory");
        fs::write(path.join("schema.cue"), "package probe\n").expect("schema");
        fs::write(
            path.join("program.cue"),
            format!("package probe\nvalue: {expression}\n"),
        )
        .expect("program");
        Self(path)
    }
    fn run(&self, operation: &str) -> Output {
        let mut command = Command::new("cue");
        command
            .arg(operation)
            .arg(self.0.join("schema.cue"))
            .arg(self.0.join("program.cue"))
            .args(["-e", "value"]);
        if operation == "export" {
            command.args(["--out", "json"]);
        }
        command.output().expect("mandatory pinned CUE on PATH")
    }
}
impl Drop for ProbeFiles {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned probe directory");
    }
}
#[derive(Clone, Copy)]
enum Oracle {
    Concrete(i64),
    Incomplete,
    Conflict,
}
fn oracle(expression: &str, expected: Oracle) {
    let probe = ProbeFiles::new(expression);
    let output = probe.run("export");
    let stderr = String::from_utf8_lossy(&output.stderr);
    match expected {
        Oracle::Concrete(n) => {
            assert!(output.status.success(), "{expression}: {stderr}");
            let actual: i64 = String::from_utf8(output.stdout)
                .expect("integer UTF-8")
                .trim()
                .parse()
                .expect("exact i64 scalar, not floating-point normalization");
            assert_eq!(actual, n, "independent CUE expectation: {expression}");
        }
        Oracle::Incomplete => {
            assert!(!output.status.success(), "{expression}");
            assert!(output.stdout.is_empty(), "no failed-export partial JSON");
            assert!(
                stderr.contains("incomplete value"),
                "{expression}: {stderr}"
            );
            let evaluated = probe.run("eval");
            assert!(
                evaluated.status.success(),
                "incompleteness is not bottom: {expression}: {}",
                String::from_utf8_lossy(&evaluated.stderr)
            );
        }
        Oracle::Conflict => {
            assert!(!output.status.success(), "{expression}");
            assert!(output.stdout.is_empty());
            assert!(stderr.contains("conflicting values"), "{stderr}");
        }
    }
}
fn both_orders(left: &str, right: &str, forward: Oracle, reverse: Oracle) {
    oracle(&format!("{left}&{right}"), forward);
    oracle(&format!("{right}&{left}"), reverse);
}

#[test]
fn conservative_demand_is_additive_to_the_old_hypothesis() {
    let graph = Graph::new(value(1)).expect("ready scalar IR");
    let report = graph.demand(Domain::Top, 4096).expect("bounded demand");
    assert_eq!(report.constraints, domains(&[1]));
    assert_eq!(
        report.readiness,
        default_provenance::Readiness::ConstraintOnly
    );
    assert_eq!(report.demand, default_provenance::Demand::Concrete(1));
    assert_eq!(report.preferred, None);
    assert!(report.groups.is_empty());
    assert_eq!(
        graph.solve(4096).expect("old API unchanged").outcome,
        Outcome::Concrete(1)
    );
}

#[test]
fn policy_entry_point_is_additive_to_native_reference() {
    let policy = default_provenance::policy::Policy::from_json(
        r#"{"formatVersion":1,"body":{"op":"return","action":"defer"}}"#,
    )
    .expect("bounded policy API fixture");
    let graph = Graph::new(value(1)).expect("ready scalar IR");
    let report = graph
        .demand_with_policy(Domain::Top, 4096, &policy)
        .expect("policy demand");
    assert_eq!(
        report.readiness,
        default_provenance::Readiness::PolicyDeferred
    );
    assert_eq!(report.demand, default_provenance::Demand::Suspended);
    assert_eq!(report.preferred, None);
    assert_eq!(
        graph
            .demand(Domain::Top, 4096)
            .expect("native reference")
            .demand,
        default_provenance::Demand::Concrete(1)
    );
}

#[test]
fn pinned_cue_is_required_not_optional() {
    let output = Command::new("cue")
        .arg("version")
        .output()
        .expect("mandatory pinned CUE");
    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).expect("version UTF-8");
    assert!(
        version
            .lines()
            .any(|line| line.trim() == "CUE language version v0.16.1"),
        "unexpected pinned toolchain: {version}"
    );
}

#[test]
fn both_order_support_elimination_gaps_and_present_bottom_control() {
    let cases = [
        (
            choice(1, 2),
            or(vec![(false, value(1)), (false, value(2)), (true, value(3))]),
            vec![1, 2],
            prefs(&[1]),
            Outcome::Concrete(1),
            "(*1|2)",
            "(1|2|*3)",
            Oracle::Concrete(1),
        ),
        (
            choice(2, 3),
            union(choice(1, 2), value(3)),
            vec![2, 3],
            prefs(&[2]),
            Outcome::Concrete(2),
            "(*2|3)",
            "((*1|2)|3)",
            Oracle::Concrete(2),
        ),
        (
            choice(1, 2),
            or(vec![(false, value(1)), (true, value(2))]),
            vec![1, 2],
            Preference::Bottom,
            Outcome::Incomplete,
            "(*1|2)",
            "(1|*2)",
            Oracle::Incomplete,
        ),
    ];
    for (a, b, ordinary, preferred, outcome, left, right, upstream) in cases {
        let report = check(
            and(a.clone(), b.clone()),
            domains(&ordinary),
            preferred,
            outcome,
        );
        check(
            and(b, a),
            domains(&ordinary),
            report.preferred,
            report.outcome,
        );
        both_orders(left, right, upstream, upstream);
    }
}

#[test]
fn known_oracle_asymmetries_have_separate_native_expectations() {
    let cases = [
        (
            choice(1, 2),
            or(vec![(false, choice(1, 2)), (true, value(3))]),
            domains(&[1, 2]),
            prefs(&[1]),
            Outcome::Concrete(1),
            "(*1|2)",
            "((*1|2)|*3)",
            Oracle::Concrete(1),
            Oracle::Incomplete,
        ),
        (
            or(vec![(true, Expr::Int), (false, value(1))]),
            union(choice(1, 2), value(3)),
            domains(&[1, 2, 3]),
            prefs(&[1]),
            Outcome::Concrete(1),
            "(*int|1)",
            "((*1|2)|3)",
            Oracle::Incomplete,
            Oracle::Concrete(1),
        ),
        (
            or(vec![
                (
                    true,
                    and(choice(1, 2), or(vec![(false, value(1)), (true, value(2))])),
                ),
                (false, value(3)),
            ]),
            or(vec![(true, value(3)), (false, value(1)), (false, value(2))]),
            domains(&[1, 2, 3]),
            Preference::Bottom,
            Outcome::Incomplete,
            "(*((*1|2)&(1|*2))|3)",
            "(*3|1|2)",
            Oracle::Incomplete,
            Oracle::Concrete(2),
        ),
    ];
    for (a, b, ordinary, preferred, outcome, left, right, forward, reverse) in cases {
        let report = check(
            and(a.clone(), b.clone()),
            ordinary.clone(),
            preferred,
            outcome,
        );
        check(and(b, a), ordinary, report.preferred, report.outcome);
        both_orders(left, right, forward, reverse);
    }
}

#[test]
fn both_order_counterexample_is_negative_production_readiness_evidence() {
    // NOT an oracle order bug: the agreed hypothesis differs in BOTH orders.
    // M3 strips inner *1; dead marked 3 becomes absent, leaving the other *2.
    let a = or(vec![(false, choice(1, 2)), (true, value(3))]);
    let b = choice(2, 1);
    for expr in [and(a.clone(), b.clone()), and(b, a)] {
        check(expr, domains(&[1, 2]), prefs(&[2]), Outcome::Concrete(2));
    }
    both_orders(
        "((*1|2)|*3)",
        "(*2|1)",
        Oracle::Incomplete,
        Oracle::Incomplete,
    );
}

#[test]
fn m1_m2_m3_late_marks_and_equal_or_distinct_preferences() {
    let cases = [
        (
            choice(1, 2),
            domains(&[1, 2]),
            prefs(&[1]),
            Outcome::Concrete(1),
        ),
        (
            or(vec![(false, value(1)), (true, value(2))]),
            domains(&[1, 2]),
            prefs(&[2]),
            Outcome::Concrete(2),
        ),
        (
            or(vec![(true, value(1)), (true, value(1))]),
            domains(&[1]),
            prefs(&[1]),
            Outcome::Concrete(1),
        ),
        (
            or(vec![(true, value(1)), (true, value(2))]),
            domains(&[1, 2]),
            prefs(&[1, 2]),
            Outcome::Incomplete,
        ),
        (
            or(vec![(true, choice(1, 2)), (false, value(3))]),
            domains(&[1, 2, 3]),
            prefs(&[1]),
            Outcome::Concrete(1),
        ),
        (
            or(vec![(false, choice(1, 2)), (true, value(3))]),
            domains(&[1, 2, 3]),
            prefs(&[3]),
            Outcome::Concrete(3),
        ),
        (
            union(choice(1, 2), value(3)),
            domains(&[1, 2, 3]),
            prefs(&[1]),
            Outcome::Concrete(1),
        ),
        (
            union(choice(1, 2), choice(3, 4)),
            domains(&[1, 2, 3, 4]),
            prefs(&[1, 3]),
            Outcome::Incomplete,
        ),
        (
            or(vec![
                (
                    true,
                    and(choice(1, 2), or(vec![(false, value(1)), (true, value(2))])),
                ),
                (false, value(3)),
            ]),
            domains(&[1, 2, 3]),
            Preference::Bottom,
            Outcome::Incomplete,
        ),
    ];
    for (expr, ordinary, preferred, outcome) in cases {
        check(expr, ordinary, preferred, outcome);
    }
    for (source, expected) in [
        ("*1|2", Oracle::Concrete(1)),
        ("1|*2", Oracle::Concrete(2)),
        ("*1|*1", Oracle::Concrete(1)),
        ("*1|*2", Oracle::Incomplete),
        ("*(*1|2)|3", Oracle::Concrete(1)),
        ("(*1|2)|*3", Oracle::Concrete(3)),
        ("(*1|2)|3", Oracle::Concrete(1)),
        ("(*1|2)|(*3|4)", Oracle::Incomplete),
        ("*((*1|2)&(1|*2))|3", Oracle::Incomplete),
    ] {
        oracle(source, expected);
    }
}

#[test]
fn unmarked_union_retains_present_bottom_without_promoting_absent_terms() {
    check(
        union(and(choice(1, 2), choice(2, 1)), value(3)),
        domains(&[1, 2, 3]),
        Preference::Bottom,
        Outcome::Incomplete,
    );
    check(
        union(and(choice(1, 2), choice(2, 1)), choice(3, 4)),
        domains(&[1, 2, 3, 4]),
        prefs(&[3]),
        Outcome::Concrete(3),
    );
    check(
        union(value(1), value(1)),
        domains(&[1]),
        Preference::Absent,
        Outcome::Concrete(1),
    );
}

#[test]
fn abstract_readiness_top_refinement_exact_reporting_and_conflict() {
    check(
        Expr::Top,
        vec![Domain::Top],
        Preference::Absent,
        Outcome::Incomplete,
    );
    check(
        Expr::Int,
        vec![Domain::Int],
        Preference::Absent,
        Outcome::Incomplete,
    );
    check(
        or(vec![(true, Expr::Int), (false, value(1))]),
        vec![Domain::Int, Domain::Value(1)],
        Preference::Values(vec![Domain::Int]),
        Outcome::Incomplete,
    );
    check(
        or(vec![(true, Expr::Top), (false, value(1))]),
        vec![Domain::Top, Domain::Value(1)],
        Preference::Values(vec![Domain::Top]),
        Outcome::Incomplete,
    );
    check(
        and(Expr::Top, choice(1, 2)),
        domains(&[1, 2]),
        prefs(&[1]),
        Outcome::Concrete(1),
    );
    check(
        and(choice(1, 2), value(2)),
        domains(&[2]),
        Preference::Absent,
        Outcome::Concrete(2),
    );
    check(
        and(Expr::Int, value(i64::MIN)),
        domains(&[i64::MIN]),
        Preference::Absent,
        Outcome::Concrete(i64::MIN),
    );
    check(
        and(value(1), value(2)),
        vec![],
        Preference::Absent,
        Outcome::Conflict,
    );
    check(
        or(vec![
            (false, value(2)),
            (false, Expr::Top),
            (false, value(-1)),
            (false, Expr::Int),
            (false, value(2)),
        ]),
        vec![
            Domain::Top,
            Domain::Int,
            Domain::Value(-1),
            Domain::Value(2),
        ],
        Preference::Absent,
        Outcome::Incomplete,
    );
    check(
        or(vec![(true, value(1)), (true, Expr::Int)]),
        vec![Domain::Int, Domain::Value(1)],
        Preference::Values(vec![Domain::Int, Domain::Value(1)]),
        Outcome::Incomplete,
    );
    oracle("*int|1", Oracle::Incomplete);
    check(
        and(or(vec![(true, Expr::Top), (false, value(1))]), value(2)),
        domains(&[2]),
        prefs(&[2]),
        Outcome::Concrete(2),
    );
    oracle("(*_ |1)&2", Oracle::Concrete(2));
    oracle("(*1|2)&2", Oracle::Concrete(2));
    oracle("1&2", Oracle::Conflict);
}

#[test]
fn sibling_visibility_and_syntactic_m2_do_not_resurrect_defaults() {
    check(
        union(and(choice(1, 2), value(2)), value(1)),
        domains(&[1, 2]),
        Preference::Absent,
        Outcome::Incomplete,
    );
    oracle("(((*1|2)&2)|1)", Oracle::Incomplete);
    // The marked outer term has a syntactic inner recipe, but it is contextually
    // absent. M2 must preserve bottom rather than switching to M1 and choosing 2.
    check(
        or(vec![(true, and(choice(1, 2), value(2))), (false, value(3))]),
        domains(&[2, 3]),
        Preference::Bottom,
        Outcome::Incomplete,
    );
    // Equal spelling in an unentered sibling cannot reactivate inner *1.
    check(
        union(and(choice(1, 2), value(2)), choice(1, 3)),
        domains(&[1, 2, 3]),
        prefs(&[1]),
        Outcome::Concrete(1),
    );
}

#[test]
fn bounded_permutation_matrix_keeps_group_and_occurrence_identity() {
    let terms = [(true, value(1)), (false, value(2)), (false, value(3))];
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for p in permutations {
        let expr = or(p.map(|i| terms[i].clone()).to_vec());
        let spellings = ["*1", "2", "3"];
        let source = format!(
            "({}|{}|{})",
            spellings[p[0]], spellings[p[1]], spellings[p[2]]
        );
        for reversed in [false, true] {
            let combined = if reversed {
                and(value(1), expr.clone())
            } else {
                and(expr.clone(), value(1))
            };
            check(combined, domains(&[1]), prefs(&[1]), Outcome::Concrete(1));
            oracle(
                &if reversed {
                    format!("1&{source}")
                } else {
                    format!("{source}&1")
                },
                Oracle::Concrete(1),
            );
        }
    }
}

fn balanced(mut nodes: Vec<Expr>) -> Expr {
    assert!(!nodes.is_empty());
    while nodes.len() > 1 {
        let mut next = Vec::new();
        let mut iter = nodes.into_iter();
        while let Some(a) = iter.next() {
            next.push(if let Some(b) = iter.next() {
                and(a, b)
            } else {
                a
            });
        }
        nodes = next;
    }
    nodes.pop().expect("one root")
}
fn alternatives(width: usize, n: i64) -> Expr {
    or((0..width).map(|_| (false, value(n))).collect())
}
fn limit(expr: Expr) {
    assert!(matches!(Graph::new(expr), Err(Error::Limit(_))));
}

#[test]
fn structural_limits_validate_dead_branches_and_root_inclusive_depth() {
    for width in [0, 1] {
        assert!(matches!(
            Graph::new(alternatives(width, 1)),
            Err(Error::Unsupported(_))
        ));
    }
    check(
        alternatives(16, 1),
        domains(&[1]),
        Preference::Absent,
        Outcome::Concrete(1),
    );
    limit(alternatives(17, 1));
    let depth = |levels: usize| (1..levels).fold(value(1), |expr, _| and(Expr::Top, expr));
    check(
        depth(16),
        domains(&[1]),
        Preference::Absent,
        Outcome::Concrete(1),
    );
    limit(depth(17));
    // 64 leaves + 63 And nodes = 127 occurrences; a binary Or adds two nodes.
    Graph::new(balanced(vec![value(1); 64])).expect("127-occurrence topology");
    let mut leaves = vec![value(1); 64];
    leaves[0] = alternatives(2, 1); // replaces one leaf with three nodes: 129.
    limit(balanced(leaves));
    let mut leaves = vec![value(1); 63]; // 125, plus a 3-child Or replacing a leaf =128.
    leaves[0] = alternatives(3, 1);
    Graph::new(balanced(leaves)).expect("exact 128-occurrence topology");
    Graph::new(balanced(vec![alternatives(2, 1); 16])).expect("16 groups");
    limit(balanced(vec![alternatives(2, 1); 17]));
    // Ordinary contradiction does not authorize skipping topology validation.
    limit(and(and(value(1), value(2)), alternatives(17, 1)));
}

#[test]
fn world_tables_and_raw_products_are_bounded_before_conflict_filtering() {
    // Equal domains still have distinct lineage: 16*4 = 64 raw worlds.
    check(
        and(alternatives(16, 1), alternatives(4, 1)),
        domains(&[1]),
        Preference::Absent,
        Outcome::Concrete(1),
    );
    for right in [1, 2] {
        let graph =
            Graph::new(and(alternatives(16, 1), alternatives(5, right))).expect("valid topology");
        assert!(
            matches!(graph.solve(4096), Err(Error::Limit(_))),
            "80 raw trials, even if every meet conflicts"
        );
    }
    let branches = (0..5).map(|_| (false, alternatives(16, 1))).collect();
    let graph = Graph::new(or(branches)).expect("six groups, bounded syntax");
    assert!(
        matches!(graph.solve(4096), Err(Error::Limit(_))),
        "80-entry union table"
    );
    // More than 64 trials in aggregate is legal when every individual batch fits.
    check(
        and(and(alternatives(8, 1), alternatives(8, 1)), value(1)),
        domains(&[1]),
        Preference::Absent,
        Outcome::Concrete(1),
    );
}

#[test]
fn lowered_work_allowances_are_cumulative_and_never_semantic_conflicts() {
    let graph = Graph::new(and(choice(1, 2), union(choice(1, 2), value(3)))).expect("graph");
    let full = graph.solve(4096).expect("full allowance");
    assert!(full.work > 1);
    assert!(matches!(graph.solve(4097), Err(Error::Limit(_))));
    for allowance in [0, 1, full.work - 1] {
        assert!(matches!(graph.solve(allowance), Err(Error::Limit(_))));
    }
    assert_eq!(
        graph.solve(full.work).expect("exact charged allowance"),
        full
    );
    assert_eq!(graph.solve(4096).expect("fresh solve after limits"), full);
}
