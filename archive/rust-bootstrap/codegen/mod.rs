//! Common codegen utilities shared across all targets

pub mod emitter;

pub use emitter::{CodeEmitter, EmitterFactory, EmitResult, EmitError};

/// Align value to boundary
pub fn align(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Align u64 value
pub fn align64(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Calculate entry point from base address and header size
pub fn entry_point(base: u64, header_size: usize) -> u64 {
    base + header_size as u64
}

/// Extract string literals and PRT operations from graph
use crate::ir::{Graph, Node, Op, Arg, Value};

pub fn extract_print_info(graph: &Graph) -> Option<(String, usize)> {
    let mut message = None;
    let mut length = 0usize;
    
    for node in &graph.nodes {
        match node {
            Node::Literal { value: Value::String(s), .. } => {
                message = Some(s.clone());
                length = s.len();
            }
            Node::Operation { op: Op::Prt, args, .. } => {
                if let Some(Arg::Lit(Value::Int(len))) = args.get(1) {
                    length = *len as usize;
                }
            }
            _ => {}
        }
    }
    
    message.map(|m| (m, length))
}

/// Syscall numbers for Linux x86-64
pub mod linux_x64_syscalls {
    pub const WRITE: u64 = 1;
    pub const READ: u64 = 0;
    pub const EXIT: u64 = 60;
    pub const OPEN: u64 = 2;
    pub const CLOSE: u64 = 3;
    pub const MMAP: u64 = 9;
    pub const SOCKET: u64 = 41;
    pub const CONNECT: u64 = 42;
    pub const BIND: u64 = 49;
    pub const LISTEN: u64 = 50;
    pub const ACCEPT: u64 = 43;
    pub const NANOSLEEP: u64 = 35;
    pub const CLOCK_GETTIME: u64 = 228;
}

/// Syscall numbers for Linux ARM64
pub mod linux_arm64_syscalls {
    pub const WRITE: u64 = 64;
    pub const READ: u64 = 63;
    pub const EXIT: u64 = 93;
    pub const OPEN: u64 = 56;
    pub const CLOSE: u64 = 57;
    pub const SOCKET: u64 = 198;
    pub const NANOSLEEP: u64 = 101;
    pub const CLOCK_GETTIME: u64 = 113;
}

/// macOS syscall offset (BSD layer)
pub const MACOS_SYSCALL_OFFSET: u64 = 0x2000000;
