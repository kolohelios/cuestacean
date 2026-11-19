#[path = "../../spikes/default_provenance.rs"]
mod spike;

use spike::{
    Demand, Domain, Error, Expr, Graph, GroupFact, Outcome, Preference, Readiness, Support,
    policy::Policy,
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn value(n: i64) -> Expr {
    Expr::Value(n)
}
fn marked(a: i64, b: i64) -> Expr {
    Expr::Or(vec![(true, value(a)), (false, value(b))])
}
fn union(a: Expr, b: Expr) -> Expr {
    Expr::Or(vec![(false, a), (false, b)])
}
fn and(a: Expr, b: Expr) -> Expr {
    Expr::And(Box::new(a), Box::new(b))
}
fn domains(ns: &[i64]) -> Vec<Domain> {
    let mut sorted = ns.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted.into_iter().map(Domain::Value).collect()
}
fn recipe(expr: &Expr) -> bool {
    match expr {
        Expr::Top | Expr::Int | Expr::Value(_) => false,
        Expr::And(a, b) => recipe(a) || recipe(b),
        Expr::Or(terms) => terms.iter().any(|(mark, child)| *mark || recipe(child)),
    }
}
// Characterizes the rejected proposal; never used for runtime admission.
fn candidate(expr: &Expr) -> bool {
    match expr {
        Expr::Top | Expr::Int => false,
        Expr::Value(_) => true,
        Expr::And(a, b) => candidate(a) && candidate(b),
        Expr::Or(terms) => {
            terms.iter().all(|(_, child)| candidate(child))
                && !(terms.iter().any(|(mark, _)| *mark)
                    && terms.iter().any(|(_, child)| recipe(child)))
        }
    }
}

struct Policies {
    dispatch: Policy,
    early: Policy,
    projection: Policy,
    late: Policy,
}
impl Policies {
    fn native() -> Self {
        let root = PathBuf::from(
            std::env::var_os("CUESTACEAN_DEMAND_POLICIES")
                .expect("mandatory native policy artifact directory"),
        );
        let read = |name: &str| {
            fs::read_to_string(root.join(format!("{name}.json")))
                .expect("mandatory native policy artifact")
        };
        Self {
            dispatch: Policy::from_json(&read("demand-policy")).expect("dispatch scope"),
            early: Policy::from_json(&read("demand-policy-defer")).expect("dispatch defer scope"),
            projection: Policy::from_projection_json(&read("preference-projection"))
                .expect("projection scope"),
            late: Policy::from_projection_json(&read("preference-projection-defer"))
                .expect("projection defer scope"),
        }
    }
}
fn flat() -> Policy {
    Policy::from_json(r#"{"formatVersion":1,"body":{"op":"return","action":"flat"}}"#)
        .expect("explicit unsafe flat request")
}

#[derive(Clone, Copy, Debug)]
enum Case {
    AsymmetricValues,
    AsymmetricBottom,
    BothOrderFailure,
    NestedAgreement,
    FlatBottom,
}
impl Case {
    fn operands(self, [a, b, c]: [i64; 3]) -> (Expr, Expr) {
        match self {
            Self::AsymmetricValues => (union(marked(a, b), marked(b, a)), marked(a, b)),
            Self::AsymmetricBottom => (
                union(and(marked(a, b), marked(a, c)), value(c)),
                marked(c, a),
            ),
            Self::BothOrderFailure => (
                union(marked(c, a), value(c)),
                union(marked(a, b), marked(c, b)),
            ),
            Self::NestedAgreement => (marked(b, c), union(marked(a, b), value(c))),
            Self::FlatBottom => (marked(a, b), marked(b, a)),
        }
    }
    // Separately authored CUE snippets, not serialization of the Expr or reports.
    fn sources(self, [a, b, c]: [i64; 3]) -> (String, String) {
        match self {
            Self::AsymmetricValues => (format!("((*{a}|{b})|(*{b}|{a}))"), format!("(*{a}|{b})")),
            Self::AsymmetricBottom => (
                format!("(((*{a}|{b})&(*{a}|{c}))|{c})"),
                format!("(*{c}|{a})"),
            ),
            Self::BothOrderFailure => (
                format!("((*{c}|{a})|{c})"),
                format!("((*{a}|{b})|(*{c}|{b}))"),
            ),
            Self::NestedAgreement => (format!("(*{b}|{c})"), format!("((*{a}|{b})|{c})")),
            Self::FlatBottom => (format!("(*{a}|{b})"), format!("(*{b}|{a})")),
        }
    }
    fn expected(self, [a, b, c]: [i64; 3]) -> (Vec<Domain>, Preference, Outcome) {
        match self {
            Self::AsymmetricValues => (
                domains(&[a, b]),
                Preference::Values(domains(&[a])),
                Outcome::Concrete(a),
            ),
            Self::AsymmetricBottom => (domains(&[a, c]), Preference::Bottom, Outcome::Incomplete),
            Self::BothOrderFailure => (
                domains(&[a, c]),
                Preference::Values(domains(&[c])),
                Outcome::Concrete(c),
            ),
            Self::NestedAgreement => (
                domains(&[b, c]),
                Preference::Values(domains(&[b])),
                Outcome::Concrete(b),
            ),
            Self::FlatBottom => (domains(&[a, b]), Preference::Bottom, Outcome::Incomplete),
        }
    }
    fn oracle(self, [a, b, c]: [i64; 3]) -> [Oracle; 2] {
        match self {
            Self::AsymmetricValues => [Oracle::Concrete(a), Oracle::Incomplete],
            Self::AsymmetricBottom => [Oracle::Concrete(c), Oracle::Incomplete],
            Self::BothOrderFailure | Self::FlatBottom => [Oracle::Incomplete; 2],
            Self::NestedAgreement => [Oracle::Concrete(b); 2],
        }
    }
    fn nested(self) -> bool {
        !matches!(self, Self::FlatBottom)
    }
}

struct Probe(PathBuf);
impl Probe {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "cuestacean-transparent-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("exclusive oracle directory");
        fs::write(root.join("schema.cue"), "package probe\n").expect("schema");
        fs::write(
            root.join("program.cue"),
            format!("package probe\nvalue: {source}\n"),
        )
        .expect("program");
        Self(root)
    }
    fn run(&self, operation: &str) -> std::process::Output {
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
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned oracle files");
    }
}
#[derive(Clone, Copy)]
enum Oracle {
    Concrete(i64),
    Incomplete,
    Conflict,
}
fn oracle(source: &str, expected: Oracle) {
    let files = Probe::new(source);
    let output = files.run("export");
    let stderr = String::from_utf8_lossy(&output.stderr);
    match expected {
        Oracle::Concrete(n) => {
            assert!(output.status.success(), "{source}: {stderr}");
            let actual: i64 = String::from_utf8(output.stdout)
                .expect("integer UTF-8")
                .trim()
                .parse()
                .expect("exact integer, no floating point normalization");
            assert_eq!(actual, n, "{source}");
        }
        Oracle::Incomplete => {
            assert!(!output.status.success(), "{source}");
            assert!(output.stdout.is_empty(), "no partial JSON: {source}");
            assert!(stderr.contains("incomplete value"), "{source}: {stderr}");
            let evaluated = files.run("eval");
            assert!(evaluated.status.success(), "not bottom: {source}");
        }
        Oracle::Conflict => {
            assert!(!output.status.success(), "{source}");
            assert!(output.stdout.is_empty(), "no partial JSON: {source}");
            assert!(stderr.contains("conflicting values"), "{source}: {stderr}");
            assert!(!files.run("eval").status.success(), "bottom: {source}");
        }
    }
}

#[test]
fn bounded_renaming_matrix_preserves_three_distinct_evidence_classes() {
    // Pinned CUE source 6d609d768f1686f9a3a2a20197cacdbb70e5c79d,
    // Nixpkgs ac588ef1c914865dad517b337ea0eb9694a4021c; display alone is not attestation.
    let version = Command::new("cue")
        .arg("version")
        .output()
        .expect("pinned CUE");
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("v0.16.1"));
    let policies = Policies::native();
    for names in [
        [1, 2, 3],
        [1, 3, 2],
        [2, 1, 3],
        [2, 3, 1],
        [3, 1, 2],
        [3, 2, 1],
    ] {
        for case in [
            Case::AsymmetricValues,
            Case::AsymmetricBottom,
            Case::BothOrderFailure,
            Case::NestedAgreement,
            Case::FlatBottom,
        ] {
            let (left, right) = case.operands(names);
            let (ls, rs) = case.sources(names);
            for (order, expr, source) in [
                (0, and(left.clone(), right.clone()), format!("{ls}&{rs}")),
                (1, and(right, left), format!("{rs}&{ls}")),
            ] {
                assert!(candidate(&expr));
                let graph = Graph::new(expr).expect("bounded independent IR");
                let report = graph.solve(4096).expect("unchanged static hypothesis");
                let (constraints, preferred, outcome) = case.expected(names);
                assert_eq!(report.constraints, constraints, "{case:?}/{order}");
                assert_eq!(report.preferred, preferred, "{case:?}/{order}");
                assert_eq!(report.outcome, outcome, "{case:?}/{order}");
                let native = graph.demand(Domain::Top, 4096).expect("native demand");
                let single = graph
                    .demand_with_policy(Domain::Top, 4096, &policies.dispatch)
                    .expect("single dispatch");
                let two = graph
                    .demand_with_policies(
                        Domain::Top,
                        4096,
                        &policies.dispatch,
                        &policies.projection,
                    )
                    .expect("two policy demand");
                for observed in [&native, &single, &two] {
                    assert_eq!(observed.constraints, constraints);
                    assert_eq!(observed.groups, native.groups);
                }
                if case.nested() {
                    assert_eq!(native.readiness, Readiness::DeferredNested);
                    for observed in [&native, &single, &two] {
                        assert_eq!(observed.preferred, None);
                        assert_eq!(observed.demand, Demand::Suspended);
                    }
                    assert_eq!(single.readiness, Readiness::PolicyDeferred);
                    assert_eq!(two.readiness, Readiness::PolicyDeferred);
                } else {
                    for observed in [&native, &single, &two] {
                        assert_eq!(observed.preferred, Some(Preference::Bottom));
                        assert_eq!(observed.demand, Demand::Incomplete);
                        assert_eq!(observed.readiness, Readiness::FlatPreference);
                    }
                }
                oracle(&source, case.oracle(names)[order]);
            }
        }
    }
}

