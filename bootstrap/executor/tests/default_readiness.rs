// Conservative test-only demand evidence, not general CUE readiness/conformance.
#[path = "../../spikes/default_provenance.rs"]
mod default_provenance;

use default_provenance::{
    Demand, DemandReport, Domain, Error, Expr, Graph, GroupFact, Outcome, Preference, Readiness,
    Support,
};
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
    values.iter().copied().map(Domain::Value).collect()
}
fn graph(expr: Expr) -> Graph {
    Graph::new(expr).expect("bounded independent scalar IR")
}
fn observe(graph: &Graph, refinement: Domain) -> DemandReport {
    let report = graph.demand(refinement, 4096).expect("bounded demand");
    assert!(report.work > 0 && report.work <= 4096);
    assert!(report.groups.windows(2).all(|g| g[0].group < g[1].group));
    assert_eq!(
        graph
            .demand(refinement, 4096)
            .expect("fresh repeated demand"),
        report
    );
    report
}
fn decision(
    report: &DemandReport,
    constraints: Vec<Domain>,
    readiness: Readiness,
    demand: Demand,
    preferred: Option<Preference>,
) {
    assert_eq!(report.constraints, constraints);
    assert_eq!(report.readiness, readiness);
    assert_eq!(report.demand, demand);
    assert_eq!(report.preferred, preferred);
}
fn fact(group: usize, support: Support, retained: &[usize], suppressed: &[usize]) -> GroupFact {
    GroupFact {
        group,
        support,
        retained: retained.to_vec(),
        suppressed: suppressed.to_vec(),
    }
}
fn counterexample() -> (Expr, Expr) {
    (
        or(vec![(false, choice(1, 2)), (true, value(3))]),
        choice(2, 1),
    )
}

// These are independent expected upstream classifications, not translations of
// native Suspended. Source attestation belongs to pinned Nix, not `(devel)`.
// CUE v0.16.1: source 6d609d768f1686f9a3a2a20197cacdbb70e5c79d.
#[derive(Clone, Copy)]
enum Oracle {
    Concrete(i64),
    Incomplete,
    Conflict,
}
struct ProbeFiles(PathBuf);
impl ProbeFiles {
    fn new(expression: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-readiness-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive owned probe directory");
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
        command.output().expect("mandatory pinned CUE process")
    }
}
impl Drop for ProbeFiles {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned probe files");
    }
}
fn oracle(expression: &str, expected: Oracle) {
    let probe = ProbeFiles::new(expression);
    let output = probe.run("export");
    let stderr = String::from_utf8_lossy(&output.stderr);
    match expected {
        Oracle::Concrete(n) => {
            assert!(output.status.success(), "{expression}: {stderr}");
            let actual: i64 = String::from_utf8(output.stdout)
                .expect("UTF-8")
                .trim()
                .parse()
                .expect("exact i64, never floating-point normalization");
            assert_eq!(actual, n, "independent oracle expectation: {expression}");
        }
        Oracle::Incomplete => {
            assert!(!output.status.success(), "{expression}");
            assert!(output.stdout.is_empty());
            assert!(
                stderr.contains("incomplete value"),
                "{expression}: {stderr}"
            );
            let evaluated = probe.run("eval");
            assert!(
                evaluated.status.success(),
                "not ordinary bottom: {expression}: {}",
                String::from_utf8_lossy(&evaluated.stderr)
            );
        }
        Oracle::Conflict => {
            assert!(!output.status.success(), "{expression}");
            assert!(output.stdout.is_empty());
            assert!(
                stderr.contains("conflicting values"),
                "{expression}: {stderr}"
            );
        }
    }
}
fn both_oracles(left: &str, right: &str, forward: Oracle, reverse: Oracle) {
    oracle(&format!("{left}&{right}"), forward);
    oracle(&format!("{right}&{left}"), reverse);
}

#[test]
fn pinned_cue_is_mandatory() {
    let output = Command::new("cue")
        .arg("version")
        .output()
        .expect("pinned CUE");
    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).expect("version UTF-8");
    assert!(
        version
            .lines()
            .any(|s| s.trim() == "CUE language version v0.16.1"),
        "unexpected toolchain: {version}"
    );
}

