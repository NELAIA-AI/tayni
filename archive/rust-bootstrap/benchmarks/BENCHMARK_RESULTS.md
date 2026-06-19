# TAYNI Cross-Language HTTP Server Benchmark

**Date:** 2026-06-18 20:29  
**Platform:** Windows 10.0.26100.0

## Standalone Executables (No Runtime Required)

| Language | Binary Size | Ratio vs TAYNI |
|----------|-------------|----------------|
| TAYNI | 10,5 KB | 1.0x (baseline) |
| Zig | 12,5 KB | 1,2x larger |
| C (Clang) | 112,5 KB | 10,7x larger |
| Rust | 141,0 KB | 13,4x larger |
| Go | 5,63 MB | 548,8x larger |

## Scripts (Require Runtime)

| Language | Script Size | Runtime Required |
|----------|-------------|------------------|
| Node.js | 423 B | Requires Node.js (~80 MB) |
| Python | 689 B | Requires Python (~100 MB) |

## Key Findings

1. **TAYNI produces the smallest standalone HTTP server executable**
2. C is typically 5-10x larger (depends on CRT linking)
3. Zig is competitive but still larger than TAYNI
4. Rust is ~13x larger than TAYNI
5. Go is ~500x larger than TAYNI

## Methodology

All servers implement identical functionality:
- Listen on TCP port
- Accept one HTTP GET request  
- Return JSON response
- Exit

### Compilation Commands

| Language | Command |
|----------|---------|
| TAYNI | Native PE generation |
| C (MSVC) | `cl /O2 c_http.c ws2_32.lib` |
| C (GCC) | `gcc -O2 -s c_http.c -lws2_32` |
| Zig | `zig build-exe -OReleaseSmall -fstrip` |
| Rust | `rustc -O` |
| Go | `go build -ldflags="-s -w"` |

## Reproduce

```powershell
cd tayni-core/archive/rust-bootstrap/benchmarks
powershell -ExecutionPolicy Bypass -File run_comparison.ps1
```
