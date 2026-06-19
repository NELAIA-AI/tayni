//! PE Template Generator v2
//! Generates a complete PE template with kernel32 imports

use std::fs::File;
use std::io::Write;

fn main() {
    // Total size: 0x400 (1024) for headers + 0x200 (512) for .text + 0x200 for .idata
    let mut pe = vec![0u8; 0x600]; // 1536 bytes
    
    // ========================================
    // DOS Header (64 bytes) at 0x00
    // ========================================
    pe[0x00] = 0x4D; // 'M'
    pe[0x01] = 0x5A; // 'Z'
    pe[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes()); // e_lfanew
    
    // ========================================
    // PE Signature at 0x80
    // ========================================
    pe[0x80..0x84].copy_from_slice(b"PE\0\0");
    
    // ========================================
    // COFF Header at 0x84 (20 bytes)
    // ========================================
    pe[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes()); // Machine: AMD64
    pe[0x86..0x88].copy_from_slice(&2u16.to_le_bytes()); // NumberOfSections: 2
    pe[0x94..0x96].copy_from_slice(&0xF0u16.to_le_bytes()); // SizeOfOptionalHeader
    pe[0x96..0x98].copy_from_slice(&0x22u16.to_le_bytes()); // Characteristics
    
    // ========================================
    // Optional Header PE32+ at 0x98
    // ========================================
    pe[0x98..0x9A].copy_from_slice(&0x020Bu16.to_le_bytes()); // Magic: PE32+
    pe[0x9A] = 0x01; // MajorLinkerVersion
    
    // SizeOfCode
    pe[0x9C..0xA0].copy_from_slice(&0x200u32.to_le_bytes());
    
    // SizeOfInitializedData
    pe[0xA0..0xA4].copy_from_slice(&0x200u32.to_le_bytes());
    
    // AddressOfEntryPoint: 0x1000
    pe[0xA8..0xAC].copy_from_slice(&0x1000u32.to_le_bytes());
    
    // BaseOfCode: 0x1000
    pe[0xAC..0xB0].copy_from_slice(&0x1000u32.to_le_bytes());
    
    // ImageBase: 0x140000000
    pe[0xB0..0xB8].copy_from_slice(&0x140000000u64.to_le_bytes());
    
    // SectionAlignment: 0x1000
    pe[0xB8..0xBC].copy_from_slice(&0x1000u32.to_le_bytes());
    
    // FileAlignment: 0x200
    pe[0xBC..0xC0].copy_from_slice(&0x200u32.to_le_bytes());
    
    // MajorOperatingSystemVersion: 6
    pe[0xC0..0xC2].copy_from_slice(&6u16.to_le_bytes());
    
    // MajorSubsystemVersion: 6
    pe[0xC8..0xCA].copy_from_slice(&6u16.to_le_bytes());
    
    // SizeOfImage: 0x4000
    pe[0xD0..0xD4].copy_from_slice(&0x4000u32.to_le_bytes());
    
    // SizeOfHeaders: 0x200
    pe[0xD4..0xD8].copy_from_slice(&0x200u32.to_le_bytes());
    
    // Subsystem: CONSOLE (3)
    pe[0xDC..0xDE].copy_from_slice(&3u16.to_le_bytes());
    
    // DllCharacteristics
    pe[0xDE..0xE0].copy_from_slice(&0x0160u16.to_le_bytes());
    
    // SizeOfStackReserve: 0x100000
    pe[0xE0..0xE8].copy_from_slice(&0x100000u64.to_le_bytes());
    
    // SizeOfStackCommit: 0x1000
    pe[0xE8..0xF0].copy_from_slice(&0x1000u64.to_le_bytes());
    
    // SizeOfHeapReserve: 0x100000
    pe[0xF0..0xF8].copy_from_slice(&0x100000u64.to_le_bytes());
    
    // SizeOfHeapCommit: 0x1000
    pe[0xF8..0x100].copy_from_slice(&0x1000u64.to_le_bytes());
    
    // NumberOfRvaAndSizes: 16
    pe[0x104..0x108].copy_from_slice(&16u32.to_le_bytes());
    
    // ========================================
    // Data Directories at 0x108 (16 * 8 = 128 bytes)
    // ========================================
    // [1] Import Directory: RVA=0x2000, Size=0x28
    pe[0x110..0x114].copy_from_slice(&0x2000u32.to_le_bytes()); // RVA
    pe[0x114..0x118].copy_from_slice(&0x28u32.to_le_bytes()); // Size
    
    // [12] IAT: RVA=0x2080, Size=0x20
    pe[0x168..0x16C].copy_from_slice(&0x2080u32.to_le_bytes()); // RVA
    pe[0x16C..0x170].copy_from_slice(&0x20u32.to_le_bytes()); // Size
    
    // ========================================
    // Section Header .text at 0x188 (40 bytes)
    // ========================================
    pe[0x188..0x190].copy_from_slice(b".text\0\0\0");
    pe[0x190..0x194].copy_from_slice(&0x200u32.to_le_bytes()); // VirtualSize
    pe[0x194..0x198].copy_from_slice(&0x1000u32.to_le_bytes()); // VirtualAddress
    pe[0x198..0x19C].copy_from_slice(&0x200u32.to_le_bytes()); // SizeOfRawData
    pe[0x19C..0x1A0].copy_from_slice(&0x200u32.to_le_bytes()); // PointerToRawData
    pe[0x1AC..0x1B0].copy_from_slice(&0x60000020u32.to_le_bytes()); // Characteristics
    
    // ========================================
    // Section Header .idata at 0x1B0 (40 bytes)
    // ========================================
    pe[0x1B0..0x1B8].copy_from_slice(b".idata\0\0");
    pe[0x1B8..0x1BC].copy_from_slice(&0x200u32.to_le_bytes()); // VirtualSize
    pe[0x1BC..0x1C0].copy_from_slice(&0x2000u32.to_le_bytes()); // VirtualAddress
    pe[0x1C0..0x1C4].copy_from_slice(&0x200u32.to_le_bytes()); // SizeOfRawData
    pe[0x1C4..0x1C8].copy_from_slice(&0x400u32.to_le_bytes()); // PointerToRawData
    pe[0x1D4..0x1D8].copy_from_slice(&0xC0000040u32.to_le_bytes()); // Characteristics
    
    // ========================================
    // .text section at 0x200 (placeholder for code)
    // ========================================
    // Will be filled by TAYNI
    
    // ========================================
    // .idata section at 0x400
    // Import Directory Table at RVA 0x2000 (file offset 0x400)
    // ========================================
    let idata_base = 0x400;
    
    // Import Directory Entry for kernel32.dll (20 bytes)
    // OriginalFirstThunk (ILT): RVA 0x2050
    pe[idata_base..idata_base+4].copy_from_slice(&0x2050u32.to_le_bytes());
    // TimeDateStamp: 0
    // ForwarderChain: 0
    // Name: RVA 0x20A0 (kernel32.dll string)
    pe[idata_base+12..idata_base+16].copy_from_slice(&0x20A0u32.to_le_bytes());
    // FirstThunk (IAT): RVA 0x2080
    pe[idata_base+16..idata_base+20].copy_from_slice(&0x2080u32.to_le_bytes());
    
    // Null terminator entry (20 bytes of zeros) at 0x414
    
    // Import Lookup Table (ILT) at RVA 0x2050 (file offset 0x450)
    let ilt_base = 0x450;
    // Entry for ExitProcess: RVA 0x20B0 (hint/name)
    pe[ilt_base..ilt_base+8].copy_from_slice(&0x20B0u64.to_le_bytes());
    // Entry for GetStdHandle: RVA 0x20C0
    pe[ilt_base+8..ilt_base+16].copy_from_slice(&0x20C0u64.to_le_bytes());
    // Entry for WriteFile: RVA 0x20D0
    pe[ilt_base+16..ilt_base+24].copy_from_slice(&0x20D0u64.to_le_bytes());
    // Null terminator
    
    // Import Address Table (IAT) at RVA 0x2080 (file offset 0x480)
    let iat_base = 0x480;
    // Same as ILT initially
    pe[iat_base..iat_base+8].copy_from_slice(&0x20B0u64.to_le_bytes());
    pe[iat_base+8..iat_base+16].copy_from_slice(&0x20C0u64.to_le_bytes());
    pe[iat_base+16..iat_base+24].copy_from_slice(&0x20D0u64.to_le_bytes());
    
    // DLL name "kernel32.dll" at RVA 0x20A0 (file offset 0x4A0)
    let name_base = 0x4A0;
    pe[name_base..name_base+13].copy_from_slice(b"kernel32.dll\0");
    
    // Hint/Name entries
    // ExitProcess at RVA 0x20B0 (file offset 0x4B0)
    pe[0x4B0..0x4B2].copy_from_slice(&0u16.to_le_bytes()); // Hint
    pe[0x4B2..0x4BE].copy_from_slice(b"ExitProcess\0");
    
    // GetStdHandle at RVA 0x20C0 (file offset 0x4C0)
    pe[0x4C0..0x4C2].copy_from_slice(&0u16.to_le_bytes()); // Hint
    pe[0x4C2..0x4CF].copy_from_slice(b"GetStdHandle\0");
    
    // WriteFile at RVA 0x20D0 (file offset 0x4D0)
    pe[0x4D0..0x4D2].copy_from_slice(&0u16.to_le_bytes()); // Hint
    pe[0x4D2..0x4DC].copy_from_slice(b"WriteFile\0");
    
    // Write to file
    let mut file = File::create("pe-template.bin").expect("Cannot create file");
    file.write_all(&pe).expect("Cannot write file");
    
    println!("Generated pe-template.bin ({} bytes)", pe.len());
    println!("  .text at file offset 0x200, RVA 0x1000");
    println!("  .idata at file offset 0x400, RVA 0x2000");
    println!("  IAT at RVA 0x2080:");
    println!("    [0] ExitProcess");
    println!("    [1] GetStdHandle");
    println!("    [2] WriteFile");
}
