//! PE Format Constants
//! Windows PE32+ (64-bit) format constants

// PE Constants
pub const DOS_HEADER_SIZE: usize = 64;
pub const PE_SIGNATURE: &[u8; 4] = b"PE\0\0";
pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
pub const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
pub const IMAGE_FILE_LARGE_ADDRESS_AWARE: u16 = 0x0020;

// Optional Header Magic
pub const PE32_PLUS_MAGIC: u16 = 0x020B;

// Subsystem
pub const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3; // Console application

// Section characteristics
pub const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;

// Alignment
pub const FILE_ALIGNMENT: u32 = 0x200;  // 512 bytes
pub const SECTION_ALIGNMENT: u32 = 0x1000; // 4KB

// Image base
pub const IMAGE_BASE: u64 = 0x140000000;

// Windows API constants
pub const STD_OUTPUT_HANDLE: i32 = -11;
pub const MB_OK: u32 = 0;

// Socket constants
pub const AF_INET: i32 = 2;
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;

// File mode constants
pub const GENERIC_READ: u32 = 0x80000000;
pub const GENERIC_WRITE: u32 = 0x40000000;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;

// Memory allocation constants
pub const MEM_COMMIT: u32 = 0x1000;
pub const MEM_RESERVE: u32 = 0x2000;
pub const PAGE_READWRITE: u32 = 0x04;
pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;

// Thread constants
pub const INFINITE: u32 = 0xFFFFFFFF;

// Windows epoch offset (100-nanosecond intervals from 1601 to 1970)
pub const WINDOWS_EPOCH_OFFSET: u64 = 0x019DB1DED53E8000;
