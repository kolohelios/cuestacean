//! Internal Pratt compiler façade, not the executor or runtime parser.
//! Shared implementation remains licensed under Apache-2.0.

#[path = "../../executor/src/lowering.rs"]
pub mod lowering;
#[path = "../../executor/src/module.rs"]
pub mod module;
#[path = "../../executor/src/pratt/compiler.rs"]
pub mod pratt;
#[path = "../../executor/src/unification/kind.rs"]
pub mod unification;

#[cfg(feature = "compiled-lowering")]
extern crate self as bootstrap_executor;

/// Freshly generated Pratt compiler policy over the shared rendering kernel.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/pratt-lowering.rs"]
pub mod compiled_pratt_lowering;

/// Freshly generated complete lowering-driver compiler policy.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/meta-lowering.rs"]
pub mod compiled_meta_lowering;
