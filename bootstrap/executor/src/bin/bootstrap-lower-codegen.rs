use std::path::PathBuf;
use std::process::ExitCode;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("usage: bootstrap-lower-codegen <policy.json> [compiler.json]"))]
    Usage,
    #[snafu(display("cannot read {}: {source}", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{source}"))]
    Generate {
        source: bootstrap_executor::lowering::Error,
    },
}

fn run() -> Result<String, Error> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or(Error::Usage)?);
    let compiler = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return UsageSnafu.fail();
    }
    let policy = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
    match compiler {
        Some(path) => {
            let compiler = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
            bootstrap_executor::lowering::generate_with_policy(&policy, &compiler)
                .context(GenerateSnafu)
        }
        None => bootstrap_executor::lowering::generate(&policy).context(GenerateSnafu),
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
