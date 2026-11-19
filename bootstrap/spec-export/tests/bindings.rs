use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

// Preserve input-file boundaries: concatenating these sources changes let scope.
struct Sources(PathBuf);
impl Sources {
    fn new(first: &str, second: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("unique probe directory");
        let sources = Self(path);
        std::fs::write(sources.0.join("first.cue"), first).expect("original first source");
        std::fs::write(sources.0.join("second.cue"), second).expect("original second source");
        sources
    }
    fn run(&self, executable: &str, oracle: bool) -> Output {
        let mut command = Command::new(executable);
        if oracle {
            command.arg("export");
        } else {
            command.env_clear().env("PATH", "/nonexistent");
        }
        command
            .arg(self.0.join("first.cue"))
            .arg(self.0.join("second.cue"))
            .output()
            .expect("probe process")
    }
    fn oracle(&self) -> Output {
        self.run("cue", true)
    }
}
impl Drop for Sources {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove probe sources");
    }
}
fn source(body: &str) -> String {
    format!("package p\n{body}\n")
}
fn check(first: &str, second: &str, expected: Value) {
    let (first, second) = (source(first), source(second));
    let native = export(&first, &second).expect("supported native binding");
    assert_eq!(
        serde_json::from_str::<Value>(&native).expect("native JSON"),
        expected
    );
    assert_eq!(
        export(&second, &first).expect("reversed sources"),
        native,
        "file order"
    );
    let cue = Sources::new(&first, &second).oracle();
    assert!(
        cue.status.success(),
        "{}",
        String::from_utf8_lossy(&cue.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&cue.stdout).expect("pinned CUE JSON"),
        expected
    );
}
fn reject(body: &str, expected: Error) {
    let (first, second) = (source(""), source(body));
    assert_eq!(export(&first, &second), Err(expected), "{body}");
    let files = Sources::new(&first, &second);
    assert!(!files.oracle().status.success(), "CUE must reject: {body}");
    let cli = files.run(env!("CARGO_BIN_EXE_cuestacean-spec-export"), false);
    assert!(!cli.status.success(), "native CLI must reject: {body}");
    assert!(cli.stdout.is_empty(), "no partial JSON: {body}");
}

#[test]
fn forward_lookup_initializer_exclusion_and_builtin_shadowing() {
    for declarations in ["let A=B\nlet B=1", "let B=1\nlet A=B"] {
        check("", &format!("{declarations}\n{{v:A}}"), json!({"v":1}));
    }
    check(
        "",
        "let x=1\n{v:{let x=x, value:x}}",
        json!({"v":{"value":1}}),
    );
    check("", "let string=1\n{v:string}", json!({"v":1}));
    check("", "let _x=1\n{v:_x}", json!({"v":1}));
    check("", "let bool=1\n{v:bool}", json!({"v":1}));
    check(
        "",
        "let a=1\nlet b=2\n{result:{let a={let b={x:a,y:b}, value:b}, value:a}}",
        json!({"result":{"value":{"value":{"x":1,"y":2}}}}),
    );
    assert_eq!(
        export(&source(""), &source("{v:bool}")),
        Err(Error::Incomplete)
    );
    let bool_incomplete = Sources::new(&source(""), &source("{v:bool}")).oracle();
    assert!(!bool_incomplete.status.success());
    assert!(bool_incomplete.stdout.is_empty());
    assert_eq!(
        export(&source(""), &source("{v:float}")),
        Err(Error::Unsupported("predeclared type"))
    );
    let incomplete = source("let string=string\n{v:string}");
    assert_eq!(export(&source(""), &incomplete), Err(Error::Incomplete));
    assert!(
        !Sources::new(&source(""), &incomplete)
            .oracle()
            .status
            .success()
    );
    // Own-initializer exclusion survives entry into another struct scope.
    check(
        "",
        "let x=1\n{v:{let x={value:x}, result:x}}",
        json!({"v":{"result":{"value":1}}}),
    );
}

