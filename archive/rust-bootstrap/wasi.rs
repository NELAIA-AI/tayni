//! WASI (WebAssembly System Interface) support for TAYNI
//! Implements WASI Preview 1 for compatibility
//! WASI Preview 2 (Component Model) support is planned

use std::collections::HashMap;

// ============================================================================
// WASI Preview 1 (wasi_snapshot_preview1)
// ============================================================================

pub const WASI_MODULE: &str = "wasi_snapshot_preview1";

// WASI error codes
pub const WASI_ERRNO_SUCCESS: u32 = 0;
pub const WASI_ERRNO_BADF: u32 = 8;
pub const WASI_ERRNO_INVAL: u32 = 28;
pub const WASI_ERRNO_NOENT: u32 = 44;

// WASI file descriptors
pub const WASI_STDIN: u32 = 0;
pub const WASI_STDOUT: u32 = 1;
pub const WASI_STDERR: u32 = 2;

// WASI rights
pub const WASI_RIGHT_FD_READ: u64 = 1 << 1;
pub const WASI_RIGHT_FD_WRITE: u64 = 1 << 6;
pub const WASI_RIGHT_PATH_OPEN: u64 = 1 << 13;

// Wasm encoding helpers
fn encode_uleb128(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}

fn encode_sleb128(mut value: i64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut more = true;
    while more {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0) {
            more = false;
        } else {
            byte |= 0x80;
        }
        result.push(byte);
    }
    result
}

fn encode_string(s: &str) -> Vec<u8> {
    let mut result = encode_uleb128(s.len() as u64);
    result.extend(s.as_bytes());
    result
}

fn encode_section(id: u8, content: &[u8]) -> Vec<u8> {
    let mut result = vec![id];
    result.extend(encode_uleb128(content.len() as u64));
    result.extend(content);
    result
}

// Wasm constants
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_MEMORY: u8 = 5;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;
const SECTION_DATA: u8 = 11;

const TYPE_I32: u8 = 0x7F;
const TYPE_I64: u8 = 0x7E;
const TYPE_FUNC: u8 = 0x60;

const OP_END: u8 = 0x0B;
const OP_CALL: u8 = 0x10;
const OP_LOCAL_GET: u8 = 0x20;
const OP_LOCAL_SET: u8 = 0x21;
const OP_I32_LOAD: u8 = 0x28;
const OP_I32_STORE: u8 = 0x36;
const OP_I32_CONST: u8 = 0x41;
const OP_I32_ADD: u8 = 0x6A;

/// WASI function type indices (after imports)
#[derive(Clone, Copy)]
pub enum WasiFn {
    FdWrite = 0,      // fd_write(fd, iovs, iovs_len, nwritten) -> errno
    FdRead = 1,       // fd_read(fd, iovs, iovs_len, nread) -> errno  
    FdClose = 2,      // fd_close(fd) -> errno
    PathOpen = 3,     // path_open(...) -> errno
    ProcExit = 4,     // proc_exit(code) -> !
    ArgsSizesGet = 5, // args_sizes_get(argc, argv_buf_size) -> errno
    ArgsGet = 6,      // args_get(argv, argv_buf) -> errno
}

/// Generate a WASI-compatible Wasm module that prints "Hello, WASI!"
pub fn generate_wasi_hello() -> Vec<u8> {
    let message = b"Hello from TAYNI via WASI!\n";
    generate_wasi_print(message)
}

