use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = include_str!("../../native-policy-package.cue");
fn source(body: &str) -> String {
    format!("package bootstrap\n{body}\n")
}
fn value(a: &str, b: &str) -> Value {
    serde_json::from_str(&export(a, b).expect("native export")).expect("JSON")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn failed(out: Output) {
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "no partial JSON");
    assert!(!out.stderr.is_empty());
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let meta = fs::metadata(path).expect("mandatory tool metadata");
    assert!(meta.is_file());
    fs::File::open(path).expect("readable tool");
    assert_ne!(meta.permissions().mode() & 0o111, 0);
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
            "scalar-projections-{}-{}",
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
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        let _jq = discover("jq");
        Self {
            dir,
            cue: discover("cue"),
            native,
        }
    }
    fn run(&self, tool: &Path, args: &[&str], inputs: &[(&str, &str)], reverse: bool) -> Output {
        executable(tool);
        let mut paths = Vec::new();
        for (name, text) in inputs {
            let path = self.dir.join(name);
            fs::write(&path, text).expect("explicit input");
            paths.push(path);
        }
        if reverse {
            paths.reverse();
        }
        Command::new(tool)
            .args(args)
            .args(paths)
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .output()
            .expect("mandatory process setup")
    }
    fn native(&self, a: &str, b: &str, reverse: bool) -> Output {
        self.run(
            &self.native,
            &[],
            &[("unprivileged-a.cue", a), ("unrelated-b.cue", b)],
            reverse,
        )
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn generic_scalars_all_fields_names_orders_capture_and_repeated_independent_uses() {
    let p = Probe::new();
    // Every one of64 target fields is selected, in independent batches of8.
    // Values/expected trees are authored here, never recovered from native/Go output.
    let literals: Vec<Value> = (0..64)
        .map(|i| match i % 4 {
            0 => json!(format!("雪\\\"\n{i}")),
            1 => json!(i % 3 == 0),
            2 => json!(i64::MIN + i),
            _ => json!(i64::MAX - i),
        })
        .collect();
    for reverse in [false, true] {
        for backwards in [false, true] {
            let mut fields: Vec<_> = literals
                .iter()
                .enumerate()
                .map(|(i, v)| format!("k{i}:{v}"))
                .collect();
            if backwards {
                fields.reverse();
            }
            let a = source(&format!("#Unprivileged:{{{}}}", fields.join(",")));
            for start in (0..64).step_by(8) {
                let mut want = json!({});
                let mut selections = Vec::new();
                for (i, literal) in literals.iter().enumerate().skip(start).take(8) {
                    want[format!("out{i}")] = literal.clone();
                    selections.push(format!("out{i}:#Unprivileged \t. k{i}"));
                }
                let b = source(&format!("{{{}}}", selections.join(",")));
                assert_eq!(decoded(p.native(&a, &b, reverse)), want);
                assert_eq!(
                    decoded(p.run(
                        &p.cue,
                        &["export"],
                        &[("a.cue", &a), ("b.cue", &b)],
                        reverse
                    )),
                    want
                );
            }
        }
        let a =
            source("let Captured={v:#Anything.answer}\n#Outer:{v:Captured}\n#Anything:{answer:42}");
        let b = source("{first:#Outer,again:#Anything.answer}");
        assert_eq!(
            decoded(p.native(&a, &b, reverse)),
            json!({"first":{"v":{"v":42}},"again":42})
        );
        for answer in [
            json!("renamed"),
            json!("\r"),
            json!(false),
            json!(-9223372036854775808_i64),
        ] {
            let a = source(&format!("#Other:{{arbitrary:{answer}}}"));
            let b = source("[#Other.arbitrary,#Other.arbitrary]");
            assert_eq!(decoded(p.native(&a, &b, reverse)), json!([answer, answer]));
        }
    }
}

#[test]
fn original_targets_and_whole_arena_exclusions_unknowns_cycles_and_cli_recovery() {
    let p = Probe::new();
    let good = source("#D:{x:1}");
    let root = source("#D.x");
    for reverse in [false, true] {
        for target in [
            "1",
            "#Alias",
            "{x:#Alias}",
            "{x:1,x:1}",
            "{x:1,unused:string}",
            "{x:1,unused:{a:1}}",
            "{x:1,unused:[1]}",
            "{x:1,unused:#Alias.x}",
            "{x:1,unused:1&1}",
            "{x:1,unused:1|1}",
            "{x:1,unused:*1|2}",
        ] {
            let a = source(&format!("#Alias:{{x:1}}\n#D:{target}"));
            assert!(
                matches!(export(&a, &root), Err(Error::Unsupported(_))),
                "{target}"
            );
            failed(p.native(&a, &root, reverse));
            assert_eq!(decoded(p.native(&good, &root, reverse)), json!(1));
        }
        // The selected target is valid: exclusions in UNUSED arena still reject.
        for unused in [
            "string",
            "bool",
            "int",
            "1&1",
            "1|1",
            "*1|2",
            "_",
            "!=1",
            "[...int]",
            "{for k,v in T {\"\\(k)\":v}}",
        ] {
            let extra = if unused.contains("for ") {
                "let T={a:\"x\"}\n"
            } else {
                ""
            };
            let a = source(&format!("{extra}#D:{{x:1}}\n#Unused:{unused}"));
            assert!(
                matches!(export(&a, &root), Err(Error::Unsupported(_))),
                "{unused}"
            );
            failed(p.native(&a, &root, reverse));
        }
        for (a, b, missing) in [
            (source("#D:{x:1}"), source("#Absent.x"), "#Absent"),
            (source("#D:{x:1}"), source("#D.absent"), "absent"),
            (source("#D:{}"), source("#D.x"), "x"),
        ] {
            let Err(Error::Unknown(name)) = export(&a, &b) else {
                panic!("Unknown required")
            };
            assert!(name.contains(missing), "actual missing name/path: {name}");
            failed(p.native(&a, &b, reverse));
        }
        for body in ["let Local=#D.x\n#D:{x:Local}\nLocal", "let Unused=1\n#D.x"] {
            assert!(matches!(
                export(&good, &source(body)),
                Err(Error::Unsupported(_))
            ));
        }
        // Above retains the duplicate-declaration negative. This ONE-#D fixture
        // separately proves projection dependency edges participate in cycles.
        let cycle = source("let Local=#D.x\n#D:{x:Local}\nLocal");
        assert_eq!(
            export(PACKAGE, &cycle),
            Err(Error::Unsupported("cyclic binding dependency"))
        );
        failed(p.native(PACKAGE, &cycle, reverse));
        for selector in [
            "#D.x.x",
            "#D.\"x\"",
            "#D._hidden",
            "#D.#Other",
            "#D[\"x\"]",
            "(#D).x",
            "#D\n.x",
            "#D.\nx",
        ] {
            assert!(matches!(
                export(&good, &source(selector)),
                Err(Error::Unsupported(_) | Error::Syntax(_))
            ));
            failed(p.native(&good, &source(selector), reverse));
        }
        // Preserve the old broad predicates, but do not use them as sole proof.
        for selector in [
            "#D.x.x",
            "#D.\"x\"",
            "#D._hidden",
            "#D.#Other",
            "(#D).x",
            "#D\n.x",
            "#D.\nx",
            "let Local=#D\nLocal.x",
            "#D\r.x",
            "#D.\rx",
            "#D \r .x",
            "#D. \r x",
            "#D\t\r\t.x",
            "#D.\t\r\tx",
            "#D\r\n.x",
            "#D.\r\nx",
        ] {
            assert!(
                matches!(export(&good, &source(selector)), Err(Error::Unsupported(_))),
                "{selector}"
            );
            failed(p.native(&good, &source(selector), reverse));
        }
        for selector in ["#D[\"x\"]", "#D(\"x\")"] {
            assert_eq!(
                export(&good, &source(selector)),
                Err(Error::Syntax("declaration separator"))
            );
        }
        assert_eq!(decoded(p.native(&good, &root, reverse)), json!(1));
    }
    // An earlier projection cannot turn this computed target into a literal target.
    for declarations in ["#A:{x:1}\n#B:{x:#A.x}", "#B:{x:#A.x}\n#A:{x:1}"] {
        assert!(matches!(
            export(&source(declarations), &source("[#A.x,#B.x]")),
            Err(Error::Unsupported(_))
        ));
    }
    failed(p.native(&source("#D:{x:9223372036854775808}"), &root, false));
    assert_eq!(
        export(&source("#D:{x:9223372036854775808}"), &root),
        Err(Error::Range)
    );
}

#[test]
fn derived_preparation_evaluation_work_shared_hard_limit_and_no_selector_old_costs() {
    // Root scalar: A=7+3n,E=6. Singleton list: A=8+3n,E=12.
    // Flat object: N=n+m+2,A=1+N+m*(4+2n),E=4+13m.
    // At n=m=1 these totals are16/23/28, DERIVED from evaluator task charges.
    let one = source("#D:{x:1}");
    for (body, total) in [("#D.x", 16), ("[#D.x]", 23), ("{a:#D.x}", 28)] {
        let b = source(body);
        assert_eq!(
            export_with_limit(&one, &b, total - 1),
            Err(Error::Limit("evaluation steps"))
        );
        export_with_limit(&one, &b, total).expect("derived exact work");
    }
    for n in [1, 2, 7, 32, 64] {
        let a = source(&format!(
            "#D:{{{}}}",
            (0..n)
                .map(|i| format!("k{i}:1"))
                .collect::<Vec<_>>()
                .join(",")
        ));
        for m in [1, 3, 8, 27] {
            let b = source(&format!(
                "{{{}}}",
                (0..m)
                    .map(|i| format!("f{i}:#D.k0"))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            let total = 7 + n + 18 * m + 2 * n * m;
            if total <= 4096 {
                assert_eq!(
                    export_with_limit(&a, &b, total - 1),
                    Err(Error::Limit("evaluation steps"))
                );
                export_with_limit(&a, &b, total).expect("no per-selector allowance reset");
            } else {
                assert_eq!(export(&a, &b), Err(Error::Limit("evaluation steps")));
            }
            // Repeated fixed list: A=n+3+5m+2nm; E=5+7m, no shallow
            // element prepass. Thus T=8+n+12m+2nm, derived independently.
            let list = source(&format!("[{}]", vec!["#D.k0"; m].join(",")));
            let total = 8 + n + 12 * m + 2 * n * m;
            if total <= 4096 {
                assert_eq!(
                    export_with_limit(&a, &list, total - 1),
                    Err(Error::Limit("evaluation steps"))
                );
                export_with_limit(&a, &list, total).expect("derived repeated-list work");
            } else {
                assert_eq!(export(&a, &list), Err(Error::Limit("evaluation steps")));
            }
        }
    }
    // No selectors: old literal3, direct Ref6, one-field ordinary object11.
    for (a, b, cost) in [
        (source(""), source("1"), 3),
        (source(""), source("\r1\r"), 3),
        (source(""), source("\r\n1\r\n"), 3),
        (source(""), source("\"\\r\""), 3),
        (source("#D:1"), source("#D"), 6),
        (source(""), source("{a:1}"), 11),
    ] {
        assert_eq!(
            export_with_limit(&a, &b, cost - 1),
            Err(Error::Limit("evaluation steps"))
        );
        export_with_limit(&a, &b, cost).expect("EXACT old cost unchanged");
    }
    assert_eq!(
        export_with_limit(&one, &source("#D.x"), 4097),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(value(&one, &source("#D.x")), json!(1));
}

#[test]
fn derived_copy_encoding_source_labels_depth_items_and_original_syntax_guards() {
    let p = Probe::new();
    let list = |n| source(&format!("[{}]", vec!["#D.x"; n].join(",")));
    let strings = |s: &str| source(&format!("#D:{{x:{}}}", json!(s)));
    // m selections copy m*s bytes (no target copy in preparation). Encoding adds
    // list punctuation/indentation: exactly65536 copied bytes can fail ONLY encoding.
    export(&strings(&"a".repeat(1024)), &list(63)).expect("below both byte guards");
    assert_eq!(
        export(&strings(&"a".repeat(1024)), &list(64)),
        Err(Error::Limit("encoded bytes"))
    );
    assert_eq!(
        export(&strings(&"a".repeat(1025)), &list(64)),
        Err(Error::Limit("copied strings"))
    );
    assert_eq!(
        export(&strings(&"\n".repeat(600)), &list(60)),
        Err(Error::Limit("encoded bytes"))
    );
    // Object copies include emitted keys once; unescaped pretty JSON is
    // 3+sum(key_bytes+value_bytes+10). Shallow demand does not copy the string.
    let a = strings("abcd");
    assert_eq!(export(&a, &source("{xyz:#D.x}")).expect("object").len(), 20);
    let mut padded = a.clone();
    let b = source("#D.x");
    padded.push_str(&" ".repeat(65536 - padded.len() - b.len()));
    export(&padded, &b).expect("combined source exact65536");
    padded.push(' ');
    assert_eq!(export(&padded, &b), Err(Error::Limit("source bytes")));
    failed(p.native(&padded, &b, false));
    for n in [64, 65] {
        let field = "f".repeat(n);
        let a = source(&format!("#D:{{{field}:1}}"));
        let b = source(&format!("#D.{field}"));
        if n == 64 {
            assert_eq!(value(&a, &b), json!(1));
        } else {
            assert_eq!(export(&a, &b), Err(Error::Limit("label bytes")));
        }
        let name = format!("#{}", "d".repeat(n - 1));
        let a = source(&format!("{name}:{{x:1}}"));
        let b = source(&format!("{name}.x"));
        if n == 64 {
            assert_eq!(value(&a, &b), json!(1));
        } else {
            assert_eq!(export(&a, &b), Err(Error::Limit("label bytes")));
        }
    }
    assert_eq!(
        export(&strings("a"), &list(65)),
        Err(Error::Limit("container items"))
    );
    let a = source("#D:{x:1}");
    let nested = |n| source(&format!("{}#D.x{}", "[".repeat(n), "]".repeat(n)));
    export(&a, &nested(31)).expect("32 root-inclusive levels; selector is ONE node");
    assert!(matches!(export(&a, &nested(32)), Err(Error::Limit(_))));
    let definitions = |n| {
        source(
            &(0..n)
                .map(|i| format!("#D{i}:{{x:1}}\n"))
                .collect::<String>(),
        )
    };
    export(&definitions(64), &source("#D0.x")).expect("64 definitions");
    assert_eq!(
        export(&definitions(65), &source("#D0.x")),
        Err(Error::Limit("definitions"))
    );
    // Balanced binary syntax can reach node2048 BEFORE token/depth/work guards.
    // Its And is intentionally inadmissible, but remains counted, even unused.
    fn tree(n: usize) -> String {
        if n <= 16 {
            return vec!["1"; n].join("&");
        }
        format!("({})&({})", tree(n / 2), tree(n - n / 2))
    }
    let syntax = |n| source(&format!("#D:{{x:1}}\n#Unused:{}", tree(n)));
    assert!(matches!(
        export(&syntax(1023), &source("#D.x")),
        Err(Error::Unsupported(_))
    )); // 2045+2+1=2048
    assert_eq!(
        export(&syntax(1024), &source("#D.x")),
        Err(Error::Limit("syntax nodes"))
    );
    let tokens = source(&format!("#D:{{x:1}}\n{}#D.x", "\n".repeat(4096)));
    assert_eq!(export(&tokens, PACKAGE), Err(Error::Limit("tokens")));
    assert_eq!(decoded(p.native(&a, &source("#D.x"), false)), json!(1));
}
