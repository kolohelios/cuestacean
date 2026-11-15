use bootstrap_executor::Error;
use bootstrap_executor::runtime::{Value, concrete, sum, unify};

#[test]
fn abstract_int_is_an_identity_for_admitted_integer_values() {
    for value in [
        Value::Int,
        Value::Integer(i64::MIN),
        Value::Integer(0),
        Value::Integer(i64::MAX),
    ] {
        assert_eq!(unify(Value::Int, value).expect("left type"), value);
        assert_eq!(unify(value, Value::Int).expect("right type"), value);
        assert_eq!(unify(value, value).expect("idempotent"), value);
    }
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
