// Test-only dispatch ownership evidence, not a contextual default solver.
#[path = "../../spikes/default_provenance.rs"]
mod default_provenance;

use default_provenance::{
    Demand, DemandReport, Domain, Error, Expr, Graph, GroupFact, Outcome, Preference, Readiness,
    Support, policy::Policy,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};

fn ret(action: &str) -> Value {
    json!({"op":"return", "action":action})
}
fn branch(field: &str, yes: Value, no: Value) -> Value {
    json!({"op":"if", "field":field, "then":yes, "else":no})
}
fn wire(body: Value) -> String {
    json!({"formatVersion":1,"body":body}).to_string()
}
fn decode(body: Value) -> Policy {
    Policy::from_json(&wire(body)).expect("valid handwritten test policy")
}
fn export(program: &str) -> Policy {
    let spikes = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spikes");
    let output = Command::new("cue")
        .arg("export")
        .arg(spikes.join("demand-policy-schema.cue"))
        .arg(spikes.join(program))
        .args(["--out", "json"])
        .output()
        .expect("mandatory pinned CUE policy export");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Policy::from_json(std::str::from_utf8(&output.stdout).expect("UTF-8 policy"))
        .expect("fresh raw CUE policy admission")
}
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
fn graph(expr: Expr) -> Graph {
    Graph::new(expr).expect("bounded independent IR")
}
fn domains(ns: &[i64]) -> Vec<Domain> {
    ns.iter().map(|n| Domain::Value(*n)).collect()
}
fn fact(group: usize, support: Support, retained: &[usize], suppressed: &[usize]) -> GroupFact {
    GroupFact {
        group,
        support,
        retained: retained.to_vec(),
        suppressed: suppressed.to_vec(),
    }
}
fn run(graph: &Graph, refinement: Domain, policy: &Policy) -> DemandReport {
    let report = graph
        .demand_with_policy(refinement, 4096, policy)
        .expect("bounded execution");
    assert!(report.work > 0 && report.work <= 4096);
    assert_eq!(
        graph
            .demand_with_policy(refinement, report.work, policy)
            .expect("exact work allowance"),
        report
    );
    assert!(matches!(
        graph.demand_with_policy(refinement, report.work - 1, policy),
        Err(Error::Limit(_))
    ));
    assert_eq!(
        graph
            .demand_with_policy(refinement, 4096, policy)
            .expect("fresh recovery"),
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
fn parity(graph: &Graph, refinement: Domain, report: &DemandReport) {
    let native = graph
        .demand(refinement, 4096)
        .expect("independent native reference dispatch");
    assert_eq!(report.constraints, native.constraints);
    assert_eq!(report.groups, native.groups);
    assert_eq!(report.preferred, native.preferred);
    assert_eq!(report.demand, native.demand);
    assert_eq!(
        report.readiness,
        if native.readiness == Readiness::DeferredNested {
            Readiness::PolicyDeferred
        } else {
            native.readiness
        }
    );
}

#[test]
fn canonical_raw_cue_dispatch_has_independent_results_and_full_native_parity() {
    let policy = export("demand-policy.cue");
    for (expr, expected, readiness, demand, preferred) in [
        (
            Expr::Top,
            vec![Domain::Top],
            Readiness::ConstraintOnly,
            Demand::Incomplete,
            None,
        ),
        (
            Expr::Int,
            vec![Domain::Int],
            Readiness::ConstraintOnly,
            Demand::Incomplete,
            None,
        ),
        (
            and(value(1), value(2)),
            vec![],
            Readiness::ConstraintOnly,
            Demand::Conflict,
            None,
        ),
        (
            union(value(2), value(2)),
            domains(&[2]),
            Readiness::ConstraintOnly,
            Demand::Concrete(2),
            None,
        ),
        (
            choice(1, 2),
            domains(&[1, 2]),
            Readiness::FlatPreference,
            Demand::Concrete(1),
            Some(Preference::Values(domains(&[1]))),
        ),
        (
            and(choice(1, 2), choice(2, 1)),
            domains(&[1, 2]),
            Readiness::FlatPreference,
            Demand::Incomplete,
            Some(Preference::Bottom),
        ),
        (
            and(
                or(vec![(true, value(3)), (false, value(1)), (false, value(2))]),
                union(value(1), value(2)),
            ),
            domains(&[1, 2]),
            Readiness::FlatPreference,
            Demand::Incomplete,
            Some(Preference::Absent),
        ),
        (
            and(
                choice(1, 2),
                or(vec![(false, value(1)), (false, value(2)), (true, value(3))]),
            ),
            domains(&[1, 2]),
            Readiness::FlatPreference,
            Demand::Concrete(1),
            Some(Preference::Values(domains(&[1]))),
        ),
        (
            union(choice(1, 2), value(3)),
            domains(&[1, 2, 3]),
            Readiness::PolicyDeferred,
            Demand::Suspended,
            None,
        ),
    ] {
        let graph = graph(expr);
        let report = run(&graph, Domain::Top, &policy);
        decision(&report, expected, readiness, demand, preferred);
        parity(&graph, Domain::Top, &report);
    }
    // One concrete candidate plus abstract candidates is never forced.
    for atom in [Expr::Top, Expr::Int] {
        let abstract_domain = if atom == Expr::Top {
            Domain::Top
        } else {
            Domain::Int
        };
        let graph = graph(or(vec![(true, atom), (false, value(1))]));
        decision(
            &run(&graph, Domain::Top, &policy),
            vec![abstract_domain, Domain::Value(1)],
            Readiness::FlatPreference,
            Demand::Incomplete,
            Some(Preference::Values(vec![abstract_domain])),
        );
        for n in [i64::MIN, 1, i64::MAX] {
            let report = run(&graph, Domain::Value(n), &policy);
            decision(
                &report,
                domains(&[n]),
                Readiness::ConstraintOnly,
                Demand::Concrete(n),
                None,
            );
            parity(&graph, Domain::Value(n), &report);
        }
    }
}

#[test]
fn raw_alternate_changes_forced_and_flat_execution_without_erasing_facts() {
    let canonical = export("demand-policy.cue");
    let alternate = export("demand-policy-defer.cue");
    let graph = graph(or(vec![
        (false, choice(1, 2)),
        (true, choice(3, 4)),
        (false, choice(5, 6)),
    ]));
    let broad_facts = vec![
        fact(2, Support::MarkedLive, &[], &[]),
        fact(5, Support::MarkedLive, &[], &[]),
        fact(8, Support::MarkedLive, &[], &[]),
        fact(9, Support::MarkedLive, &[5], &[2, 8]),
    ];
    let broad = run(&graph, Domain::Top, &canonical);
    assert_eq!(broad.groups, broad_facts);
    decision(
        &broad,
        domains(&[1, 2, 3, 4, 5, 6]),
        Readiness::PolicyDeferred,
        Demand::Suspended,
        None,
    );
    for refinement in [Domain::Top, Domain::Int, Domain::Value(1), Domain::Value(9)] {
        let original = run(&graph, refinement, &canonical);
        let deferred = run(&graph, refinement, &alternate);
        assert_eq!(deferred.constraints, original.constraints);
        assert_eq!(deferred.groups, original.groups);
        decision(
            &deferred,
            original.constraints.clone(),
            Readiness::PolicyDeferred,
            Demand::Suspended,
            None,
        );
        parity(&graph, refinement, &original);
        if refinement == Domain::Value(1) {
            decision(
                &original,
                domains(&[1]),
                Readiness::ConstraintOnly,
                Demand::Concrete(1),
                None,
            );
            assert_eq!(
                original.groups,
                vec![
                    fact(2, Support::MarkedLive, &[], &[]),
                    fact(5, Support::Unentered, &[], &[]),
                    fact(8, Support::Unentered, &[], &[]),
                    fact(9, Support::MarkedEliminated, &[5], &[2, 8])
                ]
            );
        }
        if refinement == Domain::Value(9) {
            assert!(
                original
                    .groups
                    .iter()
                    .all(|g| g.support == Support::Unentered)
            );
            assert_eq!(original.groups[3].retained, vec![5]);
            assert_eq!(original.groups[3].suppressed, vec![2, 8]);
        }
    }
    assert_eq!(run(&graph, Domain::Top, &canonical), broad);
    let flat = graph_flat();
    let original = run(&flat, Domain::Top, &canonical);
    decision(
        &original,
        domains(&[1, 2]),
        Readiness::FlatPreference,
        Demand::Concrete(1),
        Some(Preference::Values(domains(&[1]))),
    );
    assert_eq!(
        original.groups,
        vec![fact(2, Support::MarkedLive, &[], &[])]
    );
    let deferred = run(&flat, Domain::Top, &alternate);
    decision(
        &deferred,
        domains(&[1, 2]),
        Readiness::PolicyDeferred,
        Demand::Suspended,
        None,
    );
    assert_eq!(deferred.groups, original.groups);
}
fn graph_flat() -> Graph {
    graph(choice(1, 2))
}

#[test]
fn certificates_reject_unsafe_actions_without_fallback_but_allow_safe_flat_edges() {
    let constraints = decode(ret("constraints"));
    let flat = decode(ret("flat"));
    let defer = decode(ret("defer"));
    for expr in [choice(1, 2), union(choice(1, 2), value(3))] {
        let graph = graph(expr);
        let audited = run(&graph, Domain::Top, &defer);
        let certificate_budget = audited.work - 1;
        assert!(matches!(
            graph.demand_with_policy(Domain::Top, certificate_budget - 1, &constraints),
            Err(Error::Limit(_))
        ));
        assert!(matches!(
            graph.demand_with_policy(Domain::Top, certificate_budget, &constraints),
            Err(Error::Unsupported(_))
        ));
        decision(
            &audited,
            graph
                .demand(Domain::Top, 4096)
                .expect("reference")
                .constraints,
            Readiness::PolicyDeferred,
            Demand::Suspended,
            None,
        );
    }
    // A forced nested graph is still unsafe for flat: forced precedence does not bypass its certificate.
    for refinement in [Domain::Top, Domain::Value(1)] {
        let nested = graph(union(choice(1, 2), value(3)));
        let audited = run(&nested, refinement, &defer);
        // Successful deferral includes one final report charge; rejection stops before it.
        let certificate_budget = audited.work - 1;
        assert!(matches!(
            nested.demand_with_policy(refinement, certificate_budget - 1, &flat),
            Err(Error::Limit(_))
        ));
        assert!(matches!(
            nested.demand_with_policy(refinement, certificate_budget, &flat),
            Err(Error::Unsupported(_))
        ));
    }
    for (expr, expected, demand, preferred) in [
        (
            and(choice(1, 2), value(9)),
            vec![],
            Demand::Conflict,
            Preference::Absent,
        ),
        (
            and(choice(1, 2), value(1)),
            domains(&[1]),
            Demand::Concrete(1),
            Preference::Values(domains(&[1])),
        ),
    ] {
        let graph = graph(expr);
        let report = run(&graph, Domain::Top, &flat);
        decision(
            &report,
            expected,
            Readiness::FlatPreference,
            demand,
            Some(preferred),
        );
        assert_eq!(
            report.groups,
            graph
                .demand(Domain::Top, 4096)
                .expect("native facts")
                .groups
        );
    }
}

#[test]
fn counterexample_and_coverage_cost_remain_evidence_in_both_operand_orders() {
    let policy = export("demand-policy.cue");
    for (a, b, ordinary, old_value) in [
        (
            or(vec![(false, choice(1, 2)), (true, value(3))]),
            choice(2, 1),
            domains(&[1, 2]),
            2,
        ),
        (
            choice(2, 3),
            union(choice(1, 2), value(3)),
            domains(&[2, 3]),
            2,
        ),
    ] {
        for expr in [and(a.clone(), b.clone()), and(b, a)] {
            let graph = graph(expr);
            let old = graph.solve(4096).expect("unchanged diagnostic hypothesis");
            assert!(old.work > 0 && old.work <= 4096);
            assert_eq!(old.constraints, ordinary);
            assert_eq!(old.preferred, Preference::Values(domains(&[old_value])));
            assert_eq!(old.outcome, Outcome::Concrete(old_value));
            let report = run(&graph, Domain::Top, &policy);
            decision(
                &report,
                ordinary.clone(),
                Readiness::PolicyDeferred,
                Demand::Suspended,
                None,
            );
            parity(&graph, Domain::Top, &report);
            for refinement in [
                Domain::Int,
                Domain::Value(1),
                Domain::Value(2),
                Domain::Value(3),
            ] {
                parity(&graph, refinement, &run(&graph, refinement, &policy));
            }
        }
    }
    for (left, right, concrete) in [
        ("((*1|2)|*3)", "(*2|1)", None),
        ("(*2|3)", "((*1|2)|3)", Some(2)),
    ] {
        oracle(&format!("{left}&{right}"), concrete);
        oracle(&format!("{right}&{left}"), concrete);
    }
    oracle("(*1|2)&(1|2|*3)", Some(1));
    oracle("(*1|2)&(1|*2)", None);
}

#[test]
fn bounded_permutations_keep_flat_nested_and_forced_gates_in_both_orders() {
    let policy = export("demand-policy.cue");
    let terms = [(true, value(1)), (false, value(2)), (false, value(3))];
    for p in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let flat = or(p.map(|i| terms[i].clone()).to_vec());
        for reversed in [false, true] {
            let combine = |a, b| if reversed { and(b, a) } else { and(a, b) };
            for (expr, ordinary, ready, demand, preferred) in [
                (
                    combine(flat.clone(), Expr::Int),
                    domains(&[1, 2, 3]),
                    Readiness::FlatPreference,
                    Demand::Concrete(1),
                    Some(Preference::Values(domains(&[1]))),
                ),
                (
                    combine(union(flat.clone(), value(4)), Expr::Int),
                    domains(&[1, 2, 3, 4]),
                    Readiness::PolicyDeferred,
                    Demand::Suspended,
                    None,
                ),
                (
                    combine(union(flat.clone(), value(4)), value(2)),
                    domains(&[2]),
                    Readiness::ConstraintOnly,
                    Demand::Concrete(2),
                    None,
                ),
            ] {
                let graph = graph(expr);
                let report = run(&graph, Domain::Top, &policy);
                decision(&report, ordinary, ready, demand, preferred);
                parity(&graph, Domain::Top, &report);
            }
        }
    }
}

// Fresh explicit source files for every mandatory semantic oracle. Pinned Nix
// attests CUE v0.16.1 source 6d609d768f1686f9a3a2a20197cacdbb70e5c79d;
// its `(devel)` display string alone is not source provenance.
fn oracle(expression: &str, concrete: Option<i64>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "cuestacean-policy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).expect("exclusive probe directory");
    fs::write(dir.join("schema.cue"), "package probe\n").expect("schema");
    fs::write(
        dir.join("program.cue"),
        format!("package probe\nvalue: {expression}\n"),
    )
    .expect("program");
    let invoke = |op: &str| {
        Command::new("cue")
            .arg(op)
            .arg(dir.join("schema.cue"))
            .arg(dir.join("program.cue"))
            .args(["-e", "value"])
            .output()
            .expect("mandatory fresh pinned CUE")
    };
    let output = invoke("export");
    if let Some(n) = concrete {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout)
                .expect("scalar UTF-8")
                .trim(),
            n.to_string()
        );
    } else {
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete"));
        assert!(
            invoke("eval").status.success(),
            "Incomplete is not Conflict"
        );
    }
    fs::remove_dir_all(dir).expect("remove owned probe files");
}

