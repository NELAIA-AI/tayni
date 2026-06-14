//! ELF64 Generator for Linux x86-64
//! Generates minimal ELF executables without external dependencies

use crate::ir::{Graph, Node, Value, Arg, Op};
use std::collections::HashMap;

// ELF64 Constants
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;  // Little endian
const EV_CURRENT: u8 = 1;
const ELFOSABI_NONE: u8 = 0;
const ET_EXEC: u16 = 2;     // Executable
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;        // Execute
const PF_W: u32 = 2;        // Write
const PF_R: u32 = 4;        // Read

const BASE_ADDR: u64 = 0x400000;
const PAGE_SIZE: u64 = 0x1000;

/// Generate minimal ELF64 "Hello World" executable
pub fn generate_hello_elf() -> Vec<u8> {
    let message = b"Hello, NELAIA on Linux!\n";
    let msg_len = message.len();
    
    // ELF header: 64 bytes
    // Program header: 56 bytes
    // Code + data
    
    let ehdr_size: u64 = 64;
    let phdr_size: u64 = 56;
    let code_offset: u64 = ehdr_size + phdr_size;
    
    // Code: write syscall + exit syscall
    let mut code = Vec::new();
    
    // mov rax, 1 (sys_write)
    code.extend(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
    // mov rdi, 1 (stdout)
    code.extend(&[0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]);
    // lea rsi, [rip + msg_offset]
    let msg_offset_pos = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]); // placeholder
    // mov rdx, msg_len
    code.extend(&[0x48, 0xC7, 0xC2]);
    code.extend(&(msg_len as u32).to_le_bytes());
    // syscall
    code.extend(&[0x0F, 0x05]);
    
    // mov rax, 60 (sys_exit)
    code.extend(&[0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00]);
    // xor rdi, rdi (exit code 0)
    code.extend(&[0x48, 0x31, 0xFF]);
    // syscall
    code.extend(&[0x0F, 0x05]);
    
    // Message data
    let msg_start = code.len();
    code.extend(message);
    
    // Fix up message offset (RIP-relative)
    let rip_after_lea = msg_offset_pos + 7;
    let msg_rel_offset = (msg_start - rip_after_lea) as i32;
    code[msg_offset_pos + 3..msg_offset_pos + 7].copy_from_slice(&msg_rel_offset.to_le_bytes());
    
    let file_size = code_offset + code.len() as u64;
    let mem_size = file_size;
    
    let mut elf = Vec::new();
    
    // ELF Header (64 bytes)
    elf.extend(&ELF_MAGIC);
    elf.push(ELFCLASS64);
    elf.push(ELFDATA2LSB);
    elf.push(EV_CURRENT);
    elf.push(ELFOSABI_NONE);
    elf.extend(&[0u8; 8]); // padding
    elf.extend(&ET_EXEC.to_le_bytes());
    elf.extend(&EM_X86_64.to_le_bytes());
    elf.extend(&1u32.to_le_bytes()); // e_version
    elf.extend(&(BASE_ADDR + code_offset).to_le_bytes()); // e_entry
    elf.extend(&ehdr_size.to_le_bytes()); // e_phoff
    elf.extend(&0u64.to_le_bytes()); // e_shoff (no section headers)
    elf.extend(&0u32.to_le_bytes()); // e_flags
    elf.extend(&64u16.to_le_bytes()); // e_ehsize
    elf.extend(&56u16.to_le_bytes()); // e_phentsize
    elf.extend(&1u16.to_le_bytes()); // e_phnum
    elf.extend(&64u16.to_le_bytes()); // e_shentsize
    elf.extend(&0u16.to_le_bytes()); // e_shnum
    elf.extend(&0u16.to_le_bytes()); // e_shstrndx
    
    // Program Header (56 bytes)
    elf.extend(&PT_LOAD.to_le_bytes()); // p_type
    elf.extend(&(PF_R | PF_X).to_le_bytes()); // p_flags
    elf.extend(&0u64.to_le_bytes()); // p_offset
    elf.extend(&BASE_ADDR.to_le_bytes()); // p_vaddr
    elf.extend(&BASE_ADDR.to_le_bytes()); // p_paddr
    elf.extend(&file_size.to_le_bytes()); // p_filesz
    elf.extend(&mem_size.to_le_bytes()); // p_memsz
    elf.extend(&PAGE_SIZE.to_le_bytes()); // p_align
    
    // Code
    elf.extend(&code);
    
    elf
}

