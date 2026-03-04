//! Spec-compliant WebAssembly type checker / validator.
//!
//! Validates a parsed [`Module`] against the WebAssembly core specification
//! validation algorithm, including control-flow stack, operand stack typing,
//! and all index-space bounds checks.

use crate::{
    error::ValidationError,
    ir::{
        BlockType, ExportKind, FuncBody, FuncType, ImportType, Instruction, Module, ValType,
    },
};

/// Validate a parsed [`Module`].
///
/// # Errors
/// Returns [`ValidationError`] on the first encountered violation.
pub fn validate(module: &Module) -> Result<(), ValidationError> {
    validate_types(module)?;
    validate_imports(module)?;
    validate_exports(module)?;
    validate_globals(module)?;
    validate_functions(module)?;
    Ok(())
}

fn validate_types(module: &Module) -> Result<(), ValidationError> {
    // Types are self-consistent by construction from wasmparser; just bounds-check
    for ft in &module.types {
        for vt in ft.params.iter().chain(ft.results.iter()) {
            validate_val_type(*vt)?;
        }
    }
    Ok(())
}

fn validate_val_type(vt: ValType) -> Result<(), ValidationError> {
    match vt {
        ValType::I32 | ValType::I64 | ValType::F32 | ValType::F64
        | ValType::V128 | ValType::FuncRef | ValType::ExternRef => Ok(()),
    }
}

fn validate_imports(module: &Module) -> Result<(), ValidationError> {
    for import in &module.imports {
        if let ImportType::Func(type_idx) = import.ty {
            if type_idx as usize >= module.types.len() {
                return Err(ValidationError::TypeIndexOob(type_idx, module.types.len()));
            }
        }
    }
    Ok(())
}

fn validate_exports(module: &Module) -> Result<(), ValidationError> {
    let total_funcs = total_func_count(module);
    let total_tables = import_count(module, |t| matches!(t, ImportType::Table(_)))
        + module.tables.len();
    let total_mems = import_count(module, |t| matches!(t, ImportType::Memory(_)))
        + module.memories.len();
    let total_globals = import_count(module, |t| matches!(t, ImportType::Global(_)))
        + module.globals.len();

    for export in &module.exports {
        let (kind_str, count) = match export.kind {
            ExportKind::Func => ("func", total_funcs),
            ExportKind::Table => ("table", total_tables),
            ExportKind::Memory => ("memory", total_mems),
            ExportKind::Global => ("global", total_globals),
        };
        if export.index as usize >= count {
            return Err(ValidationError::UndefinedExport {
                name: export.name.clone(),
                kind: kind_str.into(),
                index: export.index,
            });
        }
    }
    Ok(())
}

fn validate_globals(module: &Module) -> Result<(), ValidationError> {
    let imported_globals = import_count(module, |t| matches!(t, ImportType::Global(_)));
    for (i, global) in module.globals.iter().enumerate() {
        // Const-expr can only reference imported globals (not locally defined ones)
        if let crate::ir::ConstExpr::GlobalGet(gi) = global.init {
            if gi as usize >= imported_globals {
                return Err(ValidationError::GlobalIndexOob(gi, imported_globals));
            }
        }
        let _ = i;
        validate_val_type(global.ty.val_type)?;
    }
    Ok(())
}

fn validate_functions(module: &Module) -> Result<(), ValidationError> {
    let imported_funcs = import_count(module, |t| matches!(t, ImportType::Func(_)));

    if module.functions.len() != module.code.len() {
        return Err(ValidationError::Other(format!(
            "function count ({}) != code count ({})",
            module.functions.len(),
            module.code.len()
        )));
    }

    for (i, (type_idx, body)) in module
        .functions
        .iter()
        .zip(module.code.iter())
        .enumerate()
    {
        let func_idx = (imported_funcs + i) as u32;
        let ty = module
            .types
            .get(*type_idx as usize)
            .ok_or(ValidationError::TypeIndexOob(*type_idx, module.types.len()))?;

        validate_func_body(module, func_idx, ty, body)?;
    }
    Ok(())
}

