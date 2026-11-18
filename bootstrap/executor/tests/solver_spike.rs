// Test-only integer IR: deliberately not a production parser or evaluator target.
#[path = "../../spikes/solver.rs"]
mod solver;

use std::{collections::BTreeMap, path::Path, process::Command};

use serde_json::{Value as Json, json};
use solver::{Domain, Error, Expr, Field, Graph, Limit, Report, Schedule, Status};

fn field(name: &str, constraints: Vec<Expr>) -> Field {
    Field {
        name: name.into(),
        constraints,
        preferred: None,
    }
}
fn preferred(name: &str, constraints: Vec<Expr>) -> Field {
    Field {
        preferred: Some(1),
        ..field(name, constraints)
    }
}
fn reference(name: &str) -> Expr {
    Expr::Ref(name.into())
}
fn add(left: Expr, right: Expr) -> Expr {
    Expr::Add(Box::new(left), Box::new(right))
}
fn plus(name: &str, value: i64) -> Expr {
    add(reference(name), Expr::Value(value))
}
fn domains(entries: &[(&str, Domain)]) -> BTreeMap<String, Domain> {
    entries
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone()))
        .collect()
}
fn configurations(mut check: impl FnMut(Schedule, bool)) {
    for schedule in [Schedule::Fifo, Schedule::Lifo] {
        for reverse in [false, true] {
            check(schedule, reverse);
        }
    }
}
fn check_graph(
    fields: Vec<Field>,
    constraints: &[(&str, Domain)],
    selected: &[(&str, Domain)],
    status: Status,
) -> Report {
    let graph = Graph::new(fields).expect("valid isolated solver IR");
    configurations(|schedule, reverse| {
        let report = graph.solve(schedule, reverse, 4096).expect("bounded solve");
        assert_eq!(report.constraints, domains(constraints));
        assert_eq!(report.selected, domains(selected));
        assert_eq!(report.status, status);
        assert!(report.work > 0 && report.work <= 4096);
    });
    // Deliberately do not compare work counters across schedules.
    graph
        .solve(Schedule::Fifo, false, 4096)
        .expect("repeat solve owns fresh state")
}
fn exact(report: &Report) -> Json {
    Json::Object(
        report
            .selected
            .iter()
            .map(|(name, domain)| {
                let Domain::Value(value) = domain else {
                    panic!("non-concrete selected field {name}")
                };
                (name.clone(), Json::from(*value))
            })
            .collect(),
    )
}
fn cue(probe: &str, operation: &str) -> std::process::Output {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spikes/solver/probes.cue");
    let mut command = Command::new("cue");
    command.arg(operation).arg(path).args(["-e", probe]);
    if operation == "export" {
        command.args(["--out", "json"]);
    }
    command
        .output()
        .expect("pinned CUE must be on PATH; upstream checks never skip")
}
fn upstream_concrete(probe: &str, report: &Report, expected: Json) {
    assert_eq!(exact(report), expected, "native {probe}");
    let output = cue(probe, "export");
    assert!(
        output.status.success(),
        "{probe}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let upstream: Json = serde_json::from_slice(&output.stdout).expect("exact integer JSON");
    assert_eq!(
        upstream, expected,
        "independent upstream expectation for {probe}"
    );
    assert_eq!(exact(report), upstream, "native versus upstream {probe}");
}
fn upstream_unresolved(probe: &str, diagnostic: &str) {
    let evaluated = cue(probe, "eval");
    assert!(
        evaluated.status.success(),
        "{probe} is not bottom: {}",
        String::from_utf8_lossy(&evaluated.stderr)
    );
    let output = cue(probe, "export");
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "failed export must not leak partial JSON"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{probe}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pinned_upstream_is_required() {
    let output = Command::new("cue")
        .arg("version")
        .output()
        .expect("pinned CUE required");
    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).expect("CUE version UTF-8");
    assert!(
        version
            .lines()
            .any(|line| line.trim() == "CUE language version v0.16.1"),
        "unexpected CUE: {version}"
    );
}

#[test]
fn forward_references_and_stable_identity() {
    use Domain::Value as V;
    let expected = [("a", V(3)), ("b", V(2))];
    for reversed in [false, true] {
        let mut fields = vec![
            field("a", vec![plus("b", 1)]),
            field("b", vec![Expr::Value(2)]),
        ];
        if reversed {
            fields.reverse();
        }
        let graph = Graph::new(fields.clone()).expect("forward names interned first");
        assert_eq!(graph.id("a"), Some(0));
        assert_eq!(graph.id("b"), Some(1));
        assert_eq!(graph.id("missing"), None);
        assert_eq!(graph.field_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        let report = check_graph(fields, &expected, &expected, Status::Concrete);
        upstream_concrete(
            if reversed {
                "forwardReversed"
            } else {
                "forward"
            },
            &report,
            json!({"a":3,"b":2}),
        );
    }
}

#[test]
fn diamond_all_twenty_four_declarations_and_four_queues() {
    use Domain::Value as V;
    let fields = [
        field("a", vec![add(reference("b"), reference("c"))]),
        field("b", vec![plus("base", 1)]),
        field("c", vec![plus("base", 2)]),
        field("base", vec![Expr::Value(2)]),
    ];
    let expected = [("a", V(7)), ("b", V(3)), ("base", V(2)), ("c", V(4))];
    let mut permutations = 0;
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let order = [a, b, c, d];
                    if (0..4).any(|i| (i + 1..4).any(|j| order[i] == order[j])) {
                        continue;
                    }
                    permutations += 1;
                    let declarations = order.map(|i| fields[i].clone()).to_vec();
                    let graph = Graph::new(declarations.clone()).expect("diamond");
                    assert_eq!(graph.field_count(), 4);
                    assert_eq!(graph.edge_count(), 4);
                    for (id, name) in ["a", "b", "base", "c"].into_iter().enumerate() {
                        assert_eq!(graph.id(name), Some(id));
                    }
                    let report = check_graph(declarations, &expected, &expected, Status::Concrete);
                    if permutations == 1 {
                        upstream_concrete("diamond", &report, json!({"a":7,"b":3,"c":4,"base":2}));
                    }
                }
            }
        }
    }
    assert_eq!(permutations, 24);
}

