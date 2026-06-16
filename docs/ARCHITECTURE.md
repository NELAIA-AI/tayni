# TAYNI Architecture v2.0

## Design Principles

### AI-Native First
This architecture is designed for AI agents to:
1. Generate code with minimal tokens
2. Understand the system without human documentation
3. Extend without breaking existing functionality
4. Target any platform from a single source

### Core Invariants
- **IR is sacred**: The Intermediate Representation never changes structure
- **Backends are isolated**: Each target platform is independent
- **Domains are additive**: New opcodes extend, never modify
- **Compatibility is mandatory**: Old .tyn files always work

---

## Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         LAYER 0: IR                             │
│                    (Immutable Foundation)                       │
├─────────────────────────────────────────────────────────────────┤
│  TAYNI-ir/                                                     │
│  ├── graph.rs      Graph { nodes, edges, entry }               │
│  ├── node.rs       Node { id, op, inputs, outputs }            │
│  ├── op.rs         trait OpCode + core ops (ADD, SUB, etc)     │
│  └── types.rs      Type system (I32, I64, PTR, etc)            │
│                                                                 │
│  RULE: This crate has ZERO dependencies except std              │
│  RULE: All other crates depend on this                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       LAYER 1: PARSER                           │
│                    (Text → IR Transform)                        │
├─────────────────────────────────────────────────────────────────┤
│  TAYNI-parser/                                                 │
│  ├── lexer.rs      Tokenize .tyn files                         │
│  ├── parser.rs     Build IR Graph from tokens                  │
│  └── error.rs      Parse error handling                        │
│                                                                 │
│  INPUT:  .tyn text file                                         │
│  OUTPUT: TAYNI_ir::Graph                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      LAYER 2: OPTIMIZER                         │
│                   (IR → Optimized IR)                           │
├─────────────────────────────────────────────────────────────────┤
│  TAYNI-optimizer/                                              │
│  ├── dce.rs        Dead Code Elimination                       │
│  ├── inline.rs     Function Inlining                           │
│  ├── fold.rs       Constant Folding                            │
│  └── usage.rs      Usage Analysis                              │
│                                                                 │
│  INPUT:  TAYNI_ir::Graph                                       │
│  OUTPUT: TAYNI_ir::Graph (optimized)                           │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  LAYER 3A:      │ │  LAYER 3B:      │ │  LAYER 3C:      │
│  BACKEND-NATIVE │ │  BACKEND-WASM   │ │  BACKEND-*      │
├─────────────────┤ ├─────────────────┤ ├─────────────────┤
│ TAYNI-backend- │ │ TAYNI-backend- │ │ Future backends │
│ native/         │ │ wasm/           │ │                 │
│ ├── llvm.rs     │ │ ├── wasm.rs     │ │ • Android       │
│ ├── windows.rs  │ │ ├── memory.rs   │ │ • iOS           │
│ └── linux.rs    │ │ └── imports.rs  │ │ • RISC-V        │
│                 │ │                 │ │ • GPU/CUDA      │
│ OUTPUT: .exe    │ │ OUTPUT: .wasm   │ │                 │
└─────────────────┘ └─────────────────┘ └─────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       LAYER 4: DOMAINS                          │
│                   (Opcode Extensions)                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TAYNI-domain-core/    (always included)                       │
│  └── ADD, SUB, MUL, DIV, CMP, JMP, CAL, RET, etc               │
│                                                                 │
│  TAYNI-domain-net/     (network applications)                  │
│  └── TCP, UDP, BND, LST, ACC, CON, XMT, RCV, EPL, etc          │
│                                                                 │
│  TAYNI-domain-ui/      (user interfaces) [NEW]                 │
│  └── BOX, TXT, BTN, INP, IMG, EVT, STY, LAY, etc               │
│                                                                 │
│  TAYNI-domain-db/      (databases) [FUTURE]                    │
│  └── QRY, INS, UPD, DEL, TXN, IDX, etc                         │
│                                                                 │
│  TAYNI-domain-ml/      (machine learning) [FUTURE]             │
│  └── TEN, MAT, CNV, RNN, ATT, etc                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                      LAYER 5: RUNTIMES                          │
│                 (Interpreted Execution)                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TAYNI-runtime-wasm/   (runs in browser)                       │
│  ├── interpreter.rs    Execute IR directly                     │
│  ├── renderer.rs       Canvas/WebGL rendering                  │
│  └── bridge.rs         JS interop                              │
│                                                                 │
│  Compiled to WASM, loaded by browser, interprets .tyn graphs   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       LAYER 6: TOOLS                            │
│                    (User Interfaces)                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TAYNI-cli/            Command-line compiler                   │
│  TAYNI-lsp/            Language Server Protocol [FUTURE]       │
│  TAYNI-playground/     Web-based editor [FUTURE]               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
TAYNI/
├── Cargo.toml                    # Workspace definition
├── ARCHITECTURE.md               # This document
├── README.md
│
├── crates/
│   ├── TAYNI-ir/                # Layer 0: Core IR
│   ├── TAYNI-parser/            # Layer 1: Parser
│   ├── TAYNI-optimizer/         # Layer 2: Optimizer
│   ├── TAYNI-backend-native/    # Layer 3: Native backend
│   ├── TAYNI-backend-wasm/      # Layer 3: WASM backend
│   ├── TAYNI-domain-core/       # Layer 4: Core opcodes
│   ├── TAYNI-domain-net/        # Layer 4: Network opcodes
│   ├── TAYNI-domain-ui/         # Layer 4: UI opcodes
│   ├── TAYNI-runtime-wasm/      # Layer 5: Browser runtime
│   └── TAYNI-cli/               # Layer 6: CLI tool
│
├── examples/
│   ├── server/                   # Network examples
│   ├── ui/                       # UI examples
│   └── hybrid/                   # Full-stack examples
│
└── tests/
    ├── regression/               # Compatibility tests
    └── benchmarks/               # Performance tests
