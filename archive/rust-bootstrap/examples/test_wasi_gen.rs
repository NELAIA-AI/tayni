//! Test WASI module generation

use tayni_c::wasi;

use std::fs;

fn main() {
    println!("=== TAYNI WASI Generator Test ===\n");
    
    // Generate WASI hello world
    println!("Generating WASI Hello World...");
    let wasm = wasi::generate_wasi_hello();
    fs::write("wasi_hello.wasm", &wasm).expect("Failed to write");
    println!("  Generated: wasi_hello.wasm ({} bytes)", wasm.len());
    
    // Verify magic
    assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6D], "Invalid magic");
    println!("  Magic: OK");
    
    println!("\nTo test:");
    println!("  wasmtime wasi_hello.wasm");
    println!("  # or");
    println!("  node --experimental-wasi-unstable-preview1 test_wasi.js");
}
