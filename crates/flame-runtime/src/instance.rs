//! WebAssembly instance — owns linear memory, tables, globals, and the JIT code.

use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use flame_core::ir::{ConstExpr, DataKind, ExportKind, Module};
use tracing::debug;

use crate::{
    global::Global, imports::Imports, memory::LinearMemory, table::Table, val::Val,
};
use flame_cranelift::CompiledModule;

/// A live WebAssembly module instance.
pub struct Instance {
    /// Linear memories (usually just one).
    pub memories: Vec<LinearMemory>,
    /// Tables.
    pub tables: Vec<Table>,
    /// Globals.
    pub globals: Vec<Global>,
    /// The compiled module (keeps JIT code alive).
    pub compiled: CompiledModule,
    /// Export name → function pointer.
    pub func_exports: HashMap<String, *const u8>,
}

impl Instance {
    /// Instantiate a compiled module with the provided imports.
    pub fn new(
        wasm: &Module,
        compiled: CompiledModule,
        _imports: &Imports,
    ) -> Result<Self> {
        // Allocate memories
        let mut memories: Vec<LinearMemory> = wasm
            .memories
            .iter()
            .map(|m| LinearMemory::new(m.limits.min, m.limits.max))
            .collect();

        // Allocate tables
        let tables: Vec<Table> = wasm
            .tables
            .iter()
            .map(|t| Table::new(t.limits.min, t.limits.max))
            .collect();

        // Allocate globals
        let globals: Vec<Global> = wasm
            .globals
            .iter()
            .map(|g| {
                let mut gl = Global::new(g.ty.val_type, g.ty.mutable);
                match &g.init {
                    ConstExpr::I32Const(v) => gl.value = Val::I32(*v),
                    ConstExpr::I64Const(v) => gl.value = Val::I64(*v),
                    ConstExpr::F32Const(v) => gl.value = Val::F32(*v),
                    ConstExpr::F64Const(v) => gl.value = Val::F64(*v),
                    _ => {}
                }
                gl
            })
            .collect();

        // Initialize data segments
        for seg in &wasm.data {
            if let DataKind::Active { memory_index, offset } = &seg.kind {
                let mem = memories
                    .get_mut(*memory_index as usize)
                    .context("data segment references invalid memory")?;
                let offset_val = eval_const_expr(offset, &globals);
                let offset_usize = offset_val as usize;
                mem.store_bytes(offset_usize, &seg.data)
                    .map_err(|t| anyhow::anyhow!("data segment init: {t}"))?;
                debug!("initialized data segment at memory[{memory_index}]+{offset_usize}, {} bytes", seg.data.len());
            }
        }

        // Collect function exports
        let func_exports = compiled.exports.iter()
            .filter_map(|(name, &ptr)| {
                wasm.exports.iter()
                    .find(|e| e.kind == ExportKind::Func && &e.name == name)
                    .map(|_| (name.clone(), ptr as *const u8))
            })
            .collect::<HashMap<_, _>>();

        let inst = Self { memories, tables, globals, compiled, func_exports };

        // Run start function if present
        if let Some(start_idx) = wasm.start {
            debug!("executing start function {start_idx}");
            // Look up start export by index
            let start_export = wasm.exports.iter().find(|e| {
                e.kind == ExportKind::Func && e.index == start_idx
            });
            if let Some(exp) = start_export {
                inst.call(&exp.name, &[])?;
            }
        }

        Ok(inst)
    }

    /// Call an exported function by name with the given arguments.
    ///
    /// # Safety
    /// The function pointer is valid as long as the [`CompiledModule`] is alive.
    pub fn call(&self, name: &str, args: &[Val]) -> Result<Vec<Val>> {
        let ptr = self
            .func_exports
            .get(name)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("export '{name}' not found"))?;

        debug!("calling export '{name}' at {ptr:?}");

        // For MVP: support () -> i32 calls (extensible for richer signatures)
        // This dispatcher handles the most common cases.
        let result = match args {
            [] => unsafe { call_fn_0(ptr) },
            [Val::I32(a)] => unsafe { call_fn_i32(ptr, *a) },
            [Val::I32(a), Val::I32(b)] => unsafe { call_fn_i32_i32(ptr, *a, *b) },
            _ => bail!("call signature not yet supported for export '{name}'"),
        };
        Ok(vec![Val::I32(result)])
    }
}

fn eval_const_expr(expr: &ConstExpr, _globals: &[Global]) -> i64 {
    match expr {
        ConstExpr::I32Const(v) => *v as i64,
        ConstExpr::I64Const(v) => *v,
        _ => 0,
    }
}

// ─── Low-level callers ────────────────────────────────────────────────────────
// These are FFI trampolines that call into JIT-compiled code.

type FnUnit = unsafe extern "C" fn() -> i32;
type FnI32  = unsafe extern "C" fn(i32) -> i32;
type FnI32I32 = unsafe extern "C" fn(i32, i32) -> i32;

unsafe fn call_fn_0(ptr: *const u8) -> i32 {
    let f: FnUnit = std::mem::transmute(ptr);
    f()
}
unsafe fn call_fn_i32(ptr: *const u8, a: i32) -> i32 {
    let f: FnI32 = std::mem::transmute(ptr);
    f(a)
}
unsafe fn call_fn_i32_i32(ptr: *const u8, a: i32, b: i32) -> i32 {
    let f: FnI32I32 = std::mem::transmute(ptr);
    f(a, b)
}
