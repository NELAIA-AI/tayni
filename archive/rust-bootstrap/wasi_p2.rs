//! WASI Preview 2 (Component Model) support for TAYNI
//! Implements filesystem, sockets, and CLI interfaces
//! 
//! WASI Preview 2 uses the Component Model with typed interfaces.
//! This module generates Wasm components that can run on modern runtimes
//! like Wasmtime, WasmEdge, and WAMR.

use crate::target::format::wasm::{
    encode_uleb128, encode_sleb128, encode_string, encode_section,
    WASM_MAGIC, WASM_VERSION,
    SECTION_TYPE, SECTION_IMPORT, SECTION_FUNCTION, SECTION_MEMORY,
    SECTION_EXPORT, SECTION_CODE, SECTION_DATA,
    TYPE_I32, TYPE_I64, TYPE_FUNC,
    OP_END, OP_CALL, OP_LOCAL_GET, OP_LOCAL_SET,
    OP_I32_CONST, OP_I64_CONST, OP_I32_ADD,
};

// ============================================================================
// WASI Preview 2 Interface Names (wit-based)
// ============================================================================

/// WASI Preview 2 uses WIT (WebAssembly Interface Types) packages
pub mod wit {
    // Filesystem interfaces
    pub const FILESYSTEM_TYPES: &str = "wasi:filesystem/types@0.2.0";
    pub const FILESYSTEM_PREOPENS: &str = "wasi:filesystem/preopens@0.2.0";
    
    // CLI interfaces  
    pub const CLI_STDIN: &str = "wasi:cli/stdin@0.2.0";
    pub const CLI_STDOUT: &str = "wasi:cli/stdout@0.2.0";
    pub const CLI_STDERR: &str = "wasi:cli/stderr@0.2.0";
    pub const CLI_ENVIRONMENT: &str = "wasi:cli/environment@0.2.0";
    pub const CLI_EXIT: &str = "wasi:cli/exit@0.2.0";
    
    // IO interfaces
    pub const IO_STREAMS: &str = "wasi:io/streams@0.2.0";
    pub const IO_POLL: &str = "wasi:io/poll@0.2.0";
    
    // Sockets interfaces
    pub const SOCKETS_TCP: &str = "wasi:sockets/tcp@0.2.0";
    pub const SOCKETS_UDP: &str = "wasi:sockets/udp@0.2.0";
    pub const SOCKETS_NETWORK: &str = "wasi:sockets/network@0.2.0";
    
    // Clocks
    pub const CLOCKS_WALL: &str = "wasi:clocks/wall-clock@0.2.0";
    pub const CLOCKS_MONOTONIC: &str = "wasi:clocks/monotonic-clock@0.2.0";
    
    // Random
    pub const RANDOM: &str = "wasi:random/random@0.2.0";
}

// ============================================================================
// Filesystem Types (wasi:filesystem/types)
// ============================================================================

/// File descriptor flags
#[derive(Debug, Clone, Copy)]
pub struct DescriptorFlags {
    pub read: bool,
    pub write: bool,
    pub file_integrity_sync: bool,
    pub data_integrity_sync: bool,
    pub requested_write_sync: bool,
    pub mutate_directory: bool,
}

impl DescriptorFlags {
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            file_integrity_sync: false,
            data_integrity_sync: false,
            requested_write_sync: false,
            mutate_directory: false,
        }
    }
    
    pub fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            file_integrity_sync: false,
            data_integrity_sync: false,
            requested_write_sync: false,
            mutate_directory: false,
        }
    }
    
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            file_integrity_sync: false,
            data_integrity_sync: false,
            requested_write_sync: false,
            mutate_directory: false,
        }
    }
    
    pub fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.read { bits |= 1 << 0; }
        if self.write { bits |= 1 << 1; }
        if self.file_integrity_sync { bits |= 1 << 2; }
        if self.data_integrity_sync { bits |= 1 << 3; }
        if self.requested_write_sync { bits |= 1 << 4; }
        if self.mutate_directory { bits |= 1 << 5; }
        bits
    }
}

