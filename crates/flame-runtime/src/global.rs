//! Global variable storage.

use flame_core::ir::ValType;

use crate::val::Val;

/// A single WebAssembly global variable.
pub struct Global {
    pub value: Val,
    pub mutable: bool,
}

impl Global {
    pub fn new(val_type: ValType, mutable: bool) -> Self {
        let value = match val_type {
            ValType::I32 | ValType::FuncRef | ValType::ExternRef => Val::I32(0),
            ValType::I64 => Val::I64(0),
            ValType::F32 => Val::F32(0.0),
            ValType::F64 => Val::F64(0.0),
            ValType::V128 => Val::I64(0),
        };
        Self { value, mutable }
    }

    pub fn get(&self) -> &Val { &self.value }

    pub fn set(&mut self, val: Val) -> anyhow::Result<()> {
        if !self.mutable {
            anyhow::bail!("attempt to write to immutable global");
        }
        self.value = val;
        Ok(())
    }
}
