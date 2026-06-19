# TAYNI Technical Architecture

> **Version:** 1.0  
> **Last Updated:** 2026-06-19

## Overview

TAYNI is an AI-first programming language that compiles directly to multiple targets without external dependencies. This document describes the technical architecture of the compiler and runtime.

## Compiler Pipeline

```
Source Code (.tayni)
        │
        ▼
┌───────────────┐
│    Lexer      │  → Tokens
└───────────────┘
        │
        ▼
┌───────────────┐
│    Parser     │  → AST
└───────────────┘
        │
        ▼
┌───────────────┐
│  Type Check   │  → Typed AST
└───────────────┘
        │
        ▼
┌───────────────┐
│   IR Gen      │  → Internal Representation
└───────────────┘
        │
        ▼
┌───────────────┐
│  Optimizer    │  → Optimized IR
└───────────────┘
        │
        ▼
┌───────────────────────────────────────────┐
│              Code Generator               │
├─────────┬─────────┬─────────┬────────────┤
│   PE    │   ELF   │  Wasm   │   WASI     │
│ x86-64  │ x86-64  │ wasm32  │ wasm32    │
│         │ ARM64   │         │ Preview 2  │
└─────────┴─────────┴─────────┴────────────┘
```

## Module Structure

### Core Modules

| Module | File | Description |
|--------|------|-------------|
| Lexer | `lexer.rs` | Tokenizes source code |
| Parser | `parser.rs` | Builds AST from tokens |
| Type Checker | `types.rs` | Type inference and checking |
| IR | `ir.rs` | Intermediate representation |
| Optimizer | `optimizer.rs` | Constant folding, DCE |

### Target Generators

| Target | File | Output |
|--------|------|--------|
| Windows PE | `pe.rs` | `.exe` x86-64 |
| Linux ELF x86-64 | `elf.rs` | Binary |
| Linux ELF ARM64 | `elf_arm64.rs` | Binary |
| WebAssembly | `wasm.rs` | `.wasm` |
| WASI Preview 1 | `wasi.rs` | `.wasm` |
| WASI Preview 2 | `wasi_p2.rs` | `.wasm` |
| WASI HTTP | `wasi_http.rs` | `.wasm` |

### Support Modules

| Module | File | Description |
|--------|------|-------------|
| ARM64 Encoder | `arm64.rs` | Instruction encoding |
| ARM64 CodeGen | `arm64_codegen.rs` | Register allocation, IR→code |
| DWARF | `dwarf.rs` | Debug information |
| JSON | `json.rs` | RFC 8259 parser |
| HTTP Client | `http_client.rs` | HTTP/1.1 client |
| Package Manager | `pkg.rs` | Semver, manifests |

## Binary Format Details

### Windows PE (x86-64)

```
┌─────────────────┐
│   DOS Header    │  64 bytes
├─────────────────┤
│   PE Signature  │  4 bytes
├─────────────────┤
│   COFF Header   │  20 bytes
├─────────────────┤
│ Optional Header │  240 bytes
├─────────────────┤
│ Section Headers │  40 bytes each
├─────────────────┤
│   .text         │  Code
├─────────────────┤
│   .rdata        │  Read-only data
├─────────────────┤
│   .data         │  Initialized data
└─────────────────┘
```

### Linux ELF (x86-64/ARM64)

```
┌─────────────────┐
│   ELF Header    │  64 bytes
├─────────────────┤
│ Program Headers │  56 bytes each
├─────────────────┤
│   .text         │  Code (PF_R | PF_X)
├─────────────────┤
│   .rodata       │  Read-only data
├─────────────────┤
│   .data         │  Initialized data
├─────────────────┤
│   .bss          │  Uninitialized data
└─────────────────┘
```

### WebAssembly

```
┌─────────────────┐
│  Magic + Ver    │  8 bytes
├─────────────────┤
│  Type Section   │  Function signatures
├─────────────────┤
│ Import Section  │  External functions
├─────────────────┤
│  Func Section   │  Function declarations
├─────────────────┤
│ Memory Section  │  Linear memory
├─────────────────┤
│ Export Section  │  Public symbols
├─────────────────┤
│  Code Section   │  Function bodies
├─────────────────┤
│  Data Section   │  Initialized data
└─────────────────┘
```

## ARM64 Code Generation

### Register Allocation

