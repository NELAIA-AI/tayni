# NELAIA v0.5 Release Notes

## Date: 2026-06-13

## Summary

NELAIA v0.5 delivers cross-platform support and compile-time optimization:
- **Windows support** via kernel32.dll (WriteFile, GetStdHandle, ExitProcess)
- **Linux support** via direct syscalls (no libc)
- **Constant folding** - arithmetic computed at compile time
- **Sub-graph foundation** - structure for future function support

---

## New Features

### 1. Cross-Platform Support

#### Windows (kernel32.dll)
```
nelaia-c program.nts --target=windows
```
- Uses `WriteFile` for output (works with redirected stdout)
- Uses `GetStdHandle(-11)` for stdout handle
- Uses `ExitProcess` for clean exit
- Entry point: `mainCRTStartup`

#### Linux (Direct Syscalls)
```
nelaia-c program.nts --target=linux
```
- Direct `syscall` instruction via inline assembly
- syscall #1 (write), #60 (exit)
- Entry point: `_start`
- Compiles with `-nostdlib -static`

### 2. Constant Folding

Arithmetic operations on known constants are computed at compile time:

**Input:**
```
.a: 10
.b: 20
.sum: ADD .a .b
.x: ADD .sum 50
```

**Generated IR:**
```llvm
%sum = add i64 0, 30  ; folded constant
%x = add i64 0, 80    ; folded constant (chained!)
```

Supported operations: ADD, SUB, MUL, DIV, MOD, NEG

### 3. Improved CLI

```
NELAIA Compiler v0.5
Usage: nelaia-c <file.nts> [output]

Options:
  --emit-llvm       Only emit LLVM IR, don't compile
  --target=linux    Target Linux (direct syscalls)
  --target=windows  Target Windows (kernel32.dll)
  --no-warn         Suppress warnings
  --help            Show this help
```

---

## Test Results (Windows)

### Hello World
```
> nelaia-c test_hello.nts
> test_hello.exe
Hello World from NELAIA v0.4!
```

### Arithmetic
```
> nelaia-c test_numbers.nts
> test_numbers.exe
Computing 10 + 20 = 
30
Computing 10 * 20 = 
200
42
-123
```

### Fibonacci
```
> nelaia-c examples/fibonacci.nts
> fibonacci.exe
Fibonacci sequence:
0
1
1
2
3
5
8
13
21
34
```

---

## Architecture

### Compilation Pipeline
```
.nts file
    ↓
[Parser] → Graph IR
    ↓
[Analyzer] → Cycle detection, dead nodes, undefined refs
    ↓
[Constant Folder] → Compute known values
    ↓
[Emitter] → Platform-specific LLVM IR
    ↓
[clang] → Native binary
```

### Platform Abstraction
```
                    ┌─────────────────┐
                    │  Common Runtime │
                    │  (strlen, itoa) │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
    ┌─────────┴─────────┐       ┌──────────┴──────────┐
    │   Linux Syscalls  │       │  Windows kernel32   │
    │   (inline asm)    │       │  (dllimport)        │
    └───────────────────┘       └─────────────────────┘
```

---

## Metrics

| Metric | v0.4.1 | v0.5 |
|--------|--------|------|
| Platforms | Linux only | Linux + Windows |
| Constant folding | ❌ | ✅ |
| Chained folding | ❌ | ✅ |
| Sub-graph structure | ❌ | ✅ (foundation) |

### Binary Sizes (Windows)
- Hello World: ~3KB
- Fibonacci: ~4KB

---

## Known Limitations

1. **Sub-graphs not fully implemented** - Structure exists but not callable
2. **Float printing** - Not yet implemented
3. **File I/O** - Not yet implemented
4. **Network** - Not yet implemented

---

## Next Steps (v0.6)

1. Full sub-graph implementation (functions)
2. File I/O syscalls
3. Network syscalls (TCP)
4. Memory allocation (mmap/VirtualAlloc)

---

*NELAIA v0.5 - Cross-Platform, Zero Dependencies*
