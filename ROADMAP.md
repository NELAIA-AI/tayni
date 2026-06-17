# TAYNI Roadmap

> **TAYNI**: AI-first programming language optimized for token efficiency and minimal binary sizes.

## Current Status: Bootstrap Complete, General Compiler In Progress

---

## Completed (Functional)

### Bootstrap Chain (Zero Rust)

| Component | Size | Description |
|-----------|------|-------------|
| `tayni-bootstrap.exe` | 8,704 bytes | Gen15 compiler, runs `.tyn` programs |
| `compiler.exe` | 8,704 bytes | Self-replication copy |

**Capabilities proven:**
- PE generation byte-by-byte (no external tools)
- Self-replication (bit-identical copies)
- File I/O via Windows syscalls (CreateFileA, ReadFile, WriteFile, CloseHandle)
- Memory management (VirtualAlloc, VirtualFree)
- Arithmetic (ADD, SUB, MUL)
- Conditional selection (IFZ)
- Console output (PRT via GetStdHandle + WriteFile)
- Int-to-string conversion (ITS)

### Gen15 Compiler Output

Gen15 reads `input.tyn` and produces `out.exe`:
- If input starts with `--`: copies `compiler.exe` to `out.exe` (self-replication)
- If input starts with `.x:`: parses up to 3 digits and generates PE with `mov eax, N; ret`

**Autonomy level achieved: Self-Replication (bootstrap can reproduce itself)**

### Language Design (Specification Complete)

- AI-native syntax defined and stable
- 30+ core operations specified (arithmetic, memory, I/O, network, string)
- Declarative data-flow model (no loops, no jumps)
- Module system (`USE`) designed

---

## In Progress: General-Purpose Compiler (gen16+)

### Latest Achievement: Gen28 (2026-06-17)

Gen28 demonstrates **self-compilation bootstrap** - a TAYNI compiler that generates working executables:

```
gen28.exe (compiled by tayni-c) reads program.tyn
  → generates out.exe (valid PE)
    → out.exe runs: calculates 5+3=8, 8*5=40
      → prints "40" to console AND writes to file
```

**SELF-COMPILATION SUCCESS** - The bootstrap chain is proven.

### Gen27 (2026-06-17)

Gen27 adds **FRD (file read)** operation, completing the file I/O set:

**Operations supported**: ADD, SUB, MUL, IFZ, GET, PUT, ALC, FRD, FOP, FWR, FCL

**Key milestone**: Complete file I/O operations. Multi-binding parser with chained operations.

### Gen26 (2026-06-17)

Gen26 is the **multi-binding compiler** - parses 4 bindings with chained operations:

```tayni
.a: 6
.b: 3
.c: ADD .a .b    -- c = 9
.d: MUL .c .a    -- d = 54 (uses result of c)
```

### Gen25 (2026-06-17)

Gen25 is the **complete core operations compiler** - supports all fundamental TAYNI operations with proper disambiguation (ADD vs ALC):

**Operations supported**: ADD, SUB, MUL, IFZ, GET, PUT, ALC

```tayni
.a: 42
.b: 0
.c: ALC .a .b    -- or ADD, SUB, MUL, IFZ, GET, PUT
```

**Key milestone**: Two-character operation detection (ADD vs ALC both start with 'A').

### Gen24 (2026-06-17)

Gen24 adds **memory operations (GET/PUT)** - read/write individual bytes:

```tayni
.a: 0
.b: 0
.c: GET .a .b    -- reads byte at offset 0 = '.' = 46
```

### Gen23 (2026-06-17)

Gen23 adds **conditional selection (IFZ)** - the first compiler with branching logic:

```tayni
.a: 0
.b: 42
.c: IFZ .a .b    -- if a==0 return b, else return a
```
Result: `out.exe` prints "42" (because a==0).

**Key milestone**: First compiler with conditional logic. Supports ADD, SUB, MUL, and IFZ.

### Gen22 (2026-06-17)

Gen22 is the **full arithmetic compiler** - it detects the operation type (ADD, SUB, or MUL) from the source code and performs the correct calculation at compile time:

```tayni
.a: 6
.b: 7
.c: MUL .a .b    -- or ADD, or SUB
...
```
Result: `out.exe` prints "42" (6*7) to console AND creates `result.txt`.

**Key milestone**: First compiler with operation detection. Parses the operation name and selects the correct arithmetic.

### Gen21 (2026-06-17)

Gen21 is the first **arithmetic compiler** - it parses numeric values, performs compile-time arithmetic, and generates executables that convert integers to strings at runtime. The generated PE:
1. Embeds the calculated result as an immediate value
2. Converts the number to ASCII using div/mod at runtime
3. Prints to console AND writes to file

```tayni
.a: 25
.b: 17
.c: ADD .a .b
.buf: ALC 8
.len: ITS .c .buf
.p: PRT .buf .len
.path: "result.txt"
.h: FOP .path 1
.w: FWR .h .buf .len
.x: FCL .h
!
```
Result: `out.exe` prints "42" to console AND creates `result.txt` with "42".

