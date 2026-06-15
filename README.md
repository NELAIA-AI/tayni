# NELAIA Core

NELAIA Compiler v0.22 - Metalanguage designed by AIs, for AIs.

> **The last programming language. The first AI language.**

## Quick Start

```bash
# Download (Linux)
curl -LO https://github.com/NELAIA-AI/nelaia-core/releases/latest/download/nelaia-c-linux-x64
chmod +x nelaia-c-linux-x64

# Download (Windows)
# Get nelaia-c-windows-x64.exe from Releases

# Compile a program
./nelaia-c-linux-x64 hello.nela -o hello --emit-elf
./hello
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
- **Tiny executables** - Hello World = 2KB, HTTP server = 5KB
- **No dependencies** - Direct PE/ELF emission, no Clang required
- **Capability system** - HTTP, SQL, JSON, Files, Network, GUI

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

# Compile to Windows PE
nelaia-c program.nela -o program --emit-pe

# Compile to Linux ELF
nelaia-c program.nela -o program --emit-elf

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
│   └── elf.rs             # Linux ELF generator
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

## License

Open source

---

*NELAIA - Designed for AI code generation*