#[test]
fn strict_original_text_admission_rejects_invalid_and_dead_arms() {
    let good = wire(ret("defer"));
    for text in ["".to_owned(), "{".to_owned(), "null".to_owned(), format!("{good} {{}}"),
        "{\"formatVersion\":1,\"formatVersion\":1,\"body\":{\"op\":\"return\",\"action\":\"defer\"}}".to_owned(),
        "{\"formatVersion\":1,\"body\":{\"op\":\"return\",\"op\":\"return\",\"action\":\"defer\"}}".to_owned(),
        "{\"formatVersion\":1,\"body\":{\"op\":\"return\",\"action\":\"defer\",\"action\":\"defer\"}}".to_owned(),
    ] { unsupported(&text); }
    for (field, replacement) in [
        ("formatVersion", json!(0)),
        ("formatVersion", json!(2)),
        ("formatVersion", json!("1")),
        ("formatVersion", json!(1.0)),
        ("formatVersion", json!(true)),
        ("body", json!([])),
        ("unknown", json!(1)),
    ] {
        let mut root: Value = serde_json::from_str(&good).expect("fixture JSON");
        root[field] = replacement;
        unsupported(&root.to_string());
    }
    let duplicated_body = good.replacen("{", &format!("{{\"body\":{},", ret("defer")), 1);
    unsupported(&duplicated_body);
    let mut missing: Value = serde_json::from_str(&good).expect("fixture");
    missing
        .as_object_mut()
        .expect("object")
        .remove("formatVersion");
    unsupported(&missing.to_string());
    for bad in [
        json!({"op":"unknown"}),
        json!({"op":"return","action":"unknown"}),
        json!({"op":"return","action":null}),
        json!({"op":"return","action":"defer","field":"empty"}),
        json!({"op":"return","action":"defer","unknown":1}),
        json!({"op":"if","field":"unknown","then":ret("defer"),"else":ret("defer")}),
        json!({"op":"if","field":false,"then":ret("defer"),"else":ret("defer")}),
        json!({"op":"if","field":"empty","then":ret("defer"),"else":ret("defer"),"unknown":1}),
        json!({"op":"if","then":ret("defer"),"else":ret("defer")}),
        json!({"op":"if","field":"empty","then":ret("defer")}),
        json!({"op":"if","field":"empty","then":ret("defer"),"else":ret("defer"),"action":"defer"}),
    ] {
        unsupported(&wire(bad.clone()));
        // On a scalar graph this else arm would be dead; admission must still reject it.
        unsupported(&wire(branch("forced", ret("constraints"), bad)));
    }
    let base = wire(branch("empty", ret("defer"), ret("defer")));
    for field in ["field", "then", "else"] {
        let mut root: Value = serde_json::from_str(&base).expect("fixture");
        let duplicate = format!("\"{field}\":{},", root["body"][field]);
        let body = root["body"].as_object_mut().expect("if object");
        // Insert a duplicate into original text; parsing to Value first would erase it.
        body.insert("placeholder".to_owned(), json!(0));
        let text = root.to_string().replace("\"placeholder\":0,", &duplicate);
        unsupported(&text);
    }
}
fn unsupported(text: &str) {
    assert!(
        matches!(Policy::from_json(text), Err(Error::Unsupported(_))),
        "{text}"
    );
}

