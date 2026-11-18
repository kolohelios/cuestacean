use std::path::Path;

use bootstrap_executor::{Error, execute_json, generated, parser};
use serde_json::{Value, json};

#[path = "../../generated/primary-call-high-minimum.rs"]
mod call_high;
#[path = "../../generated/conjunction-primary.rs"]
mod conjunction_primary;
#[path = "../../generated/pratt.rs"]
mod expression;
#[path = "../../generated/conjunction-primary-high-minimum.rs"]
mod high_minimum;
#[path = "../../generated/primary-no-conjunction.rs"]
mod old_primary;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/record-primary-high-minimum.rs"]
mod record_high;
#[path = "../../generated/record-primary.rs"]
mod record_primary;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> Value {
    json!({"kind":"integer","value":value})
}
fn conjunction(operands: Value) -> Value {
    json!({"kind":"conjunction","operands":operands})
}
fn program(entry: Value) -> Value {
    json!({"formatVersion":1,"entry":entry})
}
fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind,"left":left,"right":right})
}
fn parses(source: &str, grammar: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn selected(source: &str, grammar: &str, policy: &str) -> [Result<Value, parser::Error>; 2] {
    let default: parser::Driver = expression::execute;
    let mut drivers = [
        default,
        primary::execute,
        record_primary::execute,
        conjunction_primary::execute,
    ];
    let mut names = ["pratt", "primary", "record-primary", "conjunction-primary"];
    let (index, driver, name): (usize, parser::Driver, &str) = match policy {
        "high" => (3, high_minimum::execute, "conjunction-primary-high-minimum"),
        "old" => (1, old_primary::execute, "primary-no-conjunction"),
        "right" => (0, right_expression::execute, "pratt-right"),
        "call" => (1, call_high::execute, "primary-call-high-minimum"),
        "record" => (2, record_high::execute, "record-primary-high-minimum"),
        "default" => (3, conjunction_primary::execute, "conjunction-primary"),
        _ => panic!("unknown test policy"),
    };
    drivers[index] = driver;
    let alternate = format!("tests/valid/{name}");
    if policy != "default" {
        names[index] = &alternate;
    }
    [
        parser::parse_with_four_drivers(
            source, grammar, drivers[0], drivers[1], drivers[2], drivers[3],
        ),
        parser::parse_interpreted_with_four_drivers(
            source,
            grammar,
            &export(names[0]),
            &export(names[1]),
            &export(names[2]),
            &export(names[3]),
        ),
    ]
}
fn evaluations(tree: &Value) -> [Result<Value, Error>; 2] {
    [
        execute_json(&tree.to_string(), &export("evaluation")),
        generated::execute_json(&tree.to_string()),
    ]
}
fn source_error(result: Result<Value, parser::Error>, offset: usize, expected: &str) {
    let error = result.expect_err("source rejection");
    assert!(
        matches!(&error, parser::Error::Source { offset: actual, reason }
        if *actual == offset && reason == expected),
        "{error:?}"
    );
}

#[test]
fn direct_complete_trees_refinements_identity_and_closed_values() {
    let grammar = export("grammar");
    let list = json!({"kind":"listExpression","elements":[integer(7)]});
    let record = json!({"kind":"recordExpression","fields":[{"name":"x","value":list}]});
    let mixed = json!({"kind":"listExpression","elements":[{"kind":"null"},{"kind":"true"},integer(i64::MAX)]});
    let cases = json!([
        {"source":"and([1,1])+1", "entry":binary("add",conjunction(json!([integer(1),integer(1)])),integer(1)), "value":2},
        {"source":"and([[null,true,9223372036854775807],[null,true,9223372036854775807]])[0:3]", "entry":{"kind":"listSlice","operand":conjunction(json!([mixed.clone(),mixed])),"lower":integer(0),"upper":integer(3)}, "value":[null,true,i64::MAX]},
        {"source":"and([int,>=2,<=4,3,])", "entry":conjunction(json!([{"kind":"intType"},{"kind":"lowerBound","value":2},{"kind":"upperBound","value":4},integer(3)])), "value":3},
        {"source":"and([bool,_,true])", "entry":conjunction(json!([{"kind":"boolType"},{"kind":"top"},{"kind":"true"}])), "value":true},
        {"source":r#"and([string,"\u00e9\U0001F642"])"#, "entry":conjunction(json!([{"kind":"stringType"},{"kind":"string","value":"é🙂"}])), "value":"é🙂"},
        {"source":"and([_,null,null])", "entry":conjunction(json!([{"kind":"top"},{"kind":"null"},{"kind":"null"}])), "value":null},
        {"source":"and([>=-9223372036854775808,-9223372036854775808])", "entry":conjunction(json!([{"kind":"lowerBound","value":i64::MIN},integer(i64::MIN)])), "value":i64::MIN},
        {"source":"and([<=9223372036854775807,9223372036854775807])", "entry":conjunction(json!([{"kind":"upperBound","value":i64::MAX},integer(i64::MAX)])), "value":i64::MAX},
        {"source":"and([and([]),7])", "entry":conjunction(json!([conjunction(json!([])),integer(7)])), "value":7},
        {"source":"and([])&7", "entry":binary("unify",conjunction(json!([])),integer(7)), "value":7},
        {"source":"7&and([])", "entry":binary("unify",integer(7),conjunction(json!([]))), "value":7},
        {"source":"and([_,[7],[7]])[0]", "entry":binary("index",conjunction(json!([{"kind":"top"},list.clone(),list])),integer(0)), "value":7},
        {"source":r#"and([close({"x":[7]}),close({"x":[7]})]).x[0:1][0]"#, "entry":binary("index",json!({"kind":"listSlice","operand":{"kind":"select","operand":conjunction(json!([record.clone(),record])),"label":"x"},"lower":integer(0),"upper":integer(1)}),integer(0)), "value":7},
        {"source":"and( [ // first\n 1+2, 3, ] )", "entry":conjunction(json!([binary("add",integer(1),integer(2)),integer(3)])), "value":3}
    ]);
    for case in cases.as_array().expect("cases") {
        let source = case["source"].as_str().expect("source");
        for parsed in parses(source, &grammar)
            .into_iter()
            .chain(selected(source, &grammar, "default"))
        {
            let tree = parsed.expect(source);
            assert_eq!(tree, program(case["entry"].clone()), "{source}");
            for result in evaluations(&tree) {
                assert_eq!(result.expect(source), case["value"], "{source}");
            }
        }
    }
}

#[test]
fn incremental_conflicts_child_errors_and_incomplete_results() {
    let grammar = export("grammar");
    for (source, expected) in [
        ("and([])", Error::IncompleteValue),
        ("and([int])", Error::Incomplete),
        ("and([bool])", Error::IncompleteBool),
        ("and([string])", Error::IncompleteString),
        ("and([>=3,<=3])", Error::IncompleteBounds),
        ("and([1,2,1 div 0])", Error::Conflict { left: 1, right: 2 }),
        ("and([1,1 div 0,2])", Error::DivisionByZero),
        (
            "and([true,false,1 div 0])",
            Error::BooleanConflict {
                left: true,
                right: false,
            },
        ),
    ] {
        for parsed in parses(source, &grammar) {
            for result in evaluations(&parsed.expect("syntax precedes evaluation")) {
                let actual = result.expect_err(source);
                assert_eq!(
                    std::mem::discriminant(&actual),
                    std::mem::discriminant(&expected)
                );
                assert_eq!(actual.to_string(), expected.to_string(), "{source}");
            }
        }
    }
    for source in [
        "and([1,2,]) trailing",
        "and([1 div 0,]) trailing",
        "and([1,2,]) +",
        "and([1,2,,])",
    ] {
        for parsed in parses(source, &grammar) {
            assert!(
                matches!(parsed, Err(parser::Error::Source { .. })),
                "{source}"
            );
        }
    }
}

#[test]
fn literal_argument_delimiters_and_conservative_call_newlines() {
    let grammar = export("grammar");
    for (source, offset, reason) in [
        ("and()", 4, "expected opening bracket"),
        ("and(1)", 4, "expected opening bracket"),
        ("and(([1]))", 4, "expected opening bracket"),
        ("and([1 2])", 7, "expected comma or closing bracket"),
        ("and([1]", 7, "expected closing parenthesis"),
        ("and([1],2)", 7, "expected closing parenthesis"),
        ("and([1]+[1])", 7, "expected closing parenthesis"),
    ] {
        for parsed in parses(source, &grammar) {
            source_error(parsed, offset, reason);
        }
    }
    for source in [
        "and",
        "and([)",
        "and([,])",
        "and([1,,])",
        "and([1\n])",
        "and([1 // tail\n])",
        "and\n([])",
        "and // name\n([])",
        "and([1,...])",
        "and([1])/*no*/",
    ] {
        for parsed in parses(source, &grammar) {
            assert!(parsed.is_err(), "{source}");
        }
    }
}

#[test]
fn optional_constructor_metadata_eager_collisions_and_no_list_dependency() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    let original = json!({"token":"and","kind":"conjunction","operandsField":"operands"});
    assert_eq!(grammar["conjunction"], original);
    grammar["conjunction"] = json!({"token":"meet","kind":"customMeet","operandsField":"items"});
    for parsed in parses("meet([int,7])", &grammar.to_string()) {
        assert_eq!(
            parsed.expect("custom constructor"),
            program(json!({"kind":"customMeet","items":[{"kind":"intType"},integer(7)]}))
        );
    }
    grammar["conjunction"] = original.clone();
    let list = grammar
        .as_object_mut()
        .expect("object")
        .remove("list")
        .expect("list metadata");
    for parsed in parses("and([int,7])", &grammar.to_string()) {
        assert_eq!(
            parsed.expect("not a list AST"),
            program(conjunction(json!([{"kind":"intType"},integer(7)])))
        );
    }
    grammar["list"] = list;
    grammar
        .as_object_mut()
        .expect("object")
        .remove("conjunction");
    for parsed in parses("and([])", &grammar.to_string()) {
        assert!(parsed.is_err());
    }
    for (field, value) in [
        ("token", ""),
        ("token", "bad-token"),
        ("token", "int"),
        ("token", "len"),
        ("token", "close"),
        ("kind", ""),
        ("operandsField", ""),
        ("operandsField", "kind"),
    ] {
        grammar["conjunction"] = original.clone();
        grammar["conjunction"][field] = value.into();
        for parsed in parses("1", &grammar.to_string()) {
            assert!(matches!(
                parsed,
                Err(parser::Error::Grammar { .. } | parser::Error::Json { .. })
            ));
        }
    }
}

#[test]
fn source_budgets_remain_separate_from_json_capacity() {
    let grammar = export("grammar");
    let wide = |n| format!("and([{}])", vec!["1"; n].join(","));
    let deep = |n| format!("{}1{}", "and([".repeat(n), "])".repeat(n));
    let nodes = "expression exceeds 64 nodes";
    let levels = "expression exceeds 32 parsing levels";
    for (source, succeeds, reason) in [
        (wide(63), true, ""),
        (wide(64), false, nodes),
        (deep(32), true, ""),
        (deep(33), false, levels),
    ] {
        for parsed in parses(&source, &grammar)
            .into_iter()
            .chain(selected(&source, &grammar, "high"))
        {
            if succeeds {
                assert!(parsed.is_ok(), "{parsed:?}");
            } else {
                let error = parsed.expect_err("shared budget").to_string();
                assert!(error.contains(reason), "{error}");
            }
        }
    }
    let tree = program(conjunction(Value::Array(vec![integer(1); 64])));
    for result in evaluations(&tree) {
        assert_eq!(result.expect("independent JSON operand capacity"), json!(1));
    }
}

#[test]
fn selected_fourth_driver_reaches_every_nested_expression_context() {
    let grammar = export("grammar");
    for source in [
        "and([1+2])",
        "(and([1+2]))",
        "1+and([1+2])",
        "[and([1+2])]",
        "and([and([1+2])])",
        "close({\"x\":and([1+2])})",
        "len(and([1+2]))",
        "-(and([1+2]))",
        "[1][and([1+2])]",
        "[1][0:and([1+2])]",
        ">=(and([1+2]))",
    ] {
        assert!(parser::parse(source, &grammar).is_ok(), "{source}");
        for parsed in selected(source, &grammar, "high") {
            source_error(
                parsed,
                source.rfind('+').expect("operator"),
                "expected comma or closing bracket",
            );
        }
    }
    for source in ["and([])", "and([(1+2)])", "and([and([(1+2)])])"] {
        for parsed in selected(source, &grammar, "high") {
            assert_eq!(
                parsed.expect(source),
                parser::parse(source, &grammar).expect("default")
            );
        }
    }
    for source in [
        "and([])",
        "(and([]))",
        "1+and([])",
        "[and([])]",
        "close({\"x\":and([])})",
    ] {
        for parsed in selected(source, &grammar, "old") {
            source_error(parsed, source.find("and").expect("token"), "unknown atom");
        }
    }
}

#[test]
fn other_selected_engines_reach_conjunction_operands() {
    let grammar = export("grammar");
    let entry = json!({"kind":"conjunction","operands":[binary("add",integer(1),binary("add",integer(2),integer(3)))]});
    for parsed in selected("and([1+2+3])", &grammar, "right") {
        let tree = parsed.expect("selected expression");
        assert_eq!(tree, program(entry.clone()));
        for result in evaluations(&tree) {
            assert_eq!(result.expect("selected tree value"), json!(6));
        }
    }
    let call = "expected closing parenthesis";
    let record = "expected comma or closing brace";
    for (source, policy, reason) in [
        ("and([len(1+2)])", "call", call),
        ("and([close({\"x\":1+2})])", "record", record),
        ("and([and([len(1+2)])])", "call", call),
        ("and([and([close({\"x\":1+2})])])", "record", record),
    ] {
        for parsed in selected(source, &grammar, policy) {
            source_error(parsed, source.find('+').expect("operator"), reason);
        }
    }
}
