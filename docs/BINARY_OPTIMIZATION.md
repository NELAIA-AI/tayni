# NELAIA Binary Optimization Guide

## Current State Analysis

### Binary Composition (7,168 bytes)

```
Section Analysis (solar_nelaia_16w.exe):
┌─────────────────────────────────────────┐
│ .text (code)           ~4,096 bytes     │
│ .rdata (strings)       ~1,536 bytes     │
│ .data (globals)        ~512 bytes       │
│ PE headers             ~1,024 bytes     │
└─────────────────────────────────────────┘
```

### Breakdown by Component

| Component | Size | Optimizable |
|-----------|------|-------------|
| Syscall wrappers | 2,048 B | Yes - emit only used |
| String constants | 1,536 B | Yes - deduplicate |
| Main logic | 1,024 B | Minimal |
| PE headers | 1,024 B | Yes - custom linker |
| Thread setup | 512 B | Minimal |
| Padding/alignment | 1,024 B | Yes - pack tighter |

---

## Optimization Strategies

### 1. Dead Code Elimination (Target: -2 KB)

**Problem:** All syscall wrappers emitted regardless of usage.

**Current emitter_pure.rs:**
```rust
fn emit_windows_syscalls(&self) -> String {
    // Emits ALL wrappers: socket, bind, listen, accept, send, recv,
    // setsockopt, CreateThread, WaitForSingleObject, VirtualAlloc,
    // WriteFile, GetStdHandle, ExitProcess, etc.
}
```

**Optimized approach:**
```rust
fn emit_windows_syscalls(&self, used_ops: &HashSet<Op>) -> String {
    let mut ir = String::new();
    
    // Only emit what's actually used
    if used_ops.contains(&Op::Tcp) || used_ops.contains(&Op::Udp) {
        ir.push_str(&self.emit_socket_wrapper());
    }
    if used_ops.contains(&Op::Bnd) {
        ir.push_str(&self.emit_bind_wrapper());
    }
    // ... etc
}
```

**Implementation:**
1. Add analysis pass to collect used operations
2. Pass used_ops set to syscall emitter
3. Conditionally emit only required wrappers

### 2. String Deduplication (Target: -0.5 KB)

**Problem:** HTTP response has repeated patterns.

**Current:**
```llvm
@str_1 = "HTTP/1.1 200 OK\r\n"
@str_2 = "Content-Type: text/html\r\n"
@str_3 = "Content-Length: 1094\r\n"
@str_4 = "Connection: close\r\n\r\n"
@str_5 = "<!DOCTYPE html>..."
```

**Optimized:**
```llvm
@http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1094\r\nConnection: close\r\n\r\n<!DOCTYPE html>..."
```

**Implementation:**
1. Detect consecutive string sends
2. Merge into single constant
3. Single send() call

### 3. PE Header Optimization (Target: -0.5 KB)

**Problem:** Default LLVM/Clang PE headers are bloated.

**Current:** Standard PE with full headers
**Optimized:** Minimal PE with custom linker script

```
; Custom linker script
SECTIONS {
    .text : { *(.text*) }
    .rdata : { *(.rdata*) }
}
```

**Clang flags:**
```bash
clang -Wl,/MERGE:.rdata=.text -Wl,/ALIGN:16 -Wl,/FILEALIGN:16
```

### 4. Function Inlining (Target: -0.3 KB)

**Problem:** Small functions have call overhead.

**Current:**
```llvm
define i64 @sys_send(i64 %sock, i8* %buf, i64 %len) {
    ; wrapper code
}

call i64 @sys_send(...)
```

**Optimized:** Inline small wrappers
```llvm
; Direct inline at call site
%result = call i32 @send(i64 %sock, i8* %buf, i32 %len, i32 0)
```

### 5. Alignment Reduction (Target: -0.2 KB)

**Problem:** Default 4096-byte section alignment wastes space.

**Solution:**
```bash
clang -Wl,/ALIGN:16 -Wl,/FILEALIGN:16
```

---

## Implementation Plan

### Phase 1: Analysis Pass (v0.14.1)

```rust
// Add to ir.rs
pub struct UsageAnalysis {
    pub used_ops: HashSet<Op>,
    pub used_strings: HashSet<String>,
    pub call_graph: HashMap<String, Vec<String>>,
}

impl UsageAnalysis {
    pub fn analyze(graph: &Graph) -> Self {
        let mut analysis = UsageAnalysis::new();
        for node in &graph.nodes {
            analysis.visit_node(node);
        }
        analysis
    }
}
```

### Phase 2: Conditional Emission (v0.14.2)

```rust
// Modify emitter_pure.rs
fn emit_graph(&mut self, graph: &Graph) -> Result<String, String> {
    let analysis = UsageAnalysis::analyze(graph);
    
    // Only emit used syscalls
    ir.push_str(&self.emit_syscalls_for(&analysis.used_ops));
    
    // Deduplicate strings
    let merged_strings = self.merge_strings(&analysis.used_strings);
    
    // ...
}
```

### Phase 3: Linker Optimization (v0.14.3)

```rust
// Add to main.rs
fn get_optimized_clang_flags() -> Vec<&'static str> {
    vec![
        "-Os",                    // Optimize for size
        "-fno-exceptions",        // No C++ exceptions
        "-fno-rtti",              // No RTTI
        "-Wl,/MERGE:.rdata=.text",
        "-Wl,/ALIGN:16",
        "-Wl,/FILEALIGN:16",
        "-Wl,/OPT:REF",           // Remove unreferenced
        "-Wl,/OPT:ICF",           // Identical COMDAT folding
    ]
}
```

---

## Expected Results

| Version | Binary Size | Reduction |
|---------|-------------|-----------|
| v0.14 (current) | 7,168 B | baseline |
| v0.14.1 (analysis) | 7,168 B | 0% |
| v0.14.2 (dead code) | 4,096 B | -43% |
| v0.14.3 (strings) | 3,584 B | -50% |
| v0.15 (linker) | 2,560 B | -64% |
| v0.16 (inline) | 2,048 B | -71% |

**Target: 2 KB binary for HTTP server**

---

## Verification

### Size Check Script
```powershell
# Check binary size
$size = (Get-Item solar_nelaia_16w.exe).Length
Write-Host "Binary size: $size bytes ($([math]::Round($size/1024, 2)) KB)"

# Section analysis
dumpbin /headers solar_nelaia_16w.exe | Select-String "size of"
```

### Comparison Benchmark
```powershell
# Compare all versions
Get-ChildItem solar_*.exe | ForEach-Object {
    Write-Host "$($_.Name): $($_.Length) bytes"
}
```

---

## Design Notes

**Why binary size matters for AI:**

1. **Deployment speed** - Smaller = faster transfer
2. **Memory efficiency** - Less RAM per instance
3. **Cache utilization** - Fits in L1/L2 cache
4. **Token economy** - Smaller output = fewer tokens
5. **Verification** - Smaller = easier to verify

**Target hierarchy:**
```
1 KB = Excellent (fits in L1 cache)
2 KB = Good (fits in L2 cache)
4 KB = Acceptable (one page)
8 KB = Current (needs optimization)
```

*Binary optimization is prerequisite for stable release.*
