use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = "package named\n";
fn source(body: &str) -> String {
    format!("{PACKAGE}{body}\n")
}
fn wrapped(mut leaf: String, fields: &[&str]) -> String {
    for name in fields.iter().rev() {
        leaf = format!("{{{}:{leaf}}}", serde_json::to_string(name).expect("label"));
    }
    leaf
}
fn expected(mut value: Value, fields: &[&str]) -> Value {
    for name in fields.iter().rev() {
        value = json!({*name:value});
    }
    value
}
fn pair(n: usize, q: &str) -> (String, String) {
    let fields = vec!["v"; n];
    (
        source(&format!("#X:{}", wrapped(format!("(*1 | {q})"), &fields))),
        source("#X"),
    )
}
fn executable(path: &Path) {
    assert!(path.is_absolute(), "absolute mandatory tool");
    let meta = fs::metadata(path).expect("tool metadata");
    assert!(meta.is_file(), "regular mandatory tool");
    fs::File::open(path).expect("readable mandatory tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0, "executable tool");
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
    native: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "named-defaults-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let discover = |name| {
            let path = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute tool");
            executable(&path);
            path
        };
        let cue = discover("cue");
        let _jq = discover("jq");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        Self { dir, cue, native }
    }
    fn run(&self, op: Option<&str>, a: &str, b: &str, reverse: bool) -> Output {
        fs::write(self.dir.join("owner.cue"), a).expect("raw owner");
        fs::write(self.dir.join("selector.cue"), b).expect("raw selector");
        let mut command = Command::new(if op.is_some() {
            &self.cue
        } else {
            &self.native
        });
        if let Some(op) = op {
            command.arg(op);
        }
        command
            .args(if reverse {
                ["selector.cue", "owner.cue"]
            } else {
                ["owner.cue", "selector.cue"]
            })
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .output()
            .expect("mandatory process setup")
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete exact JSON")
}
fn failed(out: &Output) {
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "no partial JSON");
    assert!(!out.stderr.is_empty());
}

