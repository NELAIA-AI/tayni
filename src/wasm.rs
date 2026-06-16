//! WebAssembly (WASM) Generator
//! Generates minimal WASM modules from TAYNI graphs

use crate::ir::{Graph, Node, Value, Arg, Op};
use std::collections::HashMap;

// WASM Constants
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D]; // \0asm
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00]; // version 1

// Section IDs
const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_MEMORY: u8 = 5;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;
const SECTION_DATA: u8 = 11;

// Value types
const TYPE_I32: u8 = 0x7F;
const TYPE_I64: u8 = 0x7E;
const TYPE_FUNC: u8 = 0x60;

// Instructions
const OP_END: u8 = 0x0B;
const OP_CALL: u8 = 0x10;
const OP_LOCAL_GET: u8 = 0x20;
const OP_I32_CONST: u8 = 0x41;
const OP_I64_CONST: u8 = 0x42;
const OP_I32_ADD: u8 = 0x6A;
const OP_I32_SUB: u8 = 0x6B;
const OP_I32_MUL: u8 = 0x6C;
const OP_I32_DIV_S: u8 = 0x6D;

/// Encode unsigned LEB128
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

/// Encode signed LEB128
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

/// Encode a string (length-prefixed)
fn encode_string(s: &str) -> Vec<u8> {
    let mut result = encode_uleb128(s.len() as u64);
    result.extend(s.as_bytes());
    result
}

/// Encode a section
fn encode_section(id: u8, content: &[u8]) -> Vec<u8> {
    let mut result = vec![id];
    result.extend(encode_uleb128(content.len() as u64));
    result.extend(content);
    result
}

/// Generate minimal WASM module that exports a main function
pub fn generate_hello_wasm() -> Vec<u8> {
    let mut wasm = Vec::new();
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section: define function types
    // Type 0: () -> i32 (main function)
    let mut type_section = Vec::new();
    type_section.extend(encode_uleb128(1)); // 1 type
    type_section.push(TYPE_FUNC);
    type_section.extend(encode_uleb128(0)); // 0 params
    type_section.extend(encode_uleb128(1)); // 1 result
    type_section.push(TYPE_I32);
    wasm.extend(encode_section(SECTION_TYPE, &type_section));
    
    // Function section: declare functions
    let mut func_section = Vec::new();
    func_section.extend(encode_uleb128(1)); // 1 function
    func_section.extend(encode_uleb128(0)); // type index 0
    wasm.extend(encode_section(SECTION_FUNCTION, &func_section));
    
    // Memory section: 1 page (64KB)
    let mut mem_section = Vec::new();
    mem_section.extend(encode_uleb128(1)); // 1 memory
    mem_section.push(0x00); // no max
    mem_section.extend(encode_uleb128(1)); // 1 page min
    wasm.extend(encode_section(SECTION_MEMORY, &mem_section));
    
    // Export section: export main and memory
    let mut export_section = Vec::new();
    export_section.extend(encode_uleb128(2)); // 2 exports
    // Export main function
    export_section.extend(encode_string("main"));
    export_section.push(0x00); // func export
    export_section.extend(encode_uleb128(0)); // func index 0
    // Export memory
    export_section.extend(encode_string("memory"));
    export_section.push(0x02); // memory export
    export_section.extend(encode_uleb128(0)); // memory index 0
    wasm.extend(encode_section(SECTION_EXPORT, &export_section));
    
    // Code section: function bodies
    let mut code_section = Vec::new();
    code_section.extend(encode_uleb128(1)); // 1 function
    
    // Function 0 body: return 0
    let mut func_body = Vec::new();
    func_body.extend(encode_uleb128(0)); // 0 locals
    func_body.push(OP_I32_CONST);
    func_body.extend(encode_sleb128(0)); // return 0
    func_body.push(OP_END);
    
    code_section.extend(encode_uleb128(func_body.len() as u64));
    code_section.extend(func_body);
    
    wasm.extend(encode_section(SECTION_CODE, &code_section));
    
    wasm
}