#[test]
fn both_order_counterexample_preserves_old_falsification_and_defers_new_demand() {
    let (a, b) = counterexample();
    for expr in [and(a.clone(), b.clone()), and(b, a)] {
        let graph = graph(expr);
        let old = graph.solve(4096).expect("unchanged diagnostic hypothesis");
        assert_eq!(old.constraints, domains(&[1, 2]));
        assert_eq!(old.preferred, Preference::Values(domains(&[2])));
        assert_eq!(old.outcome, Outcome::Concrete(2));
        assert!(old.work > 0 && old.work <= 4096);
        assert_eq!(graph.solve(4096).expect("fresh diagnostic solve"), old);
        for refinement in [Domain::Top, Domain::Int] {
            decision(
                &observe(&graph, refinement),
                domains(&[1, 2]),
                Readiness::DeferredNested,
                Demand::Suspended,
                None,
            );
        }
    }
    both_oracles(
        "((*1|2)|*3)",
        "(*2|1)",
        Oracle::Incomplete,
        Oracle::Incomplete,
    );
}

#[test]
fn refinement_audits_all_groups_and_broad_observation_recovers_original_facts() {
    let (a, b) = counterexample();
    let graph = graph(and(a, b));
    let broad = observe(&graph, Domain::Top);
    // Handwritten occurrence fixture: compilation uses postorder IDs. These
    // expectations concern this graph only, not IDs across rebuilds/orders.
    assert_eq!(
        broad.groups,
        vec![
            fact(2, Support::MarkedLive, &[], &[]),
            fact(4, Support::MarkedEliminated, &[], &[2]),
            fact(7, Support::MarkedLive, &[], &[]),
        ]
    );
    assert_eq!(observe(&graph, Domain::Int).groups, broad.groups);
    for (n, inner, right) in [
        (1, Support::MarkedLive, Support::MarkedEliminated),
        (2, Support::MarkedEliminated, Support::MarkedLive),
    ] {
        let refined = observe(&graph, Domain::Value(n));
        decision(
            &refined,
            domains(&[n]),
            Readiness::ConstraintOnly,
            Demand::Concrete(n),
            None,
        );
        assert_eq!(
            refined.groups,
            vec![
                fact(2, inner, &[], &[]),
                fact(4, Support::MarkedEliminated, &[], &[2]),
                fact(7, right, &[], &[]),
            ]
        );
    }
    let conflict = observe(&graph, Domain::Value(3));
    decision(
        &conflict,
        vec![],
        Readiness::ConstraintOnly,
        Demand::Conflict,
        None,
    );
    assert_eq!(
        conflict.groups,
        vec![
            fact(2, Support::Unentered, &[], &[]),
            fact(4, Support::Unentered, &[], &[2]),
            fact(7, Support::Unentered, &[], &[]),
        ]
    );
    assert_eq!(observe(&graph, Domain::Top), broad);
    for n in [1, 2] {
        oracle(&format!("(((*1|2)|*3)&(*2|1))&{n}"), Oracle::Concrete(n));
    }
    oracle("(((*1|2)|*3)&(*2|1))&3", Oracle::Conflict);
}

