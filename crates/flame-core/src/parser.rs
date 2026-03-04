//! WebAssembly binary parser powered by `wasmparser` 0.218.
//! Converts raw `.wasm` bytes into the owned [`Module`] IR.

use wasmparser::{ElementKind, Encoding, ExternalKind, Parser, Payload};

use crate::{
    error::ParseError,
    ir::{
        BlockType, ConstExpr, DataKind, DataSegment, ElemKind, ElemSegment, Export, ExportKind,
        FuncBody, FuncType, Global, GlobalType, Import, ImportType, Instruction, Limits, MemArg,
        MemType, Module, TableType, ValType,
    },
};

/// Parse a WebAssembly binary into a [`Module`].
pub fn parse(bytes: &[u8]) -> Result<Module, ParseError> {
    let mut module = Module::default();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::Version { num, encoding, .. } => {
                if encoding != Encoding::Module {
                    return Err(ParseError::Other("only module encoding supported".into()));
                }
                if num != 1 {
                    return Err(ParseError::UnsupportedVersion(num.into()));
                }
            }
            Payload::TypeSection(reader) => {
                for ty in reader {
                    let rec_group = ty?;
                    for sub_ty in rec_group.types() {
                        if let wasmparser::CompositeInnerType::Func(ft) = &sub_ty.composite_type.inner {
                            module.types.push(convert_func_type(ft));
                        }
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    let ty = match import.ty {
                        wasmparser::TypeRef::Func(i) => ImportType::Func(i),
                        wasmparser::TypeRef::Table(t) => ImportType::Table(convert_table_type(&t)),
                        wasmparser::TypeRef::Memory(m) => ImportType::Memory(convert_mem_type(&m)),
                        wasmparser::TypeRef::Global(g) => ImportType::Global(convert_global_type(&g)),
                        wasmparser::TypeRef::Tag(_) => {
                            return Err(ParseError::Other("exceptions not supported".into()))
                        }
                    };
                    module.imports.push(Import {
                        module: import.module.to_owned(),
                        name: import.name.to_owned(),
                        ty,
                    });
                }
            }
            Payload::FunctionSection(reader) => {
                for idx in reader { module.functions.push(idx?); }
            }
            Payload::TableSection(reader) => {
                for table in reader {
                    let t = table?;
                    module.tables.push(convert_table_type(&t.ty));
                }
            }
            Payload::MemorySection(reader) => {
                for mem in reader { module.memories.push(convert_mem_type(&mem?)); }
            }
            Payload::GlobalSection(reader) => {
                for global in reader {
                    let g = global?;
                    module.globals.push(Global {
                        ty: convert_global_type(&g.ty),
                        init: convert_const_expr(&g.init_expr)?,
                    });
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let e = export?;
                    let kind = match e.kind {
                        ExternalKind::Func => ExportKind::Func,
                        ExternalKind::Table => ExportKind::Table,
                        ExternalKind::Memory => ExportKind::Memory,
                        ExternalKind::Global => ExportKind::Global,
                        ExternalKind::Tag => {
                            return Err(ParseError::Other("tag exports unsupported".into()))
                        }
                    };
                    module.exports.push(Export { name: e.name.to_owned(), kind, index: e.index });
                }
            }
            Payload::StartSection { func, .. } => { module.start = Some(func); }
            Payload::ElementSection(reader) => {
                for elem in reader {
                    let elem = elem?;
                    let kind = match elem.kind {
                        ElementKind::Passive => ElemKind::Passive,
                        ElementKind::Declared => ElemKind::Declarative,
                        ElementKind::Active { table_index, offset_expr } => ElemKind::Active {
                            table_index: table_index.unwrap_or(0),
                            offset: convert_const_expr(&offset_expr)?,
                        },
                    };
                    let mut items = Vec::new();
                    match elem.items {
                        wasmparser::ElementItems::Functions(reader) => {
                            for fi in reader {
                                items.push(ConstExpr::RefFunc(fi?));
                            }
                        }
                        wasmparser::ElementItems::Expressions(_, reader) => {
                            for expr in reader {
                                items.push(convert_const_expr(&expr?)?);
                            }
                        }
                    }
                    module.elements.push(ElemSegment { kind, items });
                }
            }
            Payload::DataSection(reader) => {
                for data in reader {
                    let d = data?;
                    let kind = match d.kind {
                        wasmparser::DataKind::Passive => DataKind::Passive,
                        wasmparser::DataKind::Active { memory_index, offset_expr } => {
                            DataKind::Active {
                                memory_index,
                                offset: convert_const_expr(&offset_expr)?,
                            }
                        }
                    };
                    module.data.push(DataSegment { kind, data: d.data.to_vec() });
                }
            }
            Payload::CodeSectionStart { .. } => {}
            Payload::CodeSectionEntry(body) => {
                module.code.push(parse_func_body(body)?);
            }
            Payload::CustomSection(reader) => {
                module.custom.push(crate::ir::CustomSection {
                    name: reader.name().to_owned(),
                    data: reader.data().to_vec(),
                });
            }
            Payload::End(_) => break,
            _ => {}
        }
    }
    Ok(module)
}

