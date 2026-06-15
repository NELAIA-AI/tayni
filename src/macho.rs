//! Mach-O Generator for macOS (x86-64 and ARM64)
//! Generates minimal Mach-O executables without external dependencies

use crate::ir::{Graph, Node, Value, Arg, Op};
use std::collections::HashMap;

// Mach-O Magic Numbers
const MH_MAGIC_64: u32 = 0xFEEDFACF;           // 64-bit Mach-O
const MH_CIGAM_64: u32 = 0xCFFAEDFE;           // 64-bit Mach-O (byte-swapped)

// CPU Types
const CPU_TYPE_X86_64: u32 = 0x01000007;       // x86-64
const CPU_TYPE_ARM64: u32 = 0x0100000C;        // ARM64

// CPU Subtypes
const CPU_SUBTYPE_X86_64_ALL: u32 = 3;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;

// File Types
const MH_EXECUTE: u32 = 2;                     // Executable

// Flags
const MH_NOUNDEFS: u32 = 0x1;                  // No undefined references
const MH_PIE: u32 = 0x200000;                  // Position Independent Executable

// Load Commands
const LC_SEGMENT_64: u32 = 0x19;               // 64-bit segment
const LC_MAIN: u32 = 0x80000028;               // Main entry point
const LC_UNIXTHREAD: u32 = 0x5;                // Unix thread (older method)

// Segment/Section constants
const VM_PROT_READ: u32 = 0x1;
const VM_PROT_WRITE: u32 = 0x2;
const VM_PROT_EXECUTE: u32 = 0x4;

// macOS syscall numbers (different from Linux!)
// macOS uses BSD syscalls with 0x2000000 offset
const SYS_EXIT: u64 = 0x2000001;               // exit(status)
const SYS_WRITE: u64 = 0x2000004;              // write(fd, buf, len)

#[derive(Clone, Copy, PartialEq)]
pub enum MacOSArch {
    X86_64,
    ARM64,
}

/// Generate minimal Mach-O "Hello World" executable
pub fn generate_hello_macho(arch: MacOSArch) -> Vec<u8> {
    let message = b"Hello from NELAIA on macOS!\n";
    
    match arch {
        MacOSArch::X86_64 => generate_hello_macho_x64(message),
        MacOSArch::ARM64 => generate_hello_macho_arm64(message),
    }
}

