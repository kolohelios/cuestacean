use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

#[path = "cuestacean/json.rs"]
mod json;

const USAGE: &str =
    "usage: cuestacean export [-e path] [--out json] [-o -] <input.cue|input.json|json: -|->";

struct ExportArgs {
    input: OsString,
    field: Option<SelectorPath>,
    json: bool,
}

#[derive(Debug, PartialEq)]
enum SelectorStep {
    Field(String),
    Index(usize),
}

#[derive(Debug)]
struct SelectorPath {
    text: String,
    // Each end offset identifies the full valid prefix after this step.
    steps: Vec<(SelectorStep, usize)>,
}

fn field_argument(field: Option<OsString>) -> Result<SelectorPath, String> {
    let invalid =
        || "--expression requires an ASCII field path with optional decimal indices".to_string();
    let text = field
        .and_then(|field| field.into_string().ok())
        .ok_or_else(invalid)?;
    let bytes = text.as_bytes();
    let mut steps = Vec::new();
    let mut pos = 0;
    loop {
        let start = pos;
        if !matches!(bytes.get(pos), Some(byte) if byte.is_ascii_alphabetic()) {
            return Err(invalid());
        }
        pos += 1;
        while matches!(bytes.get(pos), Some(byte) if byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            pos += 1;
        }
        let name = &text[start..pos];
        if steps.is_empty() && matches!(name, "true" | "false" | "null") {
            return Err(invalid());
        }
        steps.push((SelectorStep::Field(name.into()), pos));
        while bytes.get(pos) == Some(&b'[') {
            pos += 1;
            let start = pos;
            while matches!(bytes.get(pos), Some(byte) if byte.is_ascii_digit()) {
                pos += 1;
            }
            let digits = &text[start..pos];
            if digits.is_empty()
                || (digits.len() > 1 && digits.starts_with('0'))
                || bytes.get(pos) != Some(&b']')
            {
                return Err(invalid());
            }
            let index = digits
                .parse::<usize>()
                .ok()
                .filter(|index| *index <= 268_435_454)
                .ok_or_else(|| format!("unsupported selector index: {digits}"))?;
            pos += 1;
            steps.push((SelectorStep::Index(index), pos));
        }
        if pos == bytes.len() {
            break;
        }
        if bytes.get(pos) != Some(&b'.') {
            return Err(invalid());
        }
        pos += 1;
    }
    Ok(SelectorPath { text, steps })
}

fn export_arguments(mut args: impl Iterator<Item = OsString>) -> Result<ExportArgs, String> {
    if args.next().as_deref() != Some(OsStr::new("export")) {
        return Err(format!("only the export command is supported\n{USAGE}"));
    }
    let mut input = None;
    let mut field = None;
    let mut out_seen = false;
    let mut outfile_seen = false;
    let mut options = true;
    while let Some(arg) = args.next() {
        if options {
            match arg.to_str() {
                Some("--") => {
                    options = false;
                    continue;
                }
                Some("--out" | "--out=json") => {
                    if out_seen {
                        return Err("--out may only be specified once".into());
                    }
                    out_seen = true;
                    if arg == "--out" && args.next().as_deref() != Some(OsStr::new("json")) {
                        return Err("--out requires json (other formats are not supported)".into());
                    }
                    continue;
                }
                Some("-o" | "--outfile" | "--outfile=-") => {
                    if outfile_seen {
                        return Err("output destination may only be specified once".into());
                    }
                    outfile_seen = true;
                    if arg != "--outfile=-" && args.next().as_deref() != Some(OsStr::new("-")) {
                        return Err("only stdout (-) is supported as an output destination".into());
                    }
                    continue;
                }
                Some(option)
                    if matches!(option, "-e" | "--expression")
                        || option.starts_with("--expression=") =>
                {
                    if field.is_some() {
                        return Err("--expression may only be specified once".into());
                    }
                    let value = match option.strip_prefix("--expression=") {
                        Some(name) => Some(OsString::from(name)),
                        None => args.next(),
                    };
                    field = Some(field_argument(value)?);
                    continue;
                }
                _ if arg.as_encoded_bytes().starts_with(b"-") && arg != "-" => {
                    return Err(format!(
                        "unsupported export option: {}",
                        arg.to_string_lossy()
                    ));
                }
                _ => {}
            }
        }
        // A format marker consumes its input immediately, even after `--`.
        let arg = if arg == "json:" {
            if args.next().as_deref() != Some(OsStr::new("-")) {
                return Err("json: must be followed immediately by -".into());
            }
            OsString::from("json:")
        } else {
            arg
        };
        if input.replace(arg).is_some() {
            return Err(format!("export requires exactly one input\n{USAGE}"));
        }
    }
    let input = input.ok_or_else(|| format!("export requires exactly one input\n{USAGE}"))?;
    let json = input == "json:" || Path::new(&input).extension() == Some(OsStr::new("json"));
    if !json && input != "-" && Path::new(&input).extension() != Some(OsStr::new("cue")) {
        return Err("input must be an explicit .cue/.json file or - for stdin".into());
    }
    let input = if input == "json:" { "-".into() } else { input };
    Ok(ExportArgs { input, field, json })
}

