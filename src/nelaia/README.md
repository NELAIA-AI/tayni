# NELAIA Self-Hosting Compilers

This directory contains NELAIA compilers written in NELAIA itself.

## Files

| File | Description |
|------|-------------|
| `compiler_v33.nela` | Current version - Full parser with FSM |
| `compiler_v32.nela` | Functional parser/compiler for 2-line programs |
| `compiler_v31.nela` | Modular version |
| `parser_v1.nela` | Basic parser with FSM |
| `pe_emitter_simple.nela` | PE emission example |

## Usage

```bash
# Compile the self-hosted compiler
cargo run --release -- src/nelaia/compiler_v33.nela --emit-pe

# Use the self-hosted compiler
./compiler_v33.exe input.nela -o output.exe
```

## Archive

The `archive/` folder contains previous development versions (v02-v31, debug, modules).
Maintained as historical reference of the bootstrap process.