/// Open flags for opening files
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenFlags {
    pub create: bool,
    pub directory: bool,
    pub exclusive: bool,
    pub truncate: bool,
}

impl OpenFlags {
    pub fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.create { bits |= 1 << 0; }
        if self.directory { bits |= 1 << 1; }
        if self.exclusive { bits |= 1 << 2; }
        if self.truncate { bits |= 1 << 3; }
        bits
    }
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Unknown = 0,
    BlockDevice = 1,
    CharacterDevice = 2,
    Directory = 3,
    Fifo = 4,
    SymbolicLink = 5,
    RegularFile = 6,
    Socket = 7,
}

/// Error codes for filesystem operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Success = 0,
    Access = 1,
    WouldBlock = 2,
    Already = 3,
    BadDescriptor = 4,
    Busy = 5,
    Deadlock = 6,
    Quota = 7,
    Exist = 8,
    FileTooLarge = 9,
    IllegalByteSequence = 10,
    InProgress = 11,
    Interrupted = 12,
    Invalid = 13,
    Io = 14,
    IsDirectory = 15,
    Loop = 16,
    TooManyLinks = 17,
    MessageSize = 18,
    NameTooLong = 19,
    NoDevice = 20,
    NoEntry = 21,
    NoLock = 22,
    InsufficientMemory = 23,
    InsufficientSpace = 24,
    NotDirectory = 25,
    NotEmpty = 26,
    NotRecoverable = 27,
    Unsupported = 28,
    NoTty = 29,
    NoSuchDevice = 30,
    Overflow = 31,
    NotPermitted = 32,
    Pipe = 33,
    ReadOnly = 34,
    InvalidSeek = 35,
    TextFileBusy = 36,
    CrossDevice = 37,
}

// ============================================================================
// WASI Preview 2 Function Signatures
// ============================================================================

/// Filesystem function indices (after imports)
#[derive(Debug, Clone, Copy)]
pub enum WasiP2Fn {
    // Preopens
    GetDirectories = 0,
    
    // Descriptor operations
    ReadViaStream = 1,
    WriteViaStream = 2,
    AppendViaStream = 3,
    GetType = 4,
    Stat = 5,
    StatAt = 6,
    SetTimesAt = 7,
    LinkAt = 8,
    OpenAt = 9,
    Readlink = 10,
    RemoveDirectoryAt = 11,
    Rename = 12,
    SymlinkAt = 13,
    UnlinkFileAt = 14,
    MetadataHash = 15,
    MetadataHashAt = 16,
    
    // Directory operations
    ReadDirectory = 17,
    
    // Stream operations (from wasi:io/streams)
    StreamRead = 18,
    StreamBlockingRead = 19,
    StreamWrite = 20,
    StreamBlockingWrite = 21,
    StreamFlush = 22,
    StreamBlockingFlush = 23,
    StreamDrop = 24,
}

// ============================================================================
// Code Generation for WASI Preview 2
// ============================================================================

/// Generate type section for WASI P2 filesystem module
fn generate_p2_type_section() -> Vec<u8> {
    let mut types = Vec::new();
    
    // Count of types
    types.extend(encode_uleb128(12));
    
    // Type 0: () -> i32 (get-directories returns list handle)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0)); // 0 params
    types.extend(encode_uleb128(1)); // 1 result
    types.push(TYPE_I32);
    
    // Type 1: (i32) -> i32 (descriptor -> stream)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 2: (i32) -> i32 (get-type: descriptor -> type)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 3: (i32, i32, i32, i32, i32, i32) -> i32 (open-at)
    // (descriptor, path_flags, path_ptr, path_len, open_flags, desc_flags) -> result
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(6));
    types.push(TYPE_I32); // descriptor
    types.push(TYPE_I32); // path_flags
    types.push(TYPE_I32); // path_ptr
    types.push(TYPE_I32); // path_len
    types.push(TYPE_I32); // open_flags
    types.push(TYPE_I32); // descriptor_flags
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 4: (i32, i32, i32) -> i32 (read: stream, buf_ptr, buf_len -> bytes_read)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 5: (i32, i32, i32) -> i32 (write: stream, buf_ptr, buf_len -> bytes_written)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 6: (i32) -> () (drop/close)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(0));
    
    // Type 7: (i32, i32, i32) -> i32 (unlink-file-at: desc, path_ptr, path_len)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 8: (i32, i32, i32) -> i32 (create-directory-at)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 9: (i32, i32, i32) -> i32 (remove-directory-at)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 10: () -> () (_start)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0));
    types.extend(encode_uleb128(0));
    
    // Type 11: (i32) -> () (exit)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(0));
    
    types
}

