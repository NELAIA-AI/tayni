# TAYNI-C (Rust Bootstrap Compiler)

## Status: ACTIVE - Primary Compiler

The Rust compiler `tayni-c` generates native Windows PE x86-64 executables directly from `.tyn` source files without any external dependencies (no LLVM, no clang, no linker).

## Features (v0.24)

### Core Operations
- **Arithmetic**: ADD, SUB, MUL, DIV, MOD
- **Memory**: ALC, PUT, GET
- **Conversion**: ITS (int-to-string)
- **Strings**: SLN (length), CAT (concatenate), CMP (compare)
- **I/O**: PRT (print to stdout)

### JSON (stdlib/tier0)
- **JSON.ENCODE** `dst key val → len` — Generate `{"key":value}`
- **JSON.GET** `json key val → len` — Extract value by key
- **JSON.SET** `json key new_val → len` — Modify value in-place

### Runtime Operations (@)
All operations prefixed with `@` are computed at **runtime** (x86-64 machine code).
Operations without `@` are evaluated at **compile time**.

### Architecture
- `RuntimeCodeGen` struct: single source of x86-64 opcode emission
- Static `.data` section for buffers (no heap allocation needed)
- Buffer base pointer in `[rsp+0x48]` enables all `emit_*` methods
- PE generation: DOS header → COFF → Optional Header → .text/.rdata/.data/.idata

## Build

```bash
cargo build --release
```

## Usage

```bash
tayni-c input.tyn -o output.exe
```

## Tests

```bash
# All tests should produce correct output and exit 0
tayni-c test-arith.tyn -o t.exe && t.exe      # 27
tayni-c test-sln.tyn -o t.exe && t.exe        # 5
tayni-c test-cat.tyn -o t.exe && t.exe        # 10
tayni-c test-cmp.tyn -o t.exe && t.exe        # 0
tayni-c test-json-encode.tyn -o t.exe && t.exe # {"name":42}
tayni-c test-json-get.tyn -o t.exe && t.exe    # 25
tayni-c test-json-set.tyn -o t.exe && t.exe    # {"age":99}
```

## Files

| File | Purpose |
|------|---------|
| `pe.rs` | PE generator + RuntimeCodeGen (x86-64 emission) |
| `ir.rs` | Intermediate representation (Op, Node, Arg, Value) |
| `parser.rs` | TAYNI source parser |
| `modules.rs` | Module resolver (USE directives, stdlib tiers) |
| `main.rs` | CLI entry point |

## Next Steps

- Fase 5: TIME.NOW, TIME.SLEEP (requires IAT extension)
- Fase 6: Threading (THR, JON, MTX)
- Fase 7: HTTP alto nivel

---

*Consorcio TAYNI, 2026*
