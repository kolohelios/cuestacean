use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = "package scalarprobe\n";
fn source(root: &str) -> String {
    format!("{PACKAGE}{root}\n")
}
fn native(root: &str) -> Result<Value, Error> {
    export(PACKAGE, &source(root)).map(|s| serde_json::from_str(&s).expect("native exact JSON"))
}
struct Probe(PathBuf);
impl Probe {
    fn new(a: &str, b: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "scalar-policy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(dir.is_absolute());
        fs::create_dir(&dir).expect("fresh owned scratch");
        fs::create_dir(dir.join("home")).expect("owned HOME");
        fs::write(dir.join("z.cue"), a).expect("first raw input");
        fs::write(dir.join("a.cue"), b).expect("second raw input");
        Self(dir)
    }
    fn run(&self, operation: Option<&str>, reverse: bool) -> Output {
        let binary = if operation.is_some() {
            std::env::split_paths(&std::env::var_os("PATH").expect("pinned provider PATH"))
                .map(|p| p.join("cue"))
                .find(|p| p.is_file())
                .expect("mandatory pinned CUE")
                .canonicalize()
                .expect("absolute CUE")
        } else {
            PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
        };
        assert!(binary.is_absolute());
        fs::File::open(&binary).expect("readable provider");
        assert_ne!(
            fs::metadata(&binary)
                .expect("provider metadata")
                .permissions()
                .mode()
                & 0o111,
            0
        );
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", self.0.join("home"))
            .current_dir(&self.0);
        if let Some(op) = operation {
            command.arg(op);
        }
        command
            .args(if reverse {
                ["a.cue", "z.cue"]
            } else {
                ["z.cue", "a.cue"]
            })
            .output()
            .expect("fresh mandatory process")
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("retire owned scratch");
    }
}
fn failed(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "no partial JSON");
    assert!(!output.stderr.is_empty(), "failure diagnostic");
}
fn parsed(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("complete exact JSON")
}
// Independent expectations precede all native/oracle observations. Incomplete
// means eval succeeds but export fails; Conflict must also fail ordinary eval.
fn verify(a: &str, b: &str, expected: Result<Value, Error>) {
    let probe = Probe::new(a, b);
    for reverse in [false, true] {
        let oracle = probe.run(Some("export"), reverse);
        match &expected {
            Ok(value) => assert_eq!(parsed(&oracle), *value),
            Err(error) => {
                failed(&oracle);
                let eval = probe.run(Some("eval"), reverse);
                match error {
                    Error::Incomplete => assert!(
                        eval.status.success(),
                        "{}",
                        String::from_utf8_lossy(&eval.stderr)
                    ),
                    Error::Conflict => assert!(!eval.status.success()),
                    other => panic!("separate profile/error expectation required: {other:?}"),
                }
            }
        }
        let (a, b) = if reverse { (b, a) } else { (a, b) };
        for _ in 0..2 {
            assert_eq!(
                export(a, b).map(|s| serde_json::from_str::<Value>(&s).expect("native JSON")),
                expected
            );
        }
        let cli = probe.run(None, reverse);
        match &expected {
            Ok(value) => assert_eq!(parsed(&cli), *value),
            Err(_) => failed(&cli),
        }
    }
}
fn check(root: &str, expected: Result<Value, Error>) {
    verify(PACKAGE, &source(root), expected);
}