#[test]
fn arbitrary_names_paths_scalar_domains_and_preference_positions_select_literal_p() {
    let probe = Probe::new();
    for (p, value, qs) in [
        (
            "true",
            json!(true),
            vec!["true", "false", "bool", "string", "_"],
        ),
        (
            "false",
            json!(false),
            vec!["false", "true", "bool", "int", "_"],
        ),
        (
            r#""é\n\"quoted""#,
            json!("é\n\"quoted"),
            vec![r#""é\n\"quoted""#, r#""other""#, "string", "bool", "_"],
        ),
        (
            "-9223372036854775808",
            json!(i64::MIN),
            vec![
                "-9223372036854775808",
                "9223372036854775807",
                "int",
                "string",
                "_",
            ],
        ),
        (
            "9223372036854775807",
            json!(i64::MAX),
            vec!["0", "int", "_"],
        ),
        (
            "9007199254740993",
            json!(9_007_199_254_740_993i64),
            vec!["9007199254740993", "int", "_"],
        ),
    ] {
        for q in qs {
            for (name, fields) in [
                ("#Unrelated", vec!["arbitrary label"]),
                ("#_Different", vec!["outer", "雪", "leaf"]),
            ] {
                let want = expected(value.clone(), &fields);
                for right in [false, true] {
                    let choice = if right {
                        format!("({q} | *{p})")
                    } else {
                        format!("(*{p} | {q})")
                    };
                    let a = source(&format!("{name}:{}", wrapped(choice, &fields)));
                    let b = source(name);
                    for reverse in [false, true] {
                        let (first, second) = if reverse { (&b, &a) } else { (&a, &b) };
                        for _ in 0..2 {
                            assert_eq!(
                                serde_json::from_str::<Value>(
                                    &export(first, second).expect("native admission")
                                )
                                .expect("JSON"),
                                want
                            );
                        }
                        assert_eq!(decoded(probe.run(None, &a, &b, reverse)), want);
                        assert_eq!(decoded(probe.run(Some("export"), &a, &b, reverse)), want);
                    }
                }
            }
        }
    }
}

#[test]
fn entire_original_arena_exclusions_include_unused_and_unselected_syntax() {
    let probe = Probe::new();
    for (body, root) in [
        ("{v:(*1|2)}", "#X & {}"),
        ("{v:(*1|2)}", "#X\n#X"),
        ("{v:(*1|2)}", "{left:#X,right:#X}"),
        ("{v:(*1|2)}", "{v:#X}"),
        ("{v:(*1|2)}", "#Unused:0\n#X"),
        ("{v:(*1|2)}", "#Alias:#X\n#Alias"),
        ("{v:(*1|2)}", "let A=#X\nA"),
        ("{v:(*1|2)}", "let A=0\n#X"),
        ("{v:(*1|2),other:0}", "#X"),
        ("{v:(*1|2),v:1}", "#X"),
        ("{outer:{v:(*1|2),other:0}}", "#X"),
        ("{v:(*1|2)} & {}", "#X"),
        ("{v:((*1|2)&int)}", "#X"),
        ("{v:[(*1|2)]}", "#X"),
        ("{v:(*1|[2])}", "#X"),
        ("{v:(*[1]|2)}", "#X"),
        ("{v:(*{k:1}|{k:2})}", "#X"),
        ("{v:(*1|{k:2})}", "#X"),
        ("{v:(*int|2)}", "#X"),
        ("{v:(*_|2)}", "#X"),
        ("{v:(*1|*2)}", "#X"),
        ("{v:(*1|(2|3))}", "#X"),
        ("{v:(*(*1|2)|3)}", "#X"),
        ("{v:(*(1&int)|2)}", "#X"),
        ("{v:(*1|(int&!=2))}", "#X"),
        ("{v:(*1|#X)}", "#X"),
        ("{v:(*1|2),again:#X}", "#X"),
        ("(*1|2)", "#X"),
        ("{let A=1\nv:(*1|A)}", "#X"),
        ("{v:(*1|2)}", "#Other:{v:(*3|4)}\n#X"),
        ("{v:(*1|2)}", "#Other:[...int]\n#X"),
        ("{v:(*1|2)}", "#Other:#Other\n#X"),
        ("{v:(*1|2)}", "{for k,v in {} {\"\\(k)\":v}}\n#X"),
    ] {
        let a = source(&format!("#X:{body}"));
        let b = source(root);
        for reverse in [false, true] {
            let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
            assert!(
                matches!(export(a, b), Err(Error::Unsupported(_))),
                "{body}; {root}: {:?}",
                export(a, b)
            );
            failed(&probe.run(None, a, b, false));
        }
    }
    for (body, root, error) in [
        ("{v:(*1|9223372036854775808)}", "#X", Error::Range),
        ("{v:(*1|-9223372036854775809)}", "#X", Error::Range),
        ("{v:(*1|#Missing)}", "#X", Error::Unknown("#Missing".into())),
        ("{v:(*1|missing)}", "#X", Error::Unknown("missing".into())),
        ("{v:(*1|2)}", "#Missing", Error::Unknown("#Missing".into())),
    ] {
        let a = source(&format!("#X:{body}"));
        let b = source(root);
        assert_eq!(export(&a, &b), Err(error));
        failed(&probe.run(None, &a, &b, false));
    }
}

#[test]
fn go_only_contexts_observe_recursive_closedness_conflict_and_incomplete_without_native_admission()
{
    let probe = Probe::new();
    let a = source("#X:{outer:{v:(*1|bool)}}");
    for (context, ordinary_possible, diagnostic) in [
        ("{extra:2}", false, "field not allowed"),
        ("{outer:{extra:2}}", false, "field not allowed"),
        ("{outer:{v:3}}", false, "empty disjunction"),
        ("{outer:{v:bool}}", true, "incomplete value bool"),
    ] {
        let b = source(&format!("#X & {context}"));
        assert!(matches!(export(&a, &b), Err(Error::Unsupported(_))));
        for reverse in [false, true] {
            let out = probe.run(Some("export"), &a, &b, reverse);
            failed(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains(diagnostic));
            assert_eq!(
                probe.run(Some("eval"), &a, &b, reverse).status.success(),
                ordinary_possible
            );
        }
    }
    assert_eq!(
        decoded(probe.run(Some("export"), &a, &source("#X & {outer:{v:false}}"), false)),
        json!({"outer":{"v":false}})
    );
}

#[test]
fn independently_billed_exact_work_and_ref_inclusive_depth_boundaries() {
    // A=2+6+(n+5)+4+3+(2n+7)+1=28+3n. Add original E and root stamp1.
    for q in ["2", "int", "_"] {
        for n in [1, 2, 7, 25, 26, 27] {
            let admission = 28 + 3 * n;
            let evaluation =
                39 + 41 * n + 9 * n * (n - 1) / 2 - if q == "_" { 3 * (n + 1) } else { 0 };
            let total = admission + evaluation + 1;
            let (a, b) = pair(n, q);
            if total <= 4096 {
                let want = expected(json!(1), &vec!["v"; n]);
                assert_eq!(
                    export_with_limit(&a, &b, total - 1),
                    Err(Error::Limit("evaluation steps"))
                );
                assert_eq!(
                    serde_json::from_str::<Value>(
                        &export_with_limit(&a, &b, total).expect("formula boundary")
                    )
                    .expect("JSON"),
                    want
                );
            } else {
                assert_eq!(export(&a, &b), Err(Error::Limit("evaluation steps")));
            }
        }
    }
    // No parentheses: parser body depth32 fits; Ref makes preferred depth33.
    let a = source(&format!("#X:{}", wrapped("*1 | int".into(), &["v"; 29])));
    let b = source("#X");
    // Parsing precedes evaluation: zero work must reach admission's work guard.
    assert_eq!(
        export_with_limit(&a, &b, 0),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(export(&a, &b), Err(Error::Limit("syntax depth")));
    let (a, b) = pair(28, "int");
    assert_eq!(export(&a, &b), Err(Error::Limit("evaluation steps")));
    let (a, b) = pair(1, "int");
    assert_eq!(
        export_with_limit(&a, &b, 4097),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(&export(&a, &b).expect("fresh recovery")).expect("JSON"),
        json!({"v":1})
    );
}

#[test]
fn actual_source_token_encoded_and_unchanged_copy_guards_not_invented_node_boundaries() {
    let (a, b) = pair(1, "int");
    // Six nodes, two materialized values; 21 tokens including both package headers/LFs.
    let padded = format!("{a}//{}", "x".repeat(65536 - a.len() - b.len() - 2));
    assert!(export(&padded, &b).is_ok());
    assert_eq!(
        export(&format!("{padded}x"), &b),
        Err(Error::Limit("source bytes"))
    );
    let tokens = format!("{a}{}", "\n".repeat(4096 - 21));
    assert!(export(&tokens, &b).is_ok());
    assert_eq!(
        export(&format!("{tokens}\n"), &b),
        Err(Error::Limit("tokens"))
    );
    // LF literal syntax stays small; each decoded LF becomes two encoded bytes.
    for length in [32761, 32762] {
        let a = source(&format!(
            "#X:{{v:*\"\"\"\n{}\n\"\"\" | int}}",
            "\n".repeat(length)
        ));
        let out = export(&a, &b);
        if length == 32761 {
            let text = out.expect("exact encoded cap");
            assert_eq!(text.len(), 65536);
            assert_eq!(
                serde_json::from_str::<Value>(&text).expect("JSON"),
                json!({"v":"\n".repeat(length)})
            );
        } else {
            assert_eq!(out, Err(Error::Limit("encoded bytes")));
        }
    }
    // One-use marked paths cannot reach 64KiB copied data before source/output guards.
    // Preserve actual ordinary-reference cumulative copy behavior without admitting reuse.
    let selector = source(&format!("[{}]", vec!["#S"; 64].join(",")));
    for (size, error) in [
        (1024, Error::Limit("encoded bytes")),
        (1025, Error::Limit("copied strings")),
    ] {
        assert_eq!(
            export(&source(&format!("#S:\"{}\"", "x".repeat(size))), &selector),
            Err(error)
        );
    }
    assert!(
        export(&a, &b).is_ok(),
        "fresh recovery without reset inside an invocation"
    );
}
