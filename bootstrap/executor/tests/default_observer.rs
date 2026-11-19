//! Research-only public Go CUE observations, not a native solver or readiness claim.
//! Pinned source: 6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1).
//! Handwritten expectations never consume captured output or generated artifacts.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PROVIDER: &str = "CUESTACEAN_DEFAULT_OBSERVER";
const NO_PATH: &str = "/nonexistent-cuestacean-default-observer-path";

fn provider() -> PathBuf {
    let path = PathBuf::from(std::env::var_os(PROVIDER).expect("mandatory observer provider"));
    assert!(path.is_absolute(), "observer provider must be absolute");
    let metadata = fs::metadata(&path).expect("read observer provider metadata");
    assert!(metadata.is_file(), "observer provider must be a file");
    assert_ne!(
        metadata.permissions().mode() & 0o111,
        0,
        "executable provider"
    );
    fs::File::open(&path).expect("readable observer provider");
    path
}

struct Files(PathBuf);
impl Files {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cuestacean-default-observer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive scratch directory");
        let files = Self(path);
        files.write("probe.cue", source.as_bytes());
        files
    }
    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).expect("write owned scratch file");
        path
    }
    fn invoke(&self, args: &[&str]) -> Output {
        Command::new(provider())
            .args(args)
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", NO_PATH)
            .output()
            .expect("explicit observer execution; never PATH/CUE/native fallback")
    }
    fn run(&self, mode: &str, selectors: &[&str]) -> Output {
        let mut args = vec![mode, "probe.cue"];
        args.extend_from_slice(selectors);
        self.invoke(&args)
    }
    fn expect(&self, mode: &str, selectors: &[&str], expected: &str) {
        for _ in 0..2 {
            let output = self.run(mode, selectors);
            assert!(
                output.status.success(),
                "{mode} {selectors:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "successful observation stderr");
            assert_eq!(output.stdout, expected.as_bytes(), "{mode} {selectors:?}");
        }
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned scratch directory");
    }
}

// T/F is the actual public Default bool, NOT Preference::{Absent,Bottom,Values}.
// Suffixes are literal JSON or the two nonconcrete statuses. MarshalJSON itself
// invokes Default: these are not ordinary-constraint projections.
const FI: &str = "F:incomplete";
const TI: &str = "T:incomplete";
const FX: &str = "F:conflict";
fn wire(mode: &str, flags: Option<[bool; 2]>, expected: &str) -> String {
    let (selected, result) = expected.split_once(':').expect("handwritten observation");
    let selected = match selected {
        "T" => "true",
        "F" => "false",
        _ => panic!("invalid handwritten flag"),
    };
    let (status, value) = match result {
        "incomplete" | "conflict" => (result, "null"),
        _ => ("concrete", result),
    };
    let (left, right) = flags.map_or(("null".into(), "null".into()), |[a, b]| {
        (a.to_string(), b.to_string())
    });
    format!(
        "{{\"formatVersion\":1,\"mode\":\"{mode}\",\"leftDefault\":{left},\"rightDefault\":{right},\"defaultSelected\":{selected},\"status\":\"{status}\",\"value\":{value}}}\n"
    )
}