/// Generate WASM module from TAYNI graph
pub fn generate_wasm_from_graph(graph: &Graph) -> Vec<u8> {
    let mut wasm = Vec::new();
    let mut strings: Vec<(String, Vec<u8>)> = Vec::new();
    let mut constants: HashMap<String, i64> = HashMap::new();
    
    // Collect strings and constants
    for node in &graph.nodes {
        match node {
            Node::Literal { id, value: Value::String(s) } => {
                strings.push((id.clone(), s.as_bytes().to_vec()));
            }
            Node::Literal { id, value: Value::Int(n) } => {
                constants.insert(id.clone(), *n);
            }
            _ => {}
        }
    }
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let mut type_section = Vec::new();
    type_section.extend(encode_uleb128(2)); // 2 types
    
    // Type 0: () -> i32 (main)
    type_section.push(TYPE_FUNC);
    type_section.extend(encode_uleb128(0));
    type_section.extend(encode_uleb128(1));
    type_section.push(TYPE_I32);
    
    // Type 1: (i32, i32) -> () (print - imported from host)
    type_section.push(TYPE_FUNC);
    type_section.extend(encode_uleb128(2)); // 2 params
    type_section.push(TYPE_I32); // ptr
    type_section.push(TYPE_I32); // len
    type_section.extend(encode_uleb128(0)); // 0 results
    
    wasm.extend(encode_section(SECTION_TYPE, &type_section));
    
    // Import section: import print from host
    let mut import_section = Vec::new();
    import_section.extend(encode_uleb128(1)); // 1 import
    import_section.extend(encode_string("env"));
    import_section.extend(encode_string("print"));
    import_section.push(0x00); // func import
    import_section.extend(encode_uleb128(1)); // type index 1
    wasm.extend(encode_section(SECTION_IMPORT, &import_section));
    
    // Function section
    let mut func_section = Vec::new();
    func_section.extend(encode_uleb128(1)); // 1 function
    func_section.extend(encode_uleb128(0)); // type index 0
    wasm.extend(encode_section(SECTION_FUNCTION, &func_section));
    
    // Memory section
    let mut mem_section = Vec::new();
    mem_section.extend(encode_uleb128(1));
    mem_section.push(0x00);
    mem_section.extend(encode_uleb128(1));
    wasm.extend(encode_section(SECTION_MEMORY, &mem_section));
    
    // Export section
    let mut export_section = Vec::new();
    export_section.extend(encode_uleb128(2));
    export_section.extend(encode_string("main"));
    export_section.push(0x00);
    export_section.extend(encode_uleb128(1)); // func index 1 (after import)
    export_section.extend(encode_string("memory"));
    export_section.push(0x02);
    export_section.extend(encode_uleb128(0));
    wasm.extend(encode_section(SECTION_EXPORT, &export_section));
    
    // Code section
    let mut code_section = Vec::new();
    code_section.extend(encode_uleb128(1));
    
    let mut func_body = Vec::new();
    func_body.extend(encode_uleb128(0)); // 0 locals
    
    // Generate code for print operations
    let mut data_offset = 0u32;
    for node in &graph.nodes {
        if let Node::Operation { op: Op::Prt, args, .. } = node {
            if let Some(Arg::Ref(str_ref)) = args.first() {
                if let Some((_, data)) = strings.iter().find(|(id, _)| id == str_ref) {
                    // call print(offset, len)
                    func_body.push(OP_I32_CONST);
                    func_body.extend(encode_sleb128(data_offset as i64));
                    func_body.push(OP_I32_CONST);
                    func_body.extend(encode_sleb128(data.len() as i64));
                    func_body.push(OP_CALL);
                    func_body.extend(encode_uleb128(0)); // import index 0
                    data_offset += data.len() as u32;
                }
            }
        }
    }
    
    // Return 0
    func_body.push(OP_I32_CONST);
    func_body.extend(encode_sleb128(0));
    func_body.push(OP_END);
    
    code_section.extend(encode_uleb128(func_body.len() as u64));
    code_section.extend(func_body);
    wasm.extend(encode_section(SECTION_CODE, &code_section));
    
    // Data section: string data
    if !strings.is_empty() {
        let mut data_section = Vec::new();
        data_section.extend(encode_uleb128(strings.len() as u64));
        
        let mut offset = 0u32;
        for (_, data) in &strings {
            data_section.push(0x00); // active, memory 0
            // i32.const offset
            data_section.push(OP_I32_CONST);
            data_section.extend(encode_sleb128(offset as i64));
            data_section.push(OP_END);
            // data bytes
            data_section.extend(encode_uleb128(data.len() as u64));
            data_section.extend(data);
            offset += data.len() as u32;
        }
        
        wasm.extend(encode_section(SECTION_DATA, &data_section));
    }
    
    wasm
}
