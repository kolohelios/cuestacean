pub mod compiler;

impl From<compiler::Error> for crate::Error {
    fn from(error: compiler::Error) -> Self {
        match error {
            compiler::Error::Json { document, source } => Self::Json { document, source },
            compiler::Error::Version { document, version } => Self::Version { document, version },
            compiler::Error::Lowering { source } => Self::Lowering { source },
        }
    }
}

/// Run CUE-owned module assembly and instruction lowering.
pub fn generate(input: &str) -> Result<String, crate::Error> {
    compiler::generate(input).map_err(crate::Error::from)
}

/// Override the trusted compiler program, not the input instruction language.
pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, crate::Error> {
    compiler::generate_with_policy(input, policy).map_err(crate::Error::from)
}
