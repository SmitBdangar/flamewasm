//! Error types for the flame-core crate.

use thiserror::Error;

/// Errors that can occur during WebAssembly binary parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected end of input at offset {offset}")]
    UnexpectedEof { offset: usize },

    #[error("invalid magic number: expected \\0asm, got {got:?}")]
    InvalidMagic { got: [u8; 4] },

    #[error("unsupported WebAssembly version: {0}")]
    UnsupportedVersion(u32),

    #[error("malformed section id {id} at offset {offset}")]
    MalformedSection { id: u8, offset: usize },

    #[error("wasmparser error: {0}")]
    WasmParser(#[from] wasmparser::BinaryReaderError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Errors that can occur during module validation.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("type mismatch: expected {expected}, got {got} at instruction {instruction}")]
    TypeMismatch {
        expected: String,
        got: String,
        instruction: String,
    },

    #[error("function index {0} is out of bounds (total functions: {1})")]
    FuncIndexOob(u32, usize),

    #[error("type index {0} is out of bounds (total types: {1})")]
    TypeIndexOob(u32, usize),

    #[error("memory index {0} is out of bounds (total memories: {1})")]
    MemoryIndexOob(u32, usize),

    #[error("table index {0} is out of bounds (total tables: {1})")]
    TableIndexOob(u32, usize),

    #[error("global index {0} is out of bounds (total globals: {1})")]
    GlobalIndexOob(u32, usize),

    #[error("unreachable stack underflow in function {func_idx}")]
    StackUnderflow { func_idx: u32 },

    #[error("expected function body for function {0} but it is missing")]
    MissingFunctionBody(u32),

    #[error("break depth {depth} exceeds control-flow stack depth {stack_depth}")]
    InvalidBreakDepth { depth: u32, stack_depth: usize },

    #[error("multiple memories are not supported (found {0})")]
    MultipleMemories(usize),

    #[error("immutable global {0} cannot be mutated")]
    ImmutableGlobalWrite(u32),

    #[error("export '{name}' references undefined {kind} {index}")]
    UndefinedExport {
        name: String,
        kind: String,
        index: u32,
    },

    #[error("{0}")]
    Other(String),
}
