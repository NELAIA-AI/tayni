// Comprehensive Wasm test suite for TAYNI-generated modules
const fs = require('fs');
const path = require('path');

async function testModule(name, wasmPath, testFn, importObject = {}) {
    console.log(`\n[${name}]`);
    
    try {
        if (!fs.existsSync(wasmPath)) {
            console.log(`  SKIP: File not found`);
            return { status: 'skip', name };
        }
        
        const wasmBuffer = fs.readFileSync(wasmPath);
        console.log(`  Size: ${wasmBuffer.length} bytes`);
        
        // Validate and compile
        const module = await WebAssembly.compile(wasmBuffer);
        console.log(`  Compile: OK`);
        
        // Instantiate with provided imports
        const instance = await WebAssembly.instantiate(module, importObject);
        console.log(`  Instantiate: OK`);
        
        // Run test function
        const result = testFn(instance.exports);
        if (result.pass) {
            console.log(`  Test: PASS - ${result.message}`);
            return { status: 'pass', name };
        } else {
            console.log(`  Test: FAIL - ${result.message}`);
            return { status: 'fail', name, error: result.message };
        }
        
    } catch (e) {
        console.log(`  ERROR: ${e.message}`);
        return { status: 'error', name, error: e.message };
    }
}

async function main() {
    console.log('=== TAYNI Wasm Conformance Test Suite ===');
    console.log('Testing Wasm module generation and execution\n');
    
    const results = [];
    const dir = __dirname;
    
    // Test 1: Minimal module
    results.push(await testModule(
        'Minimal (return 0)',
        path.join(dir, 'wasm_minimal.wasm'),
        (exports) => {
            const result = exports.main();
            return {
                pass: result === 0,
                message: `main() = ${result}, expected 0`
            };
        }
    ));
    
    // Test 2: Return constant 42
    results.push(await testModule(
        'Constant 42',
        path.join(dir, 'wasm_const42.wasm'),
        (exports) => {
            const result = exports.getValue();
            return {
                pass: result === 42,
                message: `getValue() = ${result}, expected 42`
            };
        }
    ));
    
    // Test 3: Add function
    results.push(await testModule(
        'Add function',
        path.join(dir, 'wasm_add.wasm'),
        (exports) => {
            const tests = [
                [0, 0, 0],
                [1, 2, 3],
                [100, 200, 300],
                [-5, 10, 5],
                [2147483647, 1, -2147483648], // overflow
            ];
            for (const [a, b, expected] of tests) {
                const result = exports.add(a, b);
                if (result !== expected) {
                    return {
                        pass: false,
                        message: `add(${a}, ${b}) = ${result}, expected ${expected}`
                    };
                }
            }
            return { pass: true, message: `All ${tests.length} test cases passed` };
        }
    ));
    
    // Test 4: Factorial
    results.push(await testModule(
        'Factorial (recursive)',
        path.join(dir, 'wasm_factorial.wasm'),
        (exports) => {
            const tests = [
                [0, 1],
                [1, 1],
                [5, 120],
                [10, 3628800],
            ];
            for (const [n, expected] of tests) {
                const result = exports.factorial(n);
                if (result !== expected) {
                    return {
                        pass: false,
                        message: `factorial(${n}) = ${result}, expected ${expected}`
                    };
                }
            }
            return { pass: true, message: `All ${tests.length} test cases passed` };
        }
    ));
    
    // Test 5: Memory operations
    results.push(await testModule(
        'Memory store/load',
        path.join(dir, 'wasm_memory.wasm'),
        (exports) => {
            // Store and load at various addresses
            const tests = [
                [0, 42],
                [4, 100],
                [8, -1],
                [100, 999],
            ];
            for (const [addr, value] of tests) {
                exports.store(addr, value);
                const loaded = exports.load(addr);
                if (loaded !== value) {
                    return {
                        pass: false,
                        message: `store(${addr}, ${value}); load(${addr}) = ${loaded}`
                    };
                }
            }
            return { pass: true, message: `All ${tests.length} store/load tests passed` };
        }
    ));
    
    // Test existing TAYNI wasm (requires imports)
    results.push(await testModule(
        'TAYNI test-wasm.wasm (with imports)',
        path.join(dir, 'test-wasm.wasm'),
        (exports) => {
            if (exports.main) {
                const result = exports.main();
                return {
                    pass: result === 0,
                    message: `main() = ${result}`
                };
            }
            return { pass: true, message: 'Module loaded successfully' };
        },
        { env: { print: (ptr, len) => {} } } // Provide mock imports
    ));
    
    // Summary
    console.log('\n' + '='.repeat(50));
    console.log('SUMMARY');
    console.log('='.repeat(50));
    
    const passed = results.filter(r => r.status === 'pass').length;
    const failed = results.filter(r => r.status === 'fail').length;
    const errors = results.filter(r => r.status === 'error').length;
    const skipped = results.filter(r => r.status === 'skip').length;
    
    console.log(`  PASS:    ${passed}`);
    console.log(`  FAIL:    ${failed}`);
    console.log(`  ERROR:   ${errors}`);
    console.log(`  SKIP:    ${skipped}`);
    console.log(`  TOTAL:   ${results.length}`);
    
    if (failed > 0 || errors > 0) {
        console.log('\nFailed/Error tests:');
        for (const r of results.filter(r => r.status === 'fail' || r.status === 'error')) {
            console.log(`  - ${r.name}: ${r.error}`);
        }
        process.exit(1);
    } else {
        console.log('\nAll tests passed!');
        process.exit(0);
    }
}

main().catch(e => {
    console.error('Fatal error:', e);
    process.exit(1);
});
