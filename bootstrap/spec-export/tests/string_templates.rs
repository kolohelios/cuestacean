use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PACKAGE: &str = "package p\n";
fn source(body: &str) -> String {
    format!("{PACKAGE}{body}\n")
}
fn comp(rows: usize, template: &str) -> String {
    let fields = (0..rows)
        .map(|i| format!("\"r{i}\":\"v{i}\""))
        .collect::<Vec<_>>()
        .join(",");
    source(&format!(
        "let T={{{fields}}}\n{{for k,v in T {{\"\\(k)\":{template}}}}}"
    ))
}
fn decoded(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("complete JSON")
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    fs::File::open(path).expect("readable tool");
    assert_ne!(
        fs::metadata(path).expect("metadata").permissions().mode() & 0o111,
        0
    );
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "string-templates-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh owned HOME");
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned CUE")
            .canonicalize()
            .expect("absolute CUE");
        executable(&cue);
        executable(Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export")));
        Self { dir, cue }
    }
    fn run(&self, program: &str, oracle: bool, reverse: bool) -> Output {
        let paths = [self.dir.join("helper.cue"), self.dir.join("renamed.cue")];
        fs::write(&paths[0], PACKAGE).expect("package");
        fs::write(&paths[1], program).expect("source");
        let tool = if oracle {
            &self.cue
        } else {
            Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
        };
        executable(tool);
        let mut command = Command::new(tool);
        if oracle {
            command.arg("export");
        }
        for index in if reverse { [1, 0] } else { [0, 1] } {
            command.arg(&paths[index]);
        }
        command
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", "/unusable")
            .current_dir(&self.dir)
            .output()
            .expect("mandatory process setup")
    }
    fn check(&self, body: &str, want: Value) {
        let program = source(body);
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (program.as_str(), PACKAGE)
            } else {
                (PACKAGE, program.as_str())
            };
            assert_eq!(
                serde_json::from_str::<Value>(&export(a, b).expect("native API")).expect("JSON"),
                want
            );
            for oracle in [false, true] {
                assert_eq!(decoded(self.run(&program, oracle, reverse)), want);
            }
        }
    }
    fn reject(&self, body: &str) {
        let program = source(body);
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (program.as_str(), PACKAGE)
            } else {
                (PACKAGE, program.as_str())
            };
            assert!(
                matches!(export(a, b), Err(Error::Unsupported(_))),
                "{body}: {:?}",
                export(a, b)
            );
            let out = self.run(&program, false, reverse);
            assert!(!out.status.success() && out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).starts_with("unsupported: "));
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn arbitrary_strings_slots_affixes_and_json_literal_pieces() {
    let probe = Probe::new();
    let rows = [
        ("a", ""),
        ("雪", "λ"),
        ("quote\"", "line\nslash\\(outer)\0"),
    ];
    let fields = rows
        .iter()
        .map(|(k, v)| format!("{}:{}", json!(k), json!(v)))
        .collect::<Vec<_>>()
        .join(",");
    // Serialize ONLY the two literal pieces; insert the executable raw placeholder
    // between their quote boundaries. Expected values use original input strings.
    for (prefix, suffix) in [
        ("", ""),
        ("runtime::", "!"),
        ("\"\\\n\t\0", "雪\r\u{8}\u{c}"),
        ("literal\\(outer)", "\\(k)"),
    ] {
        let p = json!(prefix).to_string();
        let s = json!(suffix).to_string();
        for key_slot in [false, true] {
            let mut want = json!({});
            for (key, value) in rows {
                let selected = if key_slot { key } else { value };
                want[key] = json!({"rendered":format!("{prefix}{selected}{suffix}"),"original":[key,value]});
            }
            for (table, key, value) in [("T", "k", "v"), ("Helper", "key", "value")] {
                let slot = if key_slot { key } else { value };
                let template = format!("{}\\({slot}){}", &p[..p.len() - 1], &s[1..]);
                let body = format!(
                    "let {table}={{{fields}}}\n{{for {key},{value} in {table} {{\"\\({key})\":{{rendered:{template}, original:[{key},{value}]}}}}}}"
                );
                probe.check(&body, want.clone());
            }
        }
    }
    probe.check(
        r#"let T={a:"x"}
{for k,v in T {"\(k)":"\uD83D\uDE00\"\/\\\b\f\n\r\t\u0000\(v)\u03bb"}}"#,
        json!({"a":"😀\"/\\\u{8}\u{c}\n\r\t\0xλ"}),
    );
}

