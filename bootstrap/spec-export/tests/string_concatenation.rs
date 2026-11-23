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
            "string-concatenation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let native = PathBuf::from(env!("CARGO_BIN_EXE_cuestacean-spec-export"));
        executable(&native);
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned tool PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned CUE")
            .canonicalize()
            .expect("absolute pinned CUE");
        executable(&cue);
        Self { dir, native, cue }
    }
    fn run(&self, tool: &Path, args: &[&str], a: &str, b: &str, reverse: bool) -> Output {
        executable(tool);
        let mut paths = vec![
            self.dir.join("unrelated-owner.cue"),
            self.dir.join("arbitrary-use.cue"),
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
    // Every caller supplies object-root FILES: scalar/list accounting is API-only.
    fn go(&self, a: &str, b: &str, reverse: bool) -> Output {
        self.run(&self.cue, &["export"], a, b, reverse)
    }
    fn agree(&self, a: &str, b: &str, want: &Value) {
        for reverse in [false, true] {
            let (first, second) = if reverse { (b, a) } else { (a, b) };
            assert_eq!(value(first, second), *want);
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
            let (first, second) = if reverse { (b, a) } else { (a, b) };
            assert_eq!(
                export(first, second),
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
            assert_eq!(
                decoded(self.native(&source(""), &source("{recovered:\"o\"+\"k\"}"), reverse)),
                json!({"recovered":"ok"})
            );
            assert_eq!(value(&source(""), &source("\"o\"+\"k\"")), json!("ok"));
        }
    }
    fn go_failure(&self, a: &str, b: &str, diagnostic: &str, eval_ok: bool) {
        for reverse in [false, true] {
            let out = self.go(a, b, reverse);
            assert_eq!(out.status.code(), Some(1));
            assert!(out.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&out.stderr).contains(diagnostic),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let out = self.run(&self.cue, &["eval"], a, b, reverse);
            assert_eq!(out.status.success(), eval_ok, "separate eval/export phase");
            assert_eq!(
                decoded(self.go(&source(""), &source("{ok:\"a\"+\"b\"}"), reverse)),
                json!({"ok":"ab"})
            );
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire owned scratch");
    }
}
fn exact(a: &str, b: &str, total: usize, want: &Value) {
    assert!((1..=4096).contains(&total), "never raise hard work guard");
    for (a, b) in [(a, b), (b, a)] {
        assert_eq!(
            export_with_limit(a, b, total - 1),
            Err(Error::Limit("evaluation steps"))
        );
        let text = export_with_limit(a, b, total).expect("source-derived exact allowance");
        assert_eq!(serde_json::from_str::<Value>(&text).expect("JSON"), *want);
        assert_eq!(value(a, b), *want, "fresh recovery");
    }
}
fn repeated(item: &str, n: usize) -> String {
    format!("[{}]", vec![item; n].join(","))
}
fn chain(c: usize) -> String {
    vec!["\"x\""; c + 1].join("+")
}
fn target(n: usize, text: &str) -> String {
    source(&format!(
        "#D:{{{}}}",
        (0..n)
            .map(|i| format!("f{i}:{}", json!(text)))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

#[test]
fn independent_values_original_operands_grouping_precedence_and_constraint_results() {
    let p = Probe::new();
    for (expression, want) in [
        ("\"\"+\"\"", ""),
        ("\"\"+\"snow\"", "snow"),
        ("\"雪\"+\"é🦀\"", "雪é🦀"),
        (r#""\u96ea"+"\ud83e\udd80""#, "雪🦀"),
        (r#""\"\\\n"+"\t\/\b\f\r""#, "\"\\\n\t/\u{8}\u{c}\r"),
        ("\"a\"+\"b\"+\"c\"", "abc"),
        ("(\"a\"+\"b\")+\"c\"", "abc"),
        ("\"a\"+(\"b\"+\"c\")", "abc"),
        ("(((\"left\"))) + ((\"right\"))", "leftright"),
        ("\"a\"+\n\"b\"", "ab"),
        ("\"a\"\r+\r\"b\"", "ab"),
        ("\"a\"+ // LF ends comment\n\"b\"", "ab"),
        ("\"a\"+\"b\" & \"ab\"", "ab"),
        ("\"ab\" & \"a\"+\"b\"", "ab"),
        ("(\"a\"+\"b\") | \"ab\"", "ab"),
        ("(\"a\"+\"b\" | \"other\") & \"ab\"", "ab"),
    ] {
        p.agree(
            &source(""),
            &source(&format!("{{out:{expression}}}")),
            &json!({"out":want}),
        );
    }
    let multiline = "\"\"\"\n\tfirst\n\t 雪\n\t\"\"\" + \"\\n\"";
    p.agree(
        &source(""),
        &source(&format!("{{header:{multiline}}}")),
        &json!({"header":"first\n 雪\n"}),
    );
    p.agree(
        &source(""),
        &source("{out:\"\"\"\n\"\"\"+\"\"\"\nlast\n\"\"\"}"),
        &json!({"out":"last"}),
    );
    p.agree(
        &source("#Text:{header:\"\"\"\n  generic header\n  \"\"\",footer:\"end\"}"),
        &source("{header:#Text.header+\"\\n\",footer:#Text.footer}"),
        &json!({"header":"generic header\n","footer":"end"}),
    );
    for name in ["#Arbitrary", "#Renamed", "#NotAnOpcode"] {
        let a = source(&format!(
            "{name}:{{first:\"λ\",last:\"雪\",unused:[{{ok:true}}]}}"
        ));
        let b = source(&format!(
            "{{one:({name}.first)+{name}.last,two:{name}.last+\"\",three:\"\"+{name}.first}}"
        ));
        p.agree(&a, &b, &json!({"one":"λ雪","two":"雪","three":"λ"}));
    }
    // Forward definitions and lexical capture may CONTAIN concat; the Ref/Binding
    // itself is not used as a concat operand.
    let a = source("let Captured=\"left\"+\"!\"\n#Wrapper:{text:Captured}\n#Ordinary:\"x\"+\"y\"");
    let b = source(
        "let Captured=\"foreign\"+\"?\"\n{first:#Wrapper,second:#Wrapper,own:Captured,ordinary:#Ordinary}",
    );
    p.agree(&a, &b, &json!({"first":{"text":"left!"},"second":{"text":"left!"},"own":"foreign?","ordinary":"xy"}));
    p.agree(
        &source("#Before:{s:#Later.x+\"!\"}\n#Later:{x:\"forward\"}"),
        &source("{out:#Before}"),
        &json!({"out":{"s":"forward!"}}),
    );
    for expression in ["\"a\"+\"b\" & \"wrong\"", "\"wrong\" & \"a\"+\"b\""] {
        let b = source(&format!("{{out:{expression}}}"));
        p.reject(&source(""), &b, Error::Conflict);
        p.go_failure(&source(""), &b, "conflicting values", false);
    }
    for expression in [
        "\"a\"+\"b\" | \"other\"",
        "\"other\" | \"a\"+\"b\"",
        "\"a\"+\"b\" | string",
    ] {
        let b = source(&format!("{{out:{expression}}}"));
        p.reject(&source(""), &b, Error::Incomplete);
        p.go_failure(&source(""), &b, "incomplete value", true);
    }
    // Unmarked ordinary constraint composition includes types/inequality, without
    // backward solving through an abstract concat operand.
    p.agree(
        &source(""),
        &source("{out:(\"a\"+\"b\") & string & !=\"bad\"}"),
        &json!({"out":"ab"}),
    );
}

#[test]
fn exact_operand_original_target_whole_document_and_cli_error_phases() {
    let p = Probe::new();
    let operand = Error::Unsupported("string concatenation operand");
    let profile = Error::Unsupported("string concatenation document profile");
    for expression in [
        "1+1",
        "true+\"x\"",
        "\"x\"+false",
        "{}+\"x\"",
        "\"x\"+[]",
        "string+\"x\"",
        "\"x\"+int",
        "(\"x\"&\"x\")+\"y\"",
        "\"x\"+(\"y\"|\"y\")",
        "(!=\"q\")+\"x\"",
        "[...string]+\"x\"",
        "(\"a\"+\"b\")+1",
    ] {
        p.reject(
            &source(""),
            &source(&format!("{{out:{expression}}}")),
            operand.clone(),
        );
    }
    for (owner, expression) in [
        ("#D:\"x\"", "#D+\"y\""),
        ("#D:\"x\"+\"y\"", "#D+\"z\""),
        ("#D:{x:1}", "#D.x+\"y\""),
        ("#D:{x:false}", "\"y\"+#D.x"),
        ("#D:{x:{}}", "#D.x+\"y\""),
        ("#D:{x:[]}", "\"y\"+#D.x"),
    ] {
        p.reject(
            &source(owner),
            &source(&format!("{{out:{expression}}}")),
            operand.clone(),
        );
    }
    for body in [
        "let Local=\"x\"\n{out:Local+\"y\"}",
        "let Local=\"x\"+\"y\"\n{out:Local+\"z\"}",
    ] {
        p.reject(&source(""), &source(body), operand.clone());
    }
    // Go permits these original references/aliases, which this narrower class
    // rejects before evaluation. Lexical uses stay in their declaring FILE.
    for (owner, use_site) in [
        ("#D:\"x\"", "{out:#D+\"y\"}"),
        ("", "let Local=\"x\"\n{out:Local+\"y\"}"),
    ] {
        let a = source(owner);
        let b = source(use_site);
        for reverse in [false, true] {
            assert_eq!(decoded(p.go(&a, &b, reverse)), json!({"out":"xy"}));
        }
    }
    // Whole-arena mark/comprehension admission precedes ANY concat rewrite,
    // even if the excluded declaration/empty template is not demanded.
    for body in [
        "#Unused:*\"a\"|\"b\"\n{out:\"x\"+\"y\"}",
        "{out:\"x\"+\"y\",other:*1|int}",
        "{out:*(\"x\"+\"y\")|\"z\"}",
        "let T={}\n#Unused:{for k,v in T {\"\\(k)\":v}}\n{out:\"x\"+\"y\"}",
        "let T={a:\"x\"}\n{out:\"x\"+\"y\",other:{for k,v in T {\"\\(k)\":v}}}",
    ] {
        p.reject(&source(body), &source(""), profile.clone());
        for (a, b) in [(source(body), source("")), (source(""), source(body))] {
            assert_eq!(
                export_with_limit(&a, &b, 0),
                Err(Error::Limit("evaluation steps")),
                "entry precharge precedes document-profile admission"
            );
            assert_eq!(export_with_limit(&a, &b, 1), Err(profile.clone()));
        }
    }
    // Invalid ORIGINAL target contents are never sanitized by folding a concat
    // or replacing an earlier projection/Binding, selected or unselected.
    for owner in [
        "#D:{x:\"a\"+\"b\"}",
        "#D:{x:\"a\",unused:\"b\"+\"c\"}",
        "#D:{x:\"a\",unused:[{nested:\"b\"+\"c\"}]}",
        "#Other:{s:\"b\"}\n#D:{x:#Other.s}",
        "let Local=\"a\"\n#D:{x:Local}",
        "#Other:\"a\"\n#D:{x:#Other}",
    ] {
        p.reject(
            &source(owner),
            &source("{out:#D.x+\"!\"}"),
            Error::Unsupported("literal projection scalar required"),
        );
    }
    p.reject(
        &source("#D:\"a\""),
        &source("{out:#D.x+\"!\"}"),
        Error::Unsupported("literal projection struct required"),
    );
    p.reject(
        &source("#D:{x:\"a\",unused:{a:1,a:1}}"),
        &source("{out:#D.x+\"!\"}"),
        Error::Unsupported("duplicate projection target field"),
    );
    // Arena order deliberately differs here: old document rejection versus
    // whole-target traversal. Each order pins its precise reached phase.
    p.reject_orders(
        &source("#D:{x:\"a\",unused:string}"),
        &source("{out:#D.x+\"!\"}"),
        Error::Unsupported("scalar projection document profile"),
        Error::Unsupported("literal projection scalar required"),
    );
    for owner in [
        "#D:{x:\"a\",unused:string}",
        "#D:{x:\"a\",unused:\"b\"+\"c\"}",
    ] {
        for reverse in [false, true] {
            assert_eq!(
                decoded(p.go(&source(owner), &source("{out:#D.x+\"!\"}"), reverse)),
                json!({"out":"a!"}),
                "Go may ignore or compute an unselected sibling; native original-target rejection is deliberate"
            );
        }
    }
    for extra in [
        "#Unused:string",
        "#Unused:1&1",
        "#Unused:1|1",
        "#Unused:[...int]",
    ] {
        p.reject(
            &source(&format!("#D:{{x:\"a\"}}\n{extra}")),
            &source("{out:#D.x+\"!\"}"),
            Error::Unsupported("scalar projection document profile"),
        );
    }
    p.reject(
        &source("#D:{x:\"a\"}"),
        &source("{out:(#D.x+\"!\")&\"a!\"}"),
        Error::Unsupported("scalar projection document profile"),
    );
    // Unused concat operands are checked too, even though Go need not demand them.
    p.reject(
        &source("#Unused:1+1\n{out:\"x\"+\"y\"}"),
        &source(""),
        operand.clone(),
    );
    p.reject(
        &source("#D:{x:\"a\"}"),
        &source("{out:#Missing.x+\"!\"}"),
        Error::Unknown("#Missing".into()),
    );
    p.reject(
        &source("#D:{x:\"a\"}"),
        &source("{out:#D.absent+\"!\"}"),
        Error::Unknown("#D.absent".into()),
    );
    p.reject(
        &source("#D:{}"),
        &source("{out:#D.x+\"!\"}"),
        Error::Unknown("#D.x".into()),
    );
    p.reject(
        &source(""),
        &source("let Local=#D.x+\"!\"\n#D:{x:Local}\n{out:Local}"),
        Error::Unsupported("cyclic binding dependency"),
    );
    p.reject(
        &source(""),
        &source("let A=B+\"!\"\nlet B=A+\"?\"\n{out:A}"),
        Error::Unsupported("cyclic binding dependency"),
    );
    p.reject(
        &source("let Unused=\"a\"+\"b\""),
        &source("{out:1}"),
        Error::Unsupported("unused binding"),
    );
    p.reject(
        &source("let Name=\"a\"+\"b\"\n{Name:Name}"),
        &source(""),
        Error::Unsupported("binding/field collision"),
    );
    p.reject(
        &source("#Loop:#Loop\n{out:#Loop,computed:\"a\"+\"b\"}"),
        &source(""),
        Error::Unsupported("unproductive recursive definition"),
    );
    for (expression, error) in [
        (
            "(#D).x+\"!\"",
            Error::Unsupported("direct scalar projection required"),
        ),
        (
            "#D.x.x+\"!\"",
            Error::Unsupported("direct scalar projection required"),
        ),
        (
            "#D.\"x\"+\"!\"",
            Error::Unsupported("regular scalar projection field"),
        ),
        (
            "#D._x+\"!\"",
            Error::Unsupported("regular scalar projection field"),
        ),
        (
            "#D.#X+\"!\"",
            Error::Unsupported("regular scalar projection field"),
        ),
        (
            "#D\r.x+\"!\"",
            Error::Unsupported("direct scalar projection required"),
        ),
        (
            "#D.\rx+\"!\"",
            Error::Unsupported("direct scalar projection required"),
        ),
        (
            "#D\n.x+\"!\"",
            Error::Unsupported("direct scalar projection required"),
        ),
        (
            "#D.\nx+\"!\"",
            Error::Unsupported("regular scalar projection field"),
        ),
        ("#D[\"x\"]+\"!\"", Error::Syntax("field separator")),
    ] {
        p.reject(
            &source("#D:{x:\"a\"}"),
            &source(&format!("{{out:{expression}}}")),
            error,
        );
    }
    p.reject(
        &source(""),
        &source("\"a\"\n+\"b\""),
        Error::Syntax("expected expression"),
    );
    p.reject(
        &source(""),
        &source("\"a\"+"),
        Error::Syntax("expected expression"),
    );
    p.reject(
        &source(""),
        &source("let T={}\n{for k,v in T {\"\\(k)\":\"\\(v+v)\"}}"),
        Error::Unsupported("string interpolation name"),
    );
    p.reject(
        &source(""),
        &source("{out:\"a\"+\"\\(missing)\"}"),
        Error::Unsupported("key interpolation context"),
    );
    // I/O/usage setup errors are exercised separately, never called semantic
    // negatives or upstream disagreements.
    for paths in [
        vec![p.dir.join("missing.cue")],
        vec![p.dir.join("a.cue"), p.dir.join("b.cue")],
    ] {
        executable(&p.native);
        let out = Command::new(&p.native)
            .args(paths)
            .env_clear()
            .env("HOME", &p.dir)
            .env("PATH", "/unusable")
            .output()
            .expect("CLI setup probe");
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
    p.agree(
        &source(""),
        &source("{out:\"after\"+\" errors\"}"),
        &json!({"out":"after errors"}),
    );
}

#[test]
fn selected_container_closedness_incoming_groups_reuse_and_outer_openness_with_concat() {
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
        let a = source(&format!(
            "#D:{{item:{item},word:\"a\"}}\n#Outer:{{child:#D.item,text:#D.word+\"!\"}}"
        ));
        for backwards in [false, true] {
            let fields = if backwards {
                format!("out:{compatible},out:#D.item")
            } else {
                format!("out:#D.item,out:{compatible}")
            };
            p.agree(
                &a,
                &source(&format!(
                    "{{{fields},text:#D.word+\"!\",outside:2,again:#D.item}}"
                )),
                &json!({"out":want,"text":"a!","outside":2,"again":want}),
            );
            let fields = if backwards {
                format!("out:{extra},out:#D.item")
            } else {
                format!("out:#D.item,out:{extra}")
            };
            let b = source(&format!("{{{fields},text:#D.word+\"!\"}}"));
            p.reject(&a, &b, Error::Conflict);
            p.go_failure(&a, &b, "field not allowed", false);
        }
        p.agree(&a, &source("{one:#Outer,two:#D.item,three:#Outer,whole:#D}"), &json!({"one":{"child":want,"text":"a!"},"two":want,"three":{"child":want,"text":"a!"},"whole":{"item":want,"word":"a"}}));
    }
    p.agree(
        &source("#D:{item:{a:1},word:\"x\"}"),
        &source("let Open={a:1}\n{open:Open,open:{b:2},selected:#D.item,text:#D.word+\"y\"}"),
        &json!({"open":{"a":1,"b":2},"selected":{"a":1},"text":"xy"}),
    );
    // File-local lexical capture can cross files through a definition.
    let a = source("let Open={a:1}\n#Wrap:{inside:Open}\n#D:{left:{a:1},right:{b:2},word:\"x\"}");
    p.agree(
        &a,
        &source("{one:#D.left,two:#D.right,wrapped:#Wrap,text:#D.word+\"y\"}"),
        &json!({"one":{"a":1},"two":{"b":2},"wrapped":{"inside":{"a":1}},"text":"xy"}),
    );
    p.reject(
        &a,
        &source("{out:#D.left,out:#D.right,text:#D.word+\"y\"}"),
        Error::Conflict,
    );
    let a = source("#D:{item:[{a:1}],word:\"x\"}\n#Outer:{child:#D.item,text:#D.word+\"y\"}");
    p.agree(&a, &source("{one:#Outer,two:#D.item,three:#Outer,one:{child:[{a:1}]}}"), &json!({"one":{"child":[{"a":1}],"text":"xy"},"two":[{"a":1}],"three":{"child":[{"a":1}],"text":"xy"}}));
    p.reject(
        &a,
        &source("{one:#Outer,two:#D.item,one:{child:[{b:2}]}}"),
        Error::Conflict,
    );
    p.reject(
        &a,
        &source("{out:#D.item,out:[{a:2}],text:#D.word+\"y\"}"),
        Error::Conflict,
    );
    p.reject(
        &a,
        &source("{out:#D.item,out:[],text:#D.word+\"y\"}"),
        Error::Conflict,
    );
    p.agree(
        &source("#D:{item:[],word:\"x\"}"),
        &source("{out:#D.item,out:[],text:#D.word+\"y\"}"),
        &json!({"out":[],"text":"xy"}),
    );
}

#[test]
fn seven_charge_source_derived_work_copy_intermediates_encoding_and_unchanged_guards() {
    let p = Probe::new();
    // Literal chain: N=2C+1, preparation=1+N+7C=2+9C;
    // folded scalar evaluation=3. These are formulas, not threshold searches.
    exact(&source(""), &source("\"\"+\"\""), 14, &json!(""));
    exact(&source(""), &source("\"\"+\"\"+\"\""), 23, &json!(""));
    for c in [1, 2, 3, 15, 30, 31] {
        exact(
            &source(""),
            &source(&chain(c)),
            5 + 9 * c,
            &json!("x".repeat(c + 1)),
        );
    }
    for n in [1, 2, 8, 23, 64] {
        let a = target(n, "x");
        exact(&a, &source("#D.f0+\"\""), 19 + 3 * n, &json!("x"));
        for m in [1, 2, 4] {
            exact(
                &a,
                &source(&repeated("#D.f0+\"\"", m)),
                8 + n + 18 * m + 2 * n * m,
                &json!(vec!["x"; m]),
            );
            let fields = (0..m)
                .map(|i| format!("k{i}:#D.f0+\"\""))
                .collect::<Vec<_>>()
                .join(",");
            let want = (0..m)
                .map(|i| (format!("k{i}"), json!("x")))
                .collect::<serde_json::Map<_, _>>();
            exact(
                &a,
                &source(&format!("{{{fields}}}")),
                7 + n + 21 * m + 2 * n * m,
                &json!(want),
            );
        }
    }
    for (c, m) in [(1, 1), (2, 3), (5, 7), (30, 14)] {
        exact(
            &source(""),
            &source(&repeated(&chain(c), m)),
            7 + m * (9 * c + 5),
            &json!(vec!["x".repeat(c + 1); m]),
        );
    }
    exact(
        &target(23, "x"),
        &source(&repeated("#D.f0+\"\"", 63)),
        4063,
        &json!(vec!["x"; 63]),
    );
    // m64 T=4127, not a request to raise the API's hard4096 cap.
    p.reject(
        &target(23, "x"),
        &source(&repeated("#D.f0+\"\"", 64)),
        Error::Limit("evaluation steps"),
    );
    p.reject(
        &source(""),
        &source(&repeated(&chain(30), 15)),
        Error::Limit("evaluation steps"),
    ); // T4132.
    assert_eq!(
        export_with_limit(&source(""), &source("\"a\"+\"b\""), 4097),
        Err(Error::Limit("evaluation steps"))
    );
    // All old no-concat dispatch/costs remain exact, not merely successful.
    exact(&source(""), &source("\"old\""), 3, &json!("old"));
    exact(&source("#D:\"old\""), &source("#D"), 6, &json!("old"));
    exact(&source(""), &source("[]"), 5, &json!([]));
    for n in [1, 3, 16, 64] {
        let a = target(n, "old");
        exact(&a, &source("#D.f0"), 13 + 3 * n, &json!("old"));
        for m in [1, 2, 4] {
            exact(
                &a,
                &source(&repeated("#D.f0", m)),
                8 + n + 12 * m + 2 * n * m,
                &json!(vec!["old"; m]),
            );
            let fields = (0..m)
                .map(|i| format!("k{i}:#D.f0"))
                .collect::<Vec<_>>()
                .join(",");
            let want = (0..m)
                .map(|i| (format!("k{i}"), json!("old")))
                .collect::<serde_json::Map<_, _>>();
            exact(
                &a,
                &source(&format!("{{{fields}}}")),
                7 + n + 18 * m + 2 * n * m,
                &json!(want),
            );
        }
    }
    exact(
        &source("#D:{item:{a:1}}"),
        &source("#D.item"),
        30,
        &json!({"a":1}),
    );
    // Tiny byte witnesses establish pretty encoding independently first.
    let list = |m| source(&repeated("#D.f0+\"\"", m));
    let tiny = export(&target(1, "x"), &list(1)).expect("tiny bytes");
    assert_eq!(tiny, "[\n  \"x\"\n]\n");
    assert_eq!(tiny.len(), 10); // 3 +1*(1+6); preparation1+final1 copied.
    let tiny = export(&target(1, "\n"), &list(1)).expect("tiny escaped bytes");
    assert_eq!(tiny, "[\n  \"\\n\"\n]\n");
    assert_eq!(tiny.len(), 11);
    let a = target(1, &"x".repeat(512));
    let out = export(&a, &list(64)).expect("copied EXACT65536: 64*512 preparation plus final");
    assert_eq!(out.len(), 3 + 64 * (512 + 6));
    assert_eq!(
        serde_json::from_str::<Value>(&out).expect("JSON"),
        json!(vec!["x".repeat(512); 64])
    );
    p.reject(
        &target(1, &"x".repeat(513)),
        &list(64),
        Error::Limit("copied strings"),
    );
    let out = export(&target(1, &"\n".repeat(508)), &list(64)).expect("copied65024 encoded65411");
    assert_eq!(out.len(), 65411);
    assert_eq!(
        serde_json::from_str::<Value>(&out).expect("JSON"),
        json!(vec!["\n".repeat(508); 64])
    );
    p.reject(
        &target(1, &"\n".repeat(509)),
        &list(64),
        Error::Limit("encoded bytes"),
    ); // 65539, copied65152.
    let out = export(&target(1, &"雪".repeat(170)), &list(64))
        .expect("UTF-8 copied65280, not character counts");
    assert_eq!(out.len(), 3 + 64 * (510 + 6));
    assert_eq!(
        serde_json::from_str::<Value>(&out).expect("JSON"),
        json!(vec!["雪".repeat(170); 64])
    );
    p.reject(
        &target(1, &"雪".repeat(171)),
        &list(64),
        Error::Limit("copied strings"),
    );
    // Ordinary final object keys remain on the SAME copied-string ledger.
    let object = |m| {
        source(&format!(
            "{{{}}}",
            (0..m)
                .map(|i| format!("k{i}:#D.f0+\"\""))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    p.reject(
        &target(1, &"x".repeat(512)),
        &object(64),
        Error::Limit("copied strings"),
    );
    // Equal values have different intermediate copies: ((x+y)+empty) copies
    // 6L per result including final; x+(y+empty) copies5L. L171,m64:
    // left65664 rejects; right54720 succeeds. Neither escapes by folding early.
    let a = source(&format!(
        "#D:{{x:{},y:{}}}",
        json!("x".repeat(171)),
        json!("y".repeat(171))
    ));
    let right = source(&repeated("#D.x+(#D.y+\"\")", 64));
    for expression in ["(#D.x+#D.y)+\"\"", "#D.x+#D.y+\"\""] {
        p.reject(
            &a,
            &source(&repeated(expression, 64)),
            Error::Limit("copied strings"),
        );
    }
    let want = json!(vec![format!("{}{}", "x".repeat(171), "y".repeat(171)); 64]);
    assert_eq!(value(&a, &right), want);
    assert_eq!(value(&right, &a), want);
    // Undemanded intermediate results are still charged. Two copies of32768
    // EXACTLY exhaust65536 before root materialization, which adds zero for1.
    let unused = |n| source(&format!("#Unused:{}+\"\"+\"\"", json!("x".repeat(n))));
    exact(&unused(32768), &source("1"), 24, &json!(1)); // N6, C2: A21+E3.
    p.reject(&unused(32769), &source("1"), Error::Limit("copied strings"));
    p.reject(
        &unused(32768),
        &source("\"r\""),
        Error::Limit("copied strings"),
    );
    assert_eq!(value(&unused(32767), &source("\"r\"")), json!("r"));
    // Complete original validation must finish BEFORE any allocation/copy:
    // the huge unused chain would exceed65536, but a later invalid operand
    // rejects first, independently of which FILE is parsed first.
    p.reject(
        &source(&format!(
            "#Huge:{}+\"\"+\"\"\n#Bad:1+1",
            json!("x".repeat(40000))
        )),
        &source("{out:1}"),
        Error::Unsupported("string concatenation operand"),
    );
    // Chain associations with empty and unequal operands retain seven charges,
    // while intermediate result bytes (4+6 vs3+6) differ, final6 in both.
    for expression in ["(\"abc\"+\"d\")+\"ef\"", "\"abc\"+(\"d\"+\"ef\")"] {
        exact(&source(""), &source(expression), 23, &json!("abcdef"));
    }
    // Shared hard syntax and parse depth; parentheses add no arena nodes.
    p.reject(
        &source(""),
        &source(&chain(32)),
        Error::Limit("syntax depth"),
    );
    let grouped = |n| source(&format!("{}\"a\"+\"b\"{}", "(".repeat(n), ")".repeat(n)));
    exact(&source(""), &grouped(30), 14, &json!("ab"));
    p.reject(&source(""), &grouped(31), Error::Limit("syntax depth"));
    // Selected output depth can still exceed32 despite syntax32. Keep concat
    // alongside it, not INSIDE the original pure-literal target.
    let a = source("#D:{item:{s:1}}");
    let nested = |d| {
        source(&format!(
            "{{computed:\"a\"+\"b\",deep:{}#D.item{}}}",
            "[".repeat(d),
            "]".repeat(d)
        ))
    };
    let want = (0..29).fold(json!({"s":1}), |v, _| json!([v]));
    p.agree(&a, &nested(29), &json!({"computed":"ab","deep":want}));
    p.reject(&a, &nested(30), Error::Limit("output depth"));
    p.reject(&a, &nested(31), Error::Limit("syntax depth"));
    // Width64/65, definition64/65, binding64/65 are unchanged independent guards.
    exact(&target(64, "x"), &source("#D.f0+\"\""), 211, &json!("x"));
    p.reject(
        &target(65, "x"),
        &source("#D.f0+\"\""),
        Error::Limit("container items"),
    );
    p.reject(&target(1, "x"), &list(65), Error::Limit("container items"));
    let defs = |n| {
        source(
            &(0..n)
                .map(|i| format!("#D{i}:{{x:\"a\"}}\n"))
                .collect::<String>(),
        )
    };
    assert_eq!(value(&defs(64), &source("#D0.x+\"b\"")), json!("ab"));
    p.reject(
        &defs(65),
        &source("#D0.x+\"b\""),
        Error::Limit("definitions"),
    );
    let bindings = |n| {
        source(&format!(
            "{}\n{{used:[{}],computed:\"a\"+\"b\"}}",
            (0..n).map(|i| format!("let B{i}=1\n")).collect::<String>(),
            (0..n)
                .map(|i| format!("B{i}"))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    assert_eq!(
        value(&bindings(64), &source("")),
        json!({"used":vec![1;64],"computed":"ab"})
    );
    p.reject(&bindings(65), &source(""), Error::Limit("bindings"));
    let name = format!("#{}", "d".repeat(63));
    assert_eq!(
        value(
            &source(&format!("{name}:{{x:\"a\"}}")),
            &source(&format!("{name}.x+\"b\""))
        ),
        json!("ab")
    );
    p.reject(
        &source(&format!("#{}:{{x:\"a\"}}", "d".repeat(64))),
        &source("\"a\"+\"b\""),
        Error::Limit("label bytes"),
    );
    let fields = |label: &str| source(&format!("{{{}:\"a\"+\"b\"}}", json!(label)));
    assert_eq!(
        value(&source(""), &fields(&"é".repeat(32))),
        json!({"é".repeat(32):"ab"})
    );
    p.reject(
        &source(""),
        &fields(&"é".repeat(33)),
        Error::Limit("label bytes"),
    );
    // Aggregate ORIGINAL syntax:34*(2*29+1)+1 +39+1 +outer1=2048.
    // Work dominates at2048, but2049 is rejected by syntax BEFORE preparation.
    let syntax = |extra| {
        source(&format!(
            "[{},{}]",
            repeated(&chain(29), 34),
            repeated("1", extra)
        ))
    };
    for (a, b) in [(&source(""), syntax(39)), (&source(""), syntax(40))] {
        let expected = if b == syntax(39) {
            "evaluation steps"
        } else {
            "syntax nodes"
        };
        assert_eq!(export_with_limit(a, &b, 0), Err(Error::Limit(expected)));
        assert_eq!(export_with_limit(&b, a, 0), Err(Error::Limit(expected)));
    }
    p.reject(&source(""), &syntax(40), Error::Limit("syntax nodes"));
    let tokens = |d| {
        source(&format!(
            "{{computed:\"a\"+\"b\",extra:{{{}}}}}",
            (0..64)
                .map(|i| format!("k{i}:{}1{}", "(".repeat(d), ")".repeat(d)))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    assert_eq!(value(&source(""), &tokens(29))["computed"], json!("ab"));
    p.reject(&source(""), &tokens(30), Error::Limit("tokens"));
    let a = source("");
    let b = source("\"a\"+\"b\"");
    let padding = 65536 - a.len() - b.len();
    exact(&format!("{a}{}", " ".repeat(padding)), &b, 14, &json!("ab"));
    p.reject(
        &format!("{a}{}", " ".repeat(padding + 1)),
        &b,
        Error::Limit("source bytes"),
    );
    // Materialized2048 is still dominated by work4096. No guard is raised to
    // manufacture a reachable2048 allocation: 32 selected64-field objects have
    // 2081 occurrences plus the outer object/computed scalar; original syntax
    // and all explicit widths stay legal, but shared work rejects first.
    let tree = format!(
        "{{{}}}",
        (0..64)
            .map(|i| format!("k{i}:1"))
            .collect::<Vec<_>>()
            .join(",")
    );
    p.reject(
        &source(&format!("#D:{{item:{tree}}}")),
        &source(&format!(
            "{{computed:\"a\"+\"b\",items:{}}}",
            repeated("#D.item", 32)
        )),
        Error::Limit("evaluation steps"),
    );
    p.agree(
        &source(""),
        &source("{final:\"still\"+\" works\"}"),
        &json!({"final":"still works"}),
    );
}
