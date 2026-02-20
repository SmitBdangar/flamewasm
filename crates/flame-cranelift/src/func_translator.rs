//! Wasm → Cranelift IR function translator.

use anyhow::{bail, Result};
use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, MemFlags, Signature, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::JITModule;
use cranelift_module::Module as CraneliftModule;
use flame_core::ir::{BlockType, FuncBody, FuncType, Instruction, Module, ValType};

use crate::memory::MemoryTranslator;

/// Translate a single Wasm function into Cranelift IR.
pub struct FuncTranslator<'a> {
    pub wasm_module: &'a Module,
    pub func_type: &'a FuncType,
    pub body: &'a FuncBody,
    pub func_ids: &'a [cranelift_module::FuncId],
    pub imported_func_count: usize,
    pub func: &'a mut Function,
    pub jit_module: &'a mut JITModule,
}

impl<'a> FuncTranslator<'a> {
    pub fn new(
        wasm_module: &'a Module,
        func_type: &'a FuncType,
        body: &'a FuncBody,
        func_ids: &'a [cranelift_module::FuncId],
        imported_func_count: usize,
        func: &'a mut Function,
        jit_module: &'a mut JITModule,
    ) -> Result<Self> {
        Ok(Self { wasm_module, func_type, body, func_ids, imported_func_count, func, jit_module })
    }

    pub fn translate(&mut self) -> Result<()> {
        let mut fbctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(self.func, &mut fbctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut var_idx = 0u32;
        let mut local_vars: Vec<Variable> = Vec::new();

        for (i, &param_type) in self.func_type.params.iter().enumerate() {
            let var = Variable::from_u32(var_idx);
            var_idx += 1;
            builder.declare_var(var, val_type_to_cl(param_type));
            let param_val = builder.block_params(entry_block)[i];
            builder.def_var(var, param_val);
            local_vars.push(var);
        }

        for (count, vt) in &self.body.locals {
            for _ in 0..*count {
                let var = Variable::from_u32(var_idx);
                var_idx += 1;
                let cl_ty = val_type_to_cl(*vt);
                builder.declare_var(var, cl_ty);
                let zero = zero_val(&mut builder, *vt);
                builder.def_var(var, zero);
                local_vars.push(var);
            }
        }

        let mut value_stack: Vec<Value> = Vec::new();
        let mut ctrl_stack: Vec<CtrlFrame> = Vec::new();
        let mut terminated = false;

        for instr in &self.body.instructions {
            translate_instr(
                instr,
                &mut builder,
                self.wasm_module,
                self.func_type,
                self.func_ids,
                &local_vars,
                &mut value_stack,
                &mut ctrl_stack,
                &mut terminated,
                self.jit_module,
            )?;
        }

        if !terminated {
            let n = self.func_type.results.len();
            let mut results: Vec<Value> = (0..n)
                .map(|_| value_stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0)))
                .collect();
            results.reverse();
            builder.ins().return_(&results);
        }

        builder.finalize();
        Ok(())
    }
}

// ─── Control frame ────────────────────────────────────────────────────────────

