use bootstrap_executor::{Error, execute_json, generated, runtime, unification};
use runtime::{
    ClosedList, ClosedRecord, EvaluationContext, ListDriver, RecordDriver, Value as V,
    list_expression_with_driver as run_list, record_expression_with_driver as run_record,
};
use serde_json::{Value as J, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/list-control-homogeneous.rs"]
mod homogeneous_lists;
#[path = "../../generated/record-control-homogeneous.rs"]
mod homogeneous_records;
#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/list-control.rs"]
mod lists;
#[path = "../../generated/record-control.rs"]
mod records;
#[path = "../../generated/list-control-reverse.rs"]
mod reverse_lists;
#[path = "../../generated/record-control-reverse.rs"]
mod reverse_records;
#[path = "../../generated/list-control-scalars.rs"]
mod scalar_lists;
#[path = "../../generated/record-control-scalars.rs"]
mod scalar_records;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn owned(record: bool, children: Vec<V>) -> Result<V, Error> {
    if record {
        ClosedRecord::new(
            children
                .into_iter()
                .enumerate()
                .map(|(i, value)| (format!("f{i}"), value))
                .collect(),
        )
        .map(V::Record)
    } else {
        ClosedList::new(children).map(V::List)
    }
}
fn collection(record: bool, children: Vec<J>) -> J {
    if record {
        let fields: Vec<_> = children
            .into_iter()
            .enumerate()
            .map(|(i, value)| json!({"name":format!("f{i}"),"value":value}))
            .collect();
        json!({"kind":"recordExpression","fields":fields})
    } else {
        json!({"kind":"listExpression","elements":children})
    }
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
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
fn shape_error(result: Result<V, Error>, record: bool, depth: bool) {
    let error = result.expect_err("prospective parent exceeds shared shape");
    assert!(matches!(
        (&error, record, depth),
        (Error::RecordDepth, true, true)
            | (Error::ListDepth, false, true)
            | (Error::RecordNodes, true, false)
            | (Error::ListNodes, false, false)
    ));
}
fn output(record: bool, children: Vec<J>) -> J {
    if record {
        J::Object(
            children
                .into_iter()
                .enumerate()
                .map(|(i, v)| (format!("f{i}"), v))
                .collect(),
        )
    } else {
        J::Array(children)
    }
}

#[test]
fn alternating_depth_is_root_inclusive_for_owned_and_ast_values() {
    for root_record in [false, true] {
        for depth in [32, 33] {
            let mut value = V::Null;
            let mut ast = json!({"kind":"null"});
            let mut expected = J::Null;
            for level in (0..depth - 1).rev() {
                let record = root_record ^ (level % 2 != 0);
                ast = collection(record, vec![ast]);
                expected = output(record, vec![expected]);
                let result = owned(record, vec![value]);
                if depth == 33 && level == 0 {
                    shape_error(result, root_record, true);
                    break;
                }
                value = result.expect("at most 32 combined levels");
            }
            for result in both(ast) {
                if depth == 32 {
                    assert_eq!(result.expect("32 alternating levels"), expected);
                } else {
                    let error = result.expect_err("33 alternating levels");
                    assert!(matches!(
                        (&error, root_record),
                        (Error::RecordDepth, true) | (Error::ListDepth, false)
                    ));
                }
            }
        }
    }
}

#[test]
fn mixed_dag_occurrences_have_exact_shared_node_boundaries() {
    for record in [false, true] {
        let shared = owned(!record, vec![V::Null; 64]).expect("65 nodes");
        let accepted = owned(record, vec![shared.clone(); 63]).expect("1 + 63 * 65 = 4096");
        let expected = output(record, vec![output(!record, vec![J::Null; 64]); 63]);
        assert_eq!(
            runtime::encode_json(accepted.clone()).expect("expanded DAG"),
            expected
        );
        let mut rejected = vec![shared; 63];
        rejected.push(V::Null); // Width 64, exactly 4097 logical nodes.
        shape_error(owned(record, rejected), record, false);
        shape_error(owned(!record, vec![accepted]), !record, false);
    }
}

#[test]
fn selected_cross_kind_child_shares_storage_but_releases_parent() {
    for record in [false, true] {
        let text: Arc<str> = Arc::from("owned é🙂");
        let witness: Arc<str> = Arc::from("parent only");
        let child = owned(!record, vec![V::Text(text.clone())]).expect("child");
        let parent = owned(record, vec![child.clone(), V::Text(witness.clone())]).expect("parent");
        let selected = if record {
            runtime::record_get(&parent, &V::Text(Arc::from("f0")))
        } else {
            runtime::list_at(&parent, &V::Integer(0))
        }
        .expect("cross-kind selection");
        match (&selected, &child) {
            (V::List(a), V::List(b)) => assert_eq!(a.as_slice().as_ptr(), b.as_slice().as_ptr()),
            (V::Record(a), V::Record(b)) => assert!(std::ptr::eq(a.as_map(), b.as_map())),
            _ => panic!("selected child kind"),
        }
        drop(parent);
        drop(child);
        assert_eq!(Arc::strong_count(&witness), 1);
        assert_eq!(Arc::strong_count(&text), 2);
        let expected = output(!record, vec![json!("owned é🙂")]);
        assert_eq!(
            runtime::encode_json(selected).expect("surviving child"),
            expected
        );
        assert_eq!(Arc::strong_count(&text), 1);
    }
}

#[test]
fn exact_output_typed_lookup_content_and_root_domain_policies() {
    for record in [false, true] {
        let leaf = collection(
            record,
            vec![
                integer(i64::MIN),
                integer(i64::MAX),
                json!({"kind":"string","value":"é🙂"}),
            ],
        );
        let entry = collection(record, vec![collection(!record, vec![leaf.clone()])]);
        let leaf_expected = output(record, vec![json!(i64::MIN), json!(i64::MAX), json!("é🙂")]);
        let expected = output(record, vec![output(!record, vec![leaf_expected])]);
        for result in both(entry.clone()) {
            assert_eq!(result.expect("exact output"), expected);
        }
        let mut lookup = entry.clone();
        for kind in [record, !record, record] {
            lookup = binary(
                if kind { "recordIndex" } else { "listIndex" },
                lookup,
                if kind {
                    json!({"kind":"string","value":"f0"})
                } else {
                    integer(0)
                },
            );
        }
        for result in both(lookup) {
            assert_eq!(result.expect("typed cross-index"), json!(i64::MIN));
        }
        for (kind, wanted) in [
            ("equal", json!(true)),
            ("notEqual", json!(false)),
            ("unify", expected),
        ] {
            for result in both(binary(kind, entry.clone(), entry.clone())) {
                assert_eq!(result.expect("content policy"), wanted);
            }
        }
        let changed = collection(record, vec![collection(!record, vec![integer(1)])]);
        for result in both(binary("equal", entry.clone(), changed.clone())) {
            assert_eq!(result.expect("different contents"), json!(false));
        }
        for result in both(binary("unify", entry.clone(), changed)) {
            assert!(matches!(
                (result, record),
                (Err(Error::RecordConflict), true) | (Err(Error::ListConflict), false)
            ));
        }
        for other in [integer(0), collection(!record, vec![])] {
            for result in both(binary("equal", entry.clone(), other.clone())) {
                assert_eq!(result.expect("distinct root domains"), json!(false));
            }
            for result in both(binary("unify", entry.clone(), other)) {
                assert!(matches!(result, Err(Error::TypeConflict)));
            }
        }
        for (kind, scalar, wanted) in [("logicalAnd", "false", false), ("logicalOr", "true", true)]
        {
            for result in both(binary(kind, json!({"kind":scalar}), entry.clone())) {
                assert_eq!(result.expect("concrete collection RHS"), json!(wanted));
            }
        }
    }
}

#[test]
fn compiled_controls_distinguish_cross_homogeneous_and_scalar_admission() {
    for record in [false, true] {
        for child_record in [false, true] {
            let child = owned(child_record, vec![]).expect("empty child");
            let node = collection(record, vec![integer(0)]);
            for profile in 0..4 {
                let mut ctx = EvaluationContext::default();
                let mut eval = |_: &J, _: &mut EvaluationContext| Ok(child.clone());
                let result = if record {
                    let drivers: [RecordDriver; 4] = [
                        records::execute,
                        reverse_records::execute,
                        homogeneous_records::execute,
                        scalar_records::execute,
                    ];
                    run_record(&node, "fields", &mut ctx, &mut eval, drivers[profile])
                } else {
                    let drivers: [ListDriver; 4] = [
                        lists::execute,
                        reverse_lists::execute,
                        homogeneous_lists::execute,
                        scalar_lists::execute,
                    ];
                    run_list(&node, "elements", &mut ctx, &mut eval, drivers[profile])
                };
                if profile < 2 || (profile == 2 && record == child_record) {
                    assert_eq!(
                        result.expect("admitted child"),
                        owned(record, vec![child.clone()]).expect("expected")
                    );
                } else {
                    let error = result.expect_err("historical admission");
                    assert!(matches!(
                        (&error, record),
                        (Error::RecordElement { .. }, true) | (Error::ListElement { .. }, false)
                    ));
                    let message = error.to_string();
                    assert!(message.ends_with("requires a concrete scalar or closed collection"));
                }
            }
        }
    }
}

#[test]
fn alternating_children_share_ast_depth_and_selected_evaluator_visits() {
    for record in [false, true] {
        for (wrappers, accepted) in [(61, true), (62, false)] {
            let mut entry = integer(0);
            for _ in 0..wrappers {
                entry = json!({"kind":"negate","operand":entry});
            }
            entry = collection(record, vec![collection(!record, vec![entry])]);
            for result in both(entry) {
                if accepted {
                    assert!(result.is_ok(), "64 active AST entries");
                } else {
                    assert!(matches!(result, Err(Error::EvaluationDepth)));
                }
            }
        }
        let mut spine = integer(0);
        for _ in 0..10 {
            spine = binary("add", spine, integer(99));
        }
        for extra in [false, true] {
            // Two 2047-visit subtrees plus two alternating collection roots = 4096.
            let mut children = vec![spine.clone(), spine.clone()];
            if extra {
                children.push(integer(0));
            }
            let entry = collection(record, vec![collection(!record, children)]);
            let input = program(entry);
            let scalar = left_twice::execute(&input);
            if extra {
                assert!(matches!(scalar, Err(Error::EvaluationVisits)));
            } else {
                assert!(matches!(scalar, Err(Error::IntegerOutput)));
            }
            for result in [
                execute_json(&input, &export("tests/valid/evaluation-left-twice")),
                left_twice::execute_json(&input),
            ] {
                if extra {
                    assert!(matches!(result, Err(Error::EvaluationVisits)));
                } else {
                    let expected = output(record, vec![output(!record, vec![json!(0); 2])]);
                    assert_eq!(
                        result.expect("4096 shared visits and selected evaluator"),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn raw_literal_codecs_still_reject_collection_payloads() {
    for raw in [json!([]), json!({})] {
        for entry in [
            json!({"kind":"list","value":[raw.clone()]}),
            json!({"kind":"record","fields":[{"name":"x","value":raw}]}),
        ] {
            for result in both(entry) {
                assert!(result.is_err(), "raw codecs remain scalar-only");
            }
        }
    }
}