#[test]
fn initializer_exclusion_also_applies_to_field_collision_admission() {
    check("", "let x={x:1}\n{v:x}", json!({"v":{"x":1}}));
    check(
        "",
        "let x={nested:{x:1}}\n{v:x}",
        json!({"v":{"nested":{"x":1}}}),
    );
    check(
        "",
        "let string={string:1}\n{v:string}",
        json!({"v":{"string":1}}),
    );
    let field_reference = source("let x={x:1, v:x}\n{result:x}");
    assert!(matches!(
        export(&source(""), &field_reference),
        Err(Error::Unsupported(_))
    ));
    assert!(
        Sources::new(&source(""), &field_reference)
            .oracle()
            .status
            .success()
    );
}

#[test]
fn declaration_capture_and_definition_transport_are_not_caller_lookup() {
    check(
        "",
        "let x=1\nlet T={v:x}\n{result:{let x=2, outer:T, local:x}}",
        json!({"result":{"outer":{"v":1},"local":2}}),
    );
    check("let X=1\n#D:{v:X}", "#D", json!({"v":1}));
    // Same spelling in separate files is legal and remains file-local.
    check(
        "let X=1\n#D:{v:X}",
        "let X=2\n{a:#D,b:X}",
        json!({"a":{"v":1},"b":2}),
    );
    let (first, second) = (source("let X=1\n#D:{v:X}"), source("{v:X}"));
    assert_eq!(export(&first, &second), Err(Error::Unknown("X".into())));
    assert!(!Sources::new(&first, &second).oracle().status.success());
}

#[test]
fn open_templates_refine_independently_and_inherit_definition_closedness() {
    check("", "let T={a:1}\nT & {b:2}", json!({"a":1,"b":2}));
    for body in [
        "let T={a:int}\n{x:T & {a:1},y:T & {a:2}}",
        "let T={a:int}\n{y:{a:2} & T,x:{a:1} & T}",
    ] {
        check("", body, json!({"x":{"a":1},"y":{"a":2}}));
    }
    check(
        "",
        "let T={tag:\"a\",v:int}|{tag:\"b\",v:string}\n{x:T & {tag:\"a\",v:1},y:T & {tag:\"b\",v:\"ok\"}}",
        json!({"x":{"tag":"a","v":1},"y":{"tag":"b","v":"ok"}}),
    );
    check(
        "let T={a:1}\n#D:{nested:T}",
        "#D",
        json!({"nested":{"a":1}}),
    );
    let schema = source("let T={a:1}\n#D:{nested:T}");
    let body = source("#D & {nested:{b:2}}");
    assert_eq!(export(&schema, &body), Err(Error::Conflict));
    assert!(!Sources::new(&schema, &body).oracle().status.success());
}

#[test]
fn quoted_labels_are_data_not_identifier_declarations() {
    check("", "let x=1\n{\"x\":2,v:x}", json!({"x":2,"v":1}));
    check(
        "",
        "let x=1\n{v:{\"x\":2, value:x}}",
        json!({"v":{"x":2,"value":1}}),
    );
    let body = source("{\"x\":2,v:x}");
    assert_eq!(export(&source(""), &body), Err(Error::Unknown("x".into())));
    assert!(!Sources::new(&source(""), &body).oracle().status.success());
    // CUE can resolve ordinary fields; this native profile deliberately cannot.
    let body = source("{x:1,v:x}");
    assert!(matches!(
        export(&source(""), &body),
        Err(Error::Unsupported(_))
    ));
    assert!(Sources::new(&source(""), &body).oracle().status.success());
}

#[test]
fn admission_rejects_unused_redeclared_colliding_unknown_and_cyclic_bindings() {
    // Compare categories, not diagnostic wording owned by the resolver.
    for body in [
        "let x=1\n{v:2}",
        "let x=1\nlet x=2\n{v:x}",
        "let x=1\n{x:2,v:x}",
        "let x=1\n{v:{x:2, value:x}}",
        "let x=1\nlet T={x:2}\n{a:x,b:T}",
        "{x:1,v:{let x=2, value:x}}",
        "let A=B\nlet B=A\n{v:A}",
        "let A=#D\n#D:A\n{v:A}",
        "let A=B\nlet B=A\n{v:1|A}",
    ] {
        let error = export(&source(""), &source(body)).expect_err("admission rejection");
        assert!(matches!(error, Error::Unsupported(_)), "{body}: {error:?}");
        reject(body, error);
    }
    reject("let x=x\n{v:x}", Error::Unknown("x".into()));
    reject("let x={v:x}\n{result:x}", Error::Unknown("x".into()));
    reject("{v:missing}", Error::Unknown("missing".into()));
}

