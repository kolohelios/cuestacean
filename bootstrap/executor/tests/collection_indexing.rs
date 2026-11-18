use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, runtime, unification};
use runtime::{ClosedList, ClosedRecord, Value as V};
use serde_json::{Value as J, json};

#[path = "../../generated/index-zero.rs"]
mod alternate;
#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn text(value: &str) -> V {
    V::Text(Arc::from(value))
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("closed list"))
}
fn record(fields: Vec<(&str, V)>) -> V {
    V::Record(
        ClosedRecord::new(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
            .expect("closed record"),
    )
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn array(elements: Vec<J>) -> J {
    json!({"kind":"listExpression","elements":elements})
}
fn program(entry: J) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: J) -> [Result<J, Error>; 2] {
    let input = program(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn outcomes(left: &V, right: &V) -> [Result<V, Error>; 2] {
    [
        runtime::index(left.clone(), right.clone()),
        unification::interpret(&export("index"), left.clone(), right.clone()),
    ]
}
fn parses(source: &str, grammar: &str) -> [Result<J, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn negate(mut entry: J, count: usize) -> J {
    for _ in 0..count {
        entry = json!({"kind":"negate","operand":entry});
    }
    entry
}

// Independent AST representatives of the owned matrix's domains and lookup data.
fn domain_ast(value: &V) -> J {
    match value {
        V::Top => json!({"kind":"top"}),
        V::Null => json!({"kind":"null"}),
        V::Bool => json!({"kind":"boolType"}),
        V::Boolean(value) => json!({"kind":if *value {"true"} else {"false"}}),
        V::Int => json!({"kind":"intType"}),
        V::Integer(value) => integer(*value),
        V::String => json!({"kind":"stringType"}),
        V::Text(value) => json!({"kind":"string","value":value.as_ref()}),
        V::Bounds { .. } => json!({"kind":"lowerBound","value":0}),
        V::List(_) => json!({"kind":"list","value":[null]}),
        V::Record(_) => json!({"kind":"record","fields":[{"name":"0","value":null}]}),
    }
}

#[test]
fn complete_kind_key_matrix_and_strict_typed_apis() {
    let domains = [
        V::Top,
        V::Null,
        V::Bool,
        V::Boolean(false),
        V::Int,
        V::Integer(0),
        V::String,
        text("0"),
        V::Bounds {
            lower: Some(0),
            upper: Some(0),
        },
        list(vec![V::Null]),
        record(vec![("0", V::Null)]),
    ];
    for left in &domains {
        for right in &domains {
            let admitted = matches!(
                (left, right),
                (V::List(_), V::Integer(_)) | (V::Record(_), V::Text(_))
            );
            for result in outcomes(left, right) {
                if admitted {
                    assert_eq!(result.expect("matching concrete domains"), V::Null);
                } else {
                    assert!(
                        matches!(result, Err(Error::CollectionIndexOperands)),
                        "{left:?}[{right:?}]: {result:?}"
                    );
                }
            }
            for result in both(binary("index", domain_ast(left), domain_ast(right))) {
                if admitted {
                    assert_eq!(result.expect("AST domain pair"), J::Null);
                } else {
                    assert!(matches!(result, Err(Error::CollectionIndexOperands)));
                }
            }
            let typed_list = runtime::list_index(left.clone(), right.clone());
            if matches!((left, right), (V::List(_), V::Integer(_))) {
                assert_eq!(typed_list.expect("typed list"), V::Null);
            } else {
                assert!(matches!(typed_list, Err(Error::IndexOperands)));
            }
            let typed_record = runtime::record_index(left.clone(), right.clone());
            if matches!((left, right), (V::Record(_), V::Text(_))) {
                assert_eq!(typed_record.expect("typed record"), V::Null);
            } else {
                assert!(matches!(typed_record, Err(Error::RecordIndexOperands)));
            }
        }
    }
    assert_eq!(
        Error::CollectionIndexOperands.to_string(),
        "indexing requires a concrete closed list and integer or closed record and string"
    );
}

#[test]
fn ranges_missing_fields_and_exact_unicode_keys() {
    for (items, key) in [
        (list(vec![]), 0),
        (list(vec![V::Null]), -1),
        (list(vec![V::Null]), i64::MIN),
        (list(vec![V::Null]), i64::MAX),
        (list(vec![V::Null; 64]), 64),
    ] {
        for result in outcomes(&items, &V::Integer(key)) {
            assert!(matches!(result, Err(Error::IndexRange)));
            assert_eq!(
                result.expect_err("range").to_string(),
                "list index out of range"
            );
        }
    }
    for result in outcomes(&list(vec![V::Null; 64]), &V::Integer(63)) {
        assert_eq!(result.expect("last element"), V::Null);
    }
    let fields = record(vec![("é", V::Integer(1)), ("e\u{301}", V::Integer(2))]);
    for (key, expected) in [("é", 1), ("e\u{301}", 2)] {
        for result in outcomes(&fields, &text(key)) {
            assert_eq!(result.expect("byte-exact key"), V::Integer(expected));
        }
    }
    for fields in [fields, record(vec![])] {
        for key in ["", "missing", "É"] {
            for result in outcomes(&fields, &text(key)) {
                let error = result.expect_err("missing field");
                assert!(matches!(&error, Error::RecordMissing { field } if field == key));
                assert_eq!(
                    error.to_string(),
                    format!("missing closed record field {key:?}")
                );
            }
        }
    }
}

#[test]
fn eager_child_errors_precede_all_index_policy() {
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("add", integer(i64::MAX), integer(1));
    let empty_record = json!({"kind":"record","fields":[]});
    for (left, right, expected) in [
        (zero.clone(), overflow.clone(), Error::DivisionByZero),
        (json!({"kind":"null"}), zero.clone(), Error::DivisionByZero),
        (array(vec![]), overflow, Error::Overflow),
        (empty_record, zero, Error::DivisionByZero),
    ] {
        for result in both(binary("index", left, right)) {
            assert_eq!(
                result.expect_err("eager error").to_string(),
                expected.to_string()
            );
        }
    }
}

#[test]
fn selected_evaluator_and_shared_exact_visit_and_depth_limits() {
    for (wrappers, accepted) in [(61, true), (62, false)] {
        let items = array(vec![negate(integer(0), wrappers)]);
        let entry = binary("index", items, integer(0));
        for result in both(entry) {
            if accepted {
                assert_eq!(result.expect("64 active nodes"), json!(0));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
    for (wrappers, accepted) in [(62, true), (63, false)] {
        let entry = binary(
            "index",
            array(vec![integer(0)]),
            negate(integer(0), wrappers),
        );
        for result in both(entry) {
            if accepted {
                assert_eq!(result.expect("64 active RHS nodes"), json!(0));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
    let mut spine = integer(0);
    for _ in 0..10 {
        spine = binary("add", spine, integer(99));
    }
    for extra in [0, 1] {
        // Selected add repeats its left: each spine costs 2047 visits and returns 0.
        // Index + list + two spines = 4096; RHS shares the left child's context.
        let input = program(binary(
            "index",
            array(vec![spine.clone()]),
            negate(spine.clone(), extra),
        ));
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if extra == 0 {
                assert_eq!(result.expect("4096 shared visits"), json!(0));
            } else {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            }
        }
    }
    let collection = program(array(vec![integer(0)]));
    assert!(matches!(
        left_twice::execute(&collection),
        Err(Error::IntegerOutput)
    ));
}

#[test]
fn mixed_chains_and_selected_storage_outlive_parent_and_input() {
    let fields =
        json!({"kind":"recordExpression","fields":[{"name":"x","value":array(vec![integer(7)])}]});
    for (outer, middle, inner) in [
        ("index", "index", "index"),
        ("listIndex", "index", "listIndex"),
        ("index", "recordIndex", "index"),
    ] {
        let parent = binary(outer, array(vec![fields.clone()]), integer(0));
        let selected = binary(middle, parent, json!({"kind":"string","value":"x"}));
        let entry = binary(inner, selected, integer(0));
        for result in both(entry) {
            assert_eq!(result.expect("mixed chain"), json!(7));
        }
    }
    for record_parent in [false, true] {
        let leaf: Arc<str> = Arc::from("owned é🙂");
        let witness: Arc<str> = Arc::from("parent only");
        let child = if record_parent {
            list(vec![V::Text(leaf.clone())])
        } else {
            record(vec![("x", V::Text(leaf.clone()))])
        };
        let parent = if record_parent {
            record(vec![
                ("x", child.clone()),
                ("other", V::Text(witness.clone())),
            ])
        } else {
            list(vec![child.clone(), V::Text(witness.clone())])
        };
        let (key, child_key) = if record_parent {
            (text("x"), V::Integer(0))
        } else {
            (V::Integer(0), text("x"))
        };
        let selections = outcomes(&parent, &key);
        drop(parent);
        drop(child);
        assert_eq!(Arc::strong_count(&witness), 1);
        let weak = Arc::downgrade(&leaf);
        drop(leaf);
        for selected in selections {
            let selected =
                runtime::index(selected.expect("child"), child_key.clone()).expect("text");
            let V::Text(selected) = selected else {
                panic!("text")
            };
            assert!(Arc::ptr_eq(
                &weak.upgrade().expect("shared allocation"),
                &selected
            ));
            assert_eq!(&*selected, "owned é🙂");
        }
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn compiled_index_zero_and_binary_failure_validation_are_not_name_dispatch() {
    let policy = export("tests/valid/index-zero");
    for (left, right) in [
        (list(vec![V::Null]), V::Integer(0)),
        (record(vec![("x", V::Null)]), text("x")),
        (V::Top, V::Int),
    ] {
        assert_eq!(
            alternate::index(left.clone(), right.clone()).expect("compiled alternate"),
            V::Integer(0)
        );
        assert_eq!(
            unification::interpret(&policy, left, right).expect("reference alternate"),
            V::Integer(0)
        );
    }
    assert!(unification::interpret_unary(&policy, V::Null).is_err());
    let ignore =
        json!({"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}).to_string();
    let fail = json!({"op":"fail","reason":"collectionIndexOperands"});
    let binary = json!({"formatVersion":1,"function":"index","result":fail});
    unification::generate(&binary.to_string()).expect("binary failure");
    assert!(matches!(
        unification::interpret(&binary.to_string(), V::Null, V::Null),
        Err(Error::CollectionIndexOperands)
    ));
    for result in [
        fail.clone(),
        json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"list"},
        "then":{"op":"integer","value":0},"else":fail}),
    ] {
        let invalid =
            json!({"formatVersion":1,"function":"listLength","result":result}).to_string();
        assert!(unification::generate(&invalid).is_err());
        assert!(unification::generate_with_policy(&invalid, &ignore).is_err());
        assert!(matches!(
            unification::interpret_unary(&invalid, list(vec![])),
            Err(Error::UnificationProgram { .. })
        ));
    }
}

#[test]
fn exact_default_source_trees_and_explicit_legacy_constructor() {
    let mut grammar: J = serde_json::from_str(&export("grammar")).expect("grammar");
    assert_eq!(grammar["index"]["kind"], "index");
    let items = array(vec![integer(10), integer(20)]);
    for kind in ["index", "listIndex"] {
        grammar["index"]["kind"] = json!(kind);
        let selected = binary(kind, items.clone(), integer(0));
        for (source, expected) in [
            (
                "[10,20][0+1]",
                binary(kind, items.clone(), binary("add", integer(0), integer(1))),
            ),
            ("-[10,20][0]", json!({"kind":"negate","operand":selected})),
            ("[10,20][0][0]", binary(kind, selected, integer(0))),
        ] {
            for parsed in parses(source, &grammar.to_string()) {
                assert_eq!(
                    parsed.expect(source),
                    json!({"formatVersion":1,"entry":expected})
                );
            }
        }
        for parsed in parses("[10,20][0][0]", &grammar.to_string()) {
            let tree = parsed.expect("chain");
            for result in both(tree["entry"].clone()) {
                assert!(matches!(
                    (kind, result),
                    ("index", Err(Error::CollectionIndexOperands))
                        | ("listIndex", Err(Error::IndexOperands))
                ));
            }
        }
    }
}
