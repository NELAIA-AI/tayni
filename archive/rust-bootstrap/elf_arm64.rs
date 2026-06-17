//! ELF64 Generator for Linux ARM64 (aarch64)
//! Generates minimal ELF executables for ARM64 Linux

use crate::ir::{Graph, Node, Value, Arg, Op};
use std::collections::HashMap;

// ELF64 Constants
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EV_CURRENT: u8 = 1;
const ELFOSABI_NONE: u8 = 0;
const ET_EXEC: u16 = 2;
const EM_AARCH64: u16 = 183;  // ARM64
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

const BASE_ADDR: u64 = 0x400000;

/// Generate minimal ARM64 ELF executable
pub fn generate_hello_elf_arm64() -> Vec<u8> {
    let message = b"Hello, TAYNI!\n";
    generate_tiny_elf_arm64_with_message(message)
}

/// Generate ARM64 ELF that prints a message
fn generate_tiny_elf_arm64_with_message(message: &[u8]) -> Vec<u8> {
    let msg_len = message.len();
    
    let ehdr_size: usize = 64;
    let phdr_size: usize = 56;
    let code_offset: usize = ehdr_size + phdr_size;
    
    let mut code = Vec::new();
    
    // ARM64 syscall: write(1, msg, len)
    // x8 = syscall number (64 = write)
    // x0 = fd (1 = stdout)
    // x1 = buf
    // x2 = count
    
    // mov x0, #1 (stdout)
    code.extend(&[0x20, 0x00, 0x80, 0xD2]); // 4 bytes
    
    // adr x1, msg (PC-relative address)
    let adr_pos = code.len();
    code.extend(&[0x00, 0x00, 0x00, 0x10]); // placeholder, will fix up
    
    // mov x2, #len
    let len_imm = (msg_len as u32) << 5;
    code.extend(&((0xD2800002 | len_imm).to_le_bytes())); // 4 bytes
    
    // mov x8, #64 (sys_write)
    code.extend(&[0x08, 0x08, 0x80, 0xD2]); // 4 bytes
    
    // svc #0 (syscall)
    code.extend(&[0x01, 0x00, 0x00, 0xD4]); // 4 bytes
    
    // exit(0)
    // mov x0, #0
    code.extend(&[0x00, 0x00, 0x80, 0xD2]); // 4 bytes
    
    // mov x8, #93 (sys_exit)
    code.extend(&[0xA8, 0x0B, 0x80, 0xD2]); // 4 bytes
    
    // svc #0
    code.extend(&[0x01, 0x00, 0x00, 0xD4]); // 4 bytes
    
    // Message after code
    let msg_start = code.len();
    code.extend(message);
    
    // Fix up ADR instruction
    let adr_offset = (msg_start - adr_pos) as i32;
    let immlo = (adr_offset & 0x3) << 29;
    let immhi = ((adr_offset >> 2) & 0x7FFFF) << 5;
    let adr_instr = 0x10000001u32 | (immlo as u32) | (immhi as u32);
    code[adr_pos..adr_pos + 4].copy_from_slice(&adr_instr.to_le_bytes());
    
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
    elf.extend(&EM_AARCH64.to_le_bytes());
    elf.extend(&1u32.to_le_bytes()); // e_version
    elf.extend(&entry_point.to_le_bytes()); // e_entry
    elf.extend(&(ehdr_size as u64).to_le_bytes()); // e_phoff
    elf.extend(&0u64.to_le_bytes()); // e_shoff
    elf.extend(&0u32.to_le_bytes()); // e_flags
    elf.extend(&(ehdr_size as u16).to_le_bytes()); // e_ehsize
    elf.extend(&(phdr_size as u16).to_le_bytes()); // e_phentsize
    elf.extend(&1u16.to_le_bytes()); // e_phnum
    elf.extend(&0u16.to_le_bytes()); // e_shentsize
    elf.extend(&0u16.to_le_bytes()); // e_shnum
    elf.extend(&0u16.to_le_bytes()); // e_shstrndx
    
    // Program Header
    elf.extend(&PT_LOAD.to_le_bytes()); // p_type
    elf.extend(&(PF_R | PF_X).to_le_bytes()); // p_flags
    elf.extend(&0u64.to_le_bytes()); // p_offset
    elf.extend(&BASE_ADDR.to_le_bytes()); // p_vaddr
    elf.extend(&BASE_ADDR.to_le_bytes()); // p_paddr
    elf.extend(&file_size.to_le_bytes()); // p_filesz
    elf.extend(&file_size.to_le_bytes()); // p_memsz
    elf.extend(&0x1000u64.to_le_bytes()); // p_align
    
    // Code + Data
    elf.extend(&code);
    
    elf
}

/// Generate ARM64 ELF from TAYNI graph
pub fn generate_elf_arm64_from_graph(graph: &Graph) -> Vec<u8> {
    let mut strings: Vec<(String, Vec<u8>)> = Vec::new();
    let mut constants: HashMap<String, i64> = HashMap::new();
    
    // Collect strings and constants
    for node in &graph.nodes {
        match node {
            Node::Literal { id, value: Value::String(s) } => {
                strings.push((id.clone(), s.as_bytes().to_vec()));
            }
            Node::Literal { id, value: Value::Int(n) } => {
                constants.insert(id.clone(), *n);
            }
            _ => {}
        }
    }
    
    // Find print operations
    let mut print_ops: Vec<(String, i64)> = Vec::new();
    for node in &graph.nodes {
        if let Node::Operation { op: Op::Prt, args, .. } = node {
            if let Some(Arg::Ref(str_ref)) = args.first() {
                if let Some((_, data)) = strings.iter().find(|(id, _)| id == str_ref) {
                    print_ops.push((str_ref.clone(), data.len() as i64));
                }
            }
        }
    }
    
    if print_ops.is_empty() && !strings.is_empty() {
        let (id, data) = &strings[0];
        return generate_tiny_elf_arm64_with_message(data);
    }
    
    if let Some((str_ref, _)) = print_ops.first() {
        if let Some((_, data)) = strings.iter().find(|(id, _)| id == str_ref) {
            return generate_tiny_elf_arm64_with_message(data);
        }
    }
    
    generate_hello_elf_arm64()
}