#[test]
fn flat_elimination_and_present_bottom_are_not_suspended_or_conflated() {
    for reversed in [false, true] {
        for (right, preferred, demand, support) in [
            (
                or(vec![(false, value(1)), (false, value(2)), (true, value(3))]),
                Preference::Values(domains(&[1])),
                Demand::Concrete(1),
                Support::MarkedEliminated,
            ),
            (
                or(vec![(false, value(1)), (true, value(2))]),
                Preference::Bottom,
                Demand::Incomplete,
                Support::MarkedLive,
            ),
        ] {
            let left = choice(1, 2);
            let graph = graph(if reversed {
                and(right, left)
            } else {
                and(left, right)
            });
            let report = observe(&graph, Domain::Top);
            decision(
                &report,
                domains(&[1, 2]),
                Readiness::FlatPreference,
                demand,
                Some(preferred),
            );
            assert_eq!(report.groups.len(), 2);
            assert_eq!(report.groups[usize::from(!reversed)].support, support);
            assert!(
                report
                    .groups
                    .iter()
                    .all(|g| g.retained.is_empty() && g.suppressed.is_empty())
            );
        }
    }
    both_oracles(
        "(*1|2)",
        "(1|2|*3)",
        Oracle::Concrete(1),
        Oracle::Concrete(1),
    );
    both_oracles("(*1|2)", "(1|*2)", Oracle::Incomplete, Oracle::Incomplete);
    // Evaluated absence is also distinct from not having evaluated preference.
    let absent = observe(
        &graph(and(choice(1, 2), union(value(2), value(3)))),
        Domain::Top,
    );
    // Ordinary constraints force 2, so preference is deliberately not evaluated.
    decision(
        &absent,
        domains(&[2]),
        Readiness::ConstraintOnly,
        Demand::Concrete(2),
        None,
    );
    let absent = observe(
        &graph(and(
            or(vec![(true, value(3)), (false, value(1)), (false, value(2))]),
            union(value(1), value(2)),
        )),
        Domain::Top,
    );
    decision(
        &absent,
        domains(&[1, 2]),
        Readiness::FlatPreference,
        Demand::Incomplete,
        Some(Preference::Absent),
    );
}

#[test]
fn conservative_coverage_cost_includes_previously_resolvable_nested_gap() {
    let a = choice(2, 3);
    let b = union(choice(1, 2), value(3));
    for expr in [and(a.clone(), b.clone()), and(b, a)] {
        decision(
            &observe(&graph(expr), Domain::Top),
            domains(&[2, 3]),
            Readiness::DeferredNested,
            Demand::Suspended,
            None,
        );
    }
    // Native Suspended is not an alias for CUE Incomplete: CUE exports 2.
    both_oracles(
        "(*2|3)",
        "((*1|2)|3)",
        Oracle::Concrete(2),
        Oracle::Concrete(2),
    );
}

#[test]
fn all_known_oracle_asymmetries_are_deferred_without_imitating_order() {
    let cases = [
        (
            choice(1, 2),
            or(vec![(false, choice(1, 2)), (true, value(3))]),
            domains(&[1, 2]),
            "(*1|2)",
            "((*1|2)|*3)",
            Oracle::Concrete(1),
            Oracle::Incomplete,
        ),
        (
            or(vec![(true, Expr::Int), (false, value(1))]),
            union(choice(1, 2), value(3)),
            domains(&[1, 2, 3]),
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
            "(*((*1|2)&(1|*2))|3)",
            "(*3|1|2)",
            Oracle::Incomplete,
            Oracle::Concrete(2),
        ),
    ];
    for (a, b, ordinary, left, right, forward, reverse) in cases {
        for expr in [and(a.clone(), b.clone()), and(b, a)] {
            decision(
                &observe(&graph(expr), Domain::Top),
                ordinary.clone(),
                Readiness::DeferredNested,
                Demand::Suspended,
                None,
            );
        }
        both_oracles(left, right, forward, reverse);
    }
}

