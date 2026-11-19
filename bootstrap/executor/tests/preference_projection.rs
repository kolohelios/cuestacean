#[path = "../../spikes/default_provenance.rs"]
mod spike;

use serde_json::{Value, json};
use spike::{
    Demand, DemandReport, Domain, Error, Expr, Graph, Preference, Readiness, policy::Policy,
};
use std::{fs, path::PathBuf, process::Command};

fn ret(action: &str) -> Value {
    json!({"op":"return","action":action})
}
fn branch(field: &str, yes: Value, no: Value) -> Value {
    json!({"op":"if","field":field,"then":yes,"else":no})
}
fn wire(body: Value) -> String {
    json!({"formatVersion":1,"body":body}).to_string()
}
fn dispatch(action: &str) -> Policy {
    Policy::from_json(&wire(ret(action))).expect("dispatch scope")
}
fn projection(action: &str) -> Policy {
    Policy::from_projection_json(&wire(ret(action))).expect("projection scope")
}
fn native(name: &str) -> Policy {
    let directory = std::env::var_os("CUESTACEAN_DEMAND_POLICIES")
        .expect("mandatory native policy artifact directory");
    let text = fs::read_to_string(PathBuf::from(directory).join(format!("{name}.json")))
        .expect("mandatory native policy artifact");
    if name.starts_with("preference-projection") {
        Policy::from_projection_json(&text).expect("native projection admission")
    } else {
        Policy::from_json(&text).expect("native dispatch admission")
    }
}
fn and(a: Expr, b: Expr) -> Expr {
    Expr::And(Box::new(a), Box::new(b))
}
fn choice(a: i64, b: i64) -> Expr {
    Expr::Or(vec![(true, Expr::Value(a)), (false, Expr::Value(b))])
}
fn union(a: Expr, b: Expr) -> Expr {
    Expr::Or(vec![(false, a), (false, b)])
}
fn graph(expr: Expr) -> Graph {
    Graph::new(expr).expect("bounded graph")
}
fn values(ns: &[i64]) -> Vec<Domain> {
    ns.iter().copied().map(Domain::Value).collect()
}
fn run(g: &Graph, refinement: Domain, d: &Policy, p: &Policy) -> DemandReport {
    let result = g
        .demand_with_policies(refinement, 4096, d, p)
        .expect("two phases");
    assert_eq!(
        g.demand_with_policies(refinement, result.work, d, p),
        Ok(result_for(g, refinement, d, p))
    );
    assert!(matches!(
        g.demand_with_policies(refinement, result.work - 1, d, p),
        Err(Error::Limit(_))
    ));
    assert_eq!(result_for(g, refinement, d, p), result);
    result
}
fn result_for(g: &Graph, refinement: Domain, d: &Policy, p: &Policy) -> DemandReport {
    g.demand_with_policies(refinement, 4096, d, p)
        .expect("fresh full allowance")
}