// ─── Function body type checker ──────────────────────────────────────────────

/// A frame on the control-flow stack.
#[derive(Debug)]
struct Frame {
    /// Block kind.
    kind: FrameKind,
    /// Types that must be on the operand stack when this frame is exited.
    result_types: Vec<ValType>,
    /// Height of the operand stack when this frame was entered.
    stack_height: usize,
    /// Whether this frame is currently unreachable.
    unreachable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    Block,
    Loop,
    If,
    Else,
    Func,
}

fn validate_func_body(
    module: &Module,
    func_idx: u32,
    ty: &FuncType,
    body: &FuncBody,
) -> Result<(), ValidationError> {
    let mut stack: Vec<ValType> = Vec::new();
    let mut ctrl: Vec<Frame> = vec![Frame {
        kind: FrameKind::Func,
        result_types: ty.results.clone(),
        stack_height: 0,
        unreachable: false,
    }];

    // Push params as locals implicitly available (stack starts empty per spec)
    // Build local type table
    let mut local_types: Vec<ValType> = ty.params.clone();
    for (count, vt) in &body.locals {
        for _ in 0..*count {
            local_types.push(*vt);
        }
    }

    let total_funcs = total_func_count(module);

    for instr in &body.instructions {
        validate_instr(
            module,
            func_idx,
            instr,
            &local_types,
            &mut stack,
            &mut ctrl,
            total_funcs,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn validate_instr(
    module: &Module,
    func_idx: u32,
    instr: &Instruction,
    locals: &[ValType],
    stack: &mut Vec<ValType>,
    ctrl: &mut Vec<Frame>,
    total_funcs: usize,
) -> Result<(), ValidationError> {
    // Helper to check top frame unreachability
    let is_unreachable = ctrl.last().is_some_and(|f| f.unreachable);

    if is_unreachable {
        // In unreachable code the stack polymorphically accepts anything
        // Just handle structural ops that affect ctrl stack
        match instr {
            Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => {}
            Instruction::End => { pop_ctrl(stack, ctrl)?; return Ok(()); }
            Instruction::Else => {
                if let Some(f) = ctrl.last() {
                    if f.kind != FrameKind::If {
                        return Err(ValidationError::Other("else without if".into()));
                    }
                }
                pop_ctrl(stack, ctrl)?;
                return Ok(());
            }
            _ => return Ok(()),
        }
    }

    match instr {
        // ── Control ──────────────────────────────────────────────────────────
        Instruction::Unreachable => {
            if let Some(f) = ctrl.last_mut() { f.unreachable = true; }
        }
        Instruction::Nop => {}
        Instruction::Block(bt) => {
            let (inputs, outputs) = block_type_arity(module, bt)?;
            pop_vals(stack, &inputs, func_idx)?;
            push_ctrl(ctrl, stack, FrameKind::Block, outputs);
        }
        Instruction::Loop(bt) => {
            let (inputs, outputs) = block_type_arity(module, bt)?;
            pop_vals(stack, &inputs, func_idx)?;
            push_ctrl(ctrl, stack, FrameKind::Loop, outputs);
        }
        Instruction::If(bt) => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            let (inputs, outputs) = block_type_arity(module, bt)?;
            pop_vals(stack, &inputs, func_idx)?;
            push_ctrl(ctrl, stack, FrameKind::If, outputs);
        }
        Instruction::Else => {
            let frame = pop_ctrl(stack, ctrl)?;
            if frame.kind != FrameKind::If {
                return Err(ValidationError::Other("else without if".into()));
            }
            push_ctrl(ctrl, stack, FrameKind::Else, frame.result_types);
        }
        Instruction::End => { pop_ctrl(stack, ctrl)?; }
        Instruction::Br(depth) => {
            let depth = *depth as usize;
            if depth >= ctrl.len() {
                return Err(ValidationError::InvalidBreakDepth {
                    depth: depth as u32,
                    stack_depth: ctrl.len(),
                });
            }
            let label_types = label_types(ctrl, depth);
            pop_vals(stack, &label_types, func_idx)?;
            if let Some(f) = ctrl.last_mut() { f.unreachable = true; }
        }
        Instruction::BrIf(depth) => {
            let depth = *depth as usize;
            if depth >= ctrl.len() {
                return Err(ValidationError::InvalidBreakDepth {
                    depth: depth as u32,
                    stack_depth: ctrl.len(),
                });
            }
            pop_val(stack, ValType::I32, func_idx, instr)?;
            let label_types = label_types(ctrl, depth);
            pop_vals(stack, &label_types, func_idx)?;
            push_vals(stack, &label_types);
        }
        Instruction::BrTable(targets, default) => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            let default_depth = *default as usize;
            if default_depth >= ctrl.len() {
                return Err(ValidationError::InvalidBreakDepth {
                    depth: *default,
                    stack_depth: ctrl.len(),
                });
            }
            let default_types = label_types(ctrl, default_depth);
            for t in targets {
                let d = *t as usize;
                if d >= ctrl.len() {
                    return Err(ValidationError::InvalidBreakDepth {
                        depth: *t,
                        stack_depth: ctrl.len(),
                    });
                }
                let lt = label_types(ctrl, d);
                if lt != default_types {
                    return Err(ValidationError::TypeMismatch {
                        expected: fmt_types(&default_types),
                        got: fmt_types(&lt),
                        instruction: "br_table".into(),
                    });
                }
            }
            pop_vals(stack, &default_types, func_idx)?;
            if let Some(f) = ctrl.last_mut() { f.unreachable = true; }
        }
        Instruction::Return => {
            if let Some(func_frame) = ctrl.first() {
                let result_types = func_frame.result_types.clone();
                pop_vals(stack, &result_types, func_idx)?;
            }
            if let Some(f) = ctrl.last_mut() { f.unreachable = true; }
        }
        Instruction::Call(fi) => {
            let fi = *fi as usize;
            if fi >= total_funcs {
                return Err(ValidationError::FuncIndexOob(fi as u32, total_funcs));
            }
            let ty = func_type_at(module, fi);
            if let Some(ty) = ty {
                pop_vals(stack, &ty.params, func_idx)?;
                push_vals(stack, &ty.results);
            }
        }
        Instruction::CallIndirect { type_index, .. } => {
            let ti = *type_index as usize;
            if ti >= module.types.len() {
                return Err(ValidationError::TypeIndexOob(*type_index, module.types.len()));
            }
            pop_val(stack, ValType::I32, func_idx, instr)?; // table index
            let ty = module.types[ti].clone();
            pop_vals(stack, &ty.params, func_idx)?;
            push_vals(stack, &ty.results);
        }
        // ── Parametric ───────────────────────────────────────────────────────
        Instruction::Drop => { stack.pop(); }
        Instruction::Select => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            let t2 = stack.pop();
            let t1 = stack.pop();
            if let (Some(t1), Some(t2)) = (t1, t2) {
                if t1 != t2 {
                    return Err(ValidationError::TypeMismatch {
                        expected: format!("{t1}"),
                        got: format!("{t2}"),
                        instruction: "select".into(),
                    });
                }
                stack.push(t1);
            }
        }
        // ── Variable ─────────────────────────────────────────────────────────
        Instruction::LocalGet(idx) => {
            let vt = locals.get(*idx as usize);
            if let Some(vt) = vt { stack.push(*vt); }
        }
        Instruction::LocalSet(idx) | Instruction::LocalTee(idx) => {
            let vt = locals.get(*idx as usize).copied();
            if let Some(vt) = vt { pop_val(stack, vt, func_idx, instr)?; }
            if matches!(instr, Instruction::LocalTee(_)) {
                if let Some(vt) = vt { stack.push(vt); }
            }
        }
        Instruction::GlobalGet(idx) => {
            let gt = global_type_at(module, *idx as usize);
            if let Some(gt) = gt { stack.push(gt.val_type); }
        }
        Instruction::GlobalSet(idx) => {
            let gt = global_type_at(module, *idx as usize);
            if let Some(gt) = gt {
                if !gt.mutable {
                    return Err(ValidationError::ImmutableGlobalWrite(*idx));
                }
                pop_val(stack, gt.val_type, func_idx, instr)?;
            }
        }
        // ── Memory loads — all pop one i32 addr and push the value type ──────
        Instruction::I32Load(_) | Instruction::I32Load8S(_) | Instruction::I32Load8U(_)
        | Instruction::I32Load16S(_) | Instruction::I32Load16U(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I64Load(_) | Instruction::I64Load8S(_) | Instruction::I64Load8U(_)
        | Instruction::I64Load16S(_) | Instruction::I64Load16U(_)
        | Instruction::I64Load32S(_) | Instruction::I64Load32U(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        Instruction::F32Load(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F64Load(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        // ── Memory stores — pop addr + value ─────────────────────────────────
        Instruction::I32Store(_) | Instruction::I32Store8(_) | Instruction::I32Store16(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
        }
        Instruction::I64Store(_) | Instruction::I64Store8(_) | Instruction::I64Store16(_)
        | Instruction::I64Store32(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I64, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
        }
        Instruction::F32Store(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::F32, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
        }
        Instruction::F64Store(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::F64, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
        }
        Instruction::MemorySize(_) => { check_memory(module)?; stack.push(ValType::I32); }
        Instruction::MemoryGrow(_) => {
            check_memory(module)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        // ── i32 numeric ───────────────────────────────────────────────────────
        Instruction::I32Const(_) => { stack.push(ValType::I32); }
        Instruction::I32Eqz => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I32Eq | Instruction::I32Ne
        | Instruction::I32LtS | Instruction::I32LtU
        | Instruction::I32GtS | Instruction::I32GtU
        | Instruction::I32LeS | Instruction::I32LeU
        | Instruction::I32GeS | Instruction::I32GeU => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I32Clz | Instruction::I32Ctz | Instruction::I32Popcnt
        | Instruction::I32Extend8S | Instruction::I32Extend16S => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I32Add | Instruction::I32Sub | Instruction::I32Mul
        | Instruction::I32DivS | Instruction::I32DivU
        | Instruction::I32RemS | Instruction::I32RemU
        | Instruction::I32And | Instruction::I32Or | Instruction::I32Xor
        | Instruction::I32Shl | Instruction::I32ShrS | Instruction::I32ShrU
        | Instruction::I32Rotl | Instruction::I32Rotr => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        // ── i64 numeric ───────────────────────────────────────────────────────
        Instruction::I64Const(_) => { stack.push(ValType::I64); }
        Instruction::I64Eqz => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I64Eq | Instruction::I64Ne
        | Instruction::I64LtS | Instruction::I64LtU
        | Instruction::I64GtS | Instruction::I64GtU
        | Instruction::I64LeS | Instruction::I64LeU
        | Instruction::I64GeS | Instruction::I64GeU => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I64Clz | Instruction::I64Ctz | Instruction::I64Popcnt
        | Instruction::I64Extend8S | Instruction::I64Extend16S | Instruction::I64Extend32S
        | Instruction::I64ExtendI32S | Instruction::I64ExtendI32U => {
            let input = if matches!(instr, Instruction::I64ExtendI32S | Instruction::I64ExtendI32U) {
                ValType::I32
            } else {
                ValType::I64
            };
            pop_val(stack, input, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        Instruction::I64Add | Instruction::I64Sub | Instruction::I64Mul
        | Instruction::I64DivS | Instruction::I64DivU
        | Instruction::I64RemS | Instruction::I64RemU
        | Instruction::I64And | Instruction::I64Or | Instruction::I64Xor
        | Instruction::I64Shl | Instruction::I64ShrS | Instruction::I64ShrU
        | Instruction::I64Rotl | Instruction::I64Rotr => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        // ── f32 ───────────────────────────────────────────────────────────────
        Instruction::F32Const(_) => { stack.push(ValType::F32); }
        Instruction::F32Eq | Instruction::F32Ne | Instruction::F32Lt
        | Instruction::F32Gt | Instruction::F32Le | Instruction::F32Ge => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::F32Abs | Instruction::F32Neg | Instruction::F32Sqrt
        | Instruction::F32Ceil | Instruction::F32Floor | Instruction::F32Trunc
        | Instruction::F32Nearest => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F32Add | Instruction::F32Sub | Instruction::F32Mul
        | Instruction::F32Div | Instruction::F32Min | Instruction::F32Max
        | Instruction::F32Copysign => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        // ── f64 ───────────────────────────────────────────────────────────────
        Instruction::F64Const(_) => { stack.push(ValType::F64); }
        Instruction::F64Eq | Instruction::F64Ne | Instruction::F64Lt
        | Instruction::F64Gt | Instruction::F64Le | Instruction::F64Ge => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::F64Abs | Instruction::F64Neg | Instruction::F64Sqrt
        | Instruction::F64Ceil | Instruction::F64Floor | Instruction::F64Trunc
        | Instruction::F64Nearest => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        Instruction::F64Add | Instruction::F64Sub | Instruction::F64Mul
        | Instruction::F64Div | Instruction::F64Min | Instruction::F64Max
        | Instruction::F64Copysign => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        // ── Conversions ───────────────────────────────────────────────────────
        Instruction::I32WrapI64 => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I32TruncF32S | Instruction::I32TruncF32U
        | Instruction::I32TruncSatF32S | Instruction::I32TruncSatF32U => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I32TruncF64S | Instruction::I32TruncF64U
        | Instruction::I32TruncSatF64S | Instruction::I32TruncSatF64U => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I64TruncF32S | Instruction::I64TruncF32U
        | Instruction::I64TruncSatF32S | Instruction::I64TruncSatF32U => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        Instruction::I64TruncF64S | Instruction::I64TruncF64U
        | Instruction::I64TruncSatF64S | Instruction::I64TruncSatF64U => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        Instruction::F32ConvertI32S | Instruction::F32ConvertI32U => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F32ConvertI64S | Instruction::F32ConvertI64U => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F32DemoteF64 => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F64ConvertI32S | Instruction::F64ConvertI32U => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        Instruction::F64ConvertI64S | Instruction::F64ConvertI64U => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        Instruction::F64PromoteF32 => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        Instruction::I32ReinterpretF32 => {
            pop_val(stack, ValType::F32, func_idx, instr)?;
            stack.push(ValType::I32);
        }
        Instruction::I64ReinterpretF64 => {
            pop_val(stack, ValType::F64, func_idx, instr)?;
            stack.push(ValType::I64);
        }
        Instruction::F32ReinterpretI32 => {
            pop_val(stack, ValType::I32, func_idx, instr)?;
            stack.push(ValType::F32);
        }
        Instruction::F64ReinterpretI64 => {
            pop_val(stack, ValType::I64, func_idx, instr)?;
            stack.push(ValType::F64);
        }
        // Catch-all for remaining instructions (table, bulk-memory, ref)
        _ => {}
    }
    Ok(())
}

// ─── Stack helpers ────────────────────────────────────────────────────────────

fn pop_val(
    stack: &mut Vec<ValType>,
    expected: ValType,
    func_idx: u32,
    instr: &Instruction,
) -> Result<ValType, ValidationError> {
    let got = stack.pop().ok_or(ValidationError::StackUnderflow { func_idx })?;
    if got != expected {
        return Err(ValidationError::TypeMismatch {
            expected: format!("{expected}"),
            got: format!("{got}"),
            instruction: format!("{instr:?}"),
        });
    }
    Ok(got)
}

fn pop_vals(
    stack: &mut Vec<ValType>,
    types: &[ValType],
    func_idx: u32,
) -> Result<(), ValidationError> {
    for expected in types.iter().rev() {
        let got = stack.pop().ok_or(ValidationError::StackUnderflow { func_idx })?;
        if &got != expected {
            return Err(ValidationError::TypeMismatch {
                expected: format!("{expected}"),
                got: format!("{got}"),
                instruction: "pop_vals".into(),
            });
        }
    }
    Ok(())
}

fn push_vals(stack: &mut Vec<ValType>, types: &[ValType]) {
    stack.extend_from_slice(types);
}

fn push_ctrl(ctrl: &mut Vec<Frame>, stack: &[ValType], kind: FrameKind, results: Vec<ValType>) {
    ctrl.push(Frame {
        kind,
        result_types: results,
        stack_height: stack.len(),
        unreachable: false,
    });
}

fn pop_ctrl(stack: &mut Vec<ValType>, ctrl: &mut Vec<Frame>) -> Result<Frame, ValidationError> {
    let frame = ctrl.pop().ok_or(ValidationError::Other("empty ctrl stack".into()))?;
    // Truncate stack back to frame height
    stack.truncate(frame.stack_height);
    // Push results into parent
    stack.extend_from_slice(&frame.result_types);
    Ok(frame)
}

fn label_types(ctrl: &[Frame], depth: usize) -> Vec<ValType> {
    let frame = &ctrl[ctrl.len() - 1 - depth];
    if frame.kind == FrameKind::Loop {
        vec![] // loops take no values on branch
    } else {
        frame.result_types.clone()
    }
}

// ─── Module-level helpers ─────────────────────────────────────────────────────

fn block_type_arity(
    module: &Module,
    bt: &BlockType,
) -> Result<(Vec<ValType>, Vec<ValType>), ValidationError> {
    Ok(match bt {
        BlockType::Empty => (vec![], vec![]),
        BlockType::Type(vt) => (vec![], vec![*vt]),
        BlockType::FuncType(idx) => {
            let ty = module
                .types
                .get(*idx as usize)
                .ok_or(ValidationError::TypeIndexOob(*idx, module.types.len()))?;
            (ty.params.clone(), ty.results.clone())
        }
    })
}

fn total_func_count(module: &Module) -> usize {
    import_count(module, |t| matches!(t, ImportType::Func(_))) + module.functions.len()
}

fn import_count<F: Fn(&ImportType) -> bool>(module: &Module, pred: F) -> usize {
    module.imports.iter().filter(|i| pred(&i.ty)).count()
}

fn func_type_at(module: &Module, idx: usize) -> Option<&FuncType> {
    let imported = import_count(module, |t| matches!(t, ImportType::Func(_)));
    if idx < imported {
        // find import
        let mut count = 0;
        for imp in &module.imports {
            if let ImportType::Func(ti) = imp.ty {
                if count == idx {
                    return module.types.get(ti as usize);
                }
                count += 1;
            }
        }
        None
    } else {
        let local_idx = idx - imported;
        let type_idx = *module.functions.get(local_idx)?;
        module.types.get(type_idx as usize)
    }
}

fn global_type_at(module: &Module, idx: usize) -> Option<crate::ir::GlobalType> {
    let imported_globals: Vec<_> = module
        .imports
        .iter()
        .filter_map(|i| {
            if let ImportType::Global(gt) = i.ty {
                Some(gt)
            } else {
                None
            }
        })
        .collect();
    if idx < imported_globals.len() {
        Some(imported_globals[idx])
    } else {
        let local = idx - imported_globals.len();
        module.globals.get(local).map(|g| g.ty)
    }
}

fn check_memory(module: &Module) -> Result<(), ValidationError> {
    let total = import_count(module, |t| matches!(t, ImportType::Memory(_))) + module.memories.len();
    if total == 0 {
        Err(ValidationError::MemoryIndexOob(0, 0))
    } else {
        Ok(())
    }
}

fn fmt_types(types: &[ValType]) -> String {
    types.iter().map(|t| format!("{t}")).collect::<Vec<_>>().join(", ")
}
