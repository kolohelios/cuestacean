use bootstrap_executor::{generated, parser, scanner};

#[path = "../../generated/list-ellipsis-close.rs"]
mod close;
#[path = "../../generated/list-ellipsis.rs"]
mod marker;
use serde_json::{Value, json};

const NORMAL: &str = include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json"));
const VET: &str = include_str!(concat!(
    env!("CUESTACEAN_BOOTSTRAP_JSON"),
    "/vet-grammar.json"
));

fn fields(value: Value) -> Value {
    json!({"formatVersion":1,"entry":{"kind":"recordExpression","fields":[{"name":"x","value":value}]}})
}

fn typed(elements: Value, tail: Value) -> Value {
    json!({"kind":"typedListExpression","elements":elements,"tail":tail})
}

fn routes(source: &str) -> [Result<Value, parser::Error>; 2] {
    [
        parser::parse_vet_fields_file(source, VET),
        parser::parse_vet_fields_file_interpreted(source, VET),
    ]
}

#[test]
fn exact_schema_trees_without_open_list_evaluation() {
    let int = json!({"kind":"intType"});
    let bound = json!({"kind":"unify","left":int,"right":{"kind":"lowerBound","value":0}});
    for (source, expected) in [
        ("x: [...int]", typed(json!([]), int.clone())),
        (
            "x: [// café\r\n-9223372036854775808,\r\n... int & >=0]",
            typed(json!([{"kind":"integer","value":i64::MIN}]), bound),
        ),
        (
            "x: close({\"items\":[[...int],]})",
            json!({"kind":"recordExpression","fields":[{"name":"items","value":{
                "kind":"listExpression","elements":[typed(json!([]), int)]
            }}]}),
        ),
    ] {
        let expected = fields(expected);
        for result in routes(source) {
            let tree = result.expect(source);
            assert_eq!(tree, expected, "{source}");
            assert!(generated::execute_json(&tree.to_string()).is_err());
        }
    }
    let ordinary =
        fields(json!({"kind":"listExpression","elements":[{"kind":"integer","value":42}]}));
    for result in routes("x: [42,]") {
        assert_eq!(result.expect("fixed list"), ordinary);
    }
}

#[test]
fn profile_does_not_leak_and_malformed_markers_do_not_retry() {
    for grammar in [NORMAL, VET] {
        assert!(parser::parse("[...int]", grammar).is_err());
        assert!(parser::parse_file("x: [42,...int]", grammar).is_err());
        assert!(parser::parse_fields_file("x: [42,...int]", grammar).is_err());
        assert!(parser::parse_file_interpreted("x: [42,...int]", grammar).is_err());
    }
    assert!(parser::parse_vet_fields_file("x: [...int]", NORMAL).is_err());
    assert!(parser::parse_vet_fields_file_interpreted("x: [...int]", NORMAL).is_err());
    for source in [
        "[...int]",
        "close({\"x\":42})",
        "// no declaration\n",
        "x: [..int]",
        "x: [... ]",
        "x: [...int,]",
        "x: [...int,42]",
        "x: [...int,...int]",
        "x: [....int]",
        "x: [...int",
        "x: [...int\n]",
        "x: and([...int])",
    ] {
        let [compiled, interpreted] = routes(source);
        let compiled = compiled.expect_err(source);
        let interpreted = interpreted.expect_err(source);
        assert_eq!(compiled.to_string(), interpreted.to_string(), "{source}");
    }
}

#[test]
fn scanners_use_complete_spans_and_require_closure() {
    let program = include_str!(concat!(
        env!("CUESTACEAN_BOOTSTRAP_JSON"),
        "/list-ellipsis.json"
    ));
    for (source, start, end, flag) in [
        ("é...int", 2, 5, true),
        ("..", 0, 0, false),
        ("", 0, 0, false),
    ] {
        let expected = scanner::Scan { offset: end, flag };
        assert_eq!(marker::scan(source, start).expect(source), expected);
        assert_eq!(
            scanner::interpret(program, source, start).expect(source),
            expected
        );
    }
    let program = include_str!(concat!(
        env!("CUESTACEAN_BOOTSTRAP_JSON"),
        "/list-ellipsis-close.json"
    ));
    let expected = scanner::Scan {
        offset: 1,
        flag: false,
    };
    assert_eq!(close::scan("]", 0).expect("close"), expected);
    assert_eq!(
        scanner::interpret(program, "]", 0).expect("close"),
        expected
    );
    for source in [",]", "", "\n]"] {
        for result in [
            close::scan(source, 0),
            scanner::interpret(program, source, 0),
        ] {
            assert!(
                matches!(result, Err(scanner::Error::Rejected {offset: 0, reason})
                if reason == "expected closing bracket after list tail")
            );
        }
    }
}

#[test]
fn tail_is_charged_in_the_same_ast_budget_as_prefix_and_root() {
    for count in [61, 62] {
        let source = format!("x: [{}...int]", "0,".repeat(count));
        for result in routes(&source) {
            if count == 61 {
                let tree = result.expect("64 total nodes");
                assert_eq!(
                    tree["entry"]["fields"][0]["value"]["elements"]
                        .as_array()
                        .map(Vec::len),
                    Some(61)
                );
            } else {
                assert!(
                    result
                        .expect_err("65 total nodes")
                        .to_string()
                        .contains("exceeds 64 nodes")
                );
            }
        }
    }
}