#[test]
fn native_canonical_projection_has_independent_domains_and_reference_parity() {
    let d = native("demand-policy");
    let p = native("preference-projection");
    for (expr, constraints, preferred, demand, readiness) in [
        (
            Expr::Top,
            vec![Domain::Top],
            None,
            Demand::Incomplete,
            Readiness::ConstraintOnly,
        ),
        (
            Expr::Int,
            vec![Domain::Int],
            None,
            Demand::Incomplete,
            Readiness::ConstraintOnly,
        ),
        (
            and(Expr::Value(1), Expr::Value(2)),
            vec![],
            None,
            Demand::Conflict,
            Readiness::ConstraintOnly,
        ),
        (
            choice(1, 2),
            values(&[1, 2]),
            Some(Preference::Values(values(&[1]))),
            Demand::Concrete(1),
            Readiness::FlatPreference,
        ),
        (
            and(choice(1, 2), choice(2, 1)),
            values(&[1, 2]),
            Some(Preference::Bottom),
            Demand::Incomplete,
            Readiness::FlatPreference,
        ),
        (
            and(choice(3, 1), union(Expr::Value(1), Expr::Value(2))),
            values(&[1]),
            None,
            Demand::Concrete(1),
            Readiness::ConstraintOnly,
        ),
        (
            and(
                Expr::Or(vec![
                    (true, Expr::Value(3)),
                    (false, Expr::Value(1)),
                    (false, Expr::Value(2)),
                ]),
                union(Expr::Value(1), Expr::Value(2)),
            ),
            values(&[1, 2]),
            Some(Preference::Absent),
            Demand::Incomplete,
            Readiness::FlatPreference,
        ),
    ] {
        let g = graph(expr);
        let report = run(&g, Domain::Top, &d, &p);
        assert_eq!(report.constraints, constraints);
        assert_eq!(report.preferred, preferred);
        assert_eq!(report.demand, demand);
        assert_eq!(report.readiness, readiness);
        for reference in [
            g.demand(Domain::Top, 4096),
            g.demand_with_policy(Domain::Top, 4096, &d),
        ] {
            let reference = reference.expect("retained reference API");
            assert_eq!(report.constraints, reference.constraints);
            assert_eq!(report.groups, reference.groups);
            assert_eq!(report.preferred, reference.preferred);
            assert_eq!(report.demand, reference.demand);
            assert_eq!(report.readiness, reference.readiness);
        }
        // The old hypothesis is exercised independently, not used to seed answers.
        assert_eq!(g.solve(4096).expect("old solve").constraints, constraints);
    }
}

#[test]
fn native_alternate_defers_after_flat_observation_not_before_it() {
    let d = native("demand-policy");
    let canonical = native("preference-projection");
    let alternate = native("preference-projection-defer");
    let early = native("demand-policy-defer");
    let g = graph(choice(1, 2));
    let ready = run(&g, Domain::Top, &d, &canonical);
    let late = run(&g, Domain::Top, &d, &alternate);
    assert_eq!(late.readiness, Readiness::ProjectionDeferred);
    assert_eq!(late.demand, Demand::Suspended);
    assert_eq!(late.preferred, Some(Preference::Values(values(&[1]))));
    assert_eq!(late.constraints, values(&[1, 2]));
    assert_eq!(late.constraints, ready.constraints);
    assert_eq!(late.groups, ready.groups);
    assert_eq!(
        late.groups,
        vec![spike::GroupFact {
            group: 2,
            support: spike::Support::MarkedLive,
            retained: vec![],
            suppressed: vec![],
        }]
    );
    let deferred = run(&g, Domain::Top, &early, &alternate);
    assert_eq!(deferred.readiness, Readiness::PolicyDeferred);
    assert_eq!(deferred.demand, Demand::Suspended);
    assert_eq!(deferred.preferred, None);
    assert_eq!(deferred.groups, late.groups);
    // A valid but unsafe projection would fail if executed on either early path.
    let unsafe_projection = projection("ordinary");
    for refinement in [Domain::Value(1), Domain::Value(9)] {
        let result = run(&g, refinement, &d, &unsafe_projection);
        assert_eq!(result.readiness, Readiness::ConstraintOnly);
        assert_eq!(result.preferred, None);
    }
    assert_eq!(
        run(&g, Domain::Top, &early, &unsafe_projection),
        run(&g, Domain::Top, &early, &alternate)
    );
}

#[test]
fn abstract_preferences_never_fall_back_to_concrete_ordinary_candidates() {
    let d = native("demand-policy");
    let p = native("preference-projection");
    for (atom, domain) in [(Expr::Top, Domain::Top), (Expr::Int, Domain::Int)] {
        let g = graph(Expr::Or(vec![(true, atom), (false, Expr::Value(1))]));
        let report = run(&g, Domain::Top, &d, &p);
        assert_eq!(report.constraints, vec![domain, Domain::Value(1)]);
        assert_eq!(report.preferred, Some(Preference::Values(vec![domain])));
        assert_eq!(report.demand, Demand::Incomplete);
        assert_eq!(report.readiness, Readiness::FlatPreference);
        assert!(matches!(
            g.demand_with_policies(Domain::Top, 4096, &d, &projection("ordinary")),
            Err(Error::Unsupported(_))
        ));
        for n in [i64::MIN, 1, i64::MAX] {
            let exact = run(&g, Domain::Value(n), &d, &p);
            assert_eq!(exact.constraints, values(&[n]));
            assert_eq!(exact.demand, Demand::Concrete(n));
            assert_eq!(exact.preferred, None);
        }
        assert_eq!(run(&g, Domain::Top, &d, &p), report);
    }
}