#[test]
fn m2_retention_m3_suppression_and_source_order_survive_unentered_branches() {
    let graph = graph(or(vec![
        (false, choice(1, 2)),
        (true, choice(3, 4)),
        (false, choice(5, 6)),
    ]));
    let broad = observe(&graph, Domain::Top);
    assert_eq!(
        broad.groups,
        vec![
            fact(2, Support::MarkedLive, &[], &[]),
            fact(5, Support::MarkedLive, &[], &[]),
            fact(8, Support::MarkedLive, &[], &[]),
            fact(9, Support::MarkedLive, &[5], &[2, 8]),
        ]
    );
    decision(
        &broad,
        domains(&[1, 2, 3, 4, 5, 6]),
        Readiness::DeferredNested,
        Demand::Suspended,
        None,
    );
    let refined = observe(&graph, Domain::Value(1));
    assert_eq!(
        refined.groups,
        vec![
            fact(2, Support::MarkedLive, &[], &[]),
            fact(5, Support::Unentered, &[], &[]),
            fact(8, Support::Unentered, &[], &[]),
            fact(9, Support::MarkedEliminated, &[5], &[2, 8]),
        ]
    );
    decision(
        &refined,
        domains(&[1]),
        Readiness::ConstraintOnly,
        Demand::Concrete(1),
        None,
    );
    assert_eq!(observe(&graph, Domain::Top), broad);
    let unmarked = observe(
        &Graph::new(union(choice(1, 2), choice(3, 4))).expect("bounded unmarked fixture"),
        Domain::Top,
    );
    assert_eq!(
        unmarked.groups,
        vec![
            fact(2, Support::MarkedLive, &[], &[]),
            fact(5, Support::MarkedLive, &[], &[]),
            fact(6, Support::Unmarked, &[2, 5], &[]),
        ]
    );
    decision(
        &unmarked,
        domains(&[1, 2, 3, 4]),
        Readiness::DeferredNested,
        Demand::Suspended,
        None,
    );
}

#[test]
fn sibling_values_do_not_enter_or_reactivate_equal_spelling_occurrences() {
    // First inner group is entered through value 2, but its marked 1 is dead;
    // sibling 1 must not reactivate it. The second equal-spelling group is dead.
    let graph = graph(union(
        and(choice(1, 2), value(2)),
        union(value(1), and(choice(1, 2), value(9))),
    ));
    let broad = observe(&graph, Domain::Top);
    assert_eq!(
        broad.groups,
        vec![
            fact(2, Support::MarkedEliminated, &[], &[]),
            fact(8, Support::Unentered, &[], &[]),
            fact(11, Support::Unmarked, &[10], &[]),
            fact(12, Support::Unmarked, &[4, 11], &[]),
        ]
    );
    decision(
        &broad,
        domains(&[1, 2]),
        Readiness::DeferredNested,
        Demand::Suspended,
        None,
    );
    let refined = observe(&graph, Domain::Value(1));
    assert_eq!(refined.groups[0].support, Support::Unentered);
    assert_eq!(refined.groups[1].support, Support::Unentered);
    assert_eq!(refined.groups[2].retained, vec![10]);
    let exact_two = observe(&graph, Domain::Value(2));
    decision(
        &exact_two,
        domains(&[2]),
        Readiness::ConstraintOnly,
        Demand::Concrete(2),
        None,
    );
    assert_eq!(
        exact_two.groups,
        vec![
            fact(2, Support::MarkedEliminated, &[], &[]),
            fact(8, Support::Unentered, &[], &[]),
            fact(11, Support::Unentered, &[10], &[]),
            fact(12, Support::Unmarked, &[4, 11], &[]),
        ]
    );
    assert_eq!(observe(&graph, Domain::Top), broad);
}

#[test]
fn abstract_domains_and_forced_scalar_precedence_are_explicit() {
    for domain in [Domain::Top, Domain::Int] {
        let expr = if domain == Domain::Top {
            Expr::Top
        } else {
            Expr::Int
        };
        decision(
            &observe(&graph(expr), Domain::Top),
            vec![domain],
            Readiness::ConstraintOnly,
            Demand::Incomplete,
            None,
        );
    }
    for atom in [Expr::Top, Expr::Int] {
        let abstract_domain = if atom == Expr::Top {
            Domain::Top
        } else {
            Domain::Int
        };
        let graph = graph(or(vec![(true, atom), (false, value(1))]));
        decision(
            &observe(&graph, Domain::Top),
            vec![abstract_domain, Domain::Value(1)],
            Readiness::FlatPreference,
            Demand::Incomplete,
            Some(Preference::Values(vec![abstract_domain])),
        );
        for n in [i64::MIN, 1, i64::MAX] {
            decision(
                &observe(&graph, Domain::Value(n)),
                domains(&[n]),
                Readiness::ConstraintOnly,
                Demand::Concrete(n),
                None,
            );
        }
    }
    // Deduplication is not subsumption: no recipe and abstract + literal stays incomplete.
    decision(
        &observe(
            &graph(or(vec![
                (false, value(1)),
                (false, Expr::Int),
                (false, Expr::Top),
                (false, value(1)),
            ])),
            Domain::Top,
        ),
        vec![Domain::Top, Domain::Int, Domain::Value(1)],
        Readiness::ConstraintOnly,
        Demand::Incomplete,
        None,
    );
    let nested = graph(or(vec![(true, choice(1, 1)), (false, value(1))]));
    decision(
        &observe(&nested, Domain::Top),
        domains(&[1]),
        Readiness::ConstraintOnly,
        Demand::Concrete(1),
        None,
    );
    oracle("*int|1", Oracle::Incomplete);
    oracle("*_ |1", Oracle::Incomplete);
    oracle("*(*1|1)|1", Oracle::Concrete(1));
    oracle(&format!("int&{}", i64::MIN), Oracle::Concrete(i64::MIN));
    oracle(&format!("int&{}", i64::MAX), Oracle::Concrete(i64::MAX));
}

