use bootstrap_executor::Error;
use bootstrap_executor::runtime::{Value, concrete, lower_bound, sum, unify, upper_bound};

fn samples() -> [Value; 14] {
    [
        Value::Int,
        Value::Integer(i64::MIN),
        Value::Integer(-1),
        Value::Integer(0),
        Value::Integer(1),
        Value::Integer(i64::MAX),
        lower_bound(Value::Integer(i64::MIN)).expect("lower"),
        lower_bound(Value::Integer(0)).expect("lower"),
        lower_bound(Value::Integer(i64::MAX)).expect("lower"),
        upper_bound(Value::Integer(i64::MIN)).expect("upper"),
        upper_bound(Value::Integer(0)).expect("upper"),
        upper_bound(Value::Integer(i64::MAX)).expect("upper"),
        Value::Bounds {
            lower: Some(-1),
            upper: Some(1),
        },
        Value::Bounds {
            lower: Some(0),
            upper: Some(0),
        },
    ]
}

#[test]
fn abstract_int_is_an_identity_for_admitted_integer_values() {
    for value in samples() {
        assert_eq!(unify(Value::Int, value.clone()).expect("left type"), value);
        assert_eq!(unify(value.clone(), Value::Int).expect("right type"), value);
        assert_eq!(
            unify(value.clone(), value.clone()).expect("idempotent"),
            value
        );
    }
}

#[test]
fn intersection_is_commutative_and_associative_for_sampled_values() {
    for a in samples() {
        for b in samples() {
            // Error provenance can depend on order; conflict versus value cannot.
            assert_eq!(
                unify(a.clone(), b.clone()).ok(),
                unify(b.clone(), a.clone()).ok(),
                "{a:?}, {b:?}"
            );
            for c in samples() {
                assert_eq!(
                    unify(a.clone(), b.clone())
                        .and_then(|ab| unify(ab, c.clone()))
                        .ok(),
                    unify(b.clone(), c.clone())
                        .and_then(|bc| unify(a.clone(), bc))
                        .ok(),
                    "{a:?}, {b:?}, {c:?}"
                );
            }
        }
    }
}

#[test]
fn singleton_bounds_remain_abstract_until_given_a_candidate() {
    let lower = lower_bound(Value::Integer(3)).expect("lower");
    let upper = upper_bound(Value::Integer(3)).expect("upper");
    let singleton = unify(lower.clone(), upper).expect("nonempty intersection");
    assert!(matches!(
        concrete(singleton.clone()),
        Err(Error::IncompleteBounds)
    ));
    assert_eq!(
        unify(singleton.clone(), Value::Integer(3)).expect("candidate"),
        Value::Integer(3)
    );
    assert!(matches!(
        unify(singleton.clone(), Value::Integer(4)),
        Err(Error::BoundsConflict)
    ));
    assert!(matches!(
        unify(lower, upper_bound(Value::Integer(2)).expect("upper")),
        Err(Error::BoundsConflict)
    ));
    assert!(matches!(
        sum(singleton, Value::Integer(1)),
        Err(Error::Arithmetic)
    ));
}

#[test]
fn absent_endpoints_are_infinite_and_thresholds_must_be_concrete() {
    for bound in [
        lower_bound(Value::Integer(i64::MAX)),
        upper_bound(Value::Integer(i64::MIN)),
    ] {
        let bound = bound.expect("bound");
        assert!(matches!(
            concrete(bound.clone()),
            Err(Error::IncompleteBounds)
        ));
        assert!(matches!(
            lower_bound(bound.clone()),
            Err(Error::BoundThreshold)
        ));
        assert!(matches!(upper_bound(bound), Err(Error::BoundThreshold)));
    }
    assert!(matches!(
        lower_bound(Value::Int),
        Err(Error::BoundThreshold)
    ));
    assert!(matches!(
        upper_bound(Value::Int),
        Err(Error::BoundThreshold)
    ));
}

#[test]
fn incomplete_is_not_conflict() {
    assert!(matches!(concrete(Value::Int), Err(Error::Incomplete)));
    assert!(matches!(
        unify(Value::Integer(1), Value::Integer(2)),
        Err(Error::Conflict { left: 1, right: 2 })
    ));
    assert!(matches!(
        sum(Value::Int, Value::Integer(1)),
        Err(Error::Arithmetic)
    ));
}
