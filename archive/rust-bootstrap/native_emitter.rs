//! TAYNI Native Code Emitter - Windows x64 PE Generator
//! Generates native executables without LLVM

use crate::ir::{Graph, Node, Op, Arg, Value};
use std::collections::HashMap;

/// Generate a minimal Windows x64 PE executable
pub fn emit_native_exe(graph: &Graph) -> Vec<u8> {
    let mut emitter = NativeEmitter::new();
    emitter.emit(graph);
    emitter.build_pe()
}

struct NativeEmitter {
    code: Vec<u8>,
    data: Vec<u8>,
    values: HashMap<String, i64>,
    registers: HashMap<String, u8>, // variable -> register mapping
}

impl NativeEmitter {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
            values: HashMap::new(),
            registers: HashMap::new(),
        }
    }
    
    fn emit(&mut self, graph: &Graph) {
        // First pass: collect literals
        for node in &graph.nodes {
            if let Node::Literal { id, value } = node {
                if let Value::Int(n) = value {
                    self.values.insert(id.clone(), *n);
                }
            }
        }
        
        // Second pass: emit operations
        // We'll use RAX as accumulator
        let mut first_value = true;
        
        for node in &graph.nodes {
            match node {
                Node::Literal { id, value } => {
                    if let Value::Int(n) = value {
                        if first_value {
                            // mov rax, imm64
                            self.emit_mov_imm64(0, *n);
                            first_value = false;
                        }
                        self.values.insert(id.clone(), *n);
                    }
                }
                Node::Operation { id, op, args } => {
                    match op {
                        Op::Add => {
                            if args.len() >= 2 {
                                let val2 = self.get_operand(&args[1]);
                                // add rax, val2
                                self.emit_add_imm32(0, val2 as i32);
                                // Update computed value
                                let val1 = self.get_operand(&args[0]);
                                self.values.insert(id.clone(), val1 + val2);
                            }
                        }
                        Op::Sub => {
                            if args.len() >= 2 {
                                let val2 = self.get_operand(&args[1]);
                                self.emit_sub_imm32(0, val2 as i32);
                                let val1 = self.get_operand(&args[0]);
                                self.values.insert(id.clone(), val1 - val2);
                            }
                        }
                        Op::Mul => {
                            if args.len() >= 2 {
                                let val2 = self.get_operand(&args[1]);
                                // mov rcx, val2
                                self.emit_mov_imm64(1, val2);
                                // imul rax, rcx
                                self.emit_imul_reg(0, 1);
                                let val1 = self.get_operand(&args[0]);
                                self.values.insert(id.clone(), val1 * val2);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Epilogue: call ExitProcess with result in RCX
        // mov rcx, rax
        self.code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
        
        // The call to ExitProcess will be patched in build_pe
        // For now, emit a placeholder
        self.code.extend_from_slice(&[0xFF, 0x15, 0x00, 0x00, 0x00, 0x00]); // call [rip+0] placeholder
    }
    
    fn get_operand(&self, arg: &Arg) -> i64 {
        match arg {
            Arg::Ref(r) => *self.values.get(r).unwrap_or(&0),
            Arg::Lit(Value::Int(n)) => *n,
            _ => 0,
        }
    }
    
    // x64 instruction emitters
    fn emit_mov_imm64(&mut self, reg: u8, imm: i64) {
        // REX.W + B8+rd + imm64
        self.code.push(0x48 | ((reg >> 3) & 1)); // REX.W
        self.code.push(0xB8 + (reg & 7));
        self.code.extend_from_slice(&imm.to_le_bytes());
    }
    
    fn emit_add_imm32(&mut self, reg: u8, imm: i32) {
        // REX.W + 81 /0 + imm32
        self.code.push(0x48);
        self.code.push(0x81);
        self.code.push(0xC0 + (reg & 7)); // ModRM: 11 000 reg
        self.code.extend_from_slice(&imm.to_le_bytes());
    }
    
    fn emit_sub_imm32(&mut self, reg: u8, imm: i32) {
        // REX.W + 81 /5 + imm32
        self.code.push(0x48);
        self.code.push(0x81);
        self.code.push(0xE8 + (reg & 7)); // ModRM: 11 101 reg
        self.code.extend_from_slice(&imm.to_le_bytes());
    }
    
    fn emit_imul_reg(&mut self, dst: u8, src: u8) {
        // REX.W + 0F AF /r
        self.code.push(0x48);
        self.code.push(0x0F);
        self.code.push(0xAF);
        self.code.push(0xC0 + (dst << 3) + src); // ModRM
    }
    
    fn build_pe(&self) -> Vec<u8> {
        let mut pe = Vec::new();
        
        // DOS Header (64 bytes)
        let dos_header: [u8; 64] = [
            0x4D, 0x5A, // MZ signature
            0x90, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
            0xB8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, // e_lfanew = 0x80
        ];
        pe.extend_from_slice(&dos_header);
        
        // Padding to PE header at 0x80
        pe.resize(0x80, 0);
        
        // PE Signature
        pe.extend_from_slice(b"PE\0\0");
        
        // COFF Header (20 bytes)
        pe.extend_from_slice(&0x8664u16.to_le_bytes()); // Machine: AMD64
        pe.extend_from_slice(&2u16.to_le_bytes());      // NumberOfSections
        pe.extend_from_slice(&0u32.to_le_bytes());      // TimeDateStamp
        pe.extend_from_slice(&0u32.to_le_bytes());      // PointerToSymbolTable
        pe.extend_from_slice(&0u32.to_le_bytes());      // NumberOfSymbols
        pe.extend_from_slice(&0xF0u16.to_le_bytes());   // SizeOfOptionalHeader
        pe.extend_from_slice(&0x22u16.to_le_bytes());   // Characteristics: EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE
        
        // Optional Header (PE32+)
        pe.extend_from_slice(&0x20Bu16.to_le_bytes());  // Magic: PE32+
        pe.push(14); // MajorLinkerVersion
        pe.push(0);  // MinorLinkerVersion
        
        let code_size = ((self.code.len() + 0x1FF) & !0x1FF) as u32; // Align to 512
        let data_size = 0x200u32; // Import data
        
        pe.extend_from_slice(&code_size.to_le_bytes());  // SizeOfCode
        pe.extend_from_slice(&data_size.to_le_bytes());  // SizeOfInitializedData
        pe.extend_from_slice(&0u32.to_le_bytes());       // SizeOfUninitializedData
        pe.extend_from_slice(&0x1000u32.to_le_bytes());  // AddressOfEntryPoint
        pe.extend_from_slice(&0x1000u32.to_le_bytes());  // BaseOfCode
        pe.extend_from_slice(&0x140000000u64.to_le_bytes()); // ImageBase
        pe.extend_from_slice(&0x1000u32.to_le_bytes());  // SectionAlignment
        pe.extend_from_slice(&0x200u32.to_le_bytes());   // FileAlignment
        pe.extend_from_slice(&6u16.to_le_bytes());       // MajorOperatingSystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes());       // MinorOperatingSystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes());       // MajorImageVersion
        pe.extend_from_slice(&0u16.to_le_bytes());       // MinorImageVersion
        pe.extend_from_slice(&6u16.to_le_bytes());       // MajorSubsystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes());       // MinorSubsystemVersion
        pe.extend_from_slice(&0u32.to_le_bytes());       // Win32VersionValue
        pe.extend_from_slice(&0x4000u32.to_le_bytes());  // SizeOfImage
        pe.extend_from_slice(&0x200u32.to_le_bytes());   // SizeOfHeaders
        pe.extend_from_slice(&0u32.to_le_bytes());       // CheckSum
        pe.extend_from_slice(&3u16.to_le_bytes());       // Subsystem: CONSOLE
        pe.extend_from_slice(&0x8160u16.to_le_bytes());  // DllCharacteristics: DYNAMIC_BASE | NX_COMPAT | TERMINAL_SERVER_AWARE
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes());   // SizeOfStackCommit
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes());   // SizeOfHeapCommit
        pe.extend_from_slice(&0u32.to_le_bytes());        // LoaderFlags
        pe.extend_from_slice(&16u32.to_le_bytes());       // NumberOfRvaAndSizes
        
        // Data Directories (16 entries, 8 bytes each)
        pe.extend_from_slice(&0u64.to_le_bytes()); // Export
        pe.extend_from_slice(&0x2000u32.to_le_bytes()); // Import RVA
        pe.extend_from_slice(&0x100u32.to_le_bytes());  // Import Size
        for _ in 2..16 {
            pe.extend_from_slice(&0u64.to_le_bytes());
        }
        
        // Section Headers
        // .text section
        pe.extend_from_slice(b".text\0\0\0");           // Name
        pe.extend_from_slice(&(self.code.len() as u32).to_le_bytes()); // VirtualSize
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // VirtualAddress
        pe.extend_from_slice(&code_size.to_le_bytes()); // SizeOfRawData
        pe.extend_from_slice(&0x200u32.to_le_bytes());  // PointerToRawData
        pe.extend_from_slice(&0u32.to_le_bytes());      // PointerToRelocations
        pe.extend_from_slice(&0u32.to_le_bytes());      // PointerToLinenumbers
        pe.extend_from_slice(&0u16.to_le_bytes());      // NumberOfRelocations
        pe.extend_from_slice(&0u16.to_le_bytes());      // NumberOfLinenumbers
        pe.extend_from_slice(&0x60000020u32.to_le_bytes()); // Characteristics: CODE | EXECUTE | READ
        
        // .idata section
        pe.extend_from_slice(b".idata\0\0");            // Name
        pe.extend_from_slice(&data_size.to_le_bytes()); // VirtualSize
        pe.extend_from_slice(&0x2000u32.to_le_bytes()); // VirtualAddress
        pe.extend_from_slice(&data_size.to_le_bytes()); // SizeOfRawData
        pe.extend_from_slice(&(0x200 + code_size).to_le_bytes()); // PointerToRawData
        pe.extend_from_slice(&0u32.to_le_bytes());      // PointerToRelocations
        pe.extend_from_slice(&0u32.to_le_bytes());      // PointerToLinenumbers
        pe.extend_from_slice(&0u16.to_le_bytes());      // NumberOfRelocations
        pe.extend_from_slice(&0u16.to_le_bytes());      // NumberOfLinenumbers
        pe.extend_from_slice(&0xC0000040u32.to_le_bytes()); // Characteristics: INITIALIZED_DATA | READ | WRITE
        
        // Pad to 0x200 (file alignment)
        pe.resize(0x200, 0);
        
        // .text section content
        let mut code = self.code.clone();
        
        // Patch the call to ExitProcess
        // The IAT entry will be at RVA 0x2080
        // call [rip + offset] where offset = 0x2080 - (0x1000 + code.len() - 6 + 6)
        let call_offset = code.len() - 6;
        let iat_rva = 0x2080u32;
        let next_ip = 0x1000 + call_offset as u32 + 6;
        let rel_offset = (iat_rva as i32) - (next_ip as i32);
        code[call_offset + 2..call_offset + 6].copy_from_slice(&rel_offset.to_le_bytes());
        
        pe.extend_from_slice(&code);
        pe.resize(0x200 + code_size as usize, 0);
        
        // .idata section content (Import Directory)
        let idata_start = pe.len();
        
        // Import Directory Table (1 entry + null terminator)
        // Entry for kernel32.dll
        pe.extend_from_slice(&0x2050u32.to_le_bytes()); // OriginalFirstThunk (ILT RVA)
        pe.extend_from_slice(&0u32.to_le_bytes());      // TimeDateStamp
        pe.extend_from_slice(&0u32.to_le_bytes());      // ForwarderChain
        pe.extend_from_slice(&0x2070u32.to_le_bytes()); // Name RVA
        pe.extend_from_slice(&0x2080u32.to_le_bytes()); // FirstThunk (IAT RVA)
        
        // Null terminator
        pe.extend_from_slice(&[0u8; 20]);
        
        // Padding to 0x50
        pe.resize(idata_start + 0x50, 0);
        
        // Import Lookup Table (ILT) at 0x2050
        pe.extend_from_slice(&0x2090u64.to_le_bytes()); // Hint/Name RVA
        pe.extend_from_slice(&0u64.to_le_bytes());      // Null terminator
        
        // Padding to 0x70
        pe.resize(idata_start + 0x70, 0);
        
        // DLL Name at 0x2070
        pe.extend_from_slice(b"KERNEL32.dll\0");
        
        // Padding to 0x80
        pe.resize(idata_start + 0x80, 0);
        
        // Import Address Table (IAT) at 0x2080
        pe.extend_from_slice(&0x2090u64.to_le_bytes()); // Hint/Name RVA
        pe.extend_from_slice(&0u64.to_le_bytes());      // Null terminator
        
        // Hint/Name at 0x2090
        pe.extend_from_slice(&0u16.to_le_bytes());      // Hint
        pe.extend_from_slice(b"ExitProcess\0");
        
        // Pad to section size
        pe.resize(idata_start + data_size as usize, 0);
        
        pe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_add() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { 
            id: "a".to_string(), 
            value: Value::Int(5) 
        });
        graph.add_node(Node::Operation {
            id: "b".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Lit(Value::Int(3))],
        });
        
        let exe = emit_native_exe(&graph);
        assert!(exe.len() > 0);
        assert_eq!(&exe[0..2], b"MZ");
    }
}
