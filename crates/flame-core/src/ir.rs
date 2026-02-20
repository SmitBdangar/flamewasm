//! Internal IR (Intermediate Representation) for a decoded WebAssembly module.
//!
//! After parsing, all sections are stored in this flat, owned structure which
//! is then passed to the validator and AOT compiler.

/// A fully decoded WebAssembly module.
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// All function signatures in the type section.
    pub types: Vec<FuncType>,
    /// Imports declared in the import section.
    pub imports: Vec<Import>,
    /// Index into `types` for each locally-defined function.
    pub functions: Vec<u32>,
    /// Table definitions.
    pub tables: Vec<TableType>,
    /// Memory definitions.
    pub memories: Vec<MemType>,
    /// Global variable definitions.
    pub globals: Vec<Global>,
    /// Exports.
    pub exports: Vec<Export>,
    /// Optional start function index.
    pub start: Option<u32>,
    /// Element segments (table initializers).
    pub elements: Vec<ElemSegment>,
    /// Function bodies (code section).
    pub code: Vec<FuncBody>,
    /// Data segments (memory initializers).
    pub data: Vec<DataSegment>,
    /// Custom sections preserved for tooling.
    pub custom: Vec<CustomSection>,
}

/// A WebAssembly function signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

impl std::fmt::Display for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) -> ({})", fmt_types(&self.params), fmt_types(&self.results))
    }
}

fn fmt_types(ts: &[ValType]) -> String {
    ts.iter().map(|t| format!("{t}")).collect::<Vec<_>>().join(", ")
}

/// WebAssembly value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::V128 => write!(f, "v128"),
            Self::FuncRef => write!(f, "funcref"),
            Self::ExternRef => write!(f, "externref"),
        }
    }
}

/// A module import.
#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub ty: ImportType,
}

/// The type of a module import.
#[derive(Debug, Clone)]
pub enum ImportType {
    Func(u32),   // index into types[]
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

/// A module export.
#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub kind: ExportKind,
    pub index: u32,
}

/// The kind of item being exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Func,
    Table,
    Memory,
    Global,
}

/// Table type descriptor.
#[derive(Debug, Clone, Copy)]
pub struct TableType {
    pub element_type: ValType,
    pub limits: Limits,
}

/// Memory type descriptor.
#[derive(Debug, Clone, Copy)]
pub struct MemType {
    pub limits: Limits,
    pub shared: bool,
}

/// Resource limits (min/max pages or elements).
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

/// A global variable definition.
#[derive(Debug, Clone)]
pub struct Global {
    pub ty: GlobalType,
    pub init: ConstExpr,
}

/// Global type: its value type and mutability.
#[derive(Debug, Clone, Copy)]
pub struct GlobalType {
    pub val_type: ValType,
    pub mutable: bool,
}

/// A constant initializer expression (used in globals, elements, data).
#[derive(Debug, Clone)]
pub enum ConstExpr {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    GlobalGet(u32),
    RefNull(ValType),
    RefFunc(u32),
}

/// An element segment (table initializer).
#[derive(Debug, Clone)]
pub struct ElemSegment {
    pub kind: ElemKind,
    pub items: Vec<ConstExpr>,
}

/// How the element segment is placed.
#[derive(Debug, Clone)]
pub enum ElemKind {
    /// Active: initialized at instantiation into the given table at the given offset.
    Active { table_index: u32, offset: ConstExpr },
    /// Passive: can be copied into a table via `table.init`.
    Passive,
    /// Declarative: used only for `ref.func` validation.
    Declarative,
}

/// A data segment (memory initializer).
#[derive(Debug, Clone)]
pub struct DataSegment {
    pub kind: DataKind,
    pub data: Vec<u8>,
}

/// How the data segment is placed.
#[derive(Debug, Clone)]
pub enum DataKind {
    /// Active: copied into memory at instantiation at the given byte offset.
    Active { memory_index: u32, offset: ConstExpr },
    /// Passive: must be copied with `memory.init`.
    Passive,
}

/// The decoded body (locals + instructions) of a locally-defined function.
#[derive(Debug, Clone)]
pub struct FuncBody {
    /// Local variable declarations (count, type).
    pub locals: Vec<(u32, ValType)>,
    /// The flat instruction sequence.
    pub instructions: Vec<Instruction>,
}

/// A custom section preserved from the binary.
#[derive(Debug, Clone)]
pub struct CustomSection {
    pub name: String,
    pub data: Vec<u8>,
}

