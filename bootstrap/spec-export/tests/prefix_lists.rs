use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

// Keep file scopes intact, and require a fresh public pinned Go-CUE process.
// Expected trees below are authored independently, never captured from the oracle.
struct Sources(PathBuf);
impl Sources {
    fn new(first: &str, second: &str, renamed: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "cuestacean-prefix-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("unique probe directory");
        let sources = Self(directory);
        let names = Self::names(renamed);
        fs::write(sources.0.join(names[0]), first).expect("first raw source");
        fs::write(sources.0.join(names[1]), second).expect("second raw source");
        sources
    }
    fn names(renamed: bool) -> [&'static str; 2] {
        if renamed {
            ["unrelated-z.cue", "unrelated-a.cue"]
        } else {
            ["schema.cue", "program.cue"]
        }
    }
    fn run(&self, oracle: bool, renamed: bool, reversed: bool) -> Output {
        let mut command = Command::new(if oracle {
            "cue"
        } else {
            env!("CARGO_BIN_EXE_cuestacean-spec-export")
        });
        if oracle {
            command.arg("export");
        } else {
            command.env_clear().env("PATH", "/nonexistent");
        }
        let mut names = Self::names(renamed);
        if reversed {
            names.reverse();
        }
        command
            .current_dir(&self.0)
            .args(names)
            .output()
            .expect("mandatory fresh process")
    }
}
impl Drop for Sources {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove probe sources");
    }
}
fn source(body: &str) -> String {
    format!("package p\n{body}\n")
}
fn success(output: Output, expected: &Value) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).expect("complete exact JSON"),
        *expected
    );
}
// A definitions-only CUE file contributes a struct root, not top. Keep the
// oracle and native raw inputs identical by putting non-struct probes in a field.
fn data(body: &str) -> String {
    if body.starts_with('{') || body.starts_with("let ") {
        body.into()
    } else {
        format!("{{value:{body}}}")
    }
}
fn check(schema: &str, body: &str, expected: Value) {
    let expected = if body.starts_with('{') || body.starts_with("let ") {
        expected
    } else {
        json!({"value":expected})
    };
    let (schema, body) = (source(schema), source(&data(body)));
    let native = export(&schema, &body).expect("admitted complete value");
    assert_eq!(
        serde_json::from_str::<Value>(&native).expect("native JSON"),
        expected
    );
    assert_eq!(
        export(&body, &schema).expect("reversed sources"),
        native,
        "file order"
    );
    assert_eq!(
        export(&schema, &body).expect("repeated export"),
        native,
        "repeat"
    );
    for renamed in [false, true] {
        let files = Sources::new(&schema, &body, renamed);
        for reversed in [false, true] {
            success(files.run(true, renamed, reversed), &expected);
            let cli = files.run(false, renamed, reversed);
            assert_eq!(cli.stdout, native.as_bytes(), "canonical CLI bytes");
            success(cli, &expected);
        }
    }
}
fn no_json(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "failure must publish no JSON");
    assert!(!output.stderr.is_empty(), "failure needs a diagnostic");
}
fn semantic_rejection(schema: &str, body: &str, expected: Error) {
    let (schema, body) = (source(schema), source(&data(body)));
    for (first, second) in [(&schema, &body), (&body, &schema)] {
        assert_eq!(export(first, second), Err(expected.clone()), "{body}");
    }
    let files = Sources::new(&schema, &body, false);
    no_json(&files.run(false, false, false));
    let oracle = files.run(true, false, false);
    no_json(&oracle);
    let diagnostic = String::from_utf8_lossy(&oracle.stderr);
    assert!(
        !diagnostic.contains("not found") && !diagnostic.contains("undefined"),
        "missing references are not semantic evidence: {diagnostic}"
    );
}

#[test]
fn booleans_are_distinct_and_disjunctions_retain_readiness() {
    check(
        "",
        "{yes:true,no:false,typed:bool & true}",
        json!({"yes":true,"no":false,"typed":true}),
    );
    for body in ["true|true", "(true|false)&true", "true&(false|true)"] {
        check("", body, json!(true));
    }
    for body in ["bool", "bool|true", "true|bool", "true|false", "false|true"] {
        semantic_rejection("", body, Error::Incomplete);
    }
    for body in [
        "true&false",
        "bool&1",
        "bool&0",
        "bool&\"true\"",
        "bool&{}",
        "bool&[]",
        "int&true",
    ] {
        semantic_rejection("", body, Error::Conflict);
    }
    for body in ["{a:bool,z:true&false}", "{z:true&false,a:bool}"] {
        semantic_rejection("", body, Error::Conflict);
    }
}

