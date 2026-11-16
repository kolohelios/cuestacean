use snafu::{ResultExt, Snafu};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("usage: bootstrap-pratt-codegen <control.json> [lowering.json]"))]
    Usage,
    #[snafu(display("cannot read {}: {source}", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{source}"))]
    Generate {
        source: bootstrap_executor::pratt::Error,
    },
}
fn run() -> Result<String, Error> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or(Error::Usage)?);
    let policy = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return UsageSnafu.fail();
    }
    let input = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
    match policy {
        Some(path) => {
            let policy = std::fs::read_to_string(&path).context(ReadSnafu { path })?;
            bootstrap_executor::pratt::generate_with_policy(&input, &policy).context(GenerateSnafu)
        }
        None => bootstrap_executor::pratt::generate(&input).context(GenerateSnafu),
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