| Register | Usage |
|----------|-------|
| X0-X7 | Arguments / Return values |
| X8 | Syscall number |
| X9-X15 | Temporaries (caller-saved) |
| X16-X17 | Intra-procedure scratch |
| X19-X28 | Callee-saved |
| X29 (FP) | Frame pointer |
| X30 (LR) | Link register |
| X31 (SP/XZR) | Stack pointer / Zero |

### Calling Convention (AAPCS64)

```
Caller:
  1. Place args in X0-X7 (or stack)
  2. BL target
  3. Result in X0

Callee:
  1. STP X29, X30, [SP, #-16]!  ; Save FP, LR
  2. MOV X29, SP                ; Set frame pointer
  3. ... function body ...
  4. LDP X29, X30, [SP], #16    ; Restore FP, LR
  5. RET
```

## DWARF Debug Information

### Sections Generated

| Section | Content |
|---------|---------|
| `.debug_info` | Compilation unit, types, functions |
| `.debug_abbrev` | Abbreviation tables |
| `.debug_line` | Line number program |
| `.debug_aranges` | Address ranges |
| `.debug_str` | String table |
| `.debug_pubnames` | Public function names |
| `.debug_pubtypes` | Public type names |
| `.debug_frame` | Call frame information |

### DIE Structure

```
Compile Unit (DW_TAG_compile_unit)
├── Base Types (DW_TAG_base_type)
│   ├── i32
│   ├── i64
│   ├── f32
│   ├── f64
│   └── bool
└── Functions (DW_TAG_subprogram)
    ├── Parameters (DW_TAG_formal_parameter)
    └── Local Variables (DW_TAG_variable)
```

## WASI Implementation

### Preview 1 Functions

| Function | Description |
|----------|-------------|
| `fd_write` | Write to file descriptor |
| `fd_read` | Read from file descriptor |
| `proc_exit` | Exit process |
| `args_get` | Get command line args |
| `args_sizes_get` | Get args buffer sizes |
| `environ_get` | Get environment |
| `environ_sizes_get` | Get env buffer sizes |

### Preview 2 Extensions

| Interface | Functions |
|-----------|-----------|
| `wasi:filesystem` | open, read, write, close, stat, mkdir, remove |
| `wasi:sockets/tcp` | create, bind, listen, accept, connect, shutdown |
| `wasi:sockets/udp` | create, bind, send, receive |
| `wasi:http` | incoming-handler, outgoing-handler |

## Serverless Platforms

### Supported Platforms

| Platform | Runtime | Entry Point |
|----------|---------|-------------|
| Cloudflare Workers | V8 + Wasm | `fetch` handler |
| Deno Deploy | Deno + Wasm | `Deno.serve` |
| Vercel Edge | V8 + Wasm | Edge function |
| Fastly Compute | Wasm | `main` |
| AWS Lambda | Custom runtime | `handler` |

### WASI HTTP Handler Pattern

```tayni
fn handle(req: IncomingRequest) -> OutgoingResponse {
    match req.path() {
        "/" => Response.json({"status": "ok"}),
        "/health" => Response.text("healthy"),
        _ => Response.status(404).json({"error": "Not Found"})
    }
}
```

## Testing Strategy

### Test Categories

| Category | Count | Description |
|----------|-------|-------------|
| Unit Tests | 200+ | Individual function tests |
| Integration | 50+ | Multi-module tests |
| Conformance | 10+ | Wasm validation |
| Examples | 32 | Full program tests |

### Test Files

- `tests_comprehensive.rs` - Core module tests
- `tests_batch2.rs` - Format and encoding tests
- `tests_batch3.rs` - Extended coverage

## Performance Characteristics

### Compilation Speed

| Target | Time (hello world) |
|--------|-------------------|
| Wasm | < 10ms |
| PE | < 50ms |
| ELF | < 50ms |

### Binary Sizes

| Program | Wasm | PE | ELF |
|---------|------|-----|-----|
| Hello World | 198B | 2KB | 1KB |
| HTTP Server | 15KB | 10.5KB | 8KB |
| JSON Parser | 5KB | 8KB | 6KB |

### Runtime Performance

- Zero-copy string handling
- Stack allocation by default
- No garbage collection
- Direct syscalls (no libc)

## Future Architecture

### Planned Additions

1. **LLVM Backend** (optional)
   - GPU support via NVPTX/AMDGPU
   - Advanced optimizations

2. **Self-Hosting**
   - TAYNI compiler written in TAYNI
   - Bootstrap from Rust version

3. **Formal Verification**
   - Type system proofs
   - Memory safety guarantees

---

*TAYNI - AI-first programming language by NELAIA*
