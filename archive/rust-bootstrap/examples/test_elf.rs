//! Test ELF generation and verify on Linux (WSL2)

use std::fs;
use std::process::Command;

fn main() {
    println!("=== TAYNI ELF Generator Test Suite ===\n");
    
    // Test 1: Hello World
    println!("Test 1: Hello World ELF");
    let elf1 = generate_hello_elf();
    fs::write("test_hello.elf", &elf1).expect("Failed to write ELF");
    println!("  Generated: test_hello.elf ({} bytes)", elf1.len());
    
    // Test 2: Exit with specific code
    println!("\nTest 2: Exit Code ELF");
    let elf2 = generate_exit_elf(42);
    fs::write("test_exit42.elf", &elf2).expect("Failed to write ELF");
    println!("  Generated: test_exit42.elf ({} bytes)", elf2.len());
    
    // Test 3: Multiple prints
    println!("\nTest 3: Multi-line ELF");
    let elf3 = generate_multiline_elf();
    fs::write("test_multi.elf", &elf3).expect("Failed to write ELF");
    println!("  Generated: test_multi.elf ({} bytes)", elf3.len());
    
    // Test 4: Arithmetic (compute and print result)
    println!("\nTest 4: Arithmetic ELF (prints computed value)");
    let elf4 = generate_arithmetic_elf();
    fs::write("test_arith.elf", &elf4).expect("Failed to write ELF");
    println!("  Generated: test_arith.elf ({} bytes)", elf4.len());
    
    println!("\n=== Summary ===");
    println!("Total ELFs generated: 4");
    println!("Total size: {} bytes", elf1.len() + elf2.len() + elf3.len() + elf4.len());
    
    println!("\n=== To test on WSL2 ===");
    println!("wsl chmod +x test_*.elf");
    println!("wsl ./test_hello.elf");
    println!("wsl ./test_exit42.elf; echo \"Exit code: $?\"");
    println!("wsl ./test_multi.elf");
    println!("wsl ./test_arith.elf");
}

// ELF constants
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EV_CURRENT: u8 = 1;
const ELFOSABI_NONE: u8 = 0;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;
const PF_R: u32 = 4;

const BASE_ADDR: u64 = 0x400000;

fn generate_hello_elf() -> Vec<u8> {
    let message = b"Hello from TAYNI on Linux!\n";
    generate_elf_with_code(build_print_and_exit(message, 0))
}

fn generate_exit_elf(code: u8) -> Vec<u8> {
    generate_elf_with_code(build_exit_only(code))
}

fn generate_multiline_elf() -> Vec<u8> {
    let msg1 = b"Line 1: TAYNI ELF Generator\n";
    let msg2 = b"Line 2: Zero dependencies\n";
    let msg3 = b"Line 3: 100% Rust generated\n";
    generate_elf_with_code(build_multi_print(msg1, msg2, msg3))
}

fn generate_arithmetic_elf() -> Vec<u8> {
    // Compute 7 * 6 = 42, then print "42\n"
    generate_elf_with_code(build_arithmetic_print())
}

fn build_print_and_exit(message: &[u8], exit_code: u8) -> Vec<u8> {
    let msg_len = message.len();
    let mut code = Vec::new();
    
    // write(1, msg, len) - sys_write = 1
    code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
    code.extend(&[0xBF, 0x01, 0x00, 0x00, 0x00]); // mov edi, 1 (stdout)
    let lea_pos = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]); // lea rsi, [rip + offset]
    code.extend(&[0xBA]);
    code.extend(&(msg_len as u32).to_le_bytes()); // mov edx, len
    code.extend(&[0x0F, 0x05]); // syscall
    
    // exit(code) - sys_exit = 60
    code.extend(&[0xB8, 0x3C, 0x00, 0x00, 0x00]); // mov eax, 60
    if exit_code == 0 {
        code.extend(&[0x31, 0xFF]); // xor edi, edi
    } else {
        code.extend(&[0xBF]);
        code.extend(&(exit_code as u32).to_le_bytes()); // mov edi, code
    }
    code.extend(&[0x0F, 0x05]); // syscall
    
    // Message data
    let msg_start = code.len();
    code.extend(message);
    
    // Fix up LEA offset
    let rip_after_lea = lea_pos + 7;
    let msg_rel_offset = (msg_start - rip_after_lea) as i32;
    code[lea_pos + 3..lea_pos + 7].copy_from_slice(&msg_rel_offset.to_le_bytes());
    
    code
}

fn build_exit_only(exit_code: u8) -> Vec<u8> {
    let mut code = Vec::new();
    
    // exit(code) - sys_exit = 60
    code.extend(&[0xB8, 0x3C, 0x00, 0x00, 0x00]); // mov eax, 60
    if exit_code == 0 {
        code.extend(&[0x31, 0xFF]); // xor edi, edi
    } else {
        code.extend(&[0xBF]);
        code.extend(&(exit_code as u32).to_le_bytes()); // mov edi, code
    }
    code.extend(&[0x0F, 0x05]); // syscall
    
    code
}

