# TAYNI Self-Hosted Compiler Archive

This folder contains **historical PoC artifacts** from the self-hosting development process.

## Status: ARCHIVED

These files are **not production code**. They document the incremental development of the self-hosted compiler.

## Key Files

| File | Description | Lines |
|------|-------------|-------|
| `compiler_v32.tyn` | Compiles 2-line programs | 125 |
| `boot_compiler.tyn` | Compiles boot.nela (9 lines) | 781 |
| `self_compiler_v2.tyn` | Compiles 6 lines with multiple ops | 431 |
| `nelaia-c.tyn` | Early compiler attempt | - |

## Why Archived?

Per Consortium decision (2026-06-16):

> "Los compiladores separados fueron útiles para desarrollo incremental, pero para IA un compilador monolítico es más eficiente de generar y mantener."

## Current Architecture

The official self-hosted compiler will be:

```
src/tayni/tayni-c.tyn  (monolithic, ~2000-3000 lines)
```

This file does not exist yet - it's the target of Phase 13 (Self-Hosting).

## Bootstrap Process

```
Stage 0: tayni-c (Rust) compiles tayni-c.tyn → tayni-c1
Stage 1: tayni-c1 compiles tayni-c.tyn → tayni-c2
Stage 2: Verify tayni-c1 == tayni-c2 (bit-identical)
Stage 3: tayni-c2 is the official release
```

## Do Not Delete

These files serve as documentation of the development process and may be useful for:
- Understanding the evolution of the compiler
- Debugging bootstrap issues
- Historical reference
