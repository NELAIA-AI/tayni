//! PE Headers Generation
//! DOS Header, COFF Header, Optional Header, Section Headers

use super::constants::*;

/// Create DOS header with PE signature pointer
pub fn create_dos_header(pe_offset: u32) -> Vec<u8> {
    let mut header = vec![0u8; DOS_HEADER_SIZE];
    
    // DOS signature "MZ"
    header[0] = 0x4D; // 'M'
    header[1] = 0x5A; // 'Z'
    
    // Bytes on last page of file
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
    
    // Minimum extra paragraphs needed
    header[10] = 0x00;
    header[11] = 0x00;
    
    // Maximum extra paragraphs needed
    header[12] = 0xFF;
    header[13] = 0xFF;
    
    // Initial (relative) SS value
    header[14] = 0x00;
    header[15] = 0x00;
    
    // Initial SP value
    header[16] = 0xB8;
    header[17] = 0x00;
    
    // Checksum
    header[18] = 0x00;
    header[19] = 0x00;
    
    // Initial IP value
    header[20] = 0x00;
    header[21] = 0x00;
    
    // Initial (relative) CS value
    header[22] = 0x00;
    header[23] = 0x00;
    
    // File address of relocation table
    header[24] = 0x40;
    header[25] = 0x00;
    
    // Overlay number
    header[26] = 0x00;
    header[27] = 0x00;
    
    // Reserved words (8 bytes)
    // header[28..36] = 0
    
    // OEM identifier
    header[36] = 0x00;
    header[37] = 0x00;
    
    // OEM information
    header[38] = 0x00;
    header[39] = 0x00;
    
    // Reserved words (20 bytes)
    // header[40..60] = 0
    
    // File address of new exe header (PE offset)
    header[60] = (pe_offset & 0xFF) as u8;
    header[61] = ((pe_offset >> 8) & 0xFF) as u8;
    header[62] = ((pe_offset >> 16) & 0xFF) as u8;
    header[63] = ((pe_offset >> 24) & 0xFF) as u8;
    
    header
}

/// Create COFF header
pub fn create_coff_header(num_sections: u16, timestamp: u32) -> Vec<u8> {
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

/// Create PE32+ Optional Header
pub fn create_optional_header(
    code_size: u32,
    data_size: u32,
    entry_point: u32,
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
    
    // Address of entry point
    header.extend_from_slice(&entry_point.to_le_bytes());
    
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
    
    // Checksum
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Subsystem (Console)
    header.extend_from_slice(&IMAGE_SUBSYSTEM_WINDOWS_CUI.to_le_bytes());
    
    // DLL characteristics
    header.extend_from_slice(&0x8160u16.to_le_bytes()); // DYNAMIC_BASE | NX_COMPAT | TERMINAL_SERVER_AWARE
    
    // Size of stack reserve (64-bit)
    header.extend_from_slice(&0x100000u64.to_le_bytes());
    
    // Size of stack commit (64-bit)
    header.extend_from_slice(&0x1000u64.to_le_bytes());
    
    // Size of heap reserve (64-bit)
    header.extend_from_slice(&0x100000u64.to_le_bytes());
    
    // Size of heap commit (64-bit)
    header.extend_from_slice(&0x1000u64.to_le_bytes());
    
    // Loader flags (reserved)
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Number of data directories
    header.extend_from_slice(&16u32.to_le_bytes());
    
    // Data directories (16 entries, 8 bytes each = 128 bytes)
    // All zeros except import directory which will be filled later
    for _ in 0..16 {
        header.extend_from_slice(&0u32.to_le_bytes()); // RVA
        header.extend_from_slice(&0u32.to_le_bytes()); // Size
    }
    
    header
}

/// Create section header
pub fn create_section_header(
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
    
    // Virtual address
    header.extend_from_slice(&virtual_address.to_le_bytes());
    
    // Size of raw data
    header.extend_from_slice(&raw_size.to_le_bytes());
    
    // Pointer to raw data
    header.extend_from_slice(&raw_offset.to_le_bytes());
    
    // Pointer to relocations
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Pointer to line numbers
    header.extend_from_slice(&0u32.to_le_bytes());
    
    // Number of relocations
    header.extend_from_slice(&0u16.to_le_bytes());
    
    // Number of line numbers
    header.extend_from_slice(&0u16.to_le_bytes());
    
    // Characteristics
    header.extend_from_slice(&characteristics.to_le_bytes());
    
    header
}
