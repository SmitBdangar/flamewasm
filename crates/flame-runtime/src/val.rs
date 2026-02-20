//! WebAssembly value type used at the runtime boundary.

use flame_core::ir::ValType;

/// A runtime WebAssembly value.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    FuncRef(Option<u32>),
    ExternRef(Option<u64>),
}

impl Val {
    /// Return the [`ValType`] of this value.
    #[must_use]
    pub fn ty(&self) -> ValType {
        match self {
            Self::I32(_) => ValType::I32,
            Self::I64(_) => ValType::I64,
            Self::F32(_) => ValType::F32,
            Self::F64(_) => ValType::F64,
            Self::FuncRef(_) => ValType::FuncRef,
            Self::ExternRef(_) => ValType::ExternRef,
        }
    }

    /// Unwrap as i32, panicking if wrong variant.
    #[must_use]
    pub fn unwrap_i32(&self) -> i32 {
        if let Self::I32(v) = self { *v } else { panic!("expected i32, got {self:?}") }
    }

    /// Unwrap as i64.
    #[must_use]
    pub fn unwrap_i64(&self) -> i64 {
        if let Self::I64(v) = self { *v } else { panic!("expected i64, got {self:?}") }
    }

    /// Unwrap as f32.
    #[must_use]
    pub fn unwrap_f32(&self) -> f32 {
        if let Self::F32(v) = self { *v } else { panic!("expected f32, got {self:?}") }
    }

    /// Unwrap as f64.
    #[must_use]
    pub fn unwrap_f64(&self) -> f64 {
        if let Self::F64(v) = self { *v } else { panic!("expected f64, got {self:?}") }
    }
}