fn convert_val_type(vt: wasmparser::ValType) -> Result<ValType, ParseError> {
    Ok(match vt {
        wasmparser::ValType::I32 => ValType::I32,
        wasmparser::ValType::I64 => ValType::I64,
        wasmparser::ValType::F32 => ValType::F32,
        wasmparser::ValType::F64 => ValType::F64,
        wasmparser::ValType::V128 => ValType::V128,
        wasmparser::ValType::Ref(r) => {
            if r.is_func_ref() { ValType::FuncRef } else { ValType::ExternRef }
        }
    })
}

fn convert_func_type(ft: &wasmparser::FuncType) -> FuncType {
    FuncType {
        params: ft.params().iter().filter_map(|v| convert_val_type(*v).ok()).collect(),
        results: ft.results().iter().filter_map(|v| convert_val_type(*v).ok()).collect(),
    }
}

fn convert_table_type(t: &wasmparser::TableType) -> TableType {
    TableType {
        element_type: if t.element_type.is_func_ref() { ValType::FuncRef } else { ValType::ExternRef },
        limits: Limits { min: t.initial as u32, max: t.maximum.map(|v| v as u32) },
    }
}

fn convert_mem_type(m: &wasmparser::MemoryType) -> MemType {
    MemType {
        limits: Limits { min: m.initial as u32, max: m.maximum.map(|v| v as u32) },
        shared: m.shared,
    }
}

fn convert_global_type(g: &wasmparser::GlobalType) -> GlobalType {
    GlobalType {
        val_type: convert_val_type(g.content_type).unwrap_or(ValType::I32),
        mutable: g.mutable,
    }
}

fn convert_const_expr(expr: &wasmparser::ConstExpr) -> Result<ConstExpr, ParseError> {
    let mut reader = expr.get_binary_reader();
    Ok(match reader.read_operator()? {
        wasmparser::Operator::I32Const { value } => ConstExpr::I32Const(value),
        wasmparser::Operator::I64Const { value } => ConstExpr::I64Const(value),
        wasmparser::Operator::F32Const { value } => ConstExpr::F32Const(f32::from_bits(value.bits())),
        wasmparser::Operator::F64Const { value } => ConstExpr::F64Const(f64::from_bits(value.bits())),
        wasmparser::Operator::GlobalGet { global_index } => ConstExpr::GlobalGet(global_index),
        wasmparser::Operator::RefNull { hty } => {
            let vt = match hty {
                wasmparser::HeapType::Abstract { ty: wasmparser::AbstractHeapType::Func, .. } => ValType::FuncRef,
                _ => ValType::ExternRef,
            };
            ConstExpr::RefNull(vt)
        }
        wasmparser::Operator::RefFunc { function_index } => ConstExpr::RefFunc(function_index),
        _ => ConstExpr::I32Const(0), // unknown const expr → zero
    })
}

fn convert_memarg(m: wasmparser::MemArg) -> MemArg {
    MemArg { align: m.align, offset: m.offset, memory_index: m.memory }
}

fn convert_block_type(bt: wasmparser::BlockType) -> Result<BlockType, ParseError> {
    Ok(match bt {
        wasmparser::BlockType::Empty => BlockType::Empty,
        wasmparser::BlockType::Type(vt) => BlockType::Type(convert_val_type(vt)?),
        wasmparser::BlockType::FuncType(idx) => BlockType::FuncType(idx),
    })
}

fn parse_func_body(body: wasmparser::FunctionBody) -> Result<FuncBody, ParseError> {
    let mut locals = Vec::new();
    for local in body.get_locals_reader()? {
        let (count, ty) = local?;
        locals.push((count, convert_val_type(ty)?));
    }
    let mut instructions = Vec::new();
    let ops_reader = body.get_operators_reader()?;
    for op_result in ops_reader {
        let op = op_result?;
        if let Some(instr) = convert_op(op)? {
            instructions.push(instr);
        }
    }
    Ok(FuncBody { locals, instructions })
}