/// Generate a WASI module that prints a message to stdout
pub fn generate_wasi_print(message: &[u8]) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let mut types = Vec::new();
    types.extend(encode_uleb128(3)); // 3 types
    
    // Type 0: fd_write signature (i32, i32, i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(4)); // 4 params
    types.push(TYPE_I32); // fd
    types.push(TYPE_I32); // iovs ptr
    types.push(TYPE_I32); // iovs len
    types.push(TYPE_I32); // nwritten ptr
    types.extend(encode_uleb128(1)); // 1 result
    types.push(TYPE_I32); // errno
    
    // Type 1: proc_exit signature (i32) -> ()
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1)); // 1 param
    types.push(TYPE_I32); // exit code
    types.extend(encode_uleb128(0)); // 0 results
    
    // Type 2: _start signature () -> ()
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0)); // 0 params
    types.extend(encode_uleb128(0)); // 0 results
    
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Import section - WASI functions
    let mut imports = Vec::new();
    imports.extend(encode_uleb128(2)); // 2 imports
    
    // Import fd_write
    imports.extend(encode_string(WASI_MODULE));
    imports.extend(encode_string("fd_write"));
    imports.push(0x00); // func import
    imports.extend(encode_uleb128(0)); // type 0
    
    // Import proc_exit
    imports.extend(encode_string(WASI_MODULE));
    imports.extend(encode_string("proc_exit"));
    imports.push(0x00); // func import
    imports.extend(encode_uleb128(1)); // type 1
    
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1)); // 1 function
    funcs.extend(encode_uleb128(2)); // type 2 (_start)
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Memory section
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1)); // 1 memory
    mem.push(0x00); // no max
    mem.extend(encode_uleb128(1)); // 1 page min
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(2)); // 2 exports
    
    // Export _start (WASI entry point)
    exports.extend(encode_string("_start"));
    exports.push(0x00); // func
    exports.extend(encode_uleb128(2)); // func index 2 (after 2 imports)
    
    // Export memory
    exports.extend(encode_string("memory"));
    exports.push(0x02); // memory
    exports.extend(encode_uleb128(0)); // memory 0
    
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1)); // 1 function
    
    // _start function body
    // Memory layout:
    // 0-7: iovec struct (ptr=8, len=message.len())
    // 8+: message data
    // After message: nwritten result
    
    let iovec_ptr: i32 = 0;
    let message_ptr: i32 = 8;
    let nwritten_ptr: i32 = (8 + message.len()) as i32;
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0)); // 0 locals
    
    // Store iovec.buf = message_ptr at offset 0
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(iovec_ptr as i64)); // address 0
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(message_ptr as i64)); // value = message ptr
    body.push(OP_I32_STORE);
    body.push(0x02); // align
    body.push(0x00); // offset
    
    // Store iovec.len = message.len() at offset 4
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128((iovec_ptr + 4) as i64)); // address 4
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(message.len() as i64)); // value = len
    body.push(OP_I32_STORE);
    body.push(0x02); // align
    body.push(0x00); // offset
    
    // Call fd_write(stdout=1, iovs=0, iovs_len=1, nwritten=nwritten_ptr)
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(WASI_STDOUT as i64)); // fd = 1 (stdout)
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(iovec_ptr as i64)); // iovs ptr
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(1)); // iovs_len = 1
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(nwritten_ptr as i64)); // nwritten ptr
    body.push(OP_CALL);
    body.extend(encode_uleb128(0)); // call fd_write (import 0)
    
    // Drop the errno result (we ignore errors for simplicity)
    body.push(0x1A); // drop
    
    // Call proc_exit(0)
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0)); // exit code 0
    body.push(OP_CALL);
    body.extend(encode_uleb128(1)); // call proc_exit (import 1)
    
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    // Data section - message at offset 8
    let mut data = Vec::new();
    data.extend(encode_uleb128(1)); // 1 data segment
    data.push(0x00); // active, memory 0
    data.push(OP_I32_CONST);
    data.extend(encode_sleb128(message_ptr as i64));
    data.push(OP_END);
    data.extend(encode_uleb128(message.len() as u64));
    data.extend(message);
    wasm.extend(encode_section(SECTION_DATA, &data));
    
    wasm
}

/// Generate a WASI module that reads a file and prints its contents
pub fn generate_wasi_cat(preopened_fd: u32) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let mut types = Vec::new();
    types.extend(encode_uleb128(4)); // 4 types
    
    // Type 0: fd_write (i32, i32, i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(4));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 1: fd_read (i32, i32, i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(4));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 2: path_open (i32, i32, i32, i32, i32, i64, i64, i32, i32) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(9));
    types.push(TYPE_I32); // fd
    types.push(TYPE_I32); // dirflags
    types.push(TYPE_I32); // path ptr
    types.push(TYPE_I32); // path len
    types.push(TYPE_I32); // oflags
    types.push(TYPE_I64); // fs_rights_base
    types.push(TYPE_I64); // fs_rights_inheriting
    types.push(TYPE_I32); // fdflags
    types.push(TYPE_I32); // result fd ptr
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 3: _start () -> ()
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0));
    types.extend(encode_uleb128(0));
    
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Import section
    let mut imports = Vec::new();
    imports.extend(encode_uleb128(3)); // 3 imports
    
    imports.extend(encode_string(WASI_MODULE));
    imports.extend(encode_string("fd_write"));
    imports.push(0x00);
    imports.extend(encode_uleb128(0));
    
    imports.extend(encode_string(WASI_MODULE));
    imports.extend(encode_string("fd_read"));
    imports.push(0x00);
    imports.extend(encode_uleb128(1));
    
    imports.extend(encode_string(WASI_MODULE));
    imports.extend(encode_string("path_open"));
    imports.push(0x00);
    imports.extend(encode_uleb128(2));
    
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(3)); // _start type
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
    exports.extend(encode_uleb128(3)); // after 3 imports
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section - simplified cat implementation
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0)); // 0 locals
    // For now, just end - full implementation would read file
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasi_hello_generation() {
        let wasm = generate_wasi_hello();
        assert!(wasm.len() > 0);
        assert_eq!(&wasm[0..4], &WASM_MAGIC);
        assert_eq!(&wasm[4..8], &WASM_VERSION);
    }
}
