# TAYNI

[![Build](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml/badge.svg)](https://github.com/NELAIA-AI/tayni/actions/workflows/build.yml)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20695531.svg)](https://doi.org/10.5281/zenodo.20695531)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.96+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue.svg)](https://github.com/NELAIA-AI/tayni/releases)

TAYNI Compiler v0.24 - A graph-based language optimized for AI code generation.

> **Minimal syntax. Direct native compilation. Zero dependencies.**

## Quick Start (5 Steps)

### 1. Download

```bash
# Linux x64
curl -LO https://github.com/NELAIA-AI/tayni/releases/latest/download/tayni-c-linux-x64
chmod +x tayni-c-linux-x64

# macOS Apple Silicon
curl -LO https://github.com/NELAIA-AI/tayni/releases/latest/download/tayni-c-macos-arm64
chmod +x tayni-c-macos-arm64

# macOS Intel
curl -LO https://github.com/NELAIA-AI/tayni/releases/latest/download/tayni-c-macos-x64
chmod +x tayni-c-macos-x64

# Windows: Download tayni-c-windows-x64.exe from Releases
```

### 2. Verify Installation

```bash
./tayni-c-linux-x64 --version
# Output: tayni-c 0.24.0
```

### 3. Create a Program

```bash
cat > hello.tyn << 'EOF'
.msg: "Hello from TAYNI!\n"
.len: 20
.out: PRT .msg .len
EOF
```

### 4. Compile

```bash
./tayni-c-linux-x64 hello.tyn -o hello
# Output: OK:ELF:hello:2048 bytes (direct emission, no clang)
```

### 5. Run

```bash
./hello
# Output: Hello from TAYNI!
```

## Example Program

```tayni
-- hello.tyn
.msg: "Hello from TAYNI!\n"
.len: 20
.out: PRT .msg .len
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`docs/TAYNI-GUIDE-v0.22.md`](docs/TAYNI-GUIDE-v0.22.md) | Complete guide - all operators |
| [`docs/TAYNI-REFERENCE-v0.22.md`](docs/TAYNI-REFERENCE-v0.22.md) | EBNF grammar + operator tables |
| [`docs/TAYNI-EXAMPLES-v0.22.md`](docs/TAYNI-EXAMPLES-v0.22.md) | Examples with dependency graphs |
| [`docs/TAYNI-SEMANTICS-v0.22.md`](docs/TAYNI-SEMANTICS-v0.22.md) | Type system + semantic rules |
| [`docs/TAYNI-TRAINING-DATA.jsonl`](docs/TAYNI-TRAINING-DATA.jsonl) | 100+ input/output pairs for training |
| [`llms.txt`](llms.txt) | AI discovery manifest |

## Features

- **Graph-based paradigm** - How AIs think about computation
- **Token-efficient syntax** - Minimal tokens, maximum information
- **Tiny executables** - Hello World: 145 bytes (Linux), 1KB (Windows), 4KB (macOS)
- **Zero dependencies** - Direct PE/ELF/Mach-O emission, no Clang/GCC required
- **Capability system** - HTTP, SQL, JSON, Files, Network, GUI
- **MCP Server** - AI agents can invoke TAYNI directly

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
TAYNI-c --version

# Syntax check only
TAYNI-c program.tyn --check

# Compile to native executable (auto-detects platform)
TAYNI-c program.tyn -o program

# Force specific format
TAYNI-c program.tyn -o program --emit-pe           # Windows PE
TAYNI-c program.tyn -o program --emit-elf          # Linux ELF
TAYNI-c program.tyn -o program --emit-macho        # macOS x64 (Intel)
TAYNI-c program.tyn -o program --emit-macho-arm64  # macOS ARM64 (Apple Silicon)

# Cross-compile
TAYNI-c program.tyn --target=linux      # From any OS to Linux
TAYNI-c program.tyn --target=macos-arm64 # From any OS to macOS ARM64

# Use LLVM+Clang flow (optional, requires clang)
TAYNI-c program.tyn -o program --use-clang

# JSON output for programmatic use
TAYNI-c program.tyn --check --json
```

## Project Structure

```
TAYNI-core/
├── src/                    # Rust compiler source
│   ├── main.rs            # CLI entry point
│   ├── parser.rs          # TAYNI parser
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

# The compiler binary will be at target/release/TAYNI-c
```

## Troubleshooting

### "Permission denied" on Linux/macOS

```bash
chmod +x TAYNI-c-linux-x64
```

### "command not found"

Use the full path or add to PATH:

```bash
./TAYNI-c-linux-x64 --version
# or
export PATH=$PATH:$(pwd)
TAYNI-c-linux-x64 --version
```

### "E:PARSE: Unrecognized syntax"

Check your TAYNI syntax:
- Comments use `--` (not `//` or `#`)
- Nodes start with `.` (e.g., `.x: 42`)
- Strings use double quotes

```TAYNI
-- This is a comment
.x: 42
.msg: "Hello"
```

### "E:UNDEF: undefined reference"

All referenced nodes must be defined:

```TAYNI
-- Wrong: .y is not defined
.result: ADD .x .y

-- Correct: define all nodes
.x: 10
.y: 20
.result: ADD .x .y
```

### "E:CYCLE: circular dependency"

TAYNI graphs must be acyclic:

```TAYNI
-- Wrong: circular reference
.a: ADD .b 1
.b: ADD .a 1

-- Correct: no cycles
.a: 10
.b: ADD .a 1
```

### Binary doesn't run on target OS

Use the correct binary for your platform:
- Windows: `TAYNI-c-windows-x64.exe`
- Linux: `TAYNI-c-linux-x64`
- macOS Intel: `TAYNI-c-macos-x64`
- macOS Apple Silicon: `TAYNI-c-macos-arm64`

Or cross-compile with `--target`:

```bash
./TAYNI-c-linux-x64 program.tyn --target=windows -o program
```

### Need more help?

```bash
TAYNI-c --help
TAYNI-c program.tyn --check --json  # Machine-readable errors
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*TAYNI - Graph-based compilation for AI systems*
