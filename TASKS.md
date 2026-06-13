# NELAIA Active Tasks

## Current Sprint: v0.15 Binary Optimization

### Task 1: Dead Code Elimination [COMPLETED]
- [x] Add usage analysis pass to compiler
- [x] Track which operations are used in graph
- [x] Emit only required syscall wrappers
- Result: 7,168 → 6,656 bytes (-7%)

### Task 2: Event Loop Implementation [PENDING]
- [ ] Implement EPL/EWA server pattern
- [ ] Non-blocking socket handling
- [ ] Connection state machine
- Target: 5,000 req/s

---

## Progress Log

### 2026-06-13 (Evening)
- Implemented UsageAnalysis in ir.rs
- Added emit_windows_syscalls_optimized() with conditional emission
- GUI declarations eliminated when not used
- Binary reduced from 7,168 to 6,656 bytes (-7%)
- Performance verified: 2,530 req/s

### 2026-06-13 (Afternoon)
- Created strategic plan
- Starting binary optimization
- Target: 7 KB → 2 KB

---

## Registered Roadmap

| Version | Target | Status |
|---------|--------|--------|
| v0.15 | 2 KB binary | IN PROGRESS (6.5 KB achieved) |
| v0.16 | 5,000 req/s | PENDING |
| v0.17 | 8,000 req/s | PLANNED |
| v0.18 | 10,000 req/s | PLANNED |
| v1.0 | Self-hosting | 2027-Q1 |
| v2.0 | Hardware synth | 2030 |
| v5.0 | Universal protocol | 2036 |
