use std::{
    error::Error,
    io::{Read, Write},
    path::Path,
    process::ExitCode,
};

fn source(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(65537)
        .read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err(cuestacean_spec_export::Error::Limit("source bytes").into());
    }
    Ok(text)
}
fn run() -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.len() != 2 {
        return Err("usage: cuestacean-spec-export <schema.cue> <program.cue>".into());
    }
    let output = cuestacean_spec_export::export(
        &source(Path::new(&paths[0]))?,
        &source(Path::new(&paths[1]))?,
    )?;
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
