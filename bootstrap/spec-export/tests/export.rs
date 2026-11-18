use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}
fn expected() -> Value {
    json!({"formatVersion":1,"parameters":[{"name":"kind","type":"string"}],
        "result":{"op":"object","fields":[{"key":{"op":"string","value":"kind"},
        "value":{"op":"argument","name":"kind"}}]}})
}
fn value(schema: &str, body: &str) -> Result<Value, Error> {
    export(schema, &format!("package p\n{body}\n"))
        .map(|s| serde_json::from_str(&s).expect("native JSON"))
}
fn oracle(schema: &str, body: &str) -> std::process::Output {
    let mut child = Command::new("cue")
        .args(["export", "-", "--out", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mandatory pinned upstream oracle");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(format!("{schema}\n{body}\n").as_bytes())
        .expect("bounded probe");
    child.wait_with_output().expect("oracle exit")
}
fn check(schema: &str, body: &str, expected: Value) {
    assert_eq!(
        value(schema, body).expect("native supported probe"),
        expected
    );
    let output = oracle(schema, body);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("exact oracle JSON"),
        expected
    );
}

#[test]
fn actual_atom_is_not_a_filename_specific_answer() {
    let schema = std::fs::read_to_string(root().join("ast-schema.cue")).expect("raw schema");
    let program = std::fs::read_to_string(root().join("ast-atom.cue")).expect("raw program");
    let output = export(&schema, &program).expect("real closure");
    assert_eq!(
        serde_json::from_str::<Value>(&output).expect("JSON"),
        expected()
    );
    assert_eq!(export(&schema, &program).expect("repeat"), output);
    assert_eq!(export(&program, &schema).expect("file order"), output);
    let renamed = program.replace("value: \"kind\"", "value: \"tag\"");
    let mut changed = expected();
    changed["result"]["fields"][0]["key"]["value"] = json!("tag");
    assert_eq!(
        serde_json::from_str::<Value>(&export(&schema, &renamed).expect("rename")).expect("JSON"),
        changed
    );
    let missing = program.replace(
        "op: \"argument\", name: \"kind\"",
        "op: \"argument\", name: \"missing\"",
    );
    let mut changed = expected();
    changed["result"]["fields"][0]["value"]["name"] = json!("missing");
    assert_eq!(
        serde_json::from_str::<Value>(&export(&schema, &missing).expect("argument is data"))
            .expect("JSON"),
        changed
    );
    for program in [
        program.replace("type: \"string\"", "type: \"bool\""),
        program.replace("#BuildProgram & {", "#BuildProgram & {extra:1,"),
        program.replace("op: \"argument\"", "op: \"unknown\""),
    ] {
        assert_eq!(export(&schema, &program), Err(Error::Conflict));
    }
    let incomplete = program.replacen("name: \"kind\"", "name: string", 1);
    assert_eq!(export(&schema, &incomplete), Err(Error::Incomplete));
}

#[test]
fn recursive_templates_are_fresh_instances_and_closed_recursively() {
    let schema = "package p\n#N: {tag: \"n\", name: string, children: [...#N]}\n";
    check(
        schema,
        "#N & {name:\"outer\",children:[{name:\"inner\",children:[]}]}",
        json!({"tag":"n","name":"outer","children":[{"tag":"n","name":"inner","children":[]}]}),
    );
    assert_eq!(
        value(schema, "#N & {name:\"x\",children:[{name:\"y\",extra:1}]}"),
        Err(Error::Conflict)
    );
    check("package p\n#D: {x:1}", "{\"#D\":#D}", json!({"#D":{"x":1}}));
    assert_eq!(
        value("package p\n#D:{nested:{x:1}}", "#D & {nested:{extra:1}}"),
        Err(Error::Conflict)
    );
}

#[test]
fn declaration_and_conjunct_order_do_not_change_meaning() {
    for schema in ["package p\n#A:#B\n#B:{x:1}", "package p\n#B:{x:1}\n#A:#B"] {
        for body in ["#A & {x:1}", "{x:1} & #A"] {
            check(schema, body, json!({"x":1}));
        }
    }
    check("package p", "{x:1,x:1}", json!({"x":1}));
    assert_eq!(value("package p", "{x:1,x:2}"), Err(Error::Conflict));
    for body in ["{x:string,y:1}&{y:2}", "{y:2}&{y:1,x:string}"] {
        assert_eq!(value("package p", body), Err(Error::Conflict));
        assert!(!oracle("package p", body).status.success());
    }
}

#[test]
fn disjunction_keeps_residual_readiness_not_just_concrete_candidates() {
    for body in ["int | 1", "1 | int", "1 | 2"] {
        assert_eq!(value("package p", body), Err(Error::Incomplete));
        assert!(!oracle("package p", body).status.success());
    }
    check("package p", "1|1", json!(1));
    check("package p", r#"{x:"a\/b"}"#, json!({"x":"a/b"}));
    for schema in [
        "package p\n#A: {x:int,tag:\"a\"}|{x:1,tag:\"b\"}",
        "package p\n#A: {tag:\"a\",x:int}|{tag:\"b\",x:1}",
    ] {
        check(schema, "#A & {tag:\"b\"}", json!({"tag":"b","x":1}));
    }
}

#[test]
fn open_list_projection_does_not_erase_item_constraints() {
    check("package p", "{a:[...string]}", json!({"a":[]}));
    check("package p", "[...int] & [1,2]", json!([1, 2]));
    assert_eq!(
        value("package p", "[...int] & [\"x\"]"),
        Err(Error::Conflict)
    );
    assert_eq!(value("package p", "[1]&[1,2]"), Err(Error::Conflict));
    check("package p", "string & !=\"\" & \"ok\"", json!("ok"));
    assert_eq!(
        value("package p", "string & !=\"\" & \"\""),
        Err(Error::Conflict)
    );
    assert_eq!(value("package p", "int & !=\"\""), Err(Error::Conflict));
}

#[test]
fn unsupported_range_syntax_and_resource_failures_are_explicit() {
    for body in ["9223372036854775808", "-9223372036854775809"] {
        assert_eq!(value("package p", body), Err(Error::Range));
        assert!(
            oracle("package p", body).status.success(),
            "CUE integers are wider"
        );
    }
    for body in ["9223372036854775807", "-9223372036854775808"] {
        check(
            "package p",
            body,
            serde_json::from_str(body).expect("exact integer"),
        );
    }
    assert!(matches!(
        value("package p", "#Missing"),
        Err(Error::Unknown(_))
    ));
    assert!(matches!(
        value("package p\n#A:#A", "#A"),
        Err(Error::Unsupported(_))
    ));
    for body in ["*1|int", "{a?:1}", "{_hidden:1}", "1+1", "[1,...int]"] {
        assert!(
            matches!(
                value("package p", body),
                Err(Error::Unsupported(_) | Error::Syntax(_))
            ),
            "{body}"
        );
    }
    assert!(matches!(
        value("package p", "{a:1 b:2}"),
        Err(Error::Syntax(_))
    ));
    assert!(matches!(
        value("package p\n#A:1\n#A:1", "#A"),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        export_with_limit("package p", "package p\n1", 0),
        Err(Error::Limit(_))
    ));
    assert!(matches!(
        export_with_limit("package p", "package p\n1", 4097),
        Err(Error::Limit(_))
    ));
    assert_eq!(
        value("package p", "1"),
        Ok(json!(1)),
        "failure must not poison next solve"
    );
    assert!(matches!(
        value("package p", &format!("{{x:[{}]}}", vec!["1"; 65].join(","))),
        Err(Error::Limit(_))
    ));
    assert!(matches!(
        value(
            "package p",
            &format!("{}1{}", "(".repeat(33), ")".repeat(33))
        ),
        Err(Error::Limit(_))
    ));
    assert!(matches!(
        export(&" ".repeat(65537), "package p\n1"),
        Err(Error::Limit(_))
    ));
}

#[test]
fn independent_copy_encoding_depth_and_branch_limits() {
    let schema = format!("package p\n#S:{}", json!("a".repeat(1024)));
    let list = |n| format!("package p\n[{}]", vec!["#S"; n].join(","));
    export(&schema, &list(63)).expect("below copied and encoded caps");
    assert_eq!(
        export(&schema, &list(64)),
        Err(Error::Limit("encoded bytes")),
        "exactly 65536 copied bytes succeeds until encoding adds punctuation"
    );
    let larger = schema.replace(&"a".repeat(1024), &"a".repeat(1025));
    assert_eq!(
        export(&larger, &list(64)),
        Err(Error::Limit("copied strings"))
    );
    let escaped = format!("package p\n#S:{}", json!("\n".repeat(600)));
    assert_eq!(
        export(&escaped, &list(60)),
        Err(Error::Limit("encoded bytes"))
    );
    let trials = format!("package p\n#S:{}", json!("x".repeat(40000)));
    assert_eq!(
        export(&trials, "package p\n#S|#S"),
        Err(Error::Limit("copied strings")),
        "discarded/equal alternatives share the same copy allowance"
    );
    let chain = |n| {
        let mut schema = String::from("package p\n");
        for i in 0..n {
            schema.push_str(&format!("#N{i}:[#N{}]\n", i + 1));
        }
        schema.push_str(&format!("#N{n}:1\n"));
        schema
    };
    export(&chain(31), "package p\n#N0").expect("32 output levels inclusive");
    assert_eq!(
        export(&chain(32), "package p\n#N0"),
        Err(Error::Limit("output depth"))
    );
    let baseline = (1..256)
        .find(|work| export_with_limit("package p", "package p\n1", *work).is_ok())
        .expect("single cost");
    assert_eq!(
        export_with_limit("package p", "package p\n1|1", baseline),
        Err(Error::Limit("evaluation steps"))
    );
}

#[test]
fn label_definition_and_explicit_container_limits() {
    for label in ["a".repeat(64), "é".repeat(32)] {
        value("package p", &format!("{{{}:1}}", json!(label))).expect("64 label bytes");
    }
    assert_eq!(
        value("package p", &format!("{{{}:1}}", json!("a".repeat(65)))),
        Err(Error::Limit("label bytes"))
    );
    let name = format!("#{}", "d".repeat(63));
    value(&format!("package p\n{name}:1"), &name).expect("definition bound includes #");
    let name = format!("#{}", "d".repeat(64));
    assert_eq!(
        value(&format!("package p\n{name}:1"), &name),
        Err(Error::Limit("label bytes"))
    );
    let definitions = |n| {
        format!(
            "package p\n{}",
            (0..n).map(|i| format!("#D{i}:1\n")).collect::<String>()
        )
    };
    value(&definitions(64), "#D0").expect("64 definitions inclusive");
    assert_eq!(
        value(&definitions(65), "#D0"),
        Err(Error::Limit("definitions"))
    );
    let fields = |start, n| {
        format!(
            "{{{}}}",
            (start..start + n)
                .map(|i| format!("f{i}:1"))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    value("package p", &fields(0, 64)).expect("64 explicit fields");
    assert_eq!(
        value("package p", &fields(0, 65)),
        Err(Error::Limit("container items"))
    );
    let merged = value(
        "package p",
        &format!("{} & {}", fields(0, 64), fields(64, 64)),
    )
    .expect("merged not explicit width");
    assert_eq!(merged.as_object().expect("object").len(), 128);
}

#[test]
fn cli_publishes_nothing_on_semantic_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
        .arg(root().join("ast-schema.cue"))
        .arg(root().join("tests/invalid/ast-operation.cue"))
        .output()
        .expect("native CLI");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
