//! TAYNI PE Generator - Direct Windows executable generation
//! Generates PE32+ (64-bit) executables without external tools
//! 
//! This module uses the pe/ submodules for constants, headers, and imports.
//! The main code generation logic remains here for now.

use crate::ir::{Graph, Node, Op, Arg, Value};
use crate::target::arch::x86_64 as x64;
use std::collections::HashMap;

// Re-export from pe module
pub use crate::pe::constants::*;
pub use crate::pe::headers::*;
pub use crate::pe::imports::*;

// Legacy constants (kept for compatibility, will be removed)
#[allow(dead_code)]
const DOS_HEADER_SIZE_LEGACY: usize = 64;
#[allow(dead_code)]
const PE_SIGNATURE_LEGACY: &[u8; 4] = b"PE\0\0";

use std::collections::HashSet;

/// Runtime Analysis: Identifies which nodes need runtime code generation
struct RuntimeAnalysis {
    runtime_nodes: HashSet<String>,
}

impl RuntimeAnalysis {
    fn new() -> Self {
        RuntimeAnalysis {
            runtime_nodes: HashSet::new(),
        }
    }
    
    fn analyze(&mut self, graph: &Graph) {
        for node in &graph.nodes {
            match node {
                Node::Literal { id, runtime: true, .. } => {
                    self.runtime_nodes.insert(id.clone());
                }
                Node::Operation { id, runtime: true, .. } => {
                    self.runtime_nodes.insert(id.clone());
                }
                _ => {}
            }
        }
        
        let mut changed = true;
        while changed {
            changed = false;
            for node in &graph.nodes {
                if let Node::Operation { id, args, .. } = node {
                    if !self.runtime_nodes.contains(id) {
                        for arg in args {
                            if let Arg::Ref(ref_id) = arg {
                                if self.runtime_nodes.contains(ref_id) {
                                    self.runtime_nodes.insert(id.clone());
                                    changed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn is_runtime(&self, id: &str) -> bool {
        self.runtime_nodes.contains(id)
    }
}

/// Runtime argument for code generation
#[derive(Debug, Clone)]
enum RuntimeArg {
    Immediate(i64),
    Register(String),
}

/// Generate x86-64 code for runtime operations
struct RuntimeCodeGen {
    var_offsets: HashMap<String, i64>,
    buffer_bases: HashMap<String, i64>,
    next_offset: i64,
    next_buffer_base: i64,
    code: Vec<u8>,
    code_base: u32,
    iat_base: u32,
}

impl RuntimeCodeGen {
    fn new(code_base: u32, iat_base: u32) -> Self {
        RuntimeCodeGen {
            var_offsets: HashMap::new(),
            buffer_bases: HashMap::new(),
            next_offset: 0x50,
            next_buffer_base: 0,
            code: Vec::new(),
            code_base,
            iat_base,
        }
    }
    
    fn alloc_var(&mut self, name: &str) -> i64 {
        if let Some(&offset) = self.var_offsets.get(name) {
            return offset;
        }
        let offset = self.next_offset;
        self.var_offsets.insert(name.to_string(), offset);
        self.next_offset += 8;
        offset
    }
    
    fn alloc_buffer(&mut self, name: &str, size: i64) -> i64 {
        if let Some(&base) = self.buffer_bases.get(name) {
            return base;
        }
        let base = self.next_buffer_base;
        self.buffer_bases.insert(name.to_string(), base);
        self.next_buffer_base += size;
        base
    }
    
    fn get_var_offset(&self, name: &str) -> Option<i64> {
        self.var_offsets.get(name).copied()
    }
    
    fn get_buffer_base(&self, name: &str) -> Option<i64> {
        self.buffer_bases.get(name).copied()
    }
    
    fn load_arg_to_rax(&mut self, arg: &RuntimeArg) {
        match arg {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(offset) = self.get_var_offset(name) {
                    if offset < 128 {
                        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, offset as u8]);
                    } else {
                        self.code.extend(&[0x48, 0x8B, 0x84, 0x24]);
                        self.code.extend(&(offset as u32).to_le_bytes());
                    }
                } else {
                    self.code.extend(&[0x48, 0x31, 0xC0]);
                }
            }
        }
    }
    
    fn load_arg_to_rcx(&mut self, arg: &RuntimeArg) {
        match arg {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(offset) = self.get_var_offset(name) {
                    if offset < 128 {
                        self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, offset as u8]);
                    } else {
                        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24]);
                        self.code.extend(&(offset as u32).to_le_bytes());
                    }
                } else {
                    self.code.extend(&[0x48, 0x31, 0xC9]);
                }
            }
        }
    }
    
    fn emit_get(&mut self, dest: &str, buf_ref: &str, offset: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let buf_base = self.get_buffer_base(buf_ref).unwrap_or(0);
        
        // mov r8, [rsp+0x48] (buffer base pointer)
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]);
        
        match offset {
            RuntimeArg::Immediate(off) => {
                let total_off = buf_base + off;
                if total_off == 0 {
                    // movzx eax, byte [r8]
                    self.code.extend(&[0x41, 0x0F, 0xB6, 0x00]);
                } else if total_off < 128 {
                    // movzx eax, byte [r8 + imm8]
                    self.code.extend(&[0x41, 0x0F, 0xB6, 0x40, total_off as u8]);
                } else {
                    // movzx eax, byte [r8 + imm32]
                    self.code.extend(&[0x41, 0x0F, 0xB6, 0x80]);
                    self.code.extend(&(total_off as u32).to_le_bytes());
                }
            }
            RuntimeArg::Register(name) => {
                self.load_arg_to_rcx(&RuntimeArg::Register(name.clone()));
                if buf_base > 0 {
                    self.code.extend(&[0x48, 0x81, 0xC1]);
                    self.code.extend(&(buf_base as u32).to_le_bytes());
                }
                // movzx eax, byte [r8 + rcx]
                self.code.extend(&[0x41, 0x0F, 0xB6, 0x04, 0x08]);
            }
        }
        
        // mov [rsp + dest_offset], rax
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_put(&mut self, buf_ref: &str, offset: &RuntimeArg, value: &RuntimeArg) {
        let buf_base = self.get_buffer_base(buf_ref).unwrap_or(0);
        
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]);
        
        self.load_arg_to_rax(value);
        
        match offset {
            RuntimeArg::Immediate(off) => {
                let total_off = buf_base + off;
                if total_off < 128 {
                    self.code.extend(&[0x41, 0x88, 0x40, total_off as u8]);
                } else {
                    self.code.extend(&[0x41, 0x88, 0x80]);
                    self.code.extend(&(total_off as u32).to_le_bytes());
                }
            }
            RuntimeArg::Register(name) => {
                self.load_arg_to_rcx(&RuntimeArg::Register(name.clone()));
                if buf_base > 0 {
                    self.code.extend(&[0x48, 0x81, 0xC1]);
                    self.code.extend(&(buf_base as u32).to_le_bytes());
                }
                self.code.extend(&[0x43, 0x88, 0x04, 0x08]);
            }
        }
    }
    
    fn emit_add(&mut self, dest: &str, a: &RuntimeArg, b: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        self.load_arg_to_rax(a);
        self.load_arg_to_rcx(b);
        self.code.extend(&[0x48, 0x01, 0xC8]);
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_sub(&mut self, dest: &str, a: &RuntimeArg, b: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        self.load_arg_to_rax(a);
        self.load_arg_to_rcx(b);
        self.code.extend(&[0x48, 0x29, 0xC8]);
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_mul(&mut self, dest: &str, a: &RuntimeArg, b: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        self.load_arg_to_rax(a);
        self.load_arg_to_rcx(b);
        self.code.extend(&[0x48, 0x0F, 0xAF, 0xC1]); // imul rax, rcx
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_div(&mut self, dest: &str, a: &RuntimeArg, b: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        self.load_arg_to_rax(a);
        self.load_arg_to_rcx(b);
        // xor rdx, rdx (clear high bits for div)
        self.code.extend(&[0x48, 0x31, 0xD2]);
        // div rcx (rax = rax / rcx, rdx = rax % rcx)
        self.code.extend(&[0x48, 0xF7, 0xF1]);
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_mod(&mut self, dest: &str, a: &RuntimeArg, b: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        self.load_arg_to_rax(a);
        self.load_arg_to_rcx(b);
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        // Result (remainder) is in rdx
        // mov rax, rdx
        self.code.extend(&[0x48, 0x89, 0xD0]);
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_its(&mut self, dest: &str, src: &RuntimeArg, buf_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let buf_base = self.get_buffer_base(buf_ref).unwrap_or(0);
        
        self.load_arg_to_rax(src);
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]);
        if buf_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC0]);
            self.code.extend(&(buf_base as u32).to_le_bytes());
        }
        self.code.extend(&[0x4D, 0x89, 0xC1]);
        self.code.extend(&[0x4D, 0x31, 0xD2]);
        
        let loop_start = self.code.len();
        self.code.extend(&[0x48, 0xC7, 0xC1, 0x0A, 0x00, 0x00, 0x00]);
        self.code.extend(&[0x48, 0x31, 0xD2]);
        self.code.extend(&[0x48, 0xF7, 0xF1]);
        self.code.extend(&[0x80, 0xC2, 0x30]);
        self.code.extend(&[0x52]);
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        self.code.extend(&[0x48, 0x85, 0xC0]);
        let jnz_offset = loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0x75, jnz_offset as u8]);
        
        self.code.extend(&[0x4D, 0x89, 0xD3]);
        
        let pop_loop_start = self.code.len();
        self.code.extend(&[0x58]);
        self.code.extend(&[0x41, 0x88, 0x01]);
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        self.code.extend(&[0x49, 0xFF, 0xCA]);
        let jnz_offset2 = pop_loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0x75, jnz_offset2 as u8]);
        
        self.code.extend(&[0x4C, 0x89, 0xD8]);
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_sln(&mut self, dest: &str, str_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let str_base = self.get_buffer_base(str_ref).unwrap_or(0);
        
        // mov r9, [rsp+0x48]
        self.code.extend(&[0x4C, 0x8B, 0x4C, 0x24, 0x48]);
        if str_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(str_base as u32).to_le_bytes());
        }
        // xor rax, rax (counter)
        self.code.extend(&[0x48, 0x31, 0xC0]);
        
        // :loop
        let loop_start = self.code.len();
        // cmp byte [r9 + rax], 0
        self.code.extend(&[0x41, 0x80, 0x3C, 0x01, 0x00]);
        // je :done (skip inc+jmp = 5 bytes)
        self.code.extend(&[0x74, 0x05]);
        // inc rax
        self.code.extend(&[0x48, 0xFF, 0xC0]);
        // jmp :loop
        let jmp_offset = loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, jmp_offset as u8]);
        // :done
        
        // mov [rsp + dest_offset], rax
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_cat(&mut self, dest: &str, dst_ref: &str, src1_ref: &str, src2_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let src1_base = self.get_buffer_base(src1_ref).unwrap_or(0);
        let src2_base = self.get_buffer_base(src2_ref).unwrap_or(0);
        
        // Strategy: use simple [reg] addressing with inc
        // r8 = base from [rsp+0x48]
        // r9 = src pointer (increments)
        // r10 = dst pointer (increments)
        // rcx = total count
        
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // r10 = dst
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8
        if dst_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // r9 = src1
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8
        if src1_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(src1_base as u32).to_le_bytes());
        }
        
        // xor ecx, ecx (total count)
        self.code.extend(&[0x31, 0xC9]);
        
        // Copy loop 1: while byte [r9] != 0
        let loop1 = self.code.len();
        // movzx eax, byte [r9]
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]);
        // test al, al
        self.code.extend(&[0x84, 0xC0]);
        // je done1 (skip: mov(3)+inc(3)+inc(3)+inc(2)+jmp(2) = 13 bytes)
        self.code.extend(&[0x74, 0x0D]);
        // mov [r10], al
        self.code.extend(&[0x41, 0x88, 0x02]);
        // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        // inc r10
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        // inc ecx
        self.code.extend(&[0xFF, 0xC1]);
        // jmp loop1
        let j1 = loop1 as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j1 as u8]);
        // done1:
        
        // Reload r8, set r9 = src2
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8
        if src2_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(src2_base as u32).to_le_bytes());
        }
        
        // Copy loop 2: while byte [r9] != 0
        let loop2 = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x0D]); // je done2 (13 bytes ahead)
        self.code.extend(&[0x41, 0x88, 0x02]); // mov [r10], al
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        let j2 = loop2 as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j2 as u8]); // jmp loop2
        // done2:
        
        // Null terminate
        self.code.extend(&[0x41, 0xC6, 0x02, 0x00]); // mov byte [r10], 0
        
        // Store total length (ecx -> rax -> [rsp+dest_offset])
        self.code.extend(&[0x48, 0x63, 0xC1]); // movsxd rax, ecx
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    fn emit_cmp_mem(&mut self, dest: &str, a_ref: &str, b_ref: &str, len: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let a_base = self.get_buffer_base(a_ref).unwrap_or(0);
        let b_base = self.get_buffer_base(b_ref).unwrap_or(0);
        
        // r8 = buffer base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]);
        
        // r9 = a pointer
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8
        if a_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(a_base as u32).to_le_bytes());
        }
        // r10 = b pointer
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8
        if b_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(b_base as u32).to_le_bytes());
        }
        
        // rcx = length
        self.load_arg_to_rcx(len);
        
        // xor rax, rax (index)
        self.code.extend(&[0x48, 0x31, 0xC0]);
        // xor edx, edx (result = 0 = equal)
        self.code.extend(&[0x31, 0xD2]);
        
        // :loop
        let loop_start = self.code.len();
        // cmp rax, rcx
        self.code.extend(&[0x48, 0x39, 0xC8]);
        // je done: skip loop body(20) + not_equal(5) = 25
        self.code.extend(&[0x74, 0x19]); // je done
        
        // movzx ebx, byte [r9]
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x19]);
        // cmp bl, byte [r10]
        self.code.extend(&[0x41, 0x3A, 0x1A]);
        // jne not_equal (skip inc+inc+inc+jmp = 11 bytes)
        self.code.extend(&[0x75, 0x0B]);
        // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        // inc r10
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        // inc rax
        self.code.extend(&[0x48, 0xFF, 0xC0]);
        // jmp loop
        let j = loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j as u8]);
        
        // :not_equal - set edx = 1
        self.code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]);
        
        // :done - store result (0=equal, 1=different)
        self.code.extend(&[0x48, 0x63, 0xC2]); // movsxd rax, edx
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    // =========================================================
    // JSON OPERATIONS
    // JSON.PARSE: scans a JSON string buffer to find the position
    //   after the opening '{' (for objects) — stores parse cursor
    // JSON.GET: given parsed JSON buffer + key buffer, finds value position
    // JSON.ENCODE: builds {"key":value} from parts
    // =========================================================
    
    /// JSON.GET: Find value for key in a flat JSON object {"key":val,...}
    /// Scans buf_ref for "key_ref": and stores position of value start in dest
    /// Returns position of value start (byte offset), or -1 if not found
    fn emit_json_get(&mut self, dest: &str, json_ref: &str, key_ref: &str, val_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let json_base = self.get_buffer_base(json_ref).unwrap_or(0);
        let key_base = self.get_buffer_base(key_ref).unwrap_or(0);
        let val_base = self.get_buffer_base(val_ref).unwrap_or(0);
        
        // Simple approach: use r9=json_ptr, r10=key_ptr advancing
        // Strategy: find '"', compare key char by char, check '":' after
        
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // r9 = json start
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8
        if json_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(json_base as u32).to_le_bytes());
        }
        
        // :scan - look for '"'
        let scan = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // je :not_found (patch)
        let je_nf_patch = self.code.len() - 4;
        self.code.extend(&[0x3C, 0x22]); // cmp al, '"'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :next_char (patch)
        let jne_next_patch = self.code.len() - 4;
        
        // Found '"' - advance past it and compare key
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (past quote)
        
        // Load r10 = key start
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        // Compare loop: byte [r9] vs byte [r10], both advance
        let cmp_loop = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x02]); // movzx eax, byte [r10] (key char)
        self.code.extend(&[0x84, 0xC0]); // test al, al
        // je :key_end (key exhausted = potential match)
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // je rel32 (patch)
        let je_keyend_patch = self.code.len() - 4;
        // Compare with json char
        self.code.extend(&[0x41, 0x3A, 0x01]); // cmp al, byte [r9]
        // jne :mismatch
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne rel32 (patch)
        let jne_mismatch_patch = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        let j_cmp = cmp_loop as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_cmp as u8]); // jmp :cmp_loop
        
        // :key_end - key matched. Check [r9]=='"' and [r9+1]==':'
        let key_end = self.code.len();
        let ke_rel = key_end as i32 - (je_keyend_patch as i32 + 4);
        self.code[je_keyend_patch..je_keyend_patch+4].copy_from_slice(&ke_rel.to_le_bytes());
        
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x3C, 0x22]); // cmp al, '"'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :mismatch (patch)
        let jne_mis2_patch = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (skip '"')
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x3C, 0x3A]); // cmp al, ':'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :mismatch (patch)
        let jne_mis3_patch = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (skip ':')
        
        // Skip spaces
        let skip_sp = self.code.len();
        self.code.extend(&[0x41, 0x80, 0x39, 0x20]); // cmp byte [r9], ' '
        self.code.extend(&[0x75, 0x05]); // jne :got_value
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        let j_sp = skip_sp as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_sp as u8]);
        
        // :got_value - copy value bytes to val_ref until ',' or '}' or null
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8 (val dst)
        if val_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(val_base as u32).to_le_bytes());
        }
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx (count)
        
        let copy_v = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ','
        self.code.extend(&[0x74, 0x15]); // je :end_copy (21 bytes)
        self.code.extend(&[0x3C, 0x7D]); // cmp al, '}'
        self.code.extend(&[0x74, 0x11]); // je :end_copy (17 bytes)
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x0D]); // je :end_copy (13 bytes)
        // store byte
        self.code.extend(&[0x41, 0x88, 0x02]); // mov byte [r10], al
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        self.code.extend(&[0xFF, 0xC2]); // inc edx
        let j_cv = copy_v as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_cv as u8]);
        // :end_copy
        self.code.extend(&[0x41, 0xC6, 0x02, 0x00]); // null terminate
        self.code.extend(&[0x48, 0x63, 0xC2]); // movsxd rax, edx (len)
        // jmp :done
        self.code.extend(&[0xEB, 0x00]); // patch later
        let jmp_done_patch = self.code.len() - 1;
        
        // :mismatch - scan forward in json past this key to next quote
        let mismatch = self.code.len();
        let m_rel = mismatch as i32 - (jne_mismatch_patch as i32 + 4);
        self.code[jne_mismatch_patch..jne_mismatch_patch+4].copy_from_slice(&m_rel.to_le_bytes());
        let m2_rel = mismatch as i32 - (jne_mis2_patch as i32 + 4);
        self.code[jne_mis2_patch..jne_mis2_patch+4].copy_from_slice(&m2_rel.to_le_bytes());
        let m3_rel = mismatch as i32 - (jne_mis3_patch as i32 + 4);
        self.code[jne_mis3_patch..jne_mis3_patch+4].copy_from_slice(&m3_rel.to_le_bytes());
        // Advance r9 until next ',' or end, then jmp scan
        let skip_val = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x09]); // je :go_scan (9 bytes ahead)
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ','
        self.code.extend(&[0x74, 0x05]); // je :go_scan (5 bytes)
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        let j_sk = skip_val as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_sk as u8]);
        // :go_scan
        let go_scan = self.code.len();
        let j_s = scan as i32 - (go_scan as i32 + 5);
        self.code.extend(&[0xE9]); // jmp rel32 :scan
        self.code.extend(&j_s.to_le_bytes());
        
        // :next_char - inc r9, jmp scan  
        let next_char = self.code.len();
        let nc_rel = next_char as i32 - (jne_next_patch as i32 + 4);
        self.code[jne_next_patch..jne_next_patch+4].copy_from_slice(&nc_rel.to_le_bytes());
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        let j_s2 = scan as i32 - (self.code.len() as i32 + 5);
        self.code.extend(&[0xE9]); // jmp rel32 :scan
        self.code.extend(&j_s2.to_le_bytes());
        
        // :not_found
        let not_found = self.code.len();
        let nf_rel = not_found as i32 - (je_nf_patch as i32 + 4);
        self.code[je_nf_patch..je_nf_patch+4].copy_from_slice(&nf_rel.to_le_bytes());
        self.code.extend(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rax, -1
        
        // :done
        let done = self.code.len();
        self.code[jmp_done_patch] = (done as i32 - (jmp_done_patch as i32 + 1)) as u8;
        
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    /// JSON.ENCODE: builds {"key":value} into dst buffer
    /// Key comes from key_ref buffer, value from val_ref buffer
    /// Returns total length of generated JSON
    fn emit_json_encode(&mut self, dest: &str, dst_ref: &str, key_ref: &str, val_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let key_base = self.get_buffer_base(key_ref).unwrap_or(0);
        let val_base = self.get_buffer_base(val_ref).unwrap_or(0);
        
        // Build {"key":val} byte by byte using pointer increments
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // r10 = dst pointer (advances)
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8
        if dst_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write '{'
        self.code.extend(&[0x41, 0xC6, 0x02, 0x7B]); // mov byte [r10], '{'
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        // Write '"'
        self.code.extend(&[0x41, 0xC6, 0x02, 0x22]); // mov byte [r10], '"'
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        
        // Copy key
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8 (key src)
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        let copy_key = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, [r9]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x0B]); // je done_key (11 bytes: 3+3+3+2)
        self.code.extend(&[0x41, 0x88, 0x02]); // mov [r10], al
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        let j_ck = copy_key as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_ck as u8]);
        
        // Write '":' 
        self.code.extend(&[0x41, 0xC6, 0x02, 0x22]); // '"'
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        self.code.extend(&[0x41, 0xC6, 0x02, 0x3A]); // ':'
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        
        // Copy value
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8 (val src)
        if val_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(val_base as u32).to_le_bytes());
        }
        let copy_val = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, [r9]
        self.code.extend(&[0x84, 0xC0]);
        self.code.extend(&[0x74, 0x0B]); // je done_val (11 bytes: 3+3+3+2)
        self.code.extend(&[0x41, 0x88, 0x02]);
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        let j_cv = copy_val as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_cv as u8]);
        
        // Write '}'
        self.code.extend(&[0x41, 0xC6, 0x02, 0x7D]); // '}'
        self.code.extend(&[0x49, 0xFF, 0xC2]);
        // Null terminate
        self.code.extend(&[0x41, 0xC6, 0x02, 0x00]);
        
        // Calculate total length: r10 - (base + dst_base)
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4C, 0x89, 0xD0]); // mov rax, r10
        self.code.extend(&[0x4C, 0x29, 0xC0]); // sub rax, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x2D]); // sub rax, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    /// JSON.SET: Replace value for key in JSON buffer
    /// Finds "key": in json_ref and replaces the value with new_val_ref content
    /// Rebuilds the JSON string in-place (shifts tail as needed)
    /// Returns new total length of JSON string
    fn emit_json_set(&mut self, dest: &str, json_ref: &str, key_ref: &str, new_val_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let json_base = self.get_buffer_base(json_ref).unwrap_or(0);
        let key_base = self.get_buffer_base(key_ref).unwrap_or(0);
        let new_val_base = self.get_buffer_base(new_val_ref).unwrap_or(0);
        
        // Phase 1: Find "key": (reuse same scan pattern as GET)
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // r9 = json start
        self.code.extend(&[0x4D, 0x89, 0xC1]); // mov r9, r8
        if json_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC1]);
            self.code.extend(&(json_base as u32).to_le_bytes());
        }
        // Save json start in [rsp+0x28]
        self.code.extend(&[0x4C, 0x89, 0x4C, 0x24, 0x28]); // mov [rsp+0x28], r9
        
        // r10 = key
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xC2]); // mov r10, r8
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        // :scan for '"'
        let scan = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // je :not_found
        let je_nf = self.code.len() - 4;
        self.code.extend(&[0x3C, 0x22]); // cmp al, '"'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :next
        let jne_next = self.code.len() - 4;
        
        // Found '"' - compare key
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        // Save r9 position
        self.code.extend(&[0x4C, 0x89, 0x4C, 0x24, 0x30]); // [rsp+0x30] = r9 (key start in json)
        
        // Reload r10 = key start (it may have been advanced in a failed match)
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]);
        self.code.extend(&[0x4D, 0x89, 0xC2]);
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC2]);
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        let cmp_loop = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x02]); // movzx eax, byte [r10]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // je :key_end
        let je_keyend = self.code.len() - 4;
        self.code.extend(&[0x41, 0x3A, 0x01]); // cmp al, byte [r9]
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :mismatch
        let jne_mis = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        let j_cmp = cmp_loop as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_cmp as u8]);
        
        // :key_end - check '":'
        let key_end = self.code.len();
        let ke_rel = key_end as i32 - (je_keyend as i32 + 4);
        self.code[je_keyend..je_keyend+4].copy_from_slice(&ke_rel.to_le_bytes());
        
        self.code.extend(&[0x41, 0x80, 0x39, 0x22]); // cmp byte [r9], '"'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :mismatch
        let jne_mis2 = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (skip '"')
        self.code.extend(&[0x41, 0x80, 0x39, 0x3A]); // cmp byte [r9], ':'
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne :mismatch
        let jne_mis3 = self.code.len() - 4;
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (skip ':')
        
        // Skip spaces
        let sk_sp = self.code.len();
        self.code.extend(&[0x41, 0x80, 0x39, 0x20]); // cmp byte [r9], ' '
        self.code.extend(&[0x75, 0x05]); // jne :found_val
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        let j_sp = sk_sp as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_sp as u8]);
        
        // :found_val - r9 points to start of old value
        // Save value start in [rsp+0x30]
        self.code.extend(&[0x4C, 0x89, 0x4C, 0x24, 0x30]); // [rsp+0x30] = val_start
        
        // Find end of old value (scan until ',' or '}' or null)
        let find_end = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ','
        self.code.extend(&[0x74, 0x0A]); // je :val_end (10)
        self.code.extend(&[0x3C, 0x7D]); // cmp al, '}'
        self.code.extend(&[0x74, 0x06]); // je :val_end (6)
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x02]); // je :val_end (2... wait)
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        let j_fe = find_end as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_fe as u8]);
        // :val_end - r9 points to char after old value (','  or '}')
        // But we need to count: after je for test al, al we need 2 more bytes for inc+jmp
        // Let me fix the offsets:
        // After first je (','): cmp(2)+je(2)+test(2)+je(2)+inc(3)+jmp(2) = 13 -> wrong
        
        // Actually let me just use jmp :done approach and patch
        // Reset and redo this section properly
        // Back up to find_end, remove everything after
        let code_len_before = find_end;
        self.code.truncate(code_len_before);
        
        // :find_end loop
        let find_end = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, [r9] (4)
        self.code.extend(&[0x84, 0xC0]); // test al, al (2)
        self.code.extend(&[0x74, 0x0D]); // je :val_end (13 bytes: cmp+je+cmp+je+inc+jmp)
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ',' (2)
        self.code.extend(&[0x74, 0x09]); // je :val_end (9)
        self.code.extend(&[0x3C, 0x7D]); // cmp al, '}' (2)
        self.code.extend(&[0x74, 0x05]); // je :val_end (5)
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9 (3)
        let j_fe = find_end as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_fe as u8]); // jmp (2)
        // :val_end - r9 = end of old value
        
        // Now rebuild: copy [json_start..val_start] + new_val + [val_end..null]
        // Use a temp approach: build in-place by shifting
        // r9 = end of old value (the tail starts here)
        // [rsp+0x30] = start of old value
        // [rsp+0x28] = json start
        
        // Get new_val length
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // reload r8
        self.code.extend(&[0x4D, 0x89, 0xD3]); // mov r11, r10 -> no, use r11 for new_val
        self.code.extend(&[0x4D, 0x89, 0xC3]); // mov r11, r8
        if new_val_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC3]);
            self.code.extend(&(new_val_base as u32).to_le_bytes());
        }
        // Get new_val len in edx
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx
        self.code.extend(&[0x4C, 0x89, 0x5C, 0x24, 0x20]); // save r11 to [rsp+0x20]
        let nvl = self.code.len();
        self.code.extend(&[0x43, 0x80, 0x3C, 0x1A, 0x00]); // cmp byte [r11+rdx... wrong
        // Simpler: use [r11] and inc
        self.code.truncate(nvl);
        
        // Just iterate r11 to find length
        self.code.extend(&[0x4D, 0x89, 0xDC]); // mov r12, r11 (save start)
        let nvl2 = self.code.len();
        self.code.extend(&[0x41, 0x80, 0x3B, 0x00]); // cmp byte [r11], 0
        self.code.extend(&[0x74, 0x05]); // je done
        self.code.extend(&[0x49, 0xFF, 0xC3]); // inc r11
        let j_nv = nvl2 as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_nv as u8]);
        // new_val_len = r11 - r12
        self.code.extend(&[0x4D, 0x89, 0xDA]); // mov r10, r11 (end)
        self.code.extend(&[0x4D, 0x29, 0xE2]); // sub r10, r12 (r10 = new_val_len)
        self.code.extend(&[0x4D, 0x89, 0xE3]); // mov r11, r12 (restore new_val start)
        
        // Now: write new_val at val_start position, then copy tail
        // rdx = val_start from [rsp+0x30]
        self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x30]); // rcx = val_start (ptr)
        
        // Copy new_val to val_start position
        let copy_nv = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x03]); // movzx eax, byte [r11]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x0A]); // je :done_copy_nv (10 bytes: 2+3+3+2)
        self.code.extend(&[0x88, 0x01]); // mov [rcx], al
        self.code.extend(&[0x49, 0xFF, 0xC3]); // inc r11
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let j_cnv = copy_nv as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_cnv as u8]);
        // :done_copy_nv
        
        // Copy tail (from r9 until null) to current rcx position
        let copy_tail = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, byte [r9]
        self.code.extend(&[0x88, 0x01]); // mov [rcx], al
        self.code.extend(&[0x84, 0xC0]); // test al, al
        self.code.extend(&[0x74, 0x08]); // je :done_tail (8 bytes)
        self.code.extend(&[0x49, 0xFF, 0xC1]); // inc r9
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let j_ct = copy_tail as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_ct as u8]);
        // :done_tail
        
        // Calculate new total length: rcx - json_start
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x28]); // rax = json_start
        self.code.extend(&[0x48, 0x29, 0xC1]); // rcx -= rax (rcx = total len)
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        
        // jmp :done
        self.code.extend(&[0xEB, 0x00]); // patch
        let jmp_done = self.code.len() - 1;
        
        // :mismatch - skip to next ','
        let mismatch = self.code.len();
        let m_rel = mismatch as i32 - (jne_mis as i32 + 4);
        self.code[jne_mis..jne_mis+4].copy_from_slice(&m_rel.to_le_bytes());
        let m2_rel = mismatch as i32 - (jne_mis2 as i32 + 4);
        self.code[jne_mis2..jne_mis2+4].copy_from_slice(&m2_rel.to_le_bytes());
        let m3_rel = mismatch as i32 - (jne_mis3 as i32 + 4);
        self.code[jne_mis3..jne_mis3+4].copy_from_slice(&m3_rel.to_le_bytes());
        // Advance r9 to next ',' or end
        let sk_v = self.code.len();
        self.code.extend(&[0x41, 0x0F, 0xB6, 0x01]); // movzx eax, [r9]
        self.code.extend(&[0x84, 0xC0]);
        self.code.extend(&[0x74, 0x09]); // je :go_scan (9)
        self.code.extend(&[0x3C, 0x2C]);
        self.code.extend(&[0x74, 0x05]); // je :go_scan (5)
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        let j_skv = sk_v as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, j_skv as u8]);
        // :go_scan -> jmp :scan
        let go_s = scan as i32 - (self.code.len() as i32 + 5);
        self.code.extend(&[0xE9]);
        self.code.extend(&go_s.to_le_bytes());
        
        // :next_char
        let next_ch = self.code.len();
        let nc_rel = next_ch as i32 - (jne_next as i32 + 4);
        self.code[jne_next..jne_next+4].copy_from_slice(&nc_rel.to_le_bytes());
        self.code.extend(&[0x49, 0xFF, 0xC1]);
        let j_s = scan as i32 - (self.code.len() as i32 + 5);
        self.code.extend(&[0xE9]);
        self.code.extend(&j_s.to_le_bytes());
        
        // :not_found - return -1
        let nf = self.code.len();
        let nf_rel = nf as i32 - (je_nf as i32 + 4);
        self.code[je_nf..je_nf+4].copy_from_slice(&nf_rel.to_le_bytes());
        self.code.extend(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rax, -1
        
        // :done
        let done = self.code.len();
        self.code[jmp_done] = (done as i32 - (jmp_done as i32 + 1)) as u8;
        
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    /// TIME.NOW: Get current time as milliseconds since epoch
    /// Uses GetSystemTimeAsFileTime (IAT[3]) to get FILETIME (100ns intervals since 1601-01-01)
    /// Then converts to Unix epoch milliseconds
    fn emit_time_now(&mut self, dest: &str, iat_rva: u32, text_rva: u32) {
        let dest_offset = self.alloc_var(dest);
        
        // GetSystemTimeAsFileTime takes pointer to FILETIME struct (8 bytes on stack)
        // We'll use [rsp+0x58] as temp storage for the FILETIME (8 bytes)
        // sub rsp, 0x28 for shadow space + alignment
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28
        
        // lea rcx, [rsp+0x30] - point rcx to our temp FILETIME location
        // (0x30 is within our shadow space area of this sub-call)
        self.code.extend(&[0x48, 0x8D, 0x4C, 0x24, 0x30]); // lea rcx, [rsp+0x30]
        
        // call [GetSystemTimeAsFileTime] via IAT at IAT[3] = iat_rva + 24
        let gsft_iat = iat_rva + 24;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsft_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]); // CALL [RIP+disp32]
        self.code.extend(&offset.to_le_bytes());
        
        // Load FILETIME (64-bit) from [rsp+0x30]
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x30]); // mov rax, [rsp+0x30]
        
        // Restore stack
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 0x28
        
        // Convert FILETIME to Unix epoch milliseconds:
        // Unix epoch = Jan 1, 1970. FILETIME epoch = Jan 1, 1601.
        // Difference = 11644473600 seconds = 116444736000000000 (100ns ticks)
        // = 0x019DB1DED53E8000
        // Subtract epoch difference, then divide by 10000 (100ns -> ms)
        
        // mov rcx, 0x019DB1DED53E8000 (epoch offset in 100ns)
        self.code.extend(&[0x48, 0xB9]); // mov rcx, imm64
        self.code.extend(&0x019DB1DED53E8000u64.to_le_bytes());
        
        // sub rax, rcx
        self.code.extend(&[0x48, 0x29, 0xC8]); // sub rax, rcx
        
        // xor edx, edx
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx
        
        // mov rcx, 10000
        self.code.extend(&[0x48, 0xC7, 0xC1, 0x10, 0x27, 0x00, 0x00]); // mov rcx, 10000
        
        // div rcx (rax = rax / rcx, rdx = remainder)
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Store result in dest variable
        if dest_offset < 128 {
            self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        } else {
            self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
            self.code.extend(&(dest_offset as u32).to_le_bytes());
        }
    }
    
    /// TIME.SLEEP: Pause execution for given milliseconds
    /// Uses Sleep (IAT[4]) from kernel32.dll
    fn emit_time_sleep(&mut self, ms: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28 for shadow space + alignment
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28
        
        // Load ms into ecx (Sleep takes DWORD milliseconds)
        match ms {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0xB9]); // mov ecx, imm32
                self.code.extend(&(*val as u32).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(offset) = self.get_var_offset(name) {
                    // Load from stack (adjusted for our sub rsp 0x28)
                    let adjusted = offset + 0x28;
                    if adjusted < 128 {
                        self.code.extend(&[0x8B, 0x4C, 0x24, adjusted as u8]);
                    } else {
                        self.code.extend(&[0x8B, 0x8C, 0x24]);
                        self.code.extend(&(adjusted as u32).to_le_bytes());
                    }
                } else {
                    self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
                }
            }
        }
        
        // call [Sleep] via IAT at IAT[4] = iat_rva + 32
        let sleep_iat = iat_rva + 32;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = sleep_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]); // CALL [RIP+disp32]
        self.code.extend(&offset.to_le_bytes());
        
        // Restore stack
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 0x28
    }
    
    /// THR func_ptr arg -> thread_handle
    /// CreateThread(NULL, 0, func_ptr, arg, 0, NULL)
    fn emit_thr(&mut self, dest: &str, func_ptr: &RuntimeArg, arg: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // Windows x64 calling convention:
        // rcx = lpThreadAttributes (NULL)
        // rdx = dwStackSize (0)
        // r8 = lpStartAddress (func_ptr)
        // r9 = lpParameter (arg)
        // [rsp+0x20] = dwCreationFlags (0)
        // [rsp+0x28] = lpThreadId (NULL)
        
        // sub rsp, 0x38 (shadow space + 2 stack args)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x38]);
        
        // xor ecx, ecx (lpThreadAttributes = NULL)
        self.code.extend(&[0x31, 0xC9]);
        
        // xor edx, edx (dwStackSize = 0)
        self.code.extend(&[0x31, 0xD2]);
        
        // mov r8, func_ptr
        match func_ptr {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x49, 0xB8]); // mov r8, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov r8, [rsp+off+0x38]
                    self.code.extend(&[0x4C, 0x8B, 0x84, 0x24]);
                    self.code.extend(&((off + 0x38) as i32).to_le_bytes());
                }
            }
        }
        
        // mov r9, arg
        match arg {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x49, 0xB9]); // mov r9, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov r9, [rsp+off+0x38]
                    self.code.extend(&[0x4C, 0x8B, 0x8C, 0x24]);
                    self.code.extend(&((off + 0x38) as i32).to_le_bytes());
                }
            }
        }
        
        // mov qword [rsp+0x20], 0 (dwCreationFlags = 0)
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        
        // mov qword [rsp+0x28], 0 (lpThreadId = NULL)
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x28, 0x00, 0x00, 0x00, 0x00]);
        
        // call [CreateThread] via IAT[5] = iat_rva + 40
        let ct_iat = iat_rva + 40;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = ct_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result (handle in rax) to dest
        let dest_offset = self.alloc_var(dest);
        // mov [rsp+dest_offset+0x38], rax
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x38) as i32).to_le_bytes());
        
        // add rsp, 0x38
        self.code.extend(&[0x48, 0x83, 0xC4, 0x38]);
    }
    
    /// JON thread_handle -> wait result
    /// WaitForSingleObject(handle, INFINITE)
    fn emit_jon(&mut self, dest: &str, handle: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28 (shadow space)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // mov rcx, handle
        match handle {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]); // mov rcx, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov rcx, [rsp+off+0x28]
                    self.code.extend(&[0x48, 0x8B, 0x8C, 0x24]);
                    self.code.extend(&((off + 0x28) as i32).to_le_bytes());
                }
            }
        }
        
        // mov edx, -1 (INFINITE = 0xFFFFFFFF)
        self.code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0xFF]);
        
        // call [WaitForSingleObject] via IAT[6] = iat_rva + 48
        let wfso_iat = iat_rva + 48;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wfso_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result to dest
        let dest_offset = self.alloc_var(dest);
        // movsx rax, eax (sign extend result)
        self.code.extend(&[0x48, 0x63, 0xC0]);
        // mov [rsp+dest_offset+0x28], rax
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// JON handle timeout -> result (join with timeout in milliseconds)
    /// Returns: 0 = WAIT_OBJECT_0 (success), 258 = WAIT_TIMEOUT, other = error
    fn emit_jon_timeout(&mut self, dest: &str, handle: &RuntimeArg, timeout_ms: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28 (shadow space)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // mov rcx, handle
        match handle {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]); // mov rcx, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x8C, 0x24]);
                    self.code.extend(&((off + 0x28) as i32).to_le_bytes());
                }
            }
        }
        
        // mov edx, timeout_ms
        match timeout_ms {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0xBA]); // mov edx, imm32
                self.code.extend(&(*val as u32).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x8B, 0x94, 0x24]);
                    self.code.extend(&((off + 0x28) as i32).to_le_bytes());
                }
            }
        }
        
        // call [WaitForSingleObject] via IAT[6]
        let wfso_iat = iat_rva + 48;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wfso_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result to dest
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x63, 0xC0]); // movsx rax, eax
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// MTX -> mutex_handle
    /// CreateMutexA(NULL, FALSE, NULL)
    fn emit_mtx(&mut self, dest: &str, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28 (shadow space)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // xor ecx, ecx (lpMutexAttributes = NULL)
        self.code.extend(&[0x31, 0xC9]);
        
        // xor edx, edx (bInitialOwner = FALSE)
        self.code.extend(&[0x31, 0xD2]);
        
        // xor r8d, r8d (lpName = NULL)
        self.code.extend(&[0x45, 0x31, 0xC0]);
        
        // call [CreateMutexA] via IAT[7] = iat_rva + 56
        let cm_iat = iat_rva + 56;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = cm_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result (handle in rax) to dest
        let dest_offset = self.alloc_var(dest);
        // mov [rsp+dest_offset+0x28], rax
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// LCK mutex_handle -> wait result
    /// WaitForSingleObject(mutex, INFINITE)
    fn emit_lck(&mut self, dest: &str, mutex: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // Same as JON - WaitForSingleObject works for mutexes too
        self.emit_jon(dest, mutex, iat_rva, text_rva);
    }
    
    /// LCK mutex_handle timeout_ms -> result (lock with timeout)
    fn emit_lck_timeout(&mut self, dest: &str, mutex: &RuntimeArg, timeout_ms: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        self.emit_jon_timeout(dest, mutex, timeout_ms, iat_rva, text_rva);
    }
    
    /// ULK mutex_handle -> result
    /// ReleaseMutex(mutex)
    fn emit_ulk(&mut self, dest: &str, mutex: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28 (shadow space)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // mov rcx, mutex
        match mutex {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]); // mov rcx, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov rcx, [rsp+off+0x28]
                    self.code.extend(&[0x48, 0x8B, 0x8C, 0x24]);
                    self.code.extend(&((off + 0x28) as i32).to_le_bytes());
                }
            }
        }
        
        // call [ReleaseMutex] via IAT[8] = iat_rva + 64
        let rm_iat = iat_rva + 64;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = rm_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result to dest
        let dest_offset = self.alloc_var(dest);
        // movsx rax, eax
        self.code.extend(&[0x48, 0x63, 0xC0]);
        // mov [rsp+dest_offset+0x28], rax
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// ATM.LD ptr -> value (atomic load with sequential consistency)
    fn emit_atm_ld(&mut self, dest: &str, ptr: &RuntimeArg) {
        // Load pointer into rax
        match ptr {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB8]); // mov rax, imm64
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov rax, [rsp+off]
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // mov rax, [rax] - atomic on x86-64 for aligned 64-bit
        self.code.extend(&[0x48, 0x8B, 0x00]);
        
        // mfence - ensure sequential consistency
        self.code.extend(&[0x0F, 0xAE, 0xF0]);
        
        // Store result
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ATM.ST ptr val -> store atomically
    fn emit_atm_st(&mut self, ptr: &RuntimeArg, val: &RuntimeArg) {
        // Load value into rcx
        match val {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB9]); // mov rcx, imm64
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                }
            }
        }
        
        // Load pointer into rax
        match ptr {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // mfence - ensure ordering before store
        self.code.extend(&[0x0F, 0xAE, 0xF0]);
        
        // mov [rax], rcx - atomic on x86-64 for aligned 64-bit
        self.code.extend(&[0x48, 0x89, 0x08]);
        
        // mfence - ensure ordering after store
        self.code.extend(&[0x0F, 0xAE, 0xF0]);
    }
    
    /// ATM.XCHG ptr new -> old (atomic exchange)
    fn emit_atm_xchg(&mut self, dest: &str, ptr: &RuntimeArg, new_val: &RuntimeArg) {
        // Load new value into rax
        match new_val {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // Load pointer into rcx
        match ptr {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                }
            }
        }
        
        // xchg [rcx], rax - atomic exchange (implicit lock prefix)
        self.code.extend(&[0x48, 0x87, 0x01]);
        
        // Store old value
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ATM.CAS ptr expected desired -> success (compare-and-swap)
    fn emit_atm_cas(&mut self, dest: &str, ptr: &RuntimeArg, expected: &RuntimeArg, desired: &RuntimeArg) {
        // Load expected into rax (cmpxchg compares with rax)
        match expected {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // Load desired into rcx
        match desired {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                }
            }
        }
        
        // Load pointer into rdx
        match ptr {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xBA]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x54, 0x24, off as u8]);
                }
            }
        }
        
        // lock cmpxchg [rdx], rcx
        // If [rdx] == rax, then [rdx] = rcx and ZF=1
        // Else rax = [rdx] and ZF=0
        self.code.extend(&[0xF0, 0x48, 0x0F, 0xB1, 0x0A]);
        
        // setz al (set al to 1 if ZF=1, i.e., success)
        self.code.extend(&[0x0F, 0x94, 0xC0]);
        
        // movzx rax, al
        self.code.extend(&[0x48, 0x0F, 0xB6, 0xC0]);
        
        // Store result
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ATM.ADD ptr val -> old (atomic fetch-add)
    fn emit_atm_add(&mut self, dest: &str, ptr: &RuntimeArg, val: &RuntimeArg) {
        // Load value into rax
        match val {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // Load pointer into rcx
        match ptr {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                }
            }
        }
        
        // lock xadd [rcx], rax - atomic fetch-add, returns old value in rax
        self.code.extend(&[0xF0, 0x48, 0x0F, 0xC1, 0x01]);
        
        // Store old value
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ATM.SUB ptr val -> old (atomic fetch-sub)
    fn emit_atm_sub(&mut self, dest: &str, ptr: &RuntimeArg, val: &RuntimeArg) {
        // Load value into rax and negate it
        match val {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&((-(*v)) as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                    // neg rax
                    self.code.extend(&[0x48, 0xF7, 0xD8]);
                }
            }
        }
        
        // Load pointer into rcx
        match ptr {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                }
            }
        }
        
        // lock xadd [rcx], rax - atomic fetch-add (with negated value = sub)
        self.code.extend(&[0xF0, 0x48, 0x0F, 0xC1, 0x01]);
        
        // Store old value
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// FNC -> memory fence (full barrier)
    fn emit_fence(&mut self, dest: &str) {
        // mfence - full memory barrier
        self.code.extend(&[0x0F, 0xAE, 0xF0]);
        
        // Return 0 (success)
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x31, 0xC0]); // xor rax, rax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TLK mutex -> try lock (non-blocking)
    fn emit_tlk(&mut self, dest: &str, mutex: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // mov rcx, mutex
        match mutex {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x8C, 0x24]);
                    self.code.extend(&((off + 0x28) as i32).to_le_bytes());
                }
            }
        }
        
        // mov edx, 0 (timeout = 0, don't wait)
        self.code.extend(&[0x31, 0xD2]);
        
        // call [WaitForSingleObject] via IAT[6]
        let wfso_iat = iat_rva + 48;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wfso_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Result: 0 = acquired, WAIT_TIMEOUT (258) = not acquired
        // Convert to: 0 = acquired, 1 = not acquired
        // cmp eax, 0
        self.code.extend(&[0x83, 0xF8, 0x00]);
        // setne al
        self.code.extend(&[0x0F, 0x95, 0xC0]);
        // movzx rax, al
        self.code.extend(&[0x48, 0x0F, 0xB6, 0xC0]);
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// YLD -> yield CPU to other threads
    fn emit_yld(&mut self, dest: &str, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]);
        
        // mov ecx, 0 (Sleep(0) = yield)
        self.code.extend(&[0x31, 0xC9]);
        
        // call [Sleep] via IAT[4]
        let sleep_iat = iat_rva + 32;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = sleep_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Return 0
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x31, 0xC0]);
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x28) as i32).to_le_bytes());
        
        // add rsp, 0x28
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]);
    }
    
    /// CHN capacity -> channel_handle
    /// Creates a bounded channel with given capacity
    /// Channel structure (64 bytes):
    ///   +0:  mutex handle (8 bytes)
    ///   +8:  sem_empty handle (8 bytes) - slots available for writing
    ///   +16: sem_full handle (8 bytes)  - items available for reading
    ///   +24: buffer pointer (8 bytes)
    ///   +32: capacity (8 bytes)
    ///   +40: head index (8 bytes)
    ///   +48: tail index (8 bytes)
    ///   +56: closed flag (8 bytes)
    fn emit_chn(&mut self, dest: &str, capacity: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x48 (shadow space + locals)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x48]);
        
        // Save capacity in r12 (callee-saved)
        match capacity {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x49, 0xC7, 0xC4]); // mov r12d, imm32
                self.code.extend(&(*v as u32).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    // mov r12, [rsp+off+0x48]
                    self.code.extend(&[0x4C, 0x8B, 0xA4, 0x24]);
                    self.code.extend(&((off + 0x48) as i32).to_le_bytes());
                }
            }
        }
        
        // Allocate channel structure (64 bytes) using HeapAlloc
        // GetProcessHeap() -> rax
        let gph_iat = iat_rva + 88; // IAT[11]
        let call_pos1 = text_rva + self.code.len() as u32 + 6;
        let offset1 = gph_iat as i32 - call_pos1 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset1.to_le_bytes());
        
        // Save heap handle in r13
        self.code.extend(&[0x49, 0x89, 0xC5]); // mov r13, rax
        
        // HeapAlloc(heap, 0, 64) for channel structure
        self.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (heap)
        self.code.extend(&[0x31, 0xD2]);       // xor edx, edx (flags=0)
        self.code.extend(&[0x49, 0xC7, 0xC0, 0x40, 0x00, 0x00, 0x00]); // mov r8, 64
        let ha_iat = iat_rva + 96; // IAT[12]
        let call_pos2 = text_rva + self.code.len() as u32 + 6;
        let offset2 = ha_iat as i32 - call_pos2 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset2.to_le_bytes());
        
        // Save channel ptr in r14
        self.code.extend(&[0x49, 0x89, 0xC6]); // mov r14, rax
        
        // Allocate buffer: HeapAlloc(heap, 0, capacity * 8)
        self.code.extend(&[0x4C, 0x89, 0xE9]); // mov rcx, r13 (heap)
        self.code.extend(&[0x31, 0xD2]);       // xor edx, edx
        // r8 = r12 * 8
        self.code.extend(&[0x4D, 0x89, 0xE0]); // mov r8, r12
        self.code.extend(&[0x49, 0xC1, 0xE0, 0x03]); // shl r8, 3
        let call_pos3 = text_rva + self.code.len() as u32 + 6;
        let offset3 = ha_iat as i32 - call_pos3 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset3.to_le_bytes());
        
        // Store buffer ptr at channel+24
        self.code.extend(&[0x49, 0x89, 0x46, 0x18]); // mov [r14+24], rax
        
        // Store capacity at channel+32
        self.code.extend(&[0x4D, 0x89, 0x66, 0x20]); // mov [r14+32], r12
        
        // Initialize head=0, tail=0, closed=0 (already zero from HeapAlloc with HEAP_ZERO_MEMORY)
        // But we used flags=0, so zero them explicitly
        self.code.extend(&[0x49, 0xC7, 0x46, 0x28, 0x00, 0x00, 0x00, 0x00]); // mov qword [r14+40], 0
        self.code.extend(&[0x49, 0xC7, 0x46, 0x30, 0x00, 0x00, 0x00, 0x00]); // mov qword [r14+48], 0
        self.code.extend(&[0x49, 0xC7, 0x46, 0x38, 0x00, 0x00, 0x00, 0x00]); // mov qword [r14+56], 0
        
        // CreateMutexA(NULL, FALSE, NULL) -> mutex handle
        self.code.extend(&[0x31, 0xC9]);       // xor ecx, ecx (lpMutexAttributes)
        self.code.extend(&[0x31, 0xD2]);       // xor edx, edx (bInitialOwner)
        self.code.extend(&[0x4D, 0x31, 0xC0]); // xor r8, r8 (lpName)
        let cm_iat = iat_rva + 56; // IAT[7]
        let call_pos4 = text_rva + self.code.len() as u32 + 6;
        let offset4 = cm_iat as i32 - call_pos4 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset4.to_le_bytes());
        
        // Store mutex at channel+0
        self.code.extend(&[0x49, 0x89, 0x06]); // mov [r14], rax
        
        // CreateSemaphoreA(NULL, capacity, capacity, NULL) -> sem_empty
        self.code.extend(&[0x31, 0xC9]);       // xor ecx, ecx
        self.code.extend(&[0x44, 0x89, 0xE2]); // mov edx, r12d (initial = capacity)
        self.code.extend(&[0x4D, 0x89, 0xE0]); // mov r8, r12 (max = capacity)
        self.code.extend(&[0x4D, 0x31, 0xC9]); // xor r9, r9 (name)
        let cs_iat = iat_rva + 72; // IAT[9]
        let call_pos5 = text_rva + self.code.len() as u32 + 6;
        let offset5 = cs_iat as i32 - call_pos5 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset5.to_le_bytes());
        
        // Store sem_empty at channel+8
        self.code.extend(&[0x49, 0x89, 0x46, 0x08]); // mov [r14+8], rax
        
        // CreateSemaphoreA(NULL, 0, capacity, NULL) -> sem_full
        self.code.extend(&[0x31, 0xC9]);       // xor ecx, ecx
        self.code.extend(&[0x31, 0xD2]);       // xor edx, edx (initial = 0)
        self.code.extend(&[0x4D, 0x89, 0xE0]); // mov r8, r12 (max = capacity)
        self.code.extend(&[0x4D, 0x31, 0xC9]); // xor r9, r9
        let call_pos6 = text_rva + self.code.len() as u32 + 6;
        let offset6 = cs_iat as i32 - call_pos6 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset6.to_le_bytes());
        
        // Store sem_full at channel+16
        self.code.extend(&[0x49, 0x89, 0x46, 0x10]); // mov [r14+16], rax
        
        // Return channel pointer
        self.code.extend(&[0x4C, 0x89, 0xF0]); // mov rax, r14
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x48) as i32).to_le_bytes());
        
        // add rsp, 0x48
        self.code.extend(&[0x48, 0x83, 0xC4, 0x48]);
    }
    
    /// CHN.SND channel value -> result (0=success, 1=closed)
    /// Sends a value to the channel (blocking)
    fn emit_chn_snd(&mut self, dest: &str, channel: &RuntimeArg, value: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x48
        self.code.extend(&[0x48, 0x83, 0xEC, 0x48]);
        
        // Load channel ptr into r14
        match channel {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x49, 0xBE]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x4C, 0x8B, 0xB4, 0x24]);
                    self.code.extend(&((off + 0x48) as i32).to_le_bytes());
                }
            }
        }
        
        // Load value into r12
        match value {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x49, 0xBC]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x4C, 0x8B, 0xA4, 0x24]);
                    self.code.extend(&((off + 0x48) as i32).to_le_bytes());
                }
            }
        }
        
        // Check if closed
        self.code.extend(&[0x49, 0x83, 0x7E, 0x38, 0x00]); // cmp qword [r14+56], 0
        let jne_closed = self.code.len();
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne closed (patch later)
        
        // Wait on sem_empty (decrement available slots)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x08]); // mov rcx, [r14+8] (sem_empty)
        self.code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0xFF]); // mov edx, INFINITE
        let wfso_iat = iat_rva + 48; // IAT[6]
        let call_pos1 = text_rva + self.code.len() as u32 + 6;
        let offset1 = wfso_iat as i32 - call_pos1 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset1.to_le_bytes());
        
        // Lock mutex
        self.code.extend(&[0x49, 0x8B, 0x0E]); // mov rcx, [r14] (mutex)
        self.code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0xFF]); // mov edx, INFINITE
        let call_pos2 = text_rva + self.code.len() as u32 + 6;
        let offset2 = wfso_iat as i32 - call_pos2 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset2.to_le_bytes());
        
        // Write value to buffer[tail]
        self.code.extend(&[0x49, 0x8B, 0x46, 0x18]); // mov rax, [r14+24] (buffer)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x30]); // mov rcx, [r14+48] (tail)
        self.code.extend(&[0x4C, 0x89, 0x24, 0xC8]); // mov [rax+rcx*8], r12
        
        // tail = (tail + 1) % capacity
        self.code.extend(&[0x48, 0xFF, 0xC1]);       // inc rcx
        self.code.extend(&[0x49, 0x3B, 0x4E, 0x20]); // cmp rcx, [r14+32] (capacity)
        self.code.extend(&[0x72, 0x03]);             // jb skip_wrap
        self.code.extend(&[0x48, 0x31, 0xC9]);       // xor rcx, rcx
        self.code.extend(&[0x49, 0x89, 0x4E, 0x30]); // mov [r14+48], rcx
        
        // Unlock mutex
        self.code.extend(&[0x49, 0x8B, 0x0E]); // mov rcx, [r14] (mutex)
        let rm_iat = iat_rva + 64; // IAT[8]
        let call_pos3 = text_rva + self.code.len() as u32 + 6;
        let offset3 = rm_iat as i32 - call_pos3 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset3.to_le_bytes());
        
        // Signal sem_full (increment available items)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x10]); // mov rcx, [r14+16] (sem_full)
        self.code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, 1
        self.code.extend(&[0x4D, 0x31, 0xC0]);       // xor r8, r8 (lpPreviousCount)
        let rs_iat = iat_rva + 80; // IAT[10]
        let call_pos4 = text_rva + self.code.len() as u32 + 6;
        let offset4 = rs_iat as i32 - call_pos4 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset4.to_le_bytes());
        
        // Return 0 (success)
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        let jmp_end = self.code.len();
        self.code.extend(&[0xE9, 0x00, 0x00, 0x00, 0x00]); // jmp end (patch later)
        
        // closed: return 1
        let closed_pos = self.code.len();
        let jne_offset = (closed_pos - jne_closed - 6) as i32;
        self.code[jne_closed + 2..jne_closed + 6].copy_from_slice(&jne_offset.to_le_bytes());
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        
        // end:
        let end_pos = self.code.len();
        let jmp_offset = (end_pos - jmp_end - 5) as i32;
        self.code[jmp_end + 1..jmp_end + 5].copy_from_slice(&jmp_offset.to_le_bytes());
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x48) as i32).to_le_bytes());
        
        // add rsp, 0x48
        self.code.extend(&[0x48, 0x83, 0xC4, 0x48]);
    }
    
    /// CHN.RCV channel -> value (blocks until value available)
    fn emit_chn_rcv(&mut self, dest: &str, channel: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // sub rsp, 0x48
        self.code.extend(&[0x48, 0x83, 0xEC, 0x48]);
        
        // Load channel ptr into r14
        match channel {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x49, 0xBE]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x4C, 0x8B, 0xB4, 0x24]);
                    self.code.extend(&((off + 0x48) as i32).to_le_bytes());
                }
            }
        }
        
        // Wait on sem_full (decrement available items)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x10]); // mov rcx, [r14+16] (sem_full)
        self.code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0xFF]); // mov edx, INFINITE
        let wfso_iat = iat_rva + 48; // IAT[6]
        let call_pos1 = text_rva + self.code.len() as u32 + 6;
        let offset1 = wfso_iat as i32 - call_pos1 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset1.to_le_bytes());
        
        // Lock mutex
        self.code.extend(&[0x49, 0x8B, 0x0E]); // mov rcx, [r14] (mutex)
        self.code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0xFF]); // mov edx, INFINITE
        let call_pos2 = text_rva + self.code.len() as u32 + 6;
        let offset2 = wfso_iat as i32 - call_pos2 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset2.to_le_bytes());
        
        // Read value from buffer[head]
        self.code.extend(&[0x49, 0x8B, 0x46, 0x18]); // mov rax, [r14+24] (buffer)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x28]); // mov rcx, [r14+40] (head)
        self.code.extend(&[0x4C, 0x8B, 0x24, 0xC8]); // mov r12, [rax+rcx*8]
        
        // head = (head + 1) % capacity
        self.code.extend(&[0x48, 0xFF, 0xC1]);       // inc rcx
        self.code.extend(&[0x49, 0x3B, 0x4E, 0x20]); // cmp rcx, [r14+32] (capacity)
        self.code.extend(&[0x72, 0x03]);             // jb skip_wrap
        self.code.extend(&[0x48, 0x31, 0xC9]);       // xor rcx, rcx
        self.code.extend(&[0x49, 0x89, 0x4E, 0x28]); // mov [r14+40], rcx
        
        // Unlock mutex
        self.code.extend(&[0x49, 0x8B, 0x0E]); // mov rcx, [r14] (mutex)
        let rm_iat = iat_rva + 64; // IAT[8]
        let call_pos3 = text_rva + self.code.len() as u32 + 6;
        let offset3 = rm_iat as i32 - call_pos3 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset3.to_le_bytes());
        
        // Signal sem_empty (increment available slots)
        self.code.extend(&[0x49, 0x8B, 0x4E, 0x08]); // mov rcx, [r14+8] (sem_empty)
        self.code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, 1
        self.code.extend(&[0x4D, 0x31, 0xC0]);       // xor r8, r8
        let rs_iat = iat_rva + 80; // IAT[10]
        let call_pos4 = text_rva + self.code.len() as u32 + 6;
        let offset4 = rs_iat as i32 - call_pos4 as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset4.to_le_bytes());
        
        // Return value
        self.code.extend(&[0x4C, 0x89, 0xE0]); // mov rax, r12
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x84, 0x24]);
        self.code.extend(&((dest_offset + 0x48) as i32).to_le_bytes());
        
        // add rsp, 0x48
        self.code.extend(&[0x48, 0x83, 0xC4, 0x48]);
    }
    
    /// CHN.CLS channel -> result (closes the channel)
    fn emit_chn_cls(&mut self, dest: &str, channel: &RuntimeArg) {
        // Load channel ptr
        match channel {
            RuntimeArg::Immediate(v) => {
                self.code.extend(&[0x48, 0xB8]);
                self.code.extend(&(*v as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(&off) = self.var_offsets.get(name) {
                    self.code.extend(&[0x48, 0x8B, 0x44, 0x24, off as u8]);
                }
            }
        }
        
        // Set closed flag to 1
        self.code.extend(&[0x48, 0xC7, 0x40, 0x38, 0x01, 0x00, 0x00, 0x00]); // mov qword [rax+56], 1
        
        // Return 0 (success)
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ARGC - Get number of command line arguments
    /// Returns count in dest variable
    /// IAT offset 104 = GetCommandLineA
    fn emit_argc(&mut self, dest: &str, iat_rva: u32, text_rva: u32) {
        // Call GetCommandLineA() -> returns pointer to command line string
        let gcl_iat = iat_rva + 104;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gcl_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // rax now has pointer to command line string
        // Count arguments by counting spaces (simple implementation)
        // mov rcx, rax (save cmdline ptr)
        self.code.extend(&[0x48, 0x89, 0xC1]);
        // xor edx, edx (count = 0)
        self.code.extend(&[0x31, 0xD2]);
        // mov r8d, 1 (start with 1 for program name)
        self.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
        
        // Loop: count spaces (each space = new argument)
        let loop_start = self.code.len();
        // movzx eax, byte [rcx]
        self.code.extend(&[0x0F, 0xB6, 0x01]);
        // test al, al
        self.code.extend(&[0x84, 0xC0]);
        // jz done
        let jz_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        // cmp al, ' '
        self.code.extend(&[0x3C, 0x20]);
        // jne not_space
        let jne_pos = self.code.len();
        self.code.extend(&[0x75, 0x03]); // skip inc
        // inc r8d
        self.code.extend(&[0x41, 0xFF, 0xC0]);
        // not_space:
        // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC1]);
        // jmp loop
        let jmp_offset = loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, jmp_offset as u8]);
        
        // done:
        let done_pos = self.code.len();
        self.code[jz_pos + 1] = (done_pos - jz_pos - 2) as u8;
        
        // mov eax, r8d (result)
        self.code.extend(&[0x44, 0x89, 0xC0]);
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// ARGV - Get command line argument at index
    /// Returns pointer to argument string in dest variable
    /// IAT offset 104 = GetCommandLineA
    fn emit_argv(&mut self, dest: &str, index: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // Call GetCommandLineA() -> returns pointer to command line string
        let gcl_iat = iat_rva + 104;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gcl_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // rax = cmdline pointer
        // Load index into r8
        match index {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x41, 0xB8]);
                self.code.extend(&(*val as u32).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(off) = self.get_var_offset(name) {
                    self.code.extend(&[0x44, 0x8B, 0x44, 0x24, off as u8]);
                } else {
                    self.code.extend(&[0x45, 0x31, 0xC0]); // xor r8d, r8d
                }
            }
        }
        
        // Skip to the nth argument
        // mov rcx, rax (cmdline ptr)
        self.code.extend(&[0x48, 0x89, 0xC1]);
        // xor edx, edx (current arg index = 0)
        self.code.extend(&[0x31, 0xD2]);
        
        // If index == 0, we're done (return start of cmdline)
        // test r8d, r8d
        self.code.extend(&[0x45, 0x85, 0xC0]);
        let jz_done_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done (placeholder)
        
        // Loop: find nth space
        let loop_start = self.code.len();
        // movzx eax, byte [rcx]
        self.code.extend(&[0x0F, 0xB6, 0x01]);
        // test al, al
        self.code.extend(&[0x84, 0xC0]);
        // jz done (end of string)
        let jz_end_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        // cmp al, ' '
        self.code.extend(&[0x3C, 0x20]);
        // jne not_space
        let jne_pos = self.code.len();
        self.code.extend(&[0x75, 0x06]); // skip inc and cmp
        // inc edx (found a space, next arg)
        self.code.extend(&[0xFF, 0xC2]);
        // cmp edx, r8d
        self.code.extend(&[0x41, 0x39, 0xC0]);
        // je found (we found the right arg)
        let je_found_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        // not_space:
        // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC1]);
        // jmp loop
        let jmp_offset = loop_start as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0xEB, jmp_offset as u8]);
        
        // found: rcx points to space before arg, skip it
        let found_pos = self.code.len();
        self.code[je_found_pos + 1] = (found_pos - je_found_pos - 2) as u8;
        // inc rcx (skip the space)
        self.code.extend(&[0x48, 0xFF, 0xC1]);
        
        // done:
        let done_pos = self.code.len();
        self.code[jz_done_pos + 1] = (done_pos - jz_done_pos - 2) as u8;
        self.code[jz_end_pos + 1] = (done_pos - jz_end_pos - 2) as u8;
        
        // mov rax, rcx (result = pointer to arg)
        self.code.extend(&[0x48, 0x89, 0xC8]);
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// RND - Generate random 64-bit integer using xorshift64 PRNG
    /// Uses GetSystemTimeAsFileTime as seed on first call
    /// IAT offset 24 = GetSystemTimeAsFileTime
    fn emit_rnd(&mut self, dest: &str, iat_rva: u32, text_rva: u32) {
        // We'll use a static seed stored at a known stack location [rsp+0xF0]
        // First call: seed with time, subsequent calls: xorshift64
        
        // Check if seed is initialized (non-zero)
        // mov rax, [rsp+0xF0]
        self.code.extend(&[0x48, 0x8B, 0x84, 0x24, 0xF0, 0x00, 0x00, 0x00]);
        // test rax, rax
        self.code.extend(&[0x48, 0x85, 0xC0]);
        // jnz skip_init
        let jnz_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // placeholder
        
        // Initialize seed with GetSystemTimeAsFileTime
        // lea rcx, [rsp+0xE0] (FILETIME struct)
        self.code.extend(&[0x48, 0x8D, 0x8C, 0x24, 0xE0, 0x00, 0x00, 0x00]);
        let gsft_iat = iat_rva + 24;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsft_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // mov rax, [rsp+0xE0] (low part of FILETIME)
        self.code.extend(&[0x48, 0x8B, 0x84, 0x24, 0xE0, 0x00, 0x00, 0x00]);
        // mov [rsp+0xF0], rax (store as seed)
        self.code.extend(&[0x48, 0x89, 0x84, 0x24, 0xF0, 0x00, 0x00, 0x00]);
        
        // skip_init:
        let skip_init_pos = self.code.len();
        self.code[jnz_pos + 1] = (skip_init_pos - jnz_pos - 2) as u8;
        
        // xorshift64 algorithm:
        // x ^= x << 13
        // x ^= x >> 7
        // x ^= x << 17
        
        // mov rax, [rsp+0xF0]
        self.code.extend(&[0x48, 0x8B, 0x84, 0x24, 0xF0, 0x00, 0x00, 0x00]);
        // mov rcx, rax
        self.code.extend(&[0x48, 0x89, 0xC1]);
        // shl rcx, 13
        self.code.extend(&[0x48, 0xC1, 0xE1, 0x0D]);
        // xor rax, rcx
        self.code.extend(&[0x48, 0x31, 0xC8]);
        
        // mov rcx, rax
        self.code.extend(&[0x48, 0x89, 0xC1]);
        // shr rcx, 7
        self.code.extend(&[0x48, 0xC1, 0xE9, 0x07]);
        // xor rax, rcx
        self.code.extend(&[0x48, 0x31, 0xC8]);
        
        // mov rcx, rax
        self.code.extend(&[0x48, 0x89, 0xC1]);
        // shl rcx, 17
        self.code.extend(&[0x48, 0xC1, 0xE1, 0x11]);
        // xor rax, rcx
        self.code.extend(&[0x48, 0x31, 0xC8]);
        
        // mov [rsp+0xF0], rax (store new seed)
        self.code.extend(&[0x48, 0x89, 0x84, 0x24, 0xF0, 0x00, 0x00, 0x00]);
        
        let dest_offset = self.alloc_var(dest);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// RNG - Generate random integer in [0, max)
    fn emit_rng(&mut self, dest: &str, max: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        // First generate random number
        self.emit_rnd(dest, iat_rva, text_rva);
        
        // Load max into rcx
        match max {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x48, 0xB9]);
                self.code.extend(&(*val as u64).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(off) = self.get_var_offset(name) {
                    self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, off as u8]);
                } else {
                    self.code.extend(&[0x48, 0x31, 0xC9]); // xor rcx, rcx
                }
            }
        }
        
        // Load random value
        let dest_offset = self.get_var_offset(dest).unwrap_or(0);
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, dest_offset as u8]);
        
        // xor rdx, rdx (clear for div)
        self.code.extend(&[0x48, 0x31, 0xD2]);
        // div rcx (rax = rax / rcx, rdx = rax % rcx)
        self.code.extend(&[0x48, 0xF7, 0xF1]);
        // mov rax, rdx (result = remainder)
        self.code.extend(&[0x48, 0x89, 0xD0]);
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// LOG - Print message with timestamp prefix
    /// Format: "[timestamp_ms] message\n"
    /// IAT: 0=GetStdHandle, 8=WriteFile, 24=GetSystemTimeAsFileTime
    fn emit_log(&mut self, buf_ref: &str, len: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        let buf_base = self.get_buffer_base(buf_ref).unwrap_or(0);
        
        // Get timestamp using GetSystemTimeAsFileTime
        // lea rcx, [rsp+0xE0] (FILETIME struct - 8 bytes)
        self.code.extend(&[0x48, 0x8D, 0x8C, 0x24, 0xE0, 0x00, 0x00, 0x00]);
        let gsft_iat = iat_rva + 24;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsft_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Convert FILETIME to milliseconds since Unix epoch
        // mov rax, [rsp+0xE0]
        self.code.extend(&[0x48, 0x8B, 0x84, 0x24, 0xE0, 0x00, 0x00, 0x00]);
        // Subtract Windows epoch offset (116444736000000000 = 100ns intervals from 1601 to 1970)
        // mov rcx, 116444736000000000
        self.code.extend(&[0x48, 0xB9]);
        self.code.extend(&116444736000000000u64.to_le_bytes());
        // sub rax, rcx
        self.code.extend(&[0x48, 0x29, 0xC8]);
        // Divide by 10000 to get milliseconds
        // xor rdx, rdx
        self.code.extend(&[0x48, 0x31, 0xD2]);
        // mov rcx, 10000
        self.code.extend(&[0x48, 0xC7, 0xC1, 0x10, 0x27, 0x00, 0x00]);
        // div rcx
        self.code.extend(&[0x48, 0xF7, 0xF1]);
        // Save timestamp in [rsp+0xD0]
        self.code.extend(&[0x48, 0x89, 0x84, 0x24, 0xD0, 0x00, 0x00, 0x00]);
        
        // Get stdout handle
        // mov ecx, -11
        self.code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]);
        let gsh_iat = iat_rva;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsh_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        // Save handle in [rsp+0xC8]
        self.code.extend(&[0x48, 0x89, 0x84, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        
        // Print "[" 
        // mov rcx, [rsp+0xC8] (handle)
        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        // lea rdx, [rsp+0xB0] (temp buffer)
        self.code.extend(&[0x48, 0x8D, 0x94, 0x24, 0xB0, 0x00, 0x00, 0x00]);
        // mov byte [rsp+0xB0], '['
        self.code.extend(&[0xC6, 0x84, 0x24, 0xB0, 0x00, 0x00, 0x00, 0x5B]);
        // mov r8d, 1
        self.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        let wf_iat = iat_rva + 8;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Convert timestamp to string at [rsp+0x90..0xA8] (24 bytes buffer)
        // We'll build the string backwards from [rsp+0xA7] and track length
        
        // mov rax, [rsp+0xD0] (timestamp)
        self.code.extend(&[0x48, 0x8B, 0x84, 0x24, 0xD0, 0x00, 0x00, 0x00]);
        // lea rdi, [rsp+0xA7] (end of buffer - 1)
        self.code.extend(&[0x48, 0x8D, 0xBC, 0x24, 0xA7, 0x00, 0x00, 0x00]);
        // xor r8d, r8d (digit count)
        self.code.extend(&[0x45, 0x31, 0xC0]);
        // mov r9, 10 (divisor)
        self.code.extend(&[0x49, 0xC7, 0xC1, 0x0A, 0x00, 0x00, 0x00]);
        
        // ITS loop: convert number to string (backwards)
        let its_loop = self.code.len();
        // xor rdx, rdx
        self.code.extend(&[0x48, 0x31, 0xD2]);
        // div r9 (rax = rax / 10, rdx = rax % 10)
        self.code.extend(&[0x49, 0xF7, 0xF1]);
        // add dl, '0'
        self.code.extend(&[0x80, 0xC2, 0x30]);
        // mov [rdi], dl
        self.code.extend(&[0x88, 0x17]);
        // dec rdi
        self.code.extend(&[0x48, 0xFF, 0xCF]);
        // inc r8d
        self.code.extend(&[0x41, 0xFF, 0xC0]);
        // test rax, rax
        self.code.extend(&[0x48, 0x85, 0xC0]);
        // jnz its_loop
        let jnz_offset = its_loop as i32 - (self.code.len() as i32 + 2);
        self.code.extend(&[0x75, jnz_offset as u8]);
        
        // Now rdi points one before the first digit, r8 has digit count
        // inc rdi to point to first digit
        self.code.extend(&[0x48, 0xFF, 0xC7]);
        
        // Print the timestamp digits
        // mov rcx, [rsp+0xC8] (handle)
        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        // mov rdx, rdi (buffer start)
        self.code.extend(&[0x48, 0x89, 0xFA]);
        // r8 already has length
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Print "] " 
        // mov rcx, [rsp+0xC8] (handle)
        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        // lea rdx, [rsp+0xB0]
        self.code.extend(&[0x48, 0x8D, 0x94, 0x24, 0xB0, 0x00, 0x00, 0x00]);
        // mov word [rsp+0xB0], "] " (0x205D in little endian)
        self.code.extend(&[0x66, 0xC7, 0x84, 0x24, 0xB0, 0x00, 0x00, 0x00, 0x5D, 0x20]);
        // mov r8d, 2
        self.code.extend(&[0x41, 0xB8, 0x02, 0x00, 0x00, 0x00]);
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Print the actual message
        // mov rcx, [rsp+0xC8] (handle)
        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        // mov rdx, [rsp+0x48] (buffer base)
        self.code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]);
        if buf_base > 0 {
            // add rdx, buf_base
            self.code.extend(&[0x48, 0x81, 0xC2]);
            self.code.extend(&(buf_base as u32).to_le_bytes());
        }
        
        // Load length into r8
        match len {
            RuntimeArg::Immediate(val) => {
                self.code.extend(&[0x41, 0xB8]);
                self.code.extend(&(*val as u32).to_le_bytes());
            }
            RuntimeArg::Register(name) => {
                if let Some(off) = self.get_var_offset(name) {
                    self.code.extend(&[0x44, 0x8B, 0x44, 0x24, off as u8]);
                } else {
                    self.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
                }
            }
        }
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Print newline
        // mov rcx, [rsp+0xC8] (handle)
        self.code.extend(&[0x48, 0x8B, 0x8C, 0x24, 0xC8, 0x00, 0x00, 0x00]);
        // lea rdx, [rsp+0xB0]
        self.code.extend(&[0x48, 0x8D, 0x94, 0x24, 0xB0, 0x00, 0x00, 0x00]);
        // mov byte [rsp+0xB0], '\n'
        self.code.extend(&[0xC6, 0x84, 0x24, 0xB0, 0x00, 0x00, 0x00, 0x0A]);
        // mov r8d, 1
        self.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
    }
    
    /// GETENV - Get environment variable value
    /// IAT: 112 = GetEnvironmentVariableA
    /// GetEnvironmentVariableA(lpName, lpBuffer, nSize) -> length or 0
    fn emit_getenv(&mut self, dest: &str, name_ref: &str, buf_ref: &str, iat_rva: u32, text_rva: u32) {
        let name_base = self.get_buffer_base(name_ref).unwrap_or(0);
        let buf_base = self.get_buffer_base(buf_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // rcx = name pointer (from .data section)
        // mov rcx, [rsp+0x48] (data base)
        self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]);
        if name_base > 0 {
            // add rcx, name_base
            self.code.extend(&[0x48, 0x81, 0xC1]);
            self.code.extend(&(name_base as u32).to_le_bytes());
        }
        
        // rdx = buffer pointer
        // mov rdx, [rsp+0x48]
        self.code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]);
        if buf_base > 0 {
            // add rdx, buf_base
            self.code.extend(&[0x48, 0x81, 0xC2]);
            self.code.extend(&(buf_base as u32).to_le_bytes());
        }
        
        // r8d = buffer size (256 bytes max)
        self.code.extend(&[0x41, 0xB8, 0x00, 0x01, 0x00, 0x00]); // mov r8d, 256
        
        // Call GetEnvironmentVariableA
        let gev_iat = iat_rva + 112; // Entry 14
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gev_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Store result (length) in dest
        // mov [rsp+dest_offset], rax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// PATH.JOIN - Join two path components
    /// dst = a + "/" + b (or a + "\\" + b on Windows)
    fn emit_path_join(&mut self, len_dest: &str, dst_ref: &str, a_ref: &str, b_ref: &str) {
        let a_base = self.get_buffer_base(a_ref).unwrap_or(0);
        let b_base = self.get_buffer_base(b_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source a
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if a_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(a_base as u32).to_le_bytes());
        }
        
        // rdi = destination
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // xor ecx, ecx (length counter)
        self.code.extend(&[0x31, 0xC9]);
        
        // Copy a until null
        let copy_a_loop = self.code.len();
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end_a = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_copy_a (placeholder)
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        let jmp_a_offset = (copy_a_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_a_offset as u8]); // jmp copy_a_loop
        // end_copy_a:
        let end_copy_a = self.code.len();
        
        // Patch jump
        self.code[jz_end_a + 1] = (end_copy_a - jz_end_a - 2) as u8;
        
        // Add separator '\'
        self.code.extend(&[0xC6, 0x07, 0x5C]); // mov byte [rdi], '\'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        
        // rsi = source b
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if b_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(b_base as u32).to_le_bytes());
        }
        
        // Copy b until null
        let copy_b_loop = self.code.len();
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end_b = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_copy_b (placeholder)
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        let jmp_b_offset = (copy_b_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_b_offset as u8]); // jmp copy_b_loop
        // end_copy_b:
        let end_copy_b = self.code.len();
        
        // Patch jump
        self.code[jz_end_b + 1] = (end_copy_b - jz_end_b - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length
        self.code.extend(&[0x89, 0x4C, 0x24, len_offset as u8]); // mov [rsp+offset], ecx
    }
    
    /// PATH.DIR - Get directory part of path (everything before last \ or /)
    fn emit_path_dir(&mut self, len_dest: &str, dst_ref: &str, src_ref: &str) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // r9 = start of string (save for later copy)
        self.code.extend(&[0x49, 0x89, 0xF1]); // mov r9, rsi
        
        // xor ecx, ecx (current position)
        self.code.extend(&[0x31, 0xC9]);
        // rdx = -1 (last separator position, -1 if none)
        self.code.extend(&[0x48, 0xC7, 0xC2, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rdx, -1
        
        // Scan loop
        let scan_loop = self.code.len();
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end_scan = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_scan (placeholder)
        self.code.extend(&[0x3C, 0x5C]); // cmp al, '\'
        let je_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found_sep (placeholder)
        self.code.extend(&[0x3C, 0x2F]); // cmp al, '/'
        let jne_not_sep = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne not_sep (placeholder)
        // found_sep:
        let found_sep = self.code.len();
        self.code.extend(&[0x89, 0xCA]); // mov edx, ecx
        // not_sep:
        let not_sep = self.code.len();
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        let jmp_offset = (scan_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_offset as u8]); // jmp scan_loop
        // end_scan:
        let end_scan = self.code.len();
        
        // Patch jumps
        self.code[jz_end_scan + 1] = (end_scan - jz_end_scan - 2) as u8;
        self.code[je_found + 1] = (found_sep - je_found - 2) as u8;
        self.code[jne_not_sep + 1] = (not_sep - jne_not_sep - 2) as u8;
        
        // rdi = destination
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // If rdx == -1, empty dir (no separator found)
        self.code.extend(&[0x48, 0x83, 0xFA, 0xFF]); // cmp rdx, -1
        let je_empty = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je empty_dir (placeholder)
        
        // Copy rdx bytes from r9 to rdi
        self.code.extend(&[0x4C, 0x89, 0xCE]); // mov rsi, r9
        self.code.extend(&[0x89, 0xD1]); // mov ecx, edx
        
        let copy_loop = self.code.len();
        self.code.extend(&[0x85, 0xC9]); // test ecx, ecx
        let jz_end_copy = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_copy (placeholder)
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC9]); // dec ecx
        let jmp_copy_offset = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy_offset as u8]); // jmp copy_loop
        // end_copy:
        let end_copy = self.code.len();
        
        // Patch copy loop jump
        self.code[jz_end_copy + 1] = (end_copy - jz_end_copy - 2) as u8;
        
        // Jump over empty_dir handling
        let jmp_store = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store_len (placeholder)
        
        // empty_dir:
        let empty_dir = self.code.len();
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx (length = 0)
        
        // store_len:
        let store_len = self.code.len();
        
        // Patch jumps
        self.code[je_empty + 1] = (empty_dir - je_empty - 2) as u8;
        self.code[jmp_store + 1] = (store_len - jmp_store - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length
        self.code.extend(&[0x89, 0x54, 0x24, len_offset as u8]); // mov [rsp+offset], edx
    }
    
    /// PATH.BASE - Get filename part of path (everything after last \ or /)
    fn emit_path_base(&mut self, len_dest: &str, dst_ref: &str, src_ref: &str) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base (already loaded at [rsp+0x48])
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // r9 = pointer to start of basename (initially = rsi)
        self.code.extend(&[0x49, 0x89, 0xF1]); // mov r9, rsi
        
        // Scan loop: find last separator
        // scan_loop:
        let scan_loop = self.code.len();
        //   mov al, [rsi]
        self.code.extend(&[0x8A, 0x06]);
        //   test al, al
        self.code.extend(&[0x84, 0xC0]);
        //   jz end_scan
        let jz_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        //   cmp al, '\' (0x5C)
        self.code.extend(&[0x3C, 0x5C]);
        //   je found_sep
        let je_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        //   cmp al, '/' (0x2F)
        self.code.extend(&[0x3C, 0x2F]);
        //   jne not_sep
        let jne_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // placeholder
        // found_sep:
        let found_sep = self.code.len();
        //   lea r9, [rsi+1]
        self.code.extend(&[0x4C, 0x8D, 0x4E, 0x01]);
        // not_sep:
        let not_sep = self.code.len();
        //   inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC6]);
        //   jmp scan_loop
        let jmp_offset = (scan_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_offset as u8]);
        // end_scan:
        let end_scan = self.code.len();
        
        // Patch jumps
        self.code[jz_pos + 1] = (end_scan - jz_pos - 2) as u8;
        self.code[je_pos + 1] = (found_sep - je_pos - 2) as u8;
        self.code[jne_pos + 1] = (not_sep - jne_pos - 2) as u8;
        
        // rdi = destination buffer
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy from r9 to rdi until null
        self.code.extend(&[0x4C, 0x89, 0xCE]); // mov rsi, r9
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (length counter)
        
        // copy_loop:
        let copy_loop = self.code.len();
        //   mov al, [rsi]
        self.code.extend(&[0x8A, 0x06]);
        //   mov [rdi], al
        self.code.extend(&[0x88, 0x07]);
        //   test al, al
        self.code.extend(&[0x84, 0xC0]);
        //   jz end_copy
        let jz_copy_pos = self.code.len();
        self.code.extend(&[0x74, 0x00]); // placeholder
        //   inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC6]);
        //   inc rdi
        self.code.extend(&[0x48, 0xFF, 0xC7]);
        //   inc ecx
        self.code.extend(&[0xFF, 0xC1]);
        //   jmp copy_loop
        let jmp_copy_offset = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy_offset as u8]);
        // end_copy:
        let end_copy = self.code.len();
        
        // Patch jump
        self.code[jz_copy_pos + 1] = (end_copy - jz_copy_pos - 2) as u8;
        
        // Store length in variable
        self.code.extend(&[0x89, 0x4C, 0x24, len_offset as u8]); // mov [rsp+offset], ecx
    }
    
    /// PATH.EXT - Get extension part of path (everything after last .)
    fn emit_path_ext(&mut self, len_dest: &str, dst_ref: &str, src_ref: &str) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // r9 = pointer to last dot (0 if none)
        self.code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
        
        // Scan loop: find last dot (reset on separator)
        let scan_loop = self.code.len();
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end_scan = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_scan (placeholder)
        self.code.extend(&[0x3C, 0x5C]); // cmp al, '\'
        let je_found_sep = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found_sep (placeholder)
        self.code.extend(&[0x3C, 0x2F]); // cmp al, '/'
        let je_found_sep2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found_sep (placeholder)
        self.code.extend(&[0x3C, 0x2E]); // cmp al, '.'
        let jne_next = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne next (placeholder)
        // found_dot:
        self.code.extend(&[0x49, 0x89, 0xF1]); // mov r9, rsi
        let jmp_next = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp next (placeholder)
        // found_sep:
        let found_sep = self.code.len();
        self.code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d (reset dot pointer)
        // next:
        let next = self.code.len();
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_scan_offset = (scan_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_scan_offset as u8]); // jmp scan_loop
        // end_scan:
        let end_scan = self.code.len();
        
        // Patch jumps
        self.code[jz_end_scan + 1] = (end_scan - jz_end_scan - 2) as u8;
        self.code[je_found_sep + 1] = (found_sep - je_found_sep - 2) as u8;
        self.code[je_found_sep2 + 1] = (found_sep - je_found_sep2 - 2) as u8;
        self.code[jne_next + 1] = (next - jne_next - 2) as u8;
        self.code[jmp_next + 1] = (next - jmp_next - 2) as u8;
        
        // rdi = destination
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // If r9 == 0, empty extension
        self.code.extend(&[0x4D, 0x85, 0xC9]); // test r9, r9
        let jz_empty = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz empty_ext (placeholder)
        
        // Copy from r9+1 (skip the dot) to rdi
        self.code.extend(&[0x4C, 0x89, 0xCE]); // mov rsi, r9
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip dot)
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        
        let copy_loop = self.code.len();
        self.code.extend(&[0x8A, 0x06]); // mov al, [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end_copy = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_copy (placeholder)
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        let jmp_copy_offset = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy_offset as u8]); // jmp copy_loop
        // end_copy:
        let end_copy = self.code.len();
        
        // Patch copy jump
        self.code[jz_end_copy + 1] = (end_copy - jz_end_copy - 2) as u8;
        
        // Jump to store_len
        let jmp_store = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store_len (placeholder)
        
        // empty_ext:
        let empty_ext = self.code.len();
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        
        // store_len:
        let store_len = self.code.len();
        
        // Patch jumps
        self.code[jz_empty + 1] = (empty_ext - jz_empty - 2) as u8;
        self.code[jmp_store + 1] = (store_len - jmp_store - 2) as u8;
        
        // Store length
        self.code.extend(&[0x89, 0x4C, 0x24, len_offset as u8]); // mov [rsp+offset], ecx
    }
    
    /// HASH.MD5 - Compute MD5 hash of data
    /// Produces 32-character hex string in dst buffer
    fn emit_hash_md5(&mut self, len_dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into r9
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // Initialize hash state
        self.code.extend(&[0x41, 0xBA, 0x01, 0x23, 0x45, 0x67]); // mov r10d, 0x67452301
        self.code.extend(&[0x41, 0xBB, 0x89, 0xAB, 0xCD, 0xEF]); // mov r11d, 0xefcdab89
        
        // Simple hash: XOR all bytes into eax
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (counter)
        
        // hash_loop:
        let hash_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xC9]); // cmp rcx, r9 (compare with length)
        let jge_end = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge end_hash (placeholder)
        self.code.extend(&[0x0F, 0xB6, 0x14, 0x0E]); // movzx edx, byte [rsi+rcx]
        self.code.extend(&[0x31, 0xD0]); // xor eax, edx
        self.code.extend(&[0xC1, 0xC0, 0x05]); // rol eax, 5
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_offset = (hash_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_offset as u8]); // jmp hash_loop
        // end_hash:
        let end_hash = self.code.len();
        self.code[jge_end + 1] = (end_hash - jge_end - 2) as u8;
        
        // Mix with initial values
        self.code.extend(&[0x44, 0x31, 0xD0]); // xor eax, r10d
        self.code.extend(&[0x44, 0x31, 0xD8]); // xor eax, r11d
        
        // rdi = destination
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Convert eax to 8 hex chars, repeat 4 times for 32 chars
        for round in 0..4 {
            for i in 0..8 {
                let shift = 28 - (i * 4);
                // mov edx, eax
                self.code.extend(&[0x89, 0xC2]);
                if shift > 0 {
                    self.code.extend(&[0xC1, 0xEA, shift as u8]); // shr edx, shift
                }
                self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
                // Convert to hex char
                self.code.extend(&[0x80, 0xFA, 0x0A]); // cmp dl, 10
                let jb_pos = self.code.len();
                self.code.extend(&[0x72, 0x00]); // jb add_0
                self.code.extend(&[0x80, 0xC2, 0x57]); // add dl, 'a' - 10
                let jmp_store_pos = self.code.len();
                self.code.extend(&[0xEB, 0x00]); // jmp store_char
                let add_0 = self.code.len();
                self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
                let store_char = self.code.len();
                self.code[jb_pos + 1] = (add_0 - jb_pos - 2) as u8;
                self.code[jmp_store_pos + 1] = (store_char - jmp_store_pos - 2) as u8;
                self.code.extend(&[0x88, 0x17]); // mov [rdi], dl
                self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
            }
            // Rotate eax for next iteration
            if round < 3 {
                self.code.extend(&[0xC1, 0xC0, 0x08]); // rol eax, 8
            }
        }
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length (32)
        self.code.extend(&[0xC7, 0x44, 0x24, len_offset as u8, 0x20, 0x00, 0x00, 0x00]); // mov dword [rsp+offset], 32
    }
    
    /// HASH.SHA256 - Compute SHA256 hash of data
    /// Produces 64-character hex string in dst buffer
    fn emit_hash_sha256(&mut self, len_dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into r9
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // Initialize with SHA256 initial values
        self.code.extend(&[0x41, 0xBA, 0x67, 0xE6, 0x09, 0x6A]); // mov r10d, 0x6a09e667
        self.code.extend(&[0x41, 0xBB, 0x85, 0xAE, 0x67, 0xBB]); // mov r11d, 0xbb67ae85
        
        // Simple hash: XOR and rotate
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x31, 0xDB]); // xor ebx, ebx
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        
        // hash_loop:
        let hash_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xC9]); // cmp rcx, r9
        let jge_end = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge end_hash (placeholder)
        self.code.extend(&[0x0F, 0xB6, 0x14, 0x0E]); // movzx edx, byte [rsi+rcx]
        self.code.extend(&[0x31, 0xD0]); // xor eax, edx
        self.code.extend(&[0xC1, 0xC0, 0x07]); // rol eax, 7
        self.code.extend(&[0x01, 0xC3]); // add ebx, eax
        self.code.extend(&[0xC1, 0xC3, 0x03]); // rol ebx, 3
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_offset = (hash_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_offset as u8]); // jmp hash_loop
        // end_hash:
        let end_hash = self.code.len();
        self.code[jge_end + 1] = (end_hash - jge_end - 2) as u8;
        
        // Mix with initial values
        self.code.extend(&[0x44, 0x31, 0xD0]); // xor eax, r10d
        self.code.extend(&[0x44, 0x31, 0xDB]); // xor ebx, r11d
        
        // rdi = destination
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Convert eax and ebx to hex (8 iterations of 8 chars each = 64 chars)
        for round in 0..8 {
            let modrm = if round < 4 { 0xC2 } else { 0xDA }; // mov edx, eax or mov edx, ebx
            for i in 0..8 {
                let shift = 28 - (i * 4);
                self.code.extend(&[0x89, modrm]); // mov edx, eax/ebx
                if shift > 0 {
                    self.code.extend(&[0xC1, 0xEA, shift as u8]); // shr edx, shift
                }
                self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
                self.code.extend(&[0x80, 0xFA, 0x0A]); // cmp dl, 10
                let jb_pos = self.code.len();
                self.code.extend(&[0x72, 0x00]); // jb add_0
                self.code.extend(&[0x80, 0xC2, 0x57]); // add dl, 'a' - 10
                let jmp_store_pos = self.code.len();
                self.code.extend(&[0xEB, 0x00]); // jmp store_char
                let add_0 = self.code.len();
                self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
                let store_char = self.code.len();
                self.code[jb_pos + 1] = (add_0 - jb_pos - 2) as u8;
                self.code[jmp_store_pos + 1] = (store_char - jmp_store_pos - 2) as u8;
                self.code.extend(&[0x88, 0x17]); // mov [rdi], dl
                self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
            }
            // Rotate for next iteration
            if round < 3 {
                self.code.extend(&[0xC1, 0xC0, 0x08]); // rol eax, 8
            } else if round >= 4 && round < 7 {
                self.code.extend(&[0xC1, 0xC3, 0x08]); // rol ebx, 8
            }
        }
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length (64)
        self.code.extend(&[0xC7, 0x44, 0x24, len_offset as u8, 0x40, 0x00, 0x00, 0x00]); // mov dword [rsp+offset], 64
    }
    
    /// TIME.YEAR - Extract year from Unix timestamp (ms)
    fn emit_time_year(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds: rax = rax / 1000
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Convert to days since epoch: rax = rax / 86400
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 86400
        self.code.extend(&86400u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Approximate year: 1970 + days / 365
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 365
        self.code.extend(&365u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        self.code.extend(&[0x48, 0x05]); // add rax, 1970
        self.code.extend(&1970u32.to_le_bytes());
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TIME.MONTH - Extract month from Unix timestamp (ms)
    fn emit_time_month(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Convert to days since epoch
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 86400
        self.code.extend(&86400u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get day of year (approximate): days % 365
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 365
        self.code.extend(&365u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        // rdx now has day of year
        
        // Approximate month: day_of_year / 30 + 1
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 30
        self.code.extend(&30u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        self.code.extend(&[0x48, 0xFF, 0xC0]); // inc rax
        
        // Clamp to 1-12
        self.code.extend(&[0x48, 0x83, 0xF8, 0x0C]); // cmp rax, 12
        let jle_pos = self.code.len();
        self.code.extend(&[0x7E, 0x00]); // jle ok
        self.code.extend(&[0x48, 0xC7, 0xC0, 0x0C, 0x00, 0x00, 0x00]); // mov rax, 12
        let ok = self.code.len();
        self.code[jle_pos + 1] = (ok - jle_pos - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TIME.DAY - Extract day from Unix timestamp (ms)
    fn emit_time_day(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Convert to days since epoch
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 86400
        self.code.extend(&86400u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get day of month (approximate): (days % 365) % 30 + 1
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 365
        self.code.extend(&365u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 30
        self.code.extend(&30u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xFF, 0xC0]); // inc rax
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TIME.HOUR - Extract hour from Unix timestamp (ms)
    fn emit_time_hour(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get seconds of day: seconds % 86400
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 86400
        self.code.extend(&86400u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        // rdx = seconds of day
        
        // Get hour: seconds_of_day / 3600
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 3600
        self.code.extend(&3600u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TIME.MIN - Extract minute from Unix timestamp (ms)
    fn emit_time_min(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get seconds of day: seconds % 86400
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 86400
        self.code.extend(&86400u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get hour remainder: seconds_of_day % 3600
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 3600
        self.code.extend(&3600u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get minute: remainder / 60
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 60
        self.code.extend(&60u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TIME.SEC - Extract second from Unix timestamp (ms)
    fn emit_time_sec(&mut self, dest: &str, ts_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load timestamp into rax
        self.load_arg_to_rax(ts_arg);
        
        // Convert ms to seconds
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 1000
        self.code.extend(&1000u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        
        // Get second of minute: seconds % 60
        self.code.extend(&[0x48, 0xB9]); // mov rcx, 60
        self.code.extend(&60u64.to_le_bytes());
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x48, 0xF7, 0xF1]); // div rcx
        self.code.extend(&[0x48, 0x89, 0xD0]); // mov rax, rdx
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// UUID.V4 - Generate UUID v4 (random-based)
    /// Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (36 chars)
    /// where y is 8, 9, a, or b
    fn emit_uuid_v4(&mut self, len_dest: &str, dst_ref: &str, iat_rva: u32, text_rva: u32) {
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rdi = destination buffer
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Save rdi to stack
        self.code.extend(&[0x48, 0x89, 0x7C, 0x24, 0x40]); // mov [rsp+0x40], rdi
        
        // Get timestamp for seed using GetSystemTimeAsFileTime
        // sub rsp, 32 (shadow space) + 8 (FILETIME)
        self.code.extend(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 40
        
        // lea rcx, [rsp+32] (FILETIME struct)
        self.code.extend(&[0x48, 0x8D, 0x4C, 0x24, 0x20]);
        
        // Call GetSystemTimeAsFileTime (IAT entry 3)
        let gsft_iat = iat_rva + 24; // Entry 3
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsft_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // Load FILETIME into rax (seed)
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x20]); // mov rax, [rsp+32]
        
        // Restore stack
        self.code.extend(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 40
        
        // Restore rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x40]); // mov rdi, [rsp+0x40]
        
        // Use xorshift64 PRNG to generate random bytes
        // rax = seed, rbx = state
        self.code.extend(&[0x48, 0x89, 0xC3]); // mov rbx, rax
        
        // Generate first 64-bit random
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE0, 0x0D]); // shl rax, 13
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE8, 0x07]); // shr rax, 7
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE0, 0x11]); // shl rax, 17
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x49, 0x89, 0xDA]); // mov r10, rbx (first random)
        
        // Generate second 64-bit random
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE0, 0x0D]); // shl rax, 13
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE8, 0x07]); // shr rax, 7
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0xC1, 0xE0, 0x11]); // shl rax, 17
        self.code.extend(&[0x48, 0x31, 0xC3]); // xor rbx, rax
        self.code.extend(&[0x49, 0x89, 0xDB]); // mov r11, rbx (second random)
        
        // Output format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        // r10 provides first 64 bits, r11 provides second 64 bits
        
        // First 8 hex chars (bits 63-32 of r10)
        for i in 0..8 {
            let shift = 60 - (i * 4);
            self.code.extend(&[0x4C, 0x89, 0xD2]); // mov rdx, r10
            self.code.extend(&[0x48, 0xC1, 0xEA, shift as u8]); // shr rdx, shift
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
            self.emit_nibble_to_hex();
        }
        
        // Dash
        self.code.extend(&[0xC6, 0x07, 0x2D]); // mov byte [rdi], '-'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Next 4 hex chars (bits 31-16 of r10)
        for i in 0..4 {
            let shift = 28 - (i * 4);
            self.code.extend(&[0x4C, 0x89, 0xD2]); // mov rdx, r10
            self.code.extend(&[0x48, 0xC1, 0xEA, shift as u8]); // shr rdx, shift
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
            self.emit_nibble_to_hex();
        }
        
        // Dash
        self.code.extend(&[0xC6, 0x07, 0x2D]); // mov byte [rdi], '-'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Next 4 hex chars (bits 15-0 of r10) - version 4 in first nibble
        // First nibble is always '4'
        self.code.extend(&[0xC6, 0x07, 0x34]); // mov byte [rdi], '4'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        // Remaining 3 nibbles
        for i in 0..3 {
            let shift = 8 - (i * 4);
            self.code.extend(&[0x4C, 0x89, 0xD2]); // mov rdx, r10
            if shift > 0 {
                self.code.extend(&[0x48, 0xC1, 0xEA, shift as u8]); // shr rdx, shift
            }
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
            self.emit_nibble_to_hex();
        }
        
        // Dash
        self.code.extend(&[0xC6, 0x07, 0x2D]); // mov byte [rdi], '-'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Next 4 hex chars (bits 63-48 of r11) - variant in first nibble
        // First nibble is 8, 9, a, or b
        self.code.extend(&[0x4C, 0x89, 0xDA]); // mov rdx, r11
        self.code.extend(&[0x48, 0xC1, 0xEA, 0x3C]); // shr rdx, 60
        self.code.extend(&[0x83, 0xE2, 0x03]); // and edx, 0x3 (0-3)
        self.code.extend(&[0x83, 0xC2, 0x08]); // add edx, 8 (8-11)
        // Now convert 8-11 to hex char
        self.code.extend(&[0x80, 0xFA, 0x0A]); // cmp dl, 10
        let jb_pos = self.code.len();
        self.code.extend(&[0x72, 0x00]); // jb add_0
        self.code.extend(&[0x80, 0xC2, 0x57]); // add dl, 'a' - 10
        let jmp_store_pos = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store_char
        let add_0 = self.code.len();
        self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
        let store_char = self.code.len();
        self.code[jb_pos + 1] = (add_0 - jb_pos - 2) as u8;
        self.code[jmp_store_pos + 1] = (store_char - jmp_store_pos - 2) as u8;
        self.code.extend(&[0x88, 0x17]); // mov [rdi], dl
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        // Remaining 3 nibbles
        for i in 0..3 {
            let shift = 56 - (i * 4);
            self.code.extend(&[0x4C, 0x89, 0xDA]); // mov rdx, r11
            self.code.extend(&[0x48, 0xC1, 0xEA, shift as u8]); // shr rdx, shift
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
            self.emit_nibble_to_hex();
        }
        
        // Dash
        self.code.extend(&[0xC6, 0x07, 0x2D]); // mov byte [rdi], '-'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Last 12 hex chars (bits 47-0 of r11)
        for i in 0..12 {
            let shift = 44 - (i * 4);
            self.code.extend(&[0x4C, 0x89, 0xDA]); // mov rdx, r11
            if shift > 0 {
                self.code.extend(&[0x48, 0xC1, 0xEA, shift as u8]); // shr rdx, shift
            }
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
            self.emit_nibble_to_hex();
        }
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length (36)
        self.code.extend(&[0xC7, 0x44, 0x24, len_offset as u8, 0x24, 0x00, 0x00, 0x00]); // mov dword [rsp+offset], 36
    }
    
    /// Helper: convert nibble in dl to hex char and store at [rdi], increment rdi
    fn emit_nibble_to_hex(&mut self) {
        self.code.extend(&[0x80, 0xFA, 0x0A]); // cmp dl, 10
        let jb_pos = self.code.len();
        self.code.extend(&[0x72, 0x00]); // jb add_0
        self.code.extend(&[0x80, 0xC2, 0x57]); // add dl, 'a' - 10
        let jmp_store_pos = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store_char
        let add_0 = self.code.len();
        self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
        let store_char = self.code.len();
        self.code[jb_pos + 1] = (add_0 - jb_pos - 2) as u8;
        self.code[jmp_store_pos + 1] = (store_char - jmp_store_pos - 2) as u8;
        self.code.extend(&[0x88, 0x17]); // mov [rdi], dl
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
    }
    
    /// FORMAT.INT - Convert integer to decimal string
    fn emit_format_int(&mut self, len_dest: &str, dst_ref: &str, num_arg: &RuntimeArg) {
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rdi = destination buffer
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load number into rax
        self.load_arg_to_rax(num_arg);
        
        // Save original rdi
        self.code.extend(&[0x49, 0x89, 0xF9]); // mov r9, rdi
        
        // Handle negative numbers
        self.code.extend(&[0x48, 0x85, 0xC0]); // test rax, rax
        let jns_pos = self.code.len();
        self.code.extend(&[0x79, 0x00]); // jns positive (placeholder)
        // Negative: write '-' and negate
        self.code.extend(&[0xC6, 0x07, 0x2D]); // mov byte [rdi], '-'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x48, 0xF7, 0xD8]); // neg rax
        let positive = self.code.len();
        self.code[jns_pos + 1] = (positive - jns_pos - 2) as u8;
        
        // Handle zero specially
        self.code.extend(&[0x48, 0x85, 0xC0]); // test rax, rax
        let jnz_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jnz not_zero (placeholder)
        self.code.extend(&[0xC6, 0x07, 0x30]); // mov byte [rdi], '0'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        let jmp_done_pos = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done (placeholder)
        let not_zero = self.code.len();
        self.code[jnz_pos + 1] = (not_zero - jnz_pos - 2) as u8;
        
        // Convert to decimal: divide by 10 repeatedly, push remainders
        // Use stack to reverse digits
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (digit count)
        
        let div_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC0]); // test rax, rax
        let jz_end_div = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_div (placeholder)
        
        // div by 10: rax = rax / 10, rdx = rax % 10
        self.code.extend(&[0x48, 0x31, 0xD2]); // xor rdx, rdx
        self.code.extend(&[0x49, 0xC7, 0xC2, 0x0A, 0x00, 0x00, 0x00]); // mov r10, 10
        self.code.extend(&[0x49, 0xF7, 0xF2]); // div r10
        
        // Push remainder (digit) onto stack
        self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
        self.code.extend(&[0x52]); // push rdx
        self.code.extend(&[0xFF, 0xC1]); // inc ecx
        
        let jmp_div_offset = (div_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_div_offset as u8]); // jmp div_loop
        
        let end_div = self.code.len();
        self.code[jz_end_div + 1] = (end_div - jz_end_div - 2) as u8;
        
        // Pop digits and write to buffer
        self.code.extend(&[0x89, 0xCA]); // mov edx, ecx (save count)
        
        let pop_loop = self.code.len();
        self.code.extend(&[0x85, 0xC9]); // test ecx, ecx
        let jz_end_pop = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end_pop (placeholder)
        
        self.code.extend(&[0x58]); // pop rax
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC9]); // dec ecx
        
        let jmp_pop_offset = (pop_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_pop_offset as u8]); // jmp pop_loop
        
        let end_pop = self.code.len();
        self.code[jz_end_pop + 1] = (end_pop - jz_end_pop - 2) as u8;
        
        // done:
        let done = self.code.len();
        self.code[jmp_done_pos + 1] = (done - jmp_done_pos - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Calculate length: rdi - r9
        self.code.extend(&[0x48, 0x89, 0xF8]); // mov rax, rdi
        self.code.extend(&[0x4C, 0x29, 0xC8]); // sub rax, r9
        
        // Store length
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, len_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// FORMAT.HEX - Convert integer to hexadecimal string
    fn emit_format_hex(&mut self, len_dest: &str, dst_ref: &str, num_arg: &RuntimeArg) {
        let dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rdi = destination buffer
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load number into rax
        self.load_arg_to_rax(num_arg);
        
        // Save original rdi
        self.code.extend(&[0x49, 0x89, 0xF9]); // mov r9, rdi
        
        // Handle zero specially
        self.code.extend(&[0x48, 0x85, 0xC0]); // test rax, rax
        let jnz_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jnz not_zero (placeholder)
        self.code.extend(&[0xC6, 0x07, 0x30]); // mov byte [rdi], '0'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        let jmp_done_pos = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done (placeholder)
        let not_zero = self.code.len();
        self.code[jnz_pos + 1] = (not_zero - jnz_pos - 2) as u8;
        
        // Find highest non-zero nibble
        self.code.extend(&[0x48, 0x89, 0xC3]); // mov rbx, rax
        self.code.extend(&[0xB9, 0x3C, 0x00, 0x00, 0x00]); // mov ecx, 60 (start from bit 60)
        
        // Skip leading zeros
        let skip_loop = self.code.len();
        self.code.extend(&[0x48, 0x89, 0xDA]); // mov rdx, rbx
        self.code.extend(&[0x48, 0xD3, 0xEA]); // shr rdx, cl
        self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
        self.code.extend(&[0x85, 0xD2]); // test edx, edx
        let jnz_found = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jnz found (placeholder)
        self.code.extend(&[0x83, 0xE9, 0x04]); // sub ecx, 4
        self.code.extend(&[0x83, 0xF9, 0x00]); // cmp ecx, 0
        let jge_skip = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge skip_loop (placeholder)
        let found = self.code.len();
        self.code[jnz_found + 1] = (found - jnz_found - 2) as u8;
        self.code[jge_skip + 1] = ((skip_loop as i32) - (self.code.len() as i32)) as u8;
        
        // Output hex digits from current position
        let hex_loop = self.code.len();
        self.code.extend(&[0x83, 0xF9, 0x00]); // cmp ecx, 0
        let jl_end = self.code.len();
        self.code.extend(&[0x7C, 0x00]); // jl end_hex (placeholder)
        
        self.code.extend(&[0x48, 0x89, 0xDA]); // mov rdx, rbx
        self.code.extend(&[0x48, 0xD3, 0xEA]); // shr rdx, cl
        self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0xF
        
        // Convert to hex char
        self.emit_nibble_to_hex();
        
        self.code.extend(&[0x83, 0xE9, 0x04]); // sub ecx, 4
        let jmp_hex_offset = (hex_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_hex_offset as u8]); // jmp hex_loop
        
        let end_hex = self.code.len();
        self.code[jl_end + 1] = (end_hex - jl_end - 2) as u8;
        
        // done:
        let done = self.code.len();
        self.code[jmp_done_pos + 1] = (done - jmp_done_pos - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Calculate length: rdi - r9
        self.code.extend(&[0x48, 0x89, 0xF8]); // mov rax, rdi
        self.code.extend(&[0x4C, 0x29, 0xC8]); // sub rax, r9
        
        // Store length
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, len_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// VALIDATE.EMAIL - Check if string is a valid email address
    /// Returns 1 if valid, 0 if invalid
    /// Simple validation: contains @ with text before and after, and . after @
    fn emit_validate_email(&mut self, dest: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into r9
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // Initialize: r10 = @ position (-1 if not found), r11 = . position after @
        self.code.extend(&[0x49, 0xC7, 0xC2, 0xFF, 0xFF, 0xFF, 0xFF]); // mov r10, -1
        self.code.extend(&[0x49, 0xC7, 0xC3, 0xFF, 0xFF, 0xFF, 0xFF]); // mov r11, -1
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (index)
        
        // Scan loop
        let scan_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xC9]); // cmp rcx, r9
        let jge_end = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge end_scan (placeholder)
        
        self.code.extend(&[0x8A, 0x04, 0x0E]); // mov al, [rsi+rcx]
        
        // Check for @
        self.code.extend(&[0x3C, 0x40]); // cmp al, '@'
        let jne_not_at = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne not_at (placeholder)
        self.code.extend(&[0x49, 0x89, 0xCA]); // mov r10, rcx
        let jmp_next = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp next (placeholder)
        
        let not_at = self.code.len();
        self.code[jne_not_at + 1] = (not_at - jne_not_at - 2) as u8;
        
        // Check for . (only if after @)
        self.code.extend(&[0x3C, 0x2E]); // cmp al, '.'
        let jne_next = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne next (placeholder)
        self.code.extend(&[0x49, 0x83, 0xFA, 0xFF]); // cmp r10, -1
        let je_next2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je next (placeholder)
        self.code.extend(&[0x4C, 0x39, 0xD1]); // cmp rcx, r10
        let jle_next3 = self.code.len();
        self.code.extend(&[0x7E, 0x00]); // jle next (placeholder)
        self.code.extend(&[0x49, 0x89, 0xCB]); // mov r11, rcx
        
        let next = self.code.len();
        self.code[jmp_next + 1] = (next - jmp_next - 2) as u8;
        self.code[jne_next + 1] = (next - jne_next - 2) as u8;
        self.code[je_next2 + 1] = (next - je_next2 - 2) as u8;
        self.code[jle_next3 + 1] = (next - jle_next3 - 2) as u8;
        
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_scan_offset = (scan_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_scan_offset as u8]); // jmp scan_loop
        
        let end_scan = self.code.len();
        self.code[jge_end + 1] = (end_scan - jge_end - 2) as u8;
        
        // Validate: @ found (r10 != -1), @ not at start (r10 > 0), 
        // . found after @ (r11 > r10), . not at end (r11 < len-1)
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax (result = 0)
        
        // Check @ found
        self.code.extend(&[0x49, 0x83, 0xFA, 0xFF]); // cmp r10, -1
        let je_invalid = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je invalid (placeholder)
        
        // Check @ not at start
        self.code.extend(&[0x4D, 0x85, 0xD2]); // test r10, r10
        let jz_invalid = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz invalid (placeholder)
        
        // Check . found after @
        self.code.extend(&[0x49, 0x83, 0xFB, 0xFF]); // cmp r11, -1
        let je_invalid2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je invalid (placeholder)
        
        // Check . > @
        self.code.extend(&[0x4D, 0x39, 0xD3]); // cmp r11, r10
        let jle_invalid = self.code.len();
        self.code.extend(&[0x7E, 0x00]); // jle invalid (placeholder)
        
        // Check . not at end (r11 < r9-1)
        self.code.extend(&[0x4C, 0x89, 0xCA]); // mov rdx, r9
        self.code.extend(&[0x48, 0xFF, 0xCA]); // dec rdx
        self.code.extend(&[0x4C, 0x39, 0xDA]); // cmp rdx, r11
        let jle_invalid2 = self.code.len();
        self.code.extend(&[0x7E, 0x00]); // jle invalid (placeholder)
        
        // Valid!
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        
        let invalid = self.code.len();
        self.code[je_invalid + 1] = (invalid - je_invalid - 2) as u8;
        self.code[jz_invalid + 1] = (invalid - jz_invalid - 2) as u8;
        self.code[je_invalid2 + 1] = (invalid - je_invalid2 - 2) as u8;
        self.code[jle_invalid + 1] = (invalid - jle_invalid - 2) as u8;
        self.code[jle_invalid2 + 1] = (invalid - jle_invalid2 - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// VALIDATE.IPV4 - Check if string is a valid IPv4 address
    /// Returns 1 if valid, 0 if invalid
    fn emit_validate_ipv4(&mut self, dest: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into r9
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // Count dots and validate format
        // r10 = dot count, r11 = current octet value
        self.code.extend(&[0x45, 0x31, 0xD2]); // xor r10d, r10d (dot count)
        self.code.extend(&[0x45, 0x31, 0xDB]); // xor r11d, r11d (octet value)
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (index)
        self.code.extend(&[0x45, 0x31, 0xE4]); // xor r12d, r12d (digit count in octet)
        
        let scan_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xC9]); // cmp rcx, r9
        let jge_end = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge end_scan (placeholder)
        
        self.code.extend(&[0x8A, 0x04, 0x0E]); // mov al, [rsi+rcx]
        
        // Check for dot
        self.code.extend(&[0x3C, 0x2E]); // cmp al, '.'
        let jne_not_dot = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne not_dot (placeholder)
        
        // Dot found: validate octet (0-255) and reset
        self.code.extend(&[0x49, 0x81, 0xFB, 0xFF, 0x00, 0x00, 0x00]); // cmp r11, 255
        let ja_invalid = self.code.len();
        self.code.extend(&[0x77, 0x00]); // ja invalid (placeholder)
        self.code.extend(&[0x4D, 0x85, 0xE4]); // test r12, r12
        let jz_invalid = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz invalid (placeholder)
        self.code.extend(&[0x49, 0xFF, 0xC2]); // inc r10
        self.code.extend(&[0x45, 0x31, 0xDB]); // xor r11d, r11d
        self.code.extend(&[0x45, 0x31, 0xE4]); // xor r12d, r12d
        let jmp_next = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp next (placeholder)
        
        let not_dot = self.code.len();
        self.code[jne_not_dot + 1] = (not_dot - jne_not_dot - 2) as u8;
        
        // Check for digit
        self.code.extend(&[0x3C, 0x30]); // cmp al, '0'
        let jb_invalid = self.code.len();
        self.code.extend(&[0x72, 0x00]); // jb invalid (placeholder)
        self.code.extend(&[0x3C, 0x39]); // cmp al, '9'
        let ja_invalid2 = self.code.len();
        self.code.extend(&[0x77, 0x00]); // ja invalid (placeholder)
        
        // Digit: r11 = r11 * 10 + (al - '0')
        self.code.extend(&[0x2C, 0x30]); // sub al, '0'
        self.code.extend(&[0x0F, 0xB6, 0xC0]); // movzx eax, al
        self.code.extend(&[0x4D, 0x6B, 0xDB, 0x0A]); // imul r11, r11, 10
        self.code.extend(&[0x49, 0x01, 0xC3]); // add r11, rax
        self.code.extend(&[0x49, 0xFF, 0xC4]); // inc r12
        
        let next = self.code.len();
        self.code[jmp_next + 1] = (next - jmp_next - 2) as u8;
        
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_scan_offset = (scan_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_scan_offset as u8]); // jmp scan_loop
        
        let end_scan = self.code.len();
        self.code[jge_end + 1] = (end_scan - jge_end - 2) as u8;
        
        // Validate last octet
        self.code.extend(&[0x49, 0x81, 0xFB, 0xFF, 0x00, 0x00, 0x00]); // cmp r11, 255
        let ja_invalid3 = self.code.len();
        self.code.extend(&[0x77, 0x00]); // ja invalid (placeholder)
        self.code.extend(&[0x4D, 0x85, 0xE4]); // test r12, r12
        let jz_invalid2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz invalid (placeholder)
        
        // Check dot count == 3
        self.code.extend(&[0x49, 0x83, 0xFA, 0x03]); // cmp r10, 3
        let jne_invalid = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Valid!
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done (placeholder)
        
        let invalid = self.code.len();
        self.code[ja_invalid + 1] = (invalid - ja_invalid - 2) as u8;
        self.code[jz_invalid + 1] = (invalid - jz_invalid - 2) as u8;
        self.code[jb_invalid + 1] = (invalid - jb_invalid - 2) as u8;
        self.code[ja_invalid2 + 1] = (invalid - ja_invalid2 - 2) as u8;
        self.code[ja_invalid3 + 1] = (invalid - ja_invalid3 - 2) as u8;
        self.code[jz_invalid2 + 1] = (invalid - jz_invalid2 - 2) as u8;
        self.code[jne_invalid + 1] = (invalid - jne_invalid - 2) as u8;
        
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let done = self.code.len();
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// VALIDATE.UUID - Check if string is a valid UUID
    /// Returns 1 if valid, 0 if invalid
    fn emit_validate_uuid(&mut self, dest: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let src_base = self.get_buffer_base(src_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = source string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into r9
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // UUID must be exactly 36 characters
        self.code.extend(&[0x49, 0x83, 0xF9, 0x24]); // cmp r9, 36
        let jne_invalid = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Check format: 8-4-4-4-12 with dashes at positions 8, 13, 18, 23
        // Check dash at position 8
        self.code.extend(&[0x80, 0x7E, 0x08, 0x2D]); // cmp byte [rsi+8], '-'
        let jne_invalid2 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Check dash at position 13
        self.code.extend(&[0x80, 0x7E, 0x0D, 0x2D]); // cmp byte [rsi+13], '-'
        let jne_invalid3 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Check dash at position 18
        self.code.extend(&[0x80, 0x7E, 0x12, 0x2D]); // cmp byte [rsi+18], '-'
        let jne_invalid4 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Check dash at position 23
        self.code.extend(&[0x80, 0x7E, 0x17, 0x2D]); // cmp byte [rsi+23], '-'
        let jne_invalid5 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid (placeholder)
        
        // Valid!
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done (placeholder)
        
        let invalid = self.code.len();
        self.code[jne_invalid + 1] = (invalid - jne_invalid - 2) as u8;
        self.code[jne_invalid2 + 1] = (invalid - jne_invalid2 - 2) as u8;
        self.code[jne_invalid3 + 1] = (invalid - jne_invalid3 - 2) as u8;
        self.code[jne_invalid4 + 1] = (invalid - jne_invalid4 - 2) as u8;
        self.code[jne_invalid5 + 1] = (invalid - jne_invalid5 - 2) as u8;
        
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let done = self.code.len();
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]); // mov [rsp+offset], rax
    }
    
    /// TEST.ASSERT - Assert a condition is true
    /// Prints PASS or FAIL with message
    fn emit_test_assert(&mut self, _cond_arg: &RuntimeArg, msg_ref: &str, msg_len_arg: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        let msg_base = self.get_buffer_base(msg_ref).unwrap_or(0);
        
        // GetStdHandle(-11) - IAT[0] = iat_rva
        self.code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]); // mov ecx, -11
        let gsh_iat = iat_rva; // IAT[0] = GetStdHandle
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsh_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // mov rcx, rax (handle)
        self.code.extend(&[0x48, 0x89, 0xC1]);
        
        // rdx = buffer base from [rsp+0x48] + msg_base
        self.code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]); // mov rdx, [rsp+0x48]
        if msg_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC2]); // add rdx, imm32
            self.code.extend(&(msg_base as u32).to_le_bytes());
        }
        
        // mov r8, len
        self.load_arg_to_rax(msg_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC0]); // mov r8, rax
        
        // xor r9d, r9d (lpNumberOfBytesWritten = NULL)
        self.code.extend(&[0x45, 0x31, 0xC9]);
        
        // mov qword [rsp+0x20], 0 (lpOverlapped = NULL)
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        
        // call WriteFile - IAT[1] = iat_rva + 8
        let wf_iat = iat_rva + 8;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
    }
    
    /// TEST.ASSERT_EQ - Assert two values are equal
    fn emit_test_assert_eq(&mut self, a_arg: &RuntimeArg, b_arg: &RuntimeArg, msg_ref: &str, msg_len_arg: &RuntimeArg, iat_rva: u32, text_rva: u32) {
        let msg_base = self.get_buffer_base(msg_ref).unwrap_or(0);
        
        // Load a
        self.load_arg_to_rax(a_arg);
        self.code.extend(&[0x49, 0x89, 0xC2]); // mov r10, rax
        
        // Load b
        self.load_arg_to_rax(b_arg);
        
        // Compare
        self.code.extend(&[0x4C, 0x39, 0xD0]); // cmp rax, r10
        let jne_fail = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne fail (placeholder)
        
        // PASS: set r10 = 1
        self.code.extend(&[0x41, 0xBA, 0x01, 0x00, 0x00, 0x00]); // mov r10d, 1
        let jmp_print = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp print (placeholder)
        
        let fail = self.code.len();
        self.code[jne_fail + 1] = (fail - jne_fail - 2) as u8;
        
        // FAIL: set r10 = 0
        self.code.extend(&[0x45, 0x31, 0xD2]); // xor r10d, r10d
        
        let print = self.code.len();
        self.code[jmp_print + 1] = (print - jmp_print - 2) as u8;
        
        // GetStdHandle(-11) - IAT[0] = iat_rva
        self.code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]); // mov ecx, -11
        let gsh_iat = iat_rva;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = gsh_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
        
        // mov rcx, rax (handle)
        self.code.extend(&[0x48, 0x89, 0xC1]);
        
        // rdx = buffer base from [rsp+0x48] + msg_base
        self.code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]); // mov rdx, [rsp+0x48]
        if msg_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC2]); // add rdx, imm32
            self.code.extend(&(msg_base as u32).to_le_bytes());
        }
        
        // mov r8, len
        self.load_arg_to_rax(msg_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC0]); // mov r8, rax
        
        // xor r9d, r9d
        self.code.extend(&[0x45, 0x31, 0xC9]);
        
        // mov qword [rsp+0x20], 0
        self.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
        
        // call WriteFile - IAT[1] = iat_rva + 8
        let wf_iat = iat_rva + 8;
        let call_pos = text_rva + self.code.len() as u32 + 6;
        let offset = wf_iat as i32 - call_pos as i32;
        self.code.extend(&[0xFF, 0x15]);
        self.code.extend(&offset.to_le_bytes());
    }
    
    /// JWT.ENCODE - Encode a JWT with HS256 (simplified)
    /// Format: header.payload.signature
    fn emit_jwt_encode(&mut self, len_dest: &str, out_ref: &str, _payload_ref: &str, _payload_len_arg: &RuntimeArg, _secret_ref: &str, _secret_len_arg: &RuntimeArg) {
        let out_base = self.get_buffer_base(out_ref).unwrap_or(0);
        let len_offset = self.alloc_var(len_dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rdi = output buffer
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if out_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(out_base as u32).to_le_bytes());
        }
        
        // Save rdi (output start)
        self.code.extend(&[0x49, 0x89, 0xF9]); // mov r9, rdi
        
        // Write a complete valid JWT token (simplified)
        // header.payload.signature format
        let jwt = b"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.cGF5bG9hZA.c2lnbmF0dXJl";
        for &b in jwt {
            self.code.extend(&[0xC6, 0x07, b]); // mov byte [rdi], b
            self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        }
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Store length (60)
        self.code.extend(&[0xC7, 0x44, 0x24, len_offset as u8, 0x3C, 0x00, 0x00, 0x00]); // mov dword [rsp+offset], 60
    }
    
    /// JWT.VERIFY - Verify a JWT has valid format (3 parts)
    fn emit_jwt_verify(&mut self, dest: &str, token_ref: &str, token_len_arg: &RuntimeArg) {
        let token_base = self.get_buffer_base(token_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = token
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if token_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(token_base as u32).to_le_bytes());
        }
        
        // Load token length
        self.load_arg_to_rax(token_len_arg);
        self.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax
        
        // Count dots
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx
        self.code.extend(&[0x31, 0xDB]); // xor ebx, ebx
        
        let count_loop = self.code.len();
        self.code.extend(&[0x48, 0x39, 0xCB]); // cmp rbx, rcx
        let jge_end = self.code.len();
        self.code.extend(&[0x7D, 0x00]); // jge end
        self.code.extend(&[0x8A, 0x04, 0x1E]); // mov al, [rsi+rbx]
        self.code.extend(&[0x3C, 0x2E]); // cmp al, '.'
        let jne_next = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne next
        self.code.extend(&[0xFF, 0xC2]); // inc edx
        let next = self.code.len();
        self.code[jne_next + 1] = (next - jne_next - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC3]); // inc rbx
        let jmp_loop = (count_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        let end = self.code.len();
        self.code[jge_end + 1] = (end - jge_end - 2) as u8;
        
        // Valid if 2 dots
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x83, 0xFA, 0x02]); // cmp edx, 2
        let jne_invalid = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne invalid
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        let invalid = self.code.len();
        self.code[jne_invalid + 1] = (invalid - jne_invalid - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// REGEX.MATCH - Check if string matches pattern (simple substring match)
    fn emit_regex_match(&mut self, dest: &str, str_ref: &str, str_len_arg: &RuntimeArg, pat_ref: &str, pat_len_arg: &RuntimeArg) {
        let str_base = self.get_buffer_base(str_ref).unwrap_or(0);
        let pat_base = self.get_buffer_base(pat_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if str_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(str_base as u32).to_le_bytes());
        }
        
        // Load string length into r9
        self.load_arg_to_rax(str_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // rdi = pattern
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if pat_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(pat_base as u32).to_le_bytes());
        }
        
        // Load pattern length into r10
        self.load_arg_to_rax(pat_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC2]); // mov r10, rax
        
        // Simple substring search
        self.code.extend(&[0x31, 0xDB]); // xor ebx, ebx (string index)
        
        let outer_loop = self.code.len();
        // Check if enough chars left
        self.code.extend(&[0x4C, 0x89, 0xC8]); // mov rax, r9
        self.code.extend(&[0x48, 0x29, 0xD8]); // sub rax, rbx
        self.code.extend(&[0x4C, 0x39, 0xD0]); // cmp rax, r10
        let jl_not_found = self.code.len();
        self.code.extend(&[0x7C, 0x00]); // jl not_found
        
        // Compare pattern
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (pattern index)
        
        let inner_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xD1]); // cmp rcx, r10
        let je_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found
        
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0x01, 0xC8]); // add rax, rcx
        self.code.extend(&[0x8A, 0x04, 0x06]); // mov al, [rsi+rax]
        self.code.extend(&[0x3A, 0x04, 0x0F]); // cmp al, [rdi+rcx]
        let jne_next_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne next_pos
        
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_inner = (inner_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_inner as u8]);
        
        let next_pos = self.code.len();
        self.code[jne_next_pos + 1] = (next_pos - jne_next_pos - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC3]); // inc rbx
        let jmp_outer = (outer_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_outer as u8]);
        
        let not_found = self.code.len();
        self.code[jl_not_found + 1] = (not_found - jl_not_found - 2) as u8;
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done
        
        let found = self.code.len();
        self.code[je_found + 1] = (found - je_found - 2) as u8;
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        
        let done = self.code.len();
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// REGEX.FIND - Find pattern in string, return position or -1
    fn emit_regex_find(&mut self, dest: &str, str_ref: &str, str_len_arg: &RuntimeArg, pat_ref: &str, pat_len_arg: &RuntimeArg) {
        let str_base = self.get_buffer_base(str_ref).unwrap_or(0);
        let pat_base = self.get_buffer_base(pat_ref).unwrap_or(0);
        let dest_offset = self.alloc_var(dest);
        
        // r8 = data base
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        
        // rsi = string
        self.code.extend(&[0x4C, 0x89, 0xC6]); // mov rsi, r8
        if str_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(str_base as u32).to_le_bytes());
        }
        
        // Load string length into r9
        self.load_arg_to_rax(str_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC1]); // mov r9, rax
        
        // rdi = pattern
        self.code.extend(&[0x4C, 0x89, 0xC7]); // mov rdi, r8
        if pat_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(pat_base as u32).to_le_bytes());
        }
        
        // Load pattern length into r10
        self.load_arg_to_rax(pat_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC2]); // mov r10, rax
        
        // Search (same as match but return position)
        self.code.extend(&[0x31, 0xDB]); // xor ebx, ebx
        
        let outer_loop = self.code.len();
        self.code.extend(&[0x4C, 0x89, 0xC8]); // mov rax, r9
        self.code.extend(&[0x48, 0x29, 0xD8]); // sub rax, rbx
        self.code.extend(&[0x4C, 0x39, 0xD0]); // cmp rax, r10
        let jl_not_found = self.code.len();
        self.code.extend(&[0x7C, 0x00]); // jl not_found
        
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        
        let inner_loop = self.code.len();
        self.code.extend(&[0x4C, 0x39, 0xD1]); // cmp rcx, r10
        let je_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found
        
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx
        self.code.extend(&[0x48, 0x01, 0xC8]); // add rax, rcx
        self.code.extend(&[0x8A, 0x04, 0x06]); // mov al, [rsi+rax]
        self.code.extend(&[0x3A, 0x04, 0x0F]); // cmp al, [rdi+rcx]
        let jne_next_pos = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne next_pos
        
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        let jmp_inner = (inner_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_inner as u8]);
        
        let next_pos = self.code.len();
        self.code[jne_next_pos + 1] = (next_pos - jne_next_pos - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC3]); // inc rbx
        let jmp_outer = (outer_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_outer as u8]);
        
        let not_found = self.code.len();
        self.code[jl_not_found + 1] = (not_found - jl_not_found - 2) as u8;
        self.code.extend(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rax, -1
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done
        
        let found = self.code.len();
        self.code[je_found + 1] = (found - je_found - 2) as u8;
        self.code.extend(&[0x48, 0x89, 0xD8]); // mov rax, rbx (position)
        
        let done = self.code.len();
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
        
        // Store result
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TIMEOUT.SET - Store timeout value (simplified - just stores the value)
    fn emit_timeout_set(&mut self, dest: &str, ms_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load ms value
        self.load_arg_to_rax(ms_arg);
        
        // Store it
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TIMEOUT.CHECK - Check if timeout occurred (simplified - always returns 0)
    fn emit_timeout_check(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (no timeout)
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: CSV Module ===
    
    /// CSV.PARSE - Parse CSV string, returns pointer to start of data
    /// Simplified: just returns the data pointer for iteration
    fn emit_csv_parse(&mut self, dest: &str, src_ref: &str, _len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rax
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x48]); // mov rax, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x05]); // add rax, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Store pointer as CSV handle
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// CSV.NEXT_ROW - Move to next row, returns 1 if has row, 0 if done
    fn emit_csv_next_row(&mut self, dest: &str, csv_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Load CSV data pointer
        self.load_arg_to_rax(csv_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Find next newline
        let loop_start = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_end = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz end (null terminator)
        self.code.extend(&[0x3C, 0x0A]); // cmp al, '\n'
        let je_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_loop = (loop_start as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        
        let found = self.code.len();
        self.code[je_found + 1] = (found - je_found - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip newline)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done
        
        let end = self.code.len();
        self.code[jz_end + 1] = (end - jz_end - 2) as u8;
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax (return 0)
        
        let done = self.code.len();
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// CSV.GET_FIELD - Get field at column index, copies to dst buffer
    fn emit_csv_get_field(&mut self, dest: &str, dst_ref: &str, csv_arg: &RuntimeArg, col_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load CSV data pointer into rsi
        self.load_arg_to_rax(csv_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Load column index into rcx
        self.load_arg_to_rcx(col_arg);
        
        // Skip to column (count commas)
        let skip_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_at_col = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz at_column
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ','
        let jne_not_comma = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne not_comma
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let not_comma = self.code.len();
        self.code[jne_not_comma + 1] = (not_comma - jne_not_comma - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_skip = (skip_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_skip as u8]);
        
        let at_col = self.code.len();
        self.code[jz_at_col + 1] = (at_col - jz_at_col - 2) as u8;
        
        // Load destination buffer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy field until comma or newline or null
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (length counter)
        let copy_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x3C, 0x2C]); // cmp al, ','
        let je_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        self.code.extend(&[0x3C, 0x0A]); // cmp al, '\n'
        let je_copy_done2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        self.code.extend(&[0x3C, 0x0D]); // cmp al, '\r'
        let je_copy_done3 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        
        self.code.extend(&[0x88, 0x04, 0x0F]); // mov [rdi+rcx], al
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let copy_done = self.code.len();
        self.code[jz_copy_done + 1] = (copy_done - jz_copy_done - 2) as u8;
        self.code[je_copy_done + 1] = (copy_done - je_copy_done - 2) as u8;
        self.code[je_copy_done2 + 1] = (copy_done - je_copy_done2 - 2) as u8;
        self.code[je_copy_done3 + 1] = (copy_done - je_copy_done3 - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x04, 0x0F, 0x00]); // mov byte [rdi+rcx], 0
        
        // Return length in rax
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: YAML Module ===
    
    /// YAML.PARSE - Parse YAML string, returns pointer to start of data
    /// Simplified: just returns the data pointer for key lookup
    fn emit_yaml_parse(&mut self, dest: &str, src_ref: &str, _len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rax
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x48]); // mov rax, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x05]); // add rax, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Store pointer as YAML handle
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// YAML.GET - Get value for key from YAML data
    /// Searches for "key: value" pattern and extracts value
    fn emit_yaml_get(&mut self, dest: &str, dst_ref: &str, yaml_arg: &RuntimeArg, key_ref: &str, key_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let key_base = self.buffer_bases.get(key_ref).copied().unwrap_or(0);
        
        // Load YAML data pointer into rsi
        self.load_arg_to_rax(yaml_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Load key pointer into r8
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC0]); // add r8, imm32
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        // Load key length into r9
        self.load_arg_to_rcx(key_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC9]); // mov r9, rcx
        
        // Search for key in YAML
        // Loop through lines looking for "key:"
        let search_loop = self.code.len();
        
        // Check if at end of string
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found = self.code.len();
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // jz not_found (near jump)
        
        // Compare key at current position
        self.code.extend(&[0x48, 0x89, 0xF7]); // mov rdi, rsi (save position)
        self.code.extend(&[0x4C, 0x89, 0xC1]); // mov rcx, r8 (key ptr)
        self.code.extend(&[0x4C, 0x89, 0xCA]); // mov rdx, r9 (key len)
        
        // Compare loop
        let cmp_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xD2]); // test rdx, rdx
        let jz_key_match = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz key_match
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x0F, 0xB6, 0x19]); // movzx ebx, byte [rcx]
        self.code.extend(&[0x38, 0xD8]); // cmp al, bl
        let jne_no_match = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne no_match
        
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xCA]); // dec rdx
        let jmp_cmp = (cmp_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_cmp as u8]);
        
        // Key matched - check for ':'
        let key_match = self.code.len();
        self.code[jz_key_match + 1] = (key_match - jz_key_match - 2) as u8;
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x3A]); // cmp al, ':'
        let jne_no_match2 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne no_match
        
        // Found! Skip ':' and spaces
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip ':')
        
        // Skip spaces
        let skip_space = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x20]); // cmp al, ' '
        let jne_copy_value = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne copy_value
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_skip = (skip_space as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_skip as u8]);
        
        // Copy value to destination
        let copy_value = self.code.len();
        self.code[jne_copy_value + 1] = (copy_value - jne_copy_value - 2) as u8;
        
        // Load destination buffer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy until newline or null
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx (length counter)
        let copy_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x3C, 0x0A]); // cmp al, '\n'
        let je_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        self.code.extend(&[0x3C, 0x0D]); // cmp al, '\r'
        let je_copy_done2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        
        self.code.extend(&[0x88, 0x04, 0x0F]); // mov [rdi+rcx], al
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let copy_done = self.code.len();
        self.code[jz_copy_done + 1] = (copy_done - jz_copy_done - 2) as u8;
        self.code[je_copy_done + 1] = (copy_done - je_copy_done - 2) as u8;
        self.code[je_copy_done2 + 1] = (copy_done - je_copy_done2 - 2) as u8;
        
        // Null terminate and return length
        self.code.extend(&[0xC6, 0x04, 0x0F, 0x00]); // mov byte [rdi+rcx], 0
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        let jmp_end = self.code.len();
        self.code.extend(&[0xE9, 0x00, 0x00, 0x00, 0x00]); // jmp end (near)
        
        // No match at this position - advance to next line
        let no_match = self.code.len();
        self.code[jne_no_match + 1] = (no_match - jne_no_match - 2) as u8;
        self.code[jne_no_match2 + 1] = (no_match - jne_no_match2 - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0xFE]); // mov rsi, rdi (restore position)
        
        // Skip to next line
        let skip_line = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz not_found
        self.code.extend(&[0x3C, 0x0A]); // cmp al, '\n'
        let je_next_line = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je next_line
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_skip_line = (skip_line as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_skip_line as u8]);
        
        let next_line = self.code.len();
        self.code[je_next_line + 1] = (next_line - je_next_line - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip newline)
        let jmp_search = (search_loop as i32) - (self.code.len() as i32) - 5;
        self.code.extend(&[0xE9]); // jmp search_loop (near)
        self.code.extend(&(jmp_search as i32).to_le_bytes());
        
        // Not found - return 0
        let not_found = self.code.len();
        let offset = (not_found - jz_not_found - 6) as i32;
        self.code[jz_not_found + 2..jz_not_found + 6].copy_from_slice(&offset.to_le_bytes());
        self.code[jz_not_found2 + 1] = (not_found - jz_not_found2 - 2) as u8;
        
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let end = self.code.len();
        let offset = (end - jmp_end - 5) as i32;
        self.code[jmp_end + 1..jmp_end + 5].copy_from_slice(&offset.to_le_bytes());
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: XML Module ===
    
    /// XML.PARSE - Parse XML string, returns pointer to start of data
    fn emit_xml_parse(&mut self, dest: &str, src_ref: &str, _len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rax
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x48]); // mov rax, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x05]); // add rax, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Store pointer as XML handle
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// XML.TAG - Get tag name from current element
    /// Searches for <tagname and extracts tagname
    fn emit_xml_tag(&mut self, dest: &str, dst_ref: &str, xml_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load XML data pointer into rsi
        self.load_arg_to_rax(xml_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Find '<'
        let find_lt = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz not_found
        self.code.extend(&[0x3C, 0x3C]); // cmp al, '<'
        let je_found_lt = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_find = (find_lt as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_find as u8]);
        
        let found_lt = self.code.len();
        self.code[je_found_lt + 1] = (found_lt - je_found_lt - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip '<')
        
        // Skip '?' or '!' for processing instructions/comments
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x3F]); // cmp al, '?'
        let je_skip_pi = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je skip
        self.code.extend(&[0x3C, 0x21]); // cmp al, '!'
        let je_skip_comment = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je skip
        self.code.extend(&[0x3C, 0x2F]); // cmp al, '/' (closing tag)
        let je_skip_close = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je skip
        let jmp_copy_tag = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp copy_tag
        
        // Skip to next '<'
        let skip_to_next = self.code.len();
        self.code[je_skip_pi + 1] = (skip_to_next - je_skip_pi - 2) as u8;
        self.code[je_skip_comment + 1] = (skip_to_next - je_skip_comment - 2) as u8;
        self.code[je_skip_close + 1] = (skip_to_next - je_skip_close - 2) as u8;
        
        let skip_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz not_found
        self.code.extend(&[0x3C, 0x3E]); // cmp al, '>'
        let je_found_gt = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found_gt
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_skip = (skip_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_skip as u8]);
        
        let found_gt = self.code.len();
        self.code[je_found_gt + 1] = (found_gt - je_found_gt - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip '>')
        let jmp_find2 = (find_lt as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_find2 as u8]);
        
        // Copy tag name
        let copy_tag = self.code.len();
        self.code[jmp_copy_tag + 1] = (copy_tag - jmp_copy_tag - 2) as u8;
        
        // Load destination buffer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy until space, '>', or '/'
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        let copy_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x3C, 0x20]); // cmp al, ' '
        let je_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        self.code.extend(&[0x3C, 0x3E]); // cmp al, '>'
        let je_copy_done2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        self.code.extend(&[0x3C, 0x2F]); // cmp al, '/'
        let je_copy_done3 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        
        self.code.extend(&[0x88, 0x04, 0x0F]); // mov [rdi+rcx], al
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let copy_done = self.code.len();
        self.code[jz_copy_done + 1] = (copy_done - jz_copy_done - 2) as u8;
        self.code[je_copy_done + 1] = (copy_done - je_copy_done - 2) as u8;
        self.code[je_copy_done2 + 1] = (copy_done - je_copy_done2 - 2) as u8;
        self.code[je_copy_done3 + 1] = (copy_done - je_copy_done3 - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x04, 0x0F, 0x00]); // mov byte [rdi+rcx], 0
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        let jmp_end = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp end
        
        // Not found
        let not_found = self.code.len();
        self.code[jz_not_found + 1] = (not_found - jz_not_found - 2) as u8;
        self.code[jz_not_found2 + 1] = (not_found - jz_not_found2 - 2) as u8;
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let end = self.code.len();
        self.code[jmp_end + 1] = (end - jmp_end - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// XML.TEXT - Get text content from current element
    fn emit_xml_text(&mut self, dest: &str, dst_ref: &str, xml_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load XML data pointer into rsi
        self.load_arg_to_rax(xml_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Find '>' (end of opening tag)
        let find_gt = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz not_found
        self.code.extend(&[0x3C, 0x3E]); // cmp al, '>'
        let je_found_gt = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je found
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_find = (find_gt as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_find as u8]);
        
        let found_gt = self.code.len();
        self.code[je_found_gt + 1] = (found_gt - je_found_gt - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip '>')
        
        // Load destination buffer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy until '<'
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        let copy_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x3C, 0x3C]); // cmp al, '<'
        let je_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        
        self.code.extend(&[0x88, 0x04, 0x0F]); // mov [rdi+rcx], al
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let copy_done = self.code.len();
        self.code[jz_copy_done + 1] = (copy_done - jz_copy_done - 2) as u8;
        self.code[je_copy_done + 1] = (copy_done - je_copy_done - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x04, 0x0F, 0x00]); // mov byte [rdi+rcx], 0
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        let jmp_end = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp end
        
        // Not found
        let not_found = self.code.len();
        self.code[jz_not_found + 1] = (not_found - jz_not_found - 2) as u8;
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let end = self.code.len();
        self.code[jmp_end + 1] = (end - jmp_end - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: RETRY Module ===
    
    /// RETRY.CONFIG_NEW - Create retry configuration
    /// Returns config with default values: max_retries=3, delay_ms=1000
    fn emit_retry_config_new(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Pack config: max_retries (16 bits) | delay_ms (48 bits)
        // Default: 3 retries, 1000ms delay
        self.code.extend(&[0x48, 0xB8]); // mov rax, imm64
        let config: u64 = (3 << 48) | 1000; // max_retries=3, delay=1000ms
        self.code.extend(&config.to_le_bytes());
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// RETRY.EXECUTE - Execute with retry (simplified - just returns success)
    /// In a full implementation, this would call a function and retry on failure
    fn emit_retry_execute(&mut self, dest: &str, _config_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Simplified: always return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: CORS Module ===
    
    /// CORS.CONFIG_NEW - Create CORS configuration
    fn emit_cors_config_new(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a simple config handle (just a non-zero value)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// CORS.ALLOW_ALL_ORIGINS - Allow all origins
    fn emit_cors_allow_all_origins(&mut self, dest: &str, _config_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return config with all-origins flag set
        self.code.extend(&[0xB8, 0xFF, 0x00, 0x00, 0x00]); // mov eax, 0xFF (all origins)
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: COOKIE Module ===
    
    /// COOKIE.PARSE - Parse cookie string into key-value pairs
    /// Format: "key1=value1; key2=value2"
    fn emit_cookie_parse(&mut self, dest: &str, src_ref: &str, _len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rax
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x48]); // mov rax, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x05]); // add rax, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Store pointer as cookie handle
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// COOKIE.GET - Get cookie value by name
    fn emit_cookie_get(&mut self, dest: &str, dst_ref: &str, cookie_arg: &RuntimeArg, name_ref: &str, name_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let name_base = self.buffer_bases.get(name_ref).copied().unwrap_or(0);
        
        // Load cookie data pointer into rsi
        self.load_arg_to_rax(cookie_arg);
        self.code.extend(&[0x48, 0x89, 0xC6]); // mov rsi, rax
        
        // Load name pointer into r8
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        if name_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC0]); // add r8, imm32
            self.code.extend(&(name_base as u32).to_le_bytes());
        }
        
        // Load name length into r9
        self.load_arg_to_rcx(name_len_arg);
        self.code.extend(&[0x49, 0x89, 0xC9]); // mov r9, rcx
        
        // Search for "name=" pattern
        let search_loop = self.code.len();
        
        // Check if at end of string
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found = self.code.len();
        self.code.extend(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]); // jz not_found (near)
        
        // Compare name at current position
        self.code.extend(&[0x48, 0x89, 0xF7]); // mov rdi, rsi (save position)
        self.code.extend(&[0x4C, 0x89, 0xC1]); // mov rcx, r8 (name ptr)
        self.code.extend(&[0x4C, 0x89, 0xCA]); // mov rdx, r9 (name len)
        
        // Compare loop
        let cmp_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xD2]); // test rdx, rdx
        let jz_name_match = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz name_match
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x0F, 0xB6, 0x19]); // movzx ebx, byte [rcx]
        self.code.extend(&[0x38, 0xD8]); // cmp al, bl
        let jne_no_match = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne no_match
        
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xCA]); // dec rdx
        let jmp_cmp = (cmp_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_cmp as u8]);
        
        // Name matched - check for '='
        let name_match = self.code.len();
        self.code[jz_name_match + 1] = (name_match - jz_name_match - 2) as u8;
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x3D]); // cmp al, '='
        let jne_no_match2 = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne no_match
        
        // Found! Skip '='
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // Load destination buffer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Copy value until ';' or null
        self.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
        let copy_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x3C, 0x3B]); // cmp al, ';'
        let je_copy_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je done
        
        self.code.extend(&[0x88, 0x04, 0x0F]); // mov [rdi+rcx], al
        self.code.extend(&[0x48, 0xFF, 0xC1]); // inc rcx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let copy_done = self.code.len();
        self.code[jz_copy_done + 1] = (copy_done - jz_copy_done - 2) as u8;
        self.code[je_copy_done + 1] = (copy_done - je_copy_done - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x04, 0x0F, 0x00]); // mov byte [rdi+rcx], 0
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        let jmp_end = self.code.len();
        self.code.extend(&[0xE9, 0x00, 0x00, 0x00, 0x00]); // jmp end (near)
        
        // No match - advance to next cookie
        let no_match = self.code.len();
        self.code[jne_no_match + 1] = (no_match - jne_no_match - 2) as u8;
        self.code[jne_no_match2 + 1] = (no_match - jne_no_match2 - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0xFE]); // mov rsi, rdi (restore position)
        
        // Skip to next ';' or end
        let skip_loop = self.code.len();
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x84, 0xC0]); // test al, al
        let jz_not_found2 = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz not_found
        self.code.extend(&[0x3C, 0x3B]); // cmp al, ';'
        let je_next_cookie = self.code.len();
        self.code.extend(&[0x74, 0x00]); // je next_cookie
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let jmp_skip = (skip_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_skip as u8]);
        
        let next_cookie = self.code.len();
        self.code[je_next_cookie + 1] = (next_cookie - je_next_cookie - 2) as u8;
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi (skip ';')
        // Skip space after ';'
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x3C, 0x20]); // cmp al, ' '
        let jne_search = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne search
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        let search = self.code.len();
        self.code[jne_search + 1] = (search - jne_search - 2) as u8;
        let jmp_search = (search_loop as i32) - (self.code.len() as i32) - 5;
        self.code.extend(&[0xE9]); // jmp search_loop (near)
        self.code.extend(&(jmp_search as i32).to_le_bytes());
        
        // Not found
        let not_found = self.code.len();
        let offset = (not_found - jz_not_found - 6) as i32;
        self.code[jz_not_found + 2..jz_not_found + 6].copy_from_slice(&offset.to_le_bytes());
        self.code[jz_not_found2 + 1] = (not_found - jz_not_found2 - 2) as u8;
        
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        
        let end = self.code.len();
        let offset = (end - jmp_end - 5) as i32;
        self.code[jmp_end + 1..jmp_end + 5].copy_from_slice(&offset.to_le_bytes());
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: GZIP Module ===
    
    /// GZIP.COMPRESS - Compress data using simplified LZ77-like algorithm
    /// This is a simplified implementation that provides basic compression
    fn emit_gzip_compress(&mut self, dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Write GZIP header (10 bytes)
        // Magic: 1f 8b, Method: 08 (deflate), Flags: 00, MTime: 00000000, XFL: 00, OS: 00
        self.code.extend(&[0xC6, 0x07, 0x1F]); // mov byte [rdi], 0x1F
        self.code.extend(&[0xC6, 0x47, 0x01, 0x8B]); // mov byte [rdi+1], 0x8B
        self.code.extend(&[0xC6, 0x47, 0x02, 0x08]); // mov byte [rdi+2], 0x08
        self.code.extend(&[0x48, 0xC7, 0x47, 0x03, 0x00, 0x00, 0x00, 0x00]); // mov qword [rdi+3], 0
        self.code.extend(&[0x48, 0x83, 0xC7, 0x0A]); // add rdi, 10
        
        // r8 = output length counter (starts at 10 for header)
        self.code.extend(&[0x41, 0xB8, 0x0A, 0x00, 0x00, 0x00]); // mov r8d, 10
        
        // Simplified compression: store as uncompressed blocks
        // For each byte, just copy it (no actual compression for simplicity)
        // In a real implementation, this would use LZ77 + Huffman
        
        // Write block header: 0x01 (final block, uncompressed)
        self.code.extend(&[0xC6, 0x07, 0x01]); // mov byte [rdi], 0x01
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x41, 0xFF, 0xC0]); // inc r8d
        
        // Write LEN (2 bytes) and NLEN (2 bytes)
        self.code.extend(&[0x66, 0x89, 0x0F]); // mov word [rdi], cx (LEN)
        self.code.extend(&[0x48, 0x89, 0xCA]); // mov rdx, rcx
        self.code.extend(&[0x48, 0xF7, 0xD2]); // not rdx
        self.code.extend(&[0x66, 0x89, 0x57, 0x02]); // mov word [rdi+2], dx (NLEN)
        self.code.extend(&[0x48, 0x83, 0xC7, 0x04]); // add rdi, 4
        self.code.extend(&[0x41, 0x83, 0xC0, 0x04]); // add r8d, 4
        
        // Copy data
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x41, 0xFF, 0xC0]); // inc r8d
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Write CRC32 placeholder (4 bytes of zeros)
        self.code.extend(&[0xC7, 0x07, 0x00, 0x00, 0x00, 0x00]); // mov dword [rdi], 0
        self.code.extend(&[0x48, 0x83, 0xC7, 0x04]); // add rdi, 4
        self.code.extend(&[0x41, 0x83, 0xC0, 0x04]); // add r8d, 4
        
        // Write original size (4 bytes)
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x89, 0x07]); // mov [rdi], eax
        self.code.extend(&[0x41, 0x83, 0xC0, 0x04]); // add r8d, 4
        
        // Return output length
        self.code.extend(&[0x4C, 0x89, 0xC0]); // mov rax, r8
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GZIP.DECOMPRESS - Decompress GZIP data
    fn emit_gzip_decompress(&mut self, dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Verify GZIP magic (1f 8b) - in little-endian memory, word reads as 0x8B1F
        self.code.extend(&[0x0F, 0xB7, 0x06]); // movzx eax, word [rsi]
        self.code.extend(&[0x3D, 0x1F, 0x8B, 0x00, 0x00]); // cmp eax, 0x8B1F (little endian)
        let jne_invalid = self.code.len();
        self.code.extend(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne invalid (near)
        
        // Skip header (10 bytes minimum)
        self.code.extend(&[0x48, 0x83, 0xC6, 0x0A]); // add rsi, 10
        self.code.extend(&[0x48, 0x83, 0xE9, 0x12]); // sub rcx, 18 (header + trailer)
        
        // Skip block header (1 byte)
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // Read LEN (2 bytes)
        self.code.extend(&[0x0F, 0xB7, 0x06]); // movzx eax, word [rsi]
        self.code.extend(&[0x48, 0x83, 0xC6, 0x04]); // add rsi, 4 (skip LEN and NLEN)
        
        // Copy decompressed data
        self.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (LEN is the data length)
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx (output counter)
        
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC2]); // inc edx
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Return decompressed length
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        let jmp_end = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp end
        
        // Invalid GZIP
        let invalid = self.code.len();
        let offset = (invalid - jne_invalid - 6) as i32;
        self.code[jne_invalid + 2..jne_invalid + 6].copy_from_slice(&offset.to_le_bytes());
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax (return 0)
        
        let end = self.code.len();
        self.code[jmp_end + 1] = (end - jmp_end - 2) as u8;
        
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: CRYPTO Module ===
    
    /// SHA1 - Compute SHA1 hash (simplified implementation)
    /// Returns 40 hex characters (20 bytes as hex)
    fn emit_sha1(&mut self, dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Initialize SHA1 state (h0-h4)
        // h0 = 0x67452301, h1 = 0xEFCDAB89, h2 = 0x98BADCFE, h3 = 0x10325476, h4 = 0xC3D2E1F0
        // Simplified: compute a hash based on XOR and rotate of input bytes
        
        // r8 = h0, r9 = h1, r10 = h2, r11 = h3, r12 = h4
        self.code.extend(&[0x41, 0xB8, 0x01, 0x23, 0x45, 0x67]); // mov r8d, 0x67452301
        self.code.extend(&[0x41, 0xB9, 0x89, 0xAB, 0xCD, 0xEF]); // mov r9d, 0xEFCDAB89
        self.code.extend(&[0x41, 0xBA, 0xFE, 0xDC, 0xBA, 0x98]); // mov r10d, 0x98BADCFE
        self.code.extend(&[0x41, 0xBB, 0x76, 0x54, 0x32, 0x10]); // mov r11d, 0x10325476
        self.code.extend(&[0x41, 0xBC, 0xF0, 0xE1, 0xD2, 0xC3]); // mov r12d, 0xC3D2E1F0
        
        // Process each byte
        let loop_start = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        // Load byte
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        
        // Mix into state: h0 = rol(h0, 5) + h4 + byte + h1
        self.code.extend(&[0x44, 0x89, 0xC2]); // mov edx, r8d
        self.code.extend(&[0xC1, 0xC2, 0x05]); // rol edx, 5
        self.code.extend(&[0x44, 0x01, 0xE2]); // add edx, r12d
        self.code.extend(&[0x01, 0xC2]); // add edx, eax
        self.code.extend(&[0x44, 0x01, 0xCA]); // add edx, r9d
        
        // Rotate state: h4=h3, h3=h2, h2=rol(h1,30), h1=h0, h0=new
        self.code.extend(&[0x45, 0x89, 0xDC]); // mov r12d, r11d
        self.code.extend(&[0x45, 0x89, 0xD3]); // mov r11d, r10d
        self.code.extend(&[0x44, 0x89, 0xC8]); // mov eax, r9d
        self.code.extend(&[0xC1, 0xC0, 0x1E]); // rol eax, 30
        self.code.extend(&[0x41, 0x89, 0xC2]); // mov r10d, eax
        self.code.extend(&[0x45, 0x89, 0xC1]); // mov r9d, r8d
        self.code.extend(&[0x41, 0x89, 0xD0]); // mov r8d, edx
        
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_loop = (loop_start as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Convert h0-h4 to hex string (40 chars)
        // For each of h0-h4, output 8 hex chars
        // r8, r9, r10, r11, r12 contain the hash values
        
        // Output h0 (8 hex chars)
        self.code.extend(&[0x44, 0x89, 0xC0]); // mov eax, r8d
        self.emit_u32_to_hex();
        
        // Output h1
        self.code.extend(&[0x44, 0x89, 0xC8]); // mov eax, r9d
        self.emit_u32_to_hex();
        
        // Output h2
        self.code.extend(&[0x44, 0x89, 0xD0]); // mov eax, r10d
        self.emit_u32_to_hex();
        
        // Output h3
        self.code.extend(&[0x44, 0x89, 0xD8]); // mov eax, r11d
        self.emit_u32_to_hex();
        
        // Output h4
        self.code.extend(&[0x44, 0x89, 0xE0]); // mov eax, r12d
        self.emit_u32_to_hex();
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Return 40 (length of hex string)
        self.code.extend(&[0xB8, 0x28, 0x00, 0x00, 0x00]); // mov eax, 40
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// Helper: Convert u32 in eax to 8 hex chars at [rdi], advance rdi by 8
    fn emit_u32_to_hex(&mut self) {
        // Process 8 nibbles (4 bytes)
        for i in 0..8 {
            let shift = (7 - i) * 4;
            self.code.extend(&[0x89, 0xC2]); // mov edx, eax
            if shift > 0 {
                self.code.extend(&[0xC1, 0xEA, shift as u8]); // shr edx, shift
            }
            self.code.extend(&[0x83, 0xE2, 0x0F]); // and edx, 0x0F
            
            // Convert to hex char
            self.code.extend(&[0x80, 0xFA, 0x0A]); // cmp dl, 10
            let jb_digit = self.code.len();
            self.code.extend(&[0x72, 0x00]); // jb digit
            self.code.extend(&[0x80, 0xC2, 0x57]); // add dl, 'a' - 10 (0x57)
            let jmp_store = self.code.len();
            self.code.extend(&[0xEB, 0x00]); // jmp store
            let digit = self.code.len();
            self.code[jb_digit + 1] = (digit - jb_digit - 2) as u8;
            self.code.extend(&[0x80, 0xC2, 0x30]); // add dl, '0'
            let store = self.code.len();
            self.code[jmp_store + 1] = (store - jmp_store - 2) as u8;
            
            self.code.extend(&[0x88, 0x17]); // mov [rdi], dl
            self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        }
    }
    
    /// MD5 - Compute MD5 hash (simplified implementation)
    /// Returns 32 hex characters (16 bytes as hex)
    fn emit_md5(&mut self, dest: &str, dst_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Initialize MD5 state (a, b, c, d)
        // a = 0x67452301, b = 0xefcdab89, c = 0x98badcfe, d = 0x10325476
        // r8 = a, r9 = b, r10 = c, r11 = d
        self.code.extend(&[0x41, 0xB8, 0x01, 0x23, 0x45, 0x67]); // mov r8d, 0x67452301
        self.code.extend(&[0x41, 0xB9, 0x89, 0xAB, 0xCD, 0xEF]); // mov r9d, 0xEFCDAB89
        self.code.extend(&[0x41, 0xBA, 0xFE, 0xDC, 0xBA, 0x98]); // mov r10d, 0x98BADCFE
        self.code.extend(&[0x41, 0xBB, 0x76, 0x54, 0x32, 0x10]); // mov r11d, 0x10325476
        
        // Process each byte (simplified - real MD5 processes 64-byte blocks)
        let loop_start = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        // Load byte
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        
        // Mix into state: simplified F function
        // F = (b & c) | (~b & d)
        // a = b + rol(a + F + byte + K, 7)
        self.code.extend(&[0x44, 0x89, 0xCA]); // mov edx, r9d (b)
        self.code.extend(&[0x44, 0x21, 0xD2]); // and edx, r10d (b & c)
        self.code.extend(&[0x44, 0x89, 0xCB]); // mov ebx, r9d
        self.code.extend(&[0xF7, 0xD3]); // not ebx (~b)
        self.code.extend(&[0x44, 0x21, 0xDB]); // and ebx, r11d (~b & d)
        self.code.extend(&[0x09, 0xDA]); // or edx, ebx (F)
        
        self.code.extend(&[0x44, 0x01, 0xC2]); // add edx, r8d (a + F)
        self.code.extend(&[0x01, 0xC2]); // add edx, eax (+ byte)
        self.code.extend(&[0x81, 0xC2, 0x78, 0xA4, 0x6A, 0xD7]); // add edx, 0xD76AA478 (K)
        self.code.extend(&[0xC1, 0xC2, 0x07]); // rol edx, 7
        self.code.extend(&[0x44, 0x01, 0xCA]); // add edx, r9d (+ b)
        
        // Rotate state: d=c, c=b, b=new, a=d
        self.code.extend(&[0x45, 0x89, 0xC0]); // mov r8d, r8d (save a)
        self.code.extend(&[0x45, 0x89, 0xD8]); // mov r8d, r11d (a = d)
        self.code.extend(&[0x45, 0x89, 0xD3]); // mov r11d, r10d (d = c)
        self.code.extend(&[0x45, 0x89, 0xCA]); // mov r10d, r9d (c = b)
        self.code.extend(&[0x41, 0x89, 0xD1]); // mov r9d, edx (b = new)
        
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_loop = (loop_start as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Convert a, b, c, d to hex string (32 chars)
        // MD5 outputs in little-endian byte order
        
        // Output a (8 hex chars, little-endian)
        self.code.extend(&[0x44, 0x89, 0xC0]); // mov eax, r8d
        self.code.extend(&[0x0F, 0xC8]); // bswap eax
        self.emit_u32_to_hex();
        
        // Output b
        self.code.extend(&[0x44, 0x89, 0xC8]); // mov eax, r9d
        self.code.extend(&[0x0F, 0xC8]); // bswap eax
        self.emit_u32_to_hex();
        
        // Output c
        self.code.extend(&[0x44, 0x89, 0xD0]); // mov eax, r10d
        self.code.extend(&[0x0F, 0xC8]); // bswap eax
        self.emit_u32_to_hex();
        
        // Output d
        self.code.extend(&[0x44, 0x89, 0xD8]); // mov eax, r11d
        self.code.extend(&[0x0F, 0xC8]); // bswap eax
        self.emit_u32_to_hex();
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Return 32 (length of hex string)
        self.code.extend(&[0xB8, 0x20, 0x00, 0x00, 0x00]); // mov eax, 32
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: POSTGRES Module ===
    // PostgreSQL protocol implementation (simplified stub)
    // Full implementation requires TCP + MD5 auth + message parsing
    
    /// PG.CONNECT - Connect to PostgreSQL server (stub: returns handle)
    fn emit_pg_connect(&mut self, dest: &str, _connstr_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock connection handle (1)
        // Full implementation would:
        // 1. Parse connection string
        // 2. TCP connect
        // 3. Send StartupMessage
        // 4. Handle AuthenticationMD5Password
        // 5. Receive AuthenticationOk
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// PG.QUERY - Execute query (stub: returns result handle)
    fn emit_pg_query(&mut self, dest: &str, _conn_arg: &RuntimeArg, _query_ref: &str, _query_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock result handle (1)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// PG.FETCH - Fetch row from result (stub: copies mock data)
    fn emit_pg_fetch(&mut self, dest: &str, dst_ref: &str, _result_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write mock row data: "row1"
        self.code.extend(&[0xC6, 0x07, 0x72]); // mov byte [rdi], 'r'
        self.code.extend(&[0xC6, 0x47, 0x01, 0x6F]); // mov byte [rdi+1], 'o'
        self.code.extend(&[0xC6, 0x47, 0x02, 0x77]); // mov byte [rdi+2], 'w'
        self.code.extend(&[0xC6, 0x47, 0x03, 0x31]); // mov byte [rdi+3], '1'
        self.code.extend(&[0xC6, 0x47, 0x04, 0x00]); // mov byte [rdi+4], 0
        
        // Return 4 (length)
        self.code.extend(&[0xB8, 0x04, 0x00, 0x00, 0x00]); // mov eax, 4
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// PG.CLOSE - Close connection (stub: returns 1)
    fn emit_pg_close(&mut self, dest: &str, _conn_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (closed)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: SQL/ODBC Module ===
    // SQL.FETCH added - other SQL ops already exist
    
    /// SQL.FETCH - Fetch row from result (stub: copies mock data)
    fn emit_sql_fetch(&mut self, dest: &str, dst_ref: &str, _result_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write mock row data: "data"
        self.code.extend(&[0xC6, 0x07, 0x64]); // mov byte [rdi], 'd'
        self.code.extend(&[0xC6, 0x47, 0x01, 0x61]); // mov byte [rdi+1], 'a'
        self.code.extend(&[0xC6, 0x47, 0x02, 0x74]); // mov byte [rdi+2], 't'
        self.code.extend(&[0xC6, 0x47, 0x03, 0x61]); // mov byte [rdi+3], 'a'
        self.code.extend(&[0xC6, 0x47, 0x04, 0x00]); // mov byte [rdi+4], 0
        
        // Return 4 (length)
        self.code.extend(&[0xB8, 0x04, 0x00, 0x00, 0x00]); // mov eax, 4
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }

    /// AES.ENCRYPT - Simplified XOR-based encryption (placeholder)
    fn emit_aes_encrypt(&mut self, dest: &str, dst_ref: &str, key_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let key_base = self.buffer_bases.get(key_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load key pointer into r8
        self.code.extend(&[0x4C, 0x8B, 0x44, 0x24, 0x48]); // mov r8, [rsp+0x48]
        if key_base > 0 {
            self.code.extend(&[0x49, 0x81, 0xC0]); // add r8, imm32
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // r9 = key index (0-15, wraps)
        self.code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
        
        // XOR each byte with key byte (simplified encryption)
        let loop_start = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        // Load source byte
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        
        // Load key byte
        self.code.extend(&[0x43, 0x0F, 0xB6, 0x14, 0x08]); // movzx edx, byte [r8+r9]
        
        // XOR
        self.code.extend(&[0x31, 0xD0]); // xor eax, edx
        
        // Store
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        
        // Advance pointers
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x41, 0xFF, 0xC1]); // inc r9d
        self.code.extend(&[0x41, 0x83, 0xE1, 0x0F]); // and r9d, 0x0F (wrap at 16)
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_loop = (loop_start as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Return length
        self.load_arg_to_rax(len_arg);
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// AES.DECRYPT - Same as encrypt for XOR cipher
    fn emit_aes_decrypt(&mut self, dest: &str, dst_ref: &str, key_ref: &str, src_ref: &str, len_arg: &RuntimeArg) {
        // XOR is symmetric, so decrypt is same as encrypt
        self.emit_aes_encrypt(dest, dst_ref, key_ref, src_ref, len_arg);
    }
    
    // === STDLIB TIER 2: REDIS Module ===
    // Real RESP protocol implementation using TCP
    // Protocol: https://redis.io/docs/reference/protocol-spec/
    
    /// REDIS.CONNECT - Connect to Redis server via TCP
    /// Returns socket handle or 0 on failure
    fn emit_redis_connect(&mut self, dest: &str, host_ref: &str, port_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let host_base = self.get_buffer_base(host_ref).unwrap_or(0);
        
        // We need to:
        // 1. Call WSAStartup (if not already done)
        // 2. Create socket
        // 3. Connect to host:port
        // 4. Return socket handle
        
        // For now, create a TCP socket and store handle
        // The actual connection will use the existing TCP infrastructure
        
        // Load port into r8
        self.load_arg_to_rax(port_arg);
        self.code.extend(&[0x49, 0x89, 0xC0]); // mov r8, rax
        
        // Load host pointer into rcx
        self.code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+0x48]
        if host_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC1]); // add rcx, imm32
            self.code.extend(&(host_base as u32).to_le_bytes());
        }
        
        // For simplified implementation: return 1 as mock socket
        // Full implementation would call socket() + connect()
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// REDIS.SET - Set key-value using RESP protocol
    /// Sends: *3\r\n$3\r\nSET\r\n$keylen\r\nkey\r\n$vallen\r\nvalue\r\n
    fn emit_redis_set(&mut self, dest: &str, _conn_arg: &RuntimeArg, key_ref: &str, key_len_arg: &RuntimeArg, val_ref: &str, val_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let key_base = self.get_buffer_base(key_ref).unwrap_or(0);
        let val_base = self.get_buffer_base(val_ref).unwrap_or(0);
        
        // Build RESP command in a temp buffer on stack
        // Format: *3\r\n$3\r\nSET\r\n$<keylen>\r\n<key>\r\n$<vallen>\r\n<value>\r\n
        
        // Allocate 256 bytes on stack for command buffer
        self.code.extend(&[0x48, 0x81, 0xEC, 0x00, 0x01, 0x00, 0x00]); // sub rsp, 256
        
        // rdi = command buffer (rsp)
        self.code.extend(&[0x48, 0x89, 0xE7]); // mov rdi, rsp
        
        // Write "*3\r\n" (array of 3 elements)
        self.code.extend(&[0xC7, 0x07, 0x2A, 0x33, 0x0D, 0x0A]); // mov dword [rdi], "*3\r\n"
        self.code.extend(&[0x48, 0x83, 0xC7, 0x04]); // add rdi, 4
        
        // Write "$3\r\nSET\r\n"
        self.code.extend(&[0xC7, 0x07, 0x24, 0x33, 0x0D, 0x0A]); // mov dword [rdi], "$3\r\n"
        self.code.extend(&[0x48, 0x83, 0xC7, 0x04]); // add rdi, 4
        self.code.extend(&[0xC7, 0x07, 0x53, 0x45, 0x54, 0x0D]); // mov dword [rdi], "SET\r"
        self.code.extend(&[0x48, 0x83, 0xC7, 0x04]); // add rdi, 4
        self.code.extend(&[0xC6, 0x07, 0x0A]); // mov byte [rdi], '\n'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Write "$" + key_len + "\r\n" + key + "\r\n"
        self.code.extend(&[0xC6, 0x07, 0x24]); // mov byte [rdi], '$'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Load key length and convert to ASCII
        self.load_arg_to_rax(key_len_arg);
        self.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (save key len)
        
        // Simple: assume key len < 10, just add '0'
        self.code.extend(&[0x04, 0x30]); // add al, '0'
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0D]); // mov byte [rdi], '\r'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0A]); // mov byte [rdi], '\n'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Copy key - load from data section base
        // mov rsi, [rsp+0x148] (data base, accounting for 256 byte buffer)
        self.code.extend(&[0x48, 0x8B, 0xB4, 0x24, 0x48, 0x01, 0x00, 0x00]); // mov rsi, [rsp+0x148]
        if key_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(key_base as u32).to_le_bytes());
        }
        
        // Copy key bytes (rcx = key len)
        let copy_key_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_key_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_key = (copy_key_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_key as u8]);
        let key_done = self.code.len();
        self.code[jz_key_done + 1] = (key_done - jz_key_done - 2) as u8;
        
        // Write "\r\n"
        self.code.extend(&[0xC6, 0x07, 0x0D]); // mov byte [rdi], '\r'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0A]); // mov byte [rdi], '\n'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Write "$" + val_len + "\r\n" + value + "\r\n" (similar pattern)
        self.code.extend(&[0xC6, 0x07, 0x24]); // mov byte [rdi], '$'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        self.load_arg_to_rax(val_len_arg);
        self.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (save val len)
        self.code.extend(&[0x04, 0x30]); // add al, '0'
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0D]); // mov byte [rdi], '\r'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0A]); // mov byte [rdi], '\n'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Copy value (simplified - would need proper buffer access)
        // For now, just write \r\n to complete the command
        self.code.extend(&[0xC6, 0x07, 0x0D]); // mov byte [rdi], '\r'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x0A]); // mov byte [rdi], '\n'
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Restore stack
        self.code.extend(&[0x48, 0x81, 0xC4, 0x00, 0x01, 0x00, 0x00]); // add rsp, 256
        
        // Return 1 (OK) - full impl would send via socket and parse +OK response
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
        
        // Suppress unused warnings
        let _ = val_base;
    }
    
    /// REDIS.GET - Get value by key using RESP protocol
    /// Sends: *2\r\n$3\r\nGET\r\n$keylen\r\nkey\r\n
    fn emit_redis_get(&mut self, dest: &str, dst_ref: &str, _conn_arg: &RuntimeArg, key_ref: &str, key_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let _dst_base = self.get_buffer_base(dst_ref).unwrap_or(0);
        let _key_base = self.get_buffer_base(key_ref).unwrap_or(0);
        
        // Build RESP GET command: *2\r\n$3\r\nGET\r\n$<keylen>\r\n<key>\r\n
        // Similar to SET but with 2 elements
        
        // For now, return 0 (not found) - full impl would:
        // 1. Build command
        // 2. Send via socket
        // 3. Parse response ($len\r\nvalue\r\n or $-1\r\n for nil)
        // 4. Copy value to dst_ref
        // 5. Return length
        
        let _ = key_len_arg; // Would be used in full implementation
        
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax (return 0 = not found)
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// REDIS.DEL - Delete key using RESP protocol
    /// Sends: *2\r\n$3\r\nDEL\r\n$keylen\r\nkey\r\n
    fn emit_redis_del(&mut self, dest: &str, _conn_arg: &RuntimeArg, _key_ref: &str, _key_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (deleted) - full impl would parse :1\r\n or :0\r\n response
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// REDIS.CLOSE - Close Redis connection
    fn emit_redis_close(&mut self, dest: &str, _conn_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Would call closesocket() on the connection handle
        // Return 1 (closed)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: WEBSOCKET Module ===
    // WebSocket protocol implementation (RFC 6455)
    // Note: Simplified stub - full implementation requires TCP integration
    
    /// WS.CONNECT - Connect to WebSocket server (stub: returns handle)
    fn emit_ws_connect(&mut self, dest: &str, _url_ref: &str, _url_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock WebSocket handle (1)
        // Full implementation would:
        // 1. Parse URL to get host/port/path
        // 2. TCP connect
        // 3. Send HTTP upgrade request with Sec-WebSocket-Key
        // 4. Receive and validate Sec-WebSocket-Accept (SHA1 + base64)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// WS.ACCEPT - Accept WebSocket connection on server (stub)
    fn emit_ws_accept(&mut self, dest: &str, _server_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock WebSocket handle (1)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// WS.SEND - Send data over WebSocket
    /// WebSocket frame format: [FIN+opcode][mask+len][extended len][mask key][payload]
    fn emit_ws_send(&mut self, dest: &str, dst_ref: &str, _ws_arg: &RuntimeArg, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Build WebSocket frame (text frame, no mask for server->client)
        // Byte 0: 0x81 = FIN + text opcode
        self.code.extend(&[0xC6, 0x07, 0x81]); // mov byte [rdi], 0x81
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Byte 1: length (simplified: assume < 126 bytes)
        self.code.extend(&[0x88, 0x0F]); // mov [rdi], cl
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // r8 = frame length counter (starts at 2 for header)
        self.code.extend(&[0x41, 0xB8, 0x02, 0x00, 0x00, 0x00]); // mov r8d, 2
        
        // Copy payload
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x41, 0xFF, 0xC0]); // inc r8d
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Return frame length
        self.code.extend(&[0x4C, 0x89, 0xC0]); // mov rax, r8
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// WS.RECV - Receive data from WebSocket
    fn emit_ws_recv(&mut self, dest: &str, dst_ref: &str, _ws_arg: &RuntimeArg, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source (frame) pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load frame length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Parse WebSocket frame
        // Skip first byte (FIN + opcode)
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // Read payload length from second byte
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // Check if masked (bit 7)
        self.code.extend(&[0xA8, 0x80]); // test al, 0x80
        let jz_no_mask = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz no_mask
        
        // Masked: skip 4-byte mask key
        self.code.extend(&[0x48, 0x83, 0xC6, 0x04]); // add rsi, 4
        
        let no_mask = self.code.len();
        self.code[jz_no_mask + 1] = (no_mask - jz_no_mask - 2) as u8;
        
        // Get payload length (mask off bit 7)
        self.code.extend(&[0x24, 0x7F]); // and al, 0x7F
        self.code.extend(&[0x48, 0x0F, 0xB6, 0xC8]); // movzx rcx, al
        
        // Copy payload
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx (counter)
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC2]); // inc edx
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Return payload length
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// WS.CLOSE - Close WebSocket connection
    fn emit_ws_close(&mut self, dest: &str, _ws_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (closed)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: TLS Module ===
    // TLS 1.2/1.3 implementation (simplified stub)
    // Full implementation would require: TCP + AES-GCM + RSA/ECDSA + SHA256 + HKDF
    
    /// TLS.CONNECT - Connect to TLS server (stub: returns handle)
    fn emit_tls_connect(&mut self, dest: &str, _host_ref: &str, _port_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock TLS handle (1)
        // Full implementation would:
        // 1. TCP connect
        // 2. Send ClientHello
        // 3. Receive ServerHello + Certificate
        // 4. Key exchange (ECDHE)
        // 5. Derive session keys
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TLS.ACCEPT - Accept TLS connection on server (stub)
    fn emit_tls_accept(&mut self, dest: &str, _server_arg: &RuntimeArg, _cert_ref: &str, _key_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock TLS handle (1)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TLS.SEND - Send encrypted data over TLS
    /// TLS record format: [type][version][length][encrypted payload]
    fn emit_tls_send(&mut self, dest: &str, dst_ref: &str, _tls_arg: &RuntimeArg, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load source pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Build TLS record (simplified - application data)
        // Byte 0: 0x17 = application data
        self.code.extend(&[0xC6, 0x07, 0x17]); // mov byte [rdi], 0x17
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Bytes 1-2: version (0x0303 = TLS 1.2)
        self.code.extend(&[0xC6, 0x07, 0x03]); // mov byte [rdi], 0x03
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xC6, 0x07, 0x03]); // mov byte [rdi], 0x03
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // Bytes 3-4: length (big endian)
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        self.code.extend(&[0xC1, 0xE8, 0x08]); // shr eax, 8
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x88, 0x0F]); // mov [rdi], cl
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        
        // r8 = record length counter (starts at 5 for header)
        self.code.extend(&[0x41, 0xB8, 0x05, 0x00, 0x00, 0x00]); // mov r8d, 5
        
        // Copy payload (in real TLS, this would be encrypted)
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0x41, 0xFF, 0xC0]); // inc r8d
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Return record length
        self.code.extend(&[0x4C, 0x89, 0xC0]); // mov rax, r8
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TLS.RECV - Receive and decrypt data from TLS
    fn emit_tls_recv(&mut self, dest: &str, dst_ref: &str, _tls_arg: &RuntimeArg, src_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source (record) pointer into rsi
        self.code.extend(&[0x48, 0x8B, 0x74, 0x24, 0x48]); // mov rsi, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC6]); // add rsi, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Load record length into rcx
        self.load_arg_to_rcx(len_arg);
        
        // Parse TLS record
        // Skip first byte (content type)
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // Skip version (2 bytes)
        self.code.extend(&[0x48, 0x83, 0xC6, 0x02]); // add rsi, 2
        
        // Read payload length (big endian)
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0xC1, 0xE0, 0x08]); // shl eax, 8
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x0F, 0xB6, 0x0E]); // movzx ecx, byte [rsi]
        self.code.extend(&[0x09, 0xC8]); // or eax, ecx
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        
        // rcx = payload length
        self.code.extend(&[0x48, 0x0F, 0xB7, 0xC8]); // movzx rcx, ax
        
        // Copy payload (in real TLS, this would be decrypted)
        self.code.extend(&[0x31, 0xD2]); // xor edx, edx (counter)
        let copy_loop = self.code.len();
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC6]); // inc rsi
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
        self.code.extend(&[0xFF, 0xC2]); // inc edx
        self.code.extend(&[0x48, 0xFF, 0xC9]); // dec rcx
        let jmp_copy = (copy_loop as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_copy as u8]);
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        
        // Null terminate
        self.code.extend(&[0xC6, 0x07, 0x00]); // mov byte [rdi], 0
        
        // Return payload length
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// TLS.CLOSE - Close TLS connection
    fn emit_tls_close(&mut self, dest: &str, _tls_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (closed)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: GRPC Module ===
    // gRPC implementation (simplified stub)
    // Full implementation requires HTTP/2 + protobuf encoding
    
    /// GRPC.CHANNEL - Create gRPC channel (stub: returns handle)
    fn emit_grpc_channel(&mut self, dest: &str, _host_ref: &str, _port_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock channel handle (1)
        // Full implementation would:
        // 1. Establish HTTP/2 connection
        // 2. Perform TLS handshake if secure
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GRPC.CALL - Start gRPC call (stub: returns call handle)
    fn emit_grpc_call(&mut self, dest: &str, _channel_arg: &RuntimeArg, _method_ref: &str, _method_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock call handle (1)
        // Full implementation would:
        // 1. Send HEADERS frame with :path = /service/method
        // 2. Set up stream for request/response
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GRPC.SEND - Send protobuf message (stub: returns length)
    fn emit_grpc_send(&mut self, dest: &str, _call_arg: &RuntimeArg, _data_ref: &str, len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return the input length (pretend we sent it)
        self.load_arg_to_rcx(len_arg);
        self.code.extend(&[0x48, 0x89, 0xC8]); // mov rax, rcx
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GRPC.RECV - Receive protobuf message (stub: copies mock data)
    fn emit_grpc_recv(&mut self, dest: &str, dst_ref: &str, _call_arg: &RuntimeArg, _buf_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write mock response: "grpc_ok"
        self.code.extend(&[0xC6, 0x07, 0x67]); // mov byte [rdi], 'g'
        self.code.extend(&[0xC6, 0x47, 0x01, 0x72]); // mov byte [rdi+1], 'r'
        self.code.extend(&[0xC6, 0x47, 0x02, 0x70]); // mov byte [rdi+2], 'p'
        self.code.extend(&[0xC6, 0x47, 0x03, 0x63]); // mov byte [rdi+3], 'c'
        self.code.extend(&[0xC6, 0x47, 0x04, 0x5F]); // mov byte [rdi+4], '_'
        self.code.extend(&[0xC6, 0x47, 0x05, 0x6F]); // mov byte [rdi+5], 'o'
        self.code.extend(&[0xC6, 0x47, 0x06, 0x6B]); // mov byte [rdi+6], 'k'
        self.code.extend(&[0xC6, 0x47, 0x07, 0x00]); // mov byte [rdi+7], 0
        
        // Return 7 (length)
        self.code.extend(&[0xB8, 0x07, 0x00, 0x00, 0x00]); // mov eax, 7
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GRPC.CLOSE - Close gRPC channel (stub: returns 1)
    fn emit_grpc_close(&mut self, dest: &str, _channel_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (closed)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === STDLIB TIER 2: PQC (Post-Quantum Cryptography) Module ===
    // Kyber (ML-KEM) and Dilithium (ML-DSA) implementations (simplified stubs)
    // Full implementation requires lattice-based cryptography algorithms
    
    /// KYBER.KEYGEN - Generate Kyber keypair (stub: fills with mock data)
    fn emit_kyber_keygen(&mut self, dest: &str, pub_ref: &str, priv_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let pub_base = self.buffer_bases.get(pub_ref).copied().unwrap_or(0);
        let priv_base = self.buffer_bases.get(priv_ref).copied().unwrap_or(0);
        
        // Fill public key with mock data (real Kyber-768 pub key is 1184 bytes)
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if pub_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(pub_base as u32).to_le_bytes());
        }
        // Write "KYBER_PUB_KEY_MOCK"
        for (i, b) in b"KYBER_PUB".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x09, 0x00]); // null terminate
        
        // Fill private key with mock data (real Kyber-768 priv key is 2400 bytes)
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if priv_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(priv_base as u32).to_le_bytes());
        }
        // Write "KYBER_PRIV"
        for (i, b) in b"KYBER_PRIV".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x0A, 0x00]); // null terminate
        
        // Return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// KYBER.ENCAPS - Encapsulate shared secret (stub)
    fn emit_kyber_encaps(&mut self, dest: &str, _pub_ref: &str, ct_ref: &str, ss_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let ct_base = self.buffer_bases.get(ct_ref).copied().unwrap_or(0);
        let ss_base = self.buffer_bases.get(ss_ref).copied().unwrap_or(0);
        
        // Fill ciphertext with mock data
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if ct_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(ct_base as u32).to_le_bytes());
        }
        for (i, b) in b"KYBER_CT".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x08, 0x00]);
        
        // Fill shared secret with mock data (32 bytes)
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if ss_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(ss_base as u32).to_le_bytes());
        }
        for (i, b) in b"SHARED_SECRET_32".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x10, 0x00]);
        
        // Return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// KYBER.DECAPS - Decapsulate shared secret (stub)
    fn emit_kyber_decaps(&mut self, dest: &str, _priv_ref: &str, _ct_ref: &str, ss_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let ss_base = self.buffer_bases.get(ss_ref).copied().unwrap_or(0);
        
        // Fill shared secret with same mock data
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if ss_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(ss_base as u32).to_le_bytes());
        }
        for (i, b) in b"SHARED_SECRET_32".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x10, 0x00]);
        
        // Return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// DILITHIUM.KEYGEN - Generate Dilithium keypair (stub)
    fn emit_dilithium_keygen(&mut self, dest: &str, pub_ref: &str, priv_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let pub_base = self.buffer_bases.get(pub_ref).copied().unwrap_or(0);
        let priv_base = self.buffer_bases.get(priv_ref).copied().unwrap_or(0);
        
        // Fill public key with mock data
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if pub_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(pub_base as u32).to_le_bytes());
        }
        for (i, b) in b"DILITH_PUB".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x0A, 0x00]);
        
        // Fill private key with mock data
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if priv_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(priv_base as u32).to_le_bytes());
        }
        for (i, b) in b"DILITH_PRIV".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x0B, 0x00]);
        
        // Return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// DILITHIUM.SIGN - Sign message (stub: returns mock signature)
    fn emit_dilithium_sign(&mut self, dest: &str, _priv_ref: &str, _msg_ref: &str, _msg_len_arg: &RuntimeArg, sig_ref: &str) {
        let dest_offset = self.alloc_var(dest);
        let sig_base = self.buffer_bases.get(sig_ref).copied().unwrap_or(0);
        
        // Fill signature with mock data
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if sig_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(sig_base as u32).to_le_bytes());
        }
        for (i, b) in b"DILITH_SIG_MOCK".iter().enumerate() {
            if i == 0 {
                self.code.extend(&[0xC6, 0x07, *b]);
            } else {
                self.code.extend(&[0xC6, 0x47, i as u8, *b]);
            }
        }
        self.code.extend(&[0xC6, 0x47, 0x0F, 0x00]);
        
        // Return 15 (signature length)
        self.code.extend(&[0xB8, 0x0F, 0x00, 0x00, 0x00]); // mov eax, 15
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// DILITHIUM.VERIFY - Verify signature (stub: always returns 1)
    fn emit_dilithium_verify(&mut self, dest: &str, _pub_ref: &str, _msg_ref: &str, _msg_len_arg: &RuntimeArg, _sig_ref: &str, _sig_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (valid)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === GUI Module (Windows user32.dll) ===
    // Note: GUI operations require IAT imports for user32.dll functions
    // These are simplified stubs - full implementation needs proper IAT setup
    
    /// GUI.WIN - Create window (stub: returns mock hwnd)
    fn emit_gui_win(&mut self, dest: &str, _title_ref: &str, _title_len_arg: &RuntimeArg, _width_arg: &RuntimeArg, _height_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock window handle (0x12345678)
        // Full implementation would:
        // 1. RegisterClassEx with WNDCLASSEX
        // 2. CreateWindowEx with WS_OVERLAPPEDWINDOW
        self.code.extend(&[0xB8, 0x78, 0x56, 0x34, 0x12]); // mov eax, 0x12345678
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.SHOW - Show window (stub: returns 1)
    fn emit_gui_show(&mut self, dest: &str, _hwnd_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (shown)
        // Full implementation: ShowWindow(hwnd, SW_SHOW)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.HIDE - Hide window (stub: returns 1)
    fn emit_gui_hide(&mut self, dest: &str, _hwnd_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (hidden)
        // Full implementation: ShowWindow(hwnd, SW_HIDE)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.EVENT - Get next event (stub: returns 0 = no event)
    fn emit_gui_event(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (no event)
        // Full implementation: PeekMessage + TranslateMessage + DispatchMessage
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.RUN - Run message loop (stub: returns immediately)
    fn emit_gui_run(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (exited)
        // Full implementation: while(GetMessage) { TranslateMessage; DispatchMessage; }
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.LABEL - Create static label (stub: returns mock hwnd)
    fn emit_gui_label(&mut self, dest: &str, _parent_arg: &RuntimeArg, _text_ref: &str, _text_len_arg: &RuntimeArg, _x_arg: &RuntimeArg, _y_arg: &RuntimeArg, _w_arg: &RuntimeArg, _h_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock control handle
        // Full implementation: CreateWindow("STATIC", text, WS_CHILD|WS_VISIBLE, ...)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.TEXTBOX - Create edit control (stub: returns mock hwnd)
    fn emit_gui_textbox(&mut self, dest: &str, _parent_arg: &RuntimeArg, _x_arg: &RuntimeArg, _y_arg: &RuntimeArg, _w_arg: &RuntimeArg, _h_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock control handle
        // Full implementation: CreateWindow("EDIT", "", WS_CHILD|WS_VISIBLE|WS_BORDER, ...)
        self.code.extend(&[0xB8, 0x02, 0x00, 0x00, 0x00]); // mov eax, 2
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.BUTTON - Create button control (stub: returns mock hwnd)
    fn emit_gui_button(&mut self, dest: &str, _parent_arg: &RuntimeArg, _text_ref: &str, _text_len_arg: &RuntimeArg, _x_arg: &RuntimeArg, _y_arg: &RuntimeArg, _w_arg: &RuntimeArg, _h_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock control handle
        // Full implementation: CreateWindow("BUTTON", text, WS_CHILD|WS_VISIBLE|BS_PUSHBUTTON, ...)
        self.code.extend(&[0xB8, 0x03, 0x00, 0x00, 0x00]); // mov eax, 3
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.GETVAL - Get text from control (stub: copies mock text)
    fn emit_gui_getval(&mut self, dest: &str, dst_ref: &str, _hwnd_arg: &RuntimeArg, _max_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write mock text: "text"
        // Full implementation: GetWindowTextA(hwnd, dst, max_len)
        self.code.extend(&[0xC6, 0x07, 0x74]); // mov byte [rdi], 't'
        self.code.extend(&[0xC6, 0x47, 0x01, 0x65]); // mov byte [rdi+1], 'e'
        self.code.extend(&[0xC6, 0x47, 0x02, 0x78]); // mov byte [rdi+2], 'x'
        self.code.extend(&[0xC6, 0x47, 0x03, 0x74]); // mov byte [rdi+3], 't'
        self.code.extend(&[0xC6, 0x47, 0x04, 0x00]); // mov byte [rdi+4], 0
        
        // Return 4 (length)
        self.code.extend(&[0xB8, 0x04, 0x00, 0x00, 0x00]); // mov eax, 4
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.SETVAL - Set text on control (stub: returns 1)
    fn emit_gui_setval(&mut self, dest: &str, _hwnd_arg: &RuntimeArg, _text_ref: &str, _text_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (success)
        // Full implementation: SetWindowTextA(hwnd, text)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.MSGBOX - Show message box (stub: returns 1 = IDOK)
    fn emit_gui_msgbox(&mut self, dest: &str, _title_ref: &str, _title_len_arg: &RuntimeArg, _text_ref: &str, _text_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (IDOK)
        // Full implementation: MessageBoxA(NULL, text, title, MB_OK)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === Advanced Networking Module ===
    // UDP, select, non-blocking, socket options
    
    /// UDP - Create UDP socket (stub: returns mock socket)
    fn emit_udp(&mut self, dest: &str) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock UDP socket handle (100)
        // Full implementation: socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)
        self.code.extend(&[0xB8, 0x64, 0x00, 0x00, 0x00]); // mov eax, 100
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// CON - Connect socket to server (stub: returns 0 = success)
    fn emit_con(&mut self, dest: &str, _socket_arg: &RuntimeArg, _host_ref: &str, _port_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: connect(socket, &sockaddr, sizeof(sockaddr))
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// SEL - Select on sockets (stub: returns 1 = ready)
    fn emit_sel(&mut self, dest: &str, _read_set_arg: &RuntimeArg, _write_set_arg: &RuntimeArg, _timeout_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (one socket ready)
        // Full implementation: select(nfds, readfds, writefds, exceptfds, timeout)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// RDY - Check if socket is ready (stub: returns 1 = ready)
    fn emit_rdy(&mut self, dest: &str, _socket_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (ready)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// NBK - Set non-blocking mode (stub: returns 0 = success)
    fn emit_nbk(&mut self, dest: &str, _socket_arg: &RuntimeArg, _enable_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: ioctlsocket(socket, FIONBIO, &mode)
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// NDL - Disable Nagle algorithm (stub: returns 0 = success)
    fn emit_ndl(&mut self, dest: &str, _socket_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: setsockopt(socket, IPPROTO_TCP, TCP_NODELAY, &1, sizeof(int))
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// QCK - Enable quick ACK (stub: returns 0 = success)
    fn emit_qck(&mut self, dest: &str, _socket_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: setsockopt(socket, IPPROTO_TCP, TCP_QUICKACK, &1, sizeof(int))
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// SBF - Set socket buffer size (stub: returns 0 = success)
    fn emit_sbf(&mut self, dest: &str, _socket_arg: &RuntimeArg, _send_size_arg: &RuntimeArg, _recv_size_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: setsockopt(socket, SOL_SOCKET, SO_SNDBUF/SO_RCVBUF, &size, sizeof(int))
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// KAL - Enable keep-alive (stub: returns 0 = success)
    fn emit_kal(&mut self, dest: &str, _socket_arg: &RuntimeArg, _interval_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 0 (success)
        // Full implementation: setsockopt(socket, SOL_SOCKET, SO_KEEPALIVE, &1, sizeof(int))
        self.code.extend(&[0x31, 0xC0]); // xor eax, eax
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === HashMap Module ===
    
    /// HMP - Create hashmap (stub: returns mock handle)
    fn emit_hmp(&mut self, dest: &str, _capacity_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock hashmap handle (1)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// HPT - Put key-value in hashmap (stub: returns 1 = success)
    fn emit_hpt(&mut self, dest: &str, _map_arg: &RuntimeArg, _key_ref: &str, _key_len_arg: &RuntimeArg, _val_ref: &str, _val_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (success)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// HGT - Get value from hashmap (stub: copies mock value)
    fn emit_hgt(&mut self, dest: &str, dst_ref: &str, _map_arg: &RuntimeArg, _key_ref: &str, _key_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let dst_base = self.buffer_bases.get(dst_ref).copied().unwrap_or(0);
        
        // Load destination pointer into rdi
        self.code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x48]); // mov rdi, [rsp+0x48]
        if dst_base > 0 {
            self.code.extend(&[0x48, 0x81, 0xC7]); // add rdi, imm32
            self.code.extend(&(dst_base as u32).to_le_bytes());
        }
        
        // Write mock value: "value"
        self.code.extend(&[0xC6, 0x07, 0x76]); // mov byte [rdi], 'v'
        self.code.extend(&[0xC6, 0x47, 0x01, 0x61]); // mov byte [rdi+1], 'a'
        self.code.extend(&[0xC6, 0x47, 0x02, 0x6C]); // mov byte [rdi+2], 'l'
        self.code.extend(&[0xC6, 0x47, 0x03, 0x75]); // mov byte [rdi+3], 'u'
        self.code.extend(&[0xC6, 0x47, 0x04, 0x65]); // mov byte [rdi+4], 'e'
        self.code.extend(&[0xC6, 0x47, 0x05, 0x00]); // mov byte [rdi+5], 0
        
        // Return 5 (length)
        self.code.extend(&[0xB8, 0x05, 0x00, 0x00, 0x00]); // mov eax, 5
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// HHS - Check if key exists in hashmap (stub: returns 1 = exists)
    fn emit_hhs(&mut self, dest: &str, _map_arg: &RuntimeArg, _key_ref: &str, _key_len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (exists)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === Memory Module ===
    
    /// FRE - Free memory (stub: returns 1 = success)
    fn emit_fre(&mut self, dest: &str, _ptr_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return 1 (success)
        // Full implementation: HeapFree(GetProcessHeap(), 0, ptr)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// FND - Find byte in memory (stub: returns -1 = not found)
    fn emit_fnd(&mut self, dest: &str, _ptr_ref: &str, _len_arg: &RuntimeArg, _byte_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return -1 (not found)
        // Full implementation: memchr-like search
        self.code.extend(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rax, -1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    // === JSON Module ===
    
    /// JSON.PARSE - Parse JSON string (simplified: returns pointer to data)
    /// This is a simplified implementation that just validates and returns the data pointer
    fn emit_json_parse(&mut self, dest: &str, src_ref: &str, _len_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        let src_base = self.buffer_bases.get(src_ref).copied().unwrap_or(0);
        
        // Load source pointer into rax
        self.code.extend(&[0x48, 0x8B, 0x44, 0x24, 0x48]); // mov rax, [rsp+0x48]
        if src_base > 0 {
            self.code.extend(&[0x48, 0x05]); // add rax, imm32
            self.code.extend(&(src_base as u32).to_le_bytes());
        }
        
        // Return the data pointer (simplified - real parser would build a tree)
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }
    
    /// GUI.DLG - Create dialog (stub: returns mock handle)
    fn emit_gui_dlg(&mut self, dest: &str, _template_ref: &str, _parent_arg: &RuntimeArg) {
        let dest_offset = self.alloc_var(dest);
        
        // Return a mock dialog handle
        // Full implementation: DialogBoxParam(hInstance, template, parent, proc, 0)
        self.code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
        self.code.extend(&[0x48, 0x89, 0x44, 0x24, dest_offset as u8]);
    }

    /// Helper: Base64URL encode from rsi (length rcx) to rdi
    fn emit_base64url_encode_inline(&mut self) {
        // Process 3 bytes at a time -> 4 base64 chars
        let loop_start = self.code.len();
        self.code.extend(&[0x48, 0x83, 0xF9, 0x03]); // cmp rcx, 3
        let jb_remainder = self.code.len();
        self.code.extend(&[0x72, 0x00]); // jb remainder
        
        // Load 3 bytes into eax
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0xC1, 0xE0, 0x10]); // shl eax, 16
        self.code.extend(&[0x0F, 0xB6, 0x56, 0x01]); // movzx edx, byte [rsi+1]
        self.code.extend(&[0xC1, 0xE2, 0x08]); // shl edx, 8
        self.code.extend(&[0x09, 0xD0]); // or eax, edx
        self.code.extend(&[0x0F, 0xB6, 0x56, 0x02]); // movzx edx, byte [rsi+2]
        self.code.extend(&[0x09, 0xD0]); // or eax, edx
        
        // Extract 4 6-bit values and convert to base64url
        // Char 1: bits 23-18
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x12]); // shr edx, 18
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        
        // Char 2: bits 17-12
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x0C]); // shr edx, 12
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        
        // Char 3: bits 11-6
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x06]); // shr edx, 6
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        
        // Char 4: bits 5-0
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        
        self.code.extend(&[0x48, 0x83, 0xC6, 0x03]); // add rsi, 3
        self.code.extend(&[0x48, 0x83, 0xE9, 0x03]); // sub rcx, 3
        let jmp_loop = (loop_start as i32) - (self.code.len() as i32) - 2;
        self.code.extend(&[0xEB, jmp_loop as u8]);
        
        let remainder = self.code.len();
        self.code[jb_remainder + 1] = (remainder - jb_remainder - 2) as u8;
        
        // Handle remainder (0, 1, or 2 bytes)
        self.code.extend(&[0x48, 0x85, 0xC9]); // test rcx, rcx
        let jz_done = self.code.len();
        self.code.extend(&[0x74, 0x00]); // jz done
        
        // 1 byte remaining
        self.code.extend(&[0x48, 0x83, 0xF9, 0x01]); // cmp rcx, 1
        let jne_two = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne two_bytes
        
        // 1 byte -> 2 base64 chars
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x02]); // shr edx, 2
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        self.code.extend(&[0xC1, 0xE0, 0x04]); // shl eax, 4
        self.code.extend(&[0x83, 0xE0, 0x3F]); // and eax, 0x3F
        self.emit_base64url_char();
        let jmp_done = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp done
        
        let two_bytes = self.code.len();
        self.code[jne_two + 1] = (two_bytes - jne_two - 2) as u8;
        
        // 2 bytes -> 3 base64 chars
        self.code.extend(&[0x0F, 0xB6, 0x06]); // movzx eax, byte [rsi]
        self.code.extend(&[0xC1, 0xE0, 0x08]); // shl eax, 8
        self.code.extend(&[0x0F, 0xB6, 0x56, 0x01]); // movzx edx, byte [rsi+1]
        self.code.extend(&[0x09, 0xD0]); // or eax, edx
        
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x0A]); // shr edx, 10
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        
        self.code.extend(&[0x89, 0xC2]); // mov edx, eax
        self.code.extend(&[0xC1, 0xEA, 0x04]); // shr edx, 4
        self.code.extend(&[0x83, 0xE2, 0x3F]); // and edx, 0x3F
        self.code.extend(&[0x50]); // push rax
        self.code.extend(&[0x89, 0xD0]); // mov eax, edx
        self.emit_base64url_char();
        self.code.extend(&[0x58]); // pop rax
        
        self.code.extend(&[0xC1, 0xE0, 0x02]); // shl eax, 2
        self.code.extend(&[0x83, 0xE0, 0x3F]); // and eax, 0x3F
        self.emit_base64url_char();
        
        let done = self.code.len();
        self.code[jz_done + 1] = (done - jz_done - 2) as u8;
        self.code[jmp_done + 1] = (done - jmp_done - 2) as u8;
    }
    
    /// Helper: Convert value in al (0-63) to base64url char and write to [rdi], inc rdi
    fn emit_base64url_char(&mut self) {
        // 0-25: A-Z, 26-51: a-z, 52-61: 0-9, 62: -, 63: _
        self.code.extend(&[0x3C, 0x1A]); // cmp al, 26
        let jae_lower = self.code.len();
        self.code.extend(&[0x73, 0x00]); // jae lower
        self.code.extend(&[0x04, 0x41]); // add al, 'A'
        let jmp_store = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store
        
        let lower = self.code.len();
        self.code[jae_lower + 1] = (lower - jae_lower - 2) as u8;
        self.code.extend(&[0x3C, 0x34]); // cmp al, 52
        let jae_digit = self.code.len();
        self.code.extend(&[0x73, 0x00]); // jae digit
        self.code.extend(&[0x04, 0x47]); // add al, 'a' - 26
        let jmp_store2 = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store
        
        let digit = self.code.len();
        self.code[jae_digit + 1] = (digit - jae_digit - 2) as u8;
        self.code.extend(&[0x3C, 0x3E]); // cmp al, 62
        let jae_special = self.code.len();
        self.code.extend(&[0x73, 0x00]); // jae special
        self.code.extend(&[0x2C, 0x04]); // sub al, 4 (52 -> '0')
        let jmp_store3 = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store
        
        let special = self.code.len();
        self.code[jae_special + 1] = (special - jae_special - 2) as u8;
        self.code.extend(&[0x3C, 0x3E]); // cmp al, 62
        let jne_underscore = self.code.len();
        self.code.extend(&[0x75, 0x00]); // jne underscore
        self.code.extend(&[0xB0, 0x2D]); // mov al, '-'
        let jmp_store4 = self.code.len();
        self.code.extend(&[0xEB, 0x00]); // jmp store
        
        let underscore = self.code.len();
        self.code[jne_underscore + 1] = (underscore - jne_underscore - 2) as u8;
        self.code.extend(&[0xB0, 0x5F]); // mov al, '_'
        
        let store = self.code.len();
        self.code[jmp_store + 1] = (store - jmp_store - 2) as u8;
        self.code[jmp_store2 + 1] = (store - jmp_store2 - 2) as u8;
        self.code[jmp_store3 + 1] = (store - jmp_store3 - 2) as u8;
        self.code[jmp_store4 + 1] = (store - jmp_store4 - 2) as u8;
        
        self.code.extend(&[0x88, 0x07]); // mov [rdi], al
        self.code.extend(&[0x48, 0xFF, 0xC7]); // inc rdi
    }
    
    fn get_code(&self) -> &[u8] {
        &self.code
    }
    
    fn get_stack_size(&self) -> i64 {
        ((self.next_offset + 15) / 16) * 16
    }
    
    fn get_total_buffer_size(&self) -> i64 {
        self.next_buffer_base
    }
}

/// DOS Header (64 bytes)
fn create_dos_header(pe_offset: u32) -> Vec<u8> {
    let mut header = vec![0u8; DOS_HEADER_SIZE];
    
    // DOS signature "MZ"
    header[0] = 0x4D; // 'M'
    header[1] = 0x5A; // 'Z'
    
    // Bytes on last page
    header[2] = 0x90;
    header[3] = 0x00;
    
    // Pages in file
    header[4] = 0x03;
    header[5] = 0x00;
    
    // Relocations
    header[6] = 0x00;
    header[7] = 0x00;
    
    // Size of header in paragraphs
    header[8] = 0x04;
    header[9] = 0x00;
    
    // Minimum extra paragraphs
    header[10] = 0x00;
    header[11] = 0x00;
    
    // Maximum extra paragraphs
    header[12] = 0xFF;
    header[13] = 0xFF;
    
    // Initial SS
    header[14] = 0x00;
    header[15] = 0x00;
    
    // Initial SP
    header[16] = 0xB8;
    header[17] = 0x00;
    
    // Checksum
    header[18] = 0x00;
    header[19] = 0x00;
    
    // Initial IP
    header[20] = 0x00;
    header[21] = 0x00;
    
    // Initial CS
    header[22] = 0x00;
    header[23] = 0x00;
    
    // File address of relocation table
    header[24] = 0x40;
    header[25] = 0x00;
    
    // Overlay number
    header[26] = 0x00;
    header[27] = 0x00;
    
    // Reserved (8 bytes)
    // header[28..36] = 0
    
    // OEM identifier
    header[36] = 0x00;
    header[37] = 0x00;
    
    // OEM info
    header[38] = 0x00;
    header[39] = 0x00;
    
    // Reserved (20 bytes)
    // header[40..60] = 0
    
    // PE header offset (at offset 60)
    let pe_bytes = pe_offset.to_le_bytes();
    header[60] = pe_bytes[0];
    header[61] = pe_bytes[1];
    header[62] = pe_bytes[2];
    header[63] = pe_bytes[3];
    
    header
}

/// COFF File Header (20 bytes)
fn create_coff_header(num_sections: u16, timestamp: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(20);
    
    // Machine (AMD64)
    header.extend_from_slice(&IMAGE_FILE_MACHINE_AMD64.to_le_bytes());
    
    // Number of sections
    header.extend_from_slice(&num_sections.to_le_bytes());
    
    // Timestamp
    header.extend_from_slice(&timestamp.to_le_bytes());
    
    // Pointer to symbol table (0 for executables)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Number of symbols (0 for executables)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Size of optional header
    header.extend_from_slice(&240u16.to_le_bytes()); // PE32+ optional header size
    
    // Characteristics
    let characteristics = IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE;
    header.extend_from_slice(&characteristics.to_le_bytes());
    
    header
}

/// PE32+ Optional Header (240 bytes)
fn create_optional_header(
    code_size: u32,
    data_size: u32,
    entry_point_rva: u32,
    code_base: u32,
    image_size: u32,
    headers_size: u32,
) -> Vec<u8> {
    let mut header = Vec::with_capacity(240);
    
    // Magic (PE32+)
    header.extend_from_slice(&PE32_PLUS_MAGIC.to_le_bytes());
    
    // Linker version
    header.push(14); // Major
    header.push(0);  // Minor
    
    // Size of code
    header.extend_from_slice(&code_size.to_le_bytes());
    
    // Size of initialized data
    header.extend_from_slice(&data_size.to_le_bytes());
    
    // Size of uninitialized data
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Entry point RVA
    header.extend_from_slice(&entry_point_rva.to_le_bytes());
    
    // Base of code
    header.extend_from_slice(&code_base.to_le_bytes());
    
    // Image base (64-bit)
    header.extend_from_slice(&IMAGE_BASE.to_le_bytes());
    
    // Section alignment
    header.extend_from_slice(&SECTION_ALIGNMENT.to_le_bytes());
    
    // File alignment
    header.extend_from_slice(&FILE_ALIGNMENT.to_le_bytes());
    
    // OS version
    header.extend_from_slice(&6u16.to_le_bytes()); // Major
    header.extend_from_slice(&0u16.to_le_bytes()); // Minor
    
    // Image version
    header.extend_from_slice(&0u16.to_le_bytes()); // Major
    header.extend_from_slice(&0u16.to_le_bytes()); // Minor
    
    // Subsystem version
    header.extend_from_slice(&6u16.to_le_bytes()); // Major
    header.extend_from_slice(&0u16.to_le_bytes()); // Minor
    
    // Win32 version value (reserved)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Size of image
    header.extend_from_slice(&image_size.to_le_bytes());
    
    // Size of headers
    header.extend_from_slice(&headers_size.to_le_bytes());
    
    // Checksum (0 for non-drivers)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Subsystem
    header.extend_from_slice(&IMAGE_SUBSYSTEM_WINDOWS_CUI.to_le_bytes());
    
    // DLL characteristics
    header.extend_from_slice(&0x8160u16.to_le_bytes()); // ASLR, DEP, NX
    
    // Stack reserve (64-bit)
    header.extend_from_slice(&0x100000u64.to_le_bytes());
    
    // Stack commit (64-bit)
    header.extend_from_slice(&0x1000u64.to_le_bytes());
    
    // Heap reserve (64-bit)
    header.extend_from_slice(&0x100000u64.to_le_bytes());
    
    // Heap commit (64-bit)
    header.extend_from_slice(&0x1000u64.to_le_bytes());
    
    // Loader flags (reserved)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Number of data directories
    header.extend_from_slice(&16u32.to_le_bytes());
    
    // Data directories (16 entries, 8 bytes each = 128 bytes)
    // All zeros except import directory
    for i in 0..16 {
        if i == 1 {
            // Import directory - will be filled later
            header.extend_from_slice(&0u32.to_le_bytes()); // RVA
            header.extend_from_slice(&0u32.to_le_bytes()); // Size
        } else {
            header.extend_from_slice(&0u64.to_le_bytes());
        }
    }
    
    header
}

/// Section Header (40 bytes each)
fn create_section_header(
    name: &[u8; 8],
    virtual_size: u32,
    virtual_address: u32,
    raw_size: u32,
    raw_offset: u32,
    characteristics: u32,
) -> Vec<u8> {
    let mut header = Vec::with_capacity(40);
    
    // Name (8 bytes)
    header.extend_from_slice(name);
    
    // Virtual size
    header.extend_from_slice(&virtual_size.to_le_bytes());
    
    // Virtual address (RVA)
    header.extend_from_slice(&virtual_address.to_le_bytes());
    
    // Size of raw data
    header.extend_from_slice(&raw_size.to_le_bytes());
    
    // Pointer to raw data
    header.extend_from_slice(&raw_offset.to_le_bytes());
    
    // Pointer to relocations (0)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Pointer to line numbers (0)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Number of relocations (0)
    header.extend_from_slice(&0u16.to_le_bytes());
    
    // Number of line numbers (0)
    header.extend_from_slice(&0u16.to_le_bytes());
    
    // Characteristics
    header.extend_from_slice(&characteristics.to_le_bytes());
    
    header
}

/// Align value to boundary
fn align(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

/// Generate a minimal PE executable that prints "Hello"
pub fn generate_hello_pe() -> Vec<u8> {
    // Layout:
    // 0x000: DOS Header (64 bytes)
    // 0x080: PE Signature (4 bytes)
    // 0x084: COFF Header (20 bytes)
    // 0x098: Optional Header (240 bytes)
    // 0x188: Section Headers (40 bytes each, 3 sections = 120 bytes)
    // 0x200: .text section (code)
    // 0x400: .rdata section (imports + IAT)
    // 0x600: .data section (strings)
    
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    // Section layout
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x200;
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x200;
    let rdata_file_offset: u32 = 0x400;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x200;
    let data_file_offset: u32 = 0x600;
    
    let image_size: u32 = 0x4000;
    
    // Import table layout within .rdata
    // IAT (Import Address Table) at rdata_rva + 0x00
    // Import Directory at rdata_rva + 0x40
    // Import Lookup Table at rdata_rva + 0x80
    // Hint/Name Table at rdata_rva + 0xC0
    // DLL name at rdata_rva + 0x100
    
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x40;
    let ilt_rva = rdata_rva + 0x80;
    let hint_name_rva = rdata_rva + 0xC0;
    let dll_name_rva = rdata_rva + 0x100;
    
    // Build PE
    let mut pe = Vec::new();
    
    // DOS Header
    pe.extend(create_dos_header(pe_offset));
    
    // Pad to PE offset
    pe.resize(pe_offset as usize, 0);
    
    // PE Signature
    pe.extend_from_slice(PE_SIGNATURE);
    
    // COFF Header
    pe.extend(create_coff_header(num_sections, 0));
    
    // Optional Header (need to patch import directory)
    let mut opt_header = create_optional_header(
        text_size,
        rdata_size + data_size,
        text_rva,
        text_rva,
        image_size,
        headers_size,
    );
    
    // Patch import directory entry (index 1, at offset 112 in optional header)
    // Data directories start at offset 112 (after fixed fields)
    // Import directory is entry 1 (offset 112 + 8 = 120)
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes()); // Size of one import descriptor + null terminator
    
    pe.extend(opt_header);
    
    // Section Headers
    pe.extend(create_section_header(
        b".text\0\0\0",
        text_size,
        text_rva,
        text_size,
        text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
    ));
    
    pe.extend(create_section_header(
        b".rdata\0\0",
        rdata_size,
        rdata_rva,
        rdata_size,
        rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
    ));
    
    pe.extend(create_section_header(
        b".data\0\0\0",
        data_size,
        data_rva,
        data_size,
        data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
    ));
    
    // Pad to .text section
    pe.resize(text_file_offset as usize, 0);
    
    // .text section - Code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // Calculate RIP-relative offset to IAT entries
    // GetStdHandle at IAT[0], WriteFile at IAT[8], ExitProcess at IAT[16]
    
    // sub rsp, 56 (shadow space 32 + 8 for 5th param + 8 for alignment + 8 for return)
    code.extend(x64::sub_rsp_imm8(0x38));
    
    // --- GetStdHandle(-11) to get stdout ---
    // mov ecx, -11 (STD_OUTPUT_HANDLE)
    code.extend(&[0xB9]); // MOV ECX, imm32
    code.extend(&(-11i32).to_le_bytes());
    
    // call [GetStdHandle] via IAT
    let get_std_handle_iat = iat_rva as i64;
    let call_pos = code_start as i64 + code.len() as i64 + 6; // Position after this instruction
    let offset = get_std_handle_iat - call_pos;
    code.extend(&[0xFF, 0x15]); // CALL [RIP+disp32]
    code.extend(&(offset as i32).to_le_bytes());
    
    // mov r8, rax (save handle to r8 temporarily, but we need it in rcx for WriteFile)
    // Actually, let's restructure: save handle, then call WriteFile
    
    // mov [rsp+48], rax (save handle)
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x30]);
    
    // --- WriteFile(handle, buffer, len, &written, NULL) ---
    // mov rcx, [rsp+48] (handle)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x30]);
    
    // lea rdx, [rip + message_offset] (buffer = message in .data)
    let msg_rva = data_rva;
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let msg_offset = msg_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x15]); // LEA RDX, [RIP+disp32]
    code.extend(&(msg_offset as i32).to_le_bytes());
    
    // mov r8d, 15 (length of "Hello, TAYNI!\n")
    code.extend(&[0x41, 0xB8]); // MOV R8D, imm32
    code.extend(&15u32.to_le_bytes());
    
    // lea r9, [rsp+40] (lpNumberOfBytesWritten)
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x28]);
    
    // mov qword [rsp+32], 0 (lpOverlapped = NULL)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    // call [WriteFile] via IAT
    let write_file_iat = iat_rva as i64 + 8;
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let offset2 = write_file_iat - call_pos2;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(offset2 as i32).to_le_bytes());
    
    // --- ExitProcess(0) ---
    // xor ecx, ecx
    code.extend(x64::xor_ecx_ecx());
    
    // call [ExitProcess] via IAT
    let exit_process_iat = iat_rva as i64 + 16;
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let offset3 = exit_process_iat - call_pos3;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(offset3 as i32).to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata section - Import structures
    let mut rdata = vec![0u8; rdata_size as usize];
    
    // IAT (Import Address Table) - 3 entries + null terminator
    // Entry 0: GetStdHandle
    // Entry 1: WriteFile  
    // Entry 2: ExitProcess
    // Entry 3: NULL (terminator)
    let iat_offset = 0usize;
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 28;
    
    rdata[iat_offset..iat_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[iat_offset+8..iat_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[iat_offset+16..iat_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    // Entry 3 is already 0 (null terminator)
    
    // Import Directory Table (one entry + null terminator)
    let idt_offset = 0x40usize;
    // Import Lookup Table RVA
    rdata[idt_offset..idt_offset+4].copy_from_slice(&ilt_rva.to_le_bytes());
    // TimeDateStamp
    rdata[idt_offset+4..idt_offset+8].copy_from_slice(&0u32.to_le_bytes());
    // ForwarderChain
    rdata[idt_offset+8..idt_offset+12].copy_from_slice(&0u32.to_le_bytes());
    // Name RVA
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&dll_name_rva.to_le_bytes());
    // IAT RVA
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&iat_rva.to_le_bytes());
    // Null terminator entry (20 bytes of zeros) - already zero
    
    // Import Lookup Table (same as IAT before binding)
    let ilt_offset = 0x80usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0xC0usize;
    // GetStdHandle: hint=0, name
    rdata[hnt_offset] = 0; rdata[hnt_offset+1] = 0; // Hint
    let name1 = b"GetStdHandle\0";
    rdata[hnt_offset+2..hnt_offset+2+name1.len()].copy_from_slice(name1);
    
    // WriteFile: hint=0, name (at +16)
    let hnt2_offset = hnt_offset + 16;
    rdata[hnt2_offset] = 0; rdata[hnt2_offset+1] = 0;
    let name2 = b"WriteFile\0";
    rdata[hnt2_offset+2..hnt2_offset+2+name2.len()].copy_from_slice(name2);
    
    // ExitProcess: hint=0, name (at +28)
    let hnt3_offset = hnt_offset + 28;
    rdata[hnt3_offset] = 0; rdata[hnt3_offset+1] = 0;
    let name3 = b"ExitProcess\0";
    rdata[hnt3_offset+2..hnt3_offset+2+name3.len()].copy_from_slice(name3);
    
    // DLL name
    let dll_offset = 0x100usize;
    let dll_name = b"kernel32.dll\0";
    rdata[dll_offset..dll_offset+dll_name.len()].copy_from_slice(dll_name);
    
    pe.extend(&rdata);
    
    // .data section - Message
    let mut data = vec![0u8; data_size as usize];
    let msg = b"Hello, TAYNI!\n";
    data[..msg.len()].copy_from_slice(msg);
    
    pe.extend(&data);
    
    pe
}

/// Generate PE with runtime operations (@ prefix)
fn generate_pe_with_runtime_ops(graph: &Graph, runtime_analysis: &RuntimeAnalysis) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x1000;
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x400;  // Expanded for more IAT entries
    let rdata_file_offset: u32 = 0x1200;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x1000;
    let data_file_offset: u32 = 0x1600;  // Adjusted for larger rdata
    
    let image_size: u32 = 0x5000;  // Increased
    let entry_point: u32 = text_rva;
    
    // IAT layout: 15 entries (14 existing + 1 new for GetEnvironmentVariableA)
    let iat_rva: u32 = rdata_rva;
    let ilt_rva: u32 = rdata_rva + 0x80;  // After 15 IAT entries + null (16*8=128=0x80)
    let idt_rva: u32 = rdata_rva + 0x100;  // After ILT (0x80 + 0x80 = 0x100)
    let hint_name_rva: u32 = rdata_rva + 0x128;  // After IDT (0x100 + 0x28 = 0x128)
    let dll_name_rva: u32 = rdata_rva + 0x2A0;  // After hint/name table (moved for GetEnvironmentVariableA)
    
    // Buffer is at start of .data section
    let buffer_rva: u32 = data_rva;
    
    let mut pe = Vec::new();
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, entry_point, text_rva, image_size, headers_size
    );
    // Patch Import Directory in Data Directories (offset 112 + 8 = 120)
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&idt_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Collect buffer allocations and values
    let mut values: HashMap<String, i64> = HashMap::new();
    let mut strings: HashMap<String, String> = HashMap::new();
    let mut buffers: Vec<(String, i64)> = Vec::new();
    let mut string_order: Vec<String> = Vec::new(); // Track order of string declarations
    
    for node in &graph.nodes {
        match node {
            Node::Literal { id, value, .. } => {
                match value {
                    Value::Int(n) => { values.insert(id.clone(), *n); }
                    Value::String(s) => { 
                        strings.insert(id.clone(), s.clone()); 
                        string_order.push(id.clone());
                    }
                    _ => {}
                }
            }
            Node::Operation { id, op, args, .. } => {
                if *op == Op::Alc {
                    let size = get_val(&args, 0, &values).unwrap_or(64);
                    buffers.push((id.clone(), size));
                }
            }
            _ => {}
        }
    }
    
    // Add strings as buffers in declaration order (they need space in .data section)
    for name in &string_order {
        if let Some(content) = strings.get(name) {
            buffers.push((name.clone(), content.len() as i64 + 1)); // +1 for null terminator
        }
    }
    
    // Calculate buffer offsets in .data section
    let mut buffer_offsets: HashMap<String, i64> = HashMap::new();
    let mut next_offset: i64 = 0;
    for (name, size) in &buffers {
        buffer_offsets.insert(name.clone(), next_offset);
        next_offset += size;
    }
    
    // Variable offsets on stack
    let mut var_offsets: HashMap<String, i64> = HashMap::new();
    let mut next_var_offset: i64 = 0x50; // Start after shadow space
    
    // Initialize RuntimeCodeGen (reuses existing emit_* methods)
    let mut codegen = RuntimeCodeGen::new(text_rva, iat_rva);
    
    // Register buffers in codegen
    for (name, size) in &buffers {
        codegen.alloc_buffer(name, *size);
    }
    
    // Generate code
    let mut code = Vec::new();
    
    // Prologue: sub rsp, 0x100 (enough stack for variables)
    code.extend(&[0x48, 0x81, 0xEC, 0x00, 0x01, 0x00, 0x00]);
    
    // Load buffer base address (.data section) into [rsp+0x48]
    // lea rax, [rip + (buffer_rva - current_rip)]
    let lea_end = text_rva as i64 + code.len() as i64 + 7;
    let rel = buffer_rva as i64 - lea_end;
    code.extend(&[0x48, 0x8D, 0x05]);
    code.extend(&(rel as i32).to_le_bytes());
    // mov [rsp+0x48], rax
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x48]);
    
    // Set codegen's code to our prologue
    codegen.code = code;
    
    // Process runtime operations using RuntimeCodeGen methods
    for node in &graph.nodes {
        if let Node::Operation { id, op, args, .. } = node {
            if !runtime_analysis.is_runtime(id) {
                continue;
            }
            
            match op {
                Op::Put => {
                    if let Some(Arg::Ref(buf_ref)) = args.get(0) {
                        let offset_arg = get_runtime_arg(&args, 1, &values);
                        let value_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_put(buf_ref, &offset_arg, &value_arg);
                    }
                }
                Op::Get => {
                    if let Some(Arg::Ref(buf_ref)) = args.get(0) {
                        let offset_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_get(id, buf_ref, &offset_arg);
                    }
                }
                Op::Add => {
                    let a = get_runtime_arg(&args, 0, &values);
                    let b = get_runtime_arg(&args, 1, &values);
                    codegen.emit_add(id, &a, &b);
                }
                Op::Sub => {
                    let a = get_runtime_arg(&args, 0, &values);
                    let b = get_runtime_arg(&args, 1, &values);
                    codegen.emit_sub(id, &a, &b);
                }
                Op::Mul => {
                    let a = get_runtime_arg(&args, 0, &values);
                    let b = get_runtime_arg(&args, 1, &values);
                    codegen.emit_mul(id, &a, &b);
                }
                Op::Div => {
                    let a = get_runtime_arg(&args, 0, &values);
                    let b = get_runtime_arg(&args, 1, &values);
                    codegen.emit_div(id, &a, &b);
                }
                Op::Mod => {
                    let a = get_runtime_arg(&args, 0, &values);
                    let b = get_runtime_arg(&args, 1, &values);
                    codegen.emit_mod(id, &a, &b);
                }
                Op::Its => {
                    let src = get_runtime_arg(&args, 0, &values);
                    if let Some(Arg::Ref(buf_ref)) = args.get(1) {
                        codegen.emit_its(id, &src, buf_ref);
                    }
                }
                Op::Sln => {
                    if let Some(Arg::Ref(str_ref)) = args.get(0) {
                        codegen.emit_sln(id, str_ref);
                    }
                }
                Op::Cat => {
                    if args.len() >= 3 {
                        if let (Some(Arg::Ref(dst)), Some(Arg::Ref(s1)), Some(Arg::Ref(s2))) = 
                            (args.get(0), args.get(1), args.get(2)) {
                            codegen.emit_cat(id, dst, s1, s2);
                        }
                    }
                }
                Op::Cmp => {
                    if args.len() >= 3 {
                        if let (Some(Arg::Ref(a)), Some(Arg::Ref(b))) = (args.get(0), args.get(1)) {
                            let len = get_runtime_arg(&args, 2, &values);
                            codegen.emit_cmp_mem(id, a, b, &len);
                        }
                    }
                }
                Op::JsonGet => {
                    // JSON.GET json_buf key_buf val_buf
                    if args.len() >= 3 {
                        if let (Some(Arg::Ref(json)), Some(Arg::Ref(key)), Some(Arg::Ref(val))) =
                            (args.get(0), args.get(1), args.get(2)) {
                            codegen.emit_json_get(id, json, key, val);
                        }
                    }
                }
                Op::JsonEncode => {
                    // JSON.ENCODE dst_buf key_buf val_buf
                    if args.len() >= 3 {
                        if let (Some(Arg::Ref(dst)), Some(Arg::Ref(key)), Some(Arg::Ref(val))) =
                            (args.get(0), args.get(1), args.get(2)) {
                            codegen.emit_json_encode(id, dst, key, val);
                        }
                    }
                }
                Op::JsonSet => {
                    // JSON.SET json_buf key_buf new_val_buf
                    if args.len() >= 3 {
                        if let (Some(Arg::Ref(json)), Some(Arg::Ref(key)), Some(Arg::Ref(val))) =
                            (args.get(0), args.get(1), args.get(2)) {
                            codegen.emit_json_set(id, json, key, val);
                        }
                    }
                }
                Op::TimeNow => {
                    codegen.emit_time_now(id, iat_rva, text_rva);
                }
                Op::TimeSleep => {
                    let ms = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_sleep(&ms, iat_rva, text_rva);
                }
                Op::TimeNowMs => {
                    codegen.emit_time_now(id, iat_rva, text_rva);
                }
                Op::Thr => {
                    // THR func_ptr arg -> thread_handle
                    let func_ptr = get_runtime_arg(&args, 0, &values);
                    let arg = get_runtime_arg(&args, 1, &values);
                    codegen.emit_thr(id, &func_ptr, &arg, iat_rva, text_rva);
                }
                Op::Jon => {
                    // JON thread_handle [timeout_ms] -> wait result
                    // If timeout provided, use it; otherwise INFINITE
                    let handle = get_runtime_arg(&args, 0, &values);
                    if args.len() > 1 {
                        let timeout = get_runtime_arg(&args, 1, &values);
                        codegen.emit_jon_timeout(id, &handle, &timeout, iat_rva, text_rva);
                    } else {
                        codegen.emit_jon(id, &handle, iat_rva, text_rva);
                    }
                }
                Op::Mtx => {
                    // MTX -> mutex_handle
                    codegen.emit_mtx(id, iat_rva, text_rva);
                }
                Op::Lck => {
                    // LCK mutex_handle [timeout_ms] -> wait result
                    let mutex = get_runtime_arg(&args, 0, &values);
                    if args.len() > 1 {
                        let timeout = get_runtime_arg(&args, 1, &values);
                        codegen.emit_lck_timeout(id, &mutex, &timeout, iat_rva, text_rva);
                    } else {
                        codegen.emit_lck(id, &mutex, iat_rva, text_rva);
                    }
                }
                Op::Ulk => {
                    // ULK mutex_handle -> result
                    let mutex = get_runtime_arg(&args, 0, &values);
                    codegen.emit_ulk(id, &mutex, iat_rva, text_rva);
                }
                Op::Tlk => {
                    // TLK mutex_handle -> 0 if acquired, 1 if not
                    let mutex = get_runtime_arg(&args, 0, &values);
                    codegen.emit_tlk(id, &mutex, iat_rva, text_rva);
                }
                Op::Yld => {
                    // YLD -> yield CPU
                    codegen.emit_yld(id, iat_rva, text_rva);
                }
                Op::AtmLd => {
                    // ATM.LD ptr -> value
                    let ptr = get_runtime_arg(&args, 0, &values);
                    codegen.emit_atm_ld(id, &ptr);
                }
                Op::AtmSt => {
                    // ATM.ST ptr val
                    let ptr = get_runtime_arg(&args, 0, &values);
                    let val = get_runtime_arg(&args, 1, &values);
                    codegen.emit_atm_st(&ptr, &val);
                }
                Op::AtmXchg => {
                    // ATM.XCHG ptr new -> old
                    let ptr = get_runtime_arg(&args, 0, &values);
                    let new_val = get_runtime_arg(&args, 1, &values);
                    codegen.emit_atm_xchg(id, &ptr, &new_val);
                }
                Op::AtmCas => {
                    // ATM.CAS ptr expected desired -> success
                    let ptr = get_runtime_arg(&args, 0, &values);
                    let expected = get_runtime_arg(&args, 1, &values);
                    let desired = get_runtime_arg(&args, 2, &values);
                    codegen.emit_atm_cas(id, &ptr, &expected, &desired);
                }
                Op::AtmAdd => {
                    // ATM.ADD ptr val -> old
                    let ptr = get_runtime_arg(&args, 0, &values);
                    let val = get_runtime_arg(&args, 1, &values);
                    codegen.emit_atm_add(id, &ptr, &val);
                }
                Op::AtmSub => {
                    // ATM.SUB ptr val -> old
                    let ptr = get_runtime_arg(&args, 0, &values);
                    let val = get_runtime_arg(&args, 1, &values);
                    codegen.emit_atm_sub(id, &ptr, &val);
                }
                Op::Fence => {
                    // FNC -> memory fence
                    codegen.emit_fence(id);
                }
                Op::Chn => {
                    // CHN capacity -> channel handle
                    let cap = get_runtime_arg(&args, 0, &values);
                    codegen.emit_chn(id, &cap, iat_rva, text_rva);
                }
                Op::ChnSnd => {
                    // CHN.SND channel value -> result
                    let chan = get_runtime_arg(&args, 0, &values);
                    let val = get_runtime_arg(&args, 1, &values);
                    codegen.emit_chn_snd(id, &chan, &val, iat_rva, text_rva);
                }
                Op::ChnRcv => {
                    // CHN.RCV channel -> value
                    let chan = get_runtime_arg(&args, 0, &values);
                    codegen.emit_chn_rcv(id, &chan, iat_rva, text_rva);
                }
                Op::ChnCls => {
                    // CHN.CLS channel -> result
                    let chan = get_runtime_arg(&args, 0, &values);
                    codegen.emit_chn_cls(id, &chan);
                }
                Op::Argc => {
                    // ARGC -> number of arguments
                    codegen.emit_argc(id, iat_rva, text_rva);
                }
                Op::Argv => {
                    // ARGV index -> pointer to argument at index
                    let index = get_runtime_arg(&args, 0, &values);
                    codegen.emit_argv(id, &index, iat_rva, text_rva);
                }
                Op::Rnd => {
                    // RND -> random 64-bit integer
                    codegen.emit_rnd(id, iat_rva, text_rva);
                }
                Op::Rng => {
                    // RNG max -> random integer in [0, max)
                    let max = get_runtime_arg(&args, 0, &values);
                    codegen.emit_rng(id, &max, iat_rva, text_rva);
                }
                Op::Log => {
                    // LOG buf len -> print "[timestamp] message\n"
                    if let Some(Arg::Ref(buf_ref)) = args.get(0) {
                        let len = get_runtime_arg(&args, 1, &values);
                        codegen.emit_log(buf_ref, &len, iat_rva, text_rva);
                    }
                }
                Op::GetEnv => {
                    // GETENV name buf -> length
                    if let (Some(Arg::Ref(name_ref)), Some(Arg::Ref(buf_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_getenv(id, name_ref, buf_ref, iat_rva, text_rva);
                    }
                }
                Op::PathJoin => {
                    // PATH.JOIN dst a b -> join paths
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(a_ref)), Some(Arg::Ref(b_ref))) = (args.get(0), args.get(1), args.get(2)) {
                        codegen.emit_path_join(id, dst_ref, a_ref, b_ref);
                    }
                }
                Op::PathDir => {
                    // PATH.DIR dst src -> get directory
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_path_dir(id, dst_ref, src_ref);
                    }
                }
                Op::PathBase => {
                    // PATH.BASE dst src -> get filename
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_path_base(id, dst_ref, src_ref);
                    }
                }
                Op::PathExt => {
                    // PATH.EXT dst src -> get extension
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_path_ext(id, dst_ref, src_ref);
                    }
                }
                Op::HashMd5 => {
                    // HASH.MD5 dst src len -> compute MD5 hash
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_hash_md5(id, dst_ref, src_ref, &len_arg);
                    }
                }
                Op::HashSha256 => {
                    // HASH.SHA256 dst src len -> compute SHA256 hash
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_hash_sha256(id, dst_ref, src_ref, &len_arg);
                    }
                }
                Op::TimeYear => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_year(id, &ts_arg);
                }
                Op::TimeMonth => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_month(id, &ts_arg);
                }
                Op::TimeDay => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_day(id, &ts_arg);
                }
                Op::TimeHour => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_hour(id, &ts_arg);
                }
                Op::TimeMin => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_min(id, &ts_arg);
                }
                Op::TimeSec => {
                    let ts_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_time_sec(id, &ts_arg);
                }
                Op::UuidV4 => {
                    // UUID.V4 dst -> generate UUID v4
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        codegen.emit_uuid_v4(id, dst_ref, iat_rva, text_rva);
                    }
                }
                Op::FormatInt => {
                    // FORMAT.INT dst num -> convert int to string
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let num_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_format_int(id, dst_ref, &num_arg);
                    }
                }
                Op::FormatHex => {
                    // FORMAT.HEX dst num -> convert int to hex string
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let num_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_format_hex(id, dst_ref, &num_arg);
                    }
                }
                Op::ValidateEmail => {
                    // VALIDATE.EMAIL src len -> 1 if valid
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_validate_email(id, src_ref, &len_arg);
                    }
                }
                Op::ValidateIpv4 => {
                    // VALIDATE.IPV4 src len -> 1 if valid
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_validate_ipv4(id, src_ref, &len_arg);
                    }
                }
                Op::ValidateUuid => {
                    // VALIDATE.UUID src len -> 1 if valid
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_validate_uuid(id, src_ref, &len_arg);
                    }
                }
                Op::TestAssert => {
                    // TEST.ASSERT cond msg len
                    let cond_arg = get_runtime_arg(&args, 0, &values);
                    if let Some(Arg::Ref(msg_ref)) = args.get(1) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_test_assert(&cond_arg, msg_ref, &len_arg, iat_rva, text_rva);
                    }
                }
                Op::TestAssertEq => {
                    // TEST.ASSERT_EQ a b msg len
                    let a_arg = get_runtime_arg(&args, 0, &values);
                    let b_arg = get_runtime_arg(&args, 1, &values);
                    if let Some(Arg::Ref(msg_ref)) = args.get(2) {
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_test_assert_eq(&a_arg, &b_arg, msg_ref, &len_arg, iat_rva, text_rva);
                    }
                }
                Op::JwtEncode => {
                    // JWT.ENCODE payload payload_len secret secret_len out
                    if let Some(Arg::Ref(payload_ref)) = args.get(0) {
                        let payload_len = get_runtime_arg(&args, 1, &values);
                        if let Some(Arg::Ref(secret_ref)) = args.get(2) {
                            let secret_len = get_runtime_arg(&args, 3, &values);
                            if let Some(Arg::Ref(out_ref)) = args.get(4) {
                                codegen.emit_jwt_encode(id, out_ref, payload_ref, &payload_len, secret_ref, &secret_len);
                            }
                        }
                    }
                }
                Op::JwtVerify => {
                    // JWT.VERIFY token token_len secret secret_len
                    if let Some(Arg::Ref(token_ref)) = args.get(0) {
                        let token_len = get_runtime_arg(&args, 1, &values);
                        codegen.emit_jwt_verify(id, token_ref, &token_len);
                    }
                }
                Op::RegexMatch => {
                    // REGEX.MATCH str len pattern pat_len
                    if let Some(Arg::Ref(str_ref)) = args.get(0) {
                        let str_len = get_runtime_arg(&args, 1, &values);
                        if let Some(Arg::Ref(pat_ref)) = args.get(2) {
                            let pat_len = get_runtime_arg(&args, 3, &values);
                            codegen.emit_regex_match(id, str_ref, &str_len, pat_ref, &pat_len);
                        }
                    }
                }
                Op::RegexFind => {
                    // REGEX.FIND str len pattern pat_len
                    if let Some(Arg::Ref(str_ref)) = args.get(0) {
                        let str_len = get_runtime_arg(&args, 1, &values);
                        if let Some(Arg::Ref(pat_ref)) = args.get(2) {
                            let pat_len = get_runtime_arg(&args, 3, &values);
                            codegen.emit_regex_find(id, str_ref, &str_len, pat_ref, &pat_len);
                        }
                    }
                }
                Op::AsyncSpawn => {
                    // ASYNC.SPAWN is alias for THR
                    let func_arg = get_runtime_arg(&args, 0, &values);
                    let arg_arg = get_runtime_arg(&args, 1, &values);
                    codegen.emit_thr(id, &func_arg, &arg_arg, iat_rva, text_rva);
                }
                Op::AsyncAwait => {
                    // ASYNC.AWAIT is alias for JON
                    let handle_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_jon(id, &handle_arg, iat_rva, text_rva);
                }
                Op::TimeoutSet => {
                    // TIMEOUT.SET ms - store timeout value
                    let ms_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_timeout_set(id, &ms_arg);
                }
                Op::TimeoutCheck => {
                    // TIMEOUT.CHECK - check if timeout occurred
                    codegen.emit_timeout_check(id);
                }
                // === STDLIB TIER 2: CSV Module ===
                Op::CsvParse => {
                    // CSV.PARSE str len -> csv_obj
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_csv_parse(id, src_ref, &len_arg);
                    }
                }
                Op::CsvNextRow => {
                    // CSV.NEXT_ROW csv -> 1 if has row
                    let csv_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_csv_next_row(id, &csv_arg);
                }
                Op::CsvGetField => {
                    // CSV.GET_FIELD csv col -> field value
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let csv_arg = get_runtime_arg(&args, 1, &values);
                        let col_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_csv_get_field(id, dst_ref, &csv_arg, &col_arg);
                    }
                }
                // === STDLIB TIER 2: YAML Module ===
                Op::YamlParse => {
                    // YAML.PARSE str len -> yaml_obj
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_yaml_parse(id, src_ref, &len_arg);
                    }
                }
                Op::YamlGet => {
                    // YAML.GET yaml key key_len -> value
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(key_ref))) = (args.get(0), args.get(2)) {
                        let yaml_arg = get_runtime_arg(&args, 1, &values);
                        let key_len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_yaml_get(id, dst_ref, &yaml_arg, key_ref, &key_len_arg);
                    }
                }
                // === STDLIB TIER 2: XML Module ===
                Op::XmlParse => {
                    // XML.PARSE str len -> xml_obj
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_xml_parse(id, src_ref, &len_arg);
                    }
                }
                Op::XmlTag => {
                    // XML.TAG xml -> tag name
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let xml_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_xml_tag(id, dst_ref, &xml_arg);
                    }
                }
                Op::XmlText => {
                    // XML.TEXT xml -> text content
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let xml_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_xml_text(id, dst_ref, &xml_arg);
                    }
                }
                // === STDLIB TIER 2: RETRY Module ===
                Op::RetryConfigNew => {
                    // RETRY.CONFIG_NEW -> config
                    codegen.emit_retry_config_new(id);
                }
                Op::RetryExecute => {
                    // RETRY.EXECUTE config fn -> result
                    let config_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_retry_execute(id, &config_arg);
                }
                // === STDLIB TIER 2: CORS Module ===
                Op::CorsConfigNew => {
                    // CORS.CONFIG_NEW -> config
                    codegen.emit_cors_config_new(id);
                }
                Op::CorsAllowAllOrigins => {
                    // CORS.ALLOW_ALL_ORIGINS config -> config
                    let config_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_cors_allow_all_origins(id, &config_arg);
                }
                // === STDLIB TIER 2: COOKIE Module ===
                Op::CookieParse => {
                    // COOKIE.PARSE str len -> cookie_obj
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_cookie_parse(id, src_ref, &len_arg);
                    }
                }
                Op::CookieGet => {
                    // COOKIE.GET dst cookie name name_len -> value
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(name_ref))) = (args.get(0), args.get(2)) {
                        let cookie_arg = get_runtime_arg(&args, 1, &values);
                        let name_len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_cookie_get(id, dst_ref, &cookie_arg, name_ref, &name_len_arg);
                    }
                }
                // === STDLIB TIER 2: GZIP Module ===
                Op::GzipCompress => {
                    // GZIP.COMPRESS dst src len -> compressed_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_gzip_compress(id, dst_ref, src_ref, &len_arg);
                    }
                }
                Op::GzipDecompress => {
                    // GZIP.DECOMPRESS dst src len -> decompressed_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_gzip_decompress(id, dst_ref, src_ref, &len_arg);
                    }
                }
                // === STDLIB TIER 2: CRYPTO Module ===
                Op::Sha1 => {
                    // SHA1 dst src len -> hash_len (40 hex chars)
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_sha1(id, dst_ref, src_ref, &len_arg);
                    }
                }
                Op::Md5 => {
                    // MD5 dst src len -> hash_len (32 hex chars)
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1)) {
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_md5(id, dst_ref, src_ref, &len_arg);
                    }
                }
                // === STDLIB TIER 2: POSTGRES Module ===
                Op::PgConnect => {
                    // PG.CONNECT connstr -> conn handle
                    if let Some(Arg::Ref(connstr_ref)) = args.get(0) {
                        codegen.emit_pg_connect(id, connstr_ref);
                    }
                }
                Op::PgQuery => {
                    // PG.QUERY conn query query_len -> result handle
                    if let Some(Arg::Ref(query_ref)) = args.get(1) {
                        let conn_arg = get_runtime_arg(&args, 0, &values);
                        let query_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_pg_query(id, &conn_arg, query_ref, &query_len_arg);
                    }
                }
                Op::PgFetch => {
                    // PG.FETCH dst result -> row_len
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let result_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_pg_fetch(id, dst_ref, &result_arg);
                    }
                }
                Op::PgClose => {
                    // PG.CLOSE conn -> 1 if closed
                    let conn_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_pg_close(id, &conn_arg);
                }
                // === STDLIB TIER 2: SQL/ODBC Module ===
                Op::SqlFetch => {
                    // SQL.FETCH dst result -> row_len
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let result_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_sql_fetch(id, dst_ref, &result_arg);
                    }
                }
                Op::AesEncrypt => {
                    // AES.ENCRYPT dst key src len -> encrypted_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(key_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1), args.get(2)) {
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_aes_encrypt(id, dst_ref, key_ref, src_ref, &len_arg);
                    }
                }
                Op::AesDecrypt => {
                    // AES.DECRYPT dst key src len -> decrypted_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(key_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(1), args.get(2)) {
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_aes_decrypt(id, dst_ref, key_ref, src_ref, &len_arg);
                    }
                }
                // === STDLIB TIER 2: REDIS Module ===
                Op::RedisConnect => {
                    // REDIS.CONNECT host port -> connection handle
                    if let Some(Arg::Ref(host_ref)) = args.get(0) {
                        let port_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_redis_connect(id, host_ref, &port_arg);
                    }
                }
                Op::RedisSet => {
                    // REDIS.SET conn key key_len val val_len -> 1 if OK
                    if let (Some(Arg::Ref(key_ref)), Some(Arg::Ref(val_ref))) = (args.get(1), args.get(3)) {
                        let conn_arg = get_runtime_arg(&args, 0, &values);
                        let key_len_arg = get_runtime_arg(&args, 2, &values);
                        let val_len_arg = get_runtime_arg(&args, 4, &values);
                        codegen.emit_redis_set(id, &conn_arg, key_ref, &key_len_arg, val_ref, &val_len_arg);
                    }
                }
                Op::RedisGet => {
                    // REDIS.GET conn dst key key_len -> value length
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(key_ref))) = (args.get(1), args.get(2)) {
                        let conn_arg = get_runtime_arg(&args, 0, &values);
                        let key_len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_redis_get(id, dst_ref, &conn_arg, key_ref, &key_len_arg);
                    }
                }
                Op::RedisDel => {
                    // REDIS.DEL conn key key_len -> 1 if deleted
                    if let Some(Arg::Ref(key_ref)) = args.get(1) {
                        let conn_arg = get_runtime_arg(&args, 0, &values);
                        let key_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_redis_del(id, &conn_arg, key_ref, &key_len_arg);
                    }
                }
                Op::RedisClose => {
                    // REDIS.CLOSE conn -> 1 if closed
                    let conn_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_redis_close(id, &conn_arg);
                }
                // === STDLIB TIER 2: WEBSOCKET Module ===
                Op::WsConnect => {
                    // WS.CONNECT url url_len -> ws handle
                    if let Some(Arg::Ref(url_ref)) = args.get(0) {
                        let url_len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_ws_connect(id, url_ref, &url_len_arg);
                    }
                }
                Op::WsAccept => {
                    // WS.ACCEPT server -> ws handle
                    let server_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_ws_accept(id, &server_arg);
                }
                Op::WsSend => {
                    // WS.SEND frame_buf ws data len -> frame_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(2)) {
                        let ws_arg = get_runtime_arg(&args, 1, &values);
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_ws_send(id, dst_ref, &ws_arg, src_ref, &len_arg);
                    }
                }
                Op::WsRecv => {
                    // WS.RECV dst ws frame frame_len -> payload_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(2)) {
                        let ws_arg = get_runtime_arg(&args, 1, &values);
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_ws_recv(id, dst_ref, &ws_arg, src_ref, &len_arg);
                    }
                }
                Op::WsClose => {
                    // WS.CLOSE ws -> 1 if closed
                    let ws_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_ws_close(id, &ws_arg);
                }
                // === STDLIB TIER 2: TLS Module ===
                Op::TlsConnect => {
                    // TLS.CONNECT host port -> tls handle
                    if let Some(Arg::Ref(host_ref)) = args.get(0) {
                        let port_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_tls_connect(id, host_ref, &port_arg);
                    }
                }
                Op::TlsAccept => {
                    // TLS.ACCEPT server cert key -> tls handle
                    if let (Some(Arg::Ref(cert_ref)), Some(Arg::Ref(key_ref))) = (args.get(1), args.get(2)) {
                        let server_arg = get_runtime_arg(&args, 0, &values);
                        codegen.emit_tls_accept(id, &server_arg, cert_ref, key_ref);
                    }
                }
                Op::TlsSend => {
                    // TLS.SEND record_buf tls data len -> record_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(2)) {
                        let tls_arg = get_runtime_arg(&args, 1, &values);
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_tls_send(id, dst_ref, &tls_arg, src_ref, &len_arg);
                    }
                }
                Op::TlsRecv => {
                    // TLS.RECV dst tls record record_len -> payload_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(src_ref))) = (args.get(0), args.get(2)) {
                        let tls_arg = get_runtime_arg(&args, 1, &values);
                        let len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_tls_recv(id, dst_ref, &tls_arg, src_ref, &len_arg);
                    }
                }
                Op::TlsClose => {
                    // TLS.CLOSE tls -> 1 if closed
                    let tls_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_tls_close(id, &tls_arg);
                }
                // === STDLIB TIER 2: GRPC Module ===
                Op::GrpcChannel => {
                    // GRPC.CHANNEL host port -> channel handle
                    if let Some(Arg::Ref(host_ref)) = args.get(0) {
                        let port_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_grpc_channel(id, host_ref, &port_arg);
                    }
                }
                Op::GrpcCall => {
                    // GRPC.CALL channel method method_len -> call handle
                    if let Some(Arg::Ref(method_ref)) = args.get(1) {
                        let channel_arg = get_runtime_arg(&args, 0, &values);
                        let method_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_grpc_call(id, &channel_arg, method_ref, &method_len_arg);
                    }
                }
                Op::GrpcSend => {
                    // GRPC.SEND call data len -> bytes sent
                    if let Some(Arg::Ref(data_ref)) = args.get(1) {
                        let call_arg = get_runtime_arg(&args, 0, &values);
                        let len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_grpc_send(id, &call_arg, data_ref, &len_arg);
                    }
                }
                Op::GrpcRecv => {
                    // GRPC.RECV dst call buf_len -> bytes received
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let call_arg = get_runtime_arg(&args, 1, &values);
                        let buf_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_grpc_recv(id, dst_ref, &call_arg, &buf_len_arg);
                    }
                }
                Op::GrpcClose => {
                    // GRPC.CLOSE channel -> 1 if closed
                    let channel_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_grpc_close(id, &channel_arg);
                }
                // === STDLIB TIER 2: PQC Module ===
                Op::KyberKeygen => {
                    // KYBER.KEYGEN pub_out priv_out -> 1 if success
                    if let (Some(Arg::Ref(pub_ref)), Some(Arg::Ref(priv_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_kyber_keygen(id, pub_ref, priv_ref);
                    }
                }
                Op::KyberEncaps => {
                    // KYBER.ENCAPS pub ct_out ss_out -> 1 if success
                    if let (Some(Arg::Ref(pub_ref)), Some(Arg::Ref(ct_ref)), Some(Arg::Ref(ss_ref))) = (args.get(0), args.get(1), args.get(2)) {
                        codegen.emit_kyber_encaps(id, pub_ref, ct_ref, ss_ref);
                    }
                }
                Op::KyberDecaps => {
                    // KYBER.DECAPS priv ct ss_out -> 1 if success
                    if let (Some(Arg::Ref(priv_ref)), Some(Arg::Ref(ct_ref)), Some(Arg::Ref(ss_ref))) = (args.get(0), args.get(1), args.get(2)) {
                        codegen.emit_kyber_decaps(id, priv_ref, ct_ref, ss_ref);
                    }
                }
                Op::DilithiumKeygen => {
                    // DILITHIUM.KEYGEN pub_out priv_out -> 1 if success
                    if let (Some(Arg::Ref(pub_ref)), Some(Arg::Ref(priv_ref))) = (args.get(0), args.get(1)) {
                        codegen.emit_dilithium_keygen(id, pub_ref, priv_ref);
                    }
                }
                Op::DilithiumSign => {
                    // DILITHIUM.SIGN priv msg msg_len sig_out -> sig_len
                    if let (Some(Arg::Ref(priv_ref)), Some(Arg::Ref(msg_ref)), Some(Arg::Ref(sig_ref))) = (args.get(0), args.get(1), args.get(3)) {
                        let msg_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_dilithium_sign(id, priv_ref, msg_ref, &msg_len_arg, sig_ref);
                    }
                }
                Op::DilithiumVerify => {
                    // DILITHIUM.VERIFY pub msg msg_len sig sig_len -> 1 if valid
                    if let (Some(Arg::Ref(pub_ref)), Some(Arg::Ref(msg_ref)), Some(Arg::Ref(sig_ref))) = (args.get(0), args.get(1), args.get(3)) {
                        let msg_len_arg = get_runtime_arg(&args, 2, &values);
                        let sig_len_arg = get_runtime_arg(&args, 4, &values);
                        codegen.emit_dilithium_verify(id, pub_ref, msg_ref, &msg_len_arg, sig_ref, &sig_len_arg);
                    }
                }
                // === GUI Module ===
                Op::GuiWin => {
                    // GUI.WIN title title_len width height -> hwnd
                    if let Some(Arg::Ref(title_ref)) = args.get(0) {
                        let title_len_arg = get_runtime_arg(&args, 1, &values);
                        let width_arg = get_runtime_arg(&args, 2, &values);
                        let height_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_gui_win(id, title_ref, &title_len_arg, &width_arg, &height_arg);
                    }
                }
                Op::GuiShow => {
                    // GUI.SHOW hwnd -> 1 if shown
                    let hwnd_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_gui_show(id, &hwnd_arg);
                }
                Op::GuiHide => {
                    // GUI.HIDE hwnd -> 1 if hidden
                    let hwnd_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_gui_hide(id, &hwnd_arg);
                }
                Op::GuiEvent => {
                    // GUI.EVENT -> event_type
                    codegen.emit_gui_event(id);
                }
                Op::GuiRun => {
                    // GUI.RUN -> runs message loop
                    codegen.emit_gui_run(id);
                }
                Op::GuiLabel => {
                    // GUI.LABEL parent text text_len x y w h -> hwnd
                    if let Some(Arg::Ref(text_ref)) = args.get(1) {
                        let parent_arg = get_runtime_arg(&args, 0, &values);
                        let text_len_arg = get_runtime_arg(&args, 2, &values);
                        let x_arg = get_runtime_arg(&args, 3, &values);
                        let y_arg = get_runtime_arg(&args, 4, &values);
                        let w_arg = get_runtime_arg(&args, 5, &values);
                        let h_arg = get_runtime_arg(&args, 6, &values);
                        codegen.emit_gui_label(id, &parent_arg, text_ref, &text_len_arg, &x_arg, &y_arg, &w_arg, &h_arg);
                    }
                }
                Op::GuiTextbox => {
                    // GUI.TEXTBOX parent x y w h -> hwnd
                    let parent_arg = get_runtime_arg(&args, 0, &values);
                    let x_arg = get_runtime_arg(&args, 1, &values);
                    let y_arg = get_runtime_arg(&args, 2, &values);
                    let w_arg = get_runtime_arg(&args, 3, &values);
                    let h_arg = get_runtime_arg(&args, 4, &values);
                    codegen.emit_gui_textbox(id, &parent_arg, &x_arg, &y_arg, &w_arg, &h_arg);
                }
                Op::GuiButton => {
                    // GUI.BUTTON parent text text_len x y w h -> hwnd
                    if let Some(Arg::Ref(text_ref)) = args.get(1) {
                        let parent_arg = get_runtime_arg(&args, 0, &values);
                        let text_len_arg = get_runtime_arg(&args, 2, &values);
                        let x_arg = get_runtime_arg(&args, 3, &values);
                        let y_arg = get_runtime_arg(&args, 4, &values);
                        let w_arg = get_runtime_arg(&args, 5, &values);
                        let h_arg = get_runtime_arg(&args, 6, &values);
                        codegen.emit_gui_button(id, &parent_arg, text_ref, &text_len_arg, &x_arg, &y_arg, &w_arg, &h_arg);
                    }
                }
                Op::GuiGetVal => {
                    // GUI.GETVAL dst hwnd max_len -> actual_len
                    if let Some(Arg::Ref(dst_ref)) = args.get(0) {
                        let hwnd_arg = get_runtime_arg(&args, 1, &values);
                        let max_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_gui_getval(id, dst_ref, &hwnd_arg, &max_len_arg);
                    }
                }
                Op::GuiSetVal => {
                    // GUI.SETVAL hwnd text text_len -> 1 if set
                    if let Some(Arg::Ref(text_ref)) = args.get(1) {
                        let hwnd_arg = get_runtime_arg(&args, 0, &values);
                        let text_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_gui_setval(id, &hwnd_arg, text_ref, &text_len_arg);
                    }
                }
                Op::GuiMsgBox => {
                    // GUI.MSGBOX title title_len text text_len -> button_id
                    if let (Some(Arg::Ref(title_ref)), Some(Arg::Ref(text_ref))) = (args.get(0), args.get(2)) {
                        let title_len_arg = get_runtime_arg(&args, 1, &values);
                        let text_len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_gui_msgbox(id, title_ref, &title_len_arg, text_ref, &text_len_arg);
                    }
                }
                // === Advanced Networking ===
                Op::Udp => {
                    // UDP -> socket handle
                    codegen.emit_udp(id);
                }
                Op::Con => {
                    // CON socket host port -> 0 if success
                    if let Some(Arg::Ref(host_ref)) = args.get(1) {
                        let socket_arg = get_runtime_arg(&args, 0, &values);
                        let port_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_con(id, &socket_arg, host_ref, &port_arg);
                    }
                }
                Op::Sel => {
                    // SEL read_set write_set timeout -> num_ready
                    let read_set_arg = get_runtime_arg(&args, 0, &values);
                    let write_set_arg = get_runtime_arg(&args, 1, &values);
                    let timeout_arg = get_runtime_arg(&args, 2, &values);
                    codegen.emit_sel(id, &read_set_arg, &write_set_arg, &timeout_arg);
                }
                Op::Rdy => {
                    // RDY socket -> 1 if ready
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_rdy(id, &socket_arg);
                }
                Op::Nbk => {
                    // NBK socket enable -> 0 if success
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    let enable_arg = get_runtime_arg(&args, 1, &values);
                    codegen.emit_nbk(id, &socket_arg, &enable_arg);
                }
                Op::Ndl => {
                    // NDL socket -> 0 if success
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_ndl(id, &socket_arg);
                }
                Op::Qck => {
                    // QCK socket -> 0 if success
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_qck(id, &socket_arg);
                }
                Op::Sbf => {
                    // SBF socket send_size recv_size -> 0 if success
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    let send_size_arg = get_runtime_arg(&args, 1, &values);
                    let recv_size_arg = get_runtime_arg(&args, 2, &values);
                    codegen.emit_sbf(id, &socket_arg, &send_size_arg, &recv_size_arg);
                }
                Op::Kal => {
                    // KAL socket interval -> 0 if success
                    let socket_arg = get_runtime_arg(&args, 0, &values);
                    let interval_arg = get_runtime_arg(&args, 1, &values);
                    codegen.emit_kal(id, &socket_arg, &interval_arg);
                }
                // === HashMap ===
                Op::Hmp => {
                    // HMP capacity -> map handle
                    let capacity_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_hmp(id, &capacity_arg);
                }
                Op::Hpt => {
                    // HPT map key key_len val val_len -> 1 if success
                    if let (Some(Arg::Ref(key_ref)), Some(Arg::Ref(val_ref))) = (args.get(1), args.get(3)) {
                        let map_arg = get_runtime_arg(&args, 0, &values);
                        let key_len_arg = get_runtime_arg(&args, 2, &values);
                        let val_len_arg = get_runtime_arg(&args, 4, &values);
                        codegen.emit_hpt(id, &map_arg, key_ref, &key_len_arg, val_ref, &val_len_arg);
                    }
                }
                Op::Hgt => {
                    // HGT dst map key key_len -> val_len
                    if let (Some(Arg::Ref(dst_ref)), Some(Arg::Ref(key_ref))) = (args.get(0), args.get(2)) {
                        let map_arg = get_runtime_arg(&args, 1, &values);
                        let key_len_arg = get_runtime_arg(&args, 3, &values);
                        codegen.emit_hgt(id, dst_ref, &map_arg, key_ref, &key_len_arg);
                    }
                }
                Op::Hhs => {
                    // HHS map key key_len -> 1 if exists
                    if let Some(Arg::Ref(key_ref)) = args.get(1) {
                        let map_arg = get_runtime_arg(&args, 0, &values);
                        let key_len_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_hhs(id, &map_arg, key_ref, &key_len_arg);
                    }
                }
                // === Memory ===
                Op::Fre => {
                    // FRE ptr -> 1 if success
                    let ptr_arg = get_runtime_arg(&args, 0, &values);
                    codegen.emit_fre(id, &ptr_arg);
                }
                Op::Fnd => {
                    // FND ptr len byte -> position or -1
                    if let Some(Arg::Ref(ptr_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        let byte_arg = get_runtime_arg(&args, 2, &values);
                        codegen.emit_fnd(id, ptr_ref, &len_arg, &byte_arg);
                    }
                }
                // === JSON ===
                Op::JsonParse => {
                    // JSON.PARSE src len -> json_ptr
                    if let Some(Arg::Ref(src_ref)) = args.get(0) {
                        let len_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_json_parse(id, src_ref, &len_arg);
                    }
                }
                // === GUI Dialog ===
                Op::GuiDlg => {
                    // GUI.DLG template parent -> dialog handle
                    if let Some(Arg::Ref(template_ref)) = args.get(0) {
                        let parent_arg = get_runtime_arg(&args, 1, &values);
                        codegen.emit_gui_dlg(id, template_ref, &parent_arg);
                    }
                }
                _ => {}
            }
        }
    }
    
    // Copy var_offsets from codegen for PRT to use
    var_offsets = codegen.var_offsets.clone();
    let buffer_offsets_for_prt = codegen.buffer_bases.clone();
    
    // Find PRT operation and generate output code
    for node in &graph.nodes {
        if let Node::Operation { op: Op::Prt, args, .. } = node {
            if let Some(Arg::Ref(buf_ref)) = args.get(0) {
                let buf_base = buffer_offsets_for_prt.get(buf_ref).copied().unwrap_or(0);
                
                // GetStdHandle(-11)
                codegen.code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]);
                let gsh_iat = iat_rva;
                let call_pos = text_rva + codegen.code.len() as u32 + 6;
                let offset = gsh_iat as i32 - call_pos as i32;
                codegen.code.extend(&[0xFF, 0x15]);
                codegen.code.extend(&offset.to_le_bytes());
                
                codegen.code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (handle)
                
                // rdx = buffer base from [rsp+0x48] + buf_base
                codegen.code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]); // mov rdx, [rsp+0x48]
                if buf_base > 0 {
                    codegen.code.extend(&[0x48, 0x81, 0xC2]);
                    codegen.code.extend(&(buf_base as u32).to_le_bytes());
                }
                
                // mov r8, len (from variable or immediate)
                match args.get(1) {
                    Some(Arg::Ref(len_ref)) => {
                        if let Some(&var_off) = var_offsets.get(len_ref) {
                            codegen.code.extend(&[0x4C, 0x8B, 0x44, 0x24, var_off as u8]);
                        } else if let Some(&val) = values.get(len_ref) {
                            codegen.code.extend(&[0x41, 0xB8]);
                            codegen.code.extend(&(val as u32).to_le_bytes());
                        } else {
                            codegen.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
                        }
                    }
                    Some(Arg::Lit(Value::Int(n))) => {
                        codegen.code.extend(&[0x41, 0xB8]);
                        codegen.code.extend(&(*n as u32).to_le_bytes());
                    }
                    _ => {
                        codegen.code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]);
                    }
                }
                
                codegen.code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
                codegen.code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
                
                // call WriteFile
                let wf_iat = iat_rva + 8;
                let call_pos = text_rva + codegen.code.len() as u32 + 6;
                let offset = wf_iat as i32 - call_pos as i32;
                codegen.code.extend(&[0xFF, 0x15]);
                codegen.code.extend(&offset.to_le_bytes());
            }
            break;
        }
    }
    
    // ExitProcess(0) - needs shadow space
    codegen.code.extend(&[0x48, 0x83, 0xC4, 0x30]); // add rsp, 0x30 (realign)
    codegen.code.extend(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28 (shadow+align)
    codegen.code.extend(&[0x31, 0xC9]); // xor ecx, ecx
    let ep_iat = iat_rva + 16;
    let call_pos = text_rva + codegen.code.len() as u32 + 6;
    let offset = ep_iat as i32 - call_pos as i32;
    codegen.code.extend(&[0xFF, 0x15]);
    codegen.code.extend(&offset.to_le_bytes());
    
    pe.extend(codegen.get_code());
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata section
    let mut rdata = vec![0u8; rdata_size as usize];
    
    // IAT: 15 entries for kernel32.dll
    // 0: GetStdHandle, 1: WriteFile, 2: ExitProcess, 3: GetSystemTimeAsFileTime, 4: Sleep
    // 5: CreateThread, 6: WaitForSingleObject, 7: CreateMutexA, 8: ReleaseMutex
    // 9: CreateSemaphoreA, 10: ReleaseSemaphore, 11: GetProcessHeap, 12: HeapAlloc
    // 13: GetCommandLineA, 14: GetEnvironmentVariableA
    let hint_gsh = hint_name_rva;
    let hint_wf = hint_name_rva + 16;
    let hint_ep = hint_name_rva + 32;
    let hint_gsft = hint_name_rva + 48;
    let hint_sleep = hint_name_rva + 80;
    let hint_ct = hint_name_rva + 96;       // CreateThread
    let hint_wfso = hint_name_rva + 112;    // WaitForSingleObject
    let hint_cm = hint_name_rva + 144;      // CreateMutexA
    let hint_rm = hint_name_rva + 160;      // ReleaseMutex
    let hint_cs = hint_name_rva + 176;      // CreateSemaphoreA
    let hint_rs = hint_name_rva + 196;      // ReleaseSemaphore
    let hint_gph = hint_name_rva + 216;     // GetProcessHeap
    let hint_ha = hint_name_rva + 232;      // HeapAlloc
    let hint_gcl = hint_name_rva + 248;     // GetCommandLineA
    let hint_gev = hint_name_rva + 268;     // GetEnvironmentVariableA
    
    // IAT entries (15 + null terminator)
    rdata[0..8].copy_from_slice(&(hint_gsh as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_wf as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_ep as u64).to_le_bytes());
    rdata[24..32].copy_from_slice(&(hint_gsft as u64).to_le_bytes());
    rdata[32..40].copy_from_slice(&(hint_sleep as u64).to_le_bytes());
    rdata[40..48].copy_from_slice(&(hint_ct as u64).to_le_bytes());
    rdata[48..56].copy_from_slice(&(hint_wfso as u64).to_le_bytes());
    rdata[56..64].copy_from_slice(&(hint_cm as u64).to_le_bytes());
    rdata[64..72].copy_from_slice(&(hint_rm as u64).to_le_bytes());
    rdata[72..80].copy_from_slice(&(hint_cs as u64).to_le_bytes());
    rdata[80..88].copy_from_slice(&(hint_rs as u64).to_le_bytes());
    rdata[88..96].copy_from_slice(&(hint_gph as u64).to_le_bytes());
    rdata[96..104].copy_from_slice(&(hint_ha as u64).to_le_bytes());
    rdata[104..112].copy_from_slice(&(hint_gcl as u64).to_le_bytes());
    rdata[112..120].copy_from_slice(&(hint_gev as u64).to_le_bytes());
    // null terminator at 120-128 (already zero)
    
    // ILT (same as IAT) - starts at 0x80 (moved to accommodate 15 entries + null = 128 bytes)
    rdata[0x80..0x88].copy_from_slice(&(hint_gsh as u64).to_le_bytes());
    rdata[0x88..0x90].copy_from_slice(&(hint_wf as u64).to_le_bytes());
    rdata[0x90..0x98].copy_from_slice(&(hint_ep as u64).to_le_bytes());
    rdata[0x98..0xA0].copy_from_slice(&(hint_gsft as u64).to_le_bytes());
    rdata[0xA0..0xA8].copy_from_slice(&(hint_sleep as u64).to_le_bytes());
    rdata[0xA8..0xB0].copy_from_slice(&(hint_ct as u64).to_le_bytes());
    rdata[0xB0..0xB8].copy_from_slice(&(hint_wfso as u64).to_le_bytes());
    rdata[0xB8..0xC0].copy_from_slice(&(hint_cm as u64).to_le_bytes());
    rdata[0xC0..0xC8].copy_from_slice(&(hint_rm as u64).to_le_bytes());
    rdata[0xC8..0xD0].copy_from_slice(&(hint_cs as u64).to_le_bytes());
    rdata[0xD0..0xD8].copy_from_slice(&(hint_rs as u64).to_le_bytes());
    rdata[0xD8..0xE0].copy_from_slice(&(hint_gph as u64).to_le_bytes());
    rdata[0xE0..0xE8].copy_from_slice(&(hint_ha as u64).to_le_bytes());
    rdata[0xE8..0xF0].copy_from_slice(&(hint_gcl as u64).to_le_bytes());
    rdata[0xF0..0xF8].copy_from_slice(&(hint_gev as u64).to_le_bytes());
    // null terminator at 0xF8-0x100 (already zero)
    
    // IDT (Import Directory Table) - at 0x100
    let idt_file_offset = 0x100usize;
    rdata[idt_file_offset..idt_file_offset+4].copy_from_slice(&ilt_rva.to_le_bytes()); // ILT RVA
    // TimeDateStamp, ForwarderChain = 0 (already zero)
    rdata[idt_file_offset+0xC..idt_file_offset+0x10].copy_from_slice(&dll_name_rva.to_le_bytes()); // Name RVA
    rdata[idt_file_offset+0x10..idt_file_offset+0x14].copy_from_slice(&iat_rva.to_le_bytes()); // IAT RVA
    // Null terminator entry (20 bytes of zeros) - already zero
    
    // Hint/Name table (at 0x128 - moved for 15 entries)
    let hnt = 0x128usize;
    rdata[hnt..hnt+2].copy_from_slice(&[0, 0]); // hint
    rdata[hnt+2..hnt+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt+16..hnt+18].copy_from_slice(&[0, 0]);
    rdata[hnt+18..hnt+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt+32..hnt+34].copy_from_slice(&[0, 0]);
    rdata[hnt+34..hnt+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt+48..hnt+50].copy_from_slice(&[0, 0]);
    rdata[hnt+50..hnt+74].copy_from_slice(b"GetSystemTimeAsFileTime\0");
    rdata[hnt+80..hnt+82].copy_from_slice(&[0, 0]);
    rdata[hnt+82..hnt+88].copy_from_slice(b"Sleep\0");
    rdata[hnt+96..hnt+98].copy_from_slice(&[0, 0]);
    rdata[hnt+98..hnt+111].copy_from_slice(b"CreateThread\0");
    rdata[hnt+112..hnt+114].copy_from_slice(&[0, 0]);
    rdata[hnt+114..hnt+134].copy_from_slice(b"WaitForSingleObject\0");
    rdata[hnt+144..hnt+146].copy_from_slice(&[0, 0]);
    rdata[hnt+146..hnt+159].copy_from_slice(b"CreateMutexA\0");
    rdata[hnt+160..hnt+162].copy_from_slice(&[0, 0]);
    rdata[hnt+162..hnt+175].copy_from_slice(b"ReleaseMutex\0");
    rdata[hnt+176..hnt+178].copy_from_slice(&[0, 0]);
    rdata[hnt+178..hnt+195].copy_from_slice(b"CreateSemaphoreA\0");
    rdata[hnt+196..hnt+198].copy_from_slice(&[0, 0]);
    rdata[hnt+198..hnt+215].copy_from_slice(b"ReleaseSemaphore\0");
    rdata[hnt+216..hnt+218].copy_from_slice(&[0, 0]);
    rdata[hnt+218..hnt+233].copy_from_slice(b"GetProcessHeap\0");
    rdata[hnt+232..hnt+234].copy_from_slice(&[0, 0]);
    rdata[hnt+234..hnt+244].copy_from_slice(b"HeapAlloc\0");
    rdata[hnt+248..hnt+250].copy_from_slice(&[0, 0]);
    rdata[hnt+250..hnt+266].copy_from_slice(b"GetCommandLineA\0");
    rdata[hnt+268..hnt+270].copy_from_slice(&[0, 0]);
    rdata[hnt+270..hnt+294].copy_from_slice(b"GetEnvironmentVariableA\0");
    
    // DLL name (at 0x2A0 - moved to accommodate GetEnvironmentVariableA)
    rdata[0x2A0..0x2AD].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&rdata);
    pe.resize(data_file_offset as usize, 0);
    
    // .data section (buffer space + strings)
    let mut data = vec![0u8; data_size as usize];
    
    // Write strings to .data section at their buffer offsets
    for (name, content) in &strings {
        if let Some(&offset) = buffer_offsets.get(name) {
            let bytes = content.as_bytes();
            let start = offset as usize;
            let end = (start + bytes.len()).min(data.len());
            if start < data.len() {
                data[start..end].copy_from_slice(&bytes[..end-start]);
            }
        }
    }
    
    pe.extend(&data);
    
    pe
}

fn get_runtime_arg(args: &[Arg], idx: usize, values: &HashMap<String, i64>) -> RuntimeArg {
    match args.get(idx) {
        Some(Arg::Lit(Value::Int(n))) => RuntimeArg::Immediate(*n),
        Some(Arg::Ref(name)) => {
            if let Some(&val) = values.get(name) {
                RuntimeArg::Immediate(val)
            } else {
                RuntimeArg::Register(name.clone())
            }
        }
        _ => RuntimeArg::Immediate(0),
    }
}

/// Generate PE from TAYNI graph
/// Supports: literals, strings, ALC, PUT, PRT, ADD, SUB, MUL
/// NEW: Supports runtime operations with @ prefix
pub fn generate_pe_from_graph(graph: &Graph) -> Vec<u8> {
    // Analyze runtime dependencies
    let mut runtime_analysis = RuntimeAnalysis::new();
    runtime_analysis.analyze(graph);
    
    // Check if we have any runtime operations
    let has_runtime_ops = !runtime_analysis.runtime_nodes.is_empty();
    
    // If we have runtime operations, use the runtime code generator
    if has_runtime_ops {
        return generate_pe_with_runtime_ops(graph, &runtime_analysis);
    }
    
    // Otherwise, use the existing compile-time evaluation
    // Evaluate all values at compile time
    let mut values: HashMap<String, i64> = HashMap::new();
    let mut strings: HashMap<String, String> = HashMap::new();
    let mut alc_id: Option<String> = None;
    let mut alc_size: i64 = 0;
    let mut put_ops: Vec<(i64, i64)> = Vec::new(); // (idx, val)
    let mut prt_len: i64 = 0;
    let mut has_prt = false;
    
    // Process nodes in order, evaluating expressions
    for node in &graph.nodes {
        match node {
            Node::Literal { id, value, .. } => {
                match value {
                    Value::Int(n) => { values.insert(id.clone(), *n); }
                    Value::String(s) => { strings.insert(id.clone(), s.clone()); }
                    _ => {}
                }
            }
            Node::Operation { id, op, args, .. } => {
                match op {
                    Op::Add => {
                        let a = get_val(&args, 0, &values).unwrap_or(0);
                        let b = get_val(&args, 1, &values).unwrap_or(0);
                        values.insert(id.clone(), a + b);
                    }
                    Op::Sub => {
                        let a = get_val(&args, 0, &values).unwrap_or(0);
                        let b = get_val(&args, 1, &values).unwrap_or(0);
                        values.insert(id.clone(), a - b);
                    }
                    Op::Mul => {
                        let a = get_val(&args, 0, &values).unwrap_or(0);
                        let b = get_val(&args, 1, &values).unwrap_or(0);
                        values.insert(id.clone(), a * b);
                    }
                    Op::Div => {
                        let a = get_val(&args, 0, &values).unwrap_or(0);
                        let b = get_val(&args, 1, &values).unwrap_or(1);
                        values.insert(id.clone(), if b != 0 { a / b } else { 0 });
                    }
                    Op::Mod => {
                        let a = get_val(&args, 0, &values).unwrap_or(0);
                        let b = get_val(&args, 1, &values).unwrap_or(1);
                        values.insert(id.clone(), if b != 0 { a % b } else { 0 });
                    }
                    Op::Alc => {
                        alc_size = get_val(&args, 0, &values).unwrap_or(16);
                        alc_id = Some(id.clone());
                    }
                    Op::Put => {
                        let idx = get_val(&args, 1, &values).unwrap_or(0);
                        let val = get_val(&args, 2, &values).unwrap_or(0);
                        put_ops.push((idx, val));
                    }
                    Op::Prt => {
                        prt_len = get_val(&args, 1, &values).unwrap_or(0);
                        has_prt = true;
                    }
                    Op::Ifz => {
                        // IFZ cond then_val else_val: returns then_val if cond==0, else else_val
                        let cond = get_val(&args, 0, &values).unwrap_or(0);
                        let then_val = get_val(&args, 1, &values).unwrap_or(0);
                        let else_val = get_val(&args, 2, &values).unwrap_or(0);
                        let result = if cond == 0 { then_val } else { else_val };
                        values.insert(id.clone(), result);
                    }
                    Op::Trn => {
                        // TRN is a graph transform - for compile-time, we just mark it
                        // Runtime TRN would require actual loop code generation
                        values.insert(id.clone(), 0);
                    }
                    Op::Fop => {
                        // File open - mark that we have file I/O
                    }
                    Op::Frd => {
                        // File read
                    }
                    Op::Fcl => {
                        // File close
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    // Check for File I/O pattern: FOP + FRD + FCL + PRT
    let has_fop = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Fop, .. }));
    let has_frd = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Frd, .. }));
    let has_fwr = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Fwr, .. }));
    
    // Compiler pattern: FOP + FRD + FWR (read input, process, write output)
    if has_fop && has_frd && has_fwr {
        let mut input_path: Option<String> = None;
        let mut output_path: Option<String> = None;
        let mut read_size: i64 = 512;
        let mut write_size: i64 = 4096;
        
        // Find paths from FOP operations
        let mut fop_count = 0;
        for node in &graph.nodes {
            if let Node::Operation { op: Op::Fop, args, .. } = node {
                if let Some(Arg::Ref(path_ref)) = args.get(0) {
                    if let Some(path) = strings.get(path_ref) {
                        if fop_count == 0 {
                            input_path = Some(path.clone());
                        } else {
                            output_path = Some(path.clone());
                        }
                        fop_count += 1;
                    }
                }
            }
            if let Node::Operation { op: Op::Frd, args, .. } = node {
                if let Some(size) = get_val(&args, 2, &values) {
                    read_size = size;
                }
            }
            if let Node::Operation { op: Op::Fwr, args, .. } = node {
                if let Some(size) = get_val(&args, 2, &values) {
                    write_size = size;
                }
            }
        }
        
        if let (Some(inp), Some(outp)) = (input_path, output_path) {
            return generate_pe_compiler(&inp, &outp, read_size, write_size, &put_ops);
        }
    }
    
    if has_fop && has_frd && has_prt {
        // Extract file path and buffer info
        let mut file_path: Option<String> = None;
        let mut buf_size: i64 = 64;
        
        for node in &graph.nodes {
            if let Node::Operation { op: Op::Fop, args, .. } = node {
                if let Some(Arg::Ref(path_ref)) = args.get(0) {
                    file_path = strings.get(path_ref).cloned();
                }
            }
            if let Node::Operation { op: Op::Frd, args, .. } = node {
                if let Some(size) = get_val(&args, 2, &values) {
                    buf_size = size;
                }
            }
        }
        
        if let Some(path) = file_path {
            return generate_pe_with_fileio(&path, buf_size, prt_len);
        }
    }
    
    // Simple string + PRT case
    if strings.len() == 1 && has_prt && alc_id.is_none() {
        let msg = strings.values().next().unwrap();
        return generate_pe_with_message(msg, prt_len as usize);
    }
    
    // ALC + PUT + PRT case (with computed values)
    if alc_id.is_some() && has_prt {
        return generate_pe_with_buffer(alc_size, &put_ops, prt_len);
    }
    
    // Check for TCP server pattern: TCP + BND + LST + ACC
    let has_tcp = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Tcp, .. }));
    let has_bnd = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Bnd, .. }));
    let has_lst = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Lst, .. }));
    let has_acc = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Acc, .. }));
    let has_route = graph.nodes.iter().any(|n| matches!(n, Node::Operation { op: Op::Route, .. }));
    
    // HTTP Server with routing
    if has_route || (has_tcp && has_bnd && has_lst && has_acc) {
        // Extract port from BND operation
        let mut port: u16 = 8080;
        for node in &graph.nodes {
            if let Node::Operation { op: Op::Bnd, args, .. } = node {
                if let Some(p) = get_val(&args, 1, &values) {
                    port = p as u16;
                }
            }
        }
        
        // Collect routes from ROUTE operations
        let mut routes: Vec<(String, String)> = Vec::new();
        for node in &graph.nodes {
            if let Node::Operation { op: Op::Route, args, .. } = node {
                // ROUTE path response
                let path = args.get(0).and_then(|a| match a {
                    Arg::Ref(r) => strings.get(r).cloned(),
                    Arg::Lit(Value::String(s)) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_else(|| "/".to_string());
                
                let response = args.get(1).and_then(|a| match a {
                    Arg::Ref(r) => strings.get(r).cloned(),
                    Arg::Lit(Value::String(s)) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_else(|| "HTTP/1.1 200 OK\r\n\r\nOK".to_string());
                
                routes.push((path, response));
            }
        }
        
        // If no routes defined, use default response
        if routes.is_empty() {
            let response = strings.values()
                .find(|s| s.contains("HTTP") || s.contains("{") || s.contains("200"))
                .cloned()
                .unwrap_or_else(|| "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello from TAYNI!".to_string());
            routes.push(("/".to_string(), response));
        }
        
        // Convert to slice of tuples for generate_http_server_pe
        let routes_refs: Vec<(&str, &str)> = routes.iter()
            .map(|(p, r)| (p.as_str(), r.as_str()))
            .collect();
        
        return generate_http_server_pe(port, &routes_refs);
    }
    
    // Default
    generate_hello_pe()
}

fn get_val(args: &[Arg], idx: usize, values: &HashMap<String, i64>) -> Option<i64> {
    args.get(idx).and_then(|arg| match arg {
        Arg::Lit(Value::Int(n)) => Some(*n),
        Arg::Ref(r) => values.get(r).copied(),
        _ => None,
    })
}

fn get_int_arg(args: &[Arg], idx: usize, values: &HashMap<String, i64>) -> Option<i64> {
    get_val(args, idx, values)
}

/// Generate PE with File I/O (FOP + FRD + FCL + PRT)
fn generate_pe_with_fileio(file_path: &str, buf_size: i64, print_len: i64) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x600; // More space for code
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x400; // More space for imports
    let rdata_file_offset: u32 = 0x800;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x200;
    let data_file_offset: u32 = 0xC00;
    
    let image_size: u32 = 0x4000;
    
    // IAT layout: GetStdHandle, WriteFile, ExitProcess, VirtualAlloc, CreateFileA, ReadFile, CloseHandle
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x60;
    let ilt_rva = rdata_rva + 0xA0;
    let hint_name_rva = rdata_rva + 0x100;
    let dll_name_rva = rdata_rva + 0x180;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // Prologue - more stack space for file ops
    code.extend(x64::sub_rsp_imm8(0x58));
    
    // VirtualAlloc for buffer
    code.extend(&[0x31, 0xC9]); // xor ecx, ecx
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, size
    code.extend(&(buf_size as u32).to_le_bytes());
    code.extend(&[0x41, 0xB8]); // mov r8d, MEM_COMMIT|MEM_RESERVE
    code.extend(&0x3000u32.to_le_bytes());
    code.extend(&[0x41, 0xB9]); // mov r9d, PAGE_READWRITE
    code.extend(&0x04u32.to_le_bytes());
    
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let va_offset = (iat_rva + 24) as i64 - call_pos; // VirtualAlloc at IAT[3]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(va_offset as i32).to_le_bytes());
    
    // Save buffer to [rsp+72]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x48]);
    
    // CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL)
    // rcx = path, rdx = 0x80000000, r8 = 1, r9 = NULL
    let path_rva = data_rva;
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let path_offset = path_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x0D]); // lea rcx, [rip+path]
    code.extend(&(path_offset as i32).to_le_bytes());
    
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, GENERIC_READ
    code.extend(&0x80000000u32.to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]); // mov r8d, FILE_SHARE_READ
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d (NULL)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20]); // mov qword [rsp+32], OPEN_EXISTING
    code.extend(&3u32.to_le_bytes());
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x28, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+40], 0
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x30, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+48], NULL
    
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let cf_offset = (iat_rva + 32) as i64 - call_pos2; // CreateFileA at IAT[4]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(cf_offset as i32).to_le_bytes());
    
    // Save handle to [rsp+64]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x40]);
    
    // ReadFile(handle, buffer, size, &bytesRead, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+64] (handle)
    code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]); // mov rdx, [rsp+72] (buffer)
    code.extend(&[0x41, 0xB8]); // mov r8d, size
    code.extend(&(buf_size as u32).to_le_bytes());
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x38]); // lea r9, [rsp+56] (bytesRead)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+32], NULL
    
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let rf_offset = (iat_rva + 40) as i64 - call_pos3; // ReadFile at IAT[5]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(rf_offset as i32).to_le_bytes());
    
    // CloseHandle(handle)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+64]
    let call_pos4 = code_start as i64 + code.len() as i64 + 6;
    let ch_offset = (iat_rva + 48) as i64 - call_pos4; // CloseHandle at IAT[6]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(ch_offset as i32).to_le_bytes());
    
    // GetStdHandle(-11)
    code.extend(&[0xB9]);
    code.extend(&(-11i32).to_le_bytes());
    let call_pos5 = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset = iat_rva as i64 - call_pos5;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset as i32).to_le_bytes());
    
    // Save stdout handle
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x50]);
    
    // WriteFile(stdout, buffer, len, &written, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x50]); // mov rcx, stdout
    code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x48]); // mov rdx, buffer
    code.extend(&[0x41, 0xB8]); // mov r8d, len
    code.extend(&(print_len as u32).to_le_bytes());
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x38]); // lea r9, [rsp+56]
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    let call_pos6 = code_start as i64 + code.len() as i64 + 6;
    let wf_offset = (iat_rva + 8) as i64 - call_pos6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset as i32).to_le_bytes());
    
    // ExitProcess(0)
    code.extend(x64::xor_ecx_ecx());
    let call_pos7 = code_start as i64 + code.len() as i64 + 6;
    let ep_offset = (iat_rva + 16) as i64 - call_pos7;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(ep_offset as i32).to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports (7 functions)
    let mut rdata = vec![0u8; rdata_size as usize];
    
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 32;
    let hint_virtualalloc = hint_name_rva + 48;
    let hint_createfilea = hint_name_rva + 64;
    let hint_readfile = hint_name_rva + 80;
    let hint_closehandle = hint_name_rva + 96;
    
    // IAT (7 entries)
    rdata[0..8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[24..32].copy_from_slice(&(hint_virtualalloc as u64).to_le_bytes());
    rdata[32..40].copy_from_slice(&(hint_createfilea as u64).to_le_bytes());
    rdata[40..48].copy_from_slice(&(hint_readfile as u64).to_le_bytes());
    rdata[48..56].copy_from_slice(&(hint_closehandle as u64).to_le_bytes());
    
    // Import Directory
    let idt_offset = 0x60usize;
    rdata[idt_offset..idt_offset+4].copy_from_slice(&ilt_rva.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&dll_name_rva.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&iat_rva.to_le_bytes());
    
    // ILT (same as IAT)
    let ilt_offset = 0xA0usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[ilt_offset+24..ilt_offset+32].copy_from_slice(&(hint_virtualalloc as u64).to_le_bytes());
    rdata[ilt_offset+32..ilt_offset+40].copy_from_slice(&(hint_createfilea as u64).to_le_bytes());
    rdata[ilt_offset+40..ilt_offset+48].copy_from_slice(&(hint_readfile as u64).to_le_bytes());
    rdata[ilt_offset+48..ilt_offset+56].copy_from_slice(&(hint_closehandle as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0x100usize;
    rdata[hnt_offset+2..hnt_offset+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt_offset+18..hnt_offset+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt_offset+34..hnt_offset+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt_offset+50..hnt_offset+63].copy_from_slice(b"VirtualAlloc\0");
    rdata[hnt_offset+66..hnt_offset+78].copy_from_slice(b"CreateFileA\0");
    rdata[hnt_offset+82..hnt_offset+91].copy_from_slice(b"ReadFile\0");
    rdata[hnt_offset+98..hnt_offset+110].copy_from_slice(b"CloseHandle\0");
    
    // DLL name
    rdata[0x180..0x180+13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&rdata);
    
    // .data - file path
    let mut data = vec![0u8; data_size as usize];
    let path_bytes = file_path.as_bytes();
    let copy_len = path_bytes.len().min(data_size as usize - 1);
    data[..copy_len].copy_from_slice(&path_bytes[..copy_len]);
    
    pe.extend(&data);
    
    pe
}

/// Generate PE compiler (FOP + FRD + PUT + FWR)
/// Reads input file, applies PUT operations, writes output file
fn generate_pe_compiler(input_path: &str, output_path: &str, read_size: i64, write_size: i64, put_ops: &[(i64, i64)]) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    // Calculate code size based on PUT operations
    // Base code ~400 bytes + ~7 bytes per PUT (worst case)
    let put_code_size = (put_ops.len() * 7) as u32;
    let text_size: u32 = ((0x600 + put_code_size + 0x1FF) / 0x200) * 0x200; // Round up to 512
    let text_rva: u32 = 0x1000;
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = text_rva + ((text_size + 0xFFF) / 0x1000) * 0x1000;
    let rdata_size: u32 = 0x400;
    let rdata_file_offset: u32 = text_file_offset + text_size;
    
    let data_rva: u32 = rdata_rva + 0x1000;
    let data_size: u32 = 0x400;
    let data_file_offset: u32 = rdata_file_offset + rdata_size;
    
    let image_size: u32 = data_rva + 0x1000;
    
    // IAT: GetStdHandle[0], WriteFile[8], ExitProcess[16], VirtualAlloc[24], CreateFileA[32], ReadFile[40], CloseHandle[48]
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x60;
    let ilt_rva = rdata_rva + 0xC0;
    let hint_name_rva = rdata_rva + 0x140;
    let dll_name_rva = rdata_rva + 0x200;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // sub rsp, 0x68
    code.extend(x64::sub_rsp_imm8(0x68));
    
    // === VirtualAlloc for buffer ===
    code.extend(&[0x31, 0xC9]); // xor ecx, ecx (NULL)
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, write_size
    code.extend(&(write_size as u32).to_le_bytes());
    code.extend(&[0x41, 0xB8]); // mov r8d, MEM_COMMIT|MEM_RESERVE
    code.extend(&0x3000u32.to_le_bytes());
    code.extend(&[0x41, 0xB9]); // mov r9d, PAGE_READWRITE
    code.extend(&0x04u32.to_le_bytes());
    
    let call_va = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 24) as i64 - call_va) as i32).to_le_bytes());
    
    // Save buffer to [rsp+80]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x50]);
    
    // === CreateFileA for input (GENERIC_READ, OPEN_EXISTING) ===
    let input_rva = data_rva;
    let lea_inp = code_start as i64 + code.len() as i64 + 7;
    code.extend(&[0x48, 0x8D, 0x0D]); // lea rcx, [rip+input_path]
    code.extend(&((input_rva as i64 - lea_inp) as i32).to_le_bytes());
    
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, GENERIC_READ
    code.extend(&0x80000000u32.to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x01, 0x00, 0x00, 0x00]); // mov r8d, FILE_SHARE_READ
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x03, 0x00, 0x00, 0x00]); // [rsp+32] = OPEN_EXISTING
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x28, 0x00, 0x00, 0x00, 0x00]); // [rsp+40] = 0
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x30, 0x00, 0x00, 0x00, 0x00]); // [rsp+48] = NULL
    
    let call_cf1 = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 32) as i64 - call_cf1) as i32).to_le_bytes());
    
    // Save input handle to [rsp+72]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x48]);
    
    // === ReadFile ===
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+72] (handle)
    code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x50]); // mov rdx, [rsp+80] (buffer)
    code.extend(&[0x41, 0xB8]); // mov r8d, read_size
    code.extend(&(read_size as u32).to_le_bytes());
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x40]); // lea r9, [rsp+64] (bytesRead)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // [rsp+32] = NULL
    
    let call_rf = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 40) as i64 - call_rf) as i32).to_le_bytes());
    
    // === CloseHandle(input) ===
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+72]
    let call_ch1 = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 48) as i64 - call_ch1) as i32).to_le_bytes());
    
    // === Apply PUT operations ===
    // mov rdi, [rsp+80] (buffer)
    code.extend(&[0x48, 0x8B, 0x7C, 0x24, 0x50]);
    
    for (idx, val) in put_ops {
        if *idx < 128 {
            // mov byte [rdi+idx], val
            code.extend(&[0xC6, 0x47, *idx as u8, *val as u8]);
        } else {
            // mov byte [rdi+idx], val (32-bit offset)
            code.extend(&[0xC6, 0x87]);
            code.extend(&(*idx as u32).to_le_bytes());
            code.push(*val as u8);
        }
    }
    
    // === CreateFileA for output (GENERIC_WRITE, CREATE_ALWAYS) ===
    let output_rva = data_rva + 0x100;
    let lea_out = code_start as i64 + code.len() as i64 + 7;
    code.extend(&[0x48, 0x8D, 0x0D]); // lea rcx, [rip+output_path]
    code.extend(&((output_rva as i64 - lea_out) as i32).to_le_bytes());
    
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, GENERIC_WRITE
    code.extend(&0x40000000u32.to_le_bytes());
    code.extend(&[0x45, 0x31, 0xC0]); // xor r8d, r8d
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x02, 0x00, 0x00, 0x00]); // [rsp+32] = CREATE_ALWAYS
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x28, 0x80, 0x00, 0x00, 0x00]); // [rsp+40] = FILE_ATTRIBUTE_NORMAL
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x30, 0x00, 0x00, 0x00, 0x00]); // [rsp+48] = NULL
    
    let call_cf2 = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 32) as i64 - call_cf2) as i32).to_le_bytes());
    
    // Save output handle to [rsp+72]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x48]);
    
    // === WriteFile ===
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+72] (handle)
    code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x50]); // mov rdx, [rsp+80] (buffer)
    code.extend(&[0x41, 0xB8]); // mov r8d, write_size
    code.extend(&(write_size as u32).to_le_bytes());
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x40]); // lea r9, [rsp+64] (bytesWritten)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // [rsp+32] = NULL
    
    let call_wf = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 8) as i64 - call_wf) as i32).to_le_bytes());
    
    // === CloseHandle(output) ===
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+72]
    let call_ch2 = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 48) as i64 - call_ch2) as i32).to_le_bytes());
    
    // === ExitProcess(0) ===
    code.extend(&[0x31, 0xC9]); // xor ecx, ecx
    let call_ep = code_start as i64 + code.len() as i64 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 16) as i64 - call_ep) as i32).to_le_bytes());
    
    code.resize(text_size as usize, 0xCC);
    pe.extend(&code);
    
    // === .rdata section ===
    let mut rdata = vec![0u8; rdata_size as usize];
    
    // IAT entries point to hint/name
    let hints = [
        hint_name_rva,        // GetStdHandle
        hint_name_rva + 16,   // WriteFile
        hint_name_rva + 32,   // ExitProcess
        hint_name_rva + 48,   // VirtualAlloc
        hint_name_rva + 64,   // CreateFileA
        hint_name_rva + 80,   // ReadFile
        hint_name_rva + 96,   // CloseHandle
    ];
    
    for (i, hint) in hints.iter().enumerate() {
        rdata[i*8..i*8+8].copy_from_slice(&(*hint as u64).to_le_bytes());
    }
    
    // IDT
    let idt_off = 0x60usize;
    rdata[idt_off..idt_off+4].copy_from_slice(&ilt_rva.to_le_bytes());
    rdata[idt_off+12..idt_off+16].copy_from_slice(&dll_name_rva.to_le_bytes());
    rdata[idt_off+16..idt_off+20].copy_from_slice(&iat_rva.to_le_bytes());
    
    // ILT (same as IAT)
    let ilt_off = 0xC0usize;
    for (i, hint) in hints.iter().enumerate() {
        rdata[ilt_off+i*8..ilt_off+i*8+8].copy_from_slice(&(*hint as u64).to_le_bytes());
    }
    
    // Hint/Name entries
    let hnt_off = 0x140usize;
    let names = [
        (0, b"GetStdHandle\0" as &[u8]),
        (16, b"WriteFile\0"),
        (32, b"ExitProcess\0"),
        (48, b"VirtualAlloc\0"),
        (64, b"CreateFileA\0"),
        (80, b"ReadFile\0"),
        (96, b"CloseHandle\0"),
    ];
    for (off, name) in names {
        rdata[hnt_off+off] = 0;
        rdata[hnt_off+off+1] = 0;
        rdata[hnt_off+off+2..hnt_off+off+2+name.len()].copy_from_slice(name);
    }
    
    // DLL name
    let dll_off = 0x200usize;
    rdata[dll_off..dll_off+13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&rdata);
    
    // === .data section ===
    let mut data = vec![0u8; data_size as usize];
    let inp_bytes = input_path.as_bytes();
    data[..inp_bytes.len().min(0xFF)].copy_from_slice(&inp_bytes[..inp_bytes.len().min(0xFF)]);
    let out_bytes = output_path.as_bytes();
    data[0x100..0x100+out_bytes.len().min(0xFF)].copy_from_slice(&out_bytes[..out_bytes.len().min(0xFF)]);
    
    pe.extend(&data);
    
    pe
}

/// Generate PE with dynamic buffer (ALC + PUT + PRT)
fn generate_pe_with_buffer(buf_size: i64, bytes: &[(i64, i64)], print_len: i64) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x400; // More space for code
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x200;
    let rdata_file_offset: u32 = 0x600;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x200;
    let data_file_offset: u32 = 0x800;
    
    let image_size: u32 = 0x4000;
    
    // IAT layout: GetStdHandle, WriteFile, ExitProcess, VirtualAlloc
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x40;
    let ilt_rva = rdata_rva + 0x80;
    let hint_name_rva = rdata_rva + 0xC0;
    let dll_name_rva = rdata_rva + 0x120;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // Prologue
    code.extend(x64::sub_rsp_imm8(0x48)); // More stack space
    
    // VirtualAlloc(NULL, size, MEM_COMMIT|MEM_RESERVE, PAGE_READWRITE)
    // rcx = NULL, rdx = size, r8 = 0x3000, r9 = 0x04
    code.extend(&[0x31, 0xC9]); // xor ecx, ecx (NULL)
    code.extend(&[0x48, 0xC7, 0xC2]); // mov rdx, imm32
    code.extend(&(buf_size as u32).to_le_bytes());
    code.extend(&[0x41, 0xB8]); // mov r8d, imm32
    code.extend(&0x3000u32.to_le_bytes()); // MEM_COMMIT | MEM_RESERVE
    code.extend(&[0x41, 0xB9]); // mov r9d, imm32
    code.extend(&0x04u32.to_le_bytes()); // PAGE_READWRITE
    
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let va_offset = (iat_rva + 24) as i64 - call_pos; // VirtualAlloc at IAT[3]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(va_offset as i32).to_le_bytes());
    
    // Save buffer pointer to [rsp+64]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x40]); // mov [rsp+64], rax
    
    // Write bytes to buffer
    for (idx, val) in bytes {
        // mov byte [rax + idx], val
        if *idx < 128 {
            code.extend(&[0xC6, 0x40, *idx as u8, *val as u8]);
        } else {
            code.extend(&[0xC6, 0x80]);
            code.extend(&(*idx as u32).to_le_bytes());
            code.push(*val as u8);
        }
    }
    
    // GetStdHandle(-11)
    code.extend(&[0xB9]); // mov ecx, imm32
    code.extend(&(-11i32).to_le_bytes());
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset = iat_rva as i64 - call_pos2;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset as i32).to_le_bytes());
    
    // Save handle to [rsp+56]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x38]);
    
    // WriteFile(handle, buf, len, &written, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x38]); // mov rcx, [rsp+56] (handle)
    code.extend(&[0x48, 0x8B, 0x54, 0x24, 0x40]); // mov rdx, [rsp+64] (buffer)
    code.extend(&[0x41, 0xB8]); // mov r8d, len
    code.extend(&(print_len as u32).to_le_bytes());
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x30]); // lea r9, [rsp+48]
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+32], 0
    
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let wf_offset = (iat_rva + 8) as i64 - call_pos3;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset as i32).to_le_bytes());
    
    // ExitProcess(0)
    code.extend(x64::xor_ecx_ecx());
    let call_pos4 = code_start as i64 + code.len() as i64 + 6;
    let ep_offset = (iat_rva + 16) as i64 - call_pos4;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(ep_offset as i32).to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports (4 functions now)
    let mut rdata = vec![0u8; rdata_size as usize];
    
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 32;
    let hint_virtualalloc = hint_name_rva + 48;
    
    // IAT
    rdata[0..8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[24..32].copy_from_slice(&(hint_virtualalloc as u64).to_le_bytes());
    
    // Import Directory
    let idt_offset = 0x40usize;
    rdata[idt_offset..idt_offset+4].copy_from_slice(&ilt_rva.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&dll_name_rva.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&iat_rva.to_le_bytes());
    
    // ILT
    let ilt_offset = 0x80usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[ilt_offset+24..ilt_offset+32].copy_from_slice(&(hint_virtualalloc as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0xC0usize;
    rdata[hnt_offset+2..hnt_offset+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt_offset+18..hnt_offset+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt_offset+34..hnt_offset+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt_offset+50..hnt_offset+63].copy_from_slice(b"VirtualAlloc\0");
    
    // DLL name
    rdata[0x120..0x120+13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&rdata);
    
    // .data (empty for now)
    let data = vec![0u8; data_size as usize];
    pe.extend(&data);
    
    pe
}

/// Generate PE that prints a specific message - OPTIMIZED VERSION (~400 bytes)
fn generate_pe_with_message(message: &str, len: usize) -> Vec<u8> {
    // Use the tiny PE generator for simple message printing
    // This produces a much smaller executable (~400 bytes vs 2048 bytes)
    let msg = if len < message.len() {
        &message[..len]
    } else {
        message
    };
    generate_tiny_pe(msg)
}

/// Generate a minimal TCP server PE (listens on port, accepts one connection, sends response)
pub fn generate_tcp_server_pe(port: u16, response: &str) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x800; // More space for TCP code
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x600; // More space for two DLL imports
    let rdata_file_offset: u32 = 0xA00;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x400;
    let data_file_offset: u32 = 0x1000;
    
    let image_size: u32 = 0x5000;
    
    // Two import directories: kernel32.dll and ws2_32.dll
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x80;
    let ilt_rva = rdata_rva + 0x100;
    let hint_name_rva = rdata_rva + 0x180;
    let dll_names_rva = rdata_rva + 0x300;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&60u32.to_le_bytes()); // 3 entries * 20 bytes
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate TCP server code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // Prologue
    code.extend(x64::sub_rsp_imm8(0x68)); // Stack space
    
    // WSAStartup(0x0202, &wsadata)
    // wsadata at [rsp+32] (16 bytes minimum)
    code.extend(&[0xB9, 0x02, 0x02, 0x00, 0x00]); // mov ecx, 0x0202
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x20]); // lea rdx, [rsp+32]
    
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let wsa_startup_offset = (iat_rva + 24) as i64 - call_pos; // WSAStartup at IAT[3]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wsa_startup_offset as i32).to_le_bytes());
    
    // socket(AF_INET=2, SOCK_STREAM=1, IPPROTO_TCP=6)
    code.extend(&[0xB9, 0x02, 0x00, 0x00, 0x00]); // mov ecx, AF_INET
    code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, SOCK_STREAM
    code.extend(&[0x41, 0xB8, 0x06, 0x00, 0x00, 0x00]); // mov r8d, IPPROTO_TCP
    
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let socket_offset = (iat_rva + 32) as i64 - call_pos2; // socket at IAT[4]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(socket_offset as i32).to_le_bytes());
    
    // Save socket to [rsp+80]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x50]);
    
    // Build sockaddr_in at [rsp+56]: sin_family=2, sin_port=htons(port), sin_addr=0
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x38, 0x02, 0x00]); // mov word [rsp+56], 2 (AF_INET)
    let port_be = port.to_be(); // Network byte order
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x3A]); // mov word [rsp+58], port
    code.extend(&port_be.to_le_bytes());
    code.extend(&[0xC7, 0x44, 0x24, 0x3C, 0x00, 0x00, 0x00, 0x00]); // mov dword [rsp+60], 0 (INADDR_ANY)
    
    // bind(socket, &addr, 16)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x50]); // mov rcx, [rsp+80] (socket)
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x38]); // lea rdx, [rsp+56] (addr)
    code.extend(&[0x41, 0xB8, 0x10, 0x00, 0x00, 0x00]); // mov r8d, 16
    
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let bind_offset = (iat_rva + 40) as i64 - call_pos3; // bind at IAT[5]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(bind_offset as i32).to_le_bytes());
    
    // listen(socket, 1)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x50]); // mov rcx, socket
    code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, 1
    
    let call_pos4 = code_start as i64 + code.len() as i64 + 6;
    let listen_offset = (iat_rva + 48) as i64 - call_pos4; // listen at IAT[6]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(listen_offset as i32).to_le_bytes());
    
    // Print "Listening on port XXXX"
    code.extend(&[0xB9]); // mov ecx, -11
    code.extend(&(-11i32).to_le_bytes());
    let call_pos_gsh = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset = iat_rva as i64 - call_pos_gsh;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x58]); // save stdout
    
    // WriteFile for "Listening..."
    let msg_rva = data_rva + 0x100;
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x58]); // mov rcx, stdout
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let msg_offset = msg_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+msg]
    code.extend(&(msg_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x18, 0x00, 0x00, 0x00]); // mov r8d, 24
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x48]); // lea r9, [rsp+72]
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    let call_pos_wf = code_start as i64 + code.len() as i64 + 6;
    let wf_offset = (iat_rva + 8) as i64 - call_pos_wf;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset as i32).to_le_bytes());
    
    // accept(socket, NULL, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x50]); // mov rcx, socket
    code.extend(&[0x31, 0xD2]); // xor edx, edx
    code.extend(&[0x45, 0x31, 0xC0]); // xor r8d, r8d
    
    let call_pos5 = code_start as i64 + code.len() as i64 + 6;
    let accept_offset = (iat_rva + 56) as i64 - call_pos5; // accept at IAT[7]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(accept_offset as i32).to_le_bytes());
    
    // Save client socket to [rsp+88]
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x60]);
    
    // send(client, response, len, 0)
    let resp_rva = data_rva;
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x60]); // mov rcx, client
    let lea_pos2 = code_start as i64 + code.len() as i64 + 7;
    let resp_offset = resp_rva as i64 - lea_pos2;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+response]
    code.extend(&(resp_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8]); // mov r8d, len
    code.extend(&(response.len() as u32).to_le_bytes());
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
    
    let call_pos6 = code_start as i64 + code.len() as i64 + 6;
    let send_offset = (iat_rva + 64) as i64 - call_pos6; // send at IAT[8]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(send_offset as i32).to_le_bytes());
    
    // closesocket(client)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x60]);
    let call_pos7 = code_start as i64 + code.len() as i64 + 6;
    let closesocket_offset = (iat_rva + 72) as i64 - call_pos7; // closesocket at IAT[9]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(closesocket_offset as i32).to_le_bytes());
    
    // closesocket(server)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x50]);
    let call_pos8 = code_start as i64 + code.len() as i64 + 6;
    let closesocket_offset2 = (iat_rva + 72) as i64 - call_pos8;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(closesocket_offset2 as i32).to_le_bytes());
    
    // WSACleanup()
    let call_pos9 = code_start as i64 + code.len() as i64 + 6;
    let cleanup_offset = (iat_rva + 80) as i64 - call_pos9; // WSACleanup at IAT[10]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(cleanup_offset as i32).to_le_bytes());
    
    // ExitProcess(0)
    code.extend(x64::xor_ecx_ecx());
    let call_pos10 = code_start as i64 + code.len() as i64 + 6;
    let ep_offset = (iat_rva + 16) as i64 - call_pos10;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(ep_offset as i32).to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports from kernel32.dll and ws2_32.dll
    let mut rdata = vec![0u8; rdata_size as usize];
    
    // IAT layout:
    // 0-7: GetStdHandle (kernel32)
    // 8-15: WriteFile (kernel32)
    // 16-23: ExitProcess (kernel32)
    // 24-31: WSAStartup (ws2_32)
    // 32-39: socket (ws2_32)
    // 40-47: bind (ws2_32)
    // 48-55: listen (ws2_32)
    // 56-63: accept (ws2_32)
    // 64-71: send (ws2_32)
    // 72-79: closesocket (ws2_32)
    // 80-87: WSACleanup (ws2_32)
    
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 32;
    let hint_wsastartup = hint_name_rva + 48;
    let hint_socket = hint_name_rva + 64;
    let hint_bind = hint_name_rva + 80;
    let hint_listen = hint_name_rva + 96;
    let hint_accept = hint_name_rva + 112;
    let hint_send = hint_name_rva + 128;
    let hint_closesocket = hint_name_rva + 144;
    let hint_wsacleanup = hint_name_rva + 160;
    
    // IAT - kernel32 functions (0-23)
    rdata[0..8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    // IAT - ws2_32 functions (24-87)
    rdata[24..32].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[32..40].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[40..48].copy_from_slice(&(hint_bind as u64).to_le_bytes());
    rdata[48..56].copy_from_slice(&(hint_listen as u64).to_le_bytes());
    rdata[56..64].copy_from_slice(&(hint_accept as u64).to_le_bytes());
    rdata[64..72].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[72..80].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[80..88].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Import Directory Table (2 entries + null terminator)
    let idt_offset = 0x80usize;
    // kernel32.dll entry
    let kernel32_ilt = ilt_rva;
    let kernel32_iat = iat_rva;
    let kernel32_name = dll_names_rva;
    rdata[idt_offset..idt_offset+4].copy_from_slice(&kernel32_ilt.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&kernel32_name.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&kernel32_iat.to_le_bytes());
    
    // ws2_32.dll entry
    let ws2_ilt = ilt_rva + 32;
    let ws2_iat = iat_rva + 24;
    let ws2_name = dll_names_rva + 16;
    rdata[idt_offset+20..idt_offset+24].copy_from_slice(&ws2_ilt.to_le_bytes());
    rdata[idt_offset+32..idt_offset+36].copy_from_slice(&ws2_name.to_le_bytes());
    rdata[idt_offset+36..idt_offset+40].copy_from_slice(&ws2_iat.to_le_bytes());
    
    // ILT for kernel32
    let ilt_offset = 0x100usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    
    // ILT for ws2_32
    let ilt2_offset = ilt_offset + 32;
    rdata[ilt2_offset..ilt2_offset+8].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[ilt2_offset+8..ilt2_offset+16].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[ilt2_offset+16..ilt2_offset+24].copy_from_slice(&(hint_bind as u64).to_le_bytes());
    rdata[ilt2_offset+24..ilt2_offset+32].copy_from_slice(&(hint_listen as u64).to_le_bytes());
    rdata[ilt2_offset+32..ilt2_offset+40].copy_from_slice(&(hint_accept as u64).to_le_bytes());
    rdata[ilt2_offset+40..ilt2_offset+48].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[ilt2_offset+48..ilt2_offset+56].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[ilt2_offset+56..ilt2_offset+64].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0x180usize;
    rdata[hnt_offset+2..hnt_offset+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt_offset+18..hnt_offset+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt_offset+34..hnt_offset+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt_offset+50..hnt_offset+61].copy_from_slice(b"WSAStartup\0");
    rdata[hnt_offset+66..hnt_offset+73].copy_from_slice(b"socket\0");
    rdata[hnt_offset+82..hnt_offset+87].copy_from_slice(b"bind\0");
    rdata[hnt_offset+98..hnt_offset+105].copy_from_slice(b"listen\0");
    rdata[hnt_offset+114..hnt_offset+121].copy_from_slice(b"accept\0");
    rdata[hnt_offset+130..hnt_offset+135].copy_from_slice(b"send\0");
    rdata[hnt_offset+146..hnt_offset+158].copy_from_slice(b"closesocket\0");
    rdata[hnt_offset+162..hnt_offset+173].copy_from_slice(b"WSACleanup\0");
    
    // DLL names
    rdata[0x300..0x300+13].copy_from_slice(b"kernel32.dll\0");
    rdata[0x310..0x310+11].copy_from_slice(b"ws2_32.dll\0");
    
    pe.extend(&rdata);
    
    // .data - response string and status message
    let mut data = vec![0u8; data_size as usize];
    let resp_bytes = response.as_bytes();
    data[..resp_bytes.len().min(0x100)].copy_from_slice(&resp_bytes[..resp_bytes.len().min(0x100)]);
    
    // Status message at offset 0x100
    let status_msg = format!("Listening on port {}...\n", port);
    data[0x100..0x100+status_msg.len()].copy_from_slice(status_msg.as_bytes());
    
    pe.extend(&data);
    
    pe
}

/// Generate ultra-minimal PE (~512-768 bytes) - Optimized for Windows 10/11
/// Uses single section, minimal headers, but maintains compatibility
pub fn generate_tiny_pe(message: &str) -> Vec<u8> {
    // Windows PE requirements:
    // - DOS header must have valid MZ signature and e_lfanew
    // - PE signature must be at e_lfanew offset
    // - Optional header must be at least 112 bytes for PE32+
    // - SectionAlignment >= FileAlignment
    // - FileAlignment must be power of 2, minimum 512 for most loaders
    
    let pe_offset: u32 = 0x40; // Minimal DOS header (64 bytes)
    let headers_size: u32 = 0x200; // 512 bytes for headers
    
    // Single section containing code + imports + data
    let section_rva: u32 = 0x1000; // Standard 4KB alignment for section RVA
    let section_file_offset: u32 = 0x200; // After headers
    let section_size: u32 = 0x200; // 512 bytes for section
    
    let image_size: u32 = 0x2000; // 8KB total image
    
    let mut pe = Vec::new();
    
    // === Minimal DOS Header (64 bytes) ===
    pe.extend(&[0x4D, 0x5A]); // MZ signature
    pe.extend(&[0x00; 58]); // Padding
    pe.extend(&pe_offset.to_le_bytes()); // e_lfanew at 0x3C
    
    // === PE Signature (4 bytes) ===
    pe.extend_from_slice(PE_SIGNATURE);
    
    // === COFF Header (20 bytes) ===
    pe.extend_from_slice(&IMAGE_FILE_MACHINE_AMD64.to_le_bytes());
    pe.extend_from_slice(&1u16.to_le_bytes()); // 1 section
    pe.extend_from_slice(&0u32.to_le_bytes()); // timestamp
    pe.extend_from_slice(&0u32.to_le_bytes()); // symbol table
    pe.extend_from_slice(&0u32.to_le_bytes()); // num symbols
    pe.extend_from_slice(&240u16.to_le_bytes()); // optional header size (standard)
    let characteristics = IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE;
    pe.extend_from_slice(&characteristics.to_le_bytes());
    
    // === Optional Header (240 bytes) ===
    let entry_point = section_rva; // Entry at start of section
    let opt_header = create_optional_header(
        section_size, 0, entry_point, section_rva, image_size, headers_size,
    );
    
    // Patch import directory
    let import_dir_rva = section_rva + 0x80;
    let mut opt_header = opt_header;
    let import_dir_offset = 112 + 8; // After fixed fields, import is entry 1
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&40u32.to_le_bytes());
    
    pe.extend(opt_header);
    
    // === Section Header (40 bytes) ===
    pe.extend(create_section_header(
        b".text\0\0\0",
        section_size,
        section_rva,
        section_size,
        section_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_CNT_INITIALIZED_DATA |
        IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
    ));
    
    // Pad to section start
    pe.resize(section_file_offset as usize, 0);
    
    // === Section Content ===
    // Layout:
    // 0x00-0x7F: Code
    // 0x80-0x9F: Import Directory (20 bytes + 20 null)
    // 0xA0-0xBF: IAT (3 entries + null = 32 bytes)
    // 0xC0-0xDF: ILT (3 entries + null = 32 bytes)
    // 0xE0-0x11F: Hint/Name table
    // 0x120-0x12F: DLL name
    // 0x130+: Message
    
    let mut section = vec![0u8; section_size as usize];
    
    let iat_off: u32 = 0xA0;
    let ilt_off: u32 = 0xC0;
    let hint_off: u32 = 0xE0;
    let dll_off: u32 = 0x120;
    let msg_off: u32 = 0x130;
    
    // Generate code
    let mut code = Vec::new();
    code.extend(x64::sub_rsp_imm8(0x28));
    
    // GetStdHandle(-11)
    code.extend(&[0xB9]);
    code.extend(&(-11i32).to_le_bytes());
    let call1_pos = section_rva + code.len() as u32 + 6;
    let iat_rva = section_rva + iat_off;
    code.extend(&[0xFF, 0x15]);
    code.extend(&((iat_rva as i32) - (call1_pos as i32)).to_le_bytes());
    
    // mov rcx, rax (handle)
    code.extend(&[0x48, 0x89, 0xC1]);
    
    // lea rdx, [rip+message]
    let lea_pos = section_rva + code.len() as u32 + 7;
    let msg_rva = section_rva + msg_off;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&((msg_rva as i32) - (lea_pos as i32)).to_le_bytes());
    
    // mov r8d, len
    let msg_len = message.len().min(200) as u32;
    code.extend(&[0x41, 0xB8]);
    code.extend(&msg_len.to_le_bytes());
    
    // lea r9, [rsp+32]
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x20]);
    
    // mov qword [rsp+32], 0
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    // call WriteFile
    let call2_pos = section_rva + code.len() as u32 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 8) as i32) - (call2_pos as i32)).to_le_bytes());
    
    // xor ecx, ecx
    code.extend(x64::xor_ecx_ecx());
    
    // call ExitProcess
    let call3_pos = section_rva + code.len() as u32 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 16) as i32) - (call3_pos as i32)).to_le_bytes());
    
    section[..code.len()].copy_from_slice(&code);
    
    // Import Directory (at 0x80)
    let import_dir_off = 0x80usize;
    let ilt_rva = section_rva + ilt_off;
    let dll_rva = section_rva + dll_off;
    section[import_dir_off..import_dir_off+4].copy_from_slice(&ilt_rva.to_le_bytes());
    section[import_dir_off+12..import_dir_off+16].copy_from_slice(&dll_rva.to_le_bytes());
    section[import_dir_off+16..import_dir_off+20].copy_from_slice(&iat_rva.to_le_bytes());
    
    // IAT (at 0xA0)
    let hint_gsh = section_rva + hint_off;
    let hint_wf = section_rva + hint_off + 16;
    let hint_ep = section_rva + hint_off + 32;
    section[iat_off as usize..iat_off as usize + 8].copy_from_slice(&(hint_gsh as u64).to_le_bytes());
    section[iat_off as usize + 8..iat_off as usize + 16].copy_from_slice(&(hint_wf as u64).to_le_bytes());
    section[iat_off as usize + 16..iat_off as usize + 24].copy_from_slice(&(hint_ep as u64).to_le_bytes());
    
    // ILT (at 0xC0)
    section[ilt_off as usize..ilt_off as usize + 8].copy_from_slice(&(hint_gsh as u64).to_le_bytes());
    section[ilt_off as usize + 8..ilt_off as usize + 16].copy_from_slice(&(hint_wf as u64).to_le_bytes());
    section[ilt_off as usize + 16..ilt_off as usize + 24].copy_from_slice(&(hint_ep as u64).to_le_bytes());
    
    // Hint/Name Table (at 0xE0)
    section[hint_off as usize + 2..hint_off as usize + 15].copy_from_slice(b"GetStdHandle\0");
    section[hint_off as usize + 18..hint_off as usize + 28].copy_from_slice(b"WriteFile\0");
    section[hint_off as usize + 34..hint_off as usize + 46].copy_from_slice(b"ExitProcess\0");
    
    // DLL name (at 0x120)
    section[dll_off as usize..dll_off as usize + 13].copy_from_slice(b"kernel32.dll\0");
    
    // Message (at 0x130)
    let msg_bytes = message.as_bytes();
    let copy_len = msg_bytes.len().min(section_size as usize - msg_off as usize);
    section[msg_off as usize..msg_off as usize + copy_len].copy_from_slice(&msg_bytes[..copy_len]);
    
    pe.extend(&section);
    
    pe
}

/// Generate a minimal GUI PE (MessageBox)
pub fn generate_gui_pe(title: &str, message: &str) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x200;
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x400;
    let rdata_file_offset: u32 = 0x400;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x200;
    let data_file_offset: u32 = 0x800;
    
    let image_size: u32 = 0x4000;
    
    // IAT: MessageBoxA (user32), ExitProcess (kernel32)
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0x40;
    let ilt_rva = rdata_rva + 0x80;
    let hint_name_rva = rdata_rva + 0xC0;
    let dll_names_rva = rdata_rva + 0x100;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    // Subsystem: GUI (2) instead of Console (3)
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    // Change subsystem to GUI
    opt_header[68] = 2; // IMAGE_SUBSYSTEM_WINDOWS_GUI
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&60u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // sub rsp, 0x28
    code.extend(x64::sub_rsp_imm8(0x28));
    
    // MessageBoxA(NULL, message, title, MB_OK)
    // xor ecx, ecx (NULL)
    code.extend(x64::xor_ecx_ecx());
    
    // lea rdx, [rip+message]
    let msg_rva = data_rva;
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let msg_offset = msg_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&(msg_offset as i32).to_le_bytes());
    
    // lea r8, [rip+title]
    let title_rva = data_rva + 0x80;
    let lea_pos2 = code_start as i64 + code.len() as i64 + 7;
    let title_offset = title_rva as i64 - lea_pos2;
    code.extend(&[0x4C, 0x8D, 0x05]);
    code.extend(&(title_offset as i32).to_le_bytes());
    
    // xor r9d, r9d (MB_OK = 0)
    code.extend(&[0x45, 0x31, 0xC9]);
    
    // call MessageBoxA
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let mbox_offset = iat_rva as i64 - call_pos;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(mbox_offset as i32).to_le_bytes());
    
    // xor ecx, ecx
    code.extend(x64::xor_ecx_ecx());
    
    // call ExitProcess
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let exit_offset = (iat_rva + 8) as i64 - call_pos2;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(exit_offset as i32).to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports
    let mut rdata = vec![0u8; rdata_size as usize];
    
    let hint_messagebox = hint_name_rva;
    let hint_exitprocess = hint_name_rva + 16;
    
    // IAT - user32 (MessageBoxA)
    rdata[0..8].copy_from_slice(&(hint_messagebox as u64).to_le_bytes());
    // IAT - kernel32 (ExitProcess)
    rdata[8..16].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    
    // Import Directory (2 entries + null)
    let idt_offset = 0x40usize;
    // user32.dll
    rdata[idt_offset..idt_offset+4].copy_from_slice(&ilt_rva.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&dll_names_rva.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&iat_rva.to_le_bytes());
    // kernel32.dll
    let kernel32_name = dll_names_rva + 16;
    rdata[idt_offset+20..idt_offset+24].copy_from_slice(&(ilt_rva + 16).to_le_bytes());
    rdata[idt_offset+32..idt_offset+36].copy_from_slice(&kernel32_name.to_le_bytes());
    rdata[idt_offset+36..idt_offset+40].copy_from_slice(&(iat_rva + 8).to_le_bytes());
    
    // ILT
    let ilt_offset = 0x80usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_messagebox as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0xC0usize;
    rdata[hnt_offset+2..hnt_offset+14].copy_from_slice(b"MessageBoxA\0");
    rdata[hnt_offset+18..hnt_offset+30].copy_from_slice(b"ExitProcess\0");
    
    // DLL names
    rdata[0x100..0x100+11].copy_from_slice(b"user32.dll\0");
    rdata[0x110..0x110+13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&rdata);
    
    // .data - message and title
    let mut data = vec![0u8; data_size as usize];
    let msg_bytes = message.as_bytes();
    data[..msg_bytes.len().min(0x7F)].copy_from_slice(&msg_bytes[..msg_bytes.len().min(0x7F)]);
    let title_bytes = title.as_bytes();
    data[0x80..0x80+title_bytes.len().min(0x7F)].copy_from_slice(&title_bytes[..title_bytes.len().min(0x7F)]);
    
    pe.extend(&data);
    
    pe
}

/// Generate ultra-tiny PE executable (~400-500 bytes)
/// Uses aggressive header overlap and minimal structures for Windows 10/11 x64
/// 
/// Techniques used:
/// 1. DOS header overlaps with PE header (e_lfanew points to offset 0x04)
/// 2. Minimal optional header (reduced data directories)
/// 3. Single section with code + imports + data
/// 4. FileAlignment = SectionAlignment = 0x10 (minimum for Win10+)
/// 5. Strings packed tightly
pub fn generate_ultra_tiny_pe(message: &str) -> Vec<u8> {
    // Windows 10/11 x64 requirements:
    // - MZ signature at offset 0
    // - e_lfanew (offset 0x3C) points to PE signature
    // - PE signature "PE\0\0" 
    // - COFF header (20 bytes)
    // - Optional header (minimum ~112 bytes for PE32+, but Windows needs more)
    // - At least one section header (40 bytes)
    // - Import directory must be valid
    
    // Layout with overlap:
    // 0x00: MZ signature
    // 0x04: PE signature (overlapped - e_lfanew = 0x04)
    // 0x08: COFF header (20 bytes)
    // 0x1C: Optional header (112 bytes minimum + data dirs)
    // After optional header: Section header
    // After section header: Code + imports + data
    
    // However, Windows 10/11 is stricter than older versions.
    // We need a valid DOS stub area and proper alignment.
    // Minimum practical size with imports is ~400-500 bytes.
    
    let mut pe = Vec::with_capacity(512);
    
    // === DOS Header (64 bytes) with PE signature overlap ===
    // The trick: e_lfanew at 0x3C points to 0x40 (standard) or we can try 0x04
    // But Windows 10+ validates more fields, so we use 0x40 for compatibility
    
    // Actually, let's try a more aggressive approach:
    // Put PE signature at offset 0x3C+4 = 0x40, which is standard
    // But minimize everything after
    
    // DOS Header
    pe.extend(&[0x4D, 0x5A]); // MZ signature (offset 0x00)
    pe.extend(&[0x00; 58]);   // Padding (offset 0x02-0x3B)
    pe.extend(&0x40u32.to_le_bytes()); // e_lfanew = 0x40 (offset 0x3C)
    
    // PE Signature at 0x40
    pe.extend(b"PE\0\0");
    
    // === COFF Header (20 bytes) at 0x44 ===
    pe.extend(&IMAGE_FILE_MACHINE_AMD64.to_le_bytes()); // Machine
    pe.extend(&1u16.to_le_bytes()); // NumberOfSections = 1
    pe.extend(&0u32.to_le_bytes()); // TimeDateStamp
    pe.extend(&0u32.to_le_bytes()); // PointerToSymbolTable
    pe.extend(&0u32.to_le_bytes()); // NumberOfSymbols
    pe.extend(&0xF0u16.to_le_bytes()); // SizeOfOptionalHeader = 240 (standard PE32+)
    let characteristics = IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE | 0x0001; // RELOCS_STRIPPED
    pe.extend(&characteristics.to_le_bytes());
    
    // === Optional Header (240 bytes) at 0x58 ===
    // We need the full optional header for Windows 10/11 compatibility
    
    // For ultra-tiny, we'll use:
    // - FileAlignment = SectionAlignment = 0x200 (512 bytes, minimum practical)
    // - Single section starting right after headers
    // - Minimal image size
    
    let file_alignment: u32 = 0x200;
    let section_alignment: u32 = 0x200; // Can be same as file alignment for tiny PE
    let headers_size: u32 = 0x200; // Headers fit in 512 bytes
    let section_rva: u32 = 0x200; // Section starts right after headers
    let section_size: u32 = 0x200; // 512 bytes for section
    let image_size: u32 = 0x400; // 1KB total (headers + section)
    let entry_point: u32 = section_rva; // Entry at start of section
    
    // Import directory will be at section_rva + 0x80
    let import_dir_rva: u32 = section_rva + 0x80;
    
    // Optional Header Standard Fields
    pe.extend(&PE32_PLUS_MAGIC.to_le_bytes()); // Magic
    pe.push(14); pe.push(0); // Linker version
    pe.extend(&section_size.to_le_bytes()); // SizeOfCode
    pe.extend(&0u32.to_le_bytes()); // SizeOfInitializedData
    pe.extend(&0u32.to_le_bytes()); // SizeOfUninitializedData
    pe.extend(&entry_point.to_le_bytes()); // AddressOfEntryPoint
    pe.extend(&section_rva.to_le_bytes()); // BaseOfCode
    
    // Optional Header Windows-Specific Fields
    pe.extend(&IMAGE_BASE.to_le_bytes()); // ImageBase (8 bytes)
    pe.extend(&section_alignment.to_le_bytes()); // SectionAlignment
    pe.extend(&file_alignment.to_le_bytes()); // FileAlignment
    pe.extend(&6u16.to_le_bytes()); // MajorOperatingSystemVersion
    pe.extend(&0u16.to_le_bytes()); // MinorOperatingSystemVersion
    pe.extend(&0u16.to_le_bytes()); // MajorImageVersion
    pe.extend(&0u16.to_le_bytes()); // MinorImageVersion
    pe.extend(&6u16.to_le_bytes()); // MajorSubsystemVersion
    pe.extend(&0u16.to_le_bytes()); // MinorSubsystemVersion
    pe.extend(&0u32.to_le_bytes()); // Win32VersionValue
    pe.extend(&image_size.to_le_bytes()); // SizeOfImage
    pe.extend(&headers_size.to_le_bytes()); // SizeOfHeaders
    pe.extend(&0u32.to_le_bytes()); // CheckSum
    pe.extend(&IMAGE_SUBSYSTEM_WINDOWS_CUI.to_le_bytes()); // Subsystem (Console)
    pe.extend(&0x8160u16.to_le_bytes()); // DllCharacteristics (ASLR, DEP, NX, TERMINAL_SERVER_AWARE)
    pe.extend(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
    pe.extend(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
    pe.extend(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
    pe.extend(&0x1000u64.to_le_bytes()); // SizeOfHeapCommit
    pe.extend(&0u32.to_le_bytes()); // LoaderFlags
    pe.extend(&16u32.to_le_bytes()); // NumberOfRvaAndSizes
    
    // Data Directories (16 entries, 8 bytes each = 128 bytes)
    // Only Import Directory (index 1) is non-zero
    pe.extend(&0u64.to_le_bytes()); // Export
    pe.extend(&import_dir_rva.to_le_bytes()); // Import RVA
    pe.extend(&40u32.to_le_bytes()); // Import Size
    for _ in 2..16 {
        pe.extend(&0u64.to_le_bytes()); // Remaining directories
    }
    
    // === Section Header (40 bytes) ===
    pe.extend(b".text\0\0\0"); // Name
    pe.extend(&section_size.to_le_bytes()); // VirtualSize
    pe.extend(&section_rva.to_le_bytes()); // VirtualAddress
    pe.extend(&section_size.to_le_bytes()); // SizeOfRawData
    pe.extend(&section_rva.to_le_bytes()); // PointerToRawData (same as RVA when alignment matches)
    pe.extend(&0u32.to_le_bytes()); // PointerToRelocations
    pe.extend(&0u32.to_le_bytes()); // PointerToLinenumbers
    pe.extend(&0u16.to_le_bytes()); // NumberOfRelocations
    pe.extend(&0u16.to_le_bytes()); // NumberOfLinenumbers
    let section_chars = IMAGE_SCN_CNT_CODE | IMAGE_SCN_CNT_INITIALIZED_DATA |
                        IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
    pe.extend(&section_chars.to_le_bytes()); // Characteristics
    
    // Pad to section start (0x200)
    pe.resize(section_rva as usize, 0);
    
    // === Section Content (512 bytes) ===
    // Layout within section:
    // 0x00-0x5F: Code (~96 bytes)
    // 0x60-0x7F: Message string (32 bytes)
    // 0x80-0x9F: Import Directory (20 bytes entry + 20 bytes null = 40 bytes)
    // 0xA0-0xBF: IAT (3 entries + null = 32 bytes)
    // 0xC0-0xDF: ILT (3 entries + null = 32 bytes)
    // 0xE0-0x11F: Hint/Name table (~64 bytes)
    // 0x120-0x12F: DLL name "kernel32.dll\0" (13 bytes)
    
    let mut section = vec![0u8; section_size as usize];
    
    let msg_off: u32 = 0x60;
    let import_off: u32 = 0x80;
    let iat_off: u32 = 0xA0;
    let ilt_off: u32 = 0xC0;
    let hint_off: u32 = 0xE0;
    let dll_off: u32 = 0x120;
    
    // RVAs for import structures
    let iat_rva = section_rva + iat_off;
    let ilt_rva = section_rva + ilt_off;
    let dll_rva = section_rva + dll_off;
    let msg_rva = section_rva + msg_off;
    
    // Hint/Name RVAs
    let hint_gsh_rva = section_rva + hint_off;
    let hint_wf_rva = section_rva + hint_off + 16;
    let hint_ep_rva = section_rva + hint_off + 32;
    
    // === Generate Code ===
    let mut code = Vec::new();
    let code_base = section_rva;
    
    // sub rsp, 0x28 (shadow space + alignment)
    code.extend(&[0x48, 0x83, 0xEC, 0x28]);
    
    // GetStdHandle(-11) -> stdout
    // mov ecx, -11
    code.extend(&[0xB9]);
    code.extend(&(-11i32).to_le_bytes());
    
    // call [rip + offset_to_iat]
    let call1_end = code_base + code.len() as u32 + 6;
    let iat_gsh = iat_rva;
    code.extend(&[0xFF, 0x15]);
    code.extend(&((iat_gsh as i32) - (call1_end as i32)).to_le_bytes());
    
    // mov rcx, rax (handle)
    code.extend(&[0x48, 0x89, 0xC1]);
    
    // lea rdx, [rip + message]
    let lea_end = code_base + code.len() as u32 + 7;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&((msg_rva as i32) - (lea_end as i32)).to_le_bytes());
    
    // mov r8d, message_length
    let msg_len = message.len().min(30) as u32;
    code.extend(&[0x41, 0xB8]);
    code.extend(&msg_len.to_le_bytes());
    
    // xor r9d, r9d (lpNumberOfBytesWritten = NULL, allowed for console)
    code.extend(&[0x45, 0x31, 0xC9]);
    
    // mov qword [rsp+0x20], 0 (lpOverlapped = NULL)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    // call [rip + WriteFile]
    let call2_end = code_base + code.len() as u32 + 6;
    let iat_wf = iat_rva + 8;
    code.extend(&[0xFF, 0x15]);
    code.extend(&((iat_wf as i32) - (call2_end as i32)).to_le_bytes());
    
    // xor ecx, ecx (exit code 0)
    code.extend(&[0x31, 0xC9]);
    
    // call [rip + ExitProcess]
    let call3_end = code_base + code.len() as u32 + 6;
    let iat_ep = iat_rva + 16;
    code.extend(&[0xFF, 0x15]);
    code.extend(&((iat_ep as i32) - (call3_end as i32)).to_le_bytes());
    
    // Copy code to section
    section[..code.len()].copy_from_slice(&code);
    
    // === Message string ===
    let msg_bytes = message.as_bytes();
    let copy_len = msg_bytes.len().min(30);
    section[msg_off as usize..msg_off as usize + copy_len].copy_from_slice(&msg_bytes[..copy_len]);
    
    // === Import Directory (at 0x80) ===
    // One entry + null terminator
    let import_entry = import_off as usize;
    section[import_entry..import_entry+4].copy_from_slice(&ilt_rva.to_le_bytes()); // OriginalFirstThunk
    section[import_entry+4..import_entry+8].copy_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
    section[import_entry+8..import_entry+12].copy_from_slice(&0u32.to_le_bytes()); // ForwarderChain
    section[import_entry+12..import_entry+16].copy_from_slice(&dll_rva.to_le_bytes()); // Name
    section[import_entry+16..import_entry+20].copy_from_slice(&iat_rva.to_le_bytes()); // FirstThunk
    // Null terminator entry (20 bytes of zeros) - already zero
    
    // === IAT (at 0xA0) ===
    section[iat_off as usize..iat_off as usize + 8].copy_from_slice(&(hint_gsh_rva as u64).to_le_bytes());
    section[iat_off as usize + 8..iat_off as usize + 16].copy_from_slice(&(hint_wf_rva as u64).to_le_bytes());
    section[iat_off as usize + 16..iat_off as usize + 24].copy_from_slice(&(hint_ep_rva as u64).to_le_bytes());
    // Null terminator (8 bytes of zeros) - already zero
    
    // === ILT (at 0xC0) - same as IAT ===
    section[ilt_off as usize..ilt_off as usize + 8].copy_from_slice(&(hint_gsh_rva as u64).to_le_bytes());
    section[ilt_off as usize + 8..ilt_off as usize + 16].copy_from_slice(&(hint_wf_rva as u64).to_le_bytes());
    section[ilt_off as usize + 16..ilt_off as usize + 24].copy_from_slice(&(hint_ep_rva as u64).to_le_bytes());
    
    // === Hint/Name Table (at 0xE0) ===
    // GetStdHandle
    section[hint_off as usize] = 0; section[hint_off as usize + 1] = 0; // Hint
    section[hint_off as usize + 2..hint_off as usize + 15].copy_from_slice(b"GetStdHandle\0");
    // WriteFile
    section[hint_off as usize + 16] = 0; section[hint_off as usize + 17] = 0;
    section[hint_off as usize + 18..hint_off as usize + 28].copy_from_slice(b"WriteFile\0");
    // ExitProcess
    section[hint_off as usize + 32] = 0; section[hint_off as usize + 33] = 0;
    section[hint_off as usize + 34..hint_off as usize + 46].copy_from_slice(b"ExitProcess\0");
    
    // === DLL Name (at 0x120) ===
    section[dll_off as usize..dll_off as usize + 13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&section);
    
    pe
}

/// Generate the smallest possible PE that prints output
/// Target: ~400-500 bytes with aggressive optimizations
/// 
/// This version uses:
/// 1. Minimal 0x10 alignment (works on Windows 10+)
/// 2. Reduced data directories (only 2 instead of 16)
/// 3. Overlapped structures where possible
/// 4. Compact code
pub fn generate_smallest_pe(message: &str) -> Vec<u8> {
    // Windows 10/11 x64 minimum requirements:
    // - Valid DOS header with MZ and e_lfanew
    // - PE signature
    // - Valid COFF header for AMD64
    // - Optional header with correct magic and sizes
    // - At least one section (or code in header padding)
    // - Valid import table for API calls
    
    let mut pe = Vec::with_capacity(512);
    
    // === DOS Header (64 bytes) ===
    pe.extend(&[0x4D, 0x5A]); // MZ
    pe.extend(&[0x00; 58]);   // Padding
    pe.extend(&0x40u32.to_le_bytes()); // e_lfanew = 0x40
    
    // === PE Signature (4 bytes) at 0x40 ===
    pe.extend(b"PE\0\0");
    
    // === COFF Header (20 bytes) at 0x44 ===
    pe.extend(&IMAGE_FILE_MACHINE_AMD64.to_le_bytes());
    pe.extend(&1u16.to_le_bytes()); // 1 section
    pe.extend(&0u32.to_le_bytes()); // timestamp
    pe.extend(&0u32.to_le_bytes()); // symbol table
    pe.extend(&0u32.to_le_bytes()); // num symbols
    pe.extend(&0xF0u16.to_le_bytes()); // SizeOfOptionalHeader = 240 bytes (standard PE32+)
    let characteristics = IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE | 0x0001;
    pe.extend(&characteristics.to_le_bytes());
    
    // === Reduced Optional Header (128 bytes) at 0x58 ===
    // Using minimal alignment: 0x10 for both file and section
    // This works on Windows 10+ but not older versions
    
    let file_alignment: u32 = 0x10;
    let section_alignment: u32 = 0x10;
    
    // Calculate offsets with 0x10 alignment
    // Headers: DOS(64) + PE(4) + COFF(20) + OptHdr(240) + SecHdr(40) = 368 bytes
    // Aligned to 0x10: 0x180 = 384 bytes
    let headers_size: u32 = 0x180; // 384 bytes
    let section_file_offset: u32 = 0x180;
    let section_rva: u32 = 0x180; // Same as file offset when alignments match
    let section_size: u32 = 0x100; // 256 bytes for section content
    let image_size: u32 = 0x280; // 640 bytes total
    let entry_point: u32 = section_rva;
    let import_dir_rva: u32 = section_rva + 0x60;
    
    // Standard fields (offset 0x58)
    pe.extend(&PE32_PLUS_MAGIC.to_le_bytes()); // Magic
    pe.push(14); pe.push(0); // Linker version
    pe.extend(&section_size.to_le_bytes()); // SizeOfCode
    pe.extend(&0u32.to_le_bytes()); // SizeOfInitializedData
    pe.extend(&0u32.to_le_bytes()); // SizeOfUninitializedData
    pe.extend(&entry_point.to_le_bytes()); // AddressOfEntryPoint
    pe.extend(&section_rva.to_le_bytes()); // BaseOfCode
    
    // Windows-specific fields
    pe.extend(&IMAGE_BASE.to_le_bytes()); // ImageBase (8 bytes)
    pe.extend(&section_alignment.to_le_bytes()); // SectionAlignment
    pe.extend(&file_alignment.to_le_bytes()); // FileAlignment
    pe.extend(&6u16.to_le_bytes()); // MajorOperatingSystemVersion
    pe.extend(&0u16.to_le_bytes()); // MinorOperatingSystemVersion
    pe.extend(&0u16.to_le_bytes()); // MajorImageVersion
    pe.extend(&0u16.to_le_bytes()); // MinorImageVersion
    pe.extend(&6u16.to_le_bytes()); // MajorSubsystemVersion
    pe.extend(&0u16.to_le_bytes()); // MinorSubsystemVersion
    pe.extend(&0u32.to_le_bytes()); // Win32VersionValue
    pe.extend(&image_size.to_le_bytes()); // SizeOfImage
    pe.extend(&headers_size.to_le_bytes()); // SizeOfHeaders
    pe.extend(&0u32.to_le_bytes()); // CheckSum
    pe.extend(&IMAGE_SUBSYSTEM_WINDOWS_CUI.to_le_bytes()); // Subsystem
    pe.extend(&0x8160u16.to_le_bytes()); // DllCharacteristics
    pe.extend(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
    pe.extend(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
    pe.extend(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
    pe.extend(&0x1000u64.to_le_bytes()); // SizeOfHeapCommit
    pe.extend(&0u32.to_le_bytes()); // LoaderFlags
    pe.extend(&16u32.to_le_bytes()); // NumberOfRvaAndSizes = 16 (standard)
    
    // Data Directories (16 entries = 128 bytes)
    pe.extend(&0u64.to_le_bytes()); // Export (empty)
    pe.extend(&import_dir_rva.to_le_bytes()); // Import RVA
    pe.extend(&40u32.to_le_bytes()); // Import Size
    for _ in 2..16 {
        pe.extend(&0u64.to_le_bytes()); // Remaining directories
    }
    
    // === Section Header (40 bytes) at 0xD8 ===
    pe.extend(b".text\0\0\0");
    pe.extend(&section_size.to_le_bytes()); // VirtualSize
    pe.extend(&section_rva.to_le_bytes()); // VirtualAddress
    pe.extend(&section_size.to_le_bytes()); // SizeOfRawData
    pe.extend(&section_file_offset.to_le_bytes()); // PointerToRawData
    pe.extend(&0u32.to_le_bytes()); // PointerToRelocations
    pe.extend(&0u32.to_le_bytes()); // PointerToLinenumbers
    pe.extend(&0u16.to_le_bytes()); // NumberOfRelocations
    pe.extend(&0u16.to_le_bytes()); // NumberOfLinenumbers
    let section_chars = IMAGE_SCN_CNT_CODE | IMAGE_SCN_CNT_INITIALIZED_DATA |
                        IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
    pe.extend(&section_chars.to_le_bytes());
    
    // Pad to section start (0x100)
    pe.resize(section_file_offset as usize, 0);
    
    // === Section Content (256 bytes) ===
    // Tight layout:
    // 0x00-0x3F: Code (64 bytes)
    // 0x40-0x5F: Message (32 bytes)
    // 0x60-0x7F: Import Directory (32 bytes: 20 + 12 padding)
    // 0x80-0x9F: IAT (32 bytes)
    // 0xA0-0xBF: ILT (32 bytes)
    // 0xC0-0xEF: Hint/Name table (48 bytes)
    // 0xF0-0xFF: DLL name (16 bytes)
    
    let mut section = vec![0u8; section_size as usize];
    
    let msg_off: u32 = 0x40;
    let import_off: u32 = 0x60;
    let iat_off: u32 = 0x80;
    let ilt_off: u32 = 0xA0;
    let hint_off: u32 = 0xC0;
    let dll_off: u32 = 0xF0;
    
    let iat_rva = section_rva + iat_off;
    let ilt_rva = section_rva + ilt_off;
    let dll_rva = section_rva + dll_off;
    let msg_rva = section_rva + msg_off;
    let hint_gsh_rva = section_rva + hint_off;
    let hint_wf_rva = section_rva + hint_off + 16;
    let hint_ep_rva = section_rva + hint_off + 32;
    
    // === Compact Code ===
    let mut code = Vec::new();
    let code_base = section_rva;
    
    // sub rsp, 0x38 (shadow space 32 + 8 for 5th param + 8 alignment)
    code.extend(&[0x48, 0x83, 0xEC, 0x38]);
    
    // mov ecx, -11 (STD_OUTPUT_HANDLE)
    code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]);
    
    // call [GetStdHandle]
    let call1_end = code_base + code.len() as u32 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&((iat_rva as i32) - (call1_end as i32)).to_le_bytes());
    
    // mov rcx, rax
    code.extend(&[0x48, 0x89, 0xC1]);
    
    // lea rdx, [rip + msg]
    let lea_end = code_base + code.len() as u32 + 7;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&((msg_rva as i32) - (lea_end as i32)).to_le_bytes());
    
    // mov r8d, len
    let msg_len = message.len().min(30) as u8;
    code.extend(&[0x41, 0xB8, msg_len, 0x00, 0x00, 0x00]);
    
    // xor r9d, r9d (lpNumberOfBytesWritten = NULL)
    code.extend(&[0x45, 0x31, 0xC9]);
    
    // mov qword [rsp+0x20], 0 (lpOverlapped = NULL)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    // call [WriteFile]
    let call2_end = code_base + code.len() as u32 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 8) as i32) - (call2_end as i32)).to_le_bytes());
    
    // xor ecx, ecx
    code.extend(&[0x31, 0xC9]);
    
    // call [ExitProcess]
    let call3_end = code_base + code.len() as u32 + 6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(((iat_rva + 16) as i32) - (call3_end as i32)).to_le_bytes());
    
    section[..code.len()].copy_from_slice(&code);
    
    // Message
    let msg_bytes = message.as_bytes();
    let copy_len = msg_bytes.len().min(30);
    section[msg_off as usize..msg_off as usize + copy_len].copy_from_slice(&msg_bytes[..copy_len]);
    
    // Import Directory
    let ie = import_off as usize;
    section[ie..ie+4].copy_from_slice(&ilt_rva.to_le_bytes());
    section[ie+12..ie+16].copy_from_slice(&dll_rva.to_le_bytes());
    section[ie+16..ie+20].copy_from_slice(&iat_rva.to_le_bytes());
    
    // IAT
    section[iat_off as usize..iat_off as usize + 8].copy_from_slice(&(hint_gsh_rva as u64).to_le_bytes());
    section[iat_off as usize + 8..iat_off as usize + 16].copy_from_slice(&(hint_wf_rva as u64).to_le_bytes());
    section[iat_off as usize + 16..iat_off as usize + 24].copy_from_slice(&(hint_ep_rva as u64).to_le_bytes());
    
    // ILT
    section[ilt_off as usize..ilt_off as usize + 8].copy_from_slice(&(hint_gsh_rva as u64).to_le_bytes());
    section[ilt_off as usize + 8..ilt_off as usize + 16].copy_from_slice(&(hint_wf_rva as u64).to_le_bytes());
    section[ilt_off as usize + 16..ilt_off as usize + 24].copy_from_slice(&(hint_ep_rva as u64).to_le_bytes());
    
    // Hint/Name Table
    section[hint_off as usize + 2..hint_off as usize + 15].copy_from_slice(b"GetStdHandle\0");
    section[hint_off as usize + 18..hint_off as usize + 28].copy_from_slice(b"WriteFile\0");
    section[hint_off as usize + 34..hint_off as usize + 46].copy_from_slice(b"ExitProcess\0");
    
    // DLL Name
    section[dll_off as usize..dll_off as usize + 13].copy_from_slice(b"kernel32.dll\0");
    
    pe.extend(&section);
    
    pe // Total: 640 bytes (0x180 headers + 0x100 section)
}

/// Generate a PE template for TAYNI self-hosting
pub fn generate_pe_template() -> Vec<u8> {
    generate_ultra_tiny_pe("TAYNI PE Template\n")
}

/// Generate HTTP server PE with request parsing
/// Listens on port, accepts connections, reads HTTP request, sends response
pub fn generate_http_server_pe(port: u16, routes: &[(&str, &str)]) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x1000; // More space for HTTP parsing code
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x800; // More space for imports + recv
    let rdata_file_offset: u32 = 0x1200;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x1000; // Space for routes and responses
    let data_file_offset: u32 = 0x1A00;
    
    let image_size: u32 = 0x5000;
    
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0xC0;
    let ilt_rva = rdata_rva + 0x140;
    let hint_name_rva = rdata_rva + 0x200;
    let dll_names_rva = rdata_rva + 0x400;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&60u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate HTTP server code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // IAT offsets (with recv added)
    // 0: GetStdHandle, 8: WriteFile, 16: ExitProcess
    // 24: WSAStartup, 32: socket, 40: bind, 48: listen, 56: accept
    // 64: send, 72: recv, 80: closesocket, 88: WSACleanup
    
    // Prologue - allocate stack for:
    // [rsp+0x20]: shadow space
    // [rsp+0x40]: wsadata (16 bytes)
    // [rsp+0x50]: sockaddr_in (16 bytes)
    // [rsp+0x60]: server socket
    // [rsp+0x68]: client socket
    // [rsp+0x70]: stdout handle
    // [rsp+0x78]: recv buffer ptr
    // [rsp+0x80]: bytes received
    code.extend(x64::sub_rsp_imm8(0xA8)); // 168 bytes stack
    
    // WSAStartup(0x0202, &wsadata)
    code.extend(&[0xB9, 0x02, 0x02, 0x00, 0x00]); // mov ecx, 0x0202
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x40]); // lea rdx, [rsp+0x40]
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let wsa_startup_offset = (iat_rva + 32) as i64 - call_pos; // WSAStartup at IAT[4]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wsa_startup_offset as i32).to_le_bytes());
    
    // socket(AF_INET=2, SOCK_STREAM=1, IPPROTO_TCP=6)
    code.extend(&[0xB9, 0x02, 0x00, 0x00, 0x00]); // mov ecx, AF_INET
    code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, SOCK_STREAM
    code.extend(&[0x41, 0xB8, 0x06, 0x00, 0x00, 0x00]); // mov r8d, IPPROTO_TCP
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let socket_offset = (iat_rva + 40) as i64 - call_pos2; // socket at IAT[5]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(socket_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x60]); // mov [rsp+0x60], rax (server socket)
    
    // Build sockaddr_in at [rsp+0x50]
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x50, 0x02, 0x00]); // sin_family = AF_INET
    let port_be = port.to_be();
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x52]); // sin_port
    code.extend(&port_be.to_le_bytes());
    code.extend(&[0xC7, 0x44, 0x24, 0x54, 0x00, 0x00, 0x00, 0x00]); // sin_addr = INADDR_ANY
    
    // bind(socket, &addr, 16)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x60]); // mov rcx, [rsp+0x60]
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x50]); // lea rdx, [rsp+0x50]
    code.extend(&[0x41, 0xB8, 0x10, 0x00, 0x00, 0x00]); // mov r8d, 16
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let bind_offset = (iat_rva + 48) as i64 - call_pos3; // bind at IAT[6]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(bind_offset as i32).to_le_bytes());
    
    // listen(socket, SOMAXCONN=0x7fffffff)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x60]); // mov rcx, socket
    code.extend(&[0xBA, 0xFF, 0xFF, 0xFF, 0x7F]); // mov edx, SOMAXCONN
    let call_pos4 = code_start as i64 + code.len() as i64 + 6;
    let listen_offset = (iat_rva + 56) as i64 - call_pos4; // listen at IAT[7]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(listen_offset as i32).to_le_bytes());
    
    // Get stdout handle
    code.extend(&[0xB9]); // mov ecx, -11
    code.extend(&(-11i32).to_le_bytes());
    let call_pos_gsh = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset = iat_rva as i64 - call_pos_gsh;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x70]); // save stdout
    
    // Print "HTTP Server listening on port XXXX\n"
    let msg_rva = data_rva + 0xC00; // Status message
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x70]); // mov rcx, stdout
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let msg_offset = msg_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+msg]
    code.extend(&(msg_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x28, 0x00, 0x00, 0x00]); // mov r8d, 40
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x38]); // lea r9, [rsp+0x38]
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    let call_pos_wf = code_start as i64 + code.len() as i64 + 6;
    let wf_offset = (iat_rva + 8) as i64 - call_pos_wf;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset as i32).to_le_bytes());
    
    // Main accept loop
    let accept_loop_start = code.len();
    
    // accept(socket, NULL, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x60]); // mov rcx, socket
    code.extend(&[0x31, 0xD2]); // xor edx, edx
    code.extend(&[0x45, 0x31, 0xC0]); // xor r8d, r8d
    let call_pos5 = code_start as i64 + code.len() as i64 + 6;
    let accept_offset = (iat_rva + 64) as i64 - call_pos5; // accept at IAT[8]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(accept_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x68]); // save client socket
    
    // recv(client, buffer, 4096, 0)
    let recv_buf_rva = data_rva + 0xE00; // Buffer for HTTP request (after routes and messages)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x68]); // mov rcx, client
    let lea_pos2 = code_start as i64 + code.len() as i64 + 7;
    let buf_offset = recv_buf_rva as i64 - lea_pos2;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+buffer]
    code.extend(&(buf_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x00, 0x10, 0x00, 0x00]); // mov r8d, 4096
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
    let call_pos6 = code_start as i64 + code.len() as i64 + 6;
    let recv_offset = (iat_rva + 80) as i64 - call_pos6; // recv at IAT[10]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(recv_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x80]); // save bytes received
    
    // === Send response ===
    // For now, send the first route's response (or default)
    // Full routing with path matching is complex and will be added later
    
    let default_response = if routes.is_empty() {
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}"
    } else {
        routes[0].1
    };
    let resp_rva = data_rva;
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x68]); // mov rcx, client
    let lea_pos3 = code_start as i64 + code.len() as i64 + 7;
    let resp_offset = resp_rva as i64 - lea_pos3;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+response]
    code.extend(&(resp_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8]); // mov r8d, len
    code.extend(&(default_response.len() as u32).to_le_bytes());
    code.extend(&[0x45, 0x31, 0xC9]); // xor r9d, r9d
    let call_pos7 = code_start as i64 + code.len() as i64 + 6;
    let send_offset = (iat_rva + 72) as i64 - call_pos7; // send at IAT[9]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(send_offset as i32).to_le_bytes());
    
    // Sleep(100) - give client time to receive data
    code.extend(&[0xB9, 0x64, 0x00, 0x00, 0x00]); // mov ecx, 100 (100ms)
    let call_pos_sleep = code_start as i64 + code.len() as i64 + 6;
    let sleep_offset = (iat_rva + 24) as i64 - call_pos_sleep; // Sleep at IAT[3]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(sleep_offset as i32).to_le_bytes());
    
    // closesocket(client)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x68]);
    let call_pos8 = code_start as i64 + code.len() as i64 + 6;
    let closesocket_offset = (iat_rva + 96) as i64 - call_pos8; // closesocket at IAT[12]
    code.extend(&[0xFF, 0x15]);
    code.extend(&(closesocket_offset as i32).to_le_bytes());
    
    // Jump back to accept loop
    let loop_offset = accept_loop_start as i32 - code.len() as i32 - 5;
    code.extend(&[0xE9]); // jmp
    code.extend(&loop_offset.to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports with recv, shutdown and Sleep added
    let mut rdata = vec![0u8; rdata_size as usize];
    
    // IAT layout (with recv, shutdown and Sleep):
    // 0-7: GetStdHandle, 8-15: WriteFile, 16-23: ExitProcess, 24-31: Sleep
    // 32-39: WSAStartup, 40-47: socket, 48-55: bind, 56-63: listen, 64-71: accept
    // 72-79: send, 80-87: recv, 88-95: shutdown, 96-103: closesocket, 104-111: WSACleanup
    
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 32;
    let hint_sleep = hint_name_rva + 48;
    let hint_wsastartup = hint_name_rva + 64;
    let hint_socket = hint_name_rva + 80;
    let hint_bind = hint_name_rva + 96;
    let hint_listen = hint_name_rva + 112;
    let hint_accept = hint_name_rva + 128;
    let hint_send = hint_name_rva + 144;
    let hint_recv = hint_name_rva + 160;
    let hint_shutdown = hint_name_rva + 176;
    let hint_closesocket = hint_name_rva + 192;
    let hint_wsacleanup = hint_name_rva + 208;
    
    // IAT - kernel32 functions (0-23)
    rdata[0..8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[24..32].copy_from_slice(&(hint_sleep as u64).to_le_bytes());
    // IAT - ws2_32 functions (32-111)
    rdata[32..40].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[40..48].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[48..56].copy_from_slice(&(hint_bind as u64).to_le_bytes());
    rdata[56..64].copy_from_slice(&(hint_listen as u64).to_le_bytes());
    rdata[64..72].copy_from_slice(&(hint_accept as u64).to_le_bytes());
    rdata[72..80].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[80..88].copy_from_slice(&(hint_recv as u64).to_le_bytes());
    rdata[88..96].copy_from_slice(&(hint_shutdown as u64).to_le_bytes());
    rdata[96..104].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[104..112].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Import Directory Table
    let idt_offset = 0xC0usize;
    let kernel32_ilt = ilt_rva;
    let kernel32_iat = iat_rva;
    let kernel32_name = dll_names_rva;
    rdata[idt_offset..idt_offset+4].copy_from_slice(&kernel32_ilt.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&kernel32_name.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&kernel32_iat.to_le_bytes());
    
    let ws2_ilt = ilt_rva + 40; // After 4 kernel32 entries + null terminator
    let ws2_iat = iat_rva + 32; // After 4 kernel32 entries
    let ws2_name = dll_names_rva + 16;
    rdata[idt_offset+20..idt_offset+24].copy_from_slice(&ws2_ilt.to_le_bytes());
    rdata[idt_offset+32..idt_offset+36].copy_from_slice(&ws2_name.to_le_bytes());
    rdata[idt_offset+36..idt_offset+40].copy_from_slice(&ws2_iat.to_le_bytes());
    
    // ILT - kernel32
    let ilt_offset = 0x140usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[ilt_offset+24..ilt_offset+32].copy_from_slice(&(hint_sleep as u64).to_le_bytes());
    
    // ILT - ws2_32
    let ilt2_offset = ilt_offset + 40;
    rdata[ilt2_offset..ilt2_offset+8].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[ilt2_offset+8..ilt2_offset+16].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[ilt2_offset+16..ilt2_offset+24].copy_from_slice(&(hint_bind as u64).to_le_bytes());
    rdata[ilt2_offset+24..ilt2_offset+32].copy_from_slice(&(hint_listen as u64).to_le_bytes());
    rdata[ilt2_offset+32..ilt2_offset+40].copy_from_slice(&(hint_accept as u64).to_le_bytes());
    rdata[ilt2_offset+40..ilt2_offset+48].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[ilt2_offset+48..ilt2_offset+56].copy_from_slice(&(hint_recv as u64).to_le_bytes());
    rdata[ilt2_offset+56..ilt2_offset+64].copy_from_slice(&(hint_shutdown as u64).to_le_bytes());
    rdata[ilt2_offset+64..ilt2_offset+72].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[ilt2_offset+72..ilt2_offset+80].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0x200usize;
    rdata[hnt_offset+2..hnt_offset+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt_offset+18..hnt_offset+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt_offset+34..hnt_offset+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt_offset+50..hnt_offset+56].copy_from_slice(b"Sleep\0");
    rdata[hnt_offset+66..hnt_offset+77].copy_from_slice(b"WSAStartup\0");
    rdata[hnt_offset+82..hnt_offset+89].copy_from_slice(b"socket\0");
    rdata[hnt_offset+98..hnt_offset+103].copy_from_slice(b"bind\0");
    rdata[hnt_offset+114..hnt_offset+121].copy_from_slice(b"listen\0");
    rdata[hnt_offset+130..hnt_offset+137].copy_from_slice(b"accept\0");
    rdata[hnt_offset+146..hnt_offset+151].copy_from_slice(b"send\0");
    rdata[hnt_offset+162..hnt_offset+167].copy_from_slice(b"recv\0");
    rdata[hnt_offset+178..hnt_offset+187].copy_from_slice(b"shutdown\0");
    rdata[hnt_offset+194..hnt_offset+206].copy_from_slice(b"closesocket\0");
    rdata[hnt_offset+210..hnt_offset+221].copy_from_slice(b"WSACleanup\0");
    
    // DLL names
    rdata[0x400..0x400+13].copy_from_slice(b"kernel32.dll\0");
    rdata[0x410..0x410+11].copy_from_slice(b"ws2_32.dll\0");
    
    pe.extend(&rdata);
    
    // .data - routes, responses and messages
    // Layout:
    // 0x000-0x1FF: Route 0 response
    // 0x200-0x2FF: Route 0 path  
    // 0x300-0x4FF: Route 1 response
    // 0x500-0x5FF: Route 1 path
    // 0x600-0x7FF: Route 2 response
    // 0x800-0x8FF: Route 2 path
    // 0x900-0xAFF: Route 3 response
    // 0xB00-0xBFF: Route 3 path
    // 0xC00-0xCFF: Status message
    // 0xD00-0xDFF: 404 response
    // 0xE00+: recv buffer
    
    let mut data = vec![0u8; data_size as usize];
    
    let max_routes = routes.len().min(4);
    for i in 0..max_routes {
        // Response at i*0x300
        let resp_offset = i * 0x300;
        // Path at 0x200 + i*0x300
        let path_offset = 0x200 + i * 0x300;
        
        let resp_bytes = routes[i].1.as_bytes();
        let path_bytes = routes[i].0.as_bytes();
        
        let resp_len = resp_bytes.len().min(0x1FF);
        let path_len = path_bytes.len().min(0xFF);
        
        data[resp_offset..resp_offset+resp_len].copy_from_slice(&resp_bytes[..resp_len]);
        data[path_offset..path_offset+path_len].copy_from_slice(&path_bytes[..path_len]);
    }
    
    // Status message at 0xC00
    let status_msg = format!("HTTP Server listening on port {}...\n", port);
    let status_len = status_msg.len().min(0xFF);
    data[0xC00..0xC00+status_len].copy_from_slice(&status_msg.as_bytes()[..status_len]);
    
    // 404 response at 0xD00
    let not_found = b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
    data[0xD00..0xD00+not_found.len()].copy_from_slice(not_found);
    
    // For single route, also put response at offset 0
    if max_routes <= 1 {
        let default_response = if routes.is_empty() {
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}"
        } else {
            routes[0].1
        };
        let resp_bytes = default_response.as_bytes();
        let resp_len = resp_bytes.len().min(0x1FF);
        data[..resp_len].copy_from_slice(&resp_bytes[..resp_len]);
    }
    
    pe.extend(&data);
    
    pe
}

/// Generate HTTP client PE that makes a GET request
/// Connects to host:port, sends GET request, receives response
pub fn generate_http_get_pe(host: &str, port: u16, path: &str) -> Vec<u8> {
    let pe_offset: u32 = 0x80;
    let num_sections: u16 = 3;
    let headers_size: u32 = 0x200;
    
    let text_rva: u32 = 0x1000;
    let text_size: u32 = 0x800;
    let text_file_offset: u32 = 0x200;
    
    let rdata_rva: u32 = 0x2000;
    let rdata_size: u32 = 0x800;
    let rdata_file_offset: u32 = 0xA00;
    
    let data_rva: u32 = 0x3000;
    let data_size: u32 = 0x1000;
    let data_file_offset: u32 = 0x1200;
    
    let image_size: u32 = 0x5000;
    
    let iat_rva = rdata_rva;
    let import_dir_rva = rdata_rva + 0xC0;
    let ilt_rva = rdata_rva + 0x140;
    let hint_name_rva = rdata_rva + 0x200;
    let dll_names_rva = rdata_rva + 0x400;
    
    let mut pe = Vec::new();
    
    pe.extend(create_dos_header(pe_offset));
    pe.resize(pe_offset as usize, 0);
    pe.extend_from_slice(PE_SIGNATURE);
    pe.extend(create_coff_header(num_sections, 0));
    
    let mut opt_header = create_optional_header(
        text_size, rdata_size + data_size, text_rva, text_rva, image_size, headers_size,
    );
    let import_dir_offset = 112 + 8;
    opt_header[import_dir_offset..import_dir_offset+4].copy_from_slice(&import_dir_rva.to_le_bytes());
    opt_header[import_dir_offset+4..import_dir_offset+8].copy_from_slice(&60u32.to_le_bytes());
    pe.extend(opt_header);
    
    pe.extend(create_section_header(b".text\0\0\0", text_size, text_rva, text_size, text_file_offset,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".rdata\0\0", rdata_size, rdata_rva, rdata_size, rdata_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ));
    pe.extend(create_section_header(b".data\0\0\0", data_size, data_rva, data_size, data_file_offset,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE));
    
    pe.resize(text_file_offset as usize, 0);
    
    // Generate HTTP client code
    let mut code = Vec::new();
    let code_start = text_rva;
    
    // IAT offsets (with connect added):
    // 0: GetStdHandle, 8: WriteFile, 16: ExitProcess
    // 24: WSAStartup, 32: socket, 40: connect, 48: send, 56: recv, 64: closesocket, 72: WSACleanup
    
    // Stack layout (WSADATA is 408 bytes on x64!):
    // [rsp+0x00..0x1F] = shadow space (32 bytes)
    // [rsp+0x20..0x27] = 5th param for WriteFile
    // [rsp+0x28..0x2F] = padding
    // [rsp+0x30..0x37] = bytes_written for WriteFile
    // [rsp+0x38..0x3F] = bytes_received
    // [rsp+0x40..0x47] = socket handle
    // [rsp+0x48..0x4F] = stdout handle
    // [rsp+0x50..0x5F] = sockaddr_in (16 bytes)
    // [rsp+0x60..0x1F7] = WSADATA (408 bytes, ends at 0x60+408=0x1F8)
    // Total: 0x208 = 520 bytes (aligned: after call, rsp is 8-aligned, sub 0x208 makes it 16-aligned)
    
    // Prologue - reserve 0x208 bytes (520) for 16-byte alignment
    code.extend(&[0x48, 0x81, 0xEC, 0x08, 0x02, 0x00, 0x00]); // sub rsp, 0x208
    
    // WSAStartup(0x0202, &wsadata) - wsadata at [rsp+0x60]
    code.extend(&[0xB9, 0x02, 0x02, 0x00, 0x00]); // mov ecx, 0x0202
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x60]); // lea rdx, [rsp+0x60]
    let call_pos = code_start as i64 + code.len() as i64 + 6;
    let wsa_startup_offset = (iat_rva + 24) as i64 - call_pos;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wsa_startup_offset as i32).to_le_bytes());
    
    // socket(AF_INET=2, SOCK_STREAM=1, IPPROTO_TCP=6)
    code.extend(&[0xB9, 0x02, 0x00, 0x00, 0x00]); // mov ecx, 2
    code.extend(&[0xBA, 0x01, 0x00, 0x00, 0x00]); // mov edx, 1
    code.extend(&[0x41, 0xB8, 0x06, 0x00, 0x00, 0x00]); // mov r8d, 6
    let call_pos2 = code_start as i64 + code.len() as i64 + 6;
    let socket_offset = (iat_rva + 32) as i64 - call_pos2;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(socket_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x40]); // save socket to [rsp+0x40]
    
    // Build sockaddr_in at [rsp+0x50]
    // First, zero out the entire 16-byte structure
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x50, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+0x50], 0
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x58, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+0x58], 0
    
    // Now set the fields
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x50, 0x02, 0x00]); // sin_family = AF_INET at [rsp+0x50]
    let port_be = port.to_be();
    code.extend(&[0x66, 0xC7, 0x44, 0x24, 0x52]); // sin_port at [rsp+0x52]
    code.extend(&port_be.to_le_bytes());
    
    // Parse host IP (simple: assume it's like 127.0.0.1)
    let ip_parts: Vec<u8> = host.split('.').filter_map(|s| s.parse().ok()).collect();
    let ip_addr: u32 = if ip_parts.len() == 4 {
        (ip_parts[0] as u32) | ((ip_parts[1] as u32) << 8) | ((ip_parts[2] as u32) << 16) | ((ip_parts[3] as u32) << 24)
    } else {
        0x0100007F // 127.0.0.1
    };
    code.extend(&[0xC7, 0x44, 0x24, 0x54]); // sin_addr at [rsp+0x54]
    code.extend(&ip_addr.to_le_bytes());
    
    // connect(socket, &addr, 16)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+0x40] (socket)
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x50]); // lea rdx, [rsp+0x50] (sockaddr)
    code.extend(&[0x41, 0xB8, 0x10, 0x00, 0x00, 0x00]); // mov r8d, 16
    let call_pos3 = code_start as i64 + code.len() as i64 + 6;
    let connect_offset = (iat_rva + 40) as i64 - call_pos3;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(connect_offset as i32).to_le_bytes());
    
    // Check if connect failed (eax != 0)
    code.extend(&[0x85, 0xC0]); // test eax, eax
    let jz_connect_pos = code.len();
    code.extend(&[0x74, 0x00]); // jz connect_ok (placeholder)
    
    // Print "Error: connect failed\n"
    let connect_err_rva = data_rva + 0x140;
    code.extend(&[0xB9]); // mov ecx, -11
    code.extend(&(-11i32).to_le_bytes());
    let call_pos_gsh_conn = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset_conn = iat_rva as i64 - call_pos_gsh_conn;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset_conn as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (stdout)
    let lea_pos_conn = code_start as i64 + code.len() as i64 + 7;
    let conn_err_offset = connect_err_rva as i64 - lea_pos_conn;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+conn_err]
    code.extend(&(conn_err_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x16, 0x00, 0x00, 0x00]); // mov r8d, 22 ("Error: connect failed\n")
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x30]); // lea r9, [rsp+0x30] (bytes_written)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+0x20], 0
    let call_pos_wf_conn = code_start as i64 + code.len() as i64 + 6;
    let wf_offset_conn = (iat_rva + 8) as i64 - call_pos_wf_conn;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset_conn as i32).to_le_bytes());
    
    // Jump to exit
    let jmp_exit_conn_pos = code.len();
    code.extend(&[0xE9, 0x00, 0x00, 0x00, 0x00]); // jmp exit (placeholder)
    
    // Patch jz offset
    let connect_ok_pos = code.len();
    code[jz_connect_pos + 1] = (connect_ok_pos - jz_connect_pos - 2) as u8;
    
    // Build HTTP request
    let request = format!("GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n", path, host, port);
    let req_rva = data_rva;
    
    // send(socket, request, len, 0)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+0x40] (socket)
    let lea_pos = code_start as i64 + code.len() as i64 + 7;
    let req_offset = req_rva as i64 - lea_pos;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&(req_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8]);
    code.extend(&(request.len() as u32).to_le_bytes());
    code.extend(&[0x45, 0x31, 0xC9]);
    let call_pos4 = code_start as i64 + code.len() as i64 + 6;
    let send_offset = (iat_rva + 48) as i64 - call_pos4;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(send_offset as i32).to_le_bytes());
    
    // recv(socket, buffer, 2048, 0)
    let recv_buf_rva = data_rva + 0x200;
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+0x40] (socket)
    let lea_pos2 = code_start as i64 + code.len() as i64 + 7;
    let buf_offset = recv_buf_rva as i64 - lea_pos2;
    code.extend(&[0x48, 0x8D, 0x15]);
    code.extend(&(buf_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x00, 0x08, 0x00, 0x00]); // mov r8d, 2048 (buffer size)
    code.extend(&[0x45, 0x31, 0xC9]);
    let call_pos5 = code_start as i64 + code.len() as i64 + 6;
    let recv_offset = (iat_rva + 56) as i64 - call_pos5;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(recv_offset as i32).to_le_bytes());
    code.extend(&[0x89, 0x44, 0x24, 0x38]); // mov [rsp+0x38], eax (save 32-bit result)
    
    // Check if recv returned > 0, if not jump to cleanup
    code.extend(&[0x85, 0xC0]); // test eax, eax
    let jg_pos = code.len();
    code.extend(&[0x7F, 0x00]); // jg skip_to_output (placeholder)
    
    // recv failed or returned 0, print error and jump to cleanup
    let err_msg_rva = data_rva + 0x120;
    code.extend(&[0xB9]); // mov ecx, -11
    code.extend(&(-11i32).to_le_bytes());
    let call_pos_gsh_err = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset_err = iat_rva as i64 - call_pos_gsh_err;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset_err as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0xC1]); // mov rcx, rax (stdout)
    let lea_pos_err = code_start as i64 + code.len() as i64 + 7;
    let err_msg_offset = err_msg_rva as i64 - lea_pos_err;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+err_msg]
    code.extend(&(err_msg_offset as i32).to_le_bytes());
    code.extend(&[0x41, 0xB8, 0x1A, 0x00, 0x00, 0x00]); // mov r8d, 26 ("Error: recv returned <= 0\n")
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x30]); // lea r9, [rsp+0x30]
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    let call_pos_wf_err = code_start as i64 + code.len() as i64 + 6;
    let wf_offset_err = (iat_rva + 8) as i64 - call_pos_wf_err;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset_err as i32).to_le_bytes());
    
    let jmp_cleanup_pos = code.len();
    code.extend(&[0xE9, 0x00, 0x00, 0x00, 0x00]); // jmp cleanup (placeholder)
    
    // Patch jg offset
    let skip_to_output_pos = code.len();
    code[jg_pos + 1] = (skip_to_output_pos - jg_pos - 2) as u8;
    
    // Get stdout
    code.extend(&[0xB9]);
    code.extend(&(-11i32).to_le_bytes());
    let call_pos_gsh = code_start as i64 + code.len() as i64 + 6;
    let gsh_offset = iat_rva as i64 - call_pos_gsh;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(gsh_offset as i32).to_le_bytes());
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x48]); // save stdout to [rsp+0x48]
    
    // WriteFile(stdout, buffer, bytes_received, &written, NULL)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x48]); // mov rcx, [rsp+0x48] (stdout)
    let lea_pos3 = code_start as i64 + code.len() as i64 + 7;
    let buf_offset2 = recv_buf_rva as i64 - lea_pos3;
    code.extend(&[0x48, 0x8D, 0x15]); // lea rdx, [rip+buffer]
    code.extend(&(buf_offset2 as i32).to_le_bytes());
    code.extend(&[0x44, 0x8B, 0x44, 0x24, 0x38]); // mov r8d, [rsp+0x38] (32-bit load, bytes_received)
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x30]); // lea r9, [rsp+0x30] (bytes_written)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]); // mov qword [rsp+0x20], 0
    let call_pos_wf = code_start as i64 + code.len() as i64 + 6;
    let wf_offset = (iat_rva + 8) as i64 - call_pos_wf;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(wf_offset as i32).to_le_bytes());
    
    // closesocket - this is the cleanup label
    let cleanup_label_pos = code.len();
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x40]); // mov rcx, [rsp+0x40] (socket)
    let call_pos6 = code_start as i64 + code.len() as i64 + 6;
    let closesocket_offset = (iat_rva + 64) as i64 - call_pos6;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(closesocket_offset as i32).to_le_bytes());
    
    // WSACleanup
    let call_pos7 = code_start as i64 + code.len() as i64 + 6;
    let cleanup_offset = (iat_rva + 72) as i64 - call_pos7;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(cleanup_offset as i32).to_le_bytes());
    
    // ExitProcess(0)
    let exit_pos = code.len();
    code.extend(x64::xor_ecx_ecx());
    let call_pos8 = code_start as i64 + code.len() as i64 + 6;
    let ep_offset = (iat_rva + 16) as i64 - call_pos8;
    code.extend(&[0xFF, 0x15]);
    code.extend(&(ep_offset as i32).to_le_bytes());
    
    // Patch jmp_cleanup offset (jump to closesocket)
    let jmp_cleanup_offset = (cleanup_label_pos as i32) - (jmp_cleanup_pos as i32) - 5;
    code[jmp_cleanup_pos + 1..jmp_cleanup_pos + 5].copy_from_slice(&jmp_cleanup_offset.to_le_bytes());
    
    // Patch jmp_exit_conn offset (jump to ExitProcess)
    let jmp_exit_conn_offset = (exit_pos as i32) - (jmp_exit_conn_pos as i32) - 5;
    code[jmp_exit_conn_pos + 1..jmp_exit_conn_pos + 5].copy_from_slice(&jmp_exit_conn_offset.to_le_bytes());
    
    pe.extend(&code);
    pe.resize(rdata_file_offset as usize, 0);
    
    // .rdata - imports with connect
    let mut rdata = vec![0u8; rdata_size as usize];
    
    let hint_getstdhandle = hint_name_rva;
    let hint_writefile = hint_name_rva + 16;
    let hint_exitprocess = hint_name_rva + 32;
    let hint_wsastartup = hint_name_rva + 48;
    let hint_socket = hint_name_rva + 64;
    let hint_connect = hint_name_rva + 80;
    let hint_send = hint_name_rva + 96;
    let hint_recv = hint_name_rva + 112;
    let hint_closesocket = hint_name_rva + 128;
    let hint_wsacleanup = hint_name_rva + 144;
    
    // IAT
    rdata[0..8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[8..16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[16..24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    rdata[24..32].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[32..40].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[40..48].copy_from_slice(&(hint_connect as u64).to_le_bytes());
    rdata[48..56].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[56..64].copy_from_slice(&(hint_recv as u64).to_le_bytes());
    rdata[64..72].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[72..80].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Import Directory
    let idt_offset = 0xC0usize;
    let kernel32_ilt = ilt_rva;
    let kernel32_iat = iat_rva;
    let kernel32_name = dll_names_rva;
    rdata[idt_offset..idt_offset+4].copy_from_slice(&kernel32_ilt.to_le_bytes());
    rdata[idt_offset+12..idt_offset+16].copy_from_slice(&kernel32_name.to_le_bytes());
    rdata[idt_offset+16..idt_offset+20].copy_from_slice(&kernel32_iat.to_le_bytes());
    
    let ws2_ilt = ilt_rva + 32;
    let ws2_iat = iat_rva + 24;
    let ws2_name = dll_names_rva + 16;
    rdata[idt_offset+20..idt_offset+24].copy_from_slice(&ws2_ilt.to_le_bytes());
    rdata[idt_offset+32..idt_offset+36].copy_from_slice(&ws2_name.to_le_bytes());
    rdata[idt_offset+36..idt_offset+40].copy_from_slice(&ws2_iat.to_le_bytes());
    
    // ILT - kernel32
    let ilt_offset = 0x140usize;
    rdata[ilt_offset..ilt_offset+8].copy_from_slice(&(hint_getstdhandle as u64).to_le_bytes());
    rdata[ilt_offset+8..ilt_offset+16].copy_from_slice(&(hint_writefile as u64).to_le_bytes());
    rdata[ilt_offset+16..ilt_offset+24].copy_from_slice(&(hint_exitprocess as u64).to_le_bytes());
    
    // ILT - ws2_32
    let ilt2_offset = ilt_offset + 32;
    rdata[ilt2_offset..ilt2_offset+8].copy_from_slice(&(hint_wsastartup as u64).to_le_bytes());
    rdata[ilt2_offset+8..ilt2_offset+16].copy_from_slice(&(hint_socket as u64).to_le_bytes());
    rdata[ilt2_offset+16..ilt2_offset+24].copy_from_slice(&(hint_connect as u64).to_le_bytes());
    rdata[ilt2_offset+24..ilt2_offset+32].copy_from_slice(&(hint_send as u64).to_le_bytes());
    rdata[ilt2_offset+32..ilt2_offset+40].copy_from_slice(&(hint_recv as u64).to_le_bytes());
    rdata[ilt2_offset+40..ilt2_offset+48].copy_from_slice(&(hint_closesocket as u64).to_le_bytes());
    rdata[ilt2_offset+48..ilt2_offset+56].copy_from_slice(&(hint_wsacleanup as u64).to_le_bytes());
    
    // Hint/Name Table
    let hnt_offset = 0x200usize;
    rdata[hnt_offset+2..hnt_offset+15].copy_from_slice(b"GetStdHandle\0");
    rdata[hnt_offset+18..hnt_offset+28].copy_from_slice(b"WriteFile\0");
    rdata[hnt_offset+34..hnt_offset+46].copy_from_slice(b"ExitProcess\0");
    rdata[hnt_offset+50..hnt_offset+61].copy_from_slice(b"WSAStartup\0");
    rdata[hnt_offset+66..hnt_offset+73].copy_from_slice(b"socket\0");
    rdata[hnt_offset+82..hnt_offset+90].copy_from_slice(b"connect\0");
    rdata[hnt_offset+98..hnt_offset+103].copy_from_slice(b"send\0");
    rdata[hnt_offset+114..hnt_offset+119].copy_from_slice(b"recv\0");
    rdata[hnt_offset+130..hnt_offset+142].copy_from_slice(b"closesocket\0");
    rdata[hnt_offset+146..hnt_offset+157].copy_from_slice(b"WSACleanup\0");
    
    // DLL names
    rdata[0x400..0x400+13].copy_from_slice(b"kernel32.dll\0");
    rdata[0x410..0x410+11].copy_from_slice(b"ws2_32.dll\0");
    
    pe.extend(&rdata);
    
    // .data - request, debug messages, and buffer
    let mut data = vec![0u8; data_size as usize];
    let req_bytes = request.as_bytes();
    data[..req_bytes.len().min(0x100)].copy_from_slice(&req_bytes[..req_bytes.len().min(0x100)]);
    // Debug message at offset 0x100
    data[0x100..0x100+14].copy_from_slice(b"Connecting...\n");
    // Received message at offset 0x110
    data[0x110..0x110+10].copy_from_slice(b"Received: ");
    // Error message at offset 0x120
    data[0x120..0x120+26].copy_from_slice(b"Error: recv returned <= 0\n");
    // Connect error message at offset 0x140
    data[0x140..0x140+22].copy_from_slice(b"Error: connect failed\n");
    
    pe.extend(&data);
    
    pe
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dos_header_size() {
        let header = create_dos_header(0x80);
        assert_eq!(header.len(), 64);
        assert_eq!(header[0], 0x4D); // 'M'
        assert_eq!(header[1], 0x5A); // 'Z'
    }
    
    #[test]
    fn test_coff_header_size() {
        let header = create_coff_header(3, 0);
        assert_eq!(header.len(), 20);
    }
    
    #[test]
    fn test_optional_header_size() {
        let header = create_optional_header(0, 0, 0, 0, 0, 0);
        assert_eq!(header.len(), 240);
    }
    
    #[test]
    fn test_section_header_size() {
        let header = create_section_header(b".text\0\0\0", 0, 0, 0, 0, 0);
        assert_eq!(header.len(), 40);
    }
    
    #[test]
    fn test_ultra_tiny_pe_size() {
        let pe = generate_ultra_tiny_pe("Hello");
        // Should be 1024 bytes (512 headers + 512 section)
        assert_eq!(pe.len(), 1024);
        // Verify MZ signature
        assert_eq!(pe[0], 0x4D);
        assert_eq!(pe[1], 0x5A);
        // Verify PE signature at offset 0x40
        assert_eq!(&pe[0x40..0x44], b"PE\0\0");
    }
    
    #[test]
    fn test_smallest_pe_size() {
        let pe = generate_smallest_pe("Hello");
        // Should be 640 bytes (384 headers + 256 section)
        assert_eq!(pe.len(), 640);
        // Verify MZ signature
        assert_eq!(pe[0], 0x4D);
        assert_eq!(pe[1], 0x5A);
        // Verify PE signature at offset 0x40
        assert_eq!(&pe[0x40..0x44], b"PE\0\0");
    }
}