#[test]
fn scoped_admission_checks_dead_arms_and_all_arguments_before_work() {
    let d = dispatch("defer");
    let p = projection("defer");
    let g = graph(Expr::Value(1));
    for limit in [0, 4097] {
        for (first, second) in [(&p, &p), (&d, &d), (&p, &d)] {
            assert!(matches!(
                g.demand_with_policies(Domain::Top, limit, first, second),
                Err(Error::Unsupported(_))
            ));
        }
        assert!(matches!(
            g.demand_with_policy(Domain::Top, limit, &p),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(
            g.demand_with_policies(Domain::Top, limit, &d, &p),
            Err(Error::Limit(_))
        ));
    }
    for field in ["empty", "forced", "recipe", "nested"] {
        assert!(matches!(
            Policy::from_projection_json(&wire(branch(field, ret("defer"), ret("defer")))),
            Err(Error::Unsupported(_))
        ));
    }
    for field in ["absent", "bottom"] {
        assert!(matches!(
            Policy::from_json(&wire(branch(field, ret("defer"), ret("defer")))),
            Err(Error::Unsupported(_))
        ));
    }
    for action in ["constraints", "flat"] {
        assert!(matches!(
            Policy::from_projection_json(&wire(branch("absent", ret("defer"), ret(action)))),
            Err(Error::Unsupported(_))
        ));
    }
    for action in ["ordinary", "preferred"] {
        assert!(matches!(
            Policy::from_json(&wire(branch("empty", ret("defer"), ret(action)))),
            Err(Error::Unsupported(_))
        ));
    }
    for body in [
        branch(
            "absent",
            ret("defer"),
            branch("empty", ret("defer"), ret("defer")),
        ),
        branch("absent", ret("defer"), ret("unknown")),
        branch("unknown", ret("defer"), ret("defer")),
    ] {
        assert!(matches!(
            Policy::from_projection_json(&wire(body)),
            Err(Error::Unsupported(_))
        ));
    }
    for text in [
        "{}",
        "{",
        "{\"formatVersion\":2,\"body\":{\"op\":\"return\",\"action\":\"defer\"}}",
        "{\"formatVersion\":1,\"formatVersion\":1,\"body\":{\"op\":\"return\",\"action\":\"defer\"}}",
        "{\"formatVersion\":1,\"body\":{\"op\":\"return\",\"action\":\"defer\",\"action\":\"defer\"}}",
        "{\"formatVersion\":1,\"body\":{\"op\":\"return\",\"action\":\"defer\",\"extra\":0}}",
    ] {
        assert!(matches!(
            Policy::from_projection_json(text),
            Err(Error::Unsupported(_))
        ));
    }
    assert!(matches!(
        Policy::from_projection_json(&(wire(ret("defer")) + " false")),
        Err(Error::Unsupported(_))
    ));
}

fn balanced(leaves: usize) -> Value {
    if leaves == 1 {
        ret("defer")
    } else {
        branch(
            "absent",
            balanced(leaves / 2),
            balanced(leaves - leaves / 2),
        )
    }
}
#[test]
fn projection_admission_retains_inclusive_text_node_and_depth_caps() {
    let text = wire(ret("defer"));
    let exact = format!("{}{}", " ".repeat(16384 - text.len()), text);
    assert!(Policy::from_projection_json(&exact).is_ok());
    assert!(matches!(
        Policy::from_projection_json(&(exact + " ")),
        Err(Error::Limit(_))
    ));
    assert!(Policy::from_projection_json(&wire(balanced(32))).is_ok());
    assert!(matches!(
        Policy::from_projection_json(&wire(balanced(33))),
        Err(Error::Limit(_))
    ));
    // Count original UTF-8 bytes even when the admitted size contains invalid data.
    let invalid = wire(ret("défer"));
    let exact_invalid = format!("{}{}", " ".repeat(16384 - invalid.len()), invalid);
    assert!(matches!(
        Policy::from_projection_json(&exact_invalid),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        Policy::from_projection_json(&(exact_invalid + " ")),
        Err(Error::Limit(_))
    ));
    let mut body = ret("defer");
    for _ in 1..16 {
        body = branch("bottom", ret("defer"), body);
    }
    assert!(Policy::from_projection_json(&wire(body.clone())).is_ok());
    assert!(matches!(
        Policy::from_projection_json(&wire(branch("absent", ret("defer"), body))),
        Err(Error::Limit(_))
    ));
}

#[test]
fn certificates_precede_preference_materialization_and_extraction() {
    let flat = dispatch("flat");
    let ordinary = projection("ordinary");
    let preferred = projection("preferred");
    let defer = projection("defer");
    // Absent/bottom require no preferred vector allocation. Safe ordinary's last
    // two charges are scalar extraction and report; the certificate is before both.
    for (expr, observed, demand) in [
        (Expr::Value(1), Preference::Absent, Demand::Concrete(1)),
        (
            and(Expr::Value(1), Expr::Value(2)),
            Preference::Absent,
            Demand::Conflict,
        ),
        (
            and(choice(1, 2), choice(2, 1)),
            Preference::Bottom,
            Demand::Incomplete,
        ),
    ] {
        let g = graph(expr);
        let safe = run(&g, Domain::Top, &flat, &ordinary);
        assert_eq!(safe.preferred, Some(observed));
        assert_eq!(safe.demand, demand);
        let certificate = safe.work - 2;
        assert!(matches!(
            g.demand_with_policies(Domain::Top, certificate - 1, &flat, &preferred),
            Err(Error::Limit(_))
        ));
        assert!(matches!(
            g.demand_with_policies(Domain::Top, certificate, &flat, &preferred),
            Err(Error::Unsupported(_))
        ));
        let suspended = run(&g, Domain::Top, &flat, &defer);
        assert_eq!(suspended.work + 1, safe.work, "defer skips only extraction");
        assert_eq!(suspended.preferred, safe.preferred);
        assert_eq!(suspended.demand, Demand::Suspended);
        assert_eq!(suspended.readiness, Readiness::ProjectionDeferred);
    }
    let g = graph(choice(1, 2));
    let safe = run(&g, Domain::Top, &flat, &preferred);
    let suspended = run(&g, Domain::Top, &flat, &defer);
    assert_eq!(safe.work, suspended.work + 1);
    assert_eq!(safe.preferred, suspended.preferred);
    // This graph has two ordinary worlds and selects only the first. project()
    // charges two scans plus one insertion, with no comparison/shift: three.
    // Both policies are single returns, so subtract materialization, extraction
    // and final report to locate the unsafe ordinary certificate exactly.
    let certificate = safe.work - 3 - 1 - 1;
    assert!(matches!(
        g.demand_with_policies(Domain::Top, certificate - 1, &flat, &ordinary),
        Err(Error::Limit(_))
    ));
    assert!(matches!(
        g.demand_with_policies(Domain::Top, certificate, &flat, &ordinary),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        g.demand_with_policies(Domain::Top, 4096, &flat, &ordinary),
        Err(Error::Unsupported(_))
    ));
    // Adding one executed branch reads exactly one fact and visits one extra
    // instruction. Unvisited, structurally validated branches cost no work.
    let branching =
        Policy::from_projection_json(&wire(branch("absent", ret("preferred"), ret("preferred"))))
            .expect("valid tree");
    assert_eq!(run(&g, Domain::Top, &flat, &branching).work, safe.work + 2);
}

#[test]
fn nested_flat_rejects_before_projection_even_when_projection_would_defer() {
    let flat = dispatch("flat");
    let p = projection("defer");
    let canonical = native("demand-policy");
    for reverse in [false, true] {
        let left = Expr::Or(vec![(false, choice(1, 2)), (true, Expr::Value(3))]);
        let right = choice(2, 1);
        let g = graph(if reverse {
            and(right, left)
        } else {
            and(left, right)
        });
        assert!(matches!(
            g.demand_with_policies(Domain::Top, 4096, &flat, &p),
            Err(Error::Unsupported(_))
        ));
        // A one-return dispatch defer reaches the same dispatch certificate,
        // then only writes the report. This also pins nested rejection when
        // ordinary refinement forces a scalar: flat still is not certified.
        for refinement in [Domain::Top, Domain::Value(1)] {
            let early = run(&g, refinement, &dispatch("defer"), &p);
            let certificate = early.work - 1;
            assert!(matches!(
                g.demand_with_policies(refinement, certificate - 1, &flat, &p),
                Err(Error::Limit(_))
            ));
            assert!(matches!(
                g.demand_with_policies(refinement, certificate, &flat, &p),
                Err(Error::Unsupported(_))
            ));
        }
        let report = run(&g, Domain::Top, &canonical, &p);
        assert_eq!(report.constraints, values(&[1, 2]));
        assert_eq!(report.readiness, Readiness::PolicyDeferred);
        assert_eq!(report.preferred, None);
        assert_eq!(report.demand, Demand::Suspended);
        let inner = if reverse { 5 } else { 2 };
        let outer = if reverse { 7 } else { 4 };
        let sibling = if reverse { 2 } else { 7 };
        let mut expected = vec![
            spike::GroupFact {
                group: inner,
                support: spike::Support::MarkedLive,
                retained: vec![],
                suppressed: vec![],
            },
            spike::GroupFact {
                group: outer,
                support: spike::Support::MarkedEliminated,
                retained: vec![],
                suppressed: vec![inner],
            },
            spike::GroupFact {
                group: sibling,
                support: spike::Support::MarkedLive,
                retained: vec![],
                suppressed: vec![],
            },
        ];
        expected.sort_by_key(|fact| fact.group);
        assert_eq!(report.groups, expected);
        // Keep the falsified static hypothesis visible, not a fallback.
        assert_eq!(
            g.solve(4096).expect("retained hypothesis").outcome,
            spike::Outcome::Concrete(2)
        );
        for n in [1, 2] {
            assert_eq!(
                run(&g, Domain::Value(n), &canonical, &p).demand,
                Demand::Concrete(n)
            );
        }
        assert_eq!(run(&g, Domain::Top, &canonical, &p), report);
    }
}

#[test]
fn both_operand_orders_preserve_flat_support_and_native_semantic_oracle() {
    let d = native("demand-policy");
    let p = native("preference-projection");
    for reverse in [false, true] {
        let left = choice(1, 2);
        let right = union(Expr::Value(1), Expr::Value(3));
        let g = graph(if reverse {
            and(right, left)
        } else {
            and(left, right)
        });
        let result = run(&g, Domain::Top, &dispatch("flat"), &p);
        assert_eq!(result.preferred, Some(Preference::Values(values(&[1]))));
        assert_eq!(result.demand, Demand::Concrete(1));
        assert_eq!(run(&g, Domain::Top, &d, &p).demand, Demand::Concrete(1));
    }
    // Fresh pinned CUE processes and explicit raw files, not native JSON inputs.
    let dir = std::env::temp_dir().join(format!(
        "cuestacean-projection-oracle-{}",
        std::process::id()
    ));
    fs::create_dir(&dir).expect("exclusive oracle directory");
    fs::write(dir.join("schema.cue"), "package probe\n").expect("raw schema");
    for (expression, expected) in [
        ("(*1|2)&(1|3)", Some("1")),
        ("(1|3)&(*1|2)", Some("1")),
        ("(*1|2)&(*2|1)", None),
        ("*int|1", None),
    ] {
        fs::write(
            dir.join("program.cue"),
            format!("package probe\nvalue: {expression}\n"),
        )
        .expect("raw program");
        let output = Command::new("cue")
            .arg("export")
            .arg(dir.join("schema.cue"))
            .arg(dir.join("program.cue"))
            .args(["-e", "value"])
            .output()
            .expect("mandatory pinned CUE oracle");
        if let Some(expected) = expected {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8(output.stdout).expect("UTF-8").trim(),
                expected
            );
        } else {
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete"));
        }
    }
    fs::remove_dir_all(dir).expect("remove owned oracle files");
}