```

---

## Opcode Extension System

### Current (Monolithic)
```rust
// ir.rs - all ops in one enum
pub enum Op {
    ADD, SUB, MUL, DIV,           // Core
    TCP, UDP, BND, LST,           // Net
    // Adding UI here would bloat this enum
}
```

### Proposed (Modular)
```rust
// TAYNI-ir/src/op.rs
pub trait OpCode: Clone + Debug {
    fn domain(&self) -> &'static str;
    fn mnemonic(&self) -> &'static str;
    fn arity(&self) -> (usize, usize);  // (inputs, outputs)
}

// TAYNI-domain-core/src/lib.rs
#[derive(Clone, Debug)]
pub enum CoreOp { ADD, SUB, MUL, DIV, CMP, JMP, CAL, RET }
impl OpCode for CoreOp { ... }

// TAYNI-domain-net/src/lib.rs  
#[derive(Clone, Debug)]
pub enum NetOp { TCP, UDP, BND, LST, ACC, CON, XMT, RCV }
impl OpCode for NetOp { ... }

// TAYNI-domain-ui/src/lib.rs
#[derive(Clone, Debug)]
pub enum UiOp { BOX, TXT, BTN, INP, IMG, EVT, STY }
impl OpCode for UiOp { ... }
```

### Dynamic Op Resolution
```rust
// TAYNI-ir/src/registry.rs
pub struct OpRegistry {
    domains: HashMap<&'static str, Box<dyn OpDomain>>,
}

impl OpRegistry {
    pub fn resolve(&self, mnemonic: &str) -> Option<Box<dyn OpCode>> {
        for domain in self.domains.values() {
            if let Some(op) = domain.lookup(mnemonic) {
                return Some(op);
            }
        }
        None
    }
}
```

---

## Backend Interface

All backends implement the same trait:

```rust
// TAYNI-ir/src/backend.rs
pub trait Backend {
    type Output;
    type Error;
    