#[test]
fn raw_escape_parity_is_not_decoded_text_rescanning() {
    let probe = Probe::new();
    probe.check(
        r#"// "\(outside)" * #D
{literal:"\u005c(k)", escaped:"\\(k)"}"#,
        json!({"literal":"\\(k)","escaped":"\\(k)"}),
    );
    probe.check(
        r#"let T={a:"x"}
{for k,v in T {"\(k)": ["\\(k)","\u005c(v)","\u005c\u0028k)","\\\(v)","\\\\\(k)","\(v)\\(k)"]}}"#,
        json!({"a":["\\(k)","\\(v)","\\(k)","\\x","\\\\a","x\\(k)"]}),
    );
    probe.check(
        r#"let T={}
{for k,v in T {"\(k)":"\"\\\uD83D\uDE00\(v)\n"}}"#,
        json!({}),
    );
    probe.check(
        r#"let T={a:"x"}
{for k,_ in T {"\(k)":"pre\(k)post"}}"#,
        json!({"a":"preapost"}),
    );
    probe.check(r#"let T={a:"outer"}
let C={for k,v in T {"\(k)":"[\(v)]"}}
{result:{let T={b:"inner"}, captured:C, local:{for k,v in T {"\(k)":"[\(v)]"}}}, again:C}"#,
        json!({"result":{"captured":{"a":"[outer]"},"local":{"b":"[inner]"}},"again":{"a":"[outer]"}}));
    probe.check(
        r#"let T={a:"outer"}
{result:{let T={for k,v in T {"\(k)":"[\(v)]"}}, value:T}}"#,
        json!({"result":{"value":{"a":"[outer]"}}}),
    );
}

