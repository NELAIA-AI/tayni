# TAYNI Public Roadmap

> **TAYNI**: AI-first programming language designed for optimal token efficiency and minimal binary sizes.

## Current Status: Alpha (Autonomy 5.0)

TAYNI has achieved **Zero Rust dependency** - the compiler can generate PE executables from scratch without any external tools.

---

## Completed

### Core Compiler
- **Native PE Generation**: Generates Windows PE executables byte-by-byte
- **AI-Native Syntax**: Optimized for token efficiency
- **Direct Syscalls**: No runtime dependencies
- **Self-Hosting**: TAYNI compiler written in TAYNI (Gen15)
- **Zero Rust**: PE generation from scratch (2,560 bytes)

### Standard Library - TIER 0 (10 modules)
Essential modules for 95% of applications:

| Module | Description |
|--------|-------------|
| `file` | File I/O operations |
| `string` | String manipulation |
| `json` | JSON parsing/encoding |
| `http` | HTTP parsing |
| `url` | URL handling |
| `router` | URL routing |
| `log` | Logging |
| `base64` | Base64 encoding |
| `random` | Random number generation |
| `args` | CLI arguments |

### Standard Library - TIER 1 (12 modules)
Common modules for enhanced functionality:

| Module | Description |
|--------|-------------|
| `env` | Environment variables |
| `path` | Path manipulation |
| `hash` | SHA256, HMAC, bcrypt |
| `time` | Time operations |
| `uuid` | UUID v4/v7 generation |
| `jwt` | JWT sign/verify |
| `regex` | Pattern matching |
| `format` | String formatting |
| `validation` | Input validation |
| `test` | Testing framework |
| `async` | Async primitives |
| `timeout` | Timeout handling |

---

## In Progress

### Standard Library - TIER 2 (14 modules)
Specialized modules:
- `sql`, `postgres`, `redis` - Database
- `websocket`, `grpc` - Protocols
- `crypto`, `tls`, `pqc` - Security
- `yaml`, `csv`, `xml` - Data formats
- `cors`, `cookie`, `gzip`, `retry` - Web utilities

---

## Planned

### Multi-Target Support (17 targets)

| Category | Targets |
|----------|---------|
| Classical | ARM64 Linux/Windows, RISC-V |
| Web | WASM, WASI |
| Quantum | QIR, OpenQASM, Cirq, Quil |
| GPU | PTX, AMDGPU, SPIR-V, Metal |

### Package System
- Module distribution
- Dependency management

---

## Design Principles

1. **Token Efficient**: Minimal syntax for AI consumption
2. **Zero Dependencies**: Single executable output
3. **Declarative**: Data flow over control flow
4. **Native Performance**: Direct machine code generation
5. **AI Autonomy**: Self-hosting enables AI self-improvement

---

## Efficiency

| Metric | TAYNI | LLVM+Clang | Improvement |
|--------|-------|------------|-------------|
| Minimal PE | 1,024 bytes | 3,584 bytes | 3.5x smaller |
| With File I/O | 2,560 bytes | 3,584 bytes | 1.4x smaller |

---

## Get Involved

- **Repository**: [github.com/NELAIA-AI/tayni](https://github.com/NELAIA-AI/tayni)
- **License**: Apache 2.0

---

*This roadmap reflects current plans and is subject to change.*
