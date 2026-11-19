use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SCHEMA: &str = "package p\n";
const FLAT: &str = r#"*{tag:"a",n:1}|{tag:"b",n:_}"#;
fn program(root: &str) -> String {
    format!("package p\n{root}\n")
}
fn native(root: &str) -> Result<Value, Error> {
    export(SCHEMA, &program(root)).map(|s| serde_json::from_str(&s).expect("exact JSON"))
}

struct Files(PathBuf);
impl Files {
    fn new(a: &str, b: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-tagged-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(path.is_absolute());
        fs::create_dir(&path).expect("owned fresh probe directory");
        fs::create_dir(path.join("home")).expect("owned HOME");
        fs::write(path.join("unrelated-z.cue"), a).expect("first raw source");
        fs::write(path.join("unrelated-a.cue"), b).expect("second raw source");
        Self(path)
    }
    fn run(&self, operation: Option<&str>, reverse: bool) -> Output {
        let binary = if operation.is_some() {
            std::env::split_paths(&std::env::var_os("PATH").expect("pinned toolchain PATH"))
                .map(|p| p.join("cue"))
                .find(|p| p.is_file())
                .expect("mandatory pinned CUE CLI")
                .canonicalize()
                .expect("absolute provider")
        } else {
            PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
        };
        assert!(binary.is_absolute());
        fs::File::open(&binary).expect("readable mandatory provider");
        assert_ne!(
            fs::metadata(&binary)
                .expect("provider")
                .permissions()
                .mode()
                & 0o111,
            0
        );
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", self.0.join("home"));
        if let Some(operation) = operation {
            command.arg(operation);
        }
        let mut names = ["unrelated-z.cue", "unrelated-a.cue"];
        if reverse {
            names.reverse();
        }
        command
            .current_dir(&self.0)
            .args(names)
            .output()
            .expect("fresh mandatory process")
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("retire owned probe");
    }
}
fn no_json(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no partial JSON");
    assert!(!output.stderr.is_empty(), "failure diagnostic");
}
fn output_json(output: &Output, expected: &Value) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("exact i64 JSON"),
        *expected
    );
}
// Expectations are authored below, never captured from a provider. Oracle
// observations precede native assertions; same raw files, fresh cleared processes.
fn verify(a: &str, b: &str, expected: Result<Value, Error>) {
    let files = Files::new(a, b);
    for reverse in [false, true] {
        let oracle = files.run(Some("export"), reverse);
        match &expected {
            Ok(value) => output_json(&oracle, value),
            Err(error) => {
                no_json(&oracle);
                let evaluation = files.run(Some("eval"), reverse);
                match error {
                    Error::Incomplete => assert!(
                        evaluation.status.success(),
                        "{}",
                        String::from_utf8_lossy(&evaluation.stderr)
                    ),
                    Error::Conflict => assert!(!evaluation.status.success()),
                    _ => panic!("use distinct profile expectations"),
                }
            }
        }
        let (first, second) = if reverse { (b, a) } else { (a, b) };
        let result = export(first, second);
        assert_eq!(
            result
                .as_ref()
                .map(|s| serde_json::from_str::<Value>(s).expect("JSON"))
                .map_err(Clone::clone),
            expected,
            "{first}\n{second}"
        );
        assert_eq!(
            export(first, second),
            result,
            "fresh observation and input immutability"
        );
        let output = files.run(None, reverse);
        match &expected {
            Ok(value) => output_json(&output, value),
            Err(_) => no_json(&output),
        }
    }
}
fn check(root: &str, expected: Result<Value, Error>) {
    verify(SCHEMA, &program(root), expected);
}

#[test]
fn actual_unchanged_integer_instruction_data_has_all_three_native_paths() {
    let choice = include_str!("../../ast-integer-policy.cue")
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("integer: *")
                .map(|rest| format!("*{rest}"))
        })
        .expect("the real raw integer alternative, both branches intact");
    let expected = json!({"integer":{"op":"concat","parts":[
        {"op":"text","value":"Value::from("},
        {"op":"signed","field":"value"},
        {"op":"text","value":"i64)"}
    ]}});
    for (context, result) in [
        ("{}", Ok(expected)),
        (
            r#"{integer:{op:"text",value:"x"}}"#,
            Ok(json!({"integer":{"op":"text","value":"x"}})),
        ),
        (r#"{integer:{op:"text"}}"#, Err(Error::Incomplete)),
    ] {
        check(&format!("{{integer:({choice})}}&{context}"), result);
    }
}

