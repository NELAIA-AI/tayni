# TAYNI Benchmark Suite

## Quick Start

```powershell
cd benchmarks
.\benchmark_complete.ps1
```

## What Gets Tested

Tests 6 languages in order (slowest to fastest expected):

| # | Language | Port | Type |
|---|----------|------|------|
| 1 | Python | 8081 | Interpreted |
| 2 | Node.js | 8082 | Interpreted |
| 3 | Rust | 8087 | Compiled |
| 4 | C | 8089 | Compiled |
| 5 | Go | 8083 | Compiled |
| 6 | **TAYNI** | 8101 | Compiled |

## Latest Results

```
TAYNI v0.15:  ████████████████████████████████████████████████ 2,419 req/s
C Optimized:   ██████████████████████████████████████           1,904 req/s
Go HTTP:       ███████████████████████████████████              1,744 req/s
Rust Opt:      ██████████████████████████████████               1,701 req/s
Python:        ███████████████████████████████                  1,371 req/s
Node.js:       █████████████████████████                        1,241 req/s
```

**TAYNI is 27-95% faster than all other languages tested.**

## Requirements

- Go 1.18+ (for benchmark tool and Go server)
- Rust (for Rust server)
- Clang/LLVM (for C server and TAYNI)
- Python 3.x
- Node.js

## Scripts

| Script | Description |
|--------|-------------|
| `benchmark_complete.ps1` | Full benchmark, all languages |
| `benchmark_visual.ps1` | Generates HTML report |
| `run_benchmark.ps1` | Quick benchmark, compiled only |

## Parameters

```powershell
.\benchmark_complete.ps1 -Requests 5000 -Concurrent 100 -Rounds 3
```

## Build Servers Manually

```powershell
# Go
go build -o solar_go.exe solar_go.go

# Rust
rustc -O -o solar_rust_opt.exe solar_rust_opt.rs

# C
clang -O2 -o solar_c_opt.exe solar_c_opt.c -lws2_32

# TAYNI
cd ..
cargo run --release -- benchmarks/solar_TAYNI_16w.tayni
```

## Troubleshooting

### Port in use
```powershell
taskkill /F /IM python.exe
taskkill /F /IM node.exe
taskkill /F /IM solar_*.exe
```

### Inconsistent results
- Close other applications
- Run as Administrator
- Increase `-Rounds` parameter