#[derive(Debug)]
struct CtrlFrame {
    kind: CtrlKind,
    after_block: cranelift_codegen::ir::Block,
    else_block: Option<cranelift_codegen::ir::Block>,
    result_types: Vec<ValType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CtrlKind { Block, Loop, If }

// ─── Free-function instruction translator ────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn translate_instr(
    instr: &Instruction,
    builder: &mut FunctionBuilder,
    wasm_module: &Module,
    func_type: &FuncType,
    func_ids: &[cranelift_module::FuncId],
    locals: &[Variable],
    stack: &mut Vec<Value>,
    ctrl: &mut Vec<CtrlFrame>,
    terminated: &mut bool,
    jit_module: &mut JITModule,
) -> Result<()> {
    use cranelift_codegen::ir::condcodes::IntCC;
    match instr {
        Instruction::Unreachable => {
            builder.ins().trap(cranelift_codegen::ir::TrapCode::user(0).unwrap());
            *terminated = true;
        }
        Instruction::Nop => {}
        Instruction::Block(bt) => {
            let result_types = block_result_types(wasm_module, bt);
            let after_block = builder.create_block();
            for vt in &result_types {
                builder.append_block_param(after_block, val_type_to_cl(*vt));
            }
            ctrl.push(CtrlFrame { kind: CtrlKind::Block, after_block, else_block: None, result_types });
        }
        Instruction::Loop(bt) => {
            let result_types = block_result_types(wasm_module, bt);
            let loop_block = builder.create_block();
            if !*terminated { builder.ins().jump(loop_block, &[]); }
            builder.seal_block(loop_block);
            builder.switch_to_block(loop_block);
            *terminated = false;
            ctrl.push(CtrlFrame { kind: CtrlKind::Loop, after_block: loop_block, else_block: None, result_types });
        }
        Instruction::If(_bt) => {
            let cond = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
            let cond_b = builder.ins().icmp_imm(IntCC::NotEqual, cond, 0);
            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let after_block = builder.create_block();
            builder.ins().brif(cond_b, then_block, &[], else_block, &[]);
            builder.seal_block(then_block);
            builder.switch_to_block(then_block);
            *terminated = false;
            ctrl.push(CtrlFrame { kind: CtrlKind::If, after_block, else_block: Some(else_block), result_types: vec![] });
        }
        Instruction::Else => {
            if let Some(frame) = ctrl.last() {
                let after_block = frame.after_block;
                let else_block = frame.else_block;
                if !*terminated { builder.ins().jump(after_block, &[]); }
                *terminated = false;
                if let Some(eb) = else_block {
                    builder.seal_block(eb);
                    builder.switch_to_block(eb);
                }
            }
        }
        Instruction::End => {
            if let Some(frame) = ctrl.pop() {
                if !*terminated { builder.ins().jump(frame.after_block, &[]); }
                *terminated = false;
                // If there was an else block that was never switched to (no Else instr), fill it
                if let Some(eb) = frame.else_block {
                    // Check if else_block has any predecessors; if not, seal + fill
                    builder.switch_to_block(eb);
                    builder.seal_block(eb);
                    builder.ins().jump(frame.after_block, &[]);
                }
                builder.seal_block(frame.after_block);
                builder.switch_to_block(frame.after_block);
                *terminated = false;
                for p in builder.block_params(frame.after_block).to_vec() {
                    stack.push(p);
                }
            }
        }
        Instruction::Br(depth) => {
            let idx = ctrl.len().saturating_sub(1 + *depth as usize);
            if let Some(frame) = ctrl.get(idx) {
                builder.ins().jump(frame.after_block, &[]);
            }
            *terminated = true;
        }
        Instruction::BrIf(depth) => {
            let cond = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
            let cond_b = builder.ins().icmp_imm(IntCC::NotEqual, cond, 0);
            let idx = ctrl.len().saturating_sub(1 + *depth as usize);
            let fallthrough = builder.create_block();
            if let Some(frame) = ctrl.get(idx) {
                builder.ins().brif(cond_b, frame.after_block, &[], fallthrough, &[]);
            } else {
                builder.ins().jump(fallthrough, &[]);
            }
            builder.seal_block(fallthrough);
            builder.switch_to_block(fallthrough);
            *terminated = false;
        }
        Instruction::Return => {
            let n = func_type.results.len();
            let mut results: Vec<Value> = (0..n)
                .map(|_| stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0)))
                .collect();
            results.reverse();
            builder.ins().return_(&results);
            *terminated = true;
        }
        Instruction::Call(fi) => {
            let fi_usize = *fi as usize;
            if fi_usize >= func_ids.len() { bail!("call: function index {fi} OOB"); }
            let fid = func_ids[fi_usize];
            let callee = jit_module.declare_func_in_func(fid, builder.func);
            let n_params = func_param_count(wasm_module, fi_usize);
            let mut args: Vec<Value> = (0..n_params)
                .map(|_| stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0)))
                .collect();
            args.reverse();
            let call = builder.ins().call(callee, &args);
            for r in builder.inst_results(call).to_vec() { stack.push(r); }
        }
        Instruction::LocalGet(idx) => {
            if let Some(var) = locals.get(*idx as usize) {
                stack.push(builder.use_var(*var));
            }
        }
        Instruction::LocalSet(idx) => {
            if let Some(var) = locals.get(*idx as usize) {
                let v = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
                builder.def_var(*var, v);
            }
        }
        Instruction::LocalTee(idx) => {
            if let Some(var) = locals.get(*idx as usize) {
                let v = stack.last().copied().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
                builder.def_var(*var, v);
            }
        }
        // ── i32 ─────────────────────────────────────────────────────────────
        Instruction::I32Const(v) => { stack.push(builder.ins().iconst(types::I32, *v as i64)); }
        Instruction::I32Add => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().iadd(a,b)); }
        Instruction::I32Sub => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().isub(a,b)); }
        Instruction::I32Mul => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().imul(a,b)); }
        Instruction::I32DivS => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().sdiv(a,b)); }
        Instruction::I32DivU => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().udiv(a,b)); }
        Instruction::I32RemS => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().srem(a,b)); }
        Instruction::I32RemU => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().urem(a,b)); }
        Instruction::I32And => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().band(a,b)); }
        Instruction::I32Or  => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().bor(a,b)); }
        Instruction::I32Xor => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().bxor(a,b)); }
        Instruction::I32Shl => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().ishl(a,b)); }
        Instruction::I32ShrS=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().sshr(a,b)); }
        Instruction::I32ShrU=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().ushr(a,b)); }
        Instruction::I32Rotl=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().rotl(a,b)); }
        Instruction::I32Rotr=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().rotr(a,b)); }
        Instruction::I32Clz => { let a=pop1(stack,builder); stack.push(builder.ins().clz(a)); }
        Instruction::I32Ctz => { let a=pop1(stack,builder); stack.push(builder.ins().ctz(a)); }
        Instruction::I32Popcnt => { let a=pop1(stack,builder); stack.push(builder.ins().popcnt(a)); }
        Instruction::I32Eqz => {
            let a = pop1(stack,builder);
            let r = builder.ins().icmp_imm(IntCC::Equal, a, 0);
            stack.push(builder.ins().uextend(types::I32, r));
        }
        Instruction::I32Eq  => { icmp_push(stack,builder,IntCC::Equal); }
        Instruction::I32Ne  => { icmp_push(stack,builder,IntCC::NotEqual); }
        Instruction::I32LtS => { icmp_push(stack,builder,IntCC::SignedLessThan); }
        Instruction::I32LtU => { icmp_push(stack,builder,IntCC::UnsignedLessThan); }
        Instruction::I32GtS => { icmp_push(stack,builder,IntCC::SignedGreaterThan); }
        Instruction::I32GtU => { icmp_push(stack,builder,IntCC::UnsignedGreaterThan); }
        Instruction::I32LeS => { icmp_push(stack,builder,IntCC::SignedLessThanOrEqual); }
        Instruction::I32LeU => { icmp_push(stack,builder,IntCC::UnsignedLessThanOrEqual); }
        Instruction::I32GeS => { icmp_push(stack,builder,IntCC::SignedGreaterThanOrEqual); }
        Instruction::I32GeU => { icmp_push(stack,builder,IntCC::UnsignedGreaterThanOrEqual); }
        // ── i64 ─────────────────────────────────────────────────────────────
        Instruction::I64Const(v) => { stack.push(builder.ins().iconst(types::I64, *v)); }
        Instruction::I64Add => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().iadd(a,b)); }
        Instruction::I64Sub => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().isub(a,b)); }
        Instruction::I64Mul => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().imul(a,b)); }
        Instruction::I64DivS => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().sdiv(a,b)); }
        Instruction::I64DivU => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().udiv(a,b)); }
        Instruction::I64And => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().band(a,b)); }
        Instruction::I64Or  => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().bor(a,b)); }
        Instruction::I64Xor => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().bxor(a,b)); }
        Instruction::I64Shl => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().ishl(a,b)); }
        Instruction::I64ShrS=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().sshr(a,b)); }
        Instruction::I64ShrU=> { let (a,b) = pop2(stack,builder); stack.push(builder.ins().ushr(a,b)); }
        Instruction::I64ExtendI32S => { let a=pop1(stack,builder); stack.push(builder.ins().sextend(types::I64,a)); }
        Instruction::I64ExtendI32U => { let a=pop1(stack,builder); stack.push(builder.ins().uextend(types::I64,a)); }
        Instruction::I32WrapI64 => { let a=pop1(stack,builder); stack.push(builder.ins().ireduce(types::I32,a)); }
        // ── f32/f64 ─────────────────────────────────────────────────────────
        Instruction::F32Const(v) => { stack.push(builder.ins().f32const(cranelift_codegen::ir::immediates::Ieee32::with_float(*v))); }
        Instruction::F64Const(v) => { stack.push(builder.ins().f64const(cranelift_codegen::ir::immediates::Ieee64::with_float(*v))); }
        Instruction::F32Add => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fadd(a,b)); }
        Instruction::F32Sub => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fsub(a,b)); }
        Instruction::F32Mul => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fmul(a,b)); }
        Instruction::F32Div => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fdiv(a,b)); }
        Instruction::F32Sqrt => { let a=pop1(stack,builder); stack.push(builder.ins().sqrt(a)); }
        Instruction::F32Neg  => { let a=pop1(stack,builder); stack.push(builder.ins().fneg(a)); }
        Instruction::F32Abs  => { let a=pop1(stack,builder); stack.push(builder.ins().fabs(a)); }
        Instruction::F64Add => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fadd(a,b)); }
        Instruction::F64Sub => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fsub(a,b)); }
        Instruction::F64Mul => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fmul(a,b)); }
        Instruction::F64Div => { let (a,b) = pop2(stack,builder); stack.push(builder.ins().fdiv(a,b)); }
        Instruction::F64Sqrt => { let a=pop1(stack,builder); stack.push(builder.ins().sqrt(a)); }
        Instruction::F64Neg  => { let a=pop1(stack,builder); stack.push(builder.ins().fneg(a)); }
        Instruction::F64Abs  => { let a=pop1(stack,builder); stack.push(builder.ins().fabs(a)); }
        Instruction::F64PromoteF32  => { let a=pop1(stack,builder); stack.push(builder.ins().fpromote(types::F64,a)); }
        Instruction::F32DemoteF64   => { let a=pop1(stack,builder); stack.push(builder.ins().fdemote(types::F32,a)); }
        Instruction::F32ConvertI32S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_sint(types::F32,a)); }
        Instruction::F32ConvertI32U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_uint(types::F32,a)); }
        Instruction::F32ConvertI64S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_sint(types::F32,a)); }
        Instruction::F32ConvertI64U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_uint(types::F32,a)); }
        Instruction::F64ConvertI32S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_sint(types::F64,a)); }
        Instruction::F64ConvertI32U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_uint(types::F64,a)); }
        Instruction::F64ConvertI64S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_sint(types::F64,a)); }
        Instruction::F64ConvertI64U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_from_uint(types::F64,a)); }
        Instruction::I32TruncF32S | Instruction::I32TruncSatF32S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_sint_sat(types::I32,a)); }
        Instruction::I32TruncF32U | Instruction::I32TruncSatF32U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_uint_sat(types::I32,a)); }
        Instruction::I32TruncF64S | Instruction::I32TruncSatF64S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_sint_sat(types::I32,a)); }
        Instruction::I32TruncF64U | Instruction::I32TruncSatF64U => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_uint_sat(types::I32,a)); }
        Instruction::I64TruncF32S | Instruction::I64TruncSatF32S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_sint_sat(types::I64,a)); }
        Instruction::I64TruncF64S | Instruction::I64TruncSatF64S => { let a=pop1(stack,builder); stack.push(builder.ins().fcvt_to_sint_sat(types::I64,a)); }
        Instruction::I32ReinterpretF32 => { let a=pop1(stack,builder); stack.push(builder.ins().bitcast(types::I32,MemFlags::new(),a)); }
        Instruction::F32ReinterpretI32 => { let a=pop1(stack,builder); stack.push(builder.ins().bitcast(types::F32,MemFlags::new(),a)); }
        Instruction::I64ReinterpretF64 => { let a=pop1(stack,builder); stack.push(builder.ins().bitcast(types::I64,MemFlags::new(),a)); }
        Instruction::F64ReinterpretI64 => { let a=pop1(stack,builder); stack.push(builder.ins().bitcast(types::F64,MemFlags::new(),a)); }
        // ── Parametric ───────────────────────────────────────────────────────
        Instruction::Drop => { stack.pop(); }
        Instruction::Select => {
            let cond = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
            let b = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
            let a = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
            let cond_b = builder.ins().icmp_imm(IntCC::NotEqual, cond, 0);
            stack.push(builder.ins().select(cond_b, a, b));
        }
        // ── Memory loads/stores ──────────────────────────────────────────────
        Instruction::I32Load(ma) => { MemoryTranslator::translate_load(builder,stack,types::I32,ma.offset,false,false); }
        Instruction::I64Load(ma) => { MemoryTranslator::translate_load(builder,stack,types::I64,ma.offset,false,false); }
        Instruction::F32Load(ma) => { MemoryTranslator::translate_load(builder,stack,types::F32,ma.offset,false,false); }
        Instruction::F64Load(ma) => { MemoryTranslator::translate_load(builder,stack,types::F64,ma.offset,false,false); }
        Instruction::I32Store(ma) => { MemoryTranslator::translate_store(builder,stack,types::I32,ma.offset); }
        Instruction::I64Store(ma) => { MemoryTranslator::translate_store(builder,stack,types::I64,ma.offset); }
        Instruction::F32Store(ma) => { MemoryTranslator::translate_store(builder,stack,types::F32,ma.offset); }
        Instruction::F64Store(ma) => { MemoryTranslator::translate_store(builder,stack,types::F64,ma.offset); }
        // Skip anything else
        _ => {}
    }
    Ok(())
}