#[test]
fn feasible_incomplete_preference_never_falls_back_to_ready_other() {
    for expression in [
        r#"{v:(*{tag:"a"}|{tag:"b",n:1})}&{v:{n:int}}"#,
        r#"{v:{n:int}}&{v:({n:1,tag:"b"}|*{tag:"a"})}"#,
    ] {
        check(expression, Err(Error::Incomplete));
    }
    check(
        &format!("{{v:({FLAT})}}&{{v:{{tag:\"b\"}}}}"),
        Err(Error::Incomplete),
    );
    check(
        &format!("{{v:({FLAT})}}&{{v:_}}"),
        Ok(json!({"v":{"tag":"a","n":1}})),
    );
    check(&format!("{{v:({FLAT}),other:_}}"), Err(Error::Incomplete));
}

#[test]
fn complete_context_removes_whole_branches_without_payload_mixing() {
    for (root, expected) in [
        (
            r#"{v:(*{tag:"a",data:{x:1,y:9}}|{tag:"b",data:{x:_}})}&{v:{data:{x:2}}}"#,
            json!({"v":{"tag":"b","data":{"x":2}}}),
        ),
        (
            r#"{v:(*{tag:"a",data:[1,2]}|{tag:"b",data:[_]})}&{v:{data:[3]}}"#,
            json!({"v":{"tag":"b","data":[3]}}),
        ),
        (
            r#"{v:(*{tag:"a",x:[{a:1}]}|{tag:"b",x:[{a:2}]})}&{v:{x:[{a:2}]}}"#,
            json!({"v":{"tag":"b","x":[{"a":2}]}}),
        ),
        (
            r#"{outer:{inner:(*{k:1,data:"a"}|{k:2,data:_})}}&{outer:{inner:{k:2,data:"b"}}}"#,
            json!({"outer":{"inner":{"k":2,"data":"b"}}}),
        ),
        (
            r#"{v:(*{tag:"a",n:1}|{tag:"b",n:int})&{tag:"b"}&{n:2}}"#,
            json!({"v":{"tag":"b","n":2}}),
        ),
        (
            r#"{v:(*{tag:"a",n:1}|{tag:"b",n:_}),v:{tag:"b"},v:{n:2}}"#,
            json!({"v":{"tag":"b","n":2}}),
        ),
    ] {
        check(root, Ok(expected));
    }
    for root in [
        format!("{{v:({FLAT})}}&{{v:{{tag:\"c\"}}}}"),
        format!("{{v:({FLAT})}}&{{v:{{n:true}}}}&{{v:{{n:int}}}}"),
        format!("{{v:({FLAT})}}&{{v:{{n:2}}}}&{{v:{{n:3}}}}"),
        format!("{{outer:{{v:({FLAT})}}}}&{{outer:2}}"),
        format!("{{v:({FLAT}),a:int,z:{{x:1}}}}&{{z:{{x:2}}}}"),
        format!("{{v:({FLAT}),z:{{a:int,b:{{x:1}}}},z:{{b:{{x:2}}}}}}"),
    ] {
        check(&root, Err(Error::Conflict));
    }
    check(&format!("{{v:({FLAT}),other:int}}"), Err(Error::Incomplete));
}