fn genuine(reverse: bool) -> Graph {
    let (left, right) = Case::BothOrderFailure.operands([1, 2, 3]);
    Graph::new(if reverse {
        and(right, left)
    } else {
        and(left, right)
    })
    .expect("genuine case")
}
fn fact(group: usize, support: Support, retained: &[usize]) -> GroupFact {
    GroupFact {
        group,
        support,
        retained: retained.to_vec(),
        suppressed: vec![],
    }
}
#[test]
fn genuine_failure_has_live_support_and_retained_not_suppressed_recipes() {
    for reverse in [false, true] {
        let graph = genuine(reverse);
        let report = graph.demand(Domain::Top, 4096).expect("audited demand");
        let expected = if reverse {
            vec![
                fact(2, Support::MarkedLive, &[]),
                fact(5, Support::MarkedLive, &[]),
                fact(6, Support::Unmarked, &[2, 5]),
                fact(9, Support::MarkedLive, &[]),
                fact(11, Support::Unmarked, &[9]),
            ]
        } else {
            vec![
                fact(2, Support::MarkedLive, &[]),
                fact(4, Support::Unmarked, &[2]),
                fact(7, Support::MarkedLive, &[]),
                fact(10, Support::MarkedLive, &[]),
                fact(11, Support::Unmarked, &[7, 10]),
            ]
        };
        assert_eq!(report.groups, expected);
        assert_eq!(report.work, if reverse { 178 } else { 179 });
        assert_eq!(
            graph.solve(4096).expect("static").work,
            if reverse { 160 } else { 161 }
        );
    }
}