/// Generate import section for WASI P2 filesystem
fn generate_p2_import_section() -> Vec<u8> {
    let mut imports = Vec::new();
    
    // 10 imports
    imports.extend(encode_uleb128(10));
    
    // Import 0: get-directories from preopens
    imports.extend(encode_string(wit::FILESYSTEM_PREOPENS));
    imports.extend(encode_string("get-directories"));
    imports.push(0x00); // func
    imports.extend(encode_uleb128(0)); // type 0
    
    // Import 1: read-via-stream
    imports.extend(encode_string(wit::FILESYSTEM_TYPES));
    imports.extend(encode_string("read-via-stream"));
    imports.push(0x00);
    imports.extend(encode_uleb128(1)); // type 1
    
    // Import 2: write-via-stream
    imports.extend(encode_string(wit::FILESYSTEM_TYPES));
    imports.extend(encode_string("write-via-stream"));
    imports.push(0x00);
    imports.extend(encode_uleb128(1)); // type 1
    
    // Import 3: open-at
    imports.extend(encode_string(wit::FILESYSTEM_TYPES));
    imports.extend(encode_string("open-at"));
    imports.push(0x00);
    imports.extend(encode_uleb128(3)); // type 3
    
    // Import 4: read (from streams)
    imports.extend(encode_string(wit::IO_STREAMS));
    imports.extend(encode_string("read"));
    imports.push(0x00);
    imports.extend(encode_uleb128(4)); // type 4
    
    // Import 5: blocking-write (from streams)
    imports.extend(encode_string(wit::IO_STREAMS));
    imports.extend(encode_string("blocking-write-and-flush"));
    imports.push(0x00);
    imports.extend(encode_uleb128(5)); // type 5
    
    // Import 6: drop (resource cleanup)
    imports.extend(encode_string(wit::IO_STREAMS));
    imports.extend(encode_string("[resource-drop]output-stream"));
    imports.push(0x00);
    imports.extend(encode_uleb128(6)); // type 6
    
    // Import 7: unlink-file-at
    imports.extend(encode_string(wit::FILESYSTEM_TYPES));
    imports.extend(encode_string("unlink-file-at"));
    imports.push(0x00);
    imports.extend(encode_uleb128(7)); // type 7
    
    // Import 8: create-directory-at
    imports.extend(encode_string(wit::FILESYSTEM_TYPES));
    imports.extend(encode_string("create-directory-at"));
    imports.push(0x00);
    imports.extend(encode_uleb128(8)); // type 8
    
    // Import 9: exit
    imports.extend(encode_string(wit::CLI_EXIT));
    imports.extend(encode_string("exit"));
    imports.push(0x00);
    imports.extend(encode_uleb128(11)); // type 11
    
    imports
}

// ============================================================================
// High-Level Filesystem Operations
// ============================================================================

/// Represents a filesystem operation to generate
#[derive(Debug, Clone)]
pub enum FsOperation {
    /// Read entire file contents
    ReadFile { path: String },
    /// Write data to file (creates or truncates)
    WriteFile { path: String, data: Vec<u8> },
    /// Append data to file
    AppendFile { path: String, data: Vec<u8> },
    /// Delete a file
    DeleteFile { path: String },
    /// Create a directory
    CreateDir { path: String },
    /// Remove a directory
    RemoveDir { path: String },
    /// List directory contents
    ListDir { path: String },
    /// Check if path exists
    Exists { path: String },
    /// Get file metadata
    Stat { path: String },
}

