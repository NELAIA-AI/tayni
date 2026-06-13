# NELAIA Active Tasks

## Current Sprint: v0.15 Binary Optimization

### Task 1: Dead Code Elimination [COMPLETED]
- [x] Add usage analysis pass to compiler
- [x] Track which operations are used in graph
- [x] Emit only required syscall wrappers
- Result: 7,168 → 6,656 bytes (-7%)

### Task 2: Complete Benchmark Suite [COMPLETED]
- [x] Add Python and Node.js to benchmarks
- [x] Order tests from slowest to fastest
- [x] NELAIA tested last (champion position)
- [x] Update benchmark scripts
- Result: NELAIA #1 at 2,419 req/s

---

## Final Benchmark Results

| Rank | Language | Throughput | vs NELAIA |
|------|----------|------------|-----------|
| #1 | NELAIA | 2,419 req/s | baseline |
| #2 | C | 1,904 req/s | -21% |
| #3 | Go | 1,744 req/s | -28% |
| #4 | Rust | 1,701 req/s | -30% |
| #5 | Python | 1,371 req/s | -43% |
| #6 | Node.js | 1,241 req/s | -49% |

---

## Progress Log

### 2026-06-13 (Evening)
- Completed benchmark suite with all 6 languages
- NELAIA confirmed as fastest (2,419 req/s)
- Updated RESULTS.md and README.md

### 2026-06-13 (Afternoon)
- Implemented UsageAnalysis in ir.rs
- Added emit_windows_syscalls_optimized()
- Binary reduced from 7,168 to 6,656 bytes (-7%)

---

## Registered Roadmap

| Version | Target | Status |
|---------|--------|--------|
| v0.15 | Binary optimization | COMPLETED |
| v0.16 | Event loop (5,000 req/s) | NEXT |
| v0.17 | Zero-copy I/O | PLANNED |
| v1.0 | Self-hosting | 2027-Q1 |
