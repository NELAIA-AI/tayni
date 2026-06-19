//! TAYNI Compiler Library
//! Exposes modules for testing and external use

pub mod ir;
pub mod parser;
pub mod emitter_pure;
pub mod binary;
pub mod native_emitter;
pub mod pe;  // Modular PE constants, headers, imports
pub mod pe_gen;  // PE generator (uses pe module)
pub mod elf;
pub mod elf_arm64;
pub mod macho;
pub mod capabilities;
pub mod modules;
pub mod wasm;
pub mod wasi;
pub mod riscv;
pub mod interface;
pub mod intent;
pub mod qir;
pub mod gpu;
pub mod target;
pub mod codegen;

pub use parser::Parser;
pub use ir::{Graph, Node, Op, Value, Arg, Capability};
