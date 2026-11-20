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
fn literal(body: &str) -> String {
    format!("\"\"\"\n{body}\n\"\"\"")
}
fn comp(rows: usize, template: &str) -> String {
    let fields = (0..rows)
        .map(|i| format!("r{i}:\"v\""))
        .collect::<Vec<_>>()
        .join(",");
    source(&format!(
        "let T={{{fields}}}\n{{for k,v in T {{\"\\(k)\":{template}}}}}"
    ))
}
fn executable(path: &Path) {
    assert!(path.is_absolute());
    let metadata = fs::metadata(path).expect("tool metadata");
    assert!(metadata.is_file(), "regular tool");
    fs::File::open(path).expect("readable tool");
    assert_ne!(metadata.permissions().mode() & 0o111, 0);
}
struct Probe {
    dir: PathBuf,
    cue: PathBuf,
}
impl Probe {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "multiline-{}-{}",
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
        let paths = [self.dir.join("package.cue"), self.dir.join("renamed.cue")];
        fs::write(&paths[0], PACKAGE).expect("package");
        fs::write(&paths[1], program).expect("program");
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
        for i in if reverse { [1, 0] } else { [0, 1] } {
            command.arg(&paths[i]);
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
                let out = self.run(&program, oracle, reverse);
                assert!(
                    out.status.success(),
                    "{body}: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(
                    serde_json::from_slice::<Value>(&out.stdout).expect("complete JSON"),
                    want
                );
            }
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).expect("retire scratch");
    }
}

#[test]
fn independent_literal_values_margins_empty_lines_unicode_and_quotes() {
    let probe = Probe::new();
    // Source spellings and decoded expectations are deliberately separate.
    for (spelling, value) in [
        ("\"\"\"\n\"\"\"", ""),
        ("\"\"\"\n  \"\"\"", ""),
        ("\"\"\"\n\n  \"\"\"", ""),
        ("\"\"\"\n\n\n  \"\"\"", "\n"),
        ("\"\"\"\n  x\n  \"\"\"", "x"),
        ("\"\"\"\n  x\n\n  \"\"\"", "x\n"),
        ("\"\"\"\n\n  x\n  \"\"\"", "\nx"),
        ("\"\"\"\n  x\n  \n   \n  y\n  \"\"\"", "x\n\n \ny"),
        ("\"\"\"\n \t x \t\n \t\"\"\"", " x \t"),
        ("\"\"\"\n\t雪 λ 😀 ' \" \"\"\n\t\"\"\"", "雪 λ 😀 ' \" \"\""),
        ("\"\"\"\n  \n\"\"\"", "  "),
        ("\"\"\"\n\t\t x\t \n\t\"\"\"", "\t x\t "),
        (
            "\"\"\"\n  a\u{2028}b\u{2029}c\n  \"\"\"",
            "a\u{2028}b\u{2029}c",
        ),
    ] {
        probe.check(&format!("{{value:{spelling}}}"), json!({"value":value}));
    }
    probe.check(
        &format!("#D:{}\n{{value:#D}}", literal("definition")),
        json!({"value":"definition"}),
    );
    probe.check(
        &format!("let A={}\n{{value:A}}", literal("binding")),
        json!({"value":"binding"}),
    );
    for rows in ["", "a:\"x\""] {
        probe.check(
            &format!(
                "let T={{{rows}}}\n{{for k,v in T {{\"\\(k)\":[{},v]}}}}",
                literal("a\n\nb")
            ),
            if rows.is_empty() {
                json!({})
            } else {
                json!({"a":["a\n\nb","x"]})
            },
        );
    }
    // Existing decoded backslashes stay DATA; the multiline restriction is lexical.
    probe.check(
        r#"{x:"\\(outside)",y:"\u005c(n)",z:"\n\t\uD83D\uDE00"}"#,
        json!({"x":"\\(outside)","y":"\\(n)","z":"\n\t😀"}),
    );
}

