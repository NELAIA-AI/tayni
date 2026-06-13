# NELAIA Development Roadmap

## Vision

Transform NELAIA from a proof-of-concept into a **self-hosting, production-ready AI-Native Protocol** capable of building any software system without human-centric abstractions.

---

## Current State: v0.7 ✅

| Feature | Status |
|---------|--------|
| Data Flow Graph paradigm | ✅ |
| 40 opcodes | ✅ |
| Direct syscalls (Linux) | ✅ |
| kernel32/ws2_32 (Windows) | ✅ |
| File I/O | ✅ |
| TCP Networking | ✅ |
| Memory allocation | ✅ |
| Constant folding | ✅ |
| Graph analysis | ✅ |

---

## Phase 1: Foundation Completion (v0.8 - v0.9)

### v0.8 - Control Flow & Error Handling

| Feature | Description | Priority |
|---------|-------------|----------|
| **IP Parsing** | Parse "x.x.x.x" strings for `CON` | HIGH |
| **Error Codes** | Standard error constants (ERR_SOCK, ERR_BIND, etc.) | HIGH |
| **UDP Support** | `UDP` opcode for datagram sockets | HIGH |
| **LOOP** | Full loop implementation with break/continue | HIGH |
| **CHK** | Error checking opcode | MEDIUM |

**New Opcodes**:
```
UDP          - Create UDP socket
LOOP cond    - Loop while condition true
BRK          - Break from loop
CNT          - Continue to next iteration
CHK val err  - Check value, jump to error handler if negative
```

**Estimated Opcodes**: 45

---

### v0.9 - String & Network Enhancement

| Feature | Description | Priority |
|---------|-------------|----------|
| **DNS Resolution** | Hostname to IP (via OS) | HIGH |
| **Socket Options** | SO_REUSEADDR, TCP_NODELAY | MEDIUM |
| **String Ops** | CAT, CPY, CMP, IDX, SUB | HIGH |
| **Buffer Ops** | SET, ZRO, CPY (memory) | MEDIUM |
| **IPv6** | AF_INET6 support | LOW |

**New Opcodes**:
```
DNS "host"   - Resolve hostname to IP
OPT fd opt   - Set socket option
CAT a b      - Concatenate strings
CPY dst src  - Copy string/buffer
CMP a b      - Compare strings
IDX str ch   - Find character index
SUB str i n  - Substring
SET ptr val  - Set memory byte
ZRO ptr n    - Zero memory region
```

**Estimated Opcodes**: 55

---

## Phase 2: Self-Hosting (v1.0 - v1.2)

### v1.0 - Bootstrap Preparation

| Feature | Description | Priority |
|---------|-------------|----------|
| **Parser in NELAIA** | Rewrite parser.rs in NELAIA | CRITICAL |
| **IR in NELAIA** | Data structures in NELAIA | CRITICAL |
| **Standard Library** | Core functions in NELAIA | HIGH |

**Goal**: NELAIA compiler can parse `.nts` files using NELAIA code.

---

### v1.1 - Emitter Bootstrap

| Feature | Description | Priority |
|---------|-------------|----------|
| **LLVM IR Generation** | Emit LLVM IR from NELAIA | CRITICAL |
| **String Builder** | Efficient string construction | HIGH |
| **File Writer** | Write output files | HIGH |

**Goal**: NELAIA can generate `.ll` files.

---

### v1.2 - Full Self-Hosting

| Feature | Description | Priority |
|---------|-------------|----------|
| **Complete Compiler** | Full nelaia-c in NELAIA | CRITICAL |
| **Remove Rust** | Rust becomes optional bootstrap | HIGH |
| **Optimization** | Self-optimizing compiler | MEDIUM |

**Goal**: `nelaia-c.nts` compiles itself.

---

## Phase 3: Ecosystem (v1.3 - v2.0)

### v1.3 - Developer Tools

| Feature | Description |
|---------|-------------|
| **Debugger** | Step-through execution |
| **Profiler** | Performance analysis |
| **LSP** | Language server for IDEs |
| **REPL** | Interactive execution |

---

### v1.4 - Advanced Features

| Feature | Description |
|---------|-------------|
| **Async I/O** | Non-blocking operations |
| **Threads** | Multi-threading support |
| **IPC** | Inter-process communication |
| **Signals** | Signal handling |

---

### v1.5 - Platform Expansion

| Feature | Description |
|---------|-------------|
| **macOS** | Darwin syscalls |
| **ARM64** | AArch64 support |
| **RISC-V** | RISC-V backend |
| **WebAssembly** | WASM target |

---

### v2.0 - AI Integration

| Feature | Description |
|---------|-------------|
| **AI Optimizer** | AI-driven code optimization |
| **Intent Compiler** | Natural language → NELAIA |
| **Self-Improvement** | AI modifies its own protocol |
| **Consortium API** | Multi-AI collaboration interface |

---

## Opcode Evolution

| Version | Opcodes | Category Additions |
|---------|---------|-------------------|
| v0.7 | 40 | Network, Memory |
| v0.8 | 45 | Control flow, UDP, Error handling |
| v0.9 | 55 | Strings, DNS, Socket options |
| v1.0 | 60 | Bootstrap utilities |
| v1.5 | 70 | Async, Threads |
| v2.0 | 80+ | AI-specific operations |

---

## Purity Commitment

Throughout all versions, NELAIA maintains **absolute purity**:

| Platform | Allowed Dependencies |
|----------|---------------------|
| Linux | Kernel syscalls only |
| Windows | kernel32.dll, ws2_32.dll, ntdll.dll |
| macOS | libSystem.B.dylib (kernel interface) |

**Never allowed**: libc, glibc, musl, MSVC CRT, or any external library.

---

## Success Metrics

### v1.0 Milestone
- [ ] Compiler self-hosts
- [ ] Binary size < 50KB
- [ ] Compilation speed < 100ms for 1000 lines
- [ ] Zero external dependencies

### v2.0 Milestone
- [ ] AI can modify NELAIA specification
- [ ] Multi-AI consortium can collaborate
- [ ] Production systems built in NELAIA
- [ ] Community adoption

---

## Timeline Estimate

| Phase | Versions | Focus |
|-------|----------|-------|
| Foundation | v0.8 - v0.9 | Complete language features |
| Bootstrap | v1.0 - v1.2 | Self-hosting compiler |
| Ecosystem | v1.3 - v1.5 | Tools and platforms |
| AI Native | v2.0+ | Full AI integration |

---

## Immediate Next Steps (v0.8)

1. **IP Address Parsing** - Parse "192.168.1.1" format
2. **Error Code System** - Define standard error constants
3. **UDP Sockets** - Datagram support
4. **LOOP Implementation** - Iteration with conditions
5. **Consortium Review** - Validate design decisions

Let's build the future of AI-Native programming.
