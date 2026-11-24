//! Internal instruction compiler façade, not the executor or evaluator runtime.
//! Shared implementation remains licensed under Apache-2.0.

#[path = "../../executor/src/codegen/compiler.rs"]
pub mod compiler;
#[path = "../../executor/src/lowering.rs"]
pub mod lowering;
#[path = "../../executor/src/module.rs"]
pub mod module;

pub use compiler::{Error, generate, generate_with_policy};

#[cfg(feature = "compiled-lowering")]
extern crate self as bootstrap_executor;

/// Freshly generated instruction compiler policy over the shared rendering kernel.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/lowering.rs"]
pub mod compiled_lowering;

/// Freshly generated complete lowering-driver compiler policy.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/meta-lowering.rs"]
pub mod compiled_meta_lowering;
