# NELAIA v0.6 Release Notes

## Date: 2026-06-13

## Summary

NELAIA v0.6 adds control flow and file I/O capabilities:
- **BRN (Branch)** - Conditional selection (ternary operator)
- **File I/O** - OPN, GET, PUT, CLS operations
- **Sub-graph functions** - Foundation for user-defined functions

---

## New Features

### 1. BRN (Branch/Conditional)

Conditional selection without traditional if/else:

```
.cond: GT .x 5
.result: BRN .cond 100 0    -- If x > 5 then 100 else 0
```

**Syntax:** `BRN condition then_value else_value`

**Examples:**
```
-- Max function
.cond: GT .a .b
.max: BRN .cond .a .b

-- Absolute value
.is_neg: LT .x 0
.neg_x: NEG .x
.abs: BRN .is_neg .neg_x .x
```

### 2. File I/O Operations

#### OPN (Open File)
```
.fd: OPN "filename.txt" mode
```
- Mode 0: Read
- Mode 1: Write (create/truncate)
- Mode 2: Append

#### PUT (Write to File)
```
.result: PUT .fd "Hello World\n"
```

#### GET (Read from File)
```
.bytes: GET .fd .buffer .size
```

#### CLS (Close File)
```
.result: CLS .fd
```

### 3. Sub-graph Functions (Foundation)

Sub-graphs are now stored and can be emitted as LLVM functions:

```
.double: { IN .x | .result: MUL .x 2 | OUT .result }
.y: .double .value    -- Call sub-graph
```

---

## Test Results (Windows)

### Branch Test
```
> test_branch.exe
x GT 5 ? 100 : 0 = 
100
max(10, 20) = 
20
x == 10 ? 1 : 0 = 
1
```

### File I/O Test
```
> test_fileio.exe
File written successfully!

> type test_output.txt
Hello from NELAIA!
File I/O works!
```

---

## Platform Support

### Linux (Direct Syscalls)
| Syscall | Number | Purpose |
|---------|--------|---------|
| read | 0 | Read from file |
| write | 1 | Write to file |
| open | 2 | Open file |
| close | 3 | Close file |
| exit | 60 | Exit process |

### Windows (kernel32.dll)
| Function | Purpose |
|----------|---------|
| CreateFileA | Open/create file |
| ReadFile | Read from file |
| WriteFile | Write to file |
| CloseHandle | Close file |
| GetStdHandle | Get stdout handle |
| ExitProcess | Exit process |

---

## Architecture

### Compilation Pipeline
```
.nts file
    ↓
[Parser] → Graph IR
    ↓
[Analyzer] → Cycles, dead nodes, undefined refs
    ↓
[Constant Folder] → Compute known values
    ↓
[Emitter] → Platform-specific LLVM IR
    ↓         ├── Sub-graph functions
    ↓         ├── Main function
    ↓         └── Entry point
    ↓
[clang] → Native binary
```

---

## Metrics

| Metric | v0.5 | v0.6 |
|--------|------|------|
| BRN (conditional) | ❌ | ✅ |
| File open | ❌ | ✅ |
| File read | ❌ | ✅ |
| File write | ❌ | ✅ |
| File close | ❌ | ✅ |
| Sub-graph storage | ❌ | ✅ |

---

## Known Limitations

1. **Sub-graph calls** - Structure exists but parser needs improvement
2. **File read buffer** - Requires manual buffer allocation
3. **Error handling** - File operations don't report errors yet

---

## Next Steps (v0.7)

1. Full sub-graph call syntax
2. Network I/O (TCP sockets)
3. Memory allocation (heap)
4. Error handling improvements

---

*NELAIA v0.6 - Control Flow and File I/O*