#[test]
fn bool_uses_lexical_capture_shadowing_and_exact_initializer_exclusions() {
    check("", "let bool=1\n{v:bool}", json!({"v":1}));
    check("", "let bool=bool & true\n{v:bool}", json!({"v":true}));
    semantic_rejection("", "let bool=bool\n{v:bool}", Error::Incomplete);
    check(
        "",
        "let bool=false\n{v:{let bool=bool, value:bool}}",
        json!({"v":{"value":false}}),
    );
    check(
        "",
        "let bool={bool:true}\n{v:bool}",
        json!({"v":{"bool":true}}),
    );
    check(
        "",
        "let bool=true\nlet T={v:bool}\n{r:{let bool=false, captured:T, local:bool}}",
        json!({"r":{"captured":{"v":true},"local":false}}),
    );
    check(
        "",
        "let bool=true\nlet x=false\n{r:{let bool={let x={a:bool,b:x}, v:x}, v:bool}}",
        json!({"r":{"v":{"v":{"a":true,"b":false}}}}),
    );
    check(
        "let bool=true\n#D:{v:bool}",
        "let bool=false\n{a:#D,b:bool}",
        json!({"a":{"v":true},"b":false}),
    );
    check(
        "",
        "let bool=true\n{\"bool\":false,v:bool}",
        json!({"bool":false,"v":true}),
    );
    // CUE supports field references, but native lexical admission must not
    // accidentally fall through to the predeclared bool or an outer binding.
    for body in [
        "{bool:true,v:bool}",
        "let bool={bool:true,v:bool}\n{r:bool}",
    ] {
        let (schema, program) = (source(""), source(body));
        assert_eq!(
            export(&schema, &program),
            Err(Error::Unsupported("field value reference"))
        );
        let files = Sources::new(&schema, &program, false);
        success(
            files.run(true, false, false),
            &if body.starts_with("let") {
                json!({"r":{"bool":true,"v":true}})
            } else {
                json!({"bool":true,"v":true})
            },
        );
        no_json(&files.run(false, false, false));
    }
    for body in ["let true=1\n{v:true}", "let false=1\n{v:false}"] {
        semantic_rejection("", body, Error::Unsupported("binding name"));
    }
}

#[test]
fn prefixes_are_minima_not_fixed_lengths_and_all_overlaps_apply() {
    for body in ["[1,...int]", "[1,...int]&[...int]", "[...int]&[1,...int]"] {
        check("", body, json!([1]));
    }
    for body in [
        "[int,...int]&[1,2]",
        "[1,2]&[int,...int]",
        "[1,...int]&[1,2,...int]",
        "[1,2,...int]&[1,...int]",
    ] {
        check("", body, json!([1, 2]));
    }
    check("", "[int,...string]&[1,\"x\",...string]", json!([1, "x"]));
    check(
        "",
        "[true,\"x\",1,...bool]&[...bool|string|int]",
        json!([true, "x", 1]),
    );
    check("", "[...int]&[...string]", json!([]));
    check("", "[...bool]&[]", json!([]));
    for body in [
        "[int,...int]",
        "[bool,...bool]",
        "[1,int,...int]",
        "[int,...int]&[...int]",
    ] {
        semantic_rejection("", body, Error::Incomplete);
    }
    for body in [
        "[1,...int]&[]",
        "[]&[1,...int]",
        "[1,2,...int]&[1]",
        "[1]&[1,2,...int]",
        "[1,...int]&[1,\"x\"]",
        "[1,\"x\"]&[1,...int]",
        "[1,...int]&[1,\"x\",...string]",
        "[...string]&[1,...int]",
        "[1,...int]&[2,...int]",
        "[1,...int]&[1]&[1,2]",
        "[int,1,...int]&[int,2,...int]",
        "[int,2,...int]&[int,1,...int]",
    ] {
        semantic_rejection("", body, Error::Conflict);
    }
    for body in ["([int,...int]|[1,...int])", "([1,...int]|[int,...int])"] {
        semantic_rejection("", body, Error::Incomplete);
    }
    check("", "[1,...int]|[1,...int]", json!([1]));
}

#[test]
fn recursive_closedness_and_independent_prefix_and_tail_instances() {
    let schema = "#N:{value:int,children:[...#N]}\n#L:[#N,...#N]";
    check(
        schema,
        "{a:#L&[{value:1}],b:#L&[{value:2},{value:3,children:[{value:4}]}]}",
        json!({"a":[{"value":1,"children":[]}],"b":[{"value":2,"children":[]},{"value":3,"children":[{"value":4,"children":[]}]}]}),
    );
    for body in [
        "#L&[{value:1,extra:true}]",
        "#L&[{value:1},{value:2,extra:true}]",
        "#L&[{value:1},{value:2,children:[{value:3,extra:true}]}]",
    ] {
        semantic_rejection(schema, body, Error::Conflict);
    }
    check(
        "",
        "let T=[bool,...bool]\n{a:T&[true],b:T&[false,true]}",
        json!({"a":[true],"b":[false,true]}),
    );
    check(
        "let T={value:bool}\n#D:[T,...T]",
        "#D&[{value:true},{value:false}]",
        json!([{"value":true},{"value":false}]),
    );
    semantic_rejection(
        "let T={value:bool}\n#D:[T,...T]",
        "#D&[{value:true},{value:false,extra:1}]",
        Error::Conflict,
    );
    check(
        "",
        "let T={value:bool}\n{result:[T,...T]&[{value:true,extra:1},{value:false,extra:2}]}",
        json!({"result":[{"value":true,"extra":1},{"value":false,"extra":2}]}),
    );
}

