//! Internal AST/lowering compiler façade, not the executor or source evaluator.
//! Shared implementation remains licensed under Apache-2.0.

#[path = "../../executor/src/ast.rs"]
pub mod ast;
#[path = "../../executor/src/lowering.rs"]
pub mod lowering;
#[path = "../../executor/src/module.rs"]
pub mod module;

#[cfg(feature = "compiled-lowering")]
extern crate self as bootstrap_executor;

/// Freshly generated AST compiler policy over the shared rendering kernel.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/ast-lowering.rs"]
pub mod compiled_ast_lowering;

/// Freshly generated complete lowering-driver compiler policy.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/meta-lowering.rs"]
pub mod compiled_meta_lowering;