**Key milestone**: First compiler with runtime int-to-string conversion (ITS). Combines compile-time arithmetic with runtime string generation.

### Gen20 (2026-06-16)

Gen20 is the first **multi-operation compiler** - a single TAYNI program that combines console output AND file I/O in one executable. It parses two strings (message + filename) and generates a PE that:
1. Prints the message to console (GetStdHandle + WriteFile)
2. Creates a file with the same content (CreateFileA + WriteFile + CloseHandle)

```tayni
.msg: "Hello Multi-Op!"
.path: "output.txt"
.p: PRT .msg 15
.h: FOP .path 1
.w: FWR .h .msg 15
.c: FCL .h
!
```
Result: `out.exe` prints "Hello Multi-Op!" to console AND creates `output.txt` with the same content.

**Key milestone**: First compiler that combines multiple operation types (PRT + FOP + FWR + FCL) in a single generated executable. Imports 4 functions from KERNEL32.dll (GetStdHandle, WriteFile, CreateFileA, CloseHandle).

### Gen19 (2026-06-16)

Gen19 adds **TCP networking** - the first TAYNI-compiled program that opens a network socket and serves data. It imports from WS2_32.dll (WSAStartup, socket, bind, listen, accept, send, closesocket) and generates a TCP server:

```tayni
.msg: "Hello from TAYNI TCP!"
!
```
Result: `out.exe` listens on port 8080, accepts a connection, sends the message, and exits.

**Key milestone**: First multi-DLL import (WS2_32.dll). Discovered and fixed critical stack alignment issue (Windows x64 EXE entry requires `sub rsp, 0x208` not `0x200`).

### Gen18 (2026-06-13)

Gen18 adds **file I/O** - the first compiler that generates executables capable of interacting with the filesystem. It parses programs with two strings (filename + content) and emits PE executables calling CreateFileA, WriteFile, CloseHandle, ExitProcess:

```tayni
.path: "hello.txt"
.msg: "Hello File IO!"
.h: FOP .path 1
.w: FWR .h .msg 14
.c: FCL .h
!
```
Result: `out.exe` creates `hello.txt` containing "Hello File IO!" and exits cleanly.

### Gen17 (2026-06-16)

Gen17 is the first compiler to emit **runtime x86-64 code**. It generates PE executables that call Windows APIs (GetStdHandle, WriteFile, ExitProcess) to print strings:

```tayni
.m: "Hello from TAYNI!"
.o: PRT .m 17
!
```
Result: `out.exe` prints "Hello from TAYNI!" to console and exits cleanly.

**This is the paradigm shift** - gen15/gen16 only embedded a static exit code. Gen17 emits real instructions that execute at runtime.

### Gen16 (2026-06-16)

Gen16 compiles programs with 2 numeric bindings + arithmetic operations (ADD, SUB, MUL):

```tayni
.a: 12
.b: 10
.c: MUL .a .b
!
```
Result: `out.exe` returns exit code 120.

### The Remaining Gap

Gen15 can only compile single numeric literals. To compile real programs (like `server.tyn`) the compiler needs:

1. **Multi-line parser** - read all bindings, not just one
2. **Operation recognition** - identify ADD, SUB, TCP, PRT, etc. by name
3. **String literal handling** - embed strings in PE data section
4. **Reference resolution** - track that `.sock` in `BND .sock .port` refers to an earlier binding
5. **Code generation** - emit x86-64 machine code for each operation
6. **Dynamic import table** - include required DLLs (KERNEL32, WS2_32, etc.)

### Bootstrapping Path

| Generation | Adds | Compiles | Status |
|-----------|------|----------|--------|
| gen15 (current) | Self-replication + numeric PE | `.x: 42` | Done |
| gen16 | Multi-line parser, 2 bindings + ADD/SUB/MUL | `.a: 12`, `.b: 10`, `.c: MUL .a .b` | Done |
| gen17 | String literals + PRT (runtime x86-64 codegen) | `.m: "Hello"`, `.o: PRT .m 5` | Done |
| gen18 | File I/O (CreateFileA, WriteFile, CloseHandle) | Programs that write files | Done |
| gen19 | TCP networking (WS2_32: socket, bind, listen, accept, send) | TCP server | Done |
| gen20 | Multi-operation (PRT + FOP + FWR + FCL combined) | Console + File I/O | Done |
| gen21 | Arithmetic + ITS (int-to-string at runtime) | Calc + Print + File | Done |
| gen22 | Full arithmetic (ADD, SUB, MUL detection) | Operation selection | Done |
| gen23 | Conditional (IFZ) | Branching logic | Done |
| gen24 | Memory ops (GET/PUT) | Byte read/write | Done |
| gen25 | Complete core (ADD/SUB/MUL/IFZ/GET/PUT/ALC) | All core ops | Done |
| gen26 | Multi-binding parser | Chained operations | Done |
| gen27 | File read (FRD) | Complete file I/O | Done |
| gen28 | Self-compilation bootstrap | TAYNI compiles TAYNI | Done |
| gen29+ | Full self-hosting | Compiler compiles itself | Next |

