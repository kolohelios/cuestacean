#[path = "../../spikes/context_preferences.rs"]
mod context_preferences;
#[path = "../../spikes/default_provenance.rs"]
mod default_provenance;

use context_preferences::{ContextFact, ContextReport, solve};
use default_provenance::{
    Demand, Domain, Error, Expr, Graph, Outcome, Preference, Readiness, policy::Policy,
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn v(n: i64) -> Expr {
    Expr::Value(n)
}
fn p(a: i64, b: i64) -> Expr {
    Expr::Or(vec![(true, v(a)), (false, v(b))])
}
fn u(a: Expr, b: Expr) -> Expr {
    Expr::Or(vec![(false, a), (false, b)])
}
fn and(a: Expr, b: Expr) -> Expr {
    Expr::And(Box::new(a), Box::new(b))
}
fn outer(a: Expr, b: Expr) -> Expr {
    Expr::Or(vec![(true, a), (false, b)])
}
fn ds(values: &[i64]) -> Vec<Domain> {
    let mut values = values.to_vec();
    values.sort_unstable();
    values.dedup();
    values.into_iter().map(Domain::Value).collect()
}
fn pref(values: &[i64]) -> Preference {
    if values.is_empty() {
        Preference::Bottom
    } else {
        Preference::Values(ds(values))
    }
}
fn fact(choices: &[(usize, usize)], constraints: &[i64], preferred: Preference) -> ContextFact {
    ContextFact {
        choices: choices.to_vec(),
        constraints: ds(constraints),
        preferred,
    }
}
fn report(r: &ContextReport, constraints: &[i64], preferred: Preference, outcome: Outcome) {
    assert_eq!(r.constraints, ds(constraints));
    assert_eq!(r.preferred, preferred);
    assert_eq!(r.outcome, outcome);
    assert!(r.contexts.windows(2).all(|w| w[0].choices < w[1].choices));
    for c in &r.contexts {
        assert!(c.choices.windows(2).all(|w| w[0] < w[1]));
        assert!(c.constraints.windows(2).all(|w| w[0] < w[1]));
    }
}

struct Probe(PathBuf);
impl Probe {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "cuestacean-context-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("exclusive oracle directory");
        fs::write(root.join("schema.cue"), "package probe\n").expect("raw schema");
        fs::write(
            root.join("program.cue"),
            format!("package probe\nvalue: {source}\n"),
        )
        .expect("independent raw program");
        Self(root)
    }
    fn run(&self, op: &str) -> std::process::Output {
        Command::new("cue")
            .arg(op)
            .arg(self.0.join("schema.cue"))
            .arg(self.0.join("program.cue"))
            .args(["-e", "value"])
            .output()
            .expect("mandatory pinned CUE process")
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned oracle files");
    }
}
#[derive(Clone, Copy)]
enum Oracle {
    Value(i64),
    Incomplete,
    Conflict,
}
fn oracle(source: &str, expected: Oracle) {
    let files = Probe::new(source);
    let output = files.run("export");
    let stderr = String::from_utf8_lossy(&output.stderr);
    match expected {
        Oracle::Value(n) => {
            assert!(output.status.success(), "{source}: {stderr}");
            assert_eq!(
                String::from_utf8(output.stdout)
                    .expect("UTF-8")
                    .trim()
                    .parse::<i64>()
                    .expect("exact integer"),
                n,
                "{source}"
            );
        }
        Oracle::Incomplete | Oracle::Conflict => {
            assert!(!output.status.success(), "{source}");
            assert!(output.stdout.is_empty(), "no partial JSON: {source}");
            let incomplete = matches!(expected, Oracle::Incomplete);
            assert!(
                stderr.contains(if incomplete {
                    "incomplete value"
                } else {
                    "conflicting values"
                }),
                "{source}: {stderr}"
            );
            assert_eq!(files.run("eval").status.success(), incomplete, "{source}");
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Case {
    ContextUnionGap,
    MarkedAncestorGap,
    Transparent,
    OldMarked,
    AsymValues,
    AsymBottom,
    Agreement,
    Flat,
    Dead,
}
impl Case {
    fn operands(self, [a, b, c, d, e]: [i64; 5]) -> (Expr, Expr) {
        match self {
            Self::ContextUnionGap => (u(and(p(a, b), p(b, a)), v(a)), p(a, b)),
            Self::MarkedAncestorGap => (outer(u(p(a, b), v(c)), p(d, e)), u(v(a), v(d))),
            Self::Transparent => (u(p(c, a), v(c)), u(p(a, b), p(c, b))),
            Self::OldMarked => (Expr::Or(vec![(false, p(a, b)), (true, v(c))]), p(b, a)),
            Self::AsymValues => (u(p(a, b), p(b, a)), p(a, b)),
            Self::AsymBottom => (u(and(p(a, b), p(a, c)), v(c)), p(c, a)),
            Self::Agreement => (p(b, c), u(p(a, b), v(c))),
            Self::Flat => (p(a, b), p(b, a)),
            Self::Dead => (
                p(a, b),
                Expr::Or(vec![(false, v(a)), (false, v(b)), (true, v(c))]),
            ),
        }
    }
    // Independently authored snippets; neither Expr serialization nor oracle-derived inputs.
    fn sources(self, [a, b, c, d, e]: [i64; 5]) -> (String, String) {
        match self {
            Self::ContextUnionGap => (
                format!("(((*{a}|{b})&(*{b}|{a}))|{a})"),
                format!("(*{a}|{b})"),
            ),
            Self::MarkedAncestorGap => (
                format!("(*((*{a}|{b})|{c})|(*{d}|{e}))"),
                format!("({a}|{d})"),
            ),
            Self::Transparent => (
                format!("((*{c}|{a})|{c})"),
                format!("((*{a}|{b})|(*{c}|{b}))"),
            ),
            Self::OldMarked => (format!("((*{a}|{b})|*{c})"), format!("(*{b}|{a})")),
            Self::AsymValues => (format!("((*{a}|{b})|(*{b}|{a}))"), format!("(*{a}|{b})")),
            Self::AsymBottom => (
                format!("(((*{a}|{b})&(*{a}|{c}))|{c})"),
                format!("(*{c}|{a})"),
            ),
            Self::Agreement => (format!("(*{b}|{c})"), format!("((*{a}|{b})|{c})")),
            Self::Flat => (format!("(*{a}|{b})"), format!("(*{b}|{a})")),
            Self::Dead => (format!("(*{a}|{b})"), format!("({a}|{b}|*{c})")),
        }
    }
    // Handwritten consequences of the candidate, NOT claims of CUE compatibility.
    fn expected(self, [a, b, c, d, _]: [i64; 5], old: bool) -> (Vec<i64>, Preference, Outcome) {
        match self {
            Self::ContextUnionGap if old => (vec![a, b], Preference::Bottom, Outcome::Incomplete),
            Self::ContextUnionGap | Self::AsymValues | Self::Dead => {
                (vec![a, b], pref(&[a]), Outcome::Concrete(a))
            }
            Self::MarkedAncestorGap if old => (vec![a, d], pref(&[a]), Outcome::Concrete(a)),
            Self::MarkedAncestorGap => (vec![a, d], pref(&[a, d]), Outcome::Incomplete),
            Self::Transparent if old => (vec![a, c], pref(&[c]), Outcome::Concrete(c)),
            Self::Transparent => (vec![a, c], pref(&[a, c]), Outcome::Incomplete),
            Self::OldMarked if old => (vec![a, b], pref(&[b]), Outcome::Concrete(b)),
            Self::OldMarked | Self::Flat => (vec![a, b], Preference::Bottom, Outcome::Incomplete),
            Self::AsymBottom if old => (vec![a, c], Preference::Bottom, Outcome::Incomplete),
            Self::AsymBottom => (vec![a, c], pref(&[a, c]), Outcome::Incomplete),
            Self::Agreement => (vec![b, c], pref(&[b]), Outcome::Concrete(b)),
        }
    }
    fn cue(self, [a, b, c, _, _]: [i64; 5]) -> [Oracle; 2] {
        match self {
            Self::ContextUnionGap | Self::Transparent | Self::OldMarked | Self::Flat => {
                [Oracle::Incomplete; 2]
            }
            Self::MarkedAncestorGap | Self::Dead => [Oracle::Value(a); 2],
            Self::AsymValues => [Oracle::Value(a), Oracle::Incomplete],
            Self::AsymBottom => [Oracle::Value(c), Oracle::Incomplete],
            Self::Agreement => [Oracle::Value(b); 2],
        }
    }
}

#[test]
fn independent_matrix_keeps_new_falsifications_old_results_and_asymmetries_separate() {
    // Toolchain source pin is provided by Nix, not inferred from display text.
    let version = Command::new("cue")
        .arg("version")
        .output()
        .expect("mandatory CUE");
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("v0.16.1"));
    for names in [[1, 2, 3, 4, 5], [-9, -2, 0, 5, 12], [5, 4, 3, 2, 1]] {
        for case in [
            Case::ContextUnionGap,
            Case::MarkedAncestorGap,
            Case::Transparent,
            Case::OldMarked,
            Case::AsymValues,
            Case::AsymBottom,
            Case::Agreement,
            Case::Flat,
            Case::Dead,
        ] {
            let (l, r) = case.operands(names);
            let (ls, rs) = case.sources(names);
            for (order, expr, source) in [
                (0, and(l.clone(), r.clone()), format!("{ls}&{rs}")),
                (1, and(r, l), format!("{rs}&{ls}")),
            ] {
                let new = solve(&expr, 4096).expect("bounded candidate");
                let (cs, ps, out) = case.expected(names, false);
                report(&new, &cs, ps, out);
                let old = Graph::new(expr)
                    .expect("unchanged old construction")
                    .solve(4096)
                    .expect("old hypothesis");
                let (cs, ps, out) = case.expected(names, true);
                assert_eq!(
                    (old.constraints, old.preferred, old.outcome),
                    (ds(&cs), ps, out),
                    "{case:?}/{order}"
                );
                oracle(&source, case.cue(names)[order]);
            }
        }
    }
}

#[test]
fn context_facts_explain_new_gaps_and_motivating_correction_without_repair() {
    for reverse in [false, true] {
        let (l, r) = Case::Transparent.operands([1, 2, 3, 4, 5]);
        let x = if reverse { and(r, l) } else { and(l, r) };
        let keys = if reverse {
            [[(6, 0), (11, 0)], [(6, 1), (11, 0)], [(6, 1), (11, 1)]]
        } else {
            [[(4, 0), (11, 0)], [(4, 0), (11, 1)], [(4, 1), (11, 1)]]
        };
        assert_eq!(
            solve(&x, 4096).expect("transparent contexts").contexts,
            vec![
                fact(&keys[0], &[1], pref(&[1])),
                fact(&keys[1], &[3], pref(&[3])),
                fact(&keys[2], &[3], pref(&[3]))
            ]
        );
        let (l, r) = Case::ContextUnionGap.operands([1, 2, 3, 4, 5]);
        let x = if reverse { and(r, l) } else { and(l, r) };
        let group = if reverse { 11 } else { 8 };
        let result = solve(&x, 4096).expect("union gap");
        assert_eq!(
            result.contexts,
            vec![
                fact(&[(group, 0)], &[1, 2], Preference::Bottom),
                fact(&[(group, 1)], &[1], pref(&[1]))
            ]
        );
        let (l, r) = Case::MarkedAncestorGap.operands([1, 2, 3, 4, 5]);
        let x = if reverse { and(r, l) } else { and(l, r) };
        let group = if reverse { 7 } else { 4 };
        let result = solve(&x, 4096).expect("ancestor gap");
        assert_eq!(
            result.contexts,
            vec![
                fact(&[], &[4], pref(&[4])),
                fact(&[(group, 0)], &[1], pref(&[1]))
            ]
        );
    }
    let (x, _) = Case::MarkedAncestorGap.operands([1, 2, 3, 4, 5]);
    let r = solve(&x, 4096).expect("standalone ancestor gap");
    report(&r, &[1, 2, 3, 4, 5], pref(&[1, 4]), Outcome::Incomplete);
    assert_eq!(
        r.contexts,
        vec![
            fact(&[], &[4, 5], pref(&[4])),
            fact(&[(4, 0)], &[1, 2], pref(&[1])),
            fact(&[(4, 1)], &[3], Preference::Bottom)
        ]
    );
    oracle("(*((*1|2)|3)|(*4|5))", Oracle::Value(1));
    for (source, expected) in [
        ("(*((*1|2)|3)|(*4|5))&(1|2|3|4|5)", Oracle::Value(1)),
        ("(1|2|3|4|5)&(*((*1|2)|3)|(*4|5))", Oracle::Incomplete),
    ] {
        oracle(source, expected);
    }
    for reverse in [false, true] {
        let all = Expr::Or((1..=5).map(|n| (false, v(n))).collect());
        let expr = if reverse {
            and(all, x.clone())
        } else {
            and(x.clone(), all)
        };
        report(
            &solve(&expr, 4096).expect("broad asymmetry"),
            &[1, 2, 3, 4, 5],
            pref(&[1, 4]),
            Outcome::Incomplete,
        );
    }
}

#[test]
fn flat_bottom_lone_mark_and_syntactic_m2_are_not_rewritten() {
    let flat = solve(&and(p(1, 2), p(2, 1)), 4096).expect("flat");
    assert_eq!(flat.contexts, vec![fact(&[], &[1, 2], Preference::Bottom)]);
    let lone = Expr::Or(vec![(false, p(1, 2)), (true, v(3))]);
    let r = solve(&lone, 4096).expect("own marked choices do not partition");
    report(&r, &[1, 2, 3], pref(&[3]), Outcome::Concrete(3));
    assert_eq!(r.contexts, vec![fact(&[], &[1, 2, 3], pref(&[3]))]);
    oracle("((*1|2)|*3)", Oracle::Value(3));
    // Inner mark dies, outer marked value remains live: M2 must not become M1.
    let m2 = and(outer(p(3, 1), v(2)), u(v(1), v(2)));
    let r = solve(&m2, 4096).expect("syntactic M2");
    report(&r, &[1, 2], Preference::Bottom, Outcome::Incomplete);
    assert_eq!(r.contexts, vec![fact(&[], &[1, 2], Preference::Bottom)]);
    // Dead outer mark inherits the other child's recipe, rather than inventing a value default.
    let inherited = and(
        Expr::Or(vec![(false, p(1, 2)), (true, v(3))]),
        u(v(1), v(2)),
    );
    report(
        &solve(&inherited, 4096).expect("dead outer mark"),
        &[1, 2],
        pref(&[1]),
        Outcome::Concrete(1),
    );
    let absent = and(outer(v(3), v(1)), u(v(1), v(2)));
    report(
        &solve(&absent, 4096).expect("no child recipe"),
        &[1],
        Preference::Absent,
        Outcome::Concrete(1),
    );
    // Exact marked-ancestor syntax: CUE is asymmetric, candidate Bottom in both orders.
    oracle("(*(*3|1)|2)&(1|2)", Oracle::Incomplete);
    oracle("(1|2)&(*(*3|1)|2)", Oracle::Value(1));
    let reverse_m2 = and(u(v(1), v(2)), outer(p(3, 1), v(2)));
    report(
        &solve(&reverse_m2, 4096).expect("reverse M2"),
        &[1, 2],
        Preference::Bottom,
        Outcome::Incomplete,
    );
    // Conditional inheritance is a third genuine both-order gap, not a repair.
    oracle("((*1|2)|*3)&(1|2)", Oracle::Incomplete);
    oracle("(1|2)&((*1|2)|*3)", Oracle::Incomplete);
    let reverse_inherited = and(
        u(v(1), v(2)),
        Expr::Or(vec![(false, p(1, 2)), (true, v(3))]),
    );
    report(
        &solve(&reverse_inherited, 4096).expect("reverse dead mark"),
        &[1, 2],
        pref(&[1]),
        Outcome::Concrete(1),
    );
}

#[test]
fn equal_valued_occurrences_and_refinements_keep_exact_contexts() {
    let x = u(p(1, 2), v(1));
    let original = solve(&x, 4096).expect("equal scalar siblings");
    report(&original, &[1, 2], pref(&[1]), Outcome::Concrete(1));
    assert_eq!(
        original.contexts,
        vec![
            fact(&[(4, 0)], &[1, 2], pref(&[1])),
            fact(&[(4, 1)], &[1], Preference::Absent)
        ]
    );
    for (n, expected) in [
        (1, Outcome::Concrete(1)),
        (2, Outcome::Concrete(2)),
        (3, Outcome::Conflict),
    ] {
        let r = solve(&and(x.clone(), v(n)), 4096).expect("source conjunction refinement");
        assert_eq!(r.outcome, expected);
        assert_eq!(r.constraints, if n == 3 { vec![] } else { ds(&[n]) });
        let facts = match n {
            1 => vec![
                fact(&[(4, 0)], &[1], pref(&[1])),
                fact(&[(4, 1)], &[1], Preference::Absent),
            ],
            2 => vec![fact(&[(4, 0)], &[2], Preference::Absent)],
            _ => vec![],
        };
        assert_eq!(r.contexts, facts);
        assert_eq!(solve(&x, 4096), Ok(solve(&x, 4096).expect("fresh state")));
    }
    assert_eq!(solve(&x, 4096), Ok(original));
    oracle("((*1|2)|1)&3", Oracle::Conflict);
}

#[test]
fn source_limits_raw_products_and_shared_work_fail_without_poisoning() {
    let (l, r) = Case::Transparent.operands([1, 2, 3, 4, 5]);
    let good = and(l, r);
    let baseline = solve(&good, 4096).expect("baseline");
    assert_eq!(
        solve(&good, baseline.work),
        Ok(solve(&good, 4096).expect("exact work"))
    );
    for limit in [0, baseline.work - 1, 4097] {
        assert!(matches!(solve(&good, limit), Err(Error::Limit(_))));
        assert_eq!(
            solve(&good, 4096),
            Ok(solve(&good, 4096).expect("recovery"))
        );
    }
    for bad in [
        Expr::Top,
        Expr::Int,
        Expr::Or(vec![]),
        Expr::Or(vec![(false, v(1))]),
        u(v(1), and(v(2), Expr::Top)),
    ] {
        assert!(matches!(solve(&bad, 4096), Err(Error::Unsupported(_))));
        assert!(matches!(solve(&bad, 0), Err(Error::Unsupported(_))));
        assert!(matches!(solve(&bad, 4097), Err(Error::Limit(_))));
    }
    let mut depth = v(1);
    for _ in 0..16 {
        depth = and(depth, v(1));
    }
    let wide = Expr::Or((0..17).map(|n| (false, v(n))).collect());
    let many_nodes = and(
        Expr::Or(
            (0..8)
                .map(|_| (false, Expr::Or((0..16).map(|_| (false, v(1))).collect())))
                .collect(),
        ),
        v(1),
    );
    // Nine by eight ordinary worlds exceeds 64 BEFORE incompatible products are removed.
    let raw = and(
        Expr::Or((0..9).map(|n| (false, v(n))).collect()),
        Expr::Or((20..28).map(|n| (false, v(n))).collect()),
    );
    let world_overflow = Expr::Or(
        (0..9)
            .map(|_| (false, Expr::Or((0..8).map(|_| (false, v(1))).collect())))
            .collect(),
    );
    let groups = Expr::Or((0..16).map(|_| (false, p(1, 2))).collect());
    assert_eq!(
        solve(&raw, 4096),
        Err(Error::Limit("world table or raw product"))
    );
    for bad in [depth, wide, many_nodes, world_overflow, groups] {
        assert!(matches!(solve(&bad, 4096), Err(Error::Limit(_))));
        assert_eq!(
            solve(&good, 4096),
            Ok(solve(&good, 4096).expect("structural recovery"))
        );
    }
    // Six independent two-way wrappers yield 64 compatible worlds with distinct
    // six-pair signatures. They exhaust shared work, not a per-context allowance.
    let wrapper = || u(and(p(1, 2), v(1)), v(1));
    let mut crowded = wrapper();
    for _ in 1..6 {
        crowded = and(crowded, wrapper());
    }
    assert_eq!(solve(&crowded, 4096), Err(Error::Limit("context work")));
    assert_eq!(solve(&good, 4096), Ok(baseline));
    let at_cap = and(
        Expr::Or((0..8).map(|_| (false, v(1))).collect()),
        Expr::Or((0..8).map(|_| (false, v(1))).collect()),
    );
    let r = solve(&at_cap, 4096).expect("64 raw worlds admitted");
    report(&r, &[1], Preference::Absent, Outcome::Concrete(1));
    assert_eq!(r.contexts, vec![fact(&[], &[1], Preference::Absent)]);
}

#[test]
fn reference_apis_remain_conservative_and_keep_their_own_work_thresholds() {
    let root =
        PathBuf::from(std::env::var_os("CUESTACEAN_DEMAND_POLICIES").expect("native policies"));
    let read = |name: &str| {
        fs::read_to_string(root.join(format!("{name}.json"))).expect("policy artifact")
    };
    let dispatch = Policy::from_json(&read("demand-policy")).expect("dispatch scope");
    let projection =
        Policy::from_projection_json(&read("preference-projection")).expect("projection scope");
    let early = Policy::from_json(&read("demand-policy-defer")).expect("early scope");
    let late =
        Policy::from_projection_json(&read("preference-projection-defer")).expect("late scope");
    let (l, r) = Case::Transparent.operands([1, 2, 3, 4, 5]);
    let graph = Graph::new(and(l, r)).expect("old graph");
    let old = graph.solve(4096).expect("old result");
    assert_eq!(old.work, 161);
    assert_eq!(old.preferred, pref(&[3]));
    assert_eq!(old.outcome, Outcome::Concrete(3));
    assert_eq!(
        graph.solve(old.work),
        Ok(graph.solve(4096).expect("old exact work"))
    );
    assert!(matches!(graph.solve(old.work - 1), Err(Error::Limit(_))));
    for kind in 0..3 {
        let run = |domain, limit| match kind {
            0 => graph.demand(domain, limit),
            1 => graph.demand_with_policy(domain, limit, &dispatch),
            _ => graph.demand_with_policies(domain, limit, &dispatch, &projection),
        };
        let original = run(Domain::Top, 4096).expect("old deferred demand");
        assert_eq!(original.constraints, ds(&[1, 3]));
        assert_eq!(original.preferred, None);
        assert_eq!(original.demand, Demand::Suspended);
        assert_eq!(
            original.readiness,
            if kind == 0 {
                Readiness::DeferredNested
            } else {
                Readiness::PolicyDeferred
            }
        );
        assert_eq!(
            run(Domain::Top, original.work),
            Ok(run(Domain::Top, 4096).expect("exact demand work"))
        );
        for limit in [0, original.work - 1, 4097] {
            assert!(matches!(run(Domain::Top, limit), Err(Error::Limit(_))));
        }
        for n in [1, 2, 3] {
            let r = run(Domain::Value(n), 4096).expect("old refinement");
            assert_eq!(
                r.demand,
                if n == 2 {
                    Demand::Conflict
                } else {
                    Demand::Concrete(n)
                }
            );
            assert_eq!(r.preferred, None);
        }
        assert_eq!(
            run(Domain::Int, 4096),
            Ok(run(Domain::Top, 4096).expect("int reference"))
        );
        assert_eq!(run(Domain::Top, 4096), Ok(original));
    }
    let flat = Policy::from_json(r#"{"formatVersion":1,"body":{"op":"return","action":"flat"}}"#)
        .expect("unsafe flat policy");
    assert!(matches!(
        graph.demand_with_policy(Domain::Top, 4096, &flat),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        graph.demand_with_policies(Domain::Top, 4096, &flat, &projection),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(
        graph
            .demand_with_policies(Domain::Top, 4096, &early, &projection)
            .expect("early defer")
            .readiness,
        Readiness::PolicyDeferred
    );
    let late = Graph::new(p(1, 2))
        .expect("old flat")
        .demand_with_policies(Domain::Top, 4096, &dispatch, &late)
        .expect("late defer");
    assert_eq!(late.readiness, Readiness::ProjectionDeferred);
    assert_eq!(late.demand, Demand::Suspended);
}