#[allow(clippy::too_many_lines)]
fn convert_op(op: wasmparser::Operator) -> Result<Option<Instruction>, ParseError> {
    use wasmparser::Operator as O;
    Ok(Some(match op {
        O::Unreachable => Instruction::Unreachable,
        O::Nop => Instruction::Nop,
        O::Block { blockty } => Instruction::Block(convert_block_type(blockty)?),
        O::Loop { blockty } => Instruction::Loop(convert_block_type(blockty)?),
        O::If { blockty } => Instruction::If(convert_block_type(blockty)?),
        O::Else => Instruction::Else,
        O::End => Instruction::End,
        O::Br { relative_depth } => Instruction::Br(relative_depth),
        O::BrIf { relative_depth } => Instruction::BrIf(relative_depth),
        O::BrTable { targets } => {
            let default = targets.default();
            let table: Result<Vec<_>, _> = targets.targets().collect();
            Instruction::BrTable(table.map_err(ParseError::WasmParser)?, default)
        }
        O::Return => Instruction::Return,
        O::Call { function_index } => Instruction::Call(function_index),
        O::CallIndirect { type_index, table_index } => {
            Instruction::CallIndirect { type_index, table_index }
        }
        O::Drop => Instruction::Drop,
        O::Select => Instruction::Select,
        O::LocalGet { local_index } => Instruction::LocalGet(local_index),
        O::LocalSet { local_index } => Instruction::LocalSet(local_index),
        O::LocalTee { local_index } => Instruction::LocalTee(local_index),
        O::GlobalGet { global_index } => Instruction::GlobalGet(global_index),
        O::GlobalSet { global_index } => Instruction::GlobalSet(global_index),
        O::I32Load { memarg } => Instruction::I32Load(convert_memarg(memarg)),
        O::I64Load { memarg } => Instruction::I64Load(convert_memarg(memarg)),
        O::F32Load { memarg } => Instruction::F32Load(convert_memarg(memarg)),
        O::F64Load { memarg } => Instruction::F64Load(convert_memarg(memarg)),
        O::I32Load8S { memarg } => Instruction::I32Load8S(convert_memarg(memarg)),
        O::I32Load8U { memarg } => Instruction::I32Load8U(convert_memarg(memarg)),
        O::I32Load16S { memarg } => Instruction::I32Load16S(convert_memarg(memarg)),
        O::I32Load16U { memarg } => Instruction::I32Load16U(convert_memarg(memarg)),
        O::I64Load8S { memarg } => Instruction::I64Load8S(convert_memarg(memarg)),
        O::I64Load8U { memarg } => Instruction::I64Load8U(convert_memarg(memarg)),
        O::I64Load16S { memarg } => Instruction::I64Load16S(convert_memarg(memarg)),
        O::I64Load16U { memarg } => Instruction::I64Load16U(convert_memarg(memarg)),
        O::I64Load32S { memarg } => Instruction::I64Load32S(convert_memarg(memarg)),
        O::I64Load32U { memarg } => Instruction::I64Load32U(convert_memarg(memarg)),
        O::I32Store { memarg } => Instruction::I32Store(convert_memarg(memarg)),
        O::I64Store { memarg } => Instruction::I64Store(convert_memarg(memarg)),
        O::F32Store { memarg } => Instruction::F32Store(convert_memarg(memarg)),
        O::F64Store { memarg } => Instruction::F64Store(convert_memarg(memarg)),
        O::I32Store8 { memarg } => Instruction::I32Store8(convert_memarg(memarg)),
        O::I32Store16 { memarg } => Instruction::I32Store16(convert_memarg(memarg)),
        O::I64Store8 { memarg } => Instruction::I64Store8(convert_memarg(memarg)),
        O::I64Store16 { memarg } => Instruction::I64Store16(convert_memarg(memarg)),
        O::I64Store32 { memarg } => Instruction::I64Store32(convert_memarg(memarg)),
        O::MemorySize { mem, .. } => Instruction::MemorySize(mem),
        O::MemoryGrow { mem, .. } => Instruction::MemoryGrow(mem),
        O::I32Const { value } => Instruction::I32Const(value),
        O::I64Const { value } => Instruction::I64Const(value),
        O::F32Const { value } => Instruction::F32Const(f32::from_bits(value.bits())),
        O::F64Const { value } => Instruction::F64Const(f64::from_bits(value.bits())),
        O::I32Eqz => Instruction::I32Eqz,
        O::I32Eq => Instruction::I32Eq, O::I32Ne => Instruction::I32Ne,
        O::I32LtS => Instruction::I32LtS, O::I32LtU => Instruction::I32LtU,
        O::I32GtS => Instruction::I32GtS, O::I32GtU => Instruction::I32GtU,
        O::I32LeS => Instruction::I32LeS, O::I32LeU => Instruction::I32LeU,
        O::I32GeS => Instruction::I32GeS, O::I32GeU => Instruction::I32GeU,
        O::I64Eqz => Instruction::I64Eqz,
        O::I64Eq => Instruction::I64Eq, O::I64Ne => Instruction::I64Ne,
        O::I64LtS => Instruction::I64LtS, O::I64LtU => Instruction::I64LtU,
        O::I64GtS => Instruction::I64GtS, O::I64GtU => Instruction::I64GtU,
        O::I64LeS => Instruction::I64LeS, O::I64LeU => Instruction::I64LeU,
        O::I64GeS => Instruction::I64GeS, O::I64GeU => Instruction::I64GeU,
        O::F32Eq => Instruction::F32Eq, O::F32Ne => Instruction::F32Ne,
        O::F32Lt => Instruction::F32Lt, O::F32Gt => Instruction::F32Gt,
        O::F32Le => Instruction::F32Le, O::F32Ge => Instruction::F32Ge,
        O::F64Eq => Instruction::F64Eq, O::F64Ne => Instruction::F64Ne,
        O::F64Lt => Instruction::F64Lt, O::F64Gt => Instruction::F64Gt,
        O::F64Le => Instruction::F64Le, O::F64Ge => Instruction::F64Ge,
        O::I32Clz => Instruction::I32Clz, O::I32Ctz => Instruction::I32Ctz,
        O::I32Popcnt => Instruction::I32Popcnt,
        O::I32Add => Instruction::I32Add, O::I32Sub => Instruction::I32Sub,
        O::I32Mul => Instruction::I32Mul, O::I32DivS => Instruction::I32DivS,
        O::I32DivU => Instruction::I32DivU, O::I32RemS => Instruction::I32RemS,
        O::I32RemU => Instruction::I32RemU, O::I32And => Instruction::I32And,
        O::I32Or => Instruction::I32Or, O::I32Xor => Instruction::I32Xor,
        O::I32Shl => Instruction::I32Shl, O::I32ShrS => Instruction::I32ShrS,
        O::I32ShrU => Instruction::I32ShrU, O::I32Rotl => Instruction::I32Rotl,
        O::I32Rotr => Instruction::I32Rotr,
        O::I64Clz => Instruction::I64Clz, O::I64Ctz => Instruction::I64Ctz,
        O::I64Popcnt => Instruction::I64Popcnt,
        O::I64Add => Instruction::I64Add, O::I64Sub => Instruction::I64Sub,
        O::I64Mul => Instruction::I64Mul, O::I64DivS => Instruction::I64DivS,
        O::I64DivU => Instruction::I64DivU, O::I64RemS => Instruction::I64RemS,
        O::I64RemU => Instruction::I64RemU, O::I64And => Instruction::I64And,
        O::I64Or => Instruction::I64Or, O::I64Xor => Instruction::I64Xor,
        O::I64Shl => Instruction::I64Shl, O::I64ShrS => Instruction::I64ShrS,
        O::I64ShrU => Instruction::I64ShrU, O::I64Rotl => Instruction::I64Rotl,
        O::I64Rotr => Instruction::I64Rotr,
        O::F32Abs => Instruction::F32Abs, O::F32Neg => Instruction::F32Neg,
        O::F32Ceil => Instruction::F32Ceil, O::F32Floor => Instruction::F32Floor,
        O::F32Trunc => Instruction::F32Trunc, O::F32Nearest => Instruction::F32Nearest,
        O::F32Sqrt => Instruction::F32Sqrt,
        O::F32Add => Instruction::F32Add, O::F32Sub => Instruction::F32Sub,
        O::F32Mul => Instruction::F32Mul, O::F32Div => Instruction::F32Div,
        O::F32Min => Instruction::F32Min, O::F32Max => Instruction::F32Max,
        O::F32Copysign => Instruction::F32Copysign,
        O::F64Abs => Instruction::F64Abs, O::F64Neg => Instruction::F64Neg,
        O::F64Ceil => Instruction::F64Ceil, O::F64Floor => Instruction::F64Floor,
        O::F64Trunc => Instruction::F64Trunc, O::F64Nearest => Instruction::F64Nearest,
        O::F64Sqrt => Instruction::F64Sqrt,
        O::F64Add => Instruction::F64Add, O::F64Sub => Instruction::F64Sub,
        O::F64Mul => Instruction::F64Mul, O::F64Div => Instruction::F64Div,
        O::F64Min => Instruction::F64Min, O::F64Max => Instruction::F64Max,
        O::F64Copysign => Instruction::F64Copysign,
        O::I32WrapI64 => Instruction::I32WrapI64,
        O::I32TruncF32S => Instruction::I32TruncF32S, O::I32TruncF32U => Instruction::I32TruncF32U,
        O::I32TruncF64S => Instruction::I32TruncF64S, O::I32TruncF64U => Instruction::I32TruncF64U,
        O::I64ExtendI32S => Instruction::I64ExtendI32S, O::I64ExtendI32U => Instruction::I64ExtendI32U,
        O::I64TruncF32S => Instruction::I64TruncF32S, O::I64TruncF32U => Instruction::I64TruncF32U,
        O::I64TruncF64S => Instruction::I64TruncF64S, O::I64TruncF64U => Instruction::I64TruncF64U,
        O::F32ConvertI32S => Instruction::F32ConvertI32S, O::F32ConvertI32U => Instruction::F32ConvertI32U,
        O::F32ConvertI64S => Instruction::F32ConvertI64S, O::F32ConvertI64U => Instruction::F32ConvertI64U,
        O::F32DemoteF64 => Instruction::F32DemoteF64,
        O::F64ConvertI32S => Instruction::F64ConvertI32S, O::F64ConvertI32U => Instruction::F64ConvertI32U,
        O::F64ConvertI64S => Instruction::F64ConvertI64S, O::F64ConvertI64U => Instruction::F64ConvertI64U,
        O::F64PromoteF32 => Instruction::F64PromoteF32,
        O::I32ReinterpretF32 => Instruction::I32ReinterpretF32,
        O::I64ReinterpretF64 => Instruction::I64ReinterpretF64,
        O::F32ReinterpretI32 => Instruction::F32ReinterpretI32,
        O::F64ReinterpretI64 => Instruction::F64ReinterpretI64,
        O::I32Extend8S => Instruction::I32Extend8S, O::I32Extend16S => Instruction::I32Extend16S,
        O::I64Extend8S => Instruction::I64Extend8S, O::I64Extend16S => Instruction::I64Extend16S,
        O::I64Extend32S => Instruction::I64Extend32S,
        O::I32TruncSatF32S => Instruction::I32TruncSatF32S,
        O::I32TruncSatF32U => Instruction::I32TruncSatF32U,
        O::I32TruncSatF64S => Instruction::I32TruncSatF64S,
        O::I32TruncSatF64U => Instruction::I32TruncSatF64U,
        O::I64TruncSatF32S => Instruction::I64TruncSatF32S,
        O::I64TruncSatF32U => Instruction::I64TruncSatF32U,
        O::I64TruncSatF64S => Instruction::I64TruncSatF64S,
        O::I64TruncSatF64U => Instruction::I64TruncSatF64U,
        O::MemoryInit { data_index, mem } => Instruction::MemoryInit { data_index, mem_index: mem },
        O::DataDrop { data_index } => Instruction::DataDrop(data_index),
        O::MemoryCopy { dst_mem, src_mem } => Instruction::MemoryCopy { dst: dst_mem, src: src_mem },
        O::MemoryFill { mem } => Instruction::MemoryFill(mem),
        O::TableInit { elem_index, table } => Instruction::TableInit { elem_index, table_index: table },
        O::ElemDrop { elem_index } => Instruction::ElemDrop(elem_index),
        O::TableCopy { dst_table, src_table } => Instruction::TableCopy { dst: dst_table, src: src_table },
        O::TableGrow { table } => Instruction::TableGrow(table),
        O::TableSize { table } => Instruction::TableSize(table),
        O::TableFill { table } => Instruction::TableFill(table),
        O::TableGet { table } => Instruction::TableGet(table),
        O::TableSet { table } => Instruction::TableSet(table),
        O::RefNull { hty } => {
            let vt = match hty {
                wasmparser::HeapType::Abstract { ty: wasmparser::AbstractHeapType::Func, .. } => ValType::FuncRef,
                _ => ValType::ExternRef,
            };
            Instruction::RefNull(vt)
        }
        O::RefIsNull => Instruction::RefIsNull,
        O::RefFunc { function_index } => Instruction::RefFunc(function_index),
        _ => return Ok(None),
    }))
}