/// Generate Mach-O for x86-64 macOS - MINIMAL VERSION
/// macOS kernel requires minimum 4096 bytes, but payload is only ~200 bytes
/// Returns (binary, payload_size) where binary is padded to 4096
fn generate_hello_macho_x64(message: &[u8]) -> Vec<u8> {
    let msg_len = message.len();
    
    let mut binary = Vec::new();
    
    // === Mach-O Header (32 bytes) ===
    binary.extend(&MH_MAGIC_64.to_le_bytes());           // magic
    binary.extend(&CPU_TYPE_X86_64.to_le_bytes());       // cputype
    binary.extend(&CPU_SUBTYPE_X86_64_ALL.to_le_bytes()); // cpusubtype
    binary.extend(&MH_EXECUTE.to_le_bytes());            // filetype
    binary.extend(&2u32.to_le_bytes());                  // ncmds (2 load commands)
    
    // Calculate sizes - MINIMAL
    let header_size: u32 = 32;
    let segment_cmd_size: u32 = 72;                      // LC_SEGMENT_64
    let main_cmd_size: u32 = 24;                         // LC_MAIN
    let load_cmds_size: u32 = segment_cmd_size + main_cmd_size;
    let text_offset: u64 = (header_size + load_cmds_size) as u64;
    
    binary.extend(&load_cmds_size.to_le_bytes());        // sizeofcmds
    binary.extend(&(MH_NOUNDEFS | MH_PIE).to_le_bytes()); // flags
    binary.extend(&0u32.to_le_bytes());                  // reserved
    
    // macOS kernel enforces minimum 4096 bytes (PAGE_SIZE)
    let page_size: u64 = 0x1000; // 4KB for x86-64
    
    // === LC_SEGMENT_64 __TEXT (72 bytes) ===
    binary.extend(&LC_SEGMENT_64.to_le_bytes());         // cmd
    binary.extend(&segment_cmd_size.to_le_bytes());      // cmdsize
    binary.extend(b"__TEXT\0\0\0\0\0\0\0\0\0\0");        // segname (16 bytes)
    binary.extend(&0u64.to_le_bytes());                  // vmaddr
    binary.extend(&page_size.to_le_bytes());             // vmsize (must be page-aligned)
    binary.extend(&0u64.to_le_bytes());                  // fileoff
    binary.extend(&page_size.to_le_bytes());             // filesize (must match file size)
    binary.extend(&(VM_PROT_READ | VM_PROT_EXECUTE).to_le_bytes()); // maxprot
    binary.extend(&(VM_PROT_READ | VM_PROT_EXECUTE).to_le_bytes()); // initprot
    binary.extend(&0u32.to_le_bytes());                  // nsects
    binary.extend(&0u32.to_le_bytes());                  // flags
    
    // === LC_MAIN (24 bytes) ===
    binary.extend(&LC_MAIN.to_le_bytes());               // cmd
    binary.extend(&main_cmd_size.to_le_bytes());         // cmdsize
    binary.extend(&text_offset.to_le_bytes());           // entryoff
    binary.extend(&0u64.to_le_bytes());                  // stacksize (0 = default)
    
    // === Code Section - starts immediately after headers ===
    // x86-64 macOS syscall convention:
    // syscall number in rax (with 0x2000000 offset)
    // args: rdi, rsi, rdx, r10, r8, r9
    
    // --- write(1, message, len) ---
    // mov rax, 0x2000004 (SYS_WRITE)
    binary.extend(&[0x48, 0xC7, 0xC0]);
    binary.extend(&(SYS_WRITE as u32).to_le_bytes());
    
    // mov rdi, 1 (stdout)
    binary.extend(&[0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]);
    
    // lea rsi, [rip + offset_to_message]
    let lea_pos = binary.len();
    binary.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]); // placeholder
    
    // mov rdx, msg_len
    binary.extend(&[0x48, 0xC7, 0xC2]);
    binary.extend(&(msg_len as u32).to_le_bytes());
    
    // syscall
    binary.extend(&[0x0F, 0x05]);
    
    // --- exit(0) ---
    // mov rax, 0x2000001 (SYS_EXIT)
    binary.extend(&[0x48, 0xC7, 0xC0]);
    binary.extend(&(SYS_EXIT as u32).to_le_bytes());
    
    // xor rdi, rdi (exit code 0)
    binary.extend(&[0x48, 0x31, 0xFF]);
    
    // syscall
    binary.extend(&[0x0F, 0x05]);
    
    // Message data - immediately after code
    let msg_offset = binary.len();
    binary.extend(message);
    
    // Fix up the LEA offset (RIP-relative)
    let lea_end = lea_pos + 7;
    let rel_offset = (msg_offset as i32) - (lea_end as i32);
    binary[lea_pos + 3..lea_pos + 7].copy_from_slice(&rel_offset.to_le_bytes());
    
    // Record payload size before padding
    let payload_size = binary.len();
    
    // Pad to 4096 bytes (macOS kernel requirement)
    // Files smaller than PAGE_SIZE are killed by the kernel
    while binary.len() < page_size as usize {
        binary.push(0);
    }
    
    // Store payload size in unused header bytes for reporting
    // We use bytes at offset 8-9 (part of e_ident padding) to store payload size
    // This is a hack but allows us to report the real payload size
    binary[8] = (payload_size & 0xFF) as u8;
    binary[9] = ((payload_size >> 8) & 0xFF) as u8;
    
    binary
}

