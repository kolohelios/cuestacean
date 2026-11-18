//! Isolated native bootstrap exporter, not a general CUE implementation.
mod bindings;
mod eval;
mod model;
mod parser;
pub use model::Error;
use std::io::{self, Write};

struct Output(Vec<u8>);
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > 65536 - self.0.len() {
            return Err(io::Error::other("encoded output limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn export(schema: &str, program: &str) -> Result<String, Error> {
    export_with_limit(schema, program, 4096)
}

/// Lower the cumulative evaluation allowance; the hard maximum stays unchanged.
pub fn export_with_limit(schema: &str, program: &str, work: usize) -> Result<String, Error> {
    let document = parser::parse(schema, program)?;
    let value = eval::evaluate(&document, work)?;
    let mut output = Output(Vec::new());
    serde_json::to_writer_pretty(&mut output, &value).map_err(|_| Error::Limit("encoded bytes"))?;
    output
        .write_all(b"\n")
        .map_err(|_| Error::Limit("encoded bytes"))?;
    String::from_utf8(output.0).map_err(|_| Error::Syntax("invalid encoded UTF-8"))
}
