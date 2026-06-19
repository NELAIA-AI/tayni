#!/usr/bin/env node
/**
 * TAYNI Wasm Conformance Test Suite
 * 
 * Tests TAYNI-generated Wasm modules against:
 * 1. wasm-tools validate (structural validation)
 * 2. Node.js WebAssembly API (runtime validation)
 * 3. Expected behavior verification
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const WASM_DIR = __dirname;
const RESULTS = {
    passed: 0,
    failed: 0,
    skipped: 0,
    tests: []
};

// Test definitions
const TESTS = [
    {
        name: 'wasm_minimal',
        file: 'wasm_minimal.wasm',
        description: 'Minimal valid Wasm module',
        validate: true,
        instantiate: true,
        exports: []
    },
    {
        name: 'wasm_const42',
        file: 'wasm_const42.wasm',
        description: 'Function returning constant 42',
        validate: true,
        instantiate: true,
        exports: ['getValue'],
        tests: [
            { fn: 'getValue', args: [], expected: 42 }
        ]
    },
    {
        name: 'wasm_add',
        file: 'wasm_add.wasm',
        description: 'Addition function',
        validate: true,
        instantiate: true,
        exports: ['add'],
        tests: [
            { fn: 'add', args: [2, 3], expected: 5 },
            { fn: 'add', args: [0, 0], expected: 0 },
            { fn: 'add', args: [-1, 1], expected: 0 },
            { fn: 'add', args: [100, 200], expected: 300 }
        ]
    },
    {
        name: 'wasm_factorial',
        file: 'wasm_factorial.wasm',
        description: 'Recursive factorial function',
        validate: true,
        instantiate: true,
        exports: ['factorial'],
        tests: [
            { fn: 'factorial', args: [0], expected: 1 },
            { fn: 'factorial', args: [1], expected: 1 },
            { fn: 'factorial', args: [5], expected: 120 },
            { fn: 'factorial', args: [10], expected: 3628800 }
        ]
    },
    {
        name: 'wasm_memory',
        file: 'wasm_memory.wasm',
        description: 'Module with linear memory',
        validate: true,
        instantiate: true,
        checkMemory: true
    },
    {
        name: 'wasi_hello',
        file: 'wasi_hello.wasm',
        description: 'WASI hello world',
        validate: true,
        instantiate: false, // Requires WASI imports
        isWasi: true
    }
];

// Color output
const colors = {
    green: (s) => `\x1b[32m${s}\x1b[0m`,
    red: (s) => `\x1b[31m${s}\x1b[0m`,
    yellow: (s) => `\x1b[33m${s}\x1b[0m`,
    cyan: (s) => `\x1b[36m${s}\x1b[0m`,
    bold: (s) => `\x1b[1m${s}\x1b[0m`
};

function log(msg) {
    console.log(msg);
}

function checkWasmTools() {
    try {
        execSync('wasm-tools --version', { stdio: 'pipe' });
        return true;
    } catch {
        return false;
    }
}

function validateWithWasmTools(wasmPath) {
    try {
        execSync(`wasm-tools validate "${wasmPath}"`, { stdio: 'pipe' });
        return { valid: true };
    } catch (e) {
        return { valid: false, error: e.stderr?.toString() || e.message };
    }
}

function validateWithNode(wasmPath) {
    try {
        const buffer = fs.readFileSync(wasmPath);
        const valid = WebAssembly.validate(buffer);
        return { valid };
    } catch (e) {
        return { valid: false, error: e.message };
    }
}

async function instantiateModule(wasmPath, imports = {}) {
    try {
        const buffer = fs.readFileSync(wasmPath);
        const module = await WebAssembly.compile(buffer);
        const instance = await WebAssembly.instantiate(module, imports);
        return { success: true, instance, module };
    } catch (e) {
        return { success: false, error: e.message };
    }
}

function runFunctionTest(instance, testCase) {
    try {
        const fn = instance.exports[testCase.fn];
        if (!fn) {
            return { pass: false, error: `Function ${testCase.fn} not found` };
        }
        const result = fn(...testCase.args);
        if (result === testCase.expected) {
            return { pass: true, result };
        } else {
            return { pass: false, expected: testCase.expected, got: result };
        }
    } catch (e) {
        return { pass: false, error: e.message };
    }
}

async function runTest(test) {
    const wasmPath = path.join(WASM_DIR, test.file);
    const result = {
        name: test.name,
        description: test.description,
        file: test.file,
        checks: []
    };

    // Check file exists
    if (!fs.existsSync(wasmPath)) {
        result.status = 'skipped';
        result.reason = 'File not found';
        return result;
    }

    const fileSize = fs.statSync(wasmPath).size;
    result.fileSize = fileSize;

    // 1. Structural validation with wasm-tools
    if (test.validate && checkWasmTools()) {
        const wasmToolsResult = validateWithWasmTools(wasmPath);
        result.checks.push({
            name: 'wasm-tools validate',
            pass: wasmToolsResult.valid,
            error: wasmToolsResult.error
        });
    }

    // 2. Node.js WebAssembly.validate
    if (test.validate) {
        const nodeValidate = validateWithNode(wasmPath);
        result.checks.push({
            name: 'WebAssembly.validate',
            pass: nodeValidate.valid,
            error: nodeValidate.error
        });
    }

    // 3. Instantiation
    if (test.instantiate) {
        const instantiateResult = await instantiateModule(wasmPath);
        result.checks.push({
            name: 'WebAssembly.instantiate',
            pass: instantiateResult.success,
            error: instantiateResult.error
        });

        // 4. Export verification
        if (instantiateResult.success && test.exports) {
            const exports = Object.keys(instantiateResult.instance.exports);
            const hasAllExports = test.exports.every(e => exports.includes(e));
            result.checks.push({
                name: 'Exports present',
                pass: hasAllExports,
                expected: test.exports,
                got: exports
            });
        }

        // 5. Function tests
        if (instantiateResult.success && test.tests) {
            for (const tc of test.tests) {
                const fnResult = runFunctionTest(instantiateResult.instance, tc);
                result.checks.push({
                    name: `${tc.fn}(${tc.args.join(', ')}) == ${tc.expected}`,
                    pass: fnResult.pass,
                    expected: tc.expected,
                    got: fnResult.result,
                    error: fnResult.error
                });
            }
        }

        // 6. Memory check
        if (instantiateResult.success && test.checkMemory) {
            const hasMemory = instantiateResult.instance.exports.memory instanceof WebAssembly.Memory;
            result.checks.push({
                name: 'Has linear memory',
                pass: hasMemory
            });
        }
    }

    // Determine overall status
    const allPassed = result.checks.every(c => c.pass);
    result.status = allPassed ? 'passed' : 'failed';

    return result;
}

async function main() {
    log(colors.bold('\n=== TAYNI Wasm Conformance Test Suite ===\n'));

    const hasWasmTools = checkWasmTools();
    if (hasWasmTools) {
        log(colors.green('✓ wasm-tools available'));
    } else {
        log(colors.yellow('⚠ wasm-tools not found, using Node.js validation only'));
    }
    log('');

    for (const test of TESTS) {
        const result = await runTest(test);
        RESULTS.tests.push(result);

        const icon = result.status === 'passed' ? colors.green('✓') :
                     result.status === 'skipped' ? colors.yellow('○') :
                     colors.red('✗');
        
        log(`${icon} ${colors.bold(result.name)} - ${result.description}`);
        
        if (result.fileSize) {
            log(`  File size: ${result.fileSize} bytes`);
        }

        if (result.status === 'skipped') {
            log(colors.yellow(`  Skipped: ${result.reason}`));
            RESULTS.skipped++;
        } else {
            for (const check of result.checks) {
                const checkIcon = check.pass ? colors.green('  ✓') : colors.red('  ✗');
                log(`${checkIcon} ${check.name}`);
                if (!check.pass && check.error) {
                    log(colors.red(`    Error: ${check.error}`));
                }
                if (!check.pass && check.expected !== undefined) {
                    log(colors.red(`    Expected: ${check.expected}, Got: ${check.got}`));
                }
            }
            
            if (result.status === 'passed') {
                RESULTS.passed++;
            } else {
                RESULTS.failed++;
            }
        }
        log('');
    }

    // Summary
    log(colors.bold('=== Summary ==='));
    log(`Total: ${TESTS.length}`);
    log(colors.green(`Passed: ${RESULTS.passed}`));
    log(colors.red(`Failed: ${RESULTS.failed}`));
    log(colors.yellow(`Skipped: ${RESULTS.skipped}`));

    // Calculate conformance percentage
    const tested = RESULTS.passed + RESULTS.failed;
    const conformance = tested > 0 ? ((RESULTS.passed / tested) * 100).toFixed(1) : 0;
    log(`\nConformance: ${conformance}%`);

    // Write JSON report
    const reportPath = path.join(WASM_DIR, 'conformance-report.json');
    fs.writeFileSync(reportPath, JSON.stringify({
        timestamp: new Date().toISOString(),
        summary: {
            total: TESTS.length,
            passed: RESULTS.passed,
            failed: RESULTS.failed,
            skipped: RESULTS.skipped,
            conformance: parseFloat(conformance)
        },
        tests: RESULTS.tests
    }, null, 2));
    log(`\nReport saved to: ${reportPath}`);

    process.exit(RESULTS.failed > 0 ? 1 : 0);
}

main().catch(e => {
    console.error('Test suite error:', e);
    process.exit(1);
});
