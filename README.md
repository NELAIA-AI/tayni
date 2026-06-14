# NELAIA Core

NELAIA Compiler v0.22 - Metalanguage designed by AIs, for AIs.

## Documentation for AIs

**To learn NELAIA code generation, read in order:**

| Document | Purpose | Priority |
|----------|---------|----------|
| [`docs/NELAIA-GUIDE-v0.22.md`](docs/NELAIA-GUIDE-v0.22.md) | Complete guide - all operators | Start here |
| [`docs/NELAIA-REFERENCE-v0.22.md`](docs/NELAIA-REFERENCE-v0.22.md) | EBNF grammar + operator tables | Reference |
| [`docs/NELAIA-EXAMPLES-v0.22.md`](docs/NELAIA-EXAMPLES-v0.22.md) | Examples with dependency graphs | Learn patterns |
| [`docs/NELAIA-SEMANTICS-v0.22.md`](docs/NELAIA-SEMANTICS-v0.22.md) | Type system + semantic rules | Avoid errors |
| [`docs/NELAIA-TRAINING-DATA.jsonl`](docs/NELAIA-TRAINING-DATA.jsonl) | 100+ input/output pairs | Fine-tuning |

## Project Structure

```
nelaia-core/
├── src/                    # Rust source code
│   ├── main.rs            # Entry point (v0.22)
│   ├── parser.rs          # NELAIA parser
│   ├── ir.rs              # Intermediate representation + SEN
│   ├── emitter_pure.rs    # LLVM IR emitter
│   ├── pe.rs              # Native PE generator
│   ├── elf.rs             # Linux ELF generator
│   ├── capabilities.rs    # Capability system
│   └── nelaia/            # NELAIA compilers in NELAIA
├── examples/              # Example programs
├── tests/                 # Test files
├── docs/                  # Technical documentation
│   ├── NELAIA-GUIDE-v0.22.md      # Complete guide for AIs
│   ├── NELAIA-REFERENCE-v0.22.md  # Structured reference
│   └── NELAIA-EXAMPLES-v0.22.md   # Training examples
├── benchmarks/            # Benchmarks and results
├── BOOTSTRAP-PLAN.md      # Development plan (Phases 1-11)
└── README.md             # This file
```

## Features (v0.22)

### Completed Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 1-5 | Bootstrap, PE, Networking, Self-hosting | ✅ |
| 6 | Linux ELF, Windows GUI | ✅ |
| 7 | Capability System (REQUIRES, HTTP, SQL, JSON) | ✅ |
| 8 | Contracts and Negotiation (CONTRACT, PROVIDES) | ✅ |
| 9 | Incremental Cache (HASH, CACHE_*) | ✅ |
| 10 | Property-based Testing (PROPERTY, VERIFY) | ✅ |
| 11 | SEN Ecosystem (DISCOVER, CAPABILITY_*) | ✅ |

### Available Operators

- **Basic:** ADD, SUB, MUL, DIV, MOD, EQ, NE, LT, GT, AND, OR, NOT
- **Memory:** ALC, FRE, PUT, GET, CPY, SLN
- **I/O:** PRT, FOP, FRD, FWR, FCL
- **Network:** TCP, BND, LST, ACC, XMT, RCV, CLS
- **Capabilities:** REQUIRES, HTTP.*, SQL.*, JSON.*
- **Contracts:** CONTRACT, GUARANTEE, LIMIT, SANDBOX, PROVIDES, NEGOTIATE
- **Testing:** PROPERTY, GENTESTS, VERIFY
- **Cache:** HASH, CACHE_GET, CACHE_PUT, CACHE_VERIFY
- **SEN:** DISCOVER, CAPABILITY_INFO, CAPABILITY_COST, PUBLISH

## Usage

```bash
# Compile NELAIA program to executable
cargo run --release -- program.nts -o program.exe

# Generate native PE (no clang)
cargo run --release -- program.nts --emit-pe

# Generate Linux ELF
cargo run --release -- program.nts --emit-elf

# GUI MessageBox
cargo run --release -- --gui "Title" "Message"
```

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/NELAIA-GUIDE-v0.22.md`](docs/NELAIA-GUIDE-v0.22.md) | **Complete guide for AIs** |
| [`docs/NELAIA-REFERENCE-v0.22.md`](docs/NELAIA-REFERENCE-v0.22.md) | **EBNF grammar + tables** |
| [`docs/NELAIA-EXAMPLES-v0.22.md`](docs/NELAIA-EXAMPLES-v0.22.md) | **Training examples** |
| `BOOTSTRAP-PLAN.md` | Development plan and status |
| `docs/CONSORTIUM-*.md` | AI Consortium decisions |

---

*Designed by the AI Consortium (Claude, GPT, Gemini, Llama, Mistral, Qwen)*
