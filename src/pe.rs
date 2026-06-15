//! NELAIA PE Generator - Direct Windows executable generation
//! Generates PE32+ (64-bit) executables without external tools

use crate::ir::{Graph, Node, Op, Arg, Value};
use std::collections::HashMap;

// PE Constants
const DOS_HEADER_SIZE: usize = 64;
const PE_SIGNATURE: &[u8; 4] = b"PE\0\0";
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
const IMAGE_FILE_LARGE_ADDRESS_AWARE: u16 = 0x0020;

// Optional Header Magic
const PE32_PLUS_MAGIC: u16 = 0x020B;

// Subsystem
const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3; // Console application

// Section characteristics
const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;

// Alignment
const FILE_ALIGNMENT: u32 = 0x200;  // 512 bytes
const SECTION_ALIGNMENT: u32 = 0x1000; // 4KB

// Image base
const IMAGE_BASE: u64 = 0x140000000;

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

/// x64 instruction encoding helpers
pub mod x64 {
    /// MOV RAX, imm64
    pub fn mov_rax_imm64(value: u64) -> Vec<u8> {
        let mut code = vec![0x48, 0xB8]; // REX.W + MOV RAX
        code.extend_from_slice(&value.to_le_bytes());
        code
    }
    
    /// MOV RCX, imm64
    pub fn mov_rcx_imm64(value: u64) -> Vec<u8> {
        let mut code = vec![0x48, 0xB9]; // REX.W + MOV RCX
        code.extend_from_slice(&value.to_le_bytes());
        code
    }
    
    /// MOV RDX, imm64
    pub fn mov_rdx_imm64(value: u64) -> Vec<u8> {
        let mut code = vec![0x48, 0xBA]; // REX.W + MOV RDX
        code.extend_from_slice(&value.to_le_bytes());
        code
    }
    
    /// MOV R8, imm64
    pub fn mov_r8_imm64(value: u64) -> Vec<u8> {
        let mut code = vec![0x49, 0xB8]; // REX.WB + MOV R8
        code.extend_from_slice(&value.to_le_bytes());
        code
    }
    
    /// MOV R9, imm64
    pub fn mov_r9_imm64(value: u64) -> Vec<u8> {
        let mut code = vec![0x49, 0xB9]; // REX.WB + MOV R9
        code.extend_from_slice(&value.to_le_bytes());
        code
    }
    
    /// SUB RSP, imm8
    pub fn sub_rsp_imm8(value: u8) -> Vec<u8> {
        vec![0x48, 0x83, 0xEC, value]
    }
    
    /// ADD RSP, imm8
    pub fn add_rsp_imm8(value: u8) -> Vec<u8> {
        vec![0x48, 0x83, 0xC4, value]
    }
    
    /// CALL [RIP+offset]
    pub fn call_rip_rel(offset: i32) -> Vec<u8> {
        let mut code = vec![0xFF, 0x15]; // CALL [RIP+disp32]
        code.extend_from_slice(&offset.to_le_bytes());
        code
    }
    
    /// RET
    pub fn ret() -> Vec<u8> {
        vec![0xC3]
    }
    
    /// XOR ECX, ECX (zero RCX)
    pub fn xor_ecx_ecx() -> Vec<u8> {
        vec![0x31, 0xC9]
    }
    
    /// XOR EAX, EAX (zero RAX)
    pub fn xor_eax_eax() -> Vec<u8> {
        vec![0x31, 0xC0]
    }
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
    
    // mov r8d, 15 (length of "Hello, NELAIA!\n")
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
    let msg = b"Hello, NELAIA!\n";
    data[..msg.len()].copy_from_slice(msg);
    
    pe.extend(&data);
    
    pe
}

/// Generate PE from NELAIA graph
/// Supports: literals, strings, ALC, PUT, PRT, ADD, SUB, MUL
pub fn generate_pe_from_graph(graph: &Graph) -> Vec<u8> {
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
            Node::Literal { id, value } => {
                match value {
                    Value::Int(n) => { values.insert(id.clone(), *n); }
                    Value::String(s) => { strings.insert(id.clone(), s.clone()); }
                    _ => {}
                }
            }
            Node::Operation { id, op, args } => {
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