fn balanced_policy(leaves: usize) -> Value {
    if leaves == 1 {
        ret("defer")
    } else {
        branch(
            "empty",
            balanced_policy(leaves / 2),
            balanced_policy(leaves - leaves / 2),
        )
    }
}
#[test]
fn inclusive_utf8_text_and_instruction_node_depth_caps_are_independent() {
    let text = wire(ret("defer"));
    let exact = format!("{}{}", text, " ".repeat(16 * 1024 - text.len()));
    Policy::from_json(&exact).expect("inclusive original byte cap");
    assert!(matches!(
        Policy::from_json(&(exact + " ")),
        Err(Error::Limit(_))
    ));
    // Multibyte unknown spelling crosses the raw byte cap while still below it in characters.
    let utf8 = format!("{{\"{}\":0}}", "é".repeat(8192));
    assert!(utf8.chars().count() < 16 * 1024 && utf8.len() > 16 * 1024);
    assert!(matches!(Policy::from_json(&utf8), Err(Error::Limit(_))));
    Policy::from_json(&wire(balanced_policy(32))).expect("63 instruction nodes, not JSON values");
    assert!(matches!(
        Policy::from_json(&wire(balanced_policy(33))),
        Err(Error::Limit(_))
    ));
    let spine = |depth| {
        (1..depth).fold(ret("defer"), |child, _| {
            branch("empty", ret("defer"), child)
        })
    };
    Policy::from_json(&wire(spine(16))).expect("16 root-inclusive decision levels");
    assert!(matches!(
        Policy::from_json(&wire(spine(17))),
        Err(Error::Limit(_))
    ));
}