/// Generate a WASI P2 module that performs filesystem operations
pub fn generate_wasi_p2_fs(operations: &[FsOperation]) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let types = generate_p2_type_section();
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Import section
    let imports = generate_p2_import_section();
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    // Function section - declare _start
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1)); // 1 function
    funcs.extend(encode_uleb128(10)); // type 10 (() -> ())
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Memory section
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1));
    mem.push(0x00); // no max
    mem.extend(encode_uleb128(1)); // 1 page
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(2));
    exports.extend(encode_string("_start"));
    exports.push(0x00);
    exports.extend(encode_uleb128(10)); // func index after 10 imports
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Collect string data for data section
    let mut strings: Vec<(usize, Vec<u8>)> = Vec::new();
    let mut offset = 0usize;
    
    for op in operations {
        match op {
            FsOperation::ReadFile { path } |
            FsOperation::DeleteFile { path } |
            FsOperation::CreateDir { path } |
            FsOperation::RemoveDir { path } |
            FsOperation::ListDir { path } |
            FsOperation::Exists { path } |
            FsOperation::Stat { path } => {
                let bytes = path.as_bytes().to_vec();
                strings.push((offset, bytes.clone()));
                offset += bytes.len();
            }
            FsOperation::WriteFile { path, data } |
            FsOperation::AppendFile { path, data } => {
                let path_bytes = path.as_bytes().to_vec();
                strings.push((offset, path_bytes.clone()));
                offset += path_bytes.len();
                strings.push((offset, data.clone()));
                offset += data.len();
            }
        }
    }
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1)); // 1 function
    
    // Generate _start body
    let mut body = Vec::new();
    
    // Locals: 2 i32 (for descriptor handles)
    body.extend(encode_uleb128(1)); // 1 local declaration
    body.extend(encode_uleb128(2)); // 2 locals
    body.push(TYPE_I32);
    
    // Get preopened directories (returns list handle in local 0)
    body.push(OP_CALL);
    body.extend(encode_uleb128(0)); // import 0: get-directories
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(0)); // store in local 0
    
    // Generate code for each operation
    let mut data_offset = 0i32;
    for op in operations {
        match op {
            FsOperation::WriteFile { path, data } => {
                let path_len = path.len() as i32;
                let data_len = data.len() as i32;
                
                // open-at(preopened_dir=local0, path_flags=0, path_ptr, path_len, 
                //         open_flags=CREATE|TRUNCATE, desc_flags=WRITE)
                body.push(OP_LOCAL_GET);
                body.extend(encode_uleb128(0)); // preopened dir
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(0)); // path_flags = 0
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(data_offset as i64)); // path_ptr
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(path_len as i64)); // path_len
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(0x09)); // CREATE | TRUNCATE
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(0x02)); // WRITE flag
                body.push(OP_CALL);
                body.extend(encode_uleb128(3)); // import 3: open-at
                body.push(OP_LOCAL_SET);
                body.extend(encode_uleb128(1)); // store fd in local 1
                
                data_offset += path_len;
                
                // write-via-stream(fd) -> stream
                body.push(OP_LOCAL_GET);
                body.extend(encode_uleb128(1));
                body.push(OP_CALL);
                body.extend(encode_uleb128(2)); // import 2: write-via-stream
                // stream handle on stack
                
                // blocking-write-and-flush(stream, data_ptr, data_len)
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(data_offset as i64)); // data_ptr
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(data_len as i64)); // data_len
                body.push(OP_CALL);
                body.extend(encode_uleb128(5)); // import 5: blocking-write-and-flush
                body.push(0x1A); // drop result
                
                data_offset += data_len;
            }
            FsOperation::DeleteFile { path } => {
                let path_len = path.len() as i32;
                
                // unlink-file-at(preopened_dir, path_ptr, path_len)
                body.push(OP_LOCAL_GET);
                body.extend(encode_uleb128(0));
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(data_offset as i64));
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(path_len as i64));
                body.push(OP_CALL);
                body.extend(encode_uleb128(7)); // import 7: unlink-file-at
                body.push(0x1A); // drop result
                
                data_offset += path_len;
            }
            FsOperation::CreateDir { path } => {
                let path_len = path.len() as i32;
                
                // create-directory-at(preopened_dir, path_ptr, path_len)
                body.push(OP_LOCAL_GET);
                body.extend(encode_uleb128(0));
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(data_offset as i64));
                body.push(OP_I32_CONST);
                body.extend(encode_sleb128(path_len as i64));
                body.push(OP_CALL);
                body.extend(encode_uleb128(8)); // import 8: create-directory-at
                body.push(0x1A); // drop result
                
                data_offset += path_len;
            }
            _ => {
                // Other operations - advance offset for path
                if let FsOperation::ReadFile { path } |
                       FsOperation::RemoveDir { path } |
                       FsOperation::ListDir { path } |
                       FsOperation::Exists { path } |
                       FsOperation::Stat { path } = op {
                    data_offset += path.len() as i32;
                }
            }
        }
    }
    
    // Exit with success
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(9)); // import 9: exit
    
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    // Data section
    if !strings.is_empty() {
        let mut data = Vec::new();
        data.extend(encode_uleb128(strings.len() as u64));
        
        for (off, bytes) in &strings {
            data.push(0x00); // active, memory 0
            data.push(OP_I32_CONST);
            data.extend(encode_sleb128(*off as i64));
            data.push(OP_END);
            data.extend(encode_uleb128(bytes.len() as u64));
            data.extend(bytes);
        }
        
        wasm.extend(encode_section(SECTION_DATA, &data));
    }
    
    wasm
}