#[test]
fn productive_binding_cycles_are_an_explicit_native_profile_boundary() {
    // Admission examines dependencies even when no root demands their values.
    let unused = source("let A=#D\n#D:A\n{v:1}");
    assert!(matches!(
        export_with_limit(&source(""), &unused, 0),
        Err(Error::Unsupported(_))
    ));
    let unused_cue = Sources::new(&source(""), &unused).oracle();
    assert!(unused_cue.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&unused_cue.stdout).expect("undemanded cycle CUE JSON"),
        json!({"v":1})
    );
    let body = source("let A=#D\n#D:{children:[...A]}\n#D");
    assert!(matches!(
        export(&source(""), &body),
        Err(Error::Unsupported(_))
    ));
    let files = Sources::new(&source(""), &body);
    let cue = files.oracle();
    assert!(
        cue.status.success(),
        "{}",
        String::from_utf8_lossy(&cue.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&cue.stdout).expect("productive CUE cycle JSON"),
        json!({"children":[]})
    );
    let cli = files.run(env!("CARGO_BIN_EXE_cuestacean-spec-export"), false);
    assert!(!cli.status.success());
    assert!(cli.stdout.is_empty());
}

#[test]
fn copy_owned_style_instruction_trees_not_whole_lowering_policy() {
    // Raw templates from ast-lowering-fragments.cue, not the unsupported policy.
    let body = r#"
let Copy = {op: "concat", parts: [{op: "text", value: "_arg"}, {op: "unsigned", field: "index"}]}
let Owned = {op: "concat", parts: [{op: "text", value: "_arg"}, {op: "unsigned", field: "index"}, {
    op: "ifZero", field: "remaining", then: {op: "text", value: ""}, else: {op: "text", value: ".clone()"}
}]}
{result: [Copy, Owned, Copy]}
"#;
    let copy = json!({"op":"concat","parts":[{"op":"text","value":"_arg"},
        {"op":"unsigned","field":"index"}]});
    let owned = json!({"op":"concat","parts":[{"op":"text","value":"_arg"},
        {"op":"unsigned","field":"index"},{"op":"ifZero","field":"remaining",
        "then":{"op":"text","value":""},"else":{"op":"text","value":".clone()"}}]});
    check("", body, json!({"result":[copy,owned,copy]}));
}

#[test]
fn binding_admission_cap_and_cumulative_work_and_copy_allowances() {
    let chain = |n: usize| {
        let mut body = String::new();
        for i in 0..n {
            let value = if i + 1 == n {
                "1".into()
            } else {
                format!("X{}", i + 1)
            };
            body.push_str(&format!("let X{i}={value}\n"));
        }
        body.push_str("{v:X0}");
        body
    };
    check("", &chain(64), json!({"v":1}));
    assert!(matches!(
        export(&source(""), &source(&chain(65))),
        Err(Error::Limit(_))
    ));
    let first = source("");
    let once = source("let X=1\n{v:X}");
    let work = (1..256)
        .find(|&n| export_with_limit(&first, &once, n).is_ok())
        .expect("single expansion cost");
    assert_eq!(
        export_with_limit(&first, &source("let X=1\n{v:X|X}"), work),
        Err(Error::Limit("evaluation steps"))
    );
    let copies = |size| source(&format!("let X={}\nX|X", json!("a".repeat(size))));
    assert_eq!(
        export(&first, &copies(40000)),
        Err(Error::Limit("copied strings"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            &export(&first, &copies(30000)).expect("below shared copy cap")
        )
        .expect("native copied JSON"),
        json!("a".repeat(30000))
    );
    assert!(
        export(&first, &once).is_ok(),
        "failed solves do not poison later uses"
    );
}