/// Generate Mach-O for ARM64 macOS (Apple Silicon) - MINIMAL VERSION
/// Note: macOS requires minimum 16KB (0x4000) for ARM64 due to page size
fn generate_hello_macho_arm64(message: &[u8]) -> Vec<u8> {
    let msg_len = message.len();
    
    let mut binary = Vec::new();
    
    // === Mach-O Header (32 bytes) ===
    binary.extend(&MH_MAGIC_64.to_le_bytes());           // magic
    binary.extend(&CPU_TYPE_ARM64.to_le_bytes());        // cputype
    binary.extend(&CPU_SUBTYPE_ARM64_ALL.to_le_bytes()); // cpusubtype
    binary.extend(&MH_EXECUTE.to_le_bytes());            // filetype
    binary.extend(&2u32.to_le_bytes());                  // ncmds
    
    let header_size: u32 = 32;
    let segment_cmd_size: u32 = 72;
    let main_cmd_size: u32 = 24;
    let load_cmds_size: u32 = segment_cmd_size + main_cmd_size;
    let text_offset: u64 = (header_size + load_cmds_size) as u64;
    
    binary.extend(&load_cmds_size.to_le_bytes());        // sizeofcmds
    binary.extend(&(MH_NOUNDEFS | MH_PIE).to_le_bytes()); // flags
    binary.extend(&0u32.to_le_bytes());                  // reserved
    
    // === LC_SEGMENT_64 __TEXT ===
    // For ARM64 macOS, we MUST pad to 16KB (0x4000) due to page size requirements
    let page_size: u64 = 0x4000; // 16KB for ARM64
    
    binary.extend(&LC_SEGMENT_64.to_le_bytes());
    binary.extend(&segment_cmd_size.to_le_bytes());
    binary.extend(b"__TEXT\0\0\0\0\0\0\0\0\0\0");
    binary.extend(&0u64.to_le_bytes());                  // vmaddr
    binary.extend(&page_size.to_le_bytes());             // vmsize
    binary.extend(&0u64.to_le_bytes());                  // fileoff
    binary.extend(&page_size.to_le_bytes());             // filesize
    binary.extend(&(VM_PROT_READ | VM_PROT_EXECUTE).to_le_bytes());
    binary.extend(&(VM_PROT_READ | VM_PROT_EXECUTE).to_le_bytes());
    binary.extend(&0u32.to_le_bytes());                  // nsects
    binary.extend(&0u32.to_le_bytes());                  // flags
    
    // === LC_MAIN ===
    binary.extend(&LC_MAIN.to_le_bytes());
    binary.extend(&main_cmd_size.to_le_bytes());
    binary.extend(&text_offset.to_le_bytes());           // entryoff
    binary.extend(&0u64.to_le_bytes());                  // stacksize
    
    // === ARM64 Code ===
    // --- write(1, message, len) ---
    // mov x0, #1 (stdout)
    binary.extend(&arm64_mov_imm(0, 1));
    
    // adr x1, message (PC-relative address)
    let adr_pos = binary.len();
    binary.extend(&[0x00, 0x00, 0x00, 0x10]); // placeholder ADR x1, #offset
    
    // mov x2, #msg_len
    binary.extend(&arm64_mov_imm(2, msg_len as u64));
    
    // mov x16, #4 (SYS_WRITE)
    binary.extend(&arm64_mov_imm(16, 4));
    
    // svc #0x80
    binary.extend(&[0x01, 0x10, 0x00, 0xD4]);
    
    // --- exit(0) ---
    // mov x0, #0 (exit code)
    binary.extend(&arm64_mov_imm(0, 0));
    
    // mov x16, #1 (SYS_EXIT)
    binary.extend(&arm64_mov_imm(16, 1));
    
    // svc #0x80
    binary.extend(&[0x01, 0x10, 0x00, 0xD4]);
    
    // Message data
    let msg_offset = binary.len();
    binary.extend(message);
    
    // Fix up ADR instruction
    let adr_offset = (msg_offset - adr_pos) as i32;
    let adr_instr = arm64_adr(1, adr_offset);
    binary[adr_pos..adr_pos + 4].copy_from_slice(&adr_instr);
    
    // Record payload size before padding
    let payload_size = binary.len();
    
    // Pad to 16KB page boundary (ARM64 macOS requirement)
    while binary.len() < page_size as usize {
        binary.push(0);
    }
    
    // Store payload size in unused header bytes for reporting
    binary[8] = (payload_size & 0xFF) as u8;
    binary[9] = ((payload_size >> 8) & 0xFF) as u8;
    
    binary
}

/// Generate ARM64 MOV immediate instruction
fn arm64_mov_imm(reg: u8, value: u64) -> [u8; 4] {
    if value <= 0xFFFF {
        // MOVZ Xd, #imm16
        let instr: u32 = 0xD2800000 | ((value as u32 & 0xFFFF) << 5) | (reg as u32);
        instr.to_le_bytes()
    } else {
        // For larger values, we'd need MOVK instructions
        // For now, just handle small values
        let instr: u32 = 0xD2800000 | (((value & 0xFFFF) as u32) << 5) | (reg as u32);
        instr.to_le_bytes()
    }
}

/// Generate ARM64 ADR instruction (PC-relative address)
fn arm64_adr(reg: u8, offset: i32) -> [u8; 4] {
    // ADR Xd, label
    // Encodes a +/- 1MB offset
    let immlo = (offset & 0x3) as u32;
    let immhi = ((offset >> 2) & 0x7FFFF) as u32;
    let instr: u32 = 0x10000000 | (immlo << 29) | (immhi << 5) | (reg as u32);
    instr.to_le_bytes()
}

/// Generate Mach-O executable from NELAIA graph
pub fn generate_macho_from_graph(graph: &Graph, arch: MacOSArch) -> Vec<u8> {
    // Find all string literals and PRT operations
    let mut strings: Vec<(String, String)> = Vec::new(); // (id, value)
    let mut print_ops: Vec<(String, String, Option<i64>)> = Vec::new(); // (id, buffer_ref, len)
    
    for node in &graph.nodes {
        match node {
            Node::Literal { id, value } => {
                if let Value::String(s) = value {
                    strings.push((id.clone(), s.clone()));
                }
            }
            Node::Operation { id, op: Op::Prt, args } => {
                if let Some(Arg::Ref(buf_ref)) = args.get(0) {
                    let len = args.get(1).and_then(|a| {
                        if let Arg::Lit(Value::Int(n)) = a { Some(*n) } else { None }
                    });
                    print_ops.push((id.clone(), buf_ref.clone(), len));
                }
            }
            _ => {}
        }
    }
    
    // Build combined message
    let mut combined_msg = String::new();
    for (_id, value) in &strings {
        combined_msg.push_str(value);
    }
    
    if combined_msg.is_empty() {
        combined_msg = "Hello from NELAIA!\n".to_string();
    }
    
    // Generate appropriate binary
    match arch {
        MacOSArch::X86_64 => generate_hello_macho_x64(combined_msg.as_bytes()),
        MacOSArch::ARM64 => generate_hello_macho_arm64(combined_msg.as_bytes()),
    }
}
