//! Test Wasm generation - generates multiple Wasm modules for testing

use std::fs;

fn main() {
    println!("=== TAYNI Wasm Generator Test Suite ===\n");
    
    // Test 1: Minimal module (just returns 0)
    println!("Test 1: Minimal Wasm (return 0)");
    let wasm1 = generate_minimal_wasm();
    fs::write("wasm_minimal.wasm", &wasm1).expect("Failed to write");
    println!("  Generated: wasm_minimal.wasm ({} bytes)", wasm1.len());
    
    // Test 2: Return constant
    println!("\nTest 2: Return constant 42");
    let wasm2 = generate_return_const_wasm(42);
    fs::write("wasm_const42.wasm", &wasm2).expect("Failed to write");
    println!("  Generated: wasm_const42.wasm ({} bytes)", wasm2.len());
    
    // Test 3: Add function
    println!("\nTest 3: Add function (a + b)");
    let wasm3 = generate_add_wasm();
    fs::write("wasm_add.wasm", &wasm3).expect("Failed to write");
    println!("  Generated: wasm_add.wasm ({} bytes)", wasm3.len());
    
    // Test 4: Factorial (recursive)
    println!("\nTest 4: Factorial function");
    let wasm4 = generate_factorial_wasm();
    fs::write("wasm_factorial.wasm", &wasm4).expect("Failed to write");
    println!("  Generated: wasm_factorial.wasm ({} bytes)", wasm4.len());
    
    // Test 5: Memory operations
    println!("\nTest 5: Memory store/load");
    let wasm5 = generate_memory_wasm();
    fs::write("wasm_memory.wasm", &wasm5).expect("Failed to write");
    println!("  Generated: wasm_memory.wasm ({} bytes)", wasm5.len());
    
    println!("\n=== Summary ===");
    let total = wasm1.len() + wasm2.len() + wasm3.len() + wasm4.len() + wasm5.len();
    println!("Total: 5 modules, {} bytes", total);
    println!("\nRun: node test_wasm_suite.js");
}

// Wasm constants
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D]; // \0asm
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00]; // version 1

const SECTION_TYPE: u8 = 1;
const SECTION_FUNCTION: u8 = 3;
const SECTION_MEMORY: u8 = 5;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;

const TYPE_I32: u8 = 0x7F;
const TYPE_FUNC: u8 = 0x60;

const OP_END: u8 = 0x0B;
const OP_LOCAL_GET: u8 = 0x20;
const OP_LOCAL_SET: u8 = 0x21;
const OP_I32_CONST: u8 = 0x41;
const OP_I32_ADD: u8 = 0x6A;
const OP_I32_SUB: u8 = 0x6B;
const OP_I32_MUL: u8 = 0x6C;
const OP_I32_EQZ: u8 = 0x45;
const OP_I32_LOAD: u8 = 0x28;
const OP_I32_STORE: u8 = 0x36;
const OP_IF: u8 = 0x04;
const OP_ELSE: u8 = 0x05;
const OP_CALL: u8 = 0x10;
const OP_RETURN: u8 = 0x0F;

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

/// Minimal Wasm: () -> i32, returns 0
fn generate_minimal_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section: () -> i32
    let mut types = Vec::new();
    types.extend(encode_uleb128(1)); // 1 type
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0)); // 0 params
    types.extend(encode_uleb128(1)); // 1 result
    types.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1)); // 1 function
    funcs.extend(encode_uleb128(0)); // type 0
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(1)); // 1 export
    exports.extend(encode_string("main"));
    exports.push(0x00); // func
    exports.extend(encode_uleb128(0)); // index 0
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1)); // 1 function
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0)); // 0 locals
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(0));
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}

/// Return constant: () -> i32, returns n
fn generate_return_const_wasm(n: i32) -> Vec<u8> {
    let mut wasm = Vec::new();
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let mut types = Vec::new();
    types.extend(encode_uleb128(1));
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0));
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(1));
    exports.extend(encode_string("getValue"));
    exports.push(0x00);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0));
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(n as i64));
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}

/// Add function: (i32, i32) -> i32
fn generate_add_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section: (i32, i32) -> i32
    let mut types = Vec::new();
    types.extend(encode_uleb128(1));
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(2)); // 2 params
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1)); // 1 result
    types.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(1));
    exports.extend(encode_string("add"));
    exports.push(0x00);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0)); // 0 locals
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0)); // param 0
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(1)); // param 1
    body.push(OP_I32_ADD);
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}

/// Factorial: (i32) -> i32, recursive
fn generate_factorial_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section: (i32) -> i32
    let mut types = Vec::new();
    types.extend(encode_uleb128(1));
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1)); // 1 param
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1)); // 1 result
    types.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(1));
    exports.extend(encode_string("factorial"));
    exports.push(0x00);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    // factorial(n) = if n == 0 then 1 else n * factorial(n-1)
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    let mut body = Vec::new();
    body.extend(encode_uleb128(0)); // 0 locals
    
    // if (n == 0)
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0));
    body.push(OP_I32_EQZ);
    body.push(OP_IF);
    body.push(TYPE_I32); // result type
    
    // then: return 1
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(1));
    
    body.push(OP_ELSE);
    
    // else: n * factorial(n-1)
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0)); // n
    
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0)); // n
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(1));
    body.push(OP_I32_SUB); // n - 1
    body.push(OP_CALL);
    body.extend(encode_uleb128(0)); // call factorial
    
    body.push(OP_I32_MUL); // n * factorial(n-1)
    
    body.push(OP_END); // end if
    body.push(OP_END); // end function
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}

/// Memory operations: store and load
fn generate_memory_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let mut types = Vec::new();
    types.extend(encode_uleb128(2));
    // Type 0: (i32, i32) -> () - store
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(2));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(0));
    // Type 1: (i32) -> i32 - load
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Function section
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(2));
    funcs.extend(encode_uleb128(0)); // store
    funcs.extend(encode_uleb128(1)); // load
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Memory section
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1));
    mem.push(0x00); // no max
    mem.extend(encode_uleb128(1)); // 1 page
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(3));
    exports.extend(encode_string("store"));
    exports.push(0x00);
    exports.extend(encode_uleb128(0));
    exports.extend(encode_string("load"));
    exports.push(0x00);
    exports.extend(encode_uleb128(1));
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(2));
    
    // store(addr, value)
    let mut body1 = Vec::new();
    body1.extend(encode_uleb128(0));
    body1.push(OP_LOCAL_GET);
    body1.extend(encode_uleb128(0)); // addr
    body1.push(OP_LOCAL_GET);
    body1.extend(encode_uleb128(1)); // value
    body1.push(OP_I32_STORE);
    body1.push(0x02); // align
    body1.push(0x00); // offset
    body1.push(OP_END);
    code.extend(encode_uleb128(body1.len() as u64));
    code.extend(body1);
    
    // load(addr) -> value
    let mut body2 = Vec::new();
    body2.extend(encode_uleb128(0));
    body2.push(OP_LOCAL_GET);
    body2.extend(encode_uleb128(0)); // addr
    body2.push(OP_I32_LOAD);
    body2.push(0x02); // align
    body2.push(0x00); // offset
    body2.push(OP_END);
    code.extend(encode_uleb128(body2.len() as u64));
    code.extend(body2);
    
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    wasm
}