struct Case {
    name: &'static str,
    left: &'static str,
    right: &'static str,
    // L.Default, R.Default, Unify F/R, default-unify F/R, inline source F/R.
    observations: [&'static str; 8],
    operand_flags: [bool; 2],
    // Independently pinned CLI exports, NOT inferred from API/default flags.
    exports: [&'static str; 2],
}
macro_rules! case {
    ($name:literal, $left:literal, $right:literal, $obs:expr, $flags:expr, $cli:expr) => {
        Case {
            name: $name,
            left: $left,
            right: $right,
            observations: $obs,
            operand_flags: $flags,
            exports: $cli,
        }
    };
}

#[test]
fn handwritten_public_api_observations_and_separate_inline_exports() {
    let cases = [
        case!(
            "p12",
            "*1|2",
            "1|2",
            ["T:1", FI, "T:1", "T:1", "F:1", "F:1", "T:1", "T:1"],
            [true, false],
            ["1", "1"]
        ),
        case!(
            "unmarked",
            "1|2",
            "1|2",
            [FI, FI, FI, FI, FI, FI, FI, FI],
            [false, false],
            ["incomplete", "incomplete"]
        ),
        case!(
            "flat-bottom",
            "*1|2",
            "*2|1",
            ["T:1", "T:2", FI, FI, FX, FX, FI, FI],
            [true, true],
            ["incomplete", "incomplete"]
        ),
        case!(
            "multiple-defaults",
            "*1|*2|3",
            "1|2|3",
            [TI, FI, TI, TI, TI, TI, TI, TI],
            [true, false],
            ["incomplete", "incomplete"]
        ),
        case!(
            "abstract-default",
            "*int|1",
            "int",
            [TI, FI, TI, TI, FI, FI, TI, TI],
            [true, false],
            ["incomplete", "incomplete"]
        ),
        case!(
            "conflict",
            "1",
            "2",
            ["F:1", "F:2", FX, FX, FX, FX, FX, FX],
            [false, false],
            ["conflict", "conflict"]
        ),
        case!(
            "i64-min",
            "-9223372036854775808",
            "int",
            [
                "F:-9223372036854775808",
                FI,
                "F:-9223372036854775808",
                "F:-9223372036854775808",
                "F:-9223372036854775808",
                "F:-9223372036854775808",
                "F:-9223372036854775808",
                "F:-9223372036854775808"
            ],
            [false, false],
            ["-9223372036854775808", "-9223372036854775808"]
        ),
        case!(
            "i64-max",
            "9223372036854775807",
            "int",
            [
                "F:9223372036854775807",
                FI,
                "F:9223372036854775807",
                "F:9223372036854775807",
                "F:9223372036854775807",
                "F:9223372036854775807",
                "F:9223372036854775807",
                "F:9223372036854775807"
            ],
            [false, false],
            ["9223372036854775807", "9223372036854775807"]
        ),
        case!(
            "boolean-false",
            "false",
            "bool",
            [
                "F:false", FI, "F:false", "F:false", "F:false", "F:false", "F:false", "F:false"
            ],
            [false, false],
            ["false", "false"]
        ),
        case!(
            "null",
            "null",
            "_",
            [
                "F:null", FI, "F:null", "F:null", "F:null", "F:null", "F:null", "F:null"
            ],
            [false, false],
            ["null", "null"]
        ),
        case!(
            "support-elimination-one",
            "*1|2",
            "1|2|*3",
            ["T:1", "T:3", "T:1", "T:1", FX, FX, "T:1", "T:1"],
            [true, true],
            ["1", "1"]
        ),
        case!(
            "support-elimination-two",
            "*2|3",
            "(*1|2)|3",
            ["T:2", "T:1", "T:2", "T:2", "F:2", "F:2", "T:2", "T:2"],
            [true, true],
            ["2", "2"]
        ),
        case!(
            "176-genuine",
            "(*1|2)|*3",
            "*2|1",
            ["T:3", "T:2", FI, TI, FX, FX, FI, TI],
            [true, true],
            ["incomplete", "incomplete"]
        ),
        case!(
            "176-asymmetric",
            "*1|2",
            "(*1|2)|*3",
            ["T:1", "T:3", "T:1", FI, FX, FX, "T:1", FI],
            [true, true],
            ["1", "incomplete"]
        ),
        case!(
            "176-abstract-asymmetric",
            "*int|1",
            "(*1|2)|3",
            [TI, "T:1", TI, "T:1", "T:1", "T:1", TI, "T:1"],
            [true, true],
            ["incomplete", "1"]
        ),
        case!(
            "176-nested-asymmetric",
            "*((*1|2)&(1|*2))|3",
            "*3|1|2",
            [FI, "T:3", FI, "T:2", "F:3", "F:3", FI, "T:2"],
            [false, true],
            ["incomplete", "2"]
        ),
        case!(
            "186-asymmetric-one",
            "(*1|2)|(*2|1)",
            "*1|2",
            [TI, "T:1", "T:1", TI, "F:1", "F:1", "T:1", TI],
            [true, true],
            ["1", "incomplete"]
        ),
        case!(
            "186-asymmetric-two",
            "((*1|2)&(*1|3))|3",
            "*3|1",
            [FI, "T:3", "T:3", TI, "F:3", "F:3", "T:3", TI],
            [false, true],
            ["3", "incomplete"]
        ),
        case!(
            "186-genuine",
            "(*3|1)|3",
            "(*1|2)|(*3|2)",
            ["T:3", TI, TI, TI, TI, TI, TI, TI],
            [true, true],
            ["incomplete", "incomplete"]
        ),
        case!(
            "188-dead-mark-genuine",
            "(*1|2)|*3",
            "1|2",
            ["T:3", FI, FI, FI, FX, FX, FI, FI],
            [true, false],
            ["incomplete", "incomplete"]
        ),
        case!(
            "188-regression-genuine",
            "*((*1|2)|3)|(*4|5)",
            "1|4",
            ["T:1", FI, "T:1", "T:1", "F:1", "F:1", "T:1", "T:1"],
            [true, false],
            ["1", "1"]
        ),
        case!(
            "188-bottom-genuine",
            "((*1|2)&(*2|1))|1",
            "*1|2",
            [FI, "T:1", FI, TI, "F:1", "F:1", FI, TI],
            [false, true],
            ["incomplete", "incomplete"]
        ),
        case!(
            "188-asymmetric",
            "*((*1|2)|3)|(*4|5)",
            "1|2|3|4|5",
            ["T:1", FI, "T:1", TI, "T:1", FI, "T:1", TI],
            [true, false],
            ["1", "incomplete"]
        ),
        case!(
            "188-m2-asymmetric",
            "*(*3|1)|2",
            "1|2",
            ["T:3", FI, FI, "T:1", "F:1", "F:1", FI, "T:1"],
            [true, false],
            ["incomplete", "1"]
        ),
    ];
    for case in cases {
        // These are INDEPENDENT inline expressions, never `left & right` refs.
        let files = Files::new(&format!(
            "left: ({})\nright: ({})\nsourceExpr: ({}) & ({})\nsourceReverse: ({}) & ({})\n",
            case.left, case.right, case.left, case.right, case.right, case.left
        ));
        for (selector, index) in [
            ("left", 0),
            ("right", 1),
            ("sourceExpr", 6),
            ("sourceReverse", 7),
        ] {
            files.expect(
                "inspect",
                &[selector],
                &wire("inspect", None, case.observations[index]),
            );
        }
        for (order, selectors) in [["left", "right"], ["right", "left"]].iter().enumerate() {
            files.expect(
                "unify",
                selectors,
                &wire("unify", None, case.observations[2 + order]),
            );
            let flags = if order == 0 {
                case.operand_flags
            } else {
                [case.operand_flags[1], case.operand_flags[0]]
            };
            files.expect(
                "default-unify",
                selectors,
                &wire("default-unify", Some(flags), case.observations[4 + order]),
            );
        }
        for (selector, expected) in ["sourceExpr", "sourceReverse"].iter().zip(case.exports) {
            cli_export(&files, selector, expected, case.name);
        }
    }
}

// Separate upstream CLI experiment, NEVER a fallback for the observer. The
// existing pinned devshell supplies CUE. API Unify and inline source conjunction
// are different operations even when individually pinned observations coincide.
fn cli_export(files: &Files, selector: &str, expected: &str, name: &str) {
    let run = |operation: &str| {
        Command::new("cue")
            .args([operation, "probe.cue", "-e", selector])
            .current_dir(&files.0)
            .output()
            .expect("mandatory pinned CUE CLI for separate upstream comparison")
    };
    let output = run("export");
    match expected {
        "incomplete" | "conflict" => {
            assert!(!output.status.success(), "{name} {selector}");
            assert!(output.stdout.is_empty(), "failed export cannot emit JSON");
            assert!(!output.stderr.is_empty());
            // No diagnostic-string classification: eval succeeds for incomplete.
            assert_eq!(
                run("eval").status.success(),
                expected == "incomplete",
                "{name} {selector}"
            );
        }
        raw => {
            assert!(
                output.status.success(),
                "{name} {selector}: {:?}",
                output.stderr
            );
            assert_eq!(
                output.stdout.trim_ascii(),
                raw.as_bytes(),
                "{name} {selector}"
            );
        }
    }
}

#[test]
fn exact_protocol_repeat_and_renamed_explicit_paths() {
    let files = Files::new("package probe\nchosen: *1|2\n");
    let expected = "{\"formatVersion\":1,\"mode\":\"inspect\",\"leftDefault\":null,\"rightDefault\":null,\"defaultSelected\":true,\"status\":\"concrete\",\"value\":1}\n";
    files.expect("inspect", &["chosen"], expected);
    files.expect("inspect", &["chosen"], expected);
    let path = files.write(
        "renamed source.cue",
        "package renamed\nbox: {\"renamed field\": *1|2}\n".as_bytes(),
    );
    let path = path.to_str().expect("UTF-8 scratch path");
    let output = files.invoke(&["inspect", path, "box.\"renamed field\""]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, expected.as_bytes());
    files.write("probe.cue", b"chosen: 1|2\nbad: 1&2\n");
    files.expect("inspect", &["chosen"], "{\"formatVersion\":1,\"mode\":\"inspect\",\"leftDefault\":null,\"rightDefault\":null,\"defaultSelected\":false,\"status\":\"incomplete\",\"value\":null}\n");
    files.expect("inspect", &["bad"], &wire("inspect", None, FX));
}

fn setup_failure(output: Output) {
    assert!(!output.status.success(), "malformed setup accepted");
    assert!(
        output.stdout.is_empty(),
        "setup failure emitted partial JSON"
    );
    assert!(!output.stderr.is_empty(), "setup failure lacks diagnostic");
}

#[test]
fn malformed_setup_and_reference_free_admission() {
    let files = Files::new("left: *1|2\nright: 1|2\n");
    for args in [
        vec![],
        vec!["unknown"],
        vec!["inspect", "probe.cue"],
        vec!["inspect", "probe.cue", "left", "right"],
        vec!["unify", "probe.cue", "left"],
        vec!["default-unify", "probe.cue", "left", "right", "left"],
        vec!["inspect", "missing.cue", "left"],
        vec!["inspect", ".", "left"],
        vec!["inspect", "probe.cue", ""],
        vec!["inspect", "probe.cue", "left["],
        vec!["inspect", "probe.cue", "absent"],
        vec!["unify", "probe.cue", "left", "absent"],
        vec!["default-unify", "probe.cue", "absent", "right"],
    ] {
        setup_failure(files.invoke(&args));
    }
    // Includes resolved references: none may masquerade as an inline conjunction.
    for source in [
        "left: (",
        "import \"math\"\nleft: 1\n",
        "left: missingName\n",
        "left: 1\nright: 2\nsourceExpr: left & right\n",
        "box: {left: 1, right: left}\nleft: 1\n",
        "box: {left: 1}\nleft: box.left\n",
        "let alias = 1\nleft: alias\n",
        "left: 1+2\n",
        "left: len(\"x\")\n",
        "left: [1,2]\n",
        "#schema: 1\nleft: 1\n",
        "left?: 1\n",
        "left: true && false\n",
        "left: { [string]: 1 }\n",
    ] {
        files.write("probe.cue", source.as_bytes());
        setup_failure(files.run("inspect", &["left"]));
    }
    for label in ["_", "int", "bool", "string", "bytes", "number", "float"] {
        for quoted in [false, true] {
            let label = if quoted {
                format!("\"{label}\"")
            } else {
                label.into()
            };
            for source in [
                format!("{label}: 1\nleft: 1\n"),
                format!("box: {{{label}: 1}}\nleft: 1\n"),
            ] {
                files.write("probe.cue", source.as_bytes());
                setup_failure(files.run("inspect", &["left"]));
            }
        }
    }
    // Decoded spelling, not the raw quoted token, controls shadow rejection.
    files.write(
        "probe.cue",
        br#""\u0069nt": 1
left: 1
"#,
    );
    setup_failure(files.run("inspect", &["left"]));
}

#[test]
fn raw_source_and_utf8_selector_caps_are_inclusive() {
    let files = Files::new("left: *1|2\n");
    let mut source = b"left: *1|2\n".to_vec();
    source.resize(65536, b' ');
    files.write("probe.cue", &source);
    files.expect("inspect", &["left"], &wire("inspect", None, "T:1"));
    source.push(b' ');
    files.write("probe.cue", &source);
    setup_failure(files.run("inspect", &["left"]));
    // 127 two-byte scalars plus two quote bytes = exactly 256 UTF-8 bytes.
    let label = "é".repeat(127);
    let selector = format!("\"{label}\"");
    assert_eq!(selector.len(), 256);
    files.write("probe.cue", format!("{selector}: *1|2\n").as_bytes());
    files.expect("inspect", &[&selector], &wire("inspect", None, "T:1"));
    let too_long = format!("\"{label}a\"");
    assert_eq!(too_long.len(), 257);
    files.write(
        "probe.cue",
        format!("{too_long}: *1|2\nleft: 1\n").as_bytes(),
    );
    for (mode, selectors) in [
        ("inspect", vec![too_long.as_str()]),
        ("unify", vec![too_long.as_str(), "left"]),
        ("unify", vec!["left", too_long.as_str()]),
        ("default-unify", vec![too_long.as_str(), "left"]),
        ("default-unify", vec!["left", too_long.as_str()]),
    ] {
        setup_failure(files.run(mode, &selectors));
    }
}

// This test is also invoked as an isolated subprocess with broken providers.
// Never mutate the test runner's ambient environment (including parallel tests).
#[test]
fn provider_setup_requires_absolute_executable() {
    let files = Files::new("left: *1|2\n");
    files.expect("inspect", &["left"], &wire("inspect", None, "T:1"));
}

#[test]
fn missing_or_broken_provider_is_fatal_without_fallback() {
    let files = Files::new("left: 1\n");
    let unreadable = files.write("unreadable", b"not an oracle");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o111)).expect("unreadable");
    let not_executable = files.write("not-executable", b"not an oracle");
    let invalid_executable = files.write("invalid-executable", b"not executable format");
    fs::set_permissions(&invalid_executable, fs::Permissions::from_mode(0o755))
        .expect("executable");
    for path in [
        None,
        Some(Path::new("relative-oracle")),
        Some(files.0.as_path()),
        Some(files.0.join("absent-oracle").as_path()),
        Some(unreadable.as_path()),
        Some(not_executable.as_path()),
        Some(invalid_executable.as_path()),
    ] {
        let mut command = Command::new(std::env::current_exe().expect("current test binary"));
        command
            .args([
                "--exact",
                "provider_setup_requires_absolute_executable",
                "--nocapture",
            ])
            .env_clear()
            .env("PATH", NO_PATH);
        if let Some(path) = path {
            command.env(PROVIDER, path);
        }
        let output = command.output().expect("isolated provider setup test");
        assert!(
            !output.status.success(),
            "invalid provider accepted: {path:?}"
        );
    }
}
