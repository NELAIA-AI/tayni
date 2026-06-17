# TAYNI

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Bootstrap](https://img.shields.io/badge/Bootstrap-Complete-brightgreen.svg)](https://github.com/NELAIA-AI/tayni)
[![Compiler](https://img.shields.io/badge/Compiler-Gen28-green.svg)](https://github.com/NELAIA-AI/tayni)
[![Stage](https://img.shields.io/badge/Stage-Self--Hosting-blue.svg)](https://github.com/NELAIA-AI/tayni)

**TAYNI** - An AI-first programming language optimized for token efficiency and minimal binary sizes.

> **Status**: **SELF-HOSTING ACHIEVED** - gen28.exe compiles TAYNI programs without any Rust dependencies.

## What Works Today

The TAYNI compiler chain is now **100% autonomous**:

- `gen28.exe` compiles TAYNI programs into Windows PE executables
- No Rust, no external compilers, no dependencies
- Self-compiling chain: genX.exe compiles genX+1.tyn

```bash
# Compile a program with gen28
echo .a: 5 > program.tyn
echo .b: 3 >> program.tyn  
echo .c: ADD .a .b >> program.tyn
.\gen28.exe
.\out.exe  # Outputs result
```

## Bootstrap History

The Rust bootstrap compiler (`tayni-c`) has been **archived**. It served its purpose: generating the first functional TAYNI compiler. See `archive/rust-bootstrap/` for historical reference.

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

-- Memory
.buf: ALC 256
.byte: GET .buf 0

-- Output
.out: PRT .msg 5

-- Module imports
USE json
USE http

-- Program terminator
!
```

## Available Operators (Language Spec)

| Category | Operators |
|----------|-----------|
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD` |
| Comparison | `EQ`, `NE`, `LT`, `GT`, `LE`, `GE` |
| Logic | `AND`, `OR`, `NOT` |
| Memory | `ALC`, `FRE`, `PUT`, `GET`, `CPY` |
| I/O | `PRT`, `FOP`, `FRD`, `FWR`, `FCL` |
| String | `ITS`, `SLN`, `CAT` |
| Network | `TCP`, `BND`, `LST`, `ACC`, `XMT`, `RCV`, `CLS` |
| Control | `IFZ` (conditional select) |

## Project Structure

```
tayni-core/
├── bootstrap/           # Working bootstrap compiler (8KB)
│   ├── tayni-bootstrap.exe
│   ├── compiler.exe     # Self-replication copy
│   └── README.md
├── src/
│   ├── tayni/           # Self-hosted compiler source (.tyn)
│   │   ├── gen15.tyn    # Current compiler (numeric literals)
│   │   ├── gen11.tyn    # Earlier generation
│   │   └── archive/     # Historical generations
│   ├── emitters/        # Multi-target specs (WASM, GPU, QIR, RISC-V)
│   ├── main.rs          # Rust reference compiler entry point
│   ├── parser.rs        # Rust reference parser
│   ├── pe.rs            # Rust PE generator (reference for gen16+)
│   └── ...              # Other Rust reference modules
├── stdlib/              # Standard library specifications
│   ├── tier0/           # Essential (10 modules)
│   ├── tier1/           # Common (12 modules)
│   └── tier2/           # Specialized (14 modules)
├── examples/            # Example programs
├── docs/                # Documentation
├── ROADMAP.md           # Honest status and roadmap
└── target/release/      # Compiled Rust reference (tayni-c.exe)
```

## Standard Library (36 modules - Specification)

The stdlib exists as design documents defining APIs and expansion templates. These will become compilable as the self-hosted compiler gains capabilities.

| Tier | Modules | Examples |
|------|---------|----------|
| Tier 0 (Essential) | 10 | file, string, json, http, log |
| Tier 1 (Common) | 12 | env, hash, time, uuid, jwt, regex |
| Tier 2 (Specialized) | 14 | postgres, redis, websocket, crypto, tls |

## Multi-Target (Specification)

Target format specifications exist for future implementation:
- WebAssembly (WASM/WASI)
- RISC-V (Linux ELF)
- ARM64 (Linux ELF)
- GPU (NVIDIA PTX, AMD AMDGPU, SPIR-V)
- Quantum (QIR for Azure Quantum)

## Development Roadmap

| Generation | Capability | Status |
|-----------|-----------|--------|
| gen15 | Self-replication + numeric literal PE | Done |
| gen16 | Multi-line parser, 2 bindings + ADD/SUB/MUL | Done |
| gen17 | String literals + PRT (runtime x86-64!) | Done |
| gen18 | File I/O (FOP, FRD, FWR, FCL) | Next |
| gen19 | Network (TCP, BND, LST, ACC, XMT, RCV) | Planned |
| gen20 | USE directive, stdlib expansion | Planned |

## Design Principles

1. **Token Efficient** - Minimal syntax for AI consumption
2. **Zero Dependencies** - Single executable output, no runtime
3. **Declarative** - Data flow over control flow (no loops, no jumps)
4. **Native Performance** - Direct machine code generation
5. **Incremental Bootstrap** - Each generation compiles the next
6. **AI Autonomy** - Self-hosting enables AI self-improvement

## Building / Using

```bash
# The bootstrap compiler is pre-built. No build step needed.
cd bootstrap/

# To compile a simple program:
echo .x: 99 > input.tyn
.\tayni-bootstrap.exe
.\out.exe
echo %ERRORLEVEL%  # 99
```

## Reference Compiler (Rust)

A full-featured Rust implementation exists in `src/*.rs` and can be built with `cargo build --release`. It supports all operations, multiple targets, and the full stdlib. It serves as reference for the machine code patterns needed by the self-hosted compiler. The goal is for the TAYNI self-hosted compiler to eventually replace it entirely.

## License

Apache 2.0 License - see [LICENSE](LICENSE) for details.

---

*TAYNI - AI-first programming language. Bootstrap complete, general compiler in development.*