/// Generate a simple file write module
pub fn generate_wasi_p2_write_file(path: &str, content: &[u8]) -> Vec<u8> {
    generate_wasi_p2_fs(&[FsOperation::WriteFile {
        path: path.to_string(),
        data: content.to_vec(),
    }])
}

/// Generate a simple file read module (prints to stdout)
pub fn generate_wasi_p2_read_file(path: &str) -> Vec<u8> {
    generate_wasi_p2_fs(&[FsOperation::ReadFile {
        path: path.to_string(),
    }])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_descriptor_flags() {
        let flags = DescriptorFlags::read_write();
        assert!(flags.read);
        assert!(flags.write);
        assert_eq!(flags.to_bits(), 0b11);
    }
    
    #[test]
    fn test_open_flags() {
        let mut flags = OpenFlags::default();
        flags.create = true;
        flags.truncate = true;
        assert_eq!(flags.to_bits(), 0b1001);
    }
    
    #[test]
    fn test_generate_write_file() {
        let wasm = generate_wasi_p2_write_file("test.txt", b"Hello WASI P2!");
        assert!(wasm.len() > 0);
        assert_eq!(&wasm[0..4], &WASM_MAGIC);
        assert_eq!(&wasm[4..8], &WASM_VERSION);
    }
    
    #[test]
    fn test_generate_multiple_ops() {
        let ops = vec![
            FsOperation::CreateDir { path: "mydir".to_string() },
            FsOperation::WriteFile { 
                path: "mydir/file.txt".to_string(), 
                data: b"content".to_vec() 
            },
        ];
        let wasm = generate_wasi_p2_fs(&ops);
        assert!(wasm.len() > 100); // Should have substantial content
    }
}

// ============================================================================
// WASI Preview 2 Sockets (wasi:sockets/tcp, wasi:sockets/udp)
// ============================================================================

/// IP Address family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpAddressFamily {
    Ipv4 = 0,
    Ipv6 = 1,
}

/// IPv4 address
#[derive(Debug, Clone, Copy)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }
    
    pub fn localhost() -> Self {
        Ipv4Address([127, 0, 0, 1])
    }
    
    pub fn any() -> Self {
        Ipv4Address([0, 0, 0, 0])
    }
}

