//! PE Template Test
//! Generates a complete working exe using the template approach

use std::fs::File;
use std::io::Write;

fn main() {
    // Read template
    let template = std::fs::read("pe-template.bin").expect("Cannot read template");
    let mut pe = template.clone();
    pe.resize(0x600, 0); // Ensure size
    
    // Write x86-64 code at offset 0x200
    // This code will:
    // 1. Call GetStdHandle(-11)
    // 2. Call WriteFile to print "Hi\n"
    // 3. Call ExitProcess(0)
    
    let code_base = 0x200;
    let mut code = Vec::new();
    
    // sub rsp, 56
    code.extend(&[0x48, 0x83, 0xEC, 0x38]);
    
    // mov ecx, -11 (STD_OUTPUT_HANDLE)
    code.extend(&[0xB9, 0xF5, 0xFF, 0xFF, 0xFF]);
    
    // call [rip + offset_to_GetStdHandle]
    // GetStdHandle is at IAT[1] = RVA 0x2088
    // Current RIP after instruction = 0x1000 + 9 + 6 = 0x100F
    // Offset = 0x2088 - 0x100F = 0x1079
    code.extend(&[0xFF, 0x15]);
    let offset1 = 0x2088i32 - (0x1000 + code.len() as i32 + 4);
    code.extend(&offset1.to_le_bytes());
    
    // mov [rsp+32], rax (save handle)
    code.extend(&[0x48, 0x89, 0x44, 0x24, 0x20]);
    
    // mov dword [rsp+40], "Hi\n\0"
    code.extend(&[0xC7, 0x44, 0x24, 0x28, 0x48, 0x69, 0x0A, 0x00]);
    
    // mov rcx, [rsp+32] (handle)
    code.extend(&[0x48, 0x8B, 0x4C, 0x24, 0x20]);
    
    // lea rdx, [rsp+40] (buffer)
    code.extend(&[0x48, 0x8D, 0x54, 0x24, 0x28]);
    
    // mov r8d, 3 (length)
    code.extend(&[0x41, 0xB8, 0x03, 0x00, 0x00, 0x00]);
    
    // lea r9, [rsp+48] (&written)
    code.extend(&[0x4C, 0x8D, 0x4C, 0x24, 0x30]);
    
    // mov qword [rsp+32], 0 (lpOverlapped = NULL, 5th arg)
    code.extend(&[0x48, 0xC7, 0x44, 0x24, 0x20, 0x00, 0x00, 0x00, 0x00]);
    
    // call [rip + offset_to_WriteFile]
    // WriteFile is at IAT[2] = RVA 0x2090
    code.extend(&[0xFF, 0x15]);
    let offset2 = 0x2090i32 - (0x1000 + code.len() as i32 + 4);
    code.extend(&offset2.to_le_bytes());
    
    // xor ecx, ecx (exit code 0)
    code.extend(&[0x31, 0xC9]);
    
    // call [rip + offset_to_ExitProcess]
    // ExitProcess is at IAT[0] = RVA 0x2080
    code.extend(&[0xFF, 0x15]);
    let offset3 = 0x2080i32 - (0x1000 + code.len() as i32 + 4);
    code.extend(&offset3.to_le_bytes());
    
    // Copy code to PE
    for (i, byte) in code.iter().enumerate() {
        pe[code_base + i] = *byte;
    }
    
    // Write exe
    let mut file = File::create("hello-test.exe").expect("Cannot create file");
    file.write_all(&pe).expect("Cannot write file");
    
    println!("Generated hello-test.exe ({} bytes)", pe.len());
    println!("Code size: {} bytes", code.len());
}
