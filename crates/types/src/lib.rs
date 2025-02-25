#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_assignments, unused_variables))
))]
#![warn(missing_debug_implementations, rust_2018_idioms, unreachable_pub)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]
#![cfg_attr(feature = "unsafe", deny(unused_unsafe))]

//! Types used by [`tinywasm`](https://docs.rs/tinywasm) and [`tinywasm_parser`](https://docs.rs/tinywasm_parser).

extern crate alloc;
use alloc::boxed::Box;
use core::{fmt::Debug, ops::Range};

// log for logging (optional).
#[cfg(feature = "logging")]
#[allow(clippy::single_component_path_imports)]
use log;

#[cfg(not(feature = "logging"))]
#[macro_use]
pub(crate) mod log {
    macro_rules! error    ( ($($tt:tt)*) => {{}} );
    pub(crate) use error;
}

mod instructions;
mod value;
pub use instructions::*;
pub use value::*;

#[cfg(feature = "archive")]
pub mod archive;

/// A TinyWasm WebAssembly Module
///
/// This is the internal representation of a WebAssembly module in TinyWasm.
/// TinyWasmModules are validated before being created, so they are guaranteed to be valid (as long as they were created by TinyWasm).
/// This means you should not trust a TinyWasmModule created by a third party to be valid.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct TinyWasmModule {
    /// Optional address of the start function
    ///
    /// Corresponds to the `start` section of the original WebAssembly module.
    pub start_func: Option<FuncAddr>,

    /// Optimized and validated WebAssembly functions
    ///
    /// Contains data from to the `code`, `func`, and `type` sections of the original WebAssembly module.
    pub funcs: Box<[WasmFunction]>,

    /// A vector of type definitions, indexed by `TypeAddr`
    ///
    /// Corresponds to the `type` section of the original WebAssembly module.
    pub func_types: Box<[FuncType]>,

    /// Exported items of the WebAssembly module.
    ///
    /// Corresponds to the `export` section of the original WebAssembly module.
    pub exports: Box<[Export]>,

    /// Global components of the WebAssembly module.
    ///
    /// Corresponds to the `global` section of the original WebAssembly module.
    pub globals: Box<[Global]>,

    /// Table components of the WebAssembly module used to initialize tables.
    ///
    /// Corresponds to the `table` section of the original WebAssembly module.
    pub table_types: Box<[TableType]>,

    /// Memory components of the WebAssembly module used to initialize memories.
    ///
    /// Corresponds to the `memory` section of the original WebAssembly module.
    pub memory_types: Box<[MemoryType]>,

    /// Imports of the WebAssembly module.
    ///
    /// Corresponds to the `import` section of the original WebAssembly module.
    pub imports: Box<[Import]>,

    /// Data segments of the WebAssembly module.
    ///
    /// Corresponds to the `data` section of the original WebAssembly module.
    pub data: Box<[Data]>,

    /// Element segments of the WebAssembly module.
    ///
    /// Corresponds to the `elem` section of the original WebAssembly module.
    pub elements: Box<[Element]>,
}

/// A WebAssembly External Kind.
///
/// See <https://webassembly.github.io/spec/core/syntax/types.html#external-types>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum ExternalKind {
    /// A WebAssembly Function.
    Func,
    /// A WebAssembly Table.
    Table,
    /// A WebAssembly Memory.
    Memory,
    /// A WebAssembly Global.
    Global,
}

/// A WebAssembly Address.
///
/// These are indexes into the respective stores.
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#addresses>
pub type Addr = u32;

// aliases for clarity
pub type FuncAddr = Addr;
pub type TableAddr = Addr;
pub type MemAddr = Addr;
pub type GlobalAddr = Addr;
pub type ElemAddr = Addr;
pub type DataAddr = Addr;
pub type ExternAddr = Addr;

// additional internal addresses
pub type TypeAddr = Addr;
pub type LocalAddr = Addr;
pub type LabelAddr = Addr;
pub type ModuleInstanceAddr = Addr;

/// A WebAssembly External Value.
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Debug, Clone)]
pub enum ExternVal {
    Func(FuncAddr),
    Table(TableAddr),
    Memory(MemAddr),
    Global(GlobalAddr),
}

impl ExternVal {
    #[inline]
    pub fn kind(&self) -> ExternalKind {
        match self {
            Self::Func(_) => ExternalKind::Func,
            Self::Table(_) => ExternalKind::Table,
            Self::Memory(_) => ExternalKind::Memory,
            Self::Global(_) => ExternalKind::Global,
        }
    }

    #[inline]
    pub fn new(kind: ExternalKind, addr: Addr) -> Self {
        match kind {
            ExternalKind::Func => Self::Func(addr),
            ExternalKind::Table => Self::Table(addr),
            ExternalKind::Memory => Self::Memory(addr),
            ExternalKind::Global => Self::Global(addr),
        }
    }
}

/// The type of a WebAssembly Function.
///
/// See <https://webassembly.github.io/spec/core/syntax/types.html#function-types>
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct FuncType {
    pub params: Box<[ValType]>,
    pub results: Box<[ValType]>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct WasmFunction {
    pub instructions: Box<[Instruction]>,
    pub locals: Box<[ValType]>,
    pub ty: FuncType,
}

/// A WebAssembly Module Export
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct Export {
    /// The name of the export.
    pub name: Box<str>,
    /// The kind of the export.
    pub kind: ExternalKind,
    /// The index of the exported item.
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct Global {
    pub ty: GlobalType,
    pub init: ConstInstruction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct GlobalType {
    pub mutable: bool,
    pub ty: ValType,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct TableType {
    pub element_type: ValType,
    pub size_initial: u32,
    pub size_max: Option<u32>,
}

impl TableType {
    pub fn empty() -> Self {
        Self { element_type: ValType::RefFunc, size_initial: 0, size_max: None }
    }

    pub fn new(element_type: ValType, size_initial: u32, size_max: Option<u32>) -> Self {
        Self { element_type, size_initial, size_max }
    }
}

/// Represents a memory's type.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct MemoryType {
    pub arch: MemoryArch,
    pub page_count_initial: u64,
    pub page_count_max: Option<u64>,
}

impl MemoryType {
    pub fn new_32(page_count_initial: u64, page_count_max: Option<u64>) -> Self {
        Self { arch: MemoryArch::I32, page_count_initial, page_count_max }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum MemoryArch {
    I32,
    I64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct Import {
    pub module: Box<str>,
    pub name: Box<str>,
    pub kind: ImportKind,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum ImportKind {
    Function(TypeAddr),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

impl From<&ImportKind> for ExternalKind {
    #[inline]
    fn from(kind: &ImportKind) -> Self {
        match kind {
            ImportKind::Function(_) => Self::Func,
            ImportKind::Table(_) => Self::Table,
            ImportKind::Memory(_) => Self::Memory,
            ImportKind::Global(_) => Self::Global,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct Data {
    pub data: Box<[u8]>,
    pub range: Range<usize>,
    pub kind: DataKind,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum DataKind {
    Active { mem: MemAddr, offset: ConstInstruction },
    Passive,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub struct Element {
    pub kind: ElementKind,
    pub items: Box<[ElementItem]>,
    pub range: Range<usize>,
    pub ty: ValType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum ElementKind {
    Passive,
    Active { table: TableAddr, offset: ConstInstruction },
    Declared,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "archive", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
pub enum ElementItem {
    Func(FuncAddr),
    Expr(ConstInstruction),
}