fn run() -> Result<(), String> {
    let ExportArgs { input, field, json } = export_arguments(std::env::args_os().skip(1))?;
    let value = if json {
        if input == "-" {
            json::read(io::stdin().lock())?
        } else {
            let file = std::fs::File::open(&input)
                .map_err(|error| format!("cannot read {}: {error}", Path::new(&input).display()))?;
            json::read(file)?
        }
    } else {
        cue_value(&input)?
    };
    let result = select(&value, field.as_ref())?;
    let output = serde_json::to_vec(result)
        .map_err(|error| format!("cannot serialize export result: {error}"))?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&output)
        .and_then(|()| stdout.write_all(b"\n"))
        .and_then(|()| stdout.flush())
        .map_err(|error| format!("cannot write stdout: {error}"))
}

fn cue_value(input: &OsStr) -> Result<serde_json::Value, String> {
    let source = if input == "-" {
        let mut source = String::new();
        io::stdin()
            .lock()
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read stdin: {error}"))?;
        source
    } else {
        let path = Path::new(input);
        std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?
    };
    let program = bootstrap_executor::parser::parse_file(
        &source,
        include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json")),
    )
    .map_err(|error| error.to_string())?;
    let program = serde_json::to_string(&program)
        .map_err(|error| format!("cannot serialize parsed program: {error}"))?;
    bootstrap_executor::generated::execute_json(&program).map_err(|error| error.to_string())
}

