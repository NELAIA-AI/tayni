//! Test WASI Preview 2 filesystem generation
//! 
//! Run with: cargo run --example test_wasi_p2

use tayni_c::wasi_p2::{
    generate_wasi_p2_fs, generate_wasi_p2_write_file,
    FsOperation, DescriptorFlags, OpenFlags,
};
use std::fs;

fn main() {
    println!("=== TAYNI WASI Preview 2 Filesystem Test ===\n");
    
    // Test 1: Simple file write
    println!("Test 1: Generate write file module");
    let wasm = generate_wasi_p2_write_file("output.txt", b"Hello from TAYNI WASI P2!\n");
    println!("  Generated {} bytes", wasm.len());
    
    // Verify Wasm header
    assert_eq!(&wasm[0..4], b"\0asm", "Invalid Wasm magic");
    assert_eq!(&wasm[4..8], &[1, 0, 0, 0], "Invalid Wasm version");
    println!("  ✓ Valid Wasm header");
    
    // Save to file
    fs::write("test_write.wasm", &wasm).expect("Failed to write wasm");
    println!("  ✓ Saved to test_write.wasm");
    
    // Test 2: Multiple operations
    println!("\nTest 2: Generate multi-operation module");
    let ops = vec![
        FsOperation::CreateDir { path: "testdir".to_string() },
        FsOperation::WriteFile { 
            path: "testdir/hello.txt".to_string(), 
            data: b"Hello World!\n".to_vec() 
        },
        FsOperation::WriteFile { 
            path: "testdir/data.json".to_string(), 
            data: br#"{"name": "TAYNI", "version": "1.5"}"#.to_vec() 
        },
    ];
    
    let wasm_multi = generate_wasi_p2_fs(&ops);
    println!("  Generated {} bytes for {} operations", wasm_multi.len(), ops.len());
    
    fs::write("test_multi.wasm", &wasm_multi).expect("Failed to write wasm");
    println!("  ✓ Saved to test_multi.wasm");
    
    // Test 3: File deletion
    println!("\nTest 3: Generate delete file module");
    let delete_ops = vec![
        FsOperation::DeleteFile { path: "temp.txt".to_string() },
    ];
    let wasm_delete = generate_wasi_p2_fs(&delete_ops);
    println!("  Generated {} bytes", wasm_delete.len());
    
    fs::write("test_delete.wasm", &wasm_delete).expect("Failed to write wasm");
    println!("  ✓ Saved to test_delete.wasm");
    
    // Test 4: Descriptor flags
    println!("\nTest 4: Descriptor flags");
    let read_only = DescriptorFlags::read_only();
    let write_only = DescriptorFlags::write_only();
    let read_write = DescriptorFlags::read_write();
    
    println!("  read_only bits:  0b{:06b}", read_only.to_bits());
    println!("  write_only bits: 0b{:06b}", write_only.to_bits());
    println!("  read_write bits: 0b{:06b}", read_write.to_bits());
    
    assert_eq!(read_only.to_bits(), 0b000001);
    assert_eq!(write_only.to_bits(), 0b000010);
    assert_eq!(read_write.to_bits(), 0b000011);
    println!("  ✓ Flags correct");
    
    // Test 5: Open flags
    println!("\nTest 5: Open flags");
    let mut open_flags = OpenFlags::default();
    println!("  default bits: 0b{:04b}", open_flags.to_bits());
    
    open_flags.create = true;
    open_flags.truncate = true;
    println!("  create+truncate bits: 0b{:04b}", open_flags.to_bits());
    assert_eq!(open_flags.to_bits(), 0b1001);
    println!("  ✓ Open flags correct");
    
    println!("\n=== All WASI P2 tests passed! ===");
    println!("\nGenerated files:");
    println!("  - test_write.wasm  ({} bytes)", fs::metadata("test_write.wasm").unwrap().len());
    println!("  - test_multi.wasm  ({} bytes)", fs::metadata("test_multi.wasm").unwrap().len());
    println!("  - test_delete.wasm ({} bytes)", fs::metadata("test_delete.wasm").unwrap().len());
    
    println!("\nTo run with Wasmtime (WASI P2 support):");
    println!("  wasmtime --wasi preview2 test_write.wasm");
}