// ─── Type helpers ─────────────────────────────────────────────────────────────

pub fn val_type_to_cl(vt: ValType) -> cranelift_codegen::ir::Type {
    match vt {
        ValType::I32 | ValType::FuncRef | ValType::ExternRef => types::I32,
        ValType::I64 => types::I64,
        ValType::F32 => types::F32,
        ValType::F64 => types::F64,
        ValType::V128 => types::I8X16,
    }
}

pub fn build_signature(ft: &FuncType, module: &JITModule) -> Result<Signature> {
    let mut sig = module.make_signature();
    for &p in &ft.params { sig.params.push(AbiParam::new(val_type_to_cl(p))); }
    for &r in &ft.results { sig.returns.push(AbiParam::new(val_type_to_cl(r))); }
    Ok(sig)
}

fn block_result_types(module: &Module, bt: &BlockType) -> Vec<ValType> {
    match bt {
        BlockType::Empty => vec![],
        BlockType::Type(vt) => vec![*vt],
        BlockType::FuncType(idx) => module.types.get(*idx as usize).map(|ft| ft.results.clone()).unwrap_or_default(),
    }
}

fn func_param_count(module: &Module, global_idx: usize) -> usize {
    use flame_core::ir::ImportType;
    let imported = module.imports.iter().filter(|i| matches!(i.ty, ImportType::Func(_))).count();
    if global_idx < imported {
        let mut c = 0;
        for imp in &module.imports {
            if let ImportType::Func(ti) = imp.ty {
                if c == global_idx { return module.types.get(ti as usize).map_or(0, |f| f.params.len()); }
                c += 1;
            }
        }
        0
    } else {
        let local = global_idx - imported;
        module.functions.get(local).and_then(|ti| module.types.get(*ti as usize)).map_or(0, |f| f.params.len())
    }
}