/// Generate ELF from NELAIA graph
pub fn generate_elf_from_graph(graph: &Graph) -> Vec<u8> {
    // For now, extract message and generate simple print program
    let mut message = String::from("Hello from NELAIA!\n");
    let mut print_len: usize = message.len();
    
    // Look for string literals and PRT
    for node in &graph.nodes {
        match node {
            Node::Literal { value: Value::String(s), .. } => {
                message = s.clone();
            }
            Node::Operation { op: Op::Prt, args, .. } => {
                if let Some(Arg::Lit(Value::Int(n))) = args.get(1) {
                    print_len = *n as usize;
                }
            }
            _ => {}
        }
    }
    
    generate_elf_with_message(&message, print_len)
}

fn generate_elf_with_message(message: &str, len: usize) -> Vec<u8> {
    let msg_bytes = message.as_bytes();
    let actual_len = len.min(msg_bytes.len());
    
    let ehdr_size: u64 = 64;
    let phdr_size: u64 = 56;
    let code_offset: u64 = ehdr_size + phdr_size;
    
    let mut code = Vec::new();
    
    // mov rax, 1 (sys_write)
    code.extend(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
    // mov rdi, 1 (stdout)
    code.extend(&[0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]);
    // lea rsi, [rip + msg_offset]
    let msg_offset_pos = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]);
    // mov rdx, len
    code.extend(&[0x48, 0xC7, 0xC2]);
    code.extend(&(actual_len as u32).to_le_bytes());
    // syscall
    code.extend(&[0x0F, 0x05]);
    
    // mov rax, 60 (sys_exit)
    code.extend(&[0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00]);
    // xor rdi, rdi
    code.extend(&[0x48, 0x31, 0xFF]);
    // syscall
    code.extend(&[0x0F, 0x05]);
    
    // Message
    let msg_start = code.len();
    code.extend(&msg_bytes[..actual_len]);
    
    // Fix offset
    let rip_after_lea = msg_offset_pos + 7;
    let msg_rel_offset = (msg_start - rip_after_lea) as i32;
    code[msg_offset_pos + 3..msg_offset_pos + 7].copy_from_slice(&msg_rel_offset.to_le_bytes());
    
    let file_size = code_offset + code.len() as u64;
    
    let mut elf = Vec::new();
    
    // ELF Header
    elf.extend(&ELF_MAGIC);
    elf.push(ELFCLASS64);
    elf.push(ELFDATA2LSB);
    elf.push(EV_CURRENT);
    elf.push(ELFOSABI_NONE);
    elf.extend(&[0u8; 8]);
    elf.extend(&ET_EXEC.to_le_bytes());
    elf.extend(&EM_X86_64.to_le_bytes());
    elf.extend(&1u32.to_le_bytes());
    elf.extend(&(BASE_ADDR + code_offset).to_le_bytes());
    elf.extend(&ehdr_size.to_le_bytes());
    elf.extend(&0u64.to_le_bytes());
    elf.extend(&0u32.to_le_bytes());
    elf.extend(&64u16.to_le_bytes());
    elf.extend(&56u16.to_le_bytes());
    elf.extend(&1u16.to_le_bytes());
    elf.extend(&64u16.to_le_bytes());
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
    elf.extend(&PAGE_SIZE.to_le_bytes());
    
    elf.extend(&code);
    
    elf
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_elf_header_size() {
        let elf = generate_hello_elf();
        assert!(elf.len() >= 120); // At least header + program header
        assert_eq!(&elf[0..4], &ELF_MAGIC);
    }
}
