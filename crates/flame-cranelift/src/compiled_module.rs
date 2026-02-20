//! The compiled module produced by the AOT compiler.

use std::collections::HashMap;
use cranelift_jit::JITModule;
use cranelift_module::FuncId;

/// Holds the JIT-compiled native code and symbol metadata.
pub struct CompiledModule {
    /// The live JIT module (keeps the code pages alive).
    pub jit_module: JITModule,
    /// Function IDs in declaration order (imports first, then locals).
    pub func_ids: Vec<FuncId>,
    /// Map from export name to raw function pointer address.
    pub exports: HashMap<String, usize>,
    /// Number of imported functions (prefix of `func_ids`).
    pub imported_func_count: usize,
}

impl CompiledModule {
    /// Look up the raw pointer for an exported function by name.
    #[must_use]
    pub fn get_export(&self, name: &str) -> Option<*const u8> {
        self.exports.get(name).map(|&addr| addr as *const u8)
    }

    /// Returns all exported function names.
    pub fn export_names(&self) -> impl Iterator<Item = &str> {
        self.exports.keys().map(String::as_str)
    }
}

// Safety: JITModule keeps code pages alive; pointers are valid as long as this struct lives.
// SAFETY: The JIT code region is owned; we never mutate it after finalization.
unsafe impl Send for CompiledModule {}
unsafe impl Sync for CompiledModule {}