#[test]
fn refinements_force_or_conflict_then_restore_dormant_occurrence_facts() {
    let policies = Policies::native();
    for reverse in [false, true] {
        let graph = genuine(reverse);
        let original = graph.demand(Domain::Top, 4096).expect("original");
        for (n, expected) in [
            (1, Demand::Concrete(1)),
            (3, Demand::Concrete(3)),
            (2, Demand::Conflict),
        ] {
            let native = graph.demand(Domain::Value(n), 4096).expect("refined");
            let single = graph
                .demand_with_policy(Domain::Value(n), 4096, &policies.dispatch)
                .expect("single refined");
            let two = graph
                .demand_with_policies(
                    Domain::Value(n),
                    4096,
                    &policies.dispatch,
                    &policies.projection,
                )
                .expect("two refined");
            for report in [&native, &single, &two] {
                assert_eq!(report.demand, expected);
                assert_eq!(report.readiness, Readiness::ConstraintOnly);
                assert_eq!(report.preferred, None);
                assert_eq!(
                    report.constraints,
                    if n == 2 { vec![] } else { domains(&[n]) }
                );
                assert_eq!(report.groups, native.groups);
                assert_eq!(
                    report
                        .groups
                        .iter()
                        .map(|g| (g.group, &g.retained, &g.suppressed))
                        .collect::<Vec<_>>(),
                    original
                        .groups
                        .iter()
                        .map(|g| (g.group, &g.retained, &g.suppressed))
                        .collect::<Vec<_>>()
                );
            }
            assert!(
                native
                    .groups
                    .iter()
                    .any(|g| g.support == Support::Unentered
                        || g.support == Support::MarkedEliminated)
            );
            assert_eq!(
                graph.demand(Domain::Top, 4096),
                Ok(graph
                    .demand(Domain::Int, 4096)
                    .expect("finite int refinement"))
            );
            assert_eq!(
                graph.demand(Domain::Top, 4096),
                Ok(graph.demand(Domain::Top, 4096).expect("fresh"))
            );
        }
        assert_eq!(graph.demand(Domain::Top, 4096), Ok(original));
    }
    oracle("(((*3|1)|3)&((*1|2)|(*3|2)))&2", Oracle::Conflict);
}