fn alternatives(width: usize, n: i64) -> Expr {
    or((0..width).map(|_| (false, value(n))).collect())
}
fn balanced(mut nodes: Vec<Expr>) -> Expr {
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
    nodes.pop().expect("nonempty fixture")
}

#[test]
fn full_topology_caps_still_allow_complete_bounded_audits() {
    let full = graph(balanced(vec![choice(1, 2); 16]));
    let report = observe(&full, Domain::Top);
    decision(
        &report,
        domains(&[1, 2]),
        Readiness::FlatPreference,
        Demand::Concrete(1),
        Some(Preference::Values(domains(&[1]))),
    );
    assert_eq!(report.groups.len(), 16);
    assert!(
        report
            .groups
            .iter()
            .all(|g| g.support == Support::MarkedLive)
    );
    assert!(matches!(
        Graph::new(balanced(vec![choice(1, 2); 17])),
        Err(Error::Limit(_))
    ));
    // 63 leaves + 62 conjunctions, then replace one leaf by a four-node Or.
    let mut leaves = vec![value(1); 63];
    leaves[0] = alternatives(3, 1);
    let full = graph(balanced(leaves));
    decision(
        &observe(&full, Domain::Top),
        domains(&[1]),
        Readiness::ConstraintOnly,
        Demand::Concrete(1),
        None,
    );
    let mut leaves = vec![value(1); 64];
    leaves[0] = alternatives(2, 1); // 129 occurrences.
    assert!(matches!(Graph::new(balanced(leaves)), Err(Error::Limit(_))));
}

#[test]
fn syntactic_m2_survives_contextually_absent_child_without_m1_revival() {
    let graph = graph(or(vec![
        (true, and(choice(1, 2), value(2))),
        (false, value(3)),
    ]));
    let report = observe(&graph, Domain::Top);
    assert_eq!(
        report.groups,
        vec![
            fact(2, Support::MarkedEliminated, &[], &[]),
            fact(6, Support::MarkedLive, &[4], &[]),
        ]
    );
    decision(
        &report,
        domains(&[2, 3]),
        Readiness::DeferredNested,
        Demand::Suspended,
        None,
    );
    assert_eq!(
        graph.solve(4096).expect("diagnostic M2 solve").preferred,
        Preference::Bottom
    );
    decision(
        &observe(&graph, Domain::Value(2)),
        domains(&[2]),
        Readiness::ConstraintOnly,
        Demand::Concrete(2),
        None,
    );
    assert_eq!(observe(&graph, Domain::Top), report);
}
fn work_boundary(graph: &Graph, refinement: Domain) {
    let full = observe(graph, refinement);
    for allowance in [0, 1, full.work - 1, 4097] {
        assert!(
            matches!(graph.demand(refinement, allowance), Err(Error::Limit(_))),
            "allowance {allowance}"
        );
    }
    assert_eq!(
        graph
            .demand(refinement, full.work)
            .expect("exact charged allowance"),
        full
    );
    assert_eq!(observe(graph, refinement), full);
}

