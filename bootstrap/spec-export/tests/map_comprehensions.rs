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
fn table(rows: usize) -> String {
    (0..rows)
        .map(|i| format!("\"r{i}\":\"v{i}\""))
        .collect::<Vec<_>>()
        .join(",")
}
fn comprehension(rows: usize, template: &str) -> String {
    source(&format!(
        "let T={{{}}}\n{{for k,v in T {{\"\\(k)\":{template}}}}}",
        table(rows)
    ))
}
fn decode(text: &str) -> Value {
    serde_json::from_str(text).expect("complete JSON")
}
fn executable(tool: &Path) {
    assert!(tool.is_absolute(), "absolute tool required");
    fs::File::open(tool).expect("readable tool");
    assert_ne!(
        fs::metadata(tool)
            .expect("tool metadata")
            .permissions()
            .mode()
            & 0o111,
        0,
        "executable tool required"
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
            "map-comprehensions-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("fresh scratch");
        let cue = std::env::split_paths(&std::env::var_os("PATH").expect("pinned PATH"))
            .map(|p| p.join("cue"))
            .find(|p| p.is_file())
            .expect("mandatory pinned CUE")
            .canonicalize()
            .expect("absolute oracle");
        Self { dir, cue }
    }
    fn run(&self, a: &str, b: &str, oracle: bool, reverse: bool) -> Output {
        let paths = [
            self.dir.join("renamed-first.cue"),
            self.dir.join("renamed-second.cue"),
        ];
        fs::write(&paths[0], a).expect("first source");
        fs::write(&paths[1], b).expect("second source");
        let tool = if oracle {
            self.cue.as_path()
        } else {
            Path::new(env!("CARGO_BIN_EXE_cuestacean-spec-export"))
        };
        executable(tool);
        let mut command = Command::new(tool);
        command
            .env_clear()
            .env("PATH", "/unusable")
            .env("HOME", &self.dir)
            .current_dir(&self.dir);
        if oracle {
            command.arg("export");
        }
        for i in if reverse { [1, 0] } else { [0, 1] } {
            command.arg(&paths[i]);
        }
        command.output().expect("mandatory process setup")
    }
    fn check(&self, body: &str, expected: Value) {
        let program = source(body);
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (program.as_str(), PACKAGE)
            } else {
                (PACKAGE, program.as_str())
            };
            assert_eq!(
                decode(&export(a, b).expect("native API")),
                expected,
                "{body}"
            );
            for oracle in [false, true] {
                let out = self.run(PACKAGE, &program, oracle, reverse);
                assert!(
                    out.status.success(),
                    "{body}: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(
                    serde_json::from_slice::<Value>(&out.stdout).expect("JSON"),
                    expected
                );
            }
        }
    }
    fn reject(&self, body: &str, upstream_rejects: bool) {
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
            let out = self.run(PACKAGE, &program, false, reverse);
            assert!(!out.status.success() && out.stdout.is_empty() && !out.stderr.is_empty());
            if upstream_rejects {
                let out = self.run(PACKAGE, &program, true, reverse);
                assert!(
                    !out.status.success() && out.stdout.is_empty(),
                    "CUE must reject {body}"
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
fn independent_rows_templates_three_occurrences_and_renaming() {
    let probe = Probe::new();
    let rows = [
        ("a", "alpha"),
        ("λ", "雪"),
        ("quote\"", "line\nslash\\"),
        ("x y", ""),
    ];
    let fields = rows
        .iter()
        .map(|(k, v)| format!("{}:{}", json!(k), json!(v)))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "let T={{{fields}}}\n{{keys:{{for k,v in T {{\"\\(k)\":k}}}}, values:{{for k,v in T {{\"\\(k)\":v}}}}, nested:{{for k,v in T {{\"\\(k)\":{{\"k\":k, data:[v,true,false,-9223372036854775808,9223372036854775807,{{fixed:\"literal\"}}]}}}}}}}}"
    );
    let mut keys = json!({});
    let mut values = json!({});
    let mut nested = json!({});
    for (key, value) in rows {
        keys[key] = json!(key);
        values[key] = json!(value);
        nested[key] =
            json!({"k":key,"data":[value,true,false,i64::MIN,i64::MAX,{"fixed":"literal"}]});
    }
    let expected = json!({"keys":keys,"values":values,"nested":nested});
    probe.check(&body, expected.clone());
    probe.check(
        &body
            .replace("T", "RenamedTable")
            .replace(" k,", " key,")
            .replace("(k)", "(key)")
            .replace(":k", ":key"),
        expected,
    );
    probe.check(
        "let Empty={}\n{for k,v in Empty {\"\\(k)\":{data:[k,v,true]}}}",
        json!({}),
    );
    probe.check("let T={a:\"x\"}\n{unused:{for k,v in T {\"\\(k)\":true}}, blank:{for k,_ in T {\"\\(k)\":false}}}", json!({"unused":{"a":true},"blank":{"a":false}}));
    probe.check(
        "// * #D for k,v in T {\"\\(k)\":v}\n{escaped:\"\\\\(k)\"}",
        json!({"escaped":"\\(k)"}),
    );
    probe.check(
        "let T={a:\"x\"}\n{for string,int in T {\"\\(string)\":int}}",
        json!({"a":"x"}),
    );
    probe.check(
        "let T={a:\"x\"}\n{for k,v in T {\"\\(k)\":\"\\\\(literal)\"}}",
        json!({"a":"\\(literal)"}),
    );
    for bytes in [1, 64] {
        let key = "é".repeat(bytes / 2) + if bytes == 1 { "x" } else { "" };
        probe.check(
            &format!(
                "let T={{{}:\"v\"}}\n{{for k,v in T {{\"\\(k)\":v}}}}",
                json!(key)
            ),
            json!({key:"v"}),
        );
    }
}

#[test]
fn lexical_capture_initializer_exclusion_reuse_and_fresh_invocations() {
    let probe = Probe::new();
    probe.check("let T={a:\"outer\"}\nlet C={for k,v in T {\"\\(k)\":v}}\n{result:{let T={b:\"inner\"}, captured:C, local:{for k,v in T {\"\\(k)\":v}}},again:C}", json!({"result":{"captured":{"a":"outer"},"local":{"b":"inner"}},"again":{"a":"outer"}}));
    probe.check(
        "let T={a:\"outer\"}\n{result:{let T={for k,v in T {\"\\(k)\":v}}, value:T}}",
        json!({"result":{"value":{"a":"outer"}}}),
    );
    for value in ["first", "second", "first"] {
        probe.check(&format!("let T={{a:{}}}\n{{left:{{for k,v in T {{\"\\(k)\":v}}}},right:{{for k,v in T {{\"\\(k)\":k}}}}}}", json!(value)), json!({"left":{"a":value},"right":{"a":"a"}}));
    }
    for declarations in [
        "let T={a:\"x\"}\nlet C={for k,v in T {\"\\(k)\":v}}",
        "let C={for k,v in T {\"\\(k)\":v}}\nlet T={a:\"x\"}",
    ] {
        probe.check(
            &format!("{declarations}\n{{result:C}}"),
            json!({"result":{"a":"x"}}),
        );
    }
}

#[test]
fn eager_profile_source_and_collision_exclusions() {
    let probe = Probe::new();
    for template in [
        "outer",
        "string",
        "_",
        "#D",
        "v&\"x\"",
        "true|false",
        "{let X=1, a:X}",
        "{#D:1}",
        "{x:1,x:2}",
        "[...string]",
        "{for a,b in T {\"\\(a)\":b}}",
        "{deep:{k:v}}",
        "{deep:{v:k}}",
    ] {
        probe.reject(
            &format!("let T={{}}\n{{for k,v in T {{\"\\(k)\":{template}}}}}"),
            false,
        );
    }
    for body in [
        "let T={a:1}\n{for k,v in T {\"\\(k)\":v}}",
        "let T={a:\"x\",a:\"x\"}\n{for k,v in T {\"\\(k)\":v}}",
        "let T={a:string}\n{for k,v in T {\"\\(k)\":v}}",
        "let T={a:\"x\"}&{b:\"y\"}\n{for k,v in T {\"\\(k)\":v}}",
        "let A={a:\"x\"}\nlet T=A\n{for k,v in T {\"\\(k)\":v}}",
        // Validate ORIGINAL sources before any specialization; T is computed.
        "let A={a:\"x\"}\nlet T={for a,b in A {\"\\(a)\":b}}\n{for k,v in T {\"\\(k)\":v}}",
        "let T={}\n{for k,v in T {\"prefix\\(k)\":v}}",
        "let T={}\n{for k,v in T {\"\\(v)\":v}}",
        "let T={}\n{for k,v in T {\"\\( k )\":v}}",
        "let T={}\n{for k,v in T {\"\\(k)\":v}, sibling:1}",
        "let T={}\n{for k,v in T if true {\"\\(k)\":v}}",
        "let T={}\n{for k,v in T {\"\\(k)\":v, extra:1}}",
        "let T={}\n{for k,_ in T {\"\\(k)\":_}}",
        "let T={}\n{for _,v in T {\"\\(_)\":v}}",
        "let T={}\n{for k,k in T {\"\\(k)\":k}}",
        "{value:\"\\(k)\"}",
        "let T={}\n#D:1\n{for k,v in T {\"\\(k)\":v}}",
        "let T={}\n{choice:*1|2, result:{for k,v in T {\"\\(k)\":v}}}",
        "let T={}\nlet C={for k,v in T {\"\\(k)\":v}}\n{choice:*1|2}",
        "let T={}\n{result:{x:1}|{x:{for k,v in T {\"\\(k)\":outer}}}}",
    ] {
        probe.reject(body, false);
    }
    for body in [
        "let T={a:\"x\"}\n{for T,v in T {\"\\(T)\":v}}",
        "let T={a:\"x\"}\n{for k,T in T {\"\\(k)\":T}}",
        "let T={}\nlet k=1\n{use:k,result:{for k,v in T {\"\\(k)\":v}}}",
        "let T={}\n{result:{for k,v in T {\"\\(k)\":v}},use:k}\nlet k=1",
    ] {
        probe.reject(body, true);
    }
    // Deliberately narrower than Go: do NOT assert upstream rejection here.
    probe.reject(
        "let T={a:\"x\"}\n{result:{for k,v in T {\"\\(k)\":v}},k:1}",
        false,
    );
    let unknown = source("{for k,v in Missing {\"\\(k)\":v}}");
    assert_eq!(
        export(PACKAGE, &unknown),
        Err(Error::Unknown("Missing".into()))
    );
    probe.reject("let A=B\nlet B=A\n{for k,v in A {\"\\(k)\":v}}", true);
    probe.check(
        "let T={a:\"x\"}\n{for k,v in T {\"\\(k)\":v}}",
        json!({"a":"x"}),
    );
}

#[test]
fn analytic_shared_work_and_eager_empty_preparation() {
    for rows in [0, 1, 2, 64] {
        let program = comprehension(rows, "v");
        // P=3+4n; E=4+7n (field scan + shallow and full scalar demand).
        let total = 7 + 11 * rows;
        assert_eq!(
            export_with_limit(PACKAGE, &program, total - 1),
            Err(Error::Limit("evaluation steps"))
        );
        assert!(export_with_limit(PACKAGE, &program, total).is_ok());
        assert!(export_with_limit(PACKAGE, &program, total + 1).is_ok());
    }
    let three = source(
        "let T={}\n{a:{for k,v in T {\"\\(k)\":v}},b:{for k,v in T {\"\\(k)\":v}},c:{for k,v in T {\"\\(k)\":v}}}",
    );
    // E: root 4 + three*(field 1 + shallow 3 + full 4 + bound witness 1) = 31; P=9.
    assert_eq!(
        export_with_limit(PACKAGE, &three, 39),
        Err(Error::Limit("evaluation steps"))
    );
    assert!(export_with_limit(PACKAGE, &three, 40).is_ok());
    assert_eq!(
        export_with_limit(PACKAGE, &comprehension(0, "v"), 4097),
        Err(Error::Limit("evaluation steps"))
    );
}

#[test]
fn cumulative_syntax_depth_width_copy_and_encoded_boundaries_recover() {
    // n+3+(n+1)*T original+template+instantiated nodes; outer struct replaces comp.
    let template = |items| format!("[{}]", vec!["true"; items].join(","));
    assert_eq!(
        export(PACKAGE, &comprehension(32, &template(60))),
        Err(Error::Limit("evaluation steps"))
    ); // exactly 2048 syntax nodes
    assert_eq!(
        export(PACKAGE, &comprehension(32, &template(61))),
        Err(Error::Limit("syntax nodes"))
    );
    assert!(export(PACKAGE, &comprehension(1, &template(64))).is_ok());
    assert_eq!(
        export(PACKAGE, &comprehension(1, &template(65))),
        Err(Error::Limit("container items"))
    );
    assert_eq!(
        export(PACKAGE, &comprehension(65, "v")),
        Err(Error::Limit("container items"))
    );
    let nested = |levels| format!("{}true{}", "[".repeat(levels), "]".repeat(levels));
    assert!(export(PACKAGE, &comprehension(1, &nested(30))).is_ok());
    assert_eq!(
        export(PACKAGE, &comprehension(1, &nested(31))),
        Err(Error::Limit("syntax depth"))
    );
    let deep = source(&format!(
        "let T={{a:\"x\"}}\nlet C={{for k,v in T {{\"\\(k)\":{}}}}}\n{{out:C}}",
        nested(30)
    ));
    assert_eq!(export(PACKAGE, &deep), Err(Error::Limit("output depth")));
    // Two rows, one-byte labels, literal copied in preparation AND evaluation.
    for (bytes, succeeds) in [(16383, true), (16384, false)] {
        let program = source(&format!(
            "let T={{a:\"x\",b:\"y\"}}\n{{for k,v in T {{\"\\(k)\":{}}}}}",
            json!("z".repeat(bytes))
        ));
        if succeeds {
            assert!(export(PACKAGE, &program).is_ok());
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("copied strings"))
            );
        }
    }
    for (bytes, succeeds) in [(32767, true), (32768, false)] {
        let program = source(&format!(
            "let T={{a:{}}}\n{{for k,v in T {{\"\\(k)\":v}}}}",
            json!("z".repeat(bytes))
        ));
        if succeeds {
            assert!(export(PACKAGE, &program).is_ok());
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("copied strings"))
            );
        }
    }
    // Encoded bytes = 25 + 12*n, while copied strings = 4*(1+n).
    for (count, succeeds) in [(5459, true), (5460, false)] {
        let program = source(&format!(
            "let T={{a:\"x\",b:\"y\"}}\n{{for k,v in T {{\"\\(k)\":{}}}}}",
            json!("\u{1}".repeat(count))
        ));
        if succeeds {
            assert_eq!(
                export(PACKAGE, &program).expect("bounded output").len(),
                25 + 12 * count
            );
        } else {
            assert_eq!(
                export(PACKAGE, &program),
                Err(Error::Limit("encoded bytes"))
            );
        }
    }
    for key in [String::new(), "é".repeat(33)] {
        let program = source(&format!(
            "let T={{{}:\"x\"}}\n{{for k,v in T {{\"\\(k)\":v}}}}",
            json!(key)
        ));
        assert_eq!(export(PACKAGE, &program), Err(Error::Limit("label bytes")));
    }
    assert_eq!(
        decode(&export(PACKAGE, &comprehension(1, "v")).expect("recovery")),
        json!({"r0":"v0"})
    );
}