/// The complete WebAssembly MVP instruction set plus common extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // ── Control ──────────────────────────────────────────────────────────────
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect { type_index: u32, table_index: u32 },

    // ── Reference ────────────────────────────────────────────────────────────
    RefNull(ValType),
    RefIsNull,
    RefFunc(u32),

    // ── Parametric ───────────────────────────────────────────────────────────
    Drop,
    Select,
    SelectTyped(Vec<ValType>),

    // ── Variable ─────────────────────────────────────────────────────────────
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // ── Table ─────────────────────────────────────────────────────────────────
    TableGet(u32),
    TableSet(u32),
    TableInit { elem_index: u32, table_index: u32 },
    ElemDrop(u32),
    TableCopy { dst: u32, src: u32 },
    TableGrow(u32),
    TableSize(u32),
    TableFill(u32),

    // ── Memory ────────────────────────────────────────────────────────────────
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize(u32),
    MemoryGrow(u32),
    MemoryInit { data_index: u32, mem_index: u32 },
    DataDrop(u32),
    MemoryCopy { dst: u32, src: u32 },
    MemoryFill(u32),

    // ── Numeric: i32 ──────────────────────────────────────────────────────────
    I32Const(i32),
    I32Eqz,
    I32Eq, I32Ne, I32LtS, I32LtU, I32GtS, I32GtU, I32LeS, I32LeU, I32GeS, I32GeU,
    I32Clz, I32Ctz, I32Popcnt,
    I32Add, I32Sub, I32Mul,
    I32DivS, I32DivU, I32RemS, I32RemU,
    I32And, I32Or, I32Xor,
    I32Shl, I32ShrS, I32ShrU, I32Rotl, I32Rotr,

    // ── Numeric: i64 ──────────────────────────────────────────────────────────
    I64Const(i64),
    I64Eqz,
    I64Eq, I64Ne, I64LtS, I64LtU, I64GtS, I64GtU, I64LeS, I64LeU, I64GeS, I64GeU,
    I64Clz, I64Ctz, I64Popcnt,
    I64Add, I64Sub, I64Mul,
    I64DivS, I64DivU, I64RemS, I64RemU,
    I64And, I64Or, I64Xor,
    I64Shl, I64ShrS, I64ShrU, I64Rotl, I64Rotr,

    // ── Numeric: f32 ──────────────────────────────────────────────────────────
    F32Const(f32),
    F32Eq, F32Ne, F32Lt, F32Gt, F32Le, F32Ge,
    F32Abs, F32Neg, F32Ceil, F32Floor, F32Trunc, F32Nearest, F32Sqrt,
    F32Add, F32Sub, F32Mul, F32Div, F32Min, F32Max, F32Copysign,

    // ── Numeric: f64 ──────────────────────────────────────────────────────────
    F64Const(f64),
    F64Eq, F64Ne, F64Lt, F64Gt, F64Le, F64Ge,
    F64Abs, F64Neg, F64Ceil, F64Floor, F64Trunc, F64Nearest, F64Sqrt,
    F64Add, F64Sub, F64Mul, F64Div, F64Min, F64Max, F64Copysign,

    // ── Conversions ────────────────────────────────────────────────────────────
    I32WrapI64,
    I32TruncF32S, I32TruncF32U, I32TruncF64S, I32TruncF64U,
    I64ExtendI32S, I64ExtendI32U,
    I64TruncF32S, I64TruncF32U, I64TruncF64S, I64TruncF64U,
    F32ConvertI32S, F32ConvertI32U, F32ConvertI64S, F32ConvertI64U, F32DemoteF64,
    F64ConvertI32S, F64ConvertI32U, F64ConvertI64S, F64ConvertI64U, F64PromoteF32,
    I32ReinterpretF32, I64ReinterpretF64, F32ReinterpretI32, F64ReinterpretI64,
    // Sign-extension (post-MVP)
    I32Extend8S, I32Extend16S, I64Extend8S, I64Extend16S, I64Extend32S,
    // Saturating truncation (post-MVP)
    I32TruncSatF32S, I32TruncSatF32U, I32TruncSatF64S, I32TruncSatF64U,
    I64TruncSatF32S, I64TruncSatF32U, I64TruncSatF64S, I64TruncSatF64U,
}

/// Memory immediate (alignment + byte offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemArg {
    pub align: u8,
    pub offset: u64,
    pub memory_index: u32,
}

/// Block type for structured control flow.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Empty,
    Type(ValType),
    FuncType(u32),
}
