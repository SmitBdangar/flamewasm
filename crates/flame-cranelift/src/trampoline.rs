//! Host-function trampolines for imported WASI/host functions.
//!
//! When the JIT calls an imported function it goes through a trampoline that
//! marshals values from the native calling convention into the calling context.
//! This module is a lightweight placeholder; full trampoline generation is
//! handled by the platform ABI in Cranelift automatically via `declare_function`.

/// A host function callable from compiled Wasm code.
///
/// The raw pointer must point to a C-ABI function whose signature matches the
/// Wasm function type it was declared for.
pub struct Trampoline {
    pub name: String,
    pub ptr: *const u8,
}

// SAFETY: Trampolines hold function pointers to static host functions.
unsafe impl Send for Trampoline {}
unsafe impl Sync for Trampoline {}
