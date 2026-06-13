# NELAIA v0.4.1 Release Notes

## Date: 2026-06-13

## Summary

NELAIA v0.4.1 implements all immediate priorities from the Consortium review:
- Pure `itoa` (integer to string without libc)
- Cycle detection in graphs
- Dead node warnings
- Undefined reference detection

---

## New Features

### 1. Pure Integer Printing (`itoa`)

NELAIA can now print integers without any libc dependency. The implementation:
- Handles positive, negative, and zero values
- Uses a 21-byte static buffer (max i64 digits + sign + null)
- Converts digits in reverse order for efficiency
- Pure LLVM IR, no external calls

**Example:**
```
.a: 10
.b: 20
.sum: ADD .a .b
.sum > PRT  -- Prints: 30
```

### 2. Graph Analysis

The compiler now performs static analysis before code generation:

#### Cycle Detection
```
.a: .b
.b: .a
```
**Error:** `Cycle detected in graph: a -> b -> a`

#### Undefined Reference Detection
```
.x: ADD .a .undefined
```
**Error:** `.undefined is not defined`

#### Dead Node Warnings
```
.unused: 42  -- Warning: defined but never used
```
**Warning:** `Dead nodes: .unused`

### 3. Improved CLI

```
NELAIA Compiler v0.4.1
Usage: nelaia-c <file.nts> [output]

Options:
  --emit-llvm    Only emit LLVM IR, don't compile
  --no-warn      Suppress warnings
  --help         Show this help
```

---

## Examples

### Fibonacci Sequence
```
.n1: 0
.n2: 1
.n3: ADD .n1 .n2
.n4: ADD .n2 .n3
-- ... continues
.n3 > PRT  -- Prints: 1
.n4 > PRT  -- Prints: 2
```

### Arithmetic Operations
```
.a: 100
.b: 7
.sum: ADD .a .b   -- 107
.diff: SUB .a .b  -- 93
.prod: MUL .a .b  -- 700
.quot: DIV .a .b  -- 14
.rem: MOD .a .b   -- 2
.neg: NEG .a      -- -100
```

### Comparisons
```
.x: 10
.y: 20
.eq: EQ .x .y   -- 0 (false)
.lt: LT .x .y   -- 1 (true)
.gt: GT .x .y   -- 0 (false)
```

---

## Technical Details

### LLVM IR: Pure itoa Implementation

```llvm
; Convert i64 to string, returns pointer to start of number in buffer
define i8* @nelaia_itoa(i64 %num) {
entry:
  %buf_end = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i32 20
  store i8 0, i8* %buf_end
  %is_zero = icmp eq i64 %num, 0
  br i1 %is_zero, label %zero_case, label %check_neg
  ; ... (full implementation in emitter.rs)
}
```

### Graph Analysis Algorithm

1. **Build dependency graph** from node definitions
2. **DFS for cycle detection** with recursion stack
3. **Set difference** for dead nodes (defined - used)
4. **Set difference** for undefined refs (used - defined)

---

## Metrics

| Metric | v0.4.0 | v0.4.1 |
|--------|--------|--------|
| Integer printing | ❌ | ✅ |
| Cycle detection | ❌ | ✅ |
| Dead node warnings | ❌ | ✅ |
| Undefined ref errors | ❌ | ✅ |
| Compilation phases | 2 | 3 |

---

## Next Steps (v0.5)

Per Consortium roadmap:
1. Windows ntdll.dll syscall layer
2. Constant folding optimization
3. Sub-graph recursion

---

## Files Changed

- `src/main.rs` - Added Phase 1.5 (analysis), --no-warn flag
- `src/ir.rs` - Added `GraphAnalysis` struct and `analyze()` method
- `src/emitter.rs` - Added `nelaia_itoa`, `nelaia_print_int` functions
- `Cargo.toml` - Version bump to 0.4.1
- `examples/` - New examples: fibonacci.nts, arithmetic.nts, comparisons.nts

---

*NELAIA v0.4.1 - AI to Hardware, No Compromises*