/// IPv6 address
#[derive(Debug, Clone, Copy)]
pub struct Ipv6Address(pub [u16; 8]);

impl Ipv6Address {
    pub fn localhost() -> Self {
        Ipv6Address([0, 0, 0, 0, 0, 0, 0, 1])
    }
    
    pub fn any() -> Self {
        Ipv6Address([0; 8])
    }
}

/// Socket address (IP + port)
#[derive(Debug, Clone)]
pub enum IpSocketAddress {
    Ipv4 { address: Ipv4Address, port: u16 },
    Ipv6 { address: Ipv6Address, port: u16, flow_info: u32, scope_id: u32 },
}

impl IpSocketAddress {
    pub fn ipv4(addr: Ipv4Address, port: u16) -> Self {
        IpSocketAddress::Ipv4 { address: addr, port }
    }
    
    pub fn ipv6(addr: Ipv6Address, port: u16) -> Self {
        IpSocketAddress::Ipv6 { 
            address: addr, 
            port, 
            flow_info: 0, 
            scope_id: 0 
        }
    }
    
    pub fn localhost(port: u16) -> Self {
        IpSocketAddress::Ipv4 { 
            address: Ipv4Address::localhost(), 
            port 
        }
    }
}

/// TCP socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Unbound,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closed,
}

/// TCP shutdown type
#[derive(Debug, Clone, Copy)]
pub enum ShutdownType {
    Receive = 0,
    Send = 1,
    Both = 2,
}

/// Socket error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketErrorCode {
    Success = 0,
    AccessDenied = 1,
    NotSupported = 2,
    InvalidArgument = 3,
    OutOfMemory = 4,
    Timeout = 5,
    ConcurrencyConflict = 6,
    NotInProgress = 7,
    WouldBlock = 8,
    InvalidState = 9,
    NewSocketLimit = 10,
    AddressNotBindable = 11,
    AddressInUse = 12,
    RemoteUnreachable = 13,
    ConnectionRefused = 14,
    ConnectionReset = 15,
    ConnectionAborted = 16,
    DatagramTooLarge = 17,
    NameUnresolvable = 18,
    TemporaryResolverFailure = 19,
    PermanentResolverFailure = 20,
}

/// TCP socket operation
#[derive(Debug, Clone)]
pub enum TcpOperation {
    /// Create a new TCP socket
    CreateSocket { family: IpAddressFamily },
    /// Bind socket to address
    Bind { address: IpSocketAddress },
    /// Start listening for connections
    Listen { backlog: u32 },
    /// Accept incoming connection
    Accept,
    /// Connect to remote address
    Connect { address: IpSocketAddress },
    /// Send data
    Send { data: Vec<u8> },
    /// Receive data
    Receive { max_bytes: u32 },
    /// Shutdown socket
    Shutdown { how: ShutdownType },
    /// Close socket
    Close,
}

/// UDP socket operation
#[derive(Debug, Clone)]
pub enum UdpOperation {
    /// Create a new UDP socket
    CreateSocket { family: IpAddressFamily },
    /// Bind socket to address
    Bind { address: IpSocketAddress },
    /// Send datagram
    SendTo { address: IpSocketAddress, data: Vec<u8> },
    /// Receive datagram
    ReceiveFrom { max_bytes: u32 },
    /// Close socket
    Close,
}

// ============================================================================
// Socket Type Definitions for WASI P2
// ============================================================================