#[test]
fn eager_syntax_context_scope_and_original_source_guards() {
    let probe = Probe::new();
    for template in [
        r#""\(outer)""#,
        r#""\(_)""#,
        r#""\( v )""#,
        r#""\(v.x)""#,
        r#""\(v+v)""#,
        r#""\(len(v))""#,
        r#""\(v)\(k)""#,
        r#""\()""#,
        r#""\(1)""#,
        r#""\(v""#,
        r#""""\(v)""""#,
        r#"'\(v)'"#,
        r#""pre\(v)" & "x""#,
    ] {
        for fields in ["", "a:\"x\""] {
            probe.reject(&format!(
                "let T={{{fields}}}\n{{for k,v in T {{\"\\(k)\":{template}}}}}"
            ));
        }
    }
    for body in [
        r#"{x:"\(k)"}"#,
        r#"{"\(k)":1}"#,
        r#"let T={a:"\(v)"}
{for k,v in T {"\(k)":v}}"#,
        r#"let T={}
{for k,v in T {"pre\(k)":"\(v)"}}"#,
        r#"let T={}
{for k,v in T {"\(v)":"\(k)"}}"#,
        r#"let T={}
{for k,v in T {"\(k)":{"\(k)":"\(v)"}}}"#,
        r#"let T={}
{for k,_ in T {"\(k)":"\(v)"}}"#,
        r#"let T={}
let outer="x"
{used:outer,result:{for k,v in T {"\(k)":"\(outer)"}}}"#,
        r#"let T={}
let C={for k,v in T {"\(k)":"\(outer)"}}
{x:1}"#,
        r#"let T={}
{x:1}|{x:{for k,v in T {"\(k)":"\(outer)"}}}"#,
        r#"let T={}
#D:1
{for k,v in T {"\(k)":"\(v)"}}"#,
        r#"let T={}
{x:*1|2,result:{for k,v in T {"\(k)":"\(v)"}}}"#,
        r#"let T={a:1}
{for k,v in T {"\(k)":"\(v)"}}"#,
        r#"let T={a:"x",a:"x"}
{for k,v in T {"\(k)":"\(v)"}}"#,
        r#"let A={a:"x"}
let T={for a,b in A {"\(a)":"\(b)"}}
{for k,v in T {"\(k)":"\(v)"}}"#,
        r#"let T={}
let k=1
{used:k,result:{for k,v in T {"\(k)":"\(v)"}}}"#,
        r#"let T={}
{for k,v in T {"\(k)":{deep:{v:"\(k)"}}}}"#,
    ] {
        probe.reject(body);
    }
    for body in [r#"{x: !="pre\(v)"}"#, r#"{x: !="\(v)post"}"#] {
        probe.reject(body);
    }
    // Exact empty-affix interpolation in an inequality historically took this
    // Syntax path; newly tokenized affixed spellings above remain Unsupported.
    let program = source(r#"{x: !="\(v)"}"#);
    for reverse in [false, true] {
        let (a, b) = if reverse {
            (program.as_str(), PACKAGE)
        } else {
            (PACKAGE, program.as_str())
        };
        assert_eq!(
            export(a, b),
            Err(Error::Syntax("literal inequality operand"))
        );
        let out = probe.run(&program, false, reverse);
        assert!(!out.status.success() && out.stdout.is_empty());
        assert_eq!(out.stderr, b"syntax: literal inequality operand\n");
    }
    // Malformed decoded pieces and the inherited hash-string rejection are
    // Syntax, not an either-category weakening of Unsupported checks above.
    for template in [
        r##"#"\#(v)"#"##,
        r#""\uD800\(v)""#,
        r#""\(v)\uDC00""#,
        r#""\uZZZZ\(v)""#,
        "\"\0\\(v)\"",
    ] {
        let program = comp(0, template);
        assert!(matches!(export(PACKAGE, &program), Err(Error::Syntax(_))));
        let out = probe.run(&program, false, false);
        assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
    }
    for value in ["first", "second", "first"] {
        probe.check(
            &format!(
                r#"let T={{a:{}}}
{{for k,v in T {{"\(k)":"<\(v)>"}}}}"#,
                json!(value)
            ),
            json!({"a":format!("<{value}>")}),
        );
    }
}

#[test]
fn exact_shared_work_including_empty_segments_and_multiple_interpolations() {
    for rows in [0, 1, 2, 64] {
        for (template, per_row) in [("v", 11), (r#""\(v)""#, 13), (r#""pre\(k)post""#, 13)] {
            let program = comp(rows, template);
            let total = 7 + per_row * rows;
            assert_eq!(
                export_with_limit(PACKAGE, &program, total - 1),
                Err(Error::Limit("evaluation steps"))
            );
            assert!(export_with_limit(PACKAGE, &program, total).is_ok());
            assert!(export_with_limit(PACKAGE, &program, total + 1).is_ok());
        }
    }
    // Two scalar templates inside a list: P=3+n*(3+3+4), E=4+19n.
    for rows in [0, 1, 2, 64] {
        let program = comp(rows, r#"["\(k)","\(v)"]"#);
        let total = 7 + 29 * rows;
        assert_eq!(
            export_with_limit(PACKAGE, &program, total - 1),
            Err(Error::Limit("evaluation steps"))
        );
        assert!(export_with_limit(PACKAGE, &program, total).is_ok());
    }
    let program = source(
        r#"let T={a:"x"}
{a:{for k,v in T {"\(k)":"\(v)"}},b:{for k,v in T {"\(k)":"\(v)"}}}"#,
    );
    // P=18; E=4+2*(field1+shallow4+full11+bound1)=38.
    assert_eq!(
        export_with_limit(PACKAGE, &program, 55),
        Err(Error::Limit("evaluation steps"))
    );
    assert!(export_with_limit(PACKAGE, &program, 56).is_ok());
    assert_eq!(
        export_with_limit(PACKAGE, &comp(0, r#""\(v)""#), 4097),
        Err(Error::Limit("evaluation steps"))
    );
}

#[test]
fn cumulative_syntax_depth_width_source_token_copy_and_encoded_limits() {
    let template = |n: usize| {
        format!(
            "[{}]",
            std::iter::once(r#""\(v)""#)
                .chain(std::iter::repeat_n("true", n - 1))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    // n+3+(n+1)*T syntax nodes: interpolation remains ONE template/scalar node.
    assert_eq!(
        export(PACKAGE, &comp(32, &template(60))),
        Err(Error::Limit("evaluation steps"))
    ); // 2048 nodes
    assert_eq!(
        export(PACKAGE, &comp(32, &template(61))),
        Err(Error::Limit("syntax nodes"))
    );
    assert!(export(PACKAGE, &comp(1, &template(64))).is_ok());
    assert_eq!(
        export(PACKAGE, &comp(1, &template(65))),
        Err(Error::Limit("container items"))
    );
    assert_eq!(
        export(PACKAGE, &comp(65, r#""\(v)""#)),
        Err(Error::Limit("container items"))
    );
    let nested = |n| format!("{}\"\\(v)\"{}", "[".repeat(n), "]".repeat(n));
    assert!(export(PACKAGE, &comp(1, &nested(30))).is_ok());
    assert_eq!(
        export(PACKAGE, &comp(1, &nested(31))),
        Err(Error::Limit("syntax depth"))
    );
    let deep = source(&format!(
        "let T={{a:\"x\"}}\nlet C={{for k,v in T {{\"\\(k)\":{}}}}}\n{{out:C}}",
        nested(30)
    ));
    assert_eq!(export(PACKAGE, &deep), Err(Error::Limit("output depth")));
    for (length, ok) in [(16383, true), (16384, false)] {
        let prefix = "z".repeat(length - 2);
        let program = source(&format!(
            "let T={{a:\"x\",b:\"y\"}}\n{{for k,v in T {{\"\\(k)\":\"{prefix}\\(v)!\"}}}}"
        ));
        // Rendered L=prefix+row+suffix; P+E copy=4+4L, not just row bytes.
        if ok {
            assert!(export(PACKAGE, &program).is_ok());
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("copied strings"))
            );
        }
    }
    for (count, ok) in [(5459, true), (5460, false)] {
        let prefix = json!("\u{1}".repeat(count)).to_string();
        let program = source(&format!(
            "let T={{a:\"\",b:\"\"}}\n{{for k,v in T {{\"\\(k)\":{}\\(v)\"}}}}",
            &prefix[..prefix.len() - 1]
        ));
        if ok {
            assert_eq!(
                export(PACKAGE, &program).expect("encoded boundary").len(),
                25 + 12 * count
            );
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("encoded bytes"))
            );
        }
    }
    let program = comp(1, r#""\(v)""#);
    let padded = format!(
        "{program}//{}",
        "x".repeat(65536 - PACKAGE.len() - program.len() - 2)
    );
    assert!(export(PACKAGE, &padded).is_ok());
    assert_eq!(
        export(PACKAGE, &format!("{padded}x")),
        Err(Error::Limit("source bytes"))
    );
    // Two package headers + one one-row comprehension have 29 charged tokens.
    let tokens = format!("{program}{}", "\n".repeat(4096 - 29));
    assert!(export(PACKAGE, &tokens).is_ok());
    assert_eq!(
        export(PACKAGE, &format!("{tokens}\n")),
        Err(Error::Limit("tokens"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(&export(PACKAGE, &program).expect("recovery")).expect("JSON"),
        json!({"r0":"v0"})
    );
}
