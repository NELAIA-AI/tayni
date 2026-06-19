// Test TAYNI Wasm output with Node.js
const fs = require('fs');
const path = require('path');

async function testWasm(wasmPath) {
    console.log(`\nTesting: ${wasmPath}`);
    
    try {
        const wasmBuffer = fs.readFileSync(wasmPath);
        console.log(`  Size: ${wasmBuffer.length} bytes`);
        
        // Validate magic number
        const magic = wasmBuffer.slice(0, 4);
        if (magic[0] !== 0x00 || magic[1] !== 0x61 || magic[2] !== 0x73 || magic[3] !== 0x6d) {
            console.log('  ERROR: Invalid Wasm magic number');
            console.log(`  Got: ${Array.from(magic).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' ')}`);
            return false;
        }
        console.log('  Magic: OK (\\0asm)');
        
        // Validate version
        const version = wasmBuffer.readUInt32LE(4);
        console.log(`  Version: ${version}`);
        
        // Try to compile
        const module = await WebAssembly.compile(wasmBuffer);
        console.log('  Compile: OK');
        
        // Get exports info
        const exports = WebAssembly.Module.exports(module);
        console.log(`  Exports: ${exports.map(e => e.name + '(' + e.kind + ')').join(', ')}`);
        
        // Get imports info
        const imports = WebAssembly.Module.imports(module);
        if (imports.length > 0) {
            console.log(`  Imports: ${imports.map(i => i.module + '.' + i.name).join(', ')}`);
        }
        
        // Try to instantiate with mock imports
        const importObject = {
            env: {
                print: (ptr, len) => {
                    console.log(`  [print called: ptr=${ptr}, len=${len}]`);
                }
            }
        };
        
        try {
            const instance = await WebAssembly.instantiate(module, importObject);
            console.log('  Instantiate: OK');
            
            // Try to call main if it exists
            if (instance.exports.main) {
                const result = instance.exports.main();
                console.log(`  main() returned: ${result}`);
            }
            
            return true;
        } catch (e) {
            console.log(`  Instantiate: FAILED - ${e.message}`);
            return false;
        }
        
    } catch (e) {
        console.log(`  ERROR: ${e.message}`);
        return false;
    }
}

async function main() {
    console.log('=== TAYNI Wasm Conformance Test ===');
    
    const wasmFiles = [
        'test-wasm.wasm',
    ];
    
    let passed = 0;
    let failed = 0;
    
    for (const file of wasmFiles) {
        const fullPath = path.join(__dirname, file);
        if (fs.existsSync(fullPath)) {
            const ok = await testWasm(fullPath);
            if (ok) passed++; else failed++;
        } else {
            console.log(`\nSkipping ${file} (not found)`);
        }
    }
    
    console.log('\n=== Summary ===');
    console.log(`Passed: ${passed}`);
    console.log(`Failed: ${failed}`);
}

main().catch(console.error);