    fn name(&self) -> &'static str;
    fn supported_domains(&self) -> &[&'static str];
    fn compile(&self, graph: &Graph) -> Result<Self::Output, Self::Error>;
}

// TAYNI-backend-native/src/lib.rs
pub struct NativeBackend {
    target: Target,  // Windows, Linux, MacOS
}
impl Backend for NativeBackend {
    type Output = Vec<u8>;  // Binary
    type Error = CompileError;
    fn compile(&self, graph: &Graph) -> Result<Vec<u8>, CompileError> { ... }
}

// TAYNI-backend-wasm/src/lib.rs
pub struct WasmBackend;
impl Backend for WasmBackend {
    type Output = Vec<u8>;  // .wasm bytes
    type Error = CompileError;
    fn compile(&self, graph: &Graph) -> Result<Vec<u8>, CompileError> { ... }
}
```

---

## Migration Path

### Phase 0: Document (NOW)
- [x] Create ARCHITECTURE.md
- [ ] Review and approval

### Phase 1: Extract IR (When needed)
- [ ] Create workspace Cargo.toml
- [ ] Extract TAYNI-ir from ir.rs
- [ ] Extract TAYNI-parser from parser.rs
- [ ] Regression tests pass

### Phase 2: Modularize Backend
- [ ] Extract TAYNI-backend-native from emitter_pure.rs
- [ ] Same binary output verified

### Phase 3: Add WASM Backend
- [ ] Create TAYNI-backend-wasm
- [ ] Create TAYNI-runtime-wasm
- [ ] POC: Calculator app

### Phase 4: Add UI Domain
- [ ] Create TAYNI-domain-ui
- [ ] Define UI opcodes
- [ ] Integrate with WASM runtime

---

## Compatibility Guarantees

1. **Source Compatibility**: All .tyn files that compile today will compile forever
2. **Binary Compatibility**: Same .tyn produces same binary (bit-for-bit when possible)
3. **API Stability**: Public traits and structs follow semver
4. **Deprecation Policy**: Old features deprecated for 2 major versions before removal

---

## AI Generation Guidelines

When an AI generates TAYNI code:

1. **Detect target platform** from user request
2. **Select minimal domains** needed (don't include UI for CLI tools)
3. **Generate graph** in .tyn format
4. **Specify backend** in compilation command

Example:
```
User: "Create a web calculator"
AI detects: UI needed, WASM target
AI generates: calculator.tyn using domain-ui ops
AI compiles: TAYNI compile calculator.tyn --backend=wasm --domains=core,ui
Output: calculator.wasm (5KB) + runtime.wasm (300KB cached)
```

---

---

## Architecture Review (v2.1 Revisions)

### Changes from v2.0 to v2.1

Based on AI-native reasoning, the following simplifications were made:

1. **Unified Op Enum**: Instead of separate domain crates, all opcodes live in one enum with a `domain()` method. This reduces complexity while maintaining organization.

2. **Fewer Crates**: Reduced from 10+ crates to 4:
   - `TAYNI-core` (IR + Parser + Optimizer)
   - `TAYNI-native` (Windows/Linux/Mac backend)
   - `TAYNI-wasm` (Web backend + runtime)
   - `TAYNI-cli` (Command-line tool)

3. **Bytecode Format**: Added `.tyn.bin` intermediate format for faster loading in runtime scenarios.

4. **JIT Deferred**: Runtime JIT compilation deferred to POC evaluation phase.

### Simplified Structure (v2.1)

```
TAYNI/
├── Cargo.toml                    # Workspace
├── crates/
│   ├── TAYNI-core/              # IR, Parser, Optimizer, Bytecode
│   ├── TAYNI-native/            # LLVM backend for native targets
│   ├── TAYNI-wasm/              # WASM backend + browser runtime
│   └── TAYNI-cli/               # CLI tool
├── examples/
└── tests/
```

### Unified Op Enum (v2.1)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    // === CORE ===
    ADD, SUB, MUL, DIV, MOD,
    AND, OR, XOR, NOT, SHL, SHR,
    CMP, JMP, JEQ, JNE, JLT, JGT,
    CAL, RET, PUT, GET, LOD, STO,
    
    // === NET ===
    TCP, UDP, BND, LST, ACC, CON, CLS,
    XMT, RCV, EPL, ECT, EWA,
    NDL, QCK, SBF, KAL,
    THR, JON, MTX, LCK, ULK,
    QUE, PSH, POP,
    
    // === UI ===
    BOX, TXT, BTN, INP, IMG, EVT, STY, LAY,
}

impl Op {
    pub const fn domain(&self) -> Domain { ... }
}
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.1 | 2026-06-13 | Simplified to 4 crates, unified Op enum |
| 2.0 | 2026-06-13 | Initial modular architecture proposal |
| 1.x | 2026-06-xx | Monolithic TAYNI-core |