#[test]
fn malformed_tails_and_boolean_inequality_are_admission_not_conflicts() {
    for body in [
        "[1,...int,2]",
        "[1,...int,...int]",
        "[1 ...int]",
        "[1,,...int]",
    ] {
        let (schema, program) = (source(""), source(&data(body)));
        assert!(
            matches!(export(&schema, &program), Err(Error::Syntax(_))),
            "{body}"
        );
        let files = Sources::new(&schema, &program, false);
        no_json(&files.run(false, false, false));
        no_json(&files.run(true, false, false));
    }
    // Untyped tails and a final comma are legal upstream, but deliberately
    // excluded here. Keep their native Syntax classification distinct from
    // malformed syntax that both tools reject.
    for body in ["[1,...]", "[...]", "[1,...int,]", "[...int,]"] {
        let (schema, program) = (source(""), source(&data(body)));
        assert!(
            matches!(export(&schema, &program), Err(Error::Syntax(_))),
            "{body}"
        );
        let files = Sources::new(&schema, &program, false);
        success(
            files.run(true, false, false),
            &if body.starts_with("[1") {
                json!({"value":[1]})
            } else {
                json!({"value":[]})
            },
        );
        no_json(&files.run(false, false, false));
    }
    let (schema, program) = (source(""), source(&data("!=true")));
    assert!(matches!(
        export(&schema, &program),
        Err(Error::Unsupported(_))
    ));
    let files = Sources::new(&schema, &program, false);
    no_json(&files.run(false, false, false));
    no_json(&files.run(true, false, false));
}

#[test]
fn prefix_width_and_independent_syntax_and_output_depth_caps() {
    let prefix = |n| format!("[{},...bool]", vec!["true"; n].join(","));
    check("", &prefix(64), json!(vec![true; 64]));
    let empty = source("");
    assert_eq!(
        export(&empty, &source(&prefix(65))),
        Err(Error::Limit("container items"))
    );
    // Count the enclosing data struct and leaf: 30 lists give 32 levels.
    // Direct nesting isolates syntax depth from output materialization.
    let nested = |n| format!("{}true{}", "[".repeat(n), ",...bool]".repeat(n));
    check(
        "",
        &nested(30),
        (0..30).fold(json!(true), |child, _| json!([child])),
    );
    assert_eq!(
        export(&empty, &source(&data(&nested(31)))),
        Err(Error::Limit("syntax depth"))
    );
    // Shallow definitions isolate demanded output depth from syntax depth.
    let chain = |n| {
        let mut schema = String::new();
        for i in 0..n {
            schema.push_str(&format!("#N{i}:[#N{},...bool]\n", i + 1));
        }
        schema.push_str(&format!("#N{n}:true"));
        schema
    };
    check(
        &chain(30),
        "#N0",
        (0..30).fold(json!(true), |child, _| json!([child])),
    );
    assert_eq!(
        export(&source(&chain(31)), &source(&data("#N0"))),
        Err(Error::Limit("output depth"))
    );
}

#[test]
fn prefix_tail_work_is_cumulative_across_items_bindings_and_branches() {
    let schema = source("");
    let program = |body: &str| source(&format!("let T=[bool,...bool]\n{}", data(body)));
    let once = program("T&[true]");
    let work = (1..4097)
        .find(|&n| export_with_limit(&schema, &once, n).is_ok())
        .expect("bounded single expansion");
    for body in [
        "T&[true,false]",
        "(T&[true])|(T&[true])",
        "{a:T&[true],b:T&[false]}",
    ] {
        assert_eq!(
            export_with_limit(&schema, &program(body), work),
            Err(Error::Limit("evaluation steps")),
            "{body}"
        );
    }
    let tail = source("[...bool]&[true]");
    let baseline = (1..4097)
        .find(|&n| export_with_limit(&source(""), &tail, n).is_ok())
        .expect("one tail constraint cost");
    assert_eq!(
        export_with_limit(&source(""), &source("[...bool]&[...bool]&[true]"), baseline),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        export_with_limit(&schema, &once, 0),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            &export_with_limit(&schema, &once, work).expect("recovery with fresh allowance")
        )
        .expect("recovery JSON"),
        json!({"value":[true]})
    );
    check(
        "",
        "let T=[bool,...bool]\n{value:T&[true]}",
        json!({"value":[true]}),
    );
}