#[test]
fn defaults_project_without_rewriting_constraints() {
    use Domain::{Int, Value as V};
    let report = check_graph(
        vec![preferred("choice", vec![Expr::Int])],
        &[("choice", Int)],
        &[("choice", V(1))],
        Status::Concrete,
    );
    upstream_concrete("preferred", &report, json!({"choice":1}));
    let report = check_graph(
        vec![preferred("choice", vec![Expr::Int, Expr::Value(2)])],
        &[("choice", V(2))],
        &[("choice", V(2))],
        Status::Concrete,
    );
    upstream_concrete("refined", &report, json!({"choice":2}));
    let report = check_graph(
        vec![
            field("a", vec![plus("b", 1)]),
            preferred("b", vec![Expr::Int]),
        ],
        &[("a", Int), ("b", Int)],
        &[("a", V(2)), ("b", V(1))],
        Status::Concrete,
    );
    upstream_concrete("dependent", &report, json!({"a":2,"b":1}));
}

#[test]
fn incomplete_aliases_and_arithmetic_cycles_are_not_conflicts() {
    use Domain::{Any, Int};
    check_graph(
        vec![
            field("a", vec![reference("b")]),
            field("b", vec![reference("a")]),
        ],
        &[("a", Any), ("b", Any)],
        &[("a", Any), ("b", Any)],
        Status::Incomplete,
    );
    upstream_unresolved("mutual", "incomplete");
    check_graph(
        vec![field("a", vec![plus("a", 1)])],
        &[("a", Int)],
        &[("a", Int)],
        Status::Cycle,
    );
    upstream_unresolved("self", "cycle");
    check_graph(
        vec![
            field("a", vec![plus("b", 1)]),
            field("b", vec![plus("a", -1)]),
        ],
        &[("a", Int), ("b", Int)],
        &[("a", Int), ("b", Int)],
        Status::Cycle,
    );
    upstream_unresolved("balanced", "cycle");
    check_graph(
        vec![field("any", vec![Expr::Any]), field("int", vec![Expr::Int])],
        &[("any", Any), ("int", Int)],
        &[("any", Any), ("int", Int)],
        Status::Incomplete,
    );
}