#[test]
fn excluded_syntax_is_eager_even_unused_or_in_empty_templates() {
    let probe = Probe::new();
    let mut invalid = vec![
        "\"\"\"inline\n\"\"\"".into(),
        "\"\"\"\n\tx\n \"\"\"".into(),
        "\"\"\"\n x\n  \"\"\"".into(),
        "\"\"\"\n  x\n \n  \"\"\"".into(),
        "\"\"\"\n  x\n\t\n  \"\"\"".into(),
        "\"\"\"\n    x\n\t\"\"\"".into(),
        "\"\"\"\n x\"\"\"".into(),
        "\"\"\"\n x\"\"\"y\n\"\"\"".into(),
        "\"\"\"\r\nx\r\n\"\"\"".into(),
        literal("a\rb"),
        literal("a\u{feff}b"),
    ];
    for control in (0u8..32)
        .filter(|b| !matches!(b, b'\n' | b'\t'))
        .chain([127])
    {
        invalid.push(literal(&format!("a{}b", char::from(control))));
    }
    for slash in [
        r"\n",
        r"\t",
        r"\u0041",
        r"\U00000041",
        r"\(v)",
        r"\",
        "\\\n",
        "\\\"\"\"",
    ] {
        invalid.push(literal(slash));
    }
    for spelling in invalid {
        for body in [
            format!("{{x:{spelling}}}"),
            format!("let unused={spelling}\n{{x:1}}"),
            format!("let T={{}}\n{{for k,v in T {{\"\\(k)\":{spelling}}}}}"),
        ] {
            let program = source(&body);
            let error = export(PACKAGE, &program).expect_err("outside admitted literal class");
            assert!(matches!(error, Error::Unsupported(_)), "{body}: {error:?}");
            let out = probe.run(&program, false, false);
            assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
        }
    }
    for body in [
        "{x:\"\"\"\nunterminated}",
        "let unused=\"\"\"\nunterminated\n{x:1}",
        "let T={}\n{for k,v in T {\"\\(k)\":\"\"\"\nunterminated}}",
    ] {
        let program = source(body);
        assert_eq!(
            export(PACKAGE, &program),
            Err(Error::Syntax("unterminated multiline string"))
        );
        let out = probe.run(&program, false, false);
        assert!(!out.status.success() && out.stdout.is_empty());
        assert_eq!(out.stderr, b"syntax: unterminated multiline string\n");
    }
    let text = literal("label");
    for body in [
        format!("{{{text}:1}}"),
        format!("{{x:!={text}}}"),
        format!("let T={{}}\n{{for k,v in T {{{text}:1}}}}"),
        format!("let T={{}}\n{{for k,v in T {{\"\\(k)\":{{{text}:1}}}}}}"),
        format!("let {text}=1\n{{x:1}}"),
    ] {
        assert!(
            matches!(export(PACKAGE, &source(&body)), Err(Error::Unsupported(_))),
            "{body}"
        );
    }
    for spelling in ["'''bytes'''", "#\"\"\"\nx\n\"\"\"#"] {
        assert!(export(PACKAGE, &source(&format!("{{x:{spelling}}}"))).is_err());
    }
    probe.check(
        &format!("{{x:{}}}", literal("recovery")),
        json!({"x":"recovery"}),
    );
}

