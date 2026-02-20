//! Import registry — maps (module, name) to host function pointers.

use std::collections::HashMap;

/// A registry of host functions supplied at instantiation.
#[derive(Default)]
pub struct Imports {
    funcs: HashMap<(String, String), *const u8>,
}

impl Imports {
    pub fn new() -> Self { Self::default() }

    /// Register a host function pointer for the given `(module, name)` import key.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime of the [`Instance`] and
    /// must have a C-ABI compatible signature matching the Wasm import type.
    pub unsafe fn register_func(&mut self, module: &str, name: &str, ptr: *const u8) {
        self.funcs.insert((module.to_owned(), name.to_owned()), ptr);
    }

    /// Look up a registered host function.
    pub fn get_func(&self, module: &str, name: &str) -> Option<*const u8> {
        self.funcs.get(&(module.to_owned(), name.to_owned())).copied()
    }
}

// SAFETY: Imports only holds function pointers assigned by the host.
unsafe impl Send for Imports {}
unsafe impl Sync for Imports {}
