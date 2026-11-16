use std::path::PathBuf;
use std::process::ExitCode;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("usage: bootstrap-scan-codegen <program.json> [lowering.json]"))]
    Usage,
    #[snafu(display("cannot read {}: {source}", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{source}"))]
    Generate {
        source: bootstrap_executor::scanner::Error,
    },
}

fn run() -> Result<String, Error> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or(Error::Usage)?);
    let policy = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return UsageSnafu.fail();
    }
    let program = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
    match policy {
        Some(path) => {
            let policy = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
            bootstrap_executor::scanner::generate_with_policy(&program, &policy)
                .context(GenerateSnafu)
        }
        None => bootstrap_executor::scanner::generate(&program).context(GenerateSnafu),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(source) => {
            print!("{source}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
