use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, pratt, runtime, unification};
use runtime::{ClosedList, Value as V};
use serde_json::{Value as J, json};

#[path = "../../generated/list-index-zero.rs"]
mod alternate;
#[path = "../../generated/pratt-index-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/pratt-no-index.rs"]
mod no_index;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("flat list"))
}
fn array(elements: J) -> J {
    json!({"kind":"listExpression","elements":elements})
}
fn evaluations(entry: J) -> [Result<J, Error>; 2] {
    let input = json!({"formatVersion":1,"entry":entry}).to_string();
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn parses(source: &str, grammar: &str) -> [Result<J, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn outcomes(left: &V, right: &V) -> [Result<V, Error>; 3] {
    [
        runtime::list_at(left, right),
        runtime::list_index(left.clone(), right.clone()),
        unification::interpret(&export("list-index"), left.clone(), right.clone()),
    ]
}
fn selected(source: &str, driver: parser::Driver, name: &str) -> [Result<J, parser::Error>; 2] {
    let grammar = export("grammar");
    [
        parser::parse_with_drivers(source, &grammar, driver, primary::execute),
        parser::parse_interpreted_with_primary(
            source,
            &grammar,
            &export(&format!("tests/valid/{name}")),
            &export("primary"),
        ),
    ]
}

#[test]
fn checked_kernel_policy_and_reference_cover_scalar_selection_and_ranges() {
    let values = vec![
        V::Null,
        V::Boolean(false),
        V::Integer(i64::MIN),
        V::Integer(i64::MAX),
        V::Text(Arc::from("é🙂")),
    ];
    let items = list(values.clone());
    for (index, expected) in values.into_iter().enumerate() {
        let index = V::Integer(i64::try_from(index).expect("small index"));
        for result in outcomes(&items, &index) {
            assert_eq!(result.expect("selected scalar"), expected);
        }
    }
    for (items, index) in [
        (list(vec![V::Null; 64]), 63),
        (list(vec![]), 0),
        (list(vec![V::Null; 64]), 64),
        (list(vec![V::Null]), -1),
        (list(vec![V::Null]), i64::MIN),
        (list(vec![V::Null]), i64::MAX),
    ] {
        let index_value = V::Integer(index);
        for result in outcomes(&items, &index_value) {
            if index == 63 {
                assert_eq!(result.expect("last of 64"), V::Null);
            } else {
                assert!(
                    matches!(result, Err(Error::IndexRange)),
                    "{index}: {result:?}"
                );
            }
        }
    }
    assert_eq!(Error::IndexRange.to_string(), "list index out of range");
    assert_eq!(
        Error::IndexOperands.to_string(),
        "indexing requires a concrete closed list and integer"
    );
}

#[test]
fn all_non_list_left_and_non_integer_right_domains_reject_without_coercion() {
    let domains = [
        V::Top,
        V::Null,
        V::Bool,
        V::Boolean(true),
        V::Int,
        V::Integer(0),
        V::String,
        V::Text(Arc::from("0")),
        V::Bounds {
            lower: None,
            upper: None,
        },
        V::Bounds {
            lower: Some(0),
            upper: Some(0),
        },
        list(vec![]),
    ];
    for left in &domains {
        for right in &domains {
            let valid_domains = matches!(left, V::List(_)) && matches!(right, V::Integer(_));
            for result in outcomes(left, right) {
                assert!(
                    if valid_domains {
                        matches!(result, Err(Error::IndexRange))
                    } else {
                        matches!(result, Err(Error::IndexOperands))
                    },
                    "{left:?}[{right:?}]: {result:?}"
                );
            }
        }
    }
}

#[test]
fn selected_text_shares_allocation_and_survives_parent_drop() {
    let text: Arc<str> = Arc::from("owned é🙂");
    let items = list(vec![V::Text(text.clone()), V::Integer(1)]);
    let selected = runtime::list_at(&items, &V::Integer(0)).expect("text");
    let V::Text(selected) = selected else {
        panic!("expected text")
    };
    assert!(Arc::ptr_eq(&text, &selected));
    drop(items);
    drop(text);
    assert_eq!(&*selected, "owned é🙂");
}

#[test]
fn ast_eager_children_precede_domain_and_range_guards_left_first() {
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("add", integer(i64::MAX), integer(1));
    for (left, right, expected) in [
        (zero.clone(), overflow.clone(), Error::DivisionByZero),
        (json!({"kind":"null"}), zero, Error::DivisionByZero),
        (json!({"kind":"list","value":[]}), overflow, Error::Overflow),
        (
            json!({"kind":"list","value":[1]}),
            integer(-1),
            Error::IndexRange,
        ),
    ] {
        for result in evaluations(binary("listIndex", left, right)) {
            assert_eq!(
                result.expect_err("eager error").to_string(),
                expected.to_string()
            );
        }
    }
}

#[test]
fn compiled_entry_name_and_arity_do_not_supply_indexing_semantics() {
    let policy = export("tests/valid/list-index-zero");
    for (left, right) in [
        (list(vec![V::Null]), V::Integer(0)),
        (V::Top, V::Text(Arc::from("bad"))),
    ] {
        assert_eq!(
            alternate::list_index(left.clone(), right.clone()).expect("alternate"),
            V::Integer(0)
        );
        assert_eq!(
            unification::interpret(&policy, left, right).expect("reference alternate"),
            V::Integer(0)
        );
    }
    assert!(unification::interpret_unary(&policy, V::Null).is_err());
    let count = json!({"formatVersion":1,"function":"listIndex","result":{"op":"if","condition":{"op":"compare","left":{"op":"listCount"},"right":{"op":"integer","value":2},"relation":"equal"},"then":{"op":"integer","value":17},"else":{"op":"integer","value":19}}}).to_string();
    unification::generate(&count).expect("left-only count endpoint");
    for right in [V::Top, V::Integer(-1), list(vec![])] {
        assert_eq!(
            unification::interpret(&count, list(vec![V::Null; 2]), right)
                .expect("count ignores right"),
            V::Integer(17)
        );
    }
    let alias = count.replace("listCount", "listCountEndpoint");
    assert!(
        unification::generate(&alias).is_err(),
        "internal serialization is not wire syntax"
    );
    let code = unification::generate(&export("list-index")).expect("compiled policy");
    assert!(code.contains("fn list_index("));
    assert!(code.contains("list_at("));
    let bad = json!({"formatVersion":1,"function":"listLength","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"list"},"then":{"op":"integer","value":0},"else":{"op":"listElement"}}}).to_string();
    assert!(unification::generate(&bad).is_err());
    assert!(unification::interpret_unary(&bad, list(vec![])).is_err());
}

#[test]
fn independent_postfix_trees_precedence_minimum_zero_and_left_associative_chains() {
    let items = array(json!([integer(10), integer(20)]));
    let index = |right| binary("index", items.clone(), right);
    for (source, expected, value) in [
        (
            "[10,20][0+1]",
            index(binary("add", integer(0), integer(1))),
            json!(20),
        ),
        (
            "-[10,20][0]",
            json!({"kind":"negate","operand":index(integer(0))}),
            json!(-10),
        ),
        (
            "1+[10,20][1]*2",
            binary(
                "add",
                integer(1),
                binary("multiply", index(integer(1)), integer(2)),
            ),
            json!(41),
        ),
        ("([10,20]) [ // index\n1]", index(integer(1)), json!(20)),
        (
            "[10,20][0][0]",
            binary("index", index(integer(0)), integer(0)),
            J::Null,
        ),
    ] {
        for parsed in parses(source, &export("grammar")) {
            let tree = parsed.expect(source);
            assert_eq!(
                tree,
                json!({"formatVersion":1,"entry":expected}),
                "{source}"
            );
            for result in evaluations(tree["entry"].clone()) {
                if source.ends_with("[0][0]") {
                    assert!(matches!(result, Err(Error::CollectionIndexOperands)));
                } else {
                    assert_eq!(result.expect(source), value, "{source}");
                }
            }
        }
    }
    for source in ["[][0]", "[1][-1]"] {
        for parsed in parses(source, &export("grammar")) {
            for result in
                evaluations(parsed.expect("range errors are not syntax errors")["entry"].clone())
            {
                assert!(matches!(result, Err(Error::IndexRange)));
            }
        }
    }
}

#[test]
fn optional_index_metadata_disables_syntax_and_validates_even_when_unused() {
    let mut grammar: J = serde_json::from_str(&export("grammar")).expect("grammar");
    assert_eq!(
        grammar["index"],
        json!({"kind":"index","leftField":"left","rightField":"right"})
    );
    grammar["index"] = json!({"kind":"customIndex","leftField":"base","rightField":"offset"});
    for parsed in parses("[1][0]", &grammar.to_string()) {
        assert_eq!(
            parsed.expect("custom AST"),
            json!({"formatVersion":1,"entry":{"kind":"customIndex","base":array(json!([integer(1)])),"offset":integer(0)}})
        );
    }
    grammar.as_object_mut().expect("object").remove("index");
    for parsed in parses("[1][0]", &grammar.to_string()) {
        assert!(parsed.is_err());
    }
    for parsed in parses("[1+2]", &grammar.to_string()) {
        assert!(parsed.is_ok());
    }
    for bad in [
        json!({"kind":"","leftField":"left","rightField":"right"}),
        json!({"kind":"x","leftField":"","rightField":"right"}),
        json!({"kind":"x","leftField":"left","rightField":""}),
        json!({"kind":"x","leftField":"same","rightField":"same"}),
        json!({"kind":"x","leftField":"kind","rightField":"right"}),
        json!({"kind":"x","leftField":"left","rightField":"kind"}),
        json!({"kind":"x","leftField":"left"}),
        json!({"kind":"x","leftField":"left","rightField":"right","extra":true}),
    ] {
        grammar["index"] = bad;
        for parsed in parses("1", &grammar.to_string()) {
            assert!(matches!(
                parsed,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
}

#[test]
fn selected_expression_drivers_propagate_into_index_groups_calls_and_right_sides() {
    let items = array(json!([integer(10), integer(20)]));
    let rhs = binary(
        "subtract",
        integer(1),
        binary("subtract", integer(1), integer(1)),
    );
    let index = binary("index", items, rhs);
    for source in ["[10,20][1-1-1]", "([10,20][1-1-1])", "1+[10,20][1-1-1]"] {
        let (expected, value) = if source.starts_with("1+") {
            (binary("add", integer(1), index.clone()), 21)
        } else {
            (index.clone(), 20)
        };
        for parsed in selected(source, right_expression::execute, "pratt-right") {
            let tree = parsed.expect(source);
            assert_eq!(tree, json!({"formatVersion":1,"entry":expected}));
            for result in evaluations(tree["entry"].clone()) {
                assert_eq!(result.expect(source), json!(value));
            }
        }
    }
    for source in [
        "[1][0]",
        "([1][0])",
        "1+[1][0]",
        "len([1][0])",
        "[1][[0][0]]",
    ] {
        for parsed in selected(source, no_index::execute, "pratt-no-index") {
            assert!(parsed.is_err(), "{source}");
        }
    }
    for parsed in selected(
        "[10,20][0+1]",
        high_minimum::execute,
        "pratt-index-high-minimum",
    ) {
        assert!(
            matches!(parsed, Err(parser::Error::Source { offset: 9, reason }) if reason == "expected closing bracket")
        );
    }
    for source in ["[10,20][(0+1)]", "[10,20][1]"] {
        for parsed in selected(source, high_minimum::execute, "pratt-index-high-minimum") {
            assert!(parsed.is_ok());
        }
    }
}

#[test]
fn closing_scanner_and_operand_newline_policy_remain_cue_owned() {
    for (source, offset) in [("[1][0", 5), ("[1][0,]", 5), ("[1][0 1]", 6)] {
        for parsed in parses(source, &export("grammar")) {
            assert!(
                matches!(parsed, Err(parser::Error::Source { offset: actual, reason })
                if actual == offset && reason == "expected closing bracket"),
                "{source}"
            );
        }
    }
    for source in [
        "[1][]",
        "[1]\n[0]",
        "[1][0\n]",
        "[1][0 // comment\n]",
        "[1][0]x",
    ] {
        let [compiled, interpreted] = parses(source, &export("grammar"));
        assert_eq!(
            compiled.expect_err(source).to_string(),
            interpreted.expect_err(source).to_string()
        );
    }
}

#[test]
fn pending_postfix_returns_and_source_resource_boundaries() {
    let grammar = export("grammar");
    for suffix in [
        json!([]),
        json!([{"op":"advanceIndex"}]),
        json!([{"op":"advanceIndex"},{"op":"parseIndex","minimum":{"op":"integer","value":0}}]),
    ] {
        let mut body = vec![
            json!({"op":"primary"}),
            json!({"op":"afterOperand"}),
            json!({"op":"scanIndex"}),
        ];
        let mut pending = suffix.as_array().expect("actions").clone();
        pending.push(json!({"op":"return"}));
        body.push(json!({"op":"if","condition":{"op":"hasIndex"},"then":pending}));
        body.push(json!({"op":"return"}));
        let control = json!({"formatVersion":1,"body":body}).to_string();
        pratt::generate(&control).expect("well-typed return");
        let result = parser::parse_interpreted("[7][0]", &grammar, &control);
        assert!(
            matches!(
                result,
                Err(parser::Error::ControlState {
                    reason: "unfinished index at frame return"
                })
            ),
            "{control}: {result:?}"
        );
    }
    for (count, succeeds) in [(31, true), (32, false)] {
        let chain = format!("[0]{}", "[0]".repeat(count));
        let nested = format!("[0][{}0{}]", "(".repeat(count), ")".repeat(count));
        for (source, reason) in [
            (chain, "expression exceeds 64 nodes"),
            (nested, "expression exceeds 32 parsing levels"),
        ] {
            for result in parses(&source, &grammar) {
                if succeeds {
                    assert!(result.is_ok(), "{source}: {result:?}");
                } else {
                    assert!(
                        result
                            .expect_err("source budget")
                            .to_string()
                            .contains(reason)
                    );
                }
            }
        }
    }
}

#[test]
fn index_dsl_family_validation_visits_unused_branches() {
    for action in [
        json!({"op":"scanIndex"}),
        json!({"op":"advanceIndex"}),
        json!({"op":"parseIndex","minimum":{"op":"integer","value":0}}),
        json!({"op":"closeIndex"}),
    ] {
        let control = json!({"formatVersion":1,"target":"list","body":[{"op":"if","condition":{"op":"boolean","value":false},"then":[action]}]}).to_string();
        assert!(pratt::generate(&control).is_err());
    }
    let control = json!({"formatVersion":1,"target":"list","body":[{"op":"if","condition":{"op":"hasIndex"},"then":[]}]}).to_string();
    assert!(pratt::generate(&control).is_err());
}

#[test]
fn cli_selected_scalar_and_failures_are_atomic() {
    let path = std::env::temp_dir().join(format!(
        "cuestacean-list-indexing-{}.cue",
        std::process::id()
    ));
    for (source, expected) in [
        ("[null,true,\"é🙂\"][2]", Some("\"é🙂\"\n")),
        ("[1][-1]", None),
        ("[1][0+1]", None),
        ("[1][0,]", None),
        ("[1][1 div 0]", None),
    ] {
        std::fs::write(&path, source).expect("source file");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        if let Some(expected) = expected {
            assert!(output.status.success());
            assert_eq!(String::from_utf8(output.stdout).expect("JSON"), expected);
            assert!(output.stderr.is_empty());
        } else {
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty(), "{source}: {output:?}");
            assert!(!output.stderr.is_empty());
        }
    }
    std::fs::remove_file(path).expect("remove source");
}