#[test]
fn work_is_cumulative_through_auditing_flat_projection_and_failure_recovery() {
    for expr in [
        value(1),
        Expr::Int,
        and(choice(1, 2), choice(2, 1)),
        and(
            choice(1, 2),
            or(vec![(false, value(1)), (false, value(2)), (true, value(3))]),
        ),
        and(counterexample().0, counterexample().1),
    ] {
        let graph = graph(expr);
        for refinement in [Domain::Top, Domain::Int, Domain::Value(1), Domain::Value(9)] {
            work_boundary(&graph, refinement);
        }
    }
    let flat = graph(and(choice(1, 2), choice(2, 1)));
    let old = flat.solve(4096).expect("bounded flat diagnostic solve");
    let new = observe(&flat, Domain::Top);
    assert!(
        new.work > old.work,
        "new refinement/audit work cannot be free or reset"
    );
    assert!(matches!(
        flat.demand(Domain::Top, old.work),
        Err(Error::Limit(_))
    ));
    assert_eq!(
        flat.solve(4096)
            .expect("diagnostic solve after demand failure"),
        old
    );
}

#[test]
fn raw_product_and_union_limits_precede_filtering_even_for_exact_refinement() {
    for right in [1, 2] {
        let graph = graph(and(alternatives(16, 1), alternatives(5, right)));
        for refinement in [Domain::Top, Domain::Value(9)] {
            assert!(matches!(
                graph.demand(refinement, 4096),
                Err(Error::Limit(_))
            ));
        }
    }
    let union_table = graph(or((0..5).map(|_| (false, alternatives(16, 1))).collect()));
    assert!(matches!(
        union_table.demand(Domain::Top, 4096),
        Err(Error::Limit(_))
    ));
    for expr in [
        and(alternatives(16, 1), alternatives(4, 1)),
        and(and(alternatives(8, 1), alternatives(8, 1)), value(1)),
    ] {
        decision(
            &observe(&graph(expr), Domain::Top),
            domains(&[1]),
            Readiness::ConstraintOnly,
            Demand::Concrete(1),
            None,
        );
    }
    for width in [0, 1] {
        assert!(matches!(
            Graph::new(alternatives(width, 1)),
            Err(Error::Unsupported(_))
        ));
    }
    assert!(matches!(
        Graph::new(alternatives(17, 1)),
        Err(Error::Limit(_))
    ));
    let depth = |levels: usize| (1..levels).fold(value(1), |expr, _| and(Expr::Top, expr));
    work_boundary(&graph(depth(16)), Domain::Top);
    assert!(matches!(Graph::new(depth(17)), Err(Error::Limit(_))));
}

#[test]
fn bounded_permutations_preserve_flat_vs_nested_decisions_in_both_orders() {
    let terms = [(true, value(1)), (false, value(2)), (false, value(3))];
    let spellings = ["*1", "2", "3"];
    for p in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let flat = or(p.map(|i| terms[i].clone()).to_vec());
        let source = format!(
            "({}|{}|{})",
            spellings[p[0]], spellings[p[1]], spellings[p[2]]
        );
        for reversed in [false, true] {
            let combine = |left, right| {
                if reversed {
                    and(right, left)
                } else {
                    and(left, right)
                }
            };
            decision(
                &observe(&graph(combine(flat.clone(), Expr::Int)), Domain::Top),
                domains(&[1, 2, 3]),
                Readiness::FlatPreference,
                Demand::Concrete(1),
                Some(Preference::Values(domains(&[1]))),
            );
            decision(
                &observe(
                    &graph(combine(union(flat.clone(), value(4)), Expr::Int)),
                    Domain::Top,
                ),
                domains(&[1, 2, 3, 4]),
                Readiness::DeferredNested,
                Demand::Suspended,
                None,
            );
            decision(
                &observe(
                    &graph(combine(union(flat.clone(), value(4)), value(2))),
                    Domain::Top,
                ),
                domains(&[2]),
                Readiness::ConstraintOnly,
                Demand::Concrete(2),
                None,
            );
        }
        both_oracles(&source, "int", Oracle::Concrete(1), Oracle::Concrete(1));
    }
}