#[test]
fn literal_preferences_equal_overlapping_types_and_top_have_authored_values() {
    for (root, value) in [
        ("{v:(*true|false)}", json!(true)),
        (r#"{v:(*"p"|"q")}"#, json!("p")),
        ("{v:(*1|2)}", json!(1)),
        ("{v:(*false|false)}", json!(false)),
        (r#"{v:(*"same"|"same")}"#, json!("same")),
        ("{v:(*1|1)}", json!(1)),
        ("{v:(*true|bool)}", json!(true)),
        (r#"{v:(*"p"|string)}"#, json!("p")),
        ("{v:(*1|int)}", json!(1)),
        ("{v:(*1|_)}", json!(1)),
        ("{v:(*-9223372036854775808|int)}", json!(i64::MIN)),
        ("{v:(*9223372036854775807|int)}", json!(i64::MAX)),
        (
            "{v:(*9007199254740993|int)}",
            json!(9_007_199_254_740_993i64),
        ),
        ("{v:(*1|bool)}&{v:false}", json!(false)),
        (r#"{v:(*1|string)}&{v:"forced"}"#, json!("forced")),
        ("{v:(*true|int)}&{v:2}", json!(2)),
    ] {
        check(root, Ok(json!({"v":value})));
    }
}

#[test]
fn complete_context_can_force_top_to_scalars_or_containers_without_invention() {
    for (context, value) in [
        ("false", json!(false)),
        (r#""forced""#, json!("forced")),
        ("2", json!(2)),
        ("{x:[1,true]}", json!({"x":[1,true]})),
        ("[1,{x:false}]", json!([1,{"x":false}])),
    ] {
        check(
            &format!("{{v:(*1|_)}}&{{v:{context}}}"),
            Ok(json!({"v":value})),
        );
    }
    for root in [
        "{v:(*1|bool)}&{v:bool}",
        "{v:(*1|_)}&{v:bool}",
        "{v:(*1|_)}&{v:{x:int}}",
        "{v:(*1|_)}&{v:[string]}",
        "{v:(*1|_),other:int}",
    ] {
        check(root, Err(Error::Incomplete));
    }
    for root in [
        "{v:(*1|2)}&{v:3}",
        "{v:(*1|bool)}&{v:string}",
        "{v:(*1|_),other:{x:int,y:1}}&{other:{y:2}}",
        "{outer:{v:(*1|_)}}&{outer:2}",
        "{v:(*1|_)}&{v:[1]}&{v:[1,2]}",
    ] {
        check(root, Err(Error::Conflict));
    }
}

#[test]
fn labels_alternatives_fields_files_and_conjunction_order_do_not_choose_results() {
    for choice in ["*1|_", "_ | *1"] {
        let a = source(&format!(
            "{{outer:{{\"arbitrary label\":({choice})}},z:true}}"
        ));
        let b = source("{outer:{\"arbitrary label\":{x:2}},a:false}");
        let expected = json!({"outer":{"arbitrary label":{"x":2}},"a":false,"z":true});
        verify(&a, &b, Ok(expected.clone()));
        check(
            &format!(
                "{{a:false,outer:{{\"arbitrary label\":{{x:2}}}}}}&{{z:true,outer:{{\"arbitrary label\":({choice})}}}}"
            ),
            Ok(expected),
        );
    }
    check("{v:2,v:(*1|int)}", Ok(json!({"v":2})));
    check("{v:(*1|int),v:2}", Ok(json!({"v":2})));
}

#[test]
fn whole_document_exclusions_and_nonselected_errors_never_become_fallback() {
    for root in [
        "*1|int",
        "{v:[(*1|2)]}",
        "{v:(*int|2)}",
        "{v:(*_|2)}",
        "{v:(*1|{x:2})}",
        "{v:(*1|[2])}",
        "{v:(*1|(2&int))}",
        "{v:(*(1&int)|2)}",
        "{v:(*1|2|3)}",
        "{v:(*1|*2)}",
        "{v:(*1|2),unused:(1|1)}",
        "{v:(*1|2),unused:[...int]}",
        "{v:(*1|2),unused:(int&!=3)}",
        "{v:(*1|2),unused:(*3|4)}",
        "#Unused:1\n{v:(*1|2)}",
        "let X=1\n{v:(*1|2)}",
        "#X:1\n{v:(*1|2),unused:#X}",
        "let X=(*1|2)\n{a:X,b:X}",
        "{v:(*1|2),unused:{let X=1\nx:X}}",
    ] {
        assert!(
            matches!(native(root), Err(Error::Unsupported(_))),
            "{root}: {:?}",
            native(root)
        );
        failed(&Probe::new(PACKAGE, &source(root)).run(None, false));
    }
    for (root, error) in [
        ("{v:(*1|9223372036854775808)}", Error::Range),
        ("{v:(*1|missing)}", Error::Unknown("missing".into())),
        ("{v:(*1|2),unused:-9223372036854775809}", Error::Range),
        (
            "{v:(*1|2),unused:missing}",
            Error::Unknown("missing".into()),
        ),
    ] {
        assert_eq!(native(root), Err(error));
        failed(&Probe::new(PACKAGE, &source(root)).run(None, false));
    }
    assert_eq!(
        native("{v:_}"),
        Err(Error::Unsupported("value reference or type"))
    );
    assert_eq!(
        native("{v:_Unknown}"),
        Err(Error::Unknown("_Unknown".into()))
    );
    check(r#"{"*":"_"} // *"#, Ok(json!({"*":"_"})));
}

#[test]
fn adjacent_bottom_like_spelling_is_not_an_unmarked_top_alternative() {
    let root = "{v:(_|*1)}";
    let input = source(root);
    let probe = Probe::new(PACKAGE, &input);
    for reverse in [false, true] {
        let oracle = probe.run(Some("export"), reverse);
        failed(&oracle);
        assert!(String::from_utf8_lossy(&oracle.stderr).contains("illegal token '_|'"));
        failed(&probe.run(Some("eval"), reverse));
        let (a, b) = if reverse {
            (input.as_str(), PACKAGE)
        } else {
            (PACKAGE, input.as_str())
        };
        assert_eq!(
            export(a, b),
            Err(Error::Unsupported("value reference or type"))
        );
        failed(&probe.run(None, reverse));
    }
    assert_eq!(
        native("{v:_}"),
        Err(Error::Unsupported("value reference or type"))
    );
    check("{v:(_ | *1)}", Ok(json!({"v":1})));
    check(
        r#"{v:"_|"} // _| is also ordinary comment data"#,
        Ok(json!({"v":"_|"})),
    );
}

fn boundary(root: &str, expected: &Value) -> usize {
    let source = source(root);
    let (mut low, mut high) = (0, 4096);
    assert_eq!(native(root), Ok(expected.clone()));
    while low < high {
        let mid = low + (high - low) / 2;
        match export_with_limit(PACKAGE, &source, mid) {
            Ok(text) => {
                assert_eq!(
                    serde_json::from_str::<Value>(&text).expect("budget JSON"),
                    *expected
                );
                high = mid;
            }
            Err(Error::Limit("evaluation steps")) => low = mid + 1,
            other => panic!("unexpected budget result {other:?}"),
        }
    }
    assert!(low > 0);
    assert_eq!(
        export_with_limit(PACKAGE, &source, low - 1),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            &export_with_limit(PACKAGE, &source, low).expect("exact boundary success")
        )
        .expect("boundary JSON"),
        *expected
    );
    assert_eq!(
        export_with_limit(PACKAGE, &source, 4097),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        native(root),
        Ok(expected.clone()),
        "fresh recovery after failed solve"
    );
    low
}
#[test]
fn both_trials_share_one_exact_lowered_work_boundary_and_recover() {
    // Identical syntax shapes/counts; a contradictory Q stops before the trailing
    // type constraints, whereas equal Q must inspect them even though P survives.
    let context = "{v:int}&".repeat(8);
    let pruned = format!("{context}{{v:(*1|2)}}&{{v:1}}");
    let live = format!("{context}{{v:(*1|1)}}&{{v:1}}");
    let short = boundary(&pruned, &json!({"v":1}));
    let long = boundary(&live, &json!({"v":1}));
    assert!(
        long > short,
        "unselected feasible Q consumes shared work: {long}/{short}"
    );
    assert_eq!(
        export_with_limit(PACKAGE, &source(&live), short),
        Err(Error::Limit("evaluation steps"))
    );
    boundary("{v:(*1|_)}&{v:{x:[2,true]}}", &json!({"v":{"x":[2,true]}}));
}
