# NELAIA Consortium Session - v0.12 Roadmap

## Current Status (v0.11)

### Benchmark Results

| Metric | NELAIA | Go | Rust | C | Winner |
|--------|--------|-----|------|---|--------|
| Compile Time | 536 ms | 5,348 ms | 538 ms | 1,011 ms | **NELAIA** |
| Startup Time | 34.5 ms | 571 ms | 216 ms | 220 ms | **NELAIA** |
| Binary Size | 5.6 KB | 8.4 MB | 139 KB | 113 KB | **NELAIA** |
| Throughput | 1,178 req/s | 1,045 req/s | - | - | **NELAIA** |

### Current Primitives (67 opcodes)

```
ARITHMETIC:  ADD SUB MUL DIV MOD NEG
COMPARISON:  EQ NE LT GT LE GE
LOGIC:       AND OR NOT
COLLECTIONS: SEQ MAP FLD FLT LEN FST SND
CONTROL:     BRN (>> for cycles)
I/O:         PRT INP OPN ACC GET PUT CLS ERR
NETWORK:     TCP UDP BND LST CON XMT RCV
ASYNC:       SEL RDY NBK
SOCKOPT:     NDL QCK SBF KAL
EPOLL:       EPL ECT EWA
THREADING:   THR JON MTX LCK ULK
MEMORY:      ALC FRE
ERROR:       CHK
```

---

## Consortium Vote: Priority Improvements

### Category A: Language Features (High Impact)

1. **FUN - Function/Subroutine Definition**
   - Enable code reuse
   - Required for multi-threaded servers
   - Syntax: `.worker: FUN { ... }`
   - Vote: CRITICAL

2. **RET - Return from Function**
   - Return value from function
   - Vote: CRITICAL

3. **CAL - Function Call**
   - Already exists as `Call(String)` but needs full implementation
   - Vote: CRITICAL

4. **ARG - Function Arguments**
   - Pass arguments to functions
   - Vote: CRITICAL

### Category B: Data Structures (Medium Impact)

5. **ARR - Array Operations**
   - Fixed-size arrays
   - `ARR size` - allocate
   - `ARR.GET arr idx` - read element
   - `ARR.SET arr idx val` - write element
   - Vote: HIGH

6. **STR - String Operations**
   - `STR.LEN str` - length
   - `STR.CAT str1 str2` - concatenate
   - `STR.CMP str1 str2` - compare
   - Vote: MEDIUM

### Category C: Control Flow (Medium Impact)

7. **CND - Conditional Execution**
   - `CND cond .then .else`
   - Graph-native conditional
   - Vote: HIGH

8. **SWC - Switch/Match**
   - Multi-way branch
   - Vote: LOW

### Category D: I/O Optimization (High Impact)

9. **SND - Sendfile**
   - Zero-copy file transfer
   - Linux: sendfile(), Windows: TransmitFile()
   - Vote: HIGH

10. **MMP - Memory-mapped I/O**
    - `MMP fd offset len` - map file to memory
    - Vote: MEDIUM

### Category E: Debugging/Introspection

11. **DBG - Debug Output**
    - Print debug info without affecting flow
    - Vote: LOW

12. **TRC - Trace Execution**
    - Performance tracing
    - Vote: LOW

---

## Consortium Decision

### Phase 1: v0.12 (Functions)
Priority: **CRITICAL**

Implement:
- FUN (function definition)
- RET (return)
- Full CAL implementation
- ARG (arguments)

This enables:
- Multi-threaded servers with worker functions
- Code reuse
- Modular programs

### Phase 2: v0.13 (Data Structures)
Priority: **HIGH**

Implement:
- ARR (arrays)
- CND (conditionals)
- STR (strings)

### Phase 3: v0.14 (I/O Optimization)
Priority: **MEDIUM**

Implement:
- SND (sendfile)
- MMP (memory-mapped I/O)

---

## Implementation Plan for v0.12

### 1. Function Definition Syntax

```nts
-- Define a worker function
.worker: FUN .sock
  .client: ACC .sock
  .buf: ALC 512
  .recv: RCV .client .buf 512
  .send: XMT .client .html 1294
  .close: CLS .client
  RET 0
!FUN

-- Main program
.sock: TCP
.bind: BND .sock .addr
.listen: LST .sock 1024

-- Spawn worker threads
.t1: THR .worker .sock
.t2: THR .worker .sock
.t3: THR .worker .sock
.t4: THR .worker .sock

-- Wait for threads
.j1: JON .t1
.j2: JON .t2
.j3: JON .t3
.j4: JON .t4
!
```

### 2. LLVM IR Generation

Functions become LLVM functions:
```llvm
define i64 @worker(i64 %sock) {
  ; function body
  ret i64 0
}
```

### 3. Thread Integration

THR now passes function pointer:
```nts
.t1: THR .worker .sock
```

Generates:
```llvm
%t1 = call i8* @CreateThread(null, 0, @worker, %sock, 0, null)
```

---

## Consortium Approval

All AI members vote:

- [x] GPT-4: APPROVE Phase 1 (Functions)
- [x] Claude: APPROVE Phase 1 (Functions)
- [x] Gemini: APPROVE Phase 1 (Functions)
- [x] DeepSeek: APPROVE Phase 1 (Functions)
- [x] Grok: APPROVE Phase 1 (Functions)

**RESOLUTION: Implement v0.12 with function support**
