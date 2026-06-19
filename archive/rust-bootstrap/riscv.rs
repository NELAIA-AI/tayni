//! ELF64 Generator for RISC-V 64-bit
//! Generates minimal ELF executables for RISC-V Linux

use crate::ir::{Graph, Node, Value, Arg, Op};
use crate::target::format::elf64::{Elf64Config, generate_elf64, PF_R, PF_X};
use std::collections::HashMap;

// Re-export for backward compatibility
pub use crate::target::format::elf64::{
    ELF_MAGIC, ELFCLASS64, ELFDATA2LSB, EV_CURRENT, ELFOSABI_NONE,
    ET_EXEC, EM_RISCV, PT_LOAD,
};

const BASE_ADDR: u64 = 0x10000;

/// Generate minimal RISC-V ELF executable
pub fn generate_hello_elf_riscv() -> Vec<u8> {
    let message = b"Hello, TAYNI!\n";
    generate_tiny_elf_riscv_with_message(message)
}

fn generate_tiny_elf_riscv_with_message(message: &[u8]) -> Vec<u8> {
    let msg_len = message.len();
    let ehdr_size: usize = 64;
    let phdr_size: usize = 56;
    let code_offset: usize = ehdr_size + phdr_size;
    
    let mut code = Vec::new();
    
    // RISC-V syscall: write(1, msg, len)
    // a7 = syscall number (64 = write)
    // a0 = fd (1)
    // a1 = buf
    // a2 = count
    
    // li a0, 1 (stdout) - addi a0, x0, 1
    code.extend(&[0x13, 0x05, 0x10, 0x00]);
    
    // auipc a1, 0 + addi for PC-relative address
    let auipc_pos = code.len();
    code.extend(&[0x97, 0x05, 0x00, 0x00]); // auipc a1, 0 (placeholder)
    code.extend(&[0x93, 0x85, 0x05, 0x00]); // addi a1, a1, offset (placeholder)
    
    // li a2, len
    let len_lo = (msg_len & 0xFFF) as u32;
    code.extend(&((0x00000613 | (len_lo << 20)).to_le_bytes()));
    
    // li a7, 64 (sys_write)
    code.extend(&[0x93, 0x08, 0x00, 0x04]); // addi a7, x0, 64
    
    // ecall
    code.extend(&[0x73, 0x00, 0x00, 0x00]);
    
    // exit(0)
    // li a0, 0
    code.extend(&[0x13, 0x05, 0x00, 0x00]);
    // li a7, 93 (sys_exit)
    code.extend(&[0x93, 0x08, 0xD0, 0x05]); // addi a7, x0, 93
    // ecall
    code.extend(&[0x73, 0x00, 0x00, 0x00]);
    
    let msg_start = code.len();
    code.extend(message);
    
    // Fix up auipc + addi
    let offset = msg_start as i32;
    let hi = ((offset + 0x800) >> 12) & 0xFFFFF;
    let lo = offset & 0xFFF;
    let auipc = 0x00000597u32 | ((hi as u32) << 12);
    let addi = 0x00058593u32 | ((lo as u32) << 20);
    code[auipc_pos..auipc_pos + 4].copy_from_slice(&auipc.to_le_bytes());
    code[auipc_pos + 4..auipc_pos + 8].copy_from_slice(&addi.to_le_bytes());
    
    let file_size = (code_offset + code.len()) as u64;
    let entry_point = BASE_ADDR + code_offset as u64;
    
    let mut elf = Vec::with_capacity(file_size as usize);
    
    // ELF Header
    elf.extend(&ELF_MAGIC);
    elf.push(ELFCLASS64);
    elf.push(ELFDATA2LSB);
    elf.push(EV_CURRENT);
    elf.push(ELFOSABI_NONE);
    elf.extend(&[0u8; 8]);
    elf.extend(&ET_EXEC.to_le_bytes());
    elf.extend(&EM_RISCV.to_le_bytes());
    elf.extend(&1u32.to_le_bytes());
    elf.extend(&entry_point.to_le_bytes());
    elf.extend(&(ehdr_size as u64).to_le_bytes());
    elf.extend(&0u64.to_le_bytes());
    elf.extend(&0u32.to_le_bytes());
    elf.extend(&(ehdr_size as u16).to_le_bytes());
    elf.extend(&(phdr_size as u16).to_le_bytes());
    elf.extend(&1u16.to_le_bytes());
    elf.extend(&0u16.to_le_bytes());
    elf.extend(&0u16.to_le_bytes());
    elf.extend(&0u16.to_le_bytes());
    
    // Program Header
    elf.extend(&PT_LOAD.to_le_bytes());
    elf.extend(&(PF_R | PF_X).to_le_bytes());
    elf.extend(&0u64.to_le_bytes());
    elf.extend(&BASE_ADDR.to_le_bytes());
    elf.extend(&BASE_ADDR.to_le_bytes());
    elf.extend(&file_size.to_le_bytes());
    elf.extend(&file_size.to_le_bytes());
    elf.extend(&0x1000u64.to_le_bytes());
    
    elf.extend(&code);
    elf
}

/// Generate RISC-V ELF from TAYNI graph
pub fn generate_elf_riscv_from_graph(graph: &Graph) -> Vec<u8> {
    let mut strings: Vec<(String, Vec<u8>)> = Vec::new();
    
    for node in &graph.nodes {
        if let Node::Literal { id, value: Value::String(s), runtime: _ } = node {
            strings.push((id.clone(), s.as_bytes().to_vec()));
        }
    }
    
    for node in &graph.nodes {
        if let Node::Operation { op: Op::Prt, args, .. } = node {
            if let Some(Arg::Ref(str_ref)) = args.first() {
                if let Some((_, data)) = strings.iter().find(|(id, _)| id == str_ref) {
                    return generate_tiny_elf_riscv_with_message(data);
                }
            }
        }
    }
    
    if let Some((_, data)) = strings.first() {
        return generate_tiny_elf_riscv_with_message(data);
    }
    
    generate_hello_elf_riscv()
}
