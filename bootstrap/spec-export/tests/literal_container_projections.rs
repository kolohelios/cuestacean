use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn source(body: &str) -> String {
    format!("package bootstrap\n{body}\n")
}
fn value(a: &str, b: &str) -> Value {
    serde_json::from_str(&export(a, b).expect("native export")).expect("complete JSON")
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete process JSON")
}
fn executable(path: &Path) {
    assert!(path.is_absolute(), "absolute mandatory executable");
    let metadata = fs::metadata(path).expect("mandatory tool metadata");
    assert!(metadata.is_file(), "regular mandatory executable");
    fs::File::open(path).expect("readable mandatory executable");
    assert_ne!(metadata.permissions().mode() & 0o111, 0);
}
struct Probe {
    dir: PathBuf,
    native: PathBuf,
    cue: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "literal-container-projections-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let discover = |name: &str| {
            let path = std::env::split_paths(&std::env::var_os("PATH").expect("pinned tool PATH"))
                .map(|p| p.join(name))
                .find(|p| p.is_file())
                .expect("mandatory pinned tool")
                .canonicalize()
                .expect("absolute pinned tool");
            executable(&path);
            path
        };
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        let cue = discover("cue");
        let _jq = discover("jq");
        let _shell = discover("sh");
        Self { dir, native, cue }
    }
    fn run(&self, tool: &Path, args: &[&str], a: &str, b: &str, reverse: bool) -> Output {
        executable(tool);
        let mut paths = vec![
            self.dir.join("arbitrary-owner.cue"),
            self.dir.join("unrelated-use.cue"),
        ];
        fs::write(&paths[0], a).expect("explicit owner source");
        fs::write(&paths[1], b).expect("explicit use source");
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
        self.run(&self.native, &[], a, b, reverse)
    }
    fn go(&self, a: &str, b: &str, reverse: bool) -> Output {
        self.run(&self.cue, &["export"], a, b, reverse)
    }
    fn agree(&self, a: &str, b: &str, want: &Value) {
        assert_eq!(value(a, b), *want);
        assert_eq!(value(b, a), *want);
        for reverse in [false, true] {
            assert_eq!(decoded(self.native(a, b, reverse)), *want);
            assert_eq!(decoded(self.go(a, b, reverse)), *want);
            assert_eq!(decoded(self.native(a, b, reverse)), *want, "repeat");
        }
    }
    fn reject(&self, a: &str, b: &str, error: Error) {
        self.reject_orders(a, b, error.clone(), error);
    }
    fn reject_orders(&self, a: &str, b: &str, forward: Error, backward: Error) {
        for (reverse, error) in [(false, forward), (true, backward)] {
            let result = if reverse { export(b, a) } else { export(a, b) };
            assert_eq!(
                result,
                Err(error.clone()),
                "owner={a}; use={b}; reverse={reverse}"
            );
            let out = self.native(a, b, reverse);
            assert_eq!(out.status.code(), Some(2));
            assert!(out.stdout.is_empty(), "no partial JSON");
            assert_eq!(
                String::from_utf8(out.stderr).expect("diagnostic UTF-8"),
                format!("{error}\n")
            );
        }
        let good = source("#Recovery:{item:[{ok:true}]}");
        assert_eq!(
            decoded(self.native(&good, &source("{out:#Recovery.item}"), false)),
            json!({"out":[{"ok":true}]})
        );
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}
fn exact(a: &str, b: &str, total: usize, want: &Value) {
    assert!(total <= 4096, "lowered API never raises hard guard");
    for (a, b) in [(a, b), (b, a)] {
        assert_eq!(
            export_with_limit(a, b, total - 1),
            Err(Error::Limit("evaluation steps"))
        );
        let text = export_with_limit(a, b, total).expect("source-derived exact allowance");
        assert_eq!(serde_json::from_str::<Value>(&text).expect("JSON"), *want);
        assert_eq!(value(a, b), *want);
    }
}
fn object(n: usize) -> String {
    format!(
        "{{{}}}",
        (0..n)
            .map(|i| format!("f{i}:1"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn repeated(item: &str, n: usize) -> String {
    format!("[{}]", vec![item; n].join(","))
}
fn wanted_object(n: usize) -> Value {
    let mut result = json!({});
    for i in 0..n {
        result[format!("f{i}")] = json!(1);
    }
    result
}

#[test]
fn complete_literal_trees_names_orders_repeats_and_lexical_instances() {
    let p = Probe::new();
    let tree = json!({
        "array":[{},[],{"snow":"雪\\\"\n","range":[i64::MIN,i64::MAX,false,true]}],
        "empty":{},"emptyList":[],"quoted name":{"é":["",0,-1]}
    });
    let a = source(&format!(
        "#Unprivileged:{{item:{tree},spare:[{{other:9}}],scalar:7}}"
    ));
    let b = source("{out:#Unprivileged.item,again:#Unprivileged.item,scalar:#Unprivileged.scalar}");
    p.agree(&a, &b, &json!({"out":tree,"again":tree,"scalar":7}));
    assert_eq!(
        value(&a, &source("#Unprivileged.item")),
        tree,
        "native arbitrary root"
    );
    let whole = json!({"item":tree,"spare":[{"other":9}],"scalar":7});
    p.agree(&a, &source("{out:#Unprivileged}"), &json!({"out":whole}));
    for (field, want) in [
        ("item", tree.clone()),
        ("spare", json!([{"other":9}])),
        ("scalar", json!(7)),
    ] {
        p.agree(
            &a,
            &source(&format!("{{out:#Unprivileged \t. {field}}}")),
            &json!({"out":want}),
        );
    }
    for payload in [
        json!({}),
        json!([]),
        json!([[], {}, [{}]]),
        json!({"x":[true,"x",i64::MIN]}),
    ] {
        let a = source(&format!("#Different:{{chosen:{payload},unselected:{{}}}}"));
        p.agree(
            &a,
            &source("{out:#Different.chosen}"),
            &json!({"out":payload}),
        );
        assert_eq!(
            value(&a, &source("[#Different.chosen,#Different.chosen]")),
            json!([payload, payload])
        );
    }
    // Definitions capture the declaring file's let, not an unrelated use-file name.
    let a = source("let Captured=#D.item\n#Outer:{nested:Captured}\n#D:{item:{a:[{b:42}]}}");
    let b = source("let Captured={foreign:false}\n{out:#Outer,own:Captured,selected:#D.item}");
    p.agree(&a, &b, &json!({"out":{"nested":{"a":[{"b":42}]}},"own":{"foreign":false},"selected":{"a":[{"b":42}]}}));
    let a =
        source("let First=#D.item\nlet Second=First\n#D:{item:[{a:1}]}\n#Wrapper:{child:Second}");
    p.agree(
        &a,
        &source("{one:#Wrapper,two:#Wrapper,three:#D.item}"),
        &json!({"one":{"child":[{"a":1}]},"two":{"child":[{"a":1}]},"three":[{"a":1}]}),
    );
    // Complete tree changes are independent source effects, never scraped answers.
    for (old, new, want) in [
        ("a:1", "a:2", json!({"out":[{"a":2}]})),
        ("a:1", "renamed:false", json!({"out":[{"renamed":false}]})),
    ] {
        let a = source(&"#Arbitrary:{item:[{a:1}]}".replace(old, new));
        p.agree(&a, &source("{out:#Arbitrary.item}"), &want);
    }
}

#[test]
fn whole_original_targets_arena_exclusions_cycles_unknowns_and_cli_recovery() {
    let p = Probe::new();
    let root = source("{out:#D.item}");
    for target in [
        "{item:{a:1},unused:string}",
        "{item:{a:1},unused:bool}",
        "{item:{a:1},unused:int}",
        "{item:{a:1},unused:#Other}",
        "{item:{a:1},unused:#Other.x}",
        "{item:{a:1},unused:1&1}",
        "{item:{a:1},unused:1|1}",
        "{item:{a:1},unused:*1|2}",
        "{item:{a:1},unused:!=1}",
        "{item:{a:1},unused:[...int]}",
        "{item:[{a:string}]}",
        "{item:[{a:#Other}]}",
        "{item:[{a:1&1}]}",
        "{item:[{a:*1|2}]}",
    ] {
        let a = source(&format!("#Other:{{x:1}}\n#D:{target}"));
        let forward = if target.contains("#Other") {
            "literal projection scalar required"
        } else {
            "scalar projection document profile"
        };
        p.reject_orders(
            &a,
            &root,
            Error::Unsupported(forward),
            Error::Unsupported("literal projection scalar required"),
        );
    }
    for target in ["1", "#Other"] {
        let a = source(&format!("#Other:{{item:1}}\n#D:{target}"));
        p.reject(
            &a,
            &root,
            Error::Unsupported("literal projection struct required"),
        );
    }
    for target in [
        "{item:{a:1,a:1}}",
        "{item:[{a:1,a:1}]}",
        "{item:{a:{b:1,b:1}}}",
        "{item:{},unused:[[{a:1,a:1}]]}",
        "{item:{},item:{}}",
    ] {
        p.reject(
            &source(&format!("#D:{target}")),
            &root,
            Error::Unsupported("duplicate projection target field"),
        );
    }
    // Original Binding and Projection nodes inside a target never become literals first.
    for body in [
        "let Captured={a:1}\n#D:{item:Captured}",
        "#Other:{x:1}\n#D:{item:{a:#Other.x}}\n{first:#Other.x,out:#D.item}",
        "#Other:{x:{a:1}}\n#D:{item:#Other.x}\n{first:#Other.x,out:#D.item}",
    ] {
        p.reject(
            &source(body),
            &root,
            Error::Unsupported("literal projection scalar required"),
        );
    }
    for unused in [
        "string", "bool", "int", "1&1", "1|1", "*1|2", "!=1", "[...int]",
    ] {
        p.reject(
            &source(&format!("#D:{{item:{{a:1}}}}\n#Unused:{unused}")),
            &root,
            Error::Unsupported("scalar projection document profile"),
        );
    }
    let comprehension = "let T={a:\"x\"}\n#D:{item:{}}\n#Unused:{for k,v in T {\"\\(k)\":v}}";
    p.reject(
        &source(comprehension),
        &root,
        Error::Unsupported("scalar projection document profile"),
    );
    let target_comprehension = "let T={a:\"x\"}\n#D:{item:{},unused:{for k,v in T {\"\\(k)\":v}}}";
    p.reject_orders(
        &source(target_comprehension),
        &root,
        Error::Unsupported("scalar projection document profile"),
        Error::Unsupported("literal projection scalar required"),
    );
    for sibling in ["string", "#Loop"] {
        let a = source(&format!(
            "#Loop:#Loop\n#D:{{item:{{a:1}},unused:{sibling}}}"
        ));
        for reverse in [false, true] {
            assert_eq!(
                decoded(p.go(&a, &root, reverse)),
                json!({"out":{"a":1}}),
                "Go can ignore an unselected incomplete sibling"
            );
        }
        let forward = if sibling == "string" {
            "scalar projection document profile"
        } else {
            "literal projection scalar required"
        };
        p.reject_orders(
            &a,
            &root,
            Error::Unsupported(forward),
            Error::Unsupported("literal projection scalar required"),
        );
    }
    p.reject(
        &source("#D:{item:{}}"),
        &source("{out:#Absent.item}"),
        Error::Unknown("#Absent".into()),
    );
    p.reject(
        &source("#D:{item:{}}"),
        &source("{out:#D.absent}"),
        Error::Unknown("#D.absent".into()),
    );
    p.reject(&source("#D:{}"), &root, Error::Unknown("#D.item".into()));
    p.reject(
        &source("let Local=#D.item\n#D:{item:Local}\nLocal"),
        &source(""),
        Error::Unsupported("cyclic binding dependency"),
    );
    p.reject(
        &source("let Unused=1\n#D:{item:{}}"),
        &root,
        Error::Unsupported("unused binding"),
    );
    p.reject(
        &source("#D:{item:{}}"),
        &source("#D:#D\n#D.item"),
        Error::Unsupported("duplicate definition"),
    );
    for (selector, diagnostic) in [
        ("#D.item.item", "direct scalar projection required"),
        ("#D.\"item\"", "regular scalar projection field"),
        ("#D._hidden", "regular scalar projection field"),
        ("#D.#Other", "regular scalar projection field"),
        ("(#D).item", "direct scalar projection required"),
        ("#D\n.item", "direct scalar projection required"),
        ("#D.\nitem", "regular scalar projection field"),
        ("#D\r.item", "direct scalar projection required"),
        ("#D.\ritem", "direct scalar projection required"),
    ] {
        p.reject(
            &source("#D:{item:{}}"),
            &source(selector),
            Error::Unsupported(diagnostic),
        );
    }
    for selector in ["#D[\"item\"]", "#D(\"item\")"] {
        p.reject(
            &source("#D:{item:{}}"),
            &source(selector),
            Error::Syntax("declaration separator"),
        );
    }
    // Raw marked documents and residual alternatives do not become a new solver class.
    for use_site in ["#D.item & {a:1}", "#D.item | {a:1}", "*#D.item | {a:1}"] {
        p.reject(
            &source("#D:{item:{a:1}}"),
            &source(use_site),
            Error::Unsupported("scalar projection document profile"),
        );
    }
    let absent = p.dir.join("missing.cue");
    let out = Command::new(&p.native)
        .arg(absent)
        .env_clear()
        .env("HOME", &p.dir)
        .env("PATH", "/unusable")
        .output()
        .expect("CLI usage process");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
    executable(&p.native);
    let out = Command::new(&p.native)
        .args([p.dir.join("missing-a.cue"), p.dir.join("missing-b.cue")])
        .env_clear()
        .env("HOME", &p.dir)
        .env("PATH", "/unusable")
        .output()
        .expect("CLI read-failure process");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
    assert_eq!(
        decoded(p.native(&source("#D:{item:[]}"), &root, false)),
        json!({"out":[]})
    );
    assert_eq!(value(&source("#D:{item:[]}"), &root), json!({"out":[]}));
}

#[test]
fn selected_closedness_nested_lists_empty_objects_and_independent_use_groups() {
    let p = Probe::new();
    for (item, compatible, extra, want) in [
        ("{a:1}", "{a:1}", "{a:1,b:2}", json!({"a":1})),
        ("{}", "{}", "{b:2}", json!({})),
        ("[{a:1}]", "[{a:1}]", "[{a:1,b:2}]", json!([{"a":1}])),
        ("[[{}]]", "[[{}]]", "[[{b:2}]]", json!([[{}]])),
        (
            "{nested:[{child:{}}]}",
            "{nested:[{child:{}}]}",
            "{nested:[{child:{b:2}}]}",
            json!({"nested":[{"child":{}}]}),
        ),
    ] {
        let a = source(&format!("#D:{{item:{item},other:{{free:9}}}}"));
        for backwards in [false, true] {
            let body = if backwards {
                format!("{{out:{compatible},out:#D.item}}")
            } else {
                format!("{{out:#D.item,out:{compatible}}}")
            };
            p.agree(&a, &source(&body), &json!({"out":want}));
            let body = if backwards {
                format!("{{out:{extra},out:#D.item}}")
            } else {
                format!("{{out:#D.item,out:{extra}}}")
            };
            p.reject(&a, &source(&body), Error::Conflict);
            for reverse in [false, true] {
                let out = p.go(&a, &source(&body), reverse);
                assert_eq!(out.status.code(), Some(1));
                assert!(out.stdout.is_empty());
                assert!(String::from_utf8_lossy(&out.stderr).contains("field not allowed"));
            }
        }
        p.agree(
            &a,
            &source("{out:#D.item,extra:2,again:#D.item}"),
            &json!({"out":want,"extra":2,"again":want}),
        );
        p.agree(
            &a,
            &source("{first:#D,second:#D.item}"),
            &json!({"first":{"item":want,"other":{"free":9}},"second":want}),
        );
    }
    // Identical ordinary data is OPEN; Binding does not independently close it.
    p.agree(
        &source("let Open={a:1}\n{out:Open,out:{b:2}}"),
        &source(""),
        &json!({"out":{"a":1,"b":2}}),
    );
    let a = source("let Open={a:1}\n#D:{item:{a:1}}\n{open:Open,open:{b:2}}");
    p.agree(
        &a,
        &source("{selected:#D.item}"),
        &json!({"open":{"a":1,"b":2},"selected":{"a":1}}),
    );
    // Separate selected/direct definition uses carry independent incoming groups.
    let a = source("#D:{item:{a:1}}\n#Outer:{child:#D.item}");
    p.agree(
        &a,
        &source("{one:#Outer,two:#D.item,three:#Outer,one:{child:{a:1}}}"),
        &json!({"one":{"child":{"a":1}},"two":{"a":1},"three":{"child":{"a":1}}}),
    );
    p.reject(
        &a,
        &source("{one:#Outer,two:#D.item,one:{child:{b:2}}}"),
        Error::Conflict,
    );
    p.agree(
        &a,
        &source("{one:#Outer,two:#D.item}"),
        &json!({"one":{"child":{"a":1}},"two":{"a":1}}),
    );
    // A fresh group's admitted sets cannot union unrelated keys from another use.
    let a = source("#D:{left:{a:1},right:{b:2}}");
    p.agree(
        &a,
        &source("{one:#D.left,two:#D.right}"),
        &json!({"one":{"a":1},"two":{"b":2}}),
    );
    p.reject(&a, &source("{out:#D.left,out:#D.right}"), Error::Conflict);
    for reverse in [false, true] {
        let out = p.go(&a, &source("{out:#D.left,out:#D.right}"), reverse);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
    }
    p.reject(
        &source("#D:{item:[{a:1}]}"),
        &source("{out:#D.item,out:[{a:2}]}"),
        Error::Conflict,
    );
    p.reject(
        &source("#D:{item:[{a:1}]}"),
        &source("{out:#D.item,out:[]}"),
        Error::Conflict,
    );
    p.agree(
        &source("#D:{item:[]}"),
        &source("{out:#D.item,out:[]}"),
        &json!({"out":[]}),
    );
}

#[test]
fn source_derived_work_copy_encoding_depth_and_all_original_guards() {
    let p = Probe::new();
    for n in [0, 1, 3, 6, 16, 64] {
        let a = source(&format!("#D:{{item:{}}}", object(n)));
        exact(&a, &source("#D.item"), 18 + 12 * n, &wanted_object(n));
        exact(
            &a,
            &source("{out:#D.item}"),
            32 + 15 * n,
            &json!({"out":wanted_object(n)}),
        );
        for m in [1, 2, 4] {
            let total = 9 + n + 17 * m + 11 * n * m;
            if total <= 4096 {
                exact(
                    &a,
                    &source(&repeated("#D.item", m)),
                    total,
                    &json!(vec![wanted_object(n); m]),
                );
            }
        }
    }
    for (q, n) in [(0, 0), (1, 0), (1, 1), (2, 3), (5, 2), (59, 5)] {
        let a = source(&format!("#D:{{item:{}}}", repeated(&object(n), q)));
        exact(
            &a,
            &source("#D.item"),
            18 + 9 * q + 12 * q * n,
            &json!(vec![wanted_object(n); q]),
        );
    }
    let a = source(&format!("#D:{{item:{}}}", object(6)));
    exact(
        &a,
        &source(&repeated("#D.item", 49)),
        4082,
        &json!(vec![wanted_object(6); 49]),
    );
    p.reject(
        &a,
        &source(&repeated("#D.item", 50)),
        Error::Limit("evaluation steps"),
    );
    let a = source(&format!("#D:{{item:{}}}", repeated(&object(5), 60)));
    p.reject(&a, &source("#D.item"), Error::Limit("evaluation steps"));
    assert_eq!(
        export_with_limit(&source("#D:{item:{}}"), &source("#D.item"), 4097),
        Err(Error::Limit("evaluation steps"))
    );
    // Old flat scalar schedules remain exact, including the no-selector fast path.
    for n in [1, 3, 16, 64] {
        let a = source(&format!("#D:{}", object(n)));
        exact(&a, &source("#D.f0"), 13 + 3 * n, &json!(1));
        for m in [1, 2, 4] {
            exact(
                &a,
                &source(&repeated("#D.f0", m)),
                8 + n + 12 * m + 2 * n * m,
                &json!(vec![1; m]),
            );
            let b = source(&format!(
                "{{{}}}",
                (0..m)
                    .map(|i| format!("k{i}:#D.f0"))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            exact(
                &a,
                &b,
                7 + n + 18 * m + 2 * n * m,
                &json!(
                    (0..m)
                        .map(|i| (format!("k{i}"), json!(1)))
                        .collect::<serde_json::Map<_, _>>()
                ),
            );
        }
    }
    exact(&source(""), &source("1"), 3, &json!(1));
    exact(&source("#D:1"), &source("#D"), 6, &json!(1));
    exact(&source("#D:{item:{}}"), &source("[]"), 5, &json!([]));
    // New nested UNSELECTED target costs; Alias adds two original arena visits.
    for (target, total) in [("{x:1,unused:{a:1}}", 24), ("{x:1,unused:[1]}", 23)] {
        let a = source(&format!("#Alias:{{x:1}}\n#D:{target}"));
        exact(&a, &source("#D.x"), total, &json!(1));
        exact(&a, &source("{out:#D.x}"), total + 12, &json!({"out":1}));
    }
    let bytes = |text: &str| source(&format!("#D:{{item:{{s:{}}}}}", json!(text)));
    let list = |m| source(&repeated("#D.item", m));
    let small = export(&bytes("x"), &list(1)).expect("tiny encoded output");
    assert_eq!(small, "[\n  {\n    \"s\": \"x\"\n  }\n]\n");
    assert_eq!(small.len(), 25); // 3 +1*(1+21), copied1*(1+1).
    let a = bytes(&"x".repeat(1002));
    let out = export_with_limit(&a, &list(64), 1802).expect("below both byte caps");
    assert_eq!(out.len(), 65475); // copied64192, encoded3+64*(1002+21).
    assert_eq!(
        serde_json::from_str::<Value>(&out).expect("JSON"),
        json!(vec![json!({"s":"x".repeat(1002)}); 64])
    );
    assert_eq!(
        export_with_limit(&a, &list(64), 1801),
        Err(Error::Limit("evaluation steps"))
    );
    for (b, error) in [
        (1003, "encoded bytes"),
        (1023, "encoded bytes"),
        (1024, "copied strings"),
    ] {
        p.reject(&bytes(&"x".repeat(b)), &list(64), Error::Limit(error));
    }
    // Escaping uses two output bytes per LF; UTF-8 uses three bytes per 雪.
    let out = export(&bytes(&"\n".repeat(500)), &list(64)).expect("escaped below encoded cap");
    assert_eq!(out.len(), 3 + 64 * (2 * 500 + 21));
    p.reject(
        &bytes(&"\n".repeat(502)),
        &list(64),
        Error::Limit("encoded bytes"),
    );
    let out = export(&bytes(&"雪".repeat(334)), &list(64)).expect("UTF-8 below byte cap");
    assert_eq!(out.len(), 65475);
    p.reject(
        &bytes(&"雪".repeat(335)),
        &list(64),
        Error::Limit("encoded bytes"),
    );
    p.reject(
        &bytes(&"雪".repeat(342)),
        &list(64),
        Error::Limit("copied strings"),
    );
    let a = source("#D:{item:{s:1}}");
    let depth = |d| source(&format!("{}#D.item{}", "[".repeat(d), "]".repeat(d)));
    exact(
        &a,
        &depth(30),
        705,
        &(0..30).fold(json!({"s":1}), |v, _| json!([v])),
    );
    p.reject(&a, &depth(31), Error::Limit("output depth"));
    p.reject(&a, &depth(32), Error::Limit("syntax depth"));
    // Independent explicit widths: one target object's 65th field/list item rejects
    // before any preparation; root width has the SAME old64 limit.
    p.reject(
        &source(&format!("#D:{{item:{}}}", object(65))),
        &source("#D.item"),
        Error::Limit("container items"),
    );
    let a = source(&format!("#D:{{item:{}}}", repeated("1", 64)));
    exact(&a, &source("#D.item"), 18 + 6 * 64, &json!(vec![1; 64]));
    p.reject(
        &source(&format!("#D:{{item:{}}}", repeated("1", 65))),
        &source("#D.item"),
        Error::Limit("container items"),
    );
    p.reject(
        &source("#D:{item:{}}"),
        &list(65),
        Error::Limit("container items"),
    );
    let label = "é".repeat(32);
    let a = source(&format!("#D:{{item:{{{}:1}}}}", json!(label)));
    assert_eq!(value(&a, &source("#D.item")), json!({label:1}));
    p.reject(
        &source(&format!("#D:{{item:{{{}:1}}}}", json!("é".repeat(33)))),
        &source("#D.item"),
        Error::Limit("label bytes"),
    );
    let name = format!("#{}", "d".repeat(63));
    let a = source(&format!("{name}:{{item:{{}}}}"));
    assert_eq!(value(&a, &source(&format!("{name}.item"))), json!({}));
    p.reject(
        &source(&format!("#{}:{{item:{{}}}}", "d".repeat(64))),
        &source("{}"),
        Error::Limit("label bytes"),
    );
    let definitions = |n| {
        source(
            &(0..n)
                .map(|i| format!("#D{i}:{{item:{{}}}}\n"))
                .collect::<String>(),
        )
    };
    assert_eq!(value(&definitions(64), &source("#D0.item")), json!({}));
    p.reject(
        &definitions(65),
        &source("#D0.item"),
        Error::Limit("definitions"),
    );
    // USED bindings avoid unused/collision checks as a false resource witness.
    let bindings = |n| {
        source(&format!(
            "#D:{{item:{{}}}}\n{}\n{{used:[{}],out:#D.item}}",
            (0..n).map(|i| format!("let B{i}=1\n")).collect::<String>(),
            (0..n)
                .map(|i| format!("B{i}"))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    assert_eq!(
        value(&bindings(64), &source("")),
        json!({"used":vec![1;64],"out":{}})
    );
    p.reject(&bindings(65), &source(""), Error::Limit("bindings"));
    // Original aggregate syntax can dominate work before unsupported target analysis.
    let wide = format!(
        "{{{}}}",
        (0..32)
            .map(|i| format!("k{i}:{}", repeated("1&1", 22)))
            .collect::<Vec<_>>()
            .join(",")
    );
    p.reject(
        &source(&format!("#D:{{item:{{}}}}\n#Unused:{wide}")),
        &source("#D.item"),
        Error::Limit("syntax nodes"),
    );
    // Redundant parentheses cost tokens even though they add no arena nodes.
    let token_tree_at = |depth| {
        format!(
            "{{{}}}",
            (0..64)
                .map(|i| format!("k{i}:{}1{}", "(".repeat(depth), ")".repeat(depth)))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let token_tree = token_tree_at(29);
    assert_eq!(
        value(
            &source("#D:{item:{}}"),
            &source(&format!("{{out:#D.item,extra:{token_tree}}}"))
        )["out"],
        json!({})
    );
    let too_many_tokens = token_tree_at(30);
    p.reject(
        &source("#D:{item:{}}"),
        &source(&format!("{{out:#D.item,extra:{too_many_tokens}}}")),
        Error::Limit("tokens"),
    );
    let a = source("#D:{item:{}}");
    let b = source("#D.item");
    let padding = 65536 - a.len() - b.len();
    assert_eq!(value(&format!("{a}{}", " ".repeat(padding)), &b), json!({}));
    p.reject(
        &format!("{a}{}", " ".repeat(padding + 1)),
        &b,
        Error::Limit("source bytes"),
    );
    // 1+32*(1+64)=2081 would-be materialized values, but original syntax99 and
    // widths64/32 are legal: shared work4096 dominates T23145 first.
    p.reject(
        &source(&format!("#D:{{item:{}}}", object(64))),
        &list(32),
        Error::Limit("evaluation steps"),
    );
    // The unchanged materialized2048 ceiling is dominated by4096 work. A literal
    // selected64x32 tree has2113 output occurrences; tokens dominate before syntax
    // and work. Never raise any guard to manufacture allocation of2048 values.
    let a = source(&format!("#D:{{item:{}}}", repeated(&repeated("1", 32), 64)));
    p.reject(&a, &source("#D.item"), Error::Limit("tokens"));
    assert_eq!(
        value(&source("#D:{item:[{}]}"), &source("{out:#D.item}")),
        json!({"out":[{}]})
    );
}
