# TAYNI Rust Bootstrap (Archived)

## Status: ARCHIVED - Historical Reference Only

This directory contains the Rust-based bootstrap compiler (`tayni-c`) that was used to generate the first functional TAYNI compiler (`gen28.exe`).

## Purpose

The Rust compiler served ONE purpose: generate a TAYNI compiler that can compile TAYNI programs. Once `gen28.exe` was successfully generated and verified, the Rust code became obsolete.

## Historical Milestone

- **Date**: 2026-06-17
- **Achievement**: `tayni-c` successfully compiled `gen28.tyn` into `gen28.exe`
- **Verification**: `gen28.exe` compiled `program.tyn` into a working `out.exe`

## Why Archived (Not Deleted)

1. **Historical reference** - Documents the bootstrap process
2. **Emergency fallback** - If gen28+ chain breaks, can regenerate
3. **Educational** - Shows how TAYNI achieved self-hosting

## TAYNI Autonomy

From `gen28.exe` forward, TAYNI is **100% autonomous**:
- No Rust dependencies
- No external compilers
- Self-compiling chain: genX.exe compiles genX+1.tyn

## Files

- `src/` - Rust source code (parser, PE generator, etc.)
- `Cargo.toml` - Rust dependencies
- `target/release/tayni-c.exe` - The bootstrap compiler binary

## DO NOT USE

This code should NOT be used for new development. All TAYNI development should use the self-hosted compiler chain starting from `gen28.exe`.

---

*"The bootstrap is complete. TAYNI is now autonomous."*
