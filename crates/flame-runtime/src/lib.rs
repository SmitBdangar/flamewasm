//! `flame-runtime`: WebAssembly instance execution engine.

pub mod global;
pub mod imports;
pub mod instance;
pub mod memory;
pub mod table;
pub mod trap;
pub mod val;

pub use imports::Imports;
pub use instance::Instance;
pub use trap::Trap;
pub use val::Val;