fn select<'a>(
    value: &'a serde_json::Value,
    field: Option<&SelectorPath>,
) -> Result<&'a serde_json::Value, String> {
    let Some(path) = field else { return Ok(value) };
    let mut current = value;
    let mut prefix_len = 0;
    for (step, end) in &path.steps {
        let prefix = &path.text[..prefix_len];
        current = match step {
            SelectorStep::Field(name) => {
                let record = current.as_object().ok_or_else(|| {
                    if prefix_len == 0 {
                        "--expression requires a concrete record".into()
                    } else {
                        format!("field {prefix} is not a record")
                    }
                })?;
                record.get(name).ok_or_else(|| {
                    if prefix_len == 0 {
                        format!("record has no field named {name}")
                    } else {
                        format!("record {prefix} has no field named {name}")
                    }
                })?
            }
            SelectorStep::Index(index) => {
                let list = current
                    .as_array()
                    .ok_or_else(|| format!("field {prefix} is not a list"))?;
                list.get(*index).ok_or_else(|| {
                    format!("index out of range [{index}] with length {}", list.len())
                })?
            }
        };
        prefix_len = *end;
    }
    Ok(current)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "cuestacean: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(args: &[&str]) -> Result<ExportArgs, String> {
        export_arguments(args.iter().map(|arg| OsString::from(*arg)))
    }

    fn select_path<'a>(
        value: &'a serde_json::Value,
        path: &str,
    ) -> Result<&'a serde_json::Value, String> {
        let path = field_argument(Some(path.into()))?;
        select(value, Some(&path))
    }

    #[test]
    fn typed_selector_paths_and_argument_rejections() {
        let path = field_argument(Some("items[0][12].true".into())).unwrap();
        assert_eq!(
            path.steps,
            vec![
                (SelectorStep::Field("items".into()), 5),
                (SelectorStep::Index(0), 8),
                (SelectorStep::Index(12), 12),
                (SelectorStep::Field("true".into()), 17),
            ]
        );
        assert!(field_argument(Some("items[268435454]".into())).is_ok());
        for path in [
            "items[268435455]",
            "items[9223372036854775807]",
            "items[18446744073709551616]",
        ] {
            assert!(
                field_argument(Some(path.into()))
                    .unwrap_err()
                    .starts_with("unsupported selector index:")
            );
        }
        for path in [
            "", "[0]", "true", "false.x", "null[0]", "_x", "x..y", "x.", "x[-1]", "x[01]", "x[00]",
            "x[+1]", "x[0x0]", "x[1+1]", "x[0:1]", "x[]", "x[0", "x[0]y", "x.[0]", "x['a']",
            "x[ 0]", "x[0 ]", "x.é", "x[０]", "x[0].", "x[0]..y", "x.\"y\"",
        ] {
            // Argument parsing rejects the selector even with an unreadable input.
            let error = arguments(&["export", "does-not-exist.cue", "-e", path])
                .err()
                .expect("invalid argument");
            assert!(
                error.starts_with("--expression requires an ASCII field path"),
                "{path}: {error}"
            );
        }
        assert!(field_argument(None).is_err());
    }

    #[test]
    fn mixed_selector_values_and_full_prefix_diagnostics() {
        let value = serde_json::json!({
            "items": [[{"answer": 42, "true": null}], null, false, {}, "text"],
            "empty": [], "items[0]": 99, "scope.answer": 99,
            "scope": {"answer": null}
        });
        for (path, expected) in [
            ("items[0][0].answer", serde_json::json!(42)),
            ("items[0][0].true", serde_json::Value::Null),
            ("items[1]", serde_json::Value::Null),
            ("scope.answer", serde_json::Value::Null),
        ] {
            assert_eq!(select_path(&value, path).unwrap(), &expected);
        }
        assert_eq!(select(&value, None).unwrap(), &value);
        for (path, expected) in [
            ("missing", "record has no field named missing"),
            (
                "items[0][0].missing",
                "record items[0][0] has no field named missing",
            ),
            ("items[1].answer", "field items[1] is not a record"),
            (
                "items[0][0].answer.more",
                "field items[0][0].answer is not a record",
            ),
            ("items[0][0][0]", "field items[0][0] is not a list"),
            ("items[1][0]", "field items[1] is not a list"),
            ("items[2][0]", "field items[2] is not a list"),
            ("items[3][0]", "field items[3] is not a list"),
            ("items[4][0]", "field items[4] is not a list"),
            ("scope.answer[0]", "field scope.answer is not a list"),
            ("empty[0]", "index out of range [0] with length 0"),
            ("items[5]", "index out of range [5] with length 5"),
            (
                "items[268435454]",
                "index out of range [268435454] with length 5",
            ),
        ] {
            assert_eq!(select_path(&value, path).unwrap_err(), expected, "{path}");
        }
        assert_eq!(
            select_path(&serde_json::json!([]), "items[0]").unwrap_err(),
            "--expression requires a concrete record"
        );
        assert_eq!(
            select_path(&serde_json::json!({"items[0]": 42}), "items[0]").unwrap_err(),
            "record has no field named items"
        );
    }

    #[test]
    fn explicit_formats_options_and_selectors() {
        for args in [
            vec!["export", "json:", "-", "--out=json", "-e", "answer"],
            vec!["export", "--out", "json", "-e", "answer", "data.json"],
            vec!["export", "-e", "answer", "--", "json:", "-"],
        ] {
            let parsed = arguments(&args).expect("supported explicit input and options");
            assert!(parsed.json);
            assert_eq!(
                parsed.field.as_ref().map(|path| path.text.as_str()),
                Some("answer")
            );
        }
        for input in ["-", "file.cue"] {
            assert!(
                !arguments(&["export", input])
                    .expect("existing CUE route")
                    .json
            );
        }
        for args in [
            vec!["export", "json:"],
            vec!["export", "json:", "--out=json", "-"],
            vec!["export", "json:", "data.json"],
            vec!["export", "json:", "-", "other.json"],
            vec!["export", "data.JSON"],
        ] {
            assert!(arguments(&args).is_err());
        }
        let value =
            json::read(&br#"{"answer":42,"other":false}"#[..]).expect("valid selection data");
        assert_eq!(
            select_path(&value, "answer").expect("existing answer field"),
            &serde_json::json!(42)
        );
        assert_eq!(
            select_path(&value, "missing").expect_err("absent field"),
            "record has no field named missing"
        );
        assert_eq!(
            select_path(&serde_json::Value::Null, "answer").expect_err("non-record"),
            "--expression requires a concrete record"
        );
    }
}
