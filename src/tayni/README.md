# TAYNI Self-Hosting Compilers

This directory contains TAYNI compilers written in TAYNI itself.

## Files

| File | Description |
|------|-------------|
| `compiler_v33.tayni` | Current version - Full parser with FSM |
| `compiler_v32.tayni` | Functional parser/compiler for 2-line programs |
| `compiler_v31.tayni` | Modular version |
| `parser_v1.tayni` | Basic parser with FSM |
| `pe_emitter_simple.tayni` | PE emission example |

## Usage

```bash
# Compile the self-hosted compiler
cargo run --release -- src/TAYNI/compiler_v33.tayni --emit-pe

# Use the self-hosted compiler
./compiler_v33.exe input.tayni -o output.exe
```

## Archive

The `archive/` folder contains previous development versions (v02-v31, debug, modules).
Maintained as historical reference of the bootstrap process.
