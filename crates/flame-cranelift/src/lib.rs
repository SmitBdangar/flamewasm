//! `flame-cranelift`: Cranelift AOT compiler backend for FlameWasm.
//!
//! Translates a [`flame_core::Module`] IR into JIT-compiled native code using
//! the Cranelift code generator.

pub mod compiled_module;
pub mod compiler;
pub mod func_translator;
pub mod memory;
pub mod trampoline;

pub use compiled_module::CompiledModule;
pub use compiler::compile;
