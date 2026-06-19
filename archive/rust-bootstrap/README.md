# TAYNI Rust Compiler

[![Tests](https://img.shields.io/badge/Tests-322%20Passing-brightgreen.svg)](#testing)
[![Wasm](https://img.shields.io/badge/Wasm-100%25%20Conformance-brightgreen.svg)](#wasm)

The Rust-based TAYNI compiler generates native executables directly from `.tayni` source files.

## Features

### Compilation Targets

| Target | Status | Output |
|--------|--------|--------|
| Windows PE | ✅ Verified | `.exe` |
| Linux ELF | ✅ Verified | binary |
| WebAssembly | ✅ 100% Conformance | `.wasm` |
| WASI | ✅ Implemented | `.wasm` |

### Language Features (v1.5)

- Functions: `fn`, `return`
- Variables: `let` (mutable), `LET` (immutable)
- Control flow: `if`, `else`, `loop`, `while`, `match`
- Capabilities: `cap:net`, `cap:fs`, `cap:env`, `cap:proc`, `cap:time`
- Types: `int`, `float`, `bool`, `str`, `array`, `map`

### Built-in Operations

| Category | Functions |
|----------|-----------|
| Output | `PRT`, `PRTLN`, `PRTERR` |
| JSON | `JSON.encode`, `JSON.decode` |
| Network | `TCP.*`, `HTTP.*` (requires `cap:net`) |
| File | `File.*`, `Dir.*` (requires `cap:fs`) |
| Environment | `Env.*` (requires `cap:env`) |

## Build

```bash
cargo build --release
```

## Usage

```bash
# Compile to Windows PE
./target/release/tayni-c program.tayni -o program.exe

# Compile to Linux ELF
./target/release/tayni-c program.tayni --target elf -o program

# Compile to WebAssembly
./target/release/tayni-c program.tayni --target wasm -o program.wasm

# Compile to WASI
./target/release/tayni-c program.tayni --target wasi -o program.wasm
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_pe_generation

# Run Wasm conformance tests
node wasm-conformance-test.js
```

### Test Results

```
Tests: 322 passing
Wasm Conformance: 100%
```

## Project Structure

```
rust-bootstrap/
├── src/
│   └── lib.rs          # Main compiler library
├── pe.rs               # Windows PE generator
├── elf.rs              # Linux ELF generator
├── wasm.rs             # WebAssembly generator
├── wasi.rs             # WASI generator
├── ir.rs               # Intermediate representation
├── parser.rs           # TAYNI parser
├── modules.rs          # Module resolver
├── examples/           # Test programs
│   ├── test_elf.rs
│   ├── test_wasm_gen.rs
│   └── test_wasi_gen.rs
├── wasm-conformance-test.js  # Wasm test suite
└── benchmarks/         # Performance benchmarks
```

## Wasm Conformance

All generated Wasm modules pass validation:

```
✓ wasm_minimal    37 bytes   wasm-tools validate ✓
✓ wasm_const42    41 bytes   wasm-tools validate ✓
✓ wasm_add        41 bytes   wasm-tools validate ✓
✓ wasm_factorial  60 bytes   wasm-tools validate ✓
✓ wasm_memory     79 bytes   wasm-tools validate ✓
✓ wasi_hello     198 bytes   wasm-tools validate ✓

Conformance: 100%
```

## Binary Sizes

| Program | PE (Windows) | ELF (Linux) | Wasm |
|---------|--------------|-------------|------|
| Hello World | 2.1KB | 1.8KB | 37 bytes |
| HTTP Server | 10.5KB | 9.2KB | N/A |
| TCP Echo | 8.2KB | 7.1KB | N/A |

## Architecture

```
Source (.tayni)
     │
     ▼
  Parser
     │
     ▼
    IR (Intermediate Representation)
     │
     ├──► PE Generator ──► Windows .exe
     │
     ├──► ELF Generator ──► Linux binary
     │
     ├──► Wasm Generator ──► .wasm
     │
     └──► WASI Generator ──► .wasm (with WASI imports)
```

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## License

MIT License

---

*TAYNI Rust Compiler - Part of the TAYNI project by NELAIA*
