# TAYNI

[![Build](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml/badge.svg)](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20695531.svg)](https://doi.org/10.5281/zenodo.20695531)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.96+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS%20|%20WASM%20|%20RISC--V-blue.svg)](https://github.com/NELAIA-AI/tayni/releases)

**TAYNI Compiler v0.25** - An AI-first programming language optimized for token efficiency and minimal binary sizes.

> **Minimal syntax. Multi-target compilation. Zero dependencies. 43-module standard library.**

## Key Features

- **AI-First Design** - Optimized for AI code generation and maintenance
- **Multi-Target** - Windows, Linux, macOS, WASM, RISC-V, ARM64, Quantum (QIR), GPU (PTX/AMDGPU)
- **Tiny Executables** - Hello World: 145 bytes (Linux), 1KB (Windows)
- **Zero Dependencies** - Direct PE/ELF/Mach-O/WASM emission, no Clang required
- **Standard Library** - 43 modules in 3 tiers for common tasks
- **Self-Hosting** - Compiler written in TAYNI (bootstrap in progress)

## Quick Start

### 1. Download

```bash
# Linux x64
curl -LO https://github.com/NELAIA-AI/tayni/releases/latest/download/tayni-c-linux-x64
chmod +x tayni-c-linux-x64

# macOS Apple Silicon
curl -LO https://github.com/NELAIA-AI/tayni/releases/latest/download/tayni-c-macos-arm64
chmod +x tayni-c-macos-arm64

# Windows: Download tayni-c-windows-x64.exe from Releases
```

### 2. Hello World

```bash
cat > hello.tyn << 'EOF'
.msg: "Hello from TAYNI!\n"
.out: PRT .msg 19
!
EOF

./tayni-c-linux-x64 hello.tyn -o hello
./hello
# Output: Hello from TAYNI!
```

### 3. Using Standard Library

```bash
cat > server.tyn << 'EOF'
USE http
USE json

.port: 8080
.server: HTTP.LISTEN .port
.req: HTTP.ACCEPT .server
.body: JSON.ENCODE '{"status": "ok"}'
.resp: HTTP.RESPOND .req 200 .body
!
EOF

./tayni-c-linux-x64 server.tyn -o server
```

## Standard Library

TAYNI includes a 43-module standard library organized in 3 tiers:

### Tier 0 - Core (10 modules)
Essential modules for most programs:

| Module | Purpose |
|--------|---------|
| `args` | Command-line argument parsing |
| `base64` | Base64 encoding/decoding |
| `file` | File system operations |
| `http` | HTTP client/server |
| `json` | JSON parsing/encoding |
| `log` | Structured logging |
| `random` | Random number generation |
| `router` | HTTP routing |
| `string` | String manipulation |
| `url` | URL parsing |

### Tier 1 - Standard (12 modules)
Common utilities:

| Module | Purpose |
|--------|---------|
| `async` | Async/await patterns |
| `env` | Environment variables |
| `format` | String formatting |
| `hash` | Cryptographic hashes (SHA-256, etc.) |
| `jwt` | JSON Web Tokens |
| `path` | Path manipulation |
| `regex` | Regular expressions |
| `test` | Unit testing framework |
| `time` | Date/time operations |
| `timeout` | Timeout handling |
| `uuid` | UUID generation |
| `validation` | Input validation |

### Tier 2 - Extended (21 modules)
Specialized functionality:

| Module | Purpose |
|--------|---------|
| `cookie` | HTTP cookies |
| `cors` | CORS handling |
| `crypto` | Encryption (AES, RSA) |
| `csv` | CSV parsing |
| `gpu` | GPU computing (PTX/AMDGPU) |
| `grpc` | gRPC client/server |
| `gzip` | Compression |
| `mime` | MIME types |
| `mongodb` | MongoDB client |
| `postgres` | PostgreSQL client |
| `pqc` | Post-quantum cryptography |
| `quantum` | Quantum computing (QIR) |
| `redis` | Redis client |
| `retry` | Retry with backoff |
| `sql` | SQL query builder |
| `sqlite` | SQLite database |
| `tls` | TLS/SSL |
| `toml` | TOML parsing |
| `websocket` | WebSocket client/server |
| `xml` | XML parsing |
| `yaml` | YAML parsing |

## Multi-Target Compilation

TAYNI compiles to multiple targets from a single source:

```bash
# Classical CPU targets
tayni-c program.tyn --target=windows    # Windows PE
tayni-c program.tyn --target=linux      # Linux ELF
tayni-c program.tyn --target=macos      # macOS Mach-O
tayni-c program.tyn --target=wasm       # WebAssembly
tayni-c program.tyn --target=riscv64    # RISC-V 64-bit
tayni-c program.tyn --target=arm64      # ARM64 Linux

# Quantum targets (QIR native)
tayni-c quantum.tyn --target=qir        # Quantum IR (Azure, IonQ, Quantinuum)
tayni-c quantum.tyn --target=qir --export=qasm   # Export to OpenQASM
tayni-c quantum.tyn --target=qir --export=cirq   # Export to Cirq
tayni-c quantum.tyn --target=qir --export=quil   # Export to Quil

# GPU targets (PTX/AMDGPU native)
tayni-c gpu.tyn --target=ptx            # NVIDIA CUDA
tayni-c gpu.tyn --target=amdgpu         # AMD ROCm
tayni-c gpu.tyn --target=ptx --export=opencl  # Export to OpenCL
tayni-c gpu.tyn --target=ptx --export=spirv   # Export to SPIR-V
tayni-c gpu.tyn --target=ptx --export=wgsl    # Export to WebGPU
tayni-c gpu.tyn --target=ptx --export=metal   # Export to Metal
```

## Language Syntax

```tayni
-- Comments start with --
-- Bindings: .name: value or .name: OPERATION args

-- Constants
.x: 42
.msg: "Hello"

-- Operations
.sum: ADD .x 10
.product: MUL .sum 2

-- Output
.out: PRT .msg 5

-- Module imports
USE json
USE http

-- Program terminator
!
```

## Available Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEG` |
| Comparison | `EQ`, `NE`, `LT`, `GT`, `LE`, `GE` |
| Logic | `AND`, `OR`, `NOT` |
| Memory | `ALC`, `FRE`, `PUT`, `GET`, `CPY`, `CHR`, `SLN` |
| I/O | `PRT`, `INP`, `FOP`, `FRD`, `FWR`, `FCL` |
| String | `CAT`, `ITS`, `SBS`, `SCM` |
| Network | `TCP`, `UDP`, `BND`, `LST`, `ACC`, `XMT`, `RCV`, `CLS` |
| HTTP | `HTTP.LISTEN`, `HTTP.ACCEPT`, `HTTP.RESPOND`, `HTTP.GET`, `HTTP.POST` |
| SQL | `SQL.CONNECT`, `SQL.QUERY`, `SQL.EXEC`, `SQL.CLOSE` |
| JSON | `JSON.PARSE`, `JSON.ENCODE`, `JSON.GET`, `JSON.SET` |
| Quantum | `QH`, `QX`, `QCNOT`, `QM`, `QUBIT.ALLOC` |
| GPU | `GKERNEL`, `GLAUNCH`, `GALLOC`, `GH2D`, `GD2H` |

## Documentation

| Document | Purpose |
|----------|---------|
| [`docs/TAYNI-GUIDE-v0.22.md`](docs/TAYNI-GUIDE-v0.22.md) | Complete language guide |
| [`docs/TAYNI-REFERENCE-v0.22.md`](docs/TAYNI-REFERENCE-v0.22.md) | EBNF grammar + operator tables |
| [`docs/TAYNI-EXAMPLES-v0.22.md`](docs/TAYNI-EXAMPLES-v0.22.md) | Examples with dependency graphs |
| [`docs/TAYNI-SEMANTICS-v0.22.md`](docs/TAYNI-SEMANTICS-v0.22.md) | Type system + semantic rules |
| [`docs/TAYNI-TRAINING-DATA.jsonl`](docs/TAYNI-TRAINING-DATA.jsonl) | 100+ input/output pairs for AI training |
| [`llms.txt`](llms.txt) | AI discovery manifest |

## Project Structure

```
tayni-core/
├── src/                    # Rust compiler source
│   ├── main.rs            # CLI entry point
│   ├── parser.rs          # TAYNI parser
│   ├── ir.rs              # Intermediate representation
│   ├── emitter_pure.rs    # LLVM IR emitter
│   ├── pe.rs              # Windows PE generator
│   ├── elf.rs             # Linux ELF generator
│   ├── macho.rs           # macOS Mach-O generator
│   ├── wasm.rs            # WebAssembly generator
│   ├── riscv.rs           # RISC-V generator
│   ├── elf_arm64.rs       # ARM64 Linux generator
│   ├── qir.rs             # Quantum IR generator
│   ├── gpu.rs             # GPU (PTX/AMDGPU) generator
│   ├── modules.rs         # USE directive handler
│   ├── interface.rs       # Interface generation
│   ├── intent.rs          # Structured Intent (JSON→TAYNI)
│   └── tayni/             # Self-hosted compiler (bootstrap)
├── stdlib/                # Standard library (43 modules)
│   ├── tier0/            # Core modules (10)
│   ├── tier1/            # Standard modules (12)
│   └── tier2/            # Extended modules (21)
├── examples/              # Example programs
│   └── quantum/          # Quantum computing examples
├── docs/                  # Documentation
└── .github/workflows/     # CI/CD
```

## Building from Source

```bash
# Requires Rust 1.96+
cargo build --release

# Run tests
cargo test

# The compiler binary will be at target/release/tayni-c
```

## Self-Hosting Status

TAYNI is working towards self-hosting (compiler written in TAYNI):

- ✅ **v1.1**: File I/O, source analysis
- ✅ **v1.2**: PE header generation
- ✅ **v1.3**: CHR (char read), ITS (int-to-string), full PE headers
- ⏳ **v2.0**: Full parser in TAYNI (in progress)
- ⏳ **v3.0**: Self-compilation (bootstrap complete)

## Troubleshooting

### "Permission denied" on Linux/macOS
```bash
chmod +x tayni-c-linux-x64
```

### "E:PARSE: Unknown operation"
Check that you're using valid TAYNI operators. Use `--help` to see available options.

### "E:USE: Module not found"
Ensure the stdlib directory is in the same location as the compiler, or use `--stdlib-path`.

### Binary doesn't run on target OS
Use the correct binary for your platform or cross-compile with `--target`.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*TAYNI - AI-first programming language for efficient code generation*
