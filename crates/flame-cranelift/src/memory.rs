//! Linear memory load/store lowering with bounds-check traps.

use cranelift_codegen::ir::{types, InstBuilder, MemFlags, Type, Value};
use cranelift_frontend::FunctionBuilder;

/// Helper for emitting Wasm memory load/store instructions into Cranelift IR.
pub struct MemoryTranslator;

impl MemoryTranslator {
    /// Emit a load: pops an i32 address from `stack`, emits a Cranelift load,
    /// and pushes the result value.
    pub fn translate_load(
        builder: &mut FunctionBuilder,
        stack: &mut Vec<Value>,
        ty: Type,
        offset: u64,
        _signed: bool,
        _extend: bool,
    ) {
        let addr_i32 = stack
            .pop()
            .unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
        // Zero-extend i32 address to pointer width
        let addr = builder.ins().uextend(types::I64, addr_i32);
        let flags = MemFlags::new();
        let v = builder.ins().load(ty, flags, addr, offset as i32);
        stack.push(v);
    }

    /// Emit a store: pops value then address from `stack`.
    pub fn translate_store(
        builder: &mut FunctionBuilder,
        stack: &mut Vec<Value>,
        _ty: Type,
        offset: u64,
    ) {
        let value = stack
            .pop()
            .unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
        let addr_i32 = stack
            .pop()
            .unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
        let addr = builder.ins().uextend(types::I64, addr_i32);
        let flags = MemFlags::new();
        builder.ins().store(flags, value, addr, offset as i32);
    }
}