#[test]
fn readiness_is_not_inverse_arithmetic_or_a_domain_only_export() {
    use Domain::{Any, Int, Value as V};
    check_graph(
        vec![
            field("a", vec![plus("b", 1)]),
            field("b", vec![reference("c")]),
            field("c", vec![reference("b")]),
        ],
        &[("a", Int), ("b", Any), ("c", Any)],
        &[("a", Int), ("b", Any), ("c", Any)],
        Status::Incomplete,
    );
    upstream_unresolved("blocked", "incomplete");
    let output = cue("blocked", "export");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("cycle with field"));
    check_graph(
        vec![
            field("a", vec![Expr::Value(3), plus("b", 1)]),
            field("b", vec![Expr::Any]),
        ],
        &[("a", V(3)), ("b", Any)],
        &[("a", V(3)), ("b", Any)],
        Status::Incomplete,
    );
    upstream_unresolved("noInverse", "incomplete");
    // An inline abstract operand has no field whose domain can expose unreadiness.
    // Reject it instead of falsely declaring the concrete target exportable.
    for operand in [Expr::Any, Expr::Int] {
        for expr in [
            add(operand.clone(), Expr::Value(1)),
            add(Expr::Value(1), operand.clone()),
            add(Expr::Value(1), add(Expr::Value(1), operand)),
        ] {
            construction_error(vec![field("a", vec![Expr::Value(3), expr])], |e| {
                matches!(e, Error::Unsupported(_))
            });
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spikes/solver/abstract-operand.cue");
    let output = Command::new("cue")
        .arg("export")
        .arg(path)
        .output()
        .expect("mandatory pinned invalid-operand probe");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid operand"));
}

#[test]
fn projection_can_anchor_an_otherwise_unresolved_arithmetic_cycle() {
    use Domain::{Int, Value as V};
    let report = check_graph(
        vec![
            field("a", vec![plus("b", 1)]),
            Field {
                preferred: Some(2),
                ..field("b", vec![plus("a", -1), Expr::Int])
            },
        ],
        &[("a", Int), ("b", Int)],
        &[("a", V(3)), ("b", V(2))],
        Status::Concrete,
    );
    upstream_concrete("projectedAnchor", &report, json!({"a":3,"b":2}));
}

#[test]
fn anchors_resolve_aliases_and_balanced_arithmetic() {
    use Domain::Value as V;
    let expected = [("a", V(2)), ("b", V(2))];
    let report = check_graph(
        vec![
            field("a", vec![reference("b"), Expr::Value(2)]),
            field("b", vec![reference("a")]),
        ],
        &expected,
        &expected,
        Status::Concrete,
    );
    upstream_concrete("alias", &report, json!({"a":2,"b":2}));
    let expected = [("a", V(3)), ("b", V(2))];
    let report = check_graph(
        vec![
            field("a", vec![plus("b", 1)]),
            field("b", vec![plus("a", -1), Expr::Value(2)]),
        ],
        &expected,
        &expected,
        Status::Concrete,
    );
    upstream_concrete("anchored", &report, json!({"a":3,"b":2}));
}

fn solve_error(fields: Vec<Field>, predicate: impl Fn(&Error) -> bool) {
    let graph = Graph::new(fields).expect("structurally valid error case");
    configurations(|schedule, reverse| {
        let error = graph
            .solve(schedule, reverse, 4096)
            .expect_err("must reject");
        assert!(predicate(&error), "unexpected error: {error:?}");
    });
}

#[test]
fn conflict_overflow_and_unsupported_defaults_are_distinct() {
    solve_error(
        vec![field("a", vec![Expr::Value(1), Expr::Value(2)])],
        |e| matches!(e, Error::Conflict),
    );
    let output = cue("conflict", "export");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("conflicting values"));
    // CUE integers are unbounded: these are native range failures, not contradictions.
    for (left, right) in [(i64::MAX, 1), (i64::MIN, -1)] {
        solve_error(
            vec![field("a", vec![add(Expr::Value(left), Expr::Value(right))])],
            |e| matches!(e, Error::Overflow),
        );
    }
    let output = cue("overflow", "export");
    assert!(output.status.success(), "CUE arithmetic is not i64-bounded");
    assert_eq!(
        serde_json::from_slice::<Json>(&output.stdout).expect("exact large integer JSON"),
        json!({"a": 9223372036854775808_u64})
    );
    // Valid upstream, but outside the deliberately single-default calculus.
    let output = cue("multipleDefaults", "export");
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Json>(&output.stdout).expect("upstream defaults JSON"),
        json!({"a":1,"b":2})
    );
    let multiple = vec![
        preferred("a", vec![Expr::Int]),
        preferred("b", vec![Expr::Int]),
    ];
    match Graph::new(multiple.clone()) {
        Err(Error::Unsupported(_)) => {}
        Ok(_) => solve_error(multiple, |e| matches!(e, Error::Unsupported(_))),
        Err(error) => panic!("wrong multiple-default category: {error:?}"),
    }
    // Without a preference this is incomplete, not unsatisfiable. Choosing a=1
    // makes b=a+1 disagree with b=1; the spike cannot backtrack to a=0.
    solve_error(
        vec![
            preferred("a", vec![Expr::Int]),
            field("b", vec![plus("a", 1), Expr::Value(1)]),
        ],
        |e| matches!(e, Error::Unsupported(_)),
    );
}

