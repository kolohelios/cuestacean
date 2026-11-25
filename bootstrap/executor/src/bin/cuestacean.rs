use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

#[path = "cuestacean/json.rs"]
mod json;

const USAGE: &str =
    "usage: cuestacean export [-e name] [--out json] [-o -] <input.cue|input.json|json: -|->";

struct ExportArgs {
    input: OsString,
    field: Option<String>,
    json: bool,
}

fn field_argument(field: Option<OsString>) -> Result<String, String> {
    field
        .and_then(|field| field.into_string().ok())
        .filter(|field| {
            let mut bytes = field.bytes();
            matches!(bytes.next(), Some(first) if first.is_ascii_alphabetic())
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && !matches!(field.as_str(), "true" | "false" | "null")
        })
        .ok_or_else(|| "--expression requires a single ASCII field name (not a literal)".into())
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
    let result = select(&value, field.as_deref())?;
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
    field: Option<&str>,
) -> Result<&'a serde_json::Value, String> {
    match field {
        Some(name) => value
            .as_object()
            .ok_or("--expression requires a concrete record")?
            .get(name)
            .ok_or_else(|| format!("record has no field named {name}")),
        None => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(args: &[&str]) -> Result<ExportArgs, String> {
        export_arguments(args.iter().map(|arg| OsString::from(*arg)))
    }

    #[test]
    fn explicit_formats_options_and_selectors() {
        for args in [
            vec!["export", "json:", "-", "--out=json", "-e", "answer"],
            vec!["export", "--out", "json", "-e", "answer", "data.json"],
            vec!["export", "-e", "answer", "--", "json:", "-"],
        ] {
            let parsed = arguments(&args).unwrap();
            assert!(parsed.json);
            assert_eq!(parsed.field.as_deref(), Some("answer"));
        }
        for input in ["-", "file.cue"] {
            assert!(!arguments(&["export", input]).unwrap().json);
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
        let value = json::read(&br#"{"answer":42,"other":false}"#[..]).unwrap();
        assert_eq!(
            select(&value, Some("answer")).unwrap(),
            &serde_json::json!(42)
        );
        assert_eq!(
            select(&value, Some("missing")).unwrap_err(),
            "record has no field named missing"
        );
        assert_eq!(
            select(&serde_json::Value::Null, Some("answer")).unwrap_err(),
            "--expression requires a concrete record"
        );
    }
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