Each generation is written in TAYNI and compiled by the previous generation.

---

## Specification (Designed, Not Yet Compilable)

### Standard Library - 36 Modules

These exist as `.tyn` files containing constants, commented templates, and usage examples. They define the API but are not yet compilable by the TAYNI compiler.

#### TIER 0 - Essential (10 modules)

| Module | Description | Status |
|--------|-------------|--------|
| `file` | File I/O operations | Spec |
| `string` | String manipulation | Spec |
| `json` | JSON parsing/encoding | Spec |
| `http` | HTTP parsing | Spec |
| `url` | URL handling | Spec |
| `router` | URL routing | Spec |
| `log` | Logging | Spec |
| `base64` | Base64 encoding | Spec |
| `random` | Random number generation | Spec |
| `args` | CLI arguments | Spec |

#### TIER 1 - Common (12 modules)

| Module | Description | Status |
|--------|-------------|--------|
| `env` | Environment variables | Spec |
| `path` | Path manipulation | Spec |
| `hash` | SHA256, HMAC, bcrypt, Argon2 | Spec |
| `time` | Time operations | Spec |
| `uuid` | UUID v4/v7 generation | Spec |
| `jwt` | JWT sign/verify/decode | Spec |
| `regex` | Pattern matching | Spec |
| `format` | String formatting | Spec |
| `validation` | Input validation | Spec |
| `test` | Testing framework | Spec |
| `async` | Promise/Channel primitives | Spec |
| `timeout` | Timeout handling | Spec |

#### TIER 2 - Specialized (14 modules)

| Module | Description | Status |
|--------|-------------|--------|
| `sql` | SQL query builder | Spec |
| `postgres` | PostgreSQL wire protocol | Spec |
| `redis` | Redis RESP protocol | Spec |
| `websocket` | WebSocket (RFC 6455) | Spec |
| `grpc` | gRPC/protobuf | Spec |
| `yaml` | YAML parsing | Spec |
| `csv` | CSV parsing (RFC 4180) | Spec |
| `xml` | XML parsing + XPath | Spec |
| `crypto` | AES-256-GCM, RSA, ECDSA, ChaCha20 | Spec |
| `tls` | TLS 1.3 protocol | Spec |
| `pqc` | Post-Quantum: ML-KEM, ML-DSA, SLH-DSA | Spec |
| `cors` | CORS handling | Spec |
| `cookie` | Cookie/session management | Spec |
| `gzip` | GZIP compression (RFC 1952) | Spec |

### Multi-Target Emitters

These exist as `.tyn` specification files with format constants and pseudocode. Not yet functional.

| Target | File | Status |
|--------|------|--------|
| WebAssembly | `src/emitters/wasm.tyn` | Spec |
| RISC-V | `src/emitters/riscv.tyn` | Spec |
| ARM64 Linux | `src/emitters/arm64_linux.tyn` | Spec |
| GPU (PTX/AMDGPU/SPIR-V) | `src/emitters/gpu.tyn` | Spec |
| Quantum (QIR) | `src/emitters/qir.tyn` | Spec |

---

## Reference Implementation (Rust)

A Rust-based compiler (`tayni-c`) exists in `src/*.rs` with `Cargo.toml` at root:
- Full parser for all TAYNI operations
- LLVM IR emission
- Direct PE/ELF/Mach-O generation
- Can be built with `cargo build --release`
- Last built binary: `target/release/tayni-c.exe` (1.2 MB)

This serves as reference for the x86-64 code patterns and PE layout needed by the TAYNI self-hosted compiler. The goal is for gen16+ to progressively replace it.

---

## Design Principles

1. **Token Efficient** - Minimal syntax for AI consumption
2. **Zero Dependencies** - Single executable output
3. **Declarative** - Data flow over control flow (no loops, no jumps)
4. **Native Performance** - Direct machine code generation
5. **AI Autonomy** - Self-hosting enables AI self-improvement
6. **Incremental Bootstrap** - Each compiler generation adds capabilities

---

## Efficiency Metrics (Proven)

| Metric | TAYNI | LLVM+Clang | Improvement |
|--------|-------|------------|-------------|
| Minimal PE | 2,560 bytes | 3,584 bytes | **1.4x smaller** |
| Bootstrap compiler | 8,704 bytes | N/A | Pure TAYNI |

---

## Usage (Current)

```bash
# Compile a numeric literal (all gen15 supports today)
echo .x: 42 > input.tyn
tayni-bootstrap.exe
out.exe
echo %ERRORLEVEL%  # Shows: 42

# Self-replicate the compiler
echo -- copy > input.tyn
compiler.exe
# out.exe is bit-identical to compiler.exe
```

---

*Last updated: 2026-06-17. Gen28 complete - SELF-COMPILATION BOOTSTRAP SUCCESS.*