fn construction_error(fields: Vec<Field>, predicate: impl FnOnce(&Error) -> bool) {
    match Graph::new(fields) {
        Err(error) => assert!(
            predicate(&error),
            "unexpected construction error: {error:?}"
        ),
        Ok(_) => panic!("invalid graph admitted"),
    }
}
fn nested(levels: usize) -> Expr {
    (1..levels).fold(Expr::Value(0), |expr, _| add(expr, Expr::Value(0)))
}

#[test]
fn invalid_ir_and_name_byte_boundaries() {
    construction_error(vec![field("a", vec![]), field("a", vec![])], |e| {
        matches!(e, Error::Invalid(_))
    });
    construction_error(vec![field("", vec![Expr::Any])], |e| {
        matches!(e, Error::Invalid(_))
    });
    construction_error(vec![field(&"a".repeat(65), vec![Expr::Any])], |e| {
        matches!(e, Error::Invalid(_))
    });
    construction_error(vec![field(&"é".repeat(33), vec![Expr::Any])], |e| {
        matches!(e, Error::Invalid(_))
    });
    construction_error(
        vec![field("a", vec![reference("missing")])],
        |e| matches!(e, Error::UnknownReference(name) if name == "missing"),
    );
    for name in ["a".repeat(64), "é".repeat(32)] {
        let graph = Graph::new(vec![field(&name, vec![Expr::Any])]).expect("64 bytes inclusive");
        assert_eq!(graph.id(&name), Some(0));
    }
}

