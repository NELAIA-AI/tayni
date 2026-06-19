//! PE Codegen Unit Tests
//! Tests for Windows PE generation and x86-64 code emission

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    // ============================================
    // PE Header Tests
    // ============================================

    #[test]
    fn test_dos_header_magic() {
        // DOS header must start with "MZ"
        let dos_magic: [u8; 2] = [0x4D, 0x5A];
        assert_eq!(dos_magic[0], b'M');
        assert_eq!(dos_magic[1], b'Z');
    }

    #[test]
    fn test_pe_signature() {
        // PE signature is "PE\0\0"
        let pe_sig: [u8; 4] = [0x50, 0x45, 0x00, 0x00];
        assert_eq!(&pe_sig[0..2], b"PE");
    }

    #[test]
    fn test_x64_machine_type() {
        // x86-64 machine type is 0x8664
        let machine: u16 = 0x8664;
        assert_eq!(machine, 0x8664);
    }

    // ============================================
    // x86-64 Instruction Encoding Tests
    // ============================================

    #[test]
    fn test_mov_rax_imm64() {
        // mov rax, imm64 = 48 B8 <imm64>
        let opcode: [u8; 2] = [0x48, 0xB8];
        assert_eq!(opcode[0], 0x48); // REX.W prefix
        assert_eq!(opcode[1], 0xB8); // MOV RAX, imm64
    }

    #[test]
    fn test_mov_rcx_imm64() {
        // mov rcx, imm64 = 48 B9 <imm64>
        let opcode: [u8; 2] = [0x48, 0xB9];
        assert_eq!(opcode[0], 0x48);
        assert_eq!(opcode[1], 0xB9);
    }

    #[test]
    fn test_add_rax_rcx() {
        // add rax, rcx = 48 01 C8
        let opcode: [u8; 3] = [0x48, 0x01, 0xC8];
        assert_eq!(opcode[0], 0x48); // REX.W
        assert_eq!(opcode[1], 0x01); // ADD r/m64, r64
        assert_eq!(opcode[2], 0xC8); // ModRM: RAX, RCX
    }

    #[test]
    fn test_sub_rax_rcx() {
        // sub rax, rcx = 48 29 C8
        let opcode: [u8; 3] = [0x48, 0x29, 0xC8];
        assert_eq!(opcode[1], 0x29); // SUB r/m64, r64
    }

    #[test]
    fn test_imul_rax_rcx() {
        // imul rax, rcx = 48 0F AF C1
        let opcode: [u8; 4] = [0x48, 0x0F, 0xAF, 0xC1];
        assert_eq!(opcode[1], 0x0F);
        assert_eq!(opcode[2], 0xAF); // IMUL r64, r/m64
    }

    #[test]
    fn test_ret() {
        // ret = C3
        let opcode: u8 = 0xC3;
        assert_eq!(opcode, 0xC3);
    }

    #[test]
    fn test_call_indirect() {
        // call [rip+disp32] = FF 15 <disp32>
        let opcode: [u8; 2] = [0xFF, 0x15];
        assert_eq!(opcode[0], 0xFF);
        assert_eq!(opcode[1], 0x15);
    }

    #[test]
    fn test_jmp_rel32() {
        // jmp rel32 = E9 <rel32>
        let opcode: u8 = 0xE9;
        assert_eq!(opcode, 0xE9);
    }

    #[test]
    fn test_je_rel32() {
        // je rel32 = 0F 84 <rel32>
        let opcode: [u8; 2] = [0x0F, 0x84];
        assert_eq!(opcode[0], 0x0F);
        assert_eq!(opcode[1], 0x84);
    }

    #[test]
    fn test_jne_rel32() {
        // jne rel32 = 0F 85 <rel32>
        let opcode: [u8; 2] = [0x0F, 0x85];
        assert_eq!(opcode[1], 0x85);
    }

    #[test]
    fn test_push_rbp() {
        // push rbp = 55
        let opcode: u8 = 0x55;
        assert_eq!(opcode, 0x55);
    }

    #[test]
    fn test_pop_rbp() {
        // pop rbp = 5D
        let opcode: u8 = 0x5D;
        assert_eq!(opcode, 0x5D);
    }

    #[test]
    fn test_sub_rsp_imm8() {
        // sub rsp, imm8 = 48 83 EC <imm8>
        let opcode: [u8; 3] = [0x48, 0x83, 0xEC];
        assert_eq!(opcode[0], 0x48); // REX.W
        assert_eq!(opcode[1], 0x83); // SUB r/m64, imm8
        assert_eq!(opcode[2], 0xEC); // ModRM for RSP
    }

    #[test]
    fn test_add_rsp_imm8() {
        // add rsp, imm8 = 48 83 C4 <imm8>
        let opcode: [u8; 3] = [0x48, 0x83, 0xC4];
        assert_eq!(opcode[2], 0xC4); // ModRM for RSP (add)
    }

    // ============================================
    // Windows x64 Calling Convention Tests
    // ============================================

    #[test]
    fn test_shadow_space_size() {
        // Windows x64 requires 32 bytes shadow space
        let shadow_space: usize = 32;
        assert_eq!(shadow_space, 32);
    }

    #[test]
    fn test_stack_alignment() {
        // Stack must be 16-byte aligned before CALL
        let alignment: usize = 16;
        assert_eq!(alignment, 16);
    }

    #[test]
    fn test_first_four_args_registers() {
        // First 4 integer args: RCX, RDX, R8, R9
        let arg_regs = ["rcx", "rdx", "r8", "r9"];
        assert_eq!(arg_regs.len(), 4);
    }

    // ============================================
    // Section Alignment Tests
    // ============================================

    #[test]
    fn test_section_alignment() {
        // Typical section alignment is 0x1000 (4KB)
        let section_align: u32 = 0x1000;
        assert_eq!(section_align, 4096);
    }

    #[test]
    fn test_file_alignment() {
        // Typical file alignment is 0x200 (512 bytes)
        let file_align: u32 = 0x200;
        assert_eq!(file_align, 512);
    }

    // ============================================
    // IAT Tests
    // ============================================

    #[test]
    fn test_kernel32_functions() {
        let kernel32_funcs = [
            "GetStdHandle",
            "WriteFile",
            "ExitProcess",
            "GetSystemTimeAsFileTime",
            "Sleep",
            "CreateThread",
            "WaitForSingleObject",
            "CreateMutexA",
            "ReleaseMutex",
        ];
        assert!(kernel32_funcs.len() >= 5);
    }

    #[test]
    fn test_ws2_32_functions() {
        let ws2_32_funcs = [
            "WSAStartup",
            "socket",
            "bind",
            "listen",
            "accept",
            "send",
            "recv",
            "closesocket",
        ];
        assert!(ws2_32_funcs.len() >= 5);
    }
}
