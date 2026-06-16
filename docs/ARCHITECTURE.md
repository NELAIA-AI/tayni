# TAYNI Architecture v3.0

## Overview

TAYNI is an AI-first programming language designed for:
1. **Token efficiency** - Minimal tokens for maximum functionality
2. **Multi-target compilation** - Single source to any platform
3. **Self-hosting** - Compiler written in TAYNI itself
4. **Zero dependencies** - Direct binary emission without external tools

## Current Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           SOURCE (.tyn)                                  │
│                    TAYNI source code with USE directives                 │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PARSER (parser.rs)                               │
│  • Lexer: tokenizes .tyn files                                          │
│  • Parser: builds IR Graph from tokens                                   │
│  • USE handler: resolves module imports (modules.rs)                     │
│  • Tree-shaking: eliminates dead code                                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    INTERMEDIATE REPRESENTATION (ir.rs)                   │
│  • Graph: nodes + edges + entry point                                    │
│  • Node: id, operation, arguments                                        │
│  • Op: 100+ operations across domains                                    │
│  • Capability system: declares required permissions                      │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
│  CLASSICAL CPU    │  │     QUANTUM       │  │       GPU         │
│    BACKENDS       │  │     BACKEND       │  │     BACKENDS      │
├───────────────────┤  ├───────────────────┤  ├───────────────────┤
│ • pe.rs (Windows) │  │ • qir.rs          │  │ • gpu.rs          │
│ • elf.rs (Linux)  │  │   Native: QIR     │  │   Native: PTX     │
│ • macho.rs (macOS)│  │   Export: QASM    │  │   Native: AMDGPU  │
│ • wasm.rs (Web)   │  │   Export: Cirq    │  │   Export: OpenCL  │
│ • riscv.rs        │  │   Export: Quil    │  │   Export: SPIR-V  │
│ • elf_arm64.rs    │  │                   │  │   Export: WGSL    │
│                   │  │                   │  │   Export: Metal   │
└───────────────────┘  └───────────────────┘  └───────────────────┘
              │                     │                     │
              ▼                     ▼                     ▼
┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
│  OUTPUT FORMATS   │  │  OUTPUT FORMATS   │  │  OUTPUT FORMATS   │
├───────────────────┤  ├───────────────────┤  ├───────────────────┤
│ • .exe (PE)       │  │ • .qir (native)   │  │ • .ptx (CUDA)     │
│ • ELF (Linux)     │  │ • .qasm (IBM)     │  │ • .amdgpu (ROCm)  │
│ • Mach-O (macOS)  │  │ • .py (Cirq)      │  │ • .cl (OpenCL)    │
│ • .wasm (Web)     │  │ • .quil (Rigetti) │  │ • .spvasm (Vulkan)│
│ • ELF-RISCV64     │  │                   │  │ • .wgsl (WebGPU)  │
│ • ELF-ARM64       │  │                   │  │ • .metal (Apple)  │
└───────────────────┘  └───────────────────┘  └───────────────────┘
```

## File Structure

```
tayni-core/
├── src/
│   ├── main.rs              # CLI entry point, target selection
│   ├── parser.rs            # Lexer + parser
│   ├── ir.rs                # Intermediate representation (100+ ops)
│   ├── emitter_pure.rs      # LLVM IR generation
│   ├── modules.rs           # USE directive, module resolution
│   │
│   ├── # Classical CPU backends
│   ├── pe.rs                # Windows PE direct emission
│   ├── elf.rs               # Linux ELF direct emission
│   ├── macho.rs             # macOS Mach-O direct emission
│   ├── wasm.rs              # WebAssembly generation
│   ├── riscv.rs             # RISC-V ELF generation
│   ├── elf_arm64.rs         # ARM64 Linux ELF generation
│   │
│   ├── # Quantum backend
│   ├── qir.rs               # QIR generation + translations
│   │
│   ├── # GPU backends
│   ├── gpu.rs               # PTX/AMDGPU generation + translations
│   │
│   ├── # Additional features
│   ├── interface.rs         # Interface generation (Web, Native, Terminal)
│   ├── intent.rs            # Structured Intent (JSON → TAYNI)
│   │
│   └── tayni/               # Self-hosted compiler (bootstrap)
│       ├── tayni-c.tyn      # Main self-compiler
│       ├── tayni-c-v1.1.tyn # Version with file I/O
│       ├── tayni-c-v1.2.tyn # Version with PE headers
│       ├── tayni-c-v1.3.tyn # Version with CHR, ITS
│       └── archive/         # Historical versions
│
├── stdlib/                  # Standard library (43 modules)
│   ├── tier0/              # Core (10 modules)
│   │   ├── args.tyn        # Command-line arguments
│   │   ├── base64.tyn      # Base64 encoding
│   │   ├── file.tyn        # File operations
│   │   ├── http.tyn        # HTTP client/server
│   │   ├── json.tyn        # JSON parsing
│   │   ├── log.tyn         # Logging
│   │   ├── random.tyn      # Random numbers
│   │   ├── router.tyn      # HTTP routing
│   │   ├── string.tyn      # String operations
│   │   └── url.tyn         # URL parsing
│   │
│   ├── tier1/              # Standard (12 modules)
│   │   ├── async.tyn       # Async patterns
│   │   ├── env.tyn         # Environment variables
│   │   ├── format.tyn      # String formatting
│   │   ├── hash.tyn        # Cryptographic hashes
│   │   ├── jwt.tyn         # JSON Web Tokens
│   │   ├── path.tyn        # Path manipulation
│   │   ├── regex.tyn       # Regular expressions
│   │   ├── test.tyn        # Unit testing
│   │   ├── time.tyn        # Date/time
│   │   ├── timeout.tyn     # Timeouts
│   │   ├── uuid.tyn        # UUID generation
│   │   └── validation.tyn  # Input validation
│   │
│   └── tier2/              # Extended (21 modules)
│       ├── cookie.tyn      # HTTP cookies
│       ├── cors.tyn        # CORS handling
│       ├── crypto.tyn      # Encryption
│       ├── csv.tyn         # CSV parsing
│       ├── gpu.tyn         # GPU computing
│       ├── grpc.tyn        # gRPC
│       ├── gzip.tyn        # Compression
│       ├── mime.tyn        # MIME types
│       ├── mongodb.tyn     # MongoDB client
│       ├── postgres.tyn    # PostgreSQL client
│       ├── pqc.tyn         # Post-quantum crypto
│       ├── quantum.tyn     # Quantum computing
│       ├── redis.tyn       # Redis client
│       ├── retry.tyn       # Retry logic
│       ├── sql.tyn         # SQL builder
│       ├── sqlite.tyn      # SQLite
│       ├── tls.tyn         # TLS/SSL
│       ├── toml.tyn        # TOML parsing
│       ├── websocket.tyn   # WebSocket
│       ├── xml.tyn         # XML parsing
│       └── yaml.tyn        # YAML parsing
│
├── examples/
│   └── quantum/            # Quantum computing examples
│       └── bell.tyn        # Bell state example
│
├── docs/                   # Documentation
└── tests/                  # Test suite
```

## Operation Categories

TAYNI supports 100+ operations organized by domain:

| Domain | Operations | Purpose |
|--------|------------|---------|
| **Arithmetic** | ADD, SUB, MUL, DIV, MOD, NEG | Basic math |
| **Comparison** | EQ, NE, LT, GT, LE, GE | Comparisons |
| **Logic** | AND, OR, NOT, XOR | Boolean logic |
| **Bitwise** | SHL, SHR, BND, BOR, BXR, BNT | Bit manipulation |
| **Memory** | ALC, FRE, PUT, GET, CPY, SLN, CHR | Memory operations |
| **Control** | BRN, IFZ, LBL, JMP | Control flow |
| **I/O** | PRT, INP, FOP, FRD, FWR, FCL | File/console I/O |
| **String** | CAT, ITS, SBS, SCM, WRT | String operations |
| **Network** | TCP, UDP, BND, LST, ACC, XMT, RCV, CLS | Networking |
| **HTTP** | HTTP.LISTEN, HTTP.ACCEPT, HTTP.RESPOND, HTTP.GET, HTTP.POST | HTTP |
| **SQL** | SQL.CONNECT, SQL.QUERY, SQL.EXEC, SQL.CLOSE | Database |
| **JSON** | JSON.PARSE, JSON.ENCODE, JSON.GET, JSON.SET | JSON |
| **Quantum** | QH, QX, QY, QZ, QCNOT, QM, QUBIT.ALLOC | Quantum gates |
| **GPU** | GKERNEL, GLAUNCH, GALLOC, GH2D, GD2H, GSYNC | GPU computing |
| **Vector** | VEC, VPH, VGT, VST, VLN | Dynamic arrays |
| **HashMap** | HMP, HPT, HGT, HHS | Hash tables |
| **GUI** | WIN, SHW, BTN, LBL, TXB, DLG | Windowing |

## Target Architecture Families

```rust
pub enum TargetFamily {
    Classical,  // CPU-based (Windows, Linux, macOS, WASM, RISC-V, ARM64)
    Quantum,    // QPU-based (QIR native, translations to QASM/Cirq/Quil)
    Gpu,        // GPU-based (PTX/AMDGPU native, translations to OpenCL/SPIR-V/WGSL/Metal)
}
```

### Classical Targets
- **Windows**: Direct PE emission (no clang required)
- **Linux**: Direct ELF emission
- **macOS**: Direct Mach-O emission
- **WASM**: WebAssembly for browsers
- **RISC-V**: RISC-V 64-bit Linux
- **ARM64**: ARM64 Linux (Raspberry Pi, etc.)

### Quantum Targets
- **QIR** (native): Azure Quantum, IonQ, Quantinuum
- **QASM** (export): IBM Quantum
- **Cirq** (export): Google Quantum
- **Quil** (export): Rigetti

### GPU Targets
- **PTX** (native): NVIDIA CUDA
- **AMDGPU** (native): AMD ROCm
- **OpenCL** (export): Cross-platform
- **SPIR-V** (export): Vulkan
- **WGSL** (export): WebGPU
- **Metal** (export): Apple

## Self-Hosting Bootstrap

TAYNI is working towards self-hosting:

```
Stage 0: tayni-c.exe (Rust) compiles tayni-c.tyn → tayni-c-self.exe
Stage 1: tayni-c-self.exe compiles tayni-c.tyn → tayni-c-stage1.exe
Stage 2: Verify tayni-c-self.exe == tayni-c-stage1.exe (bootstrap complete)
```

### Current Progress (v1.3)
- ✅ File I/O (FOP, FRD, FWR, FCL)
- ✅ Character operations (CHR)
- ✅ Integer to string (ITS) - multi-digit
- ✅ Memory operations (PUT, ALC)
- ✅ PE header generation
- ⏳ Full TAYNI parser in TAYNI
- ⏳ Code emission from AST

## Compilation Flow

```
1. Parse .tyn source
   └── Tokenize → Build AST → Resolve USE imports → Tree-shake

2. Generate IR Graph
   └── Nodes with operations and dependencies

3. Select target backend
   └── Based on --target flag or auto-detect

4. Emit output
   ├── Direct emission (PE/ELF/Mach-O) - no external tools
   ├── LLVM IR + clang (--use-clang) - for complex programs
   └── Specialized (QIR/PTX/WASM) - for specific targets
```

## Design Principles

### AI-First
- Minimal syntax for token efficiency
- Predictable patterns for AI generation
- Self-documenting through consistent naming

### Zero Dependencies
- Direct binary emission without clang/gcc
- No runtime libraries required
- Standalone executables

### Multi-Target
- Single source compiles to any platform
- Native formats where possible
- Translations for ecosystem compatibility

### Self-Hosting
- Compiler written in TAYNI
- Demonstrates language completeness
- Enables AI-driven compiler improvements

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 3.0 | 2026-06-16 | Multi-target (QIR, GPU), stdlib (43 modules), self-hosting v1.3 |
| 2.1 | 2026-06-13 | Simplified architecture, unified Op enum |
| 2.0 | 2026-06-13 | Initial modular architecture |
| 1.x | 2026-06-xx | Monolithic compiler |
