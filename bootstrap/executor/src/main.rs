use std::path::PathBuf;
use std::process::ExitCode;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("usage: bootstrap-executor <program.json> <rules.json>"))]
    Usage,
    #[snafu(display("cannot read {}: {source}", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{source}"))]
    Execute { source: bootstrap_executor::Error },
}

fn run() -> Result<serde_json::Value, Error> {
    let mut args = std::env::args_os().skip(1);
    let program = PathBuf::from(args.next().ok_or(Error::Usage)?);
    let rules = PathBuf::from(args.next().ok_or(Error::Usage)?);
    if args.next().is_some() {
        return UsageSnafu.fail();
    }
    let program = std::fs::read_to_string(&program).context(ReadSnafu { path: program })?;
    let rules = std::fs::read_to_string(&rules).context(ReadSnafu { path: rules })?;
    bootstrap_executor::execute_json(&program, &rules).context(ExecuteSnafu)
}

fn main() -> ExitCode {
    match run() {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
