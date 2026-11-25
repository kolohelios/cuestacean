use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: cuestacean export [--out json] [-o -] <expression.cue|->";

fn input_argument(mut args: impl Iterator<Item = OsString>) -> Result<OsString, String> {
    if args.next().as_deref() != Some(OsStr::new("export")) {
        return Err(format!("only the export command is supported\n{USAGE}"));
    }
    let mut input = None;
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
                _ if arg.as_encoded_bytes().starts_with(b"-") && arg != "-" => {
                    return Err(format!(
                        "unsupported export option: {}",
                        arg.to_string_lossy()
                    ));
                }
                _ => {}
            }
        }
        if input.replace(arg).is_some() {
            return Err(format!("export requires exactly one input\n{USAGE}"));
        }
    }
    let input = input.ok_or_else(|| format!("export requires exactly one input\n{USAGE}"))?;
    if input != "-" && Path::new(&input).extension() != Some(OsStr::new("cue")) {
        return Err("input must be an explicit .cue file or - for stdin".into());
    }
    Ok(input)
}

fn run() -> Result<(), String> {
    let input = input_argument(std::env::args_os().skip(1))?;
    let source = if input == "-" {
        let mut source = String::new();
        io::stdin()
            .lock()
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read stdin: {error}"))?;
        source
    } else {
        let path = Path::new(&input);
        std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?
    };
    let program = bootstrap_executor::parser::parse(
        &source,
        include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json")),
    )
    .map_err(|error| error.to_string())?;
    let program = serde_json::to_string(&program)
        .map_err(|error| format!("cannot serialize parsed program: {error}"))?;
    let value =
        bootstrap_executor::generated::execute_json(&program).map_err(|error| error.to_string())?;
    let output = serde_json::to_vec(&value)
        .map_err(|error| format!("cannot serialize export result: {error}"))?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&output)
        .and_then(|()| stdout.write_all(b"\n"))
        .and_then(|()| stdout.flush())
        .map_err(|error| format!("cannot write stdout: {error}"))
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
