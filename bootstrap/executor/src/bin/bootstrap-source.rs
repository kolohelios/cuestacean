use std::path::PathBuf;
use std::process::ExitCode;

use snafu::{ResultExt, Snafu};

#[path = "../../../generated/evaluation.rs"]
mod evaluator;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("usage: bootstrap-source <expression.cue>"))]
    Usage,
    #[snafu(display("cannot read {}: {source}", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{source}"))]
    Parse {
        source: bootstrap_executor::parser::Error,
    },
    #[snafu(display("cannot serialize parsed program: {source}"))]
    Serialize { source: serde_json::Error },
    #[snafu(display("{source}"))]
    Execute { source: bootstrap_executor::Error },
}

fn run() -> Result<i64, Error> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or(Error::Usage)?);
    if args.next().is_some() {
        return UsageSnafu.fail();
    }
    let source = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
    let program = bootstrap_executor::parser::parse(&source, include_str!("../../../grammar.json"))
        .context(ParseSnafu)?;
    let program = serde_json::to_string(&program).context(SerializeSnafu)?;
    evaluator::execute(&program).context(ExecuteSnafu)
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
