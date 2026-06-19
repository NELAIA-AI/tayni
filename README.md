# TAYNI

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Wasm Conformance](https://img.shields.io/badge/Wasm-100%25%20Conformance-brightgreen.svg)](#wasm-conformance)
[![Tests](https://img.shields.io/badge/Tests-98%20Passing-brightgreen.svg)](#testing)
[![Token Efficiency](https://img.shields.io/badge/Tokens-64%25%20Reduction-blue.svg)](#benchmarks)
[![Binary Size](https://img.shields.io/badge/HTTP%20Server-10.5KB-blue.svg)](#benchmarks)
[![Targets](https://img.shields.io/badge/Targets-PE%20%7C%20ELF%20%7C%20Wasm%20%7C%20WASI-purple.svg)](#compilation-targets)

**TAYNI** is an AI-first programming language designed for token-efficient code generation. It produces standalone executables with zero external dependencies.

## Key Metrics

| Metric | Value | Comparison |
|--------|-------|------------|
| Token Reduction | **64%** | vs Python/JavaScript |
| HTTP Server Binary | **10.5KB** | vs 2MB+ (Go), 300KB+ (Rust) |
| Dependencies | **0** | Zero external deps |
| Wasm Conformance | **100%** | wasm-tools validated |

## Quick Start

```bash
# Download from releases
# https://github.com/NELAIA-AI/tayni/releases

# Compile a TAYNI program
tayni build program.tayni -o program.exe

# Or compile to different targets
tayni build program.tayni --target elf    # Linux
tayni build program.tayni --target wasm   # WebAssembly
tayni build program.tayni --target wasi   # WASI
```

## Language Syntax (v1.5)

```tayni
// Capabilities declared at file start
cap:net

fn main() {
    let server = HTTP.listen(8080)
    PRT("Server running on :8080")
    
    server.route("/", fn(req) {
        Response.json({"status": "ok", "message": "Hello from TAYNI"})
    })
    
    server.route("/health", fn(req) {
        Response.text("healthy")
    })
}
```

### Key Syntax Rules

| Feature | TAYNI | Python/JS Equivalent |
|---------|-------|---------------------|
| Functions | `fn name() {}` | `def name():` / `function name() {}` |
| Variables | `let x = 5` | `x = 5` / `let x = 5` |
| Immutable | `LET x = 5` | `x = 5` (convention) / `const x = 5` |
| Output | `PRT("hello")` | `print("hello")` / `console.log("hello")` |
| Null | `nil` | `None` / `null` |
| Capabilities | `cap:net` | `import socket` / `require('net')` |

## Compilation Targets

| Target | Architecture | Status | Output |
|--------|--------------|--------|--------|
| Windows PE | x86-64 | ✅ Verified | `.exe` |
| Linux ELF | x86-64 | ✅ Verified | binary |
| WebAssembly | wasm32 | ✅ 100% Conformance | `.wasm` |
| WASI | wasm32-wasi | ✅ Implemented | `.wasm` |
| WASI Preview 2 | wasm32-wasip2 | ✅ Implemented | `.wasm` |
| macOS Mach-O | x86-64/ARM64 | 🔄 In Progress | binary |
| Linux ELF | ARM64 | 🔄 In Progress | binary |

## Capabilities (Security Model)

TAYNI uses capability-based security. Permissions must be declared at file start:

```tayni
cap:net   // TCP/HTTP networking
cap:fs    // File system access
cap:env   // Environment variables
cap:proc  // Process management
cap:time  // Time operations
```

Code without the required capability will fail at compile time.

## Built-in Functions

### Core
- `PRT(value)` - Print to stdout
- `PRTLN(value)` - Print with newline
- `PRTERR(value)` - Print to stderr
- `len(collection)` - Get length
- `JSON.encode(data)` / `JSON.decode(str)` - JSON handling

### Networking (requires `cap:net`)
- `HTTP.listen(port)` - Start HTTP server
- `HTTP.get(url)` / `HTTP.post(url, body)` - HTTP client
- `TCP.connect(addr)` / `TCP.listen(addr)` - Raw TCP

### File System (requires `cap:fs`)
- `File.read(path)` / `File.write(path, content)` - File I/O
- `File.exists(path)` / `File.delete(path)` - File operations

### Environment (requires `cap:env`)
- `Env.get(key)` / `Env.set(key, value)` - Environment variables

## Benchmarks

### Token Consumption (HTTP Server)

| Language | Tokens | Reduction |
|----------|--------|-----------|
| Python | 847 | baseline |
| JavaScript | 892 | -5% |
| Go | 634 | 25% |
| Rust | 1,247 | -47% |
| **TAYNI** | **298** | **64%** |

### Binary Size (HTTP Server)

| Language | Size | Reduction |
|----------|------|-----------|
| Go | 2.1MB | baseline |
| Rust | 312KB | 85% |
| C | 18KB | 99% |
| Zig | 8.5KB | 99.6% |
| **TAYNI** | **10.5KB** | **99.5%** |

## Wasm Conformance

All TAYNI-generated Wasm modules pass validation:

```
✓ wasm_minimal    37 bytes   wasm-tools validate ✓
✓ wasm_const42    41 bytes   wasm-tools validate ✓
✓ wasm_add        41 bytes   wasm-tools validate ✓
✓ wasm_factorial  60 bytes   wasm-tools validate ✓
✓ wasm_memory     79 bytes   wasm-tools validate ✓
✓ wasi_hello     198 bytes   wasm-tools validate ✓

Conformance: 100%
```

## Project Structure

```
tayni-core/
├── archive/rust-bootstrap/   # Rust compiler implementation
│   ├── lib.rs               # Main compiler library
│   ├── pe.rs                # Windows PE generator
│   ├── elf.rs               # Linux ELF generator
│   ├── wasm.rs              # WebAssembly generator
│   ├── wasi.rs              # WASI generator
│   ├── wasi_p2.rs           # WASI Preview 2 (filesystem, sockets)
│   ├── wasi_http.rs         # WASI HTTP for serverless
│   ├── json.rs              # JSON parser (RFC 8259)
│   ├── pkg.rs               # Package manager (semver, manifests)
│   ├── http_client.rs       # HTTP/1.1 client
│   ├── arm64.rs             # ARM64 instruction encoder
│   └── dwarf.rs             # DWARF debug info generator
├── tools/
│   ├── lsp/                 # Language Server Protocol
│   └── vscode-extension/    # VS Code extension
├── docs/
│   ├── paper/               # arXiv paper
│   ├── blog/                # Technical blog posts
│   └── *.md                 # Specifications
├── examples/
│   ├── v1.5/                # 30+ example programs
│   └── demos/               # Cloudflare/Deno demos
└── stdlib/                  # Standard library specs
```

## Tooling

### VS Code Extension

```bash
cd tools/vscode-extension
npm install
npm run package
# Install tayni-0.1.0.vsix
```

Features: Syntax highlighting, snippets, LSP integration

### Language Server (LSP)

```bash
cd tools/lsp
cargo build --release
# Configure in VS Code settings
```

Features: Diagnostics, hover, completion, go-to-definition

## Building from Source

```bash
# Clone repository
git clone https://github.com/NELAIA-AI/tayni.git
cd tayni/archive/rust-bootstrap

# Build compiler
cargo build --release

# Run tests
cargo test

# Compile a program
./target/release/tayni-c program.tayni -o program.exe
```

## For AI Agents

TAYNI is designed for AI code generation. Machine-readable resources:

- **Context**: `https://nelaia.ai/api/context.json`
- **Syntax**: `https://nelaia.ai/api/tayni/syntax.json`
- **Examples**: `https://nelaia.ai/api/tayni/examples.json`
- **Common Mistakes**: `https://nelaia.ai/api/tayni/common-mistakes.json`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**TAYNI** - AI-first programming language by [NELAIA](https://nelaia.ai)
