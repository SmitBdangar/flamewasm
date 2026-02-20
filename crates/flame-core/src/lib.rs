//! `flame-core`: WebAssembly binary decoder, IR, and spec-compliant validator.
//!
//! # Example
//! ```no_run
//! use flame_core::{parse, validate};
//!
//! let bytes = std::fs::read("hello.wasm").unwrap();
//! let module = parse(&bytes).expect("failed to parse");
//! validate(&module).expect("validation failed");
//! ```

pub mod error;
pub mod ir;
pub mod parser;
pub mod validator;

pub use error::{ParseError, ValidationError};
pub use ir::Module;
pub use parser::parse;
pub use validator::validate;