#[test]
fn every_reference_api_retains_exact_work_limits_and_recovers() {
    let policies = Policies::native();
    let graph = genuine(false);
    let solved = graph.solve(4096).expect("solve");
    assert_eq!(
        graph.solve(solved.work),
        Ok(graph.solve(4096).expect("fresh solve"))
    );
    assert!(matches!(graph.solve(solved.work - 1), Err(Error::Limit(_))));
    for kind in 0..3 {
        let run = |limit| match kind {
            0 => graph.demand(Domain::Top, limit),
            1 => graph.demand_with_policy(Domain::Top, limit, &policies.dispatch),
            _ => graph.demand_with_policies(
                Domain::Top,
                limit,
                &policies.dispatch,
                &policies.projection,
            ),
        };
        let report = run(4096).expect("full report");
        assert_eq!(run(report.work), Ok(run(4096).expect("exact allowance")));
        for limit in [0, report.work - 1, 4097] {
            assert!(matches!(run(limit), Err(Error::Limit(_))));
        }
        assert_eq!(run(4096), Ok(report));
    }
}

#[test]
fn nested_flat_requests_remain_unsupported_and_scopes_precede_work() {
    let policies = Policies::native();
    let graph = genuine(false);
    for reverse in [false, true] {
        let graph = genuine(reverse);
        assert!(matches!(
            graph.demand_with_policy(Domain::Top, 4096, &flat()),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(
            graph.demand_with_policies(Domain::Top, 4096, &flat(), &policies.projection),
            Err(Error::Unsupported(_))
        ));
    }
    for limit in [0, 4097] {
        assert!(matches!(
            graph.demand_with_policy(Domain::Top, limit, &policies.projection),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(
            graph.demand_with_policies(
                Domain::Top,
                limit,
                &policies.projection,
                &policies.dispatch
            ),
            Err(Error::Unsupported(_))
        ));
    }
    let early = graph
        .demand_with_policies(Domain::Top, 4096, &policies.early, &policies.projection)
        .expect("early defer");
    assert_eq!(early.readiness, Readiness::PolicyDeferred);
    assert_eq!(early.preferred, None);
    assert_eq!(early.demand, Demand::Suspended);
    let flat_graph = Graph::new(marked(1, 2)).expect("flat graph");
    let late = flat_graph
        .demand_with_policies(Domain::Top, 4096, &policies.dispatch, &policies.late)
        .expect("late defer");
    assert_eq!(late.readiness, Readiness::ProjectionDeferred);
    assert_eq!(late.preferred, Some(Preference::Values(domains(&[1]))));
    assert_eq!(late.demand, Demand::Suspended);
}

#[test]
fn candidate_predicate_is_whole_syntax_characterization_not_a_new_gate() {
    for atom in [Expr::Top, Expr::Int] {
        assert!(!candidate(&union(value(1), and(value(1), atom))));
    }
    for mark_inner in [false, true] {
        let bad = Expr::Or(vec![(true, value(3)), (mark_inner, marked(1, 2))]);
        let unreachable = union(value(1), and(value(4), bad));
        assert!(!candidate(&unreachable));
        let graph = Graph::new(unreachable).expect("old construction still accepts syntax");
        let report = graph
            .demand(Domain::Value(1), 4096)
            .expect("ordinary forced value");
        assert_eq!(report.demand, Demand::Concrete(1));
        assert_eq!(report.preferred, None);
        assert!(
            report
                .groups
                .iter()
                .any(|g| g.support == Support::Unentered)
        );
    }
    let absent = Graph::new(and(
        Expr::Or(vec![(true, value(3)), (false, value(1)), (false, value(2))]),
        union(value(1), value(2)),
    ))
    .expect("flat eliminated mark");
    let report = absent.demand(Domain::Top, 4096).expect("flat absence");
    assert_eq!(report.preferred, Some(Preference::Absent));
    assert_eq!(report.demand, Demand::Incomplete);
    assert!(
        report
            .groups
            .iter()
            .any(|g| g.support == Support::MarkedEliminated)
    );
}