#[test]
fn scalar_and_template_literals_share_exact_work_without_per_line_charges() {
    for value in ["", "x", "a\n\nb\n", "雪\t \"\""] {
        let multiline = literal(value);
        let ordinary = json!(value).to_string();
        for text in [&multiline, &ordinary] {
            let program = source(text);
            assert_eq!(
                export_with_limit(PACKAGE, &program, 2),
                Err(Error::Limit("evaluation steps"))
            );
            assert!(export_with_limit(PACKAGE, &program, 3).is_ok());
            for rows in [0, 1, 2, 64] {
                let program = comp(rows, text);
                let total = 7 + 11 * rows; // P=3+4n, E=4+7n, exactly the old scalar template.
                assert_eq!(
                    export_with_limit(PACKAGE, &program, total - 1),
                    Err(Error::Limit("evaluation steps"))
                );
                assert!(export_with_limit(PACKAGE, &program, total).is_ok());
            }
        }
    }
    let text = literal("x\ny");
    let program = source(&format!(
        "let T={{a:\"v\"}}\n{{a:{{for k,v in T {{\"\\(k)\":{text}}}}},b:{{for k,v in T {{\"\\(k)\":{text}}}}}}}"
    ));
    // P=14; E=38. Both comprehensions consume the same ledger.
    assert_eq!(
        export_with_limit(PACKAGE, &program, 51),
        Err(Error::Limit("evaluation steps"))
    );
    assert!(export_with_limit(PACKAGE, &program, 52).is_ok());
}

#[test]
fn unchanged_cumulative_syntax_depth_items_token_source_copy_and_output_caps() {
    let text = literal("x\ny");
    let template = |n| format!("[{}]", vec![text.as_str(); n].join(","));
    // n+3+(n+1)*T = 2048 original/template/generated nodes, not line count.
    assert_eq!(
        export(PACKAGE, &comp(32, &template(60))),
        Err(Error::Limit("evaluation steps"))
    );
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
        export(PACKAGE, &comp(65, &text)),
        Err(Error::Limit("container items"))
    );
    let nested = |n| format!("{}{text}{}", "[".repeat(n), "]".repeat(n));
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
    for bytes in [16383, 16384] {
        let program = source(&format!(
            "let T={{a:\"x\",b:\"y\"}}\n{{for k,v in T {{\"\\(k)\":{}}}}}",
            literal(&"z".repeat(bytes))
        ));
        // Four literal copies plus two labels in preparation and evaluation: 4L+4.
        if bytes == 16383 {
            assert!(export(PACKAGE, &program).is_ok());
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("copied strings"))
            );
        }
    }
    let schema = source(&format!("#S:{}", literal(&"a".repeat(1024))));
    let list = |n| source(&format!("[{}]", vec!["#S"; n].join(",")));
    assert!(export(&schema, &list(63)).is_ok());
    assert_eq!(
        export(&schema, &list(64)),
        Err(Error::Limit("encoded bytes"))
    ); // copy exactly 65536
    assert_eq!(
        export(
            &source(&format!("#S:{}", literal(&"a".repeat(1025)))),
            &list(64)
        ),
        Err(Error::Limit("copied strings"))
    );
    // LF becomes two encoded bytes. Pretty array overhead is 3+6n, including final LF.
    for bytes in [508, 509] {
        let schema = source(&format!("#S:{}", literal(&"\n".repeat(bytes))));
        let result = export(&schema, &list(64));
        if bytes == 508 {
            assert_eq!(
                result.expect("encoded boundary").len(),
                3 + 64 * (6 + 2 * bytes)
            );
        } else {
            assert_eq!(result, Err(Error::Limit("encoded bytes")));
        }
    }
    let program = source(&text);
    let padded = format!(
        "{program}//{}",
        "x".repeat(65536 - PACKAGE.len() - program.len() - 2)
    );
    assert!(export(PACKAGE, &padded).is_ok());
    assert_eq!(
        export(PACKAGE, &format!("{padded}x")),
        Err(Error::Limit("source bytes"))
    );
    // Two package headers (six tokens), one scalar token, one trailing LF: eight.
    let tokens = format!("{program}{}", "\n".repeat(4096 - 8));
    assert!(export(PACKAGE, &tokens).is_ok());
    assert_eq!(
        export(PACKAGE, &format!("{tokens}\n")),
        Err(Error::Limit("tokens"))
    );
    assert_eq!(
        export_with_limit(PACKAGE, &program, 4097),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        serde_json::from_str::<Value>(&export(PACKAGE, &program).expect("recovery")).expect("JSON"),
        json!("x\ny")
    );
}
