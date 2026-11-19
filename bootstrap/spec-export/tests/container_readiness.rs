use cuestacean_spec_export::{Error, export, export_with_limit};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

// Both implementations receive these exact two file scopes. Every probe uses an
// explicit value field, including structs; definitions-only files are not top.
fn sources(definitions: &str, expression: &str) -> (String, String) {
    (
        format!("package p\n{definitions}\n"),
        format!("package p\n{{value:{expression}}}\n"),
    )
}

struct Files(PathBuf);
impl Files {
    fn new(first: &str, second: &str, renamed: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "cuestacean-container-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("unique probe directory");
        let files = Self(directory);
        let names = Self::names(renamed);
        fs::write(files.0.join(names[0]), first).expect("first raw source");
        fs::write(files.0.join(names[1]), second).expect("second raw source");
        files
    }
    fn names(renamed: bool) -> [&'static str; 2] {
        if renamed {
            ["unrelated-z.cue", "unrelated-a.cue"]
        } else {
            ["schema.cue", "program.cue"]
        }
    }
    fn run(&self, oracle: bool, renamed: bool, reversed: bool) -> Output {
        // The pinned test shell supplies cue; no optional oracle skip or fixture
        // output is permitted. Every observation starts a fresh public process.
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
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove raw probe sources");
    }
}
fn no_json(output: &Output) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "failure must not publish JSON");
    assert!(!output.stderr.is_empty(), "failure needs a diagnostic");
}
fn expected_json(output: &Output, expected: &Value) {
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
fn check(definitions: &str, expression: &str, expected: Result<Value, Error>) {
    let (first, second) = sources(definitions, expression);
    let expected = expected.map(|value| json!({"value":value}));
    // Observe upstream before checking native: baseline native false successes
    // must not prevent the independent oracle from exercising a regression.
    for renamed in [false, true] {
        let files = Files::new(&first, &second, renamed);
        for reversed in [false, true] {
            let oracle = files.run(true, renamed, reversed);
            match &expected {
                Ok(value) => expected_json(&oracle, value),
                Err(error) => {
                    no_json(&oracle);
                    let diagnostic = String::from_utf8_lossy(&oracle.stderr);
                    let semantic = match error {
                        Error::Incomplete => diagnostic.contains("incomplete value"),
                        Error::Conflict => {
                            diagnostic.contains("conflicting values")
                                || diagnostic.contains("field not allowed")
                        }
                        _ => panic!("use separate oracle expectations for profile limits"),
                    };
                    assert!(semantic, "{expression}: {diagnostic}");
                }
            }
        }
    }
    for (a, b) in [(&first, &second), (&second, &first), (&first, &second)] {
        let result = export(a, b)
            .map(|text| serde_json::from_str::<Value>(&text).expect("native complete JSON"));
        assert_eq!(result, expected, "{expression}");
    }
    for renamed in [false, true] {
        let files = Files::new(&first, &second, renamed);
        for reversed in [false, true] {
            let native = files.run(false, renamed, reversed);
            match &expected {
                Ok(value) => {
                    expected_json(&native, value);
                    assert_eq!(
                        native.stdout,
                        export(&first, &second).expect("canonical bytes").as_bytes()
                    );
                }
                Err(_) => no_json(&native),
            }
        }
    }
}
fn alternatives(definitions: &str, left: &str, right: &str, expected: Result<Value, Error>) {
    for (a, b) in [(left, right), (right, left)] {
        check(definitions, &format!("({a})|({b})"), expected.clone());
    }
}

#[test]
fn list_bounds_not_projected_json_determine_readiness() {
    for (left, right) in [("[1,...int]", "[1]"), ("[...int]", "[]")] {
        alternatives("", left, right, Err(Error::Incomplete));
    }
    for (left, right, expected) in [
        ("[1,...int]", "[1,...int]", json!([1])),
        ("[1,...int]", "[1,...string]", json!([1])),
        ("[...int]", "[...string]", json!([])),
        ("[1]", "[1]", json!([1])),
        ("[]", "[]", json!([])),
        // Contradictory unseen tails do not turn a known prefix into a fixed list.
        ("([1,...int]&[1,...string])", "[1,...int]", json!([1])),
    ] {
        alternatives("", left, right, Ok(expected));
    }
    for (left, right) in [
        ("([...int]&[...string])", "[]"),
        ("([1,...int]&[1,...string])", "[1]"),
        ("[1,...(int&string)]", "[1]"),
    ] {
        alternatives("", left, right, Err(Error::Incomplete));
    }
    alternatives("", "[1]", "[2]", Err(Error::Incomplete));
    alternatives("", "true", "true", Ok(json!(true)));
    alternatives("", "1", "1", Ok(json!(1)));
    check("", "[1,...int]&[1,\"x\"]", Err(Error::Conflict));
}

#[test]
fn structural_closedness_is_not_definition_identity() {
    let definitions = "#D:{a:1}\n#E:{a:1}\n#Empty:{}";
    alternatives(definitions, "#D", "{a:1}", Err(Error::Incomplete));
    alternatives(definitions, "#Empty", "{}", Err(Error::Incomplete));
    alternatives(definitions, "#D", "#E", Ok(json!({"a":1})));
    alternatives(definitions, "#D", "(#D&{a:1})", Ok(json!({"a":1})));
    alternatives("", "{a:1,b:2}", "{b:2,a:1}", Ok(json!({"a":1,"b":2})));
    check(definitions, "#D&{b:2}", Err(Error::Conflict));
    let nested = "#D:{a:1}\n#F:{x:{a:1}}\n#H:{x:#D}";
    alternatives(nested, "#F", "#H", Ok(json!({"x":{"a":1}})));
}

#[test]
fn witnesses_preserve_positions_not_just_number_of_open_containers() {
    for (left, right) in [
        ("{xs:[1,...int]}", "{xs:[1]}"),
        ("{a:[1,...int],b:[1]}", "{b:[1,...int],a:[1]}"),
        ("[[1,...int],[1]]", "[[1],[1,...int]]"),
        ("{a:#D,b:{a:1}}", "{b:#D,a:{a:1}}"),
        ("[{a:[1,...int]},[1]]", "[{a:[1]},[1,...int]]"),
        ("{a:[#D],b:[{a:1}]}", "{b:[#D],a:[{a:1}]}"),
    ] {
        alternatives("#D:{a:1}", left, right, Err(Error::Incomplete));
    }
    alternatives(
        "",
        "{z:[1,...int],a:{x:[2]}}",
        "{a:{x:[2]},z:[1,...string]}",
        Ok(json!({"a":{"x":[2]},"z":[1]})),
    );
}

#[test]
fn lexical_templates_capture_without_closing_or_cross_instance_mutation() {
    let definitions = "let X={a:1}\n#Open:X\n#D:{a:1}";
    // A definition closes a captured open binding; using that binding itself does not.
    alternatives(definitions, "#Open", "#D", Ok(json!({"a":1})));
    let (first, _) = sources("let X={a:1}\n#D:{a:1}", "1");
    // File-local lexical scope is preserved in identical raw texts for both tools.
    let second = "package p\nlet X=[1,...int]\n{value:X|[1]}\n";
    let files = Files::new("package p\n", second, false);
    let oracle = files.run(true, false, false);
    no_json(&oracle);
    assert!(String::from_utf8_lossy(&oracle.stderr).contains("incomplete value"));
    assert_eq!(export("package p\n", second), Err(Error::Incomplete));
    no_json(&files.run(false, false, false));
    // Capture is declaration-time, not caller-time: file-local X here differs.
    let program = "package p\nlet X={a:2}\n{value:{captured:#D,local:X}}\n";
    let expected = json!({"value":{"captured":{"a":1},"local":{"a":2}}});
    // Use an explicit captured definition so every binding has a static use.
    let first = first.replace("#D:{a:1}", "#D:X");
    let files = Files::new(&first, program, true);
    expected_json(&files.run(true, true, false), &expected);
    assert_eq!(
        serde_json::from_str::<Value>(&export(&first, program).expect("capture"))
            .expect("captured JSON"),
        expected
    );
    check(
        "let X=[1,...int]\n#L:X",
        "{fixed:#L&[1,2],open:#L,again:#L&[1]}",
        Ok(json!({"fixed":[1,2],"open":[1],"again":[1]})),
    );
}

#[test]
fn productive_recursive_definitions_keep_inherited_closures() {
    let definitions = "#R:{children:[...#R]}\n#S:{children:[...#S]}";
    alternatives(definitions, "#R", "#S", Ok(json!({"children":[]})));
    alternatives(
        definitions,
        "#R&{children:[{}]}",
        "#S&{children:[{}]}",
        Ok(json!({"children":[{"children":[]}]})),
    );
    alternatives(
        definitions,
        "#R&{children:[{}]}",
        "{children:[{children:[]}]}",
        Err(Error::Incomplete),
    );
    check(
        definitions,
        "#R&{children:[{extra:1}]}",
        Err(Error::Conflict),
    );
    check(
        definitions,
        "{one:#R&{children:[{}]},two:#R}",
        Ok(json!({"one":{"children":[{"children":[]}]},"two":{"children":[]}})),
    );
}

#[test]
fn contextual_refinement_replaces_stale_open_witnesses() {
    for expression in [
        "([1,...int]|[1])&[1]",
        "[1]&([1]|[1,...int])",
        "([1,...int]|[1])&[1,2]",
        "[1,2]&([1]|[1,...int])",
    ] {
        let expected = if expression.contains("1,2") {
            json!([1, 2])
        } else {
            json!([1])
        };
        check("", expression, Ok(expected));
    }
    alternatives("", "([1,...int]&[1])", "[1]", Ok(json!([1])));
    check("#D:{a:1}", "(#D|{a:1})&#D", Ok(json!({"a":1})));
    check(
        "",
        "({xs:[1,...int]}|{xs:[1]})&{xs:[1]}",
        Ok(json!({"xs":[1]})),
    );
    check("", "([1,...int]|[1])&[2]", Err(Error::Conflict));
    check(
        "#L:[1,...int]",
        "{first:(#L&[1])|[1],second:#L,third:(#L&[1,2])|[1,2]}",
        Ok(json!({"first":[1],"second":[1],"third":[1,2]})),
    );
}

fn minimum_work(first: &str, second: &str) -> usize {
    (1..=4096)
        .find(|&work| export_with_limit(first, second, work).is_ok())
        .expect("must fit unchanged hard work cap")
}

#[test]
fn depth_and_cumulative_witness_work_copy_comparison_and_recovery() {
    let chain = |n: usize| {
        let mut definitions = String::new();
        for i in 0..n {
            definitions.push_str(&format!("#N{i}:[#N{}]\n", i + 1));
        }
        definitions.push_str(&format!("#N{n}:1"));
        definitions
    };
    // The explicit root object adds a level: 30 lists + root + scalar = 32.
    let mut expected = json!(1);
    for _ in 0..30 {
        expected = json!([expected]);
    }
    check(&chain(30), "#N0", Ok(expected));
    let (first, second) = sources(&chain(31), "#N0");
    assert_eq!(export(&first, &second), Err(Error::Limit("output depth")));
    let files = Files::new(&first, &second, false);
    expected_json(&files.run(true, false, false), &{
        let mut value = json!(1);
        for _ in 0..31 {
            value = json!([value]);
        }
        json!({"value":value})
    });
    no_json(&files.run(false, false, false));

    let (first, single) = sources("let X={a:[1,...int]}\n#D:X", "#D");
    let (_, repeated) = sources("", "#D|#D");
    let (_, wider) = sources("", "{a:#D,b:#D}");
    let baseline = minimum_work(&first, &single);
    for additional in [&repeated, &wider] {
        let required = minimum_work(&first, additional);
        assert!(required > baseline, "child/branch work must accumulate");
        assert_eq!(
            export_with_limit(&first, additional, baseline),
            Err(Error::Limit("evaluation steps"))
        );
        assert_eq!(
            export_with_limit(&first, additional, required - 1),
            Err(Error::Limit("evaluation steps"))
        );
        export_with_limit(&first, additional, required).expect("exact minimum recovers");
    }
    assert_eq!(
        export_with_limit(&first, &single, 0),
        Err(Error::Limit("evaluation steps"))
    );
    assert_eq!(
        export_with_limit(&first, &single, 4097),
        Err(Error::Limit("evaluation steps"))
    );
    let definitions = format!("#D:{{a:{}}}", json!("x".repeat(34000)));
    let (first, second) = sources(&definitions, "#D|#D");
    assert_eq!(export(&first, &second), Err(Error::Limit("copied strings")));
    let files = Files::new(&first, &second, false);
    expected_json(
        &files.run(true, false, false),
        &json!({"value":{"a":"x".repeat(34000)}}),
    );
    no_json(&files.run(false, false, false));
    // Neither a depleted allowance nor a failed trial may poison a later call.
    check("", "{a:[1]}", Ok(json!({"a":[1]})));
}

#[test]
fn optional_fields_remain_an_explicit_profile_exclusion() {
    // A bound witness is not a certificate for optional-only residuals. These
    // upstream successes remain Unsupported even inside unused definitions.
    for (definitions, expression) in [("", "{a?:1}"), ("#Unused:{a?:1}", "{a:1}")] {
        let (first, second) = sources(definitions, expression);
        assert_eq!(
            export(&first, &second),
            Err(Error::Unsupported("syntax outside export profile"))
        );
        let expected = if definitions.is_empty() {
            json!({})
        } else {
            json!({"a":1})
        };
        let files = Files::new(&first, &second, false);
        expected_json(&files.run(true, false, false), &json!({"value":expected}));
        no_json(&files.run(false, false, false));
    }
    // The unchanged full-scanner Unsupported boundary remains independently
    // exercised by scanner_boundary.rs; do not narrow its raw input here.
}