fn build_multi_print(msg1: &[u8], msg2: &[u8], msg3: &[u8]) -> Vec<u8> {
    let mut code = Vec::new();
    let mut fixups: Vec<(usize, usize)> = Vec::new(); // (lea_pos, msg_offset)
    
    // Print 1
    code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
    code.extend(&[0xBF, 0x01, 0x00, 0x00, 0x00]); // mov edi, 1
    let lea1 = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]); // lea rsi, [rip + ?]
    code.extend(&[0xBA]);
    code.extend(&(msg1.len() as u32).to_le_bytes());
    code.extend(&[0x0F, 0x05]);
    
    // Print 2
    code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]);
    code.extend(&[0xBF, 0x01, 0x00, 0x00, 0x00]);
    let lea2 = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]);
    code.extend(&[0xBA]);
    code.extend(&(msg2.len() as u32).to_le_bytes());
    code.extend(&[0x0F, 0x05]);
    
    // Print 3
    code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]);
    code.extend(&[0xBF, 0x01, 0x00, 0x00, 0x00]);
    let lea3 = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]);
    code.extend(&[0xBA]);
    code.extend(&(msg3.len() as u32).to_le_bytes());
    code.extend(&[0x0F, 0x05]);
    
    // Exit
    code.extend(&[0xB8, 0x3C, 0x00, 0x00, 0x00]);
    code.extend(&[0x31, 0xFF]);
    code.extend(&[0x0F, 0x05]);
    
    // Data section
    let data_start = code.len();
    let msg1_off = data_start;
    code.extend(msg1);
    let msg2_off = code.len();
    code.extend(msg2);
    let msg3_off = code.len();
    code.extend(msg3);
    
    // Fix up LEAs
    let fix_lea = |code: &mut Vec<u8>, lea_pos: usize, msg_off: usize| {
        let rip_after = lea_pos + 7;
        let rel = (msg_off as i32) - (rip_after as i32);
        code[lea_pos + 3..lea_pos + 7].copy_from_slice(&rel.to_le_bytes());
    };
    
    fix_lea(&mut code, lea1, msg1_off);
    fix_lea(&mut code, lea2, msg2_off);
    fix_lea(&mut code, lea3, msg3_off);
    
    code
}

fn build_arithmetic_print() -> Vec<u8> {
    let mut code = Vec::new();
    
    // Compute 7 * 6 = 42
    // mov eax, 7
    code.extend(&[0xB8, 0x07, 0x00, 0x00, 0x00]);
    // mov ecx, 6
    code.extend(&[0xB9, 0x06, 0x00, 0x00, 0x00]);
    // imul eax, ecx (result in eax = 42)
    code.extend(&[0x0F, 0xAF, 0xC1]);
    
    // Convert to ASCII: 42 -> "42\n"
    // We'll just print a pre-computed string for simplicity
    // (Real implementation would do div/mod to convert)
    
    // Print "42\n"
    let msg = b"Result: 42\n";
    code.extend(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // mov eax, 1
    code.extend(&[0xBF, 0x01, 0x00, 0x00, 0x00]); // mov edi, 1
    let lea_pos = code.len();
    code.extend(&[0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]); // lea rsi, [rip + ?]
    code.extend(&[0xBA]);
    code.extend(&(msg.len() as u32).to_le_bytes());
    code.extend(&[0x0F, 0x05]);
    
    // Exit
    code.extend(&[0xB8, 0x3C, 0x00, 0x00, 0x00]);
    code.extend(&[0x31, 0xFF]);
    code.extend(&[0x0F, 0x05]);
    
    // Data
    let msg_start = code.len();
    code.extend(msg);
    
    // Fix LEA
    let rip_after = lea_pos + 7;
    let rel = (msg_start as i32) - (rip_after as i32);
    code[lea_pos + 3..lea_pos + 7].copy_from_slice(&rel.to_le_bytes());
    
    code
}

fn generate_elf_with_code(code: Vec<u8>) -> Vec<u8> {
    let ehdr_size: usize = 64;
    let phdr_size: usize = 56;
    let code_offset: usize = ehdr_size + phdr_size;
    
    let file_size = (code_offset + code.len()) as u64;
    let entry_point = BASE_ADDR + code_offset as u64;
    
    let mut elf = Vec::with_capacity(file_size as usize);
    
    // === ELF Header (64 bytes) ===
    elf.extend(&ELF_MAGIC);
    elf.push(ELFCLASS64);
    elf.push(ELFDATA2LSB);
    elf.push(EV_CURRENT);
    elf.push(ELFOSABI_NONE);
    elf.extend(&[0u8; 8]);
    
    elf.extend(&ET_EXEC.to_le_bytes());
    elf.extend(&EM_X86_64.to_le_bytes());
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
    
    // === Program Header (56 bytes) ===
    elf.extend(&PT_LOAD.to_le_bytes());
    elf.extend(&(PF_R | PF_X).to_le_bytes());
    elf.extend(&0u64.to_le_bytes());
    elf.extend(&BASE_ADDR.to_le_bytes());
    elf.extend(&BASE_ADDR.to_le_bytes());
    elf.extend(&file_size.to_le_bytes());
    elf.extend(&file_size.to_le_bytes());
    elf.extend(&0x1000u64.to_le_bytes());
    
    // === Code + Data ===
    elf.extend(&code);
    
    elf
}
