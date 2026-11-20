//! Internal unification compiler façade, not the executor or runtime evaluator.
//! Shared implementation remains licensed under Apache-2.0.

#[path = "../../executor/src/lowering.rs"]
pub mod lowering;
#[path = "../../executor/src/module.rs"]
pub mod module;
#[path = "../../executor/src/unification/compiler.rs"]
pub mod unification;

#[cfg(feature = "compiled-lowering")]
extern crate self as bootstrap_executor;

/// Freshly generated unification compiler policy over the shared rendering kernel.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/unify-lowering.rs"]
pub mod compiled_unify_lowering;

/// Freshly generated complete lowering-driver compiler policy.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/meta-lowering.rs"]
pub mod compiled_meta_lowering;