#[test]
fn instruction_fact_certificate_charges_share_observation_and_projection_budget() {
    let direct = decode(ret("defer"));
    let one_if = decode(branch("forced", ret("defer"), ret("defer")));
    let unvisited = decode(branch("forced", ret("defer"), balanced_policy(16)));
    let graph = graph(value(7));
    let baseline = run(&graph, Domain::Top, &direct);
    let conditional = run(&graph, Domain::Top, &one_if);
    assert_eq!(
        conditional.work,
        baseline.work + 2,
        "one instruction plus one fact read"
    );
    assert_eq!(
        run(&graph, Domain::Top, &unvisited).work,
        conditional.work,
        "admitted dead instructions are not executed"
    );
    // Forced native extraction adds no policy-dependent work beyond instruction/read/certificate.
    let native = graph
        .demand(Domain::Top, 4096)
        .expect("native constraint reference");
    let constraints = run(&graph, Domain::Top, &decode(ret("constraints")));
    assert_eq!(
        constraints.work,
        native.work + 2,
        "return instruction and response certificate"
    );
    for policy in [&direct, &one_if] {
        for allowance in [0, 1, 4097] {
            assert!(matches!(
                graph.demand_with_policy(Domain::Top, allowance, policy),
                Err(Error::Limit(_))
            ));
        }
    }
    let flat = graph_flat();
    let native = flat
        .demand(Domain::Top, 4096)
        .expect("native flat reference");
    let explicit = run(&flat, Domain::Top, &decode(ret("flat")));
    assert_eq!(
        explicit.work,
        native.work + 2,
        "flat projection shares the original observation allowance"
    );
    assert!(matches!(
        flat.demand_with_policy(Domain::Top, native.work, &decode(ret("flat"))),
        Err(Error::Limit(_))
    ));
    // Forcing/refining/defer cannot bypass raw product caps before conflict filtering.
    let alternatives = |width, n| or((0..width).map(|_| (false, value(n))).collect());
    for n in [1, 2] {
        let oversized = self::graph(and(alternatives(16, 1), alternatives(5, n)));
        for refinement in [Domain::Top, Domain::Value(9)] {
            assert!(matches!(
                oversized.demand_with_policy(refinement, 4096, &direct),
                Err(Error::Limit(_))
            ));
        }
    }
    let bounded = self::graph(and(alternatives(16, 1), alternatives(4, 1)));
    decision(
        &run(&bounded, Domain::Top, &direct),
        domains(&[1]),
        Readiness::PolicyDeferred,
        Demand::Suspended,
        None,
    );
}
