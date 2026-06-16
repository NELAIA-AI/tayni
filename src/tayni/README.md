# TAYNI Self-Hosting Compilers

This directory contains TAYNI compilers written in TAYNI itself.

## Files

| File | Description |
|------|-------------|
| `compiler_v33.tyn` | Current version - Full parser with FSM |
| `compiler_v32.tyn` | Functional parser/compiler for 2-line programs |
| `compiler_v31.tyn` | Modular version |
| `parser_v1.tyn` | Basic parser with FSM |
| `pe_emitter_simple.tyn` | PE emission example |

## Usage

```bash
# Compile the self-hosted compiler
cargo run --release -- src/TAYNI/compiler_v33.tyn --emit-pe

# Use the self-hosted compiler
./compiler_v33.exe input.tyn -o output.exe
```

## Archive

The `archive/` folder contains previous development versions (v02-v31, debug, modules).
Maintained as historical reference of the bootstrap process.
