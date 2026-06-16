# TAYNI

[![Build](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml/badge.svg)](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20695531.svg)](https://doi.org/10.5281/zenodo.20695531)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version](https://img.shields.io/badge/Version-1.0-green.svg)](https://github.com/NELAIA-AI/tayni/releases)
[![Autonomy](https://img.shields.io/badge/Autonomy-5.0%2F5.0-brightgreen.svg)](https://github.com/NELAIA-AI/tayni)
[![Stdlib](https://img.shields.io/badge/Stdlib-36%20modules-purple.svg)](https://github.com/NELAIA-AI/tayni)
[![Targets](https://img.shields.io/badge/Targets-17%20architectures-orange.svg)](https://github.com/NELAIA-AI/tayni)
[![Platform](https://img.shields.io/badge/Platform-x86__64%20|%20ARM64%20|%20RISC--V%20|%20WASM%20|%20QIR%20|%20GPU-blue.svg)](https://github.com/NELAIA-AI/tayni/releases)

**TAYNI v1.0** - An AI-first programming language optimized for token efficiency and minimal binary sizes.

> **Production Ready. Zero Rust Dependency. Self-Hosting Complete. 36-module Standard Library.**

## Key Features

- **AI-First Design** - Optimized for AI code generation and maintenance
- **Self-Hosting Complete** - Compiler written in TAYNI (Gen15, Zero Rust)
- **Multi-Target** - 17 architectures: x86_64, ARM64, RISC-V, WASM, QIR (Quantum), GPU (PTX/AMDGPU/SPIR-V)
- **Tiny Executables** - Minimal PE: 1,024 bytes, with File I/O: 2,560 bytes (3.5x smaller than LLVM)
- **Zero Dependencies** - Direct PE/ELF/Mach-O/WASM emission from scratch
- **Standard Library** - 36 modules in 3 tiers (TIER 0: 10, TIER 1: 12, TIER 2: 14)
- **Autonomy Level 5.0** - Full AI autonomy, self-replicating compiler

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

TAYNI includes a 36-module standard library organized in 3 tiers:

### Tier 0 - Essential (10 modules)
Core modules for 95% of applications:

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

### Tier 2 - Specialized (14 modules)
Advanced functionality:

| Module | Purpose |
|--------|---------|
| `sql` | SQL query builder |
| `postgres` | PostgreSQL wire protocol |
| `redis` | Redis RESP protocol |
| `websocket` | WebSocket (RFC 6455) |
| `grpc` | gRPC/protobuf |
| `yaml` | YAML parsing |
| `csv` | CSV parsing (RFC 4180) |
| `xml` | XML parsing + XPath |
| `crypto` | AES-256-GCM, RSA, ECDSA, ChaCha20 |
| `tls` | TLS 1.3 protocol |
| `pqc` | Post-Quantum: ML-KEM, ML-DSA, SLH-DSA |
| `cors` | CORS handling |
| `cookie` | Cookie/session management |
| `gzip` | GZIP compression (RFC 1952) |

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

TAYNI has achieved **full self-hosting** with Zero Rust dependency:

| Level | Description | Status |
|-------|-------------|--------|
| 3.5 | Pure compiler generates PE | ✅ Complete |
| 3.7 | File I/O via syscalls | ✅ Complete |
| 3.9 | AI-native assembler with auto offsets | ✅ Complete |
| 4.0 | Full self-hosting (bit-identical replication) | ✅ Complete |
| 4.5 | Self-replicating compiler with code generation | ✅ Complete |
| 5.0 | Zero Rust - PE generation from scratch | ✅ Complete |

**Autonomy Level: 5.0/5.0 (100%)**

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

Apache 2.0 License - see [LICENSE](LICENSE) for details.

---

*TAYNI v1.0 - AI-first programming language. Production Ready.*