fn zero_val(builder: &mut FunctionBuilder, vt: ValType) -> Value {
    match vt {
        ValType::I32 | ValType::FuncRef | ValType::ExternRef => builder.ins().iconst(types::I32, 0),
        ValType::I64 => builder.ins().iconst(types::I64, 0),
        ValType::F32 => builder.ins().f32const(cranelift_codegen::ir::immediates::Ieee32::with_float(0.0)),
        ValType::F64 => builder.ins().f64const(cranelift_codegen::ir::immediates::Ieee64::with_float(0.0)),
        ValType::V128 => builder.ins().iconst(types::I64, 0),
    }
}

fn pop1(stack: &mut Vec<Value>, builder: &mut FunctionBuilder) -> Value {
    stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0))
}
fn pop2(stack: &mut Vec<Value>, builder: &mut FunctionBuilder) -> (Value, Value) {
    let b = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
    let a = stack.pop().unwrap_or_else(|| builder.ins().iconst(types::I32, 0));
    (a, b)
}
fn icmp_push(stack: &mut Vec<Value>, builder: &mut FunctionBuilder, cc: cranelift_codegen::ir::condcodes::IntCC) {
    let (a, b) = pop2(stack, builder);
    let r = builder.ins().icmp(cc, a, b);
    let r = builder.ins().uextend(types::I32, r);
    stack.push(r);
}