#[test]
fn top_and_generic_discriminants_have_no_opcode_or_shape_privileges() {
    for (payload, expected) in [
        ("7", json!(7)),
        ("true", json!(true)),
        (r#""text""#, json!("text")),
        ("{x:[1,true]}", json!({"x":[1,true]})),
        ("[1,2]", json!([1, 2])),
    ] {
        check(
            &format!("{{v:(*{{d:true,p:[]}}|{{d:false,p:_}})}}&{{v:{{d:false,p:{payload}}}}}"),
            Ok(json!({"v":{"d":false,"p":expected}})),
        );
    }
    for (left, right, expected) in [
        (
            "-9223372036854775808",
            "9223372036854775807",
            json!(i64::MAX),
        ),
        ("true", "17", json!(17)),
        (r#""left""#, "false", json!(false)),
    ] {
        check(
            &format!(
                "{{renamed:({{key:{right},body:_}}|*{{body:0,key:{left}}})}}&{{renamed:{{key:{right},body:8}}}}"
            ),
            Ok(json!({"renamed":{"key":expected,"body":8}})),
        );
    }
}

#[test]
fn file_conjunction_field_and_alternative_order_preserve_contextual_results() {
    for choice in [FLAT, r#"{n:_,tag:"b"}|*{n:1,tag:"a"}"#] {
        for (context, expected) in [
            (r#"{tag:"a"}"#, Ok(json!({"v":{"tag":"a","n":1}}))),
            (r#"{n:7,tag:"b"}"#, Ok(json!({"v":{"tag":"b","n":7}}))),
            (r#"{tag:"b"}"#, Err(Error::Incomplete)),
            (r#"{tag:"other"}"#, Err(Error::Conflict)),
        ] {
            let a = program(&format!("{{v:({choice})}}"));
            let b = program(&format!("{{v:{context}}}"));
            verify(&a, &b, expected.clone());
            check(&format!("{{v:{context}}}&{{v:({choice})}}"), expected);
        }
    }
}

#[test]
fn whole_document_guard_cannot_be_bypassed_by_unused_or_unselected_content() {
    check(r#"{v:(*1|2)}"#, Ok(json!({"v":1})));
    let base = format!("{{v:({FLAT})}}");
    for root in [
        format!("#Unused:{{x:1}}\n{base}"),
        format!("let X=1\n{{v:({FLAT}),used:X}}"),
        format!("let X=({FLAT})\n{{a:X,b:X}}"),
        format!("{{v:({FLAT}),other:{{let X=1\nused:X}}}}"),
        format!("{{v:({FLAT}),other:(1|1)}}"),
        format!("{{v:({FLAT}),other:[...int]}}"),
        format!("{{v:({FLAT}),other:(1&!=2)}}"),
        format!("{{v:[({FLAT})]}}"),
        format!("{{list:[{{v:({FLAT})}}]}}"),
        format!("({FLAT})"),
        r#"{v:(*{tag:"a",n:1}|{tag:"a",n:_})}"#.into(),
        r#"{v:(*{tag:"a",n:1,n:1}|{tag:"b",n:_})}"#.into(),
        r#"{v:(*{tag:"a",n:int}|{tag:"b",n:2})}&{v:{n:1}}"#.into(),
        r#"{v:(*{tag:"a"}|{tag:"b",n:[...int]})}"#.into(),
        r#"{v:(*{tag:"a"}|{tag:"b",n:(1|1)})}"#.into(),
        r#"{v:((*{tag:"a"}|{tag:"b"})|{tag:"c"})}"#.into(),
        r#"{v:(*{tag:"a"}|*{tag:"b"})}"#.into(),
        r#"{v:*( {tag:"a"}&{n:1} )|{tag:"b"}}"#.into(),
        r#"{v:*{tag:"a"}}"#.into(),
    ] {
        assert!(
            matches!(native(&root), Err(Error::Unsupported(_))),
            "{root}: {:?}",
            native(&root)
        );
        no_json(&Files::new(SCHEMA, &program(&root)).run(None, false));
    }
    for root in [
        r#"{v:(*{tag:"a"}|{tag:"b",n:missing})}"#,
        r#"{v:(*{tag:"a"}|{tag:"b",n:9223372036854775808})}"#,
    ] {
        let expected = if root.contains("missing") {
            Error::Unknown("missing".into())
        } else {
            Error::Range
        };
        assert_eq!(native(root), Err(expected));
        no_json(&Files::new(SCHEMA, &program(root)).run(None, false));
    }
    // These source forms were exclusions before #218; do not reinterpret them.
    for root in ["*", "{v:_|}", "{v:**{tag:\"a\"}}"] {
        assert!(matches!(
            native(root),
            Err(Error::Syntax(_) | Error::Unsupported(_))
        ));
    }
}

#[test]
fn ordinary_errors_literals_and_current_language_remain_unchanged() {
    assert_eq!(
        native("{v:_}"),
        Err(Error::Unsupported("value reference or type"))
    );
    assert_eq!(
        native("{v:_Unknown}"),
        Err(Error::Unknown("_Unknown".into()))
    );
    check(
        r#"{"*":"_",other:"a*b"} // * does not enable Top"#,
        Ok(json!({"*":"_","other":"a*b"})),
    );
    assert_eq!(
        native("{v:_,text:\"*\"}// *"),
        Err(Error::Unsupported("value reference or type"))
    );
    assert_eq!(native("let string=1\n{v:string}"), Ok(json!({"v":1})));
    assert_eq!(native("{v:[1,...int]}"), Ok(json!({"v":[1]})));
    assert_eq!(native("{v:1|2}"), Err(Error::Incomplete));
    // Distinguish valid excluded CUE from a native semantic contradiction.
    let root = r#"{v:(*{tag:"a",n:1}|{tag:"a",n:_})}"#;
    output_json(
        &Files::new(SCHEMA, &program(root)).run(Some("export"), false),
        &json!({"v":{"tag":"a","n":1}}),
    );
    assert!(matches!(native(root), Err(Error::Unsupported(_))));
}

fn required_work(root: &str, expected: &Value) -> usize {
    let source = program(root);
    let mut low = 0;
    let mut high = 4096;
    assert_eq!(native(root), Ok(expected.clone()));
    while low < high {
        let mid = low + (high - low) / 2;
        match export_with_limit(SCHEMA, &source, mid) {
            Ok(text) => {
                assert_eq!(
                    serde_json::from_str::<Value>(&text).expect("JSON"),
                    *expected
                );
                high = mid;
            }
            Err(Error::Limit("evaluation steps")) => low = mid + 1,
            other => panic!("unexpected lowered-work result {other:?}"),
        }
    }
    assert!(low > 0);
    assert_eq!(
        export_with_limit(SCHEMA, &source, low - 1),
        Err(Error::Limit("evaluation steps"))
    );
    assert!(export_with_limit(SCHEMA, &source, low).is_ok());
    assert!(export_with_limit(SCHEMA, &source, 4097).is_err());
    assert_eq!(native(root), Ok(expected.clone()), "post-error fresh solve");
    low
}
#[test]
fn both_feasibility_trials_and_repeated_demands_share_existing_work_limit() {
    let payload = (0..12)
        .map(|i| format!("f{i}:1"))
        .collect::<Vec<_>>()
        .join(",");
    let choice = format!(
        "*{{tag:\"a\",data:{{x:1,y:{{{payload}}}}}}}|{{tag:\"b\",data:{{x:2,y:{{{payload}}}}}}}"
    );
    let mut fields = serde_json::Map::new();
    for i in 0..12 {
        fields.insert(format!("f{i}"), json!(1));
    }
    let expected = json!({"v":{"tag":"a","data":{"x":1,"y":fields}}});
    let pruned = format!("{{v:({choice})}}&{{v:{{data:{{x:1}}}}}}");
    let live = format!("{{v:({choice})}}&{{v:{{data:{{x:int}}}}}}");
    let short = required_work(&pruned, &expected);
    let long = required_work(&live, &expected);
    assert!(
        long > short,
        "unselected feasible branch still consumes work: {long}/{short}"
    );
    for (root, expected) in [
        (
            "{v:(*{tag:\"a\",n:1}|{tag:\"b\",n:_})}",
            json!({"v":{"tag":"a","n":1}}),
        ),
        (
            "{v:(*{tag:\"a\",n:1}|{tag:\"b\",n:_})}&{v:{tag:\"b\",n:2}}",
            json!({"v":{"tag":"b","n":2}}),
        ),
    ] {
        required_work(root, &expected);
    }
    assert!(matches!(
        export(&" ".repeat(65537), &program("{}")),
        Err(Error::Limit("source bytes"))
    ));
    let wide = format!("{{v:({FLAT}),items:[{}]}}", vec!["1"; 65].join(","));
    assert!(matches!(
        native(&wide),
        Err(Error::Limit("container items"))
    ));
    let deep = format!("{{v:({FLAT}),x:{}1{}}}", "[".repeat(33), "]".repeat(33));
    assert!(matches!(native(&deep), Err(Error::Limit("syntax depth"))));
}