#[test]
fn structural_hard_caps_are_inclusive_and_cannot_be_raised() {
    let fields = |n| {
        (0..n)
            .map(|i| field(&format!("f{i}"), vec![Expr::Any]))
            .collect()
    };
    assert_eq!(Graph::new(fields(64)).expect("64 fields").field_count(), 64);
    construction_error(fields(65), |e| matches!(e, Error::Limit(Limit::Fields)));
    Graph::new(vec![field("a", vec![Expr::Any; 256])]).expect("256 expression nodes");
    construction_error(vec![field("a", vec![Expr::Any; 257])], |e| {
        matches!(e, Error::Limit(Limit::Expressions))
    });
    let refs = |n| {
        vec![
            field("a", vec![reference("b"); n]),
            field("b", vec![Expr::Any]),
        ]
    };
    Graph::new(refs(128)).expect("128 reference occurrences");
    construction_error(refs(129), |e| matches!(e, Error::Limit(Limit::Edges)));
    Graph::new(vec![field("a", vec![nested(16)])]).expect("16 expression levels");
    construction_error(vec![field("a", vec![nested(17)])], |e| {
        matches!(e, Error::Limit(Limit::Depth))
    });
}

#[test]
fn full_width_alias_chain_solves_and_classifies_without_reference_expansion() {
    for anchor in [false, true] {
        let fields: Vec<_> = (0..64)
            .map(|i| {
                let expr = if i < 63 {
                    reference(&format!("n{:02}", i + 1))
                } else if anchor {
                    Expr::Value(2)
                } else {
                    Expr::Any
                };
                field(&format!("n{i:02}"), vec![expr])
            })
            .collect();
        let graph = Graph::new(fields).expect("64-field alias chain");
        assert_eq!(graph.field_count(), 64);
        assert_eq!(graph.edge_count(), 63);
        configurations(|schedule, reverse| {
            let report = graph
                .solve(schedule, reverse, 4096)
                .expect("bounded chain solve");
            let expected = if anchor {
                Domain::Value(2)
            } else {
                Domain::Any
            };
            assert!(
                report
                    .constraints
                    .values()
                    .all(|domain| *domain == expected)
            );
            assert_eq!(report.constraints, report.selected);
            assert_eq!(
                report.status,
                if anchor {
                    Status::Concrete
                } else {
                    Status::Incomplete
                }
            );
            assert!(report.work > 0 && report.work <= 4096);
        });
    }
}

#[test]
fn work_limit_cannot_reset_between_phases() {
    let literal = Graph::new(vec![field("a", vec![Expr::Value(1)])]).expect("literal");
    let default = Graph::new(vec![preferred("a", vec![Expr::Int])]).expect("default");
    configurations(|schedule, reverse| {
        let one_phase = literal
            .solve(schedule, reverse, 4096)
            .expect("literal result");
        let two_phase = default
            .solve(schedule, reverse, 4096)
            .expect("default result");
        assert_eq!(one_phase.selected, two_phase.selected);
        // The extra domain pass cannot be hidden by resetting its counter.
        assert!(two_phase.work > one_phase.work);
        assert!(matches!(
            default.solve(schedule, reverse, one_phase.work),
            Err(Error::Limit(Limit::Work))
        ));
    });
}

#[test]
fn cumulative_work_budget_covers_constraint_and_projection_phases() {
    for b in [
        field("b", vec![Expr::Value(1)]),
        preferred("b", vec![Expr::Int]),
    ] {
        let graph = Graph::new(vec![field("a", vec![plus("b", 1)]), b]).expect("work graph");
        configurations(|schedule, reverse| {
            assert!(matches!(
                graph.solve(schedule, reverse, 0),
                Err(Error::Limit(Limit::Work))
            ));
            assert!(matches!(
                graph.solve(schedule, reverse, 4097),
                Err(Error::Invalid(_))
            ));
            let report = graph.solve(schedule, reverse, 4096).expect("hard maximum");
            assert!(report.work > 1 && report.work <= 4096);
            assert!(matches!(
                graph.solve(schedule, reverse, report.work - 1),
                Err(Error::Limit(Limit::Work))
            ));
            let same = graph
                .solve(schedule, reverse, report.work)
                .expect("exact cumulative budget");
            assert_eq!(same.constraints, report.constraints);
            assert_eq!(same.selected, report.selected);
            assert_eq!(same.status, Status::Concrete);
        });
    }
}
