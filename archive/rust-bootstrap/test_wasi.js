// Test WASI modules with Node.js WASI support
const fs = require('fs');
const path = require('path');
const { WASI } = require('wasi');

async function testWasiModule(name, wasmPath) {
    console.log(`\n[${name}]`);
    
    try {
        if (!fs.existsSync(wasmPath)) {
            console.log(`  SKIP: File not found`);
            return { status: 'skip', name };
        }
        
        const wasmBuffer = fs.readFileSync(wasmPath);
        console.log(`  Size: ${wasmBuffer.length} bytes`);
        
        // Compile
        const module = await WebAssembly.compile(wasmBuffer);
        console.log(`  Compile: OK`);
        
        // Check imports
        const imports = WebAssembly.Module.imports(module);
        console.log(`  Imports: ${imports.map(i => i.module + '.' + i.name).join(', ')}`);
        
        // Check exports
        const exports = WebAssembly.Module.exports(module);
        console.log(`  Exports: ${exports.map(e => e.name).join(', ')}`);
        
        // Create WASI instance
        const wasi = new WASI({
            version: 'preview1',
            args: ['wasm-program'],
            env: {},
            preopens: {
                '/': '.'
            }
        });
        
        // Instantiate with WASI imports
        const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
        console.log(`  Instantiate: OK`);
        
        // Run _start if it exists
        if (instance.exports._start) {
            console.log(`  Running _start...`);
            try {
                wasi.start(instance);
                console.log(`  Execution: OK`);
                return { status: 'pass', name };
            } catch (e) {
                if (e.message && e.message.includes('exit')) {
                    // Normal exit
                    console.log(`  Execution: OK (exited)`);
                    return { status: 'pass', name };
                }
                throw e;
            }
        } else {
            console.log(`  No _start export`);
            return { status: 'pass', name, note: 'no _start' };
        }
        
    } catch (e) {
        console.log(`  ERROR: ${e.message}`);
        return { status: 'error', name, error: e.message };
    }
}

async function main() {
    console.log('=== TAYNI WASI Test Suite ===');
    console.log(`Node.js ${process.version}`);
    
    const results = [];
    const dir = __dirname;
    
    // Test WASI hello
    results.push(await testWasiModule(
        'WASI Hello',
        path.join(dir, 'wasi_hello.wasm')
    ));
    
    // Summary
    console.log('\n' + '='.repeat(40));
    console.log('SUMMARY');
    console.log('='.repeat(40));
    
    const passed = results.filter(r => r.status === 'pass').length;
    const failed = results.filter(r => r.status === 'fail').length;
    const errors = results.filter(r => r.status === 'error').length;
    const skipped = results.filter(r => r.status === 'skip').length;
    
    console.log(`  PASS:  ${passed}`);
    console.log(`  FAIL:  ${failed}`);
    console.log(`  ERROR: ${errors}`);
    console.log(`  SKIP:  ${skipped}`);
    
    if (errors > 0) {
        console.log('\nErrors:');
        for (const r of results.filter(r => r.status === 'error')) {
            console.log(`  - ${r.name}: ${r.error}`);
        }
    }
}

main().catch(console.error);
