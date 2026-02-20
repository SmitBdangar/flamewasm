//! Trap types and handling for FlameWasm.

use thiserror::Error;

/// A WebAssembly trap (synchronous runtime error).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Trap {
    #[error("unreachable instruction executed")]
    Unreachable,

    #[error("out-of-bounds memory access at offset {offset:#x}")]
    MemoryOutOfBounds { offset: usize },

    #[error("out-of-bounds table access at index {index}")]
    TableOutOfBounds { index: u32 },

    #[error("integer divide by zero")]
    IntegerDivisionByZero,

    #[error("integer overflow in integer operation")]
    IntegerOverflow,

    #[error("invalid conversion to integer (NaN or infinity)")]
    InvalidConversionToInt,

    #[error("call_indirect: null function reference")]
    NullFunctionReference,

    #[error("call_indirect: type signature mismatch (expected type {expected_type_idx})")]
    BadSignature { expected_type_idx: u32 },

    #[error("stack overflow")]
    StackOverflow,

    #[error("host function error: {0}")]
    HostTrap(String),

    #[error("WASI exit with code {0}")]
    Exit(i32),
}