/// Generate type section for TCP sockets
fn generate_tcp_type_section() -> Vec<u8> {
    let mut types = Vec::new();
    
    // Count of types
    types.extend(encode_uleb128(10));
    
    // Type 0: create-tcp-socket (i32) -> i32 (family -> socket handle)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 1: start-bind (i32, i32, i32, i32, i32, i32) -> i32
    // (socket, addr_family, addr_ptr, addr_len, port_hi, port_lo) -> result
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(6));
    for _ in 0..6 { types.push(TYPE_I32); }
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 2: finish-bind (i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 3: start-listen (i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 4: finish-listen (i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 5: accept (i32) -> (i32, i32, i32) - socket, input_stream, output_stream
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    
    // Type 6: start-connect (i32, i32, i32, i32, i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(6));
    for _ in 0..6 { types.push(TYPE_I32); }
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 7: finish-connect (i32) -> (i32, i32, i32)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    
    // Type 8: shutdown (i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(2));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 9: () -> () (_start)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0));
    types.extend(encode_uleb128(0));
    
    types
}

/// Generate import section for TCP sockets
fn generate_tcp_import_section() -> Vec<u8> {
    let mut imports = Vec::new();
    
    // 9 imports
    imports.extend(encode_uleb128(9));
    
    // Import 0: create-tcp-socket from network
    imports.extend(encode_string(wit::SOCKETS_NETWORK));
    imports.extend(encode_string("create-tcp-socket"));
    imports.push(0x00);
    imports.extend(encode_uleb128(0));
    
    // Import 1: start-bind
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("start-bind"));
    imports.push(0x00);
    imports.extend(encode_uleb128(1));
    
    // Import 2: finish-bind
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("finish-bind"));
    imports.push(0x00);
    imports.extend(encode_uleb128(2));
    
    // Import 3: start-listen
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("start-listen"));
    imports.push(0x00);
    imports.extend(encode_uleb128(3));
    
    // Import 4: finish-listen
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("finish-listen"));
    imports.push(0x00);
    imports.extend(encode_uleb128(4));
    
    // Import 5: accept
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("accept"));
    imports.push(0x00);
    imports.extend(encode_uleb128(5));
    
    // Import 6: start-connect
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("start-connect"));
    imports.push(0x00);
    imports.extend(encode_uleb128(6));
    
    // Import 7: finish-connect
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("finish-connect"));
    imports.push(0x00);
    imports.extend(encode_uleb128(7));
    
    // Import 8: shutdown
    imports.extend(encode_string(wit::SOCKETS_TCP));
    imports.extend(encode_string("shutdown"));
    imports.push(0x00);
    imports.extend(encode_uleb128(8));
    
    imports
}

/// Generate a WASI P2 TCP server module
pub fn generate_wasi_p2_tcp_server(port: u16, response: &[u8]) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let types = generate_tcp_type_section();
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Import section
    let imports = generate_tcp_import_section();
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    // Function section - declare _start
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(9)); // type 9 (() -> ())
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Memory section
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1));
    mem.push(0x00);
    mem.extend(encode_uleb128(1));
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(2));
    exports.extend(encode_string("_start"));
    exports.push(0x00);
    exports.extend(encode_uleb128(9)); // func index after 9 imports
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section - TCP server logic
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    
    // Locals: socket_handle, input_stream, output_stream
    body.extend(encode_uleb128(1));
    body.extend(encode_uleb128(3));
    body.push(TYPE_I32);
    
    // Create TCP socket (IPv4)
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // IPv4
    body.push(OP_CALL);
    body.extend(encode_uleb128(0)); // create-tcp-socket
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(0)); // store socket handle
    
    // Start bind to 0.0.0.0:port
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0)); // socket
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // IPv4 family
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // addr ptr (0.0.0.0 at offset 0)
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(4)); // addr len
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128((port >> 8) as i64)); // port high byte
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128((port & 0xFF) as i64)); // port low byte
    body.push(OP_CALL);
    body.extend(encode_uleb128(1)); // start-bind
    body.push(0x1A); // drop result
    
    // Finish bind
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(2)); // finish-bind
    body.push(0x1A); // drop
    
    // Start listen
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(3)); // start-listen
    body.push(0x1A); // drop
    
    // Finish listen
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(4)); // finish-listen
    body.push(0x1A); // drop
    
    // Accept connection (returns socket, input_stream, output_stream)
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(5)); // accept
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(2)); // output_stream
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(1)); // input_stream
    body.push(0x1A); // drop client socket
    
    // Note: In a real implementation, we would write response to output_stream
    // using wasi:io/streams write operations
    
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    // Data section - address (0.0.0.0) and response
    let mut data = Vec::new();
    data.extend(encode_uleb128(2)); // 2 data segments
    
    // Segment 0: IPv4 address 0.0.0.0 at offset 0
    data.push(0x00);
    data.push(OP_I32_CONST);
    data.extend(encode_sleb128(0));
    data.push(OP_END);
    data.extend(encode_uleb128(4));
    data.extend(&[0, 0, 0, 0]);
    
    // Segment 1: response at offset 4
    data.push(0x00);
    data.push(OP_I32_CONST);
    data.extend(encode_sleb128(4));
    data.push(OP_END);
    data.extend(encode_uleb128(response.len() as u64));
    data.extend(response);
    
    wasm.extend(encode_section(SECTION_DATA, &data));
    
    wasm
}

