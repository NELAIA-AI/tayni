# NELAIA

[![Build](https://github.com/NELAIA-AI/nelaia/actions/workflows/build.yml/badge.svg)](https://github.com/NELAIA-AI/nelaia/actions/workflows/build.yml)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20695531.svg)](https://doi.org/10.5281/zenodo.20695531)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue.svg)](https://github.com/NELAIA-AI/nelaia/releases)

NELAIA Compiler v0.23 - A graph-based language optimized for AI code generation.

> **Minimal syntax. Direct native compilation. Zero dependencies.**

## Quick Start (5 Steps)

### 1. Download

```bash
# Linux x64
curl -LO https://github.com/NELAIA-AI/nelaia/releases/latest/download/nelaia-c-linux-x64
chmod +x nelaia-c-linux-x64

# macOS Apple Silicon
curl -LO https://github.com/NELAIA-AI/nelaia/releases/latest/download/nelaia-c-macos-arm64
chmod +x nelaia-c-macos-arm64

# macOS Intel
curl -LO https://github.com/NELAIA-AI/nelaia/releases/latest/download/nelaia-c-macos-x64
chmod +x nelaia-c-macos-x64

# Windows: Download nelaia-c-windows-x64.exe from Releases
```

### 2. Verify Installation

```bash
./nelaia-c-linux-x64 --version
# Output: nelaia-c 0.23.0
```

### 3. Create a Program

```bash
cat > hello.nela << 'EOF'
.msg: "Hello from NELAIA!\n"
.len: 20
.out: PRT .msg .len
EOF
```

### 4. Compile

```bash
./nelaia-c-linux-x64 hello.nela -o hello
# Output: OK:ELF:hello:2048 bytes (direct emission, no clang)
```

### 5. Run

```bash
./hello
# Output: Hello from NELAIA!
```

## Example Program

```nelaia
-- hello.nela
.msg: "Hello from NELAIA!\n"
.len: 20
.out: PRT .msg .len
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`docs/NELAIA-GUIDE-v0.22.md`](docs/NELAIA-GUIDE-v0.22.md) | Complete guide - all operators |
| [`docs/NELAIA-REFERENCE-v0.22.md`](docs/NELAIA-REFERENCE-v0.22.md) | EBNF grammar + operator tables |
| [`docs/NELAIA-EXAMPLES-v0.22.md`](docs/NELAIA-EXAMPLES-v0.22.md) | Examples with dependency graphs |
| [`docs/NELAIA-SEMANTICS-v0.22.md`](docs/NELAIA-SEMANTICS-v0.22.md) | Type system + semantic rules |
| [`docs/NELAIA-TRAINING-DATA.jsonl`](docs/NELAIA-TRAINING-DATA.jsonl) | 100+ input/output pairs for training |
| [`llms.txt`](llms.txt) | AI discovery manifest |

## Features

- **Graph-based paradigm** - How AIs think about computation
- **Token-efficient syntax** - Minimal tokens, maximum information
- **Tiny executables** - Hello World: 145 bytes (Linux), 1KB (Windows), 4KB (macOS)
- **Zero dependencies** - Direct PE/ELF/Mach-O emission, no Clang/GCC required
- **Capability system** - HTTP, SQL, JSON, Files, Network, GUI
- **MCP Server** - AI agents can invoke NELAIA directly

## Available Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | ADD, SUB, MUL, DIV, MOD, NEG |
| Comparison | EQ, NE, LT, GT, LE, GE |
| Logic | AND, OR, NOT |
| Memory | ALC, FRE, PUT, GET, CPY, SLN |
| I/O | PRT, INP, FOP, FRD, FWR, FCL |
| Network | TCP, UDP, BND, LST, ACC, XMT, RCV, CLS |
| HTTP | HTTP.LISTEN, HTTP.ACCEPT, HTTP.RESPOND, HTTP.GET, HTTP.POST |
| SQL | SQL.CONNECT, SQL.QUERY, SQL.CLOSE |
| JSON | JSON.PARSE, JSON.ENCODE, JSON.GET, JSON.SET |

## Usage

```bash
# Check version
nelaia-c --version

# Syntax check only
nelaia-c program.nela --check

# Compile to native executable (auto-detects platform)
nelaia-c program.nela -o program

# Force specific format
nelaia-c program.nela -o program --emit-pe           # Windows PE
nelaia-c program.nela -o program --emit-elf          # Linux ELF
nelaia-c program.nela -o program --emit-macho        # macOS x64 (Intel)
nelaia-c program.nela -o program --emit-macho-arm64  # macOS ARM64 (Apple Silicon)

# Cross-compile
nelaia-c program.nela --target=linux      # From any OS to Linux
nelaia-c program.nela --target=macos-arm64 # From any OS to macOS ARM64

# Use LLVM+Clang flow (optional, requires clang)
nelaia-c program.nela -o program --use-clang

# JSON output for programmatic use
nelaia-c program.nela --check --json
```

## Project Structure

```
nelaia-core/
├── src/                    # Rust compiler source
│   ├── main.rs            # CLI entry point
│   ├── parser.rs          # NELAIA parser
│   ├── ir.rs              # Intermediate representation
│   ├── pe.rs              # Windows PE generator
│   ├── elf.rs             # Linux ELF generator
│   └── macho.rs           # macOS Mach-O generator
├── examples/              # Example programs
├── docs/                  # Documentation
└── .github/workflows/     # CI/CD
```

## Building from Source

```bash
# Requires Rust
cargo build --release

# Run tests
cargo test

# The compiler binary will be at target/release/nelaia-c
```

## Troubleshooting

### "Permission denied" on Linux/macOS

```bash
chmod +x nelaia-c-linux-x64
```

### "command not found"

Use the full path or add to PATH:

```bash
./nelaia-c-linux-x64 --version
# or
export PATH=$PATH:$(pwd)
nelaia-c-linux-x64 --version
```

### "E:PARSE: Unrecognized syntax"

Check your NELAIA syntax:
- Comments use `--` (not `//` or `#`)
- Nodes start with `.` (e.g., `.x: 42`)
- Strings use double quotes

```nelaia
-- This is a comment
.x: 42
.msg: "Hello"
```

### "E:UNDEF: undefined reference"

All referenced nodes must be defined:

```nelaia
-- Wrong: .y is not defined
.result: ADD .x .y

-- Correct: define all nodes
.x: 10
.y: 20
.result: ADD .x .y
```

### "E:CYCLE: circular dependency"

NELAIA graphs must be acyclic:

```nelaia
-- Wrong: circular reference
.a: ADD .b 1
.b: ADD .a 1

-- Correct: no cycles
.a: 10
.b: ADD .a 1
```

### Binary doesn't run on target OS

Use the correct binary for your platform:
- Windows: `nelaia-c-windows-x64.exe`
- Linux: `nelaia-c-linux-x64`
- macOS Intel: `nelaia-c-macos-x64`
- macOS Apple Silicon: `nelaia-c-macos-arm64`

Or cross-compile with `--target`:

```bash
./nelaia-c-linux-x64 program.nela --target=windows -o program
```

### Need more help?

```bash
nelaia-c --help
nelaia-c program.nela --check --json  # Machine-readable errors
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*NELAIA - Graph-based compilation for AI systems*