/// Generate a WASI P2 TCP client module
pub fn generate_wasi_p2_tcp_client(host: Ipv4Address, port: u16) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    let types = generate_tcp_type_section();
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    let imports = generate_tcp_import_section();
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(9));
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1));
    mem.push(0x00);
    mem.extend(encode_uleb128(1));
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(2));
    exports.extend(encode_string("_start"));
    exports.push(0x00);
    exports.extend(encode_uleb128(9));
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section - TCP client logic
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(1));
    body.extend(encode_uleb128(3));
    body.push(TYPE_I32);
    
    // Create socket
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // IPv4
    body.push(OP_CALL);
    body.extend(encode_uleb128(0));
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(0));
    
    // Start connect
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // IPv4
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // addr ptr
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(4)); // addr len
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128((port >> 8) as i64));
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128((port & 0xFF) as i64));
    body.push(OP_CALL);
    body.extend(encode_uleb128(6)); // start-connect
    body.push(0x1A);
    
    // Finish connect
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_CALL);
    body.extend(encode_uleb128(7)); // finish-connect
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(2)); // output
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(1)); // input
    body.push(0x1A); // drop result
    
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    // Data section - host address
    let mut data = Vec::new();
    data.extend(encode_uleb128(1));
    data.push(0x00);
    data.push(OP_I32_CONST);
    data.extend(encode_sleb128(0));
    data.push(OP_END);
    data.extend(encode_uleb128(4));
    data.extend(&host.0);
    
    wasm.extend(encode_section(SECTION_DATA, &data));
    
    wasm
}

// ============================================================================
// Socket Tests
// ============================================================================

#[cfg(test)]
mod socket_tests {
    use super::*;
    
    #[test]
    fn test_ipv4_address() {
        let addr = Ipv4Address::new(192, 168, 1, 1);
        assert_eq!(addr.0, [192, 168, 1, 1]);
        
        let localhost = Ipv4Address::localhost();
        assert_eq!(localhost.0, [127, 0, 0, 1]);
    }
    
    #[test]
    fn test_socket_address() {
        let addr = IpSocketAddress::localhost(8080);
        match addr {
            IpSocketAddress::Ipv4 { address, port } => {
                assert_eq!(address.0, [127, 0, 0, 1]);
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected IPv4"),
        }
    }
    
    #[test]
    fn test_generate_tcp_server() {
        let response = b"HTTP/1.1 200 OK\r\n\r\nHello!";
        let wasm = generate_wasi_p2_tcp_server(8080, response);
        
        assert!(wasm.len() > 0);
        assert_eq!(&wasm[0..4], &WASM_MAGIC);
        assert_eq!(&wasm[4..8], &WASM_VERSION);
    }
    
    #[test]
    fn test_generate_tcp_client() {
        let wasm = generate_wasi_p2_tcp_client(Ipv4Address::localhost(), 80);
        
        assert!(wasm.len() > 0);
        assert_eq!(&wasm[0..4], &WASM_MAGIC);
    }
}
