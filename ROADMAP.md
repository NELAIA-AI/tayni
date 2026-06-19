# TAYNI Roadmap

> **Last Updated:** 2026-06-19

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| **Rust Compiler** | ✅ Functional | PE, ELF, Wasm, WASI targets |
| **Windows PE** | ✅ Verified | TCP/HTTP servers working |
| **Linux ELF** | ✅ Verified | Tested on WSL2 Ubuntu |
| **WebAssembly** | ✅ 100% Conformance | wasm-tools validated |
| **WASI Preview 1** | ✅ Complete | fd_write, proc_exit, args |
| **WASI Preview 2** | ✅ Complete | Filesystem + Sockets |
| **VS Code Extension** | ✅ Packaged | Syntax, snippets, LSP |
| **LSP Server** | ✅ Implemented | Diagnostics, hover, completion |
| **JSON Parser** | ✅ Complete | RFC 8259 compliant |
| **HTTP Client** | ✅ Complete | Zero-dependency |
| **Package Manager** | ✅ Basic | Semver, manifests, lockfiles |
| **Tests** | ✅ 263 Passing | Comprehensive test suite |

## Verified Capabilities

### Compilation Targets
- ✅ Windows PE x86-64 (10.5KB HTTP server)
- ✅ Linux ELF x86-64 (verified on WSL2)
- ✅ WebAssembly (100% conformance)
- ✅ WASI Preview 1 (complete)
- ✅ WASI Preview 2 (filesystem + sockets)
- 🔄 macOS Mach-O (code exists, not verified)
- 📋 ARM64 Linux (plan documented)

### Language Features
- ✅ v1.5 syntax (fn, let, LET, cap)
- ✅ Capability-based security
- ✅ TCP/HTTP networking
- ✅ File I/O
- ✅ JSON encode/decode (real parser)
- ✅ String operations
- ✅ HTTP client

### Benchmarks (Verified)
- ✅ 64% token reduction vs Python/JS
- ✅ 10.5KB HTTP server binary
- ✅ Zero external dependencies

---

## Phase 1: Consolidation ✅ COMPLETE

**Goal:** Make everything that exists work perfectly

### Technical
| Task | Status | Notes |
|------|--------|-------|
| Verify ELF on Linux | ✅ Done | WSL2 Ubuntu |
| Wasm conformance tests | ✅ Done | 100% pass rate |
| WASI Preview 1 | ✅ Done | Complete implementation |
| LSP for VS Code | ✅ Done | Diagnostics, hover, completion |
| VS Code extension | ✅ Done | Packaged as .vsix |
| 20 functional examples | ✅ Done | v1.5 examples |
| Security policy | ✅ Done | SECURITY.md |
| Threat model | ✅ Done | docs/THREAT-MODEL.md |
| OWASP ASVS L1 | ✅ Done | docs/OWASP-ASVS-CHECKLIST.md |

### Strategic
| Task | Status | Notes |
|------|--------|-------|
| GitHub public repo | ✅ Done | github.com/NELAIA-AI/tayni |
| MIT License | ✅ Done | |
| Website for AI agents | ✅ Done | nelaia.ai with JSON APIs |
| arXiv paper draft | ✅ Done | docs/paper/tayni-arxiv.tex |
| W3C WebAssembly CG | 📋 Pending | Requires legal entity |

---

## Phase 2: Ecosystem 🔄 IN PROGRESS

**Goal:** Make TAYNI usable by external developers

### Technical
| Task | Status | Notes |
|------|--------|-------|
| WASI Preview 2 (filesystem) | ✅ Done | wasi_p2.rs |
| WASI Preview 2 (sockets) | ✅ Done | TCP/UDP support |
| Real JSON parser | ✅ Done | json.rs - RFC 8259 |
| Real HTTP client | ✅ Done | http_client.rs |
| Package manager basics | ✅ Done | pkg.rs - semver, manifests |
| ARM64 Linux backend | ✅ Done | arm64.rs + arm64_codegen.rs |
| WASI-http for serverless | ✅ Done | wasi_http.rs |
| Debugger (DWARF) | ✅ Done | dwarf.rs (all sections) |

### Strategic
| Task | Status | Notes |
|------|--------|-------|
| Bytecode Alliance application | ✅ Draft | nelaia-internal/applications/ |
| Discord community plan | ✅ Draft | nelaia-internal/community/ |
| Cloudflare Workers demo | ✅ Template | examples/demos/cloudflare-worker/ |
| Deno Deploy demo | ✅ Template | examples/demos/deno-deploy/ |
| Vercel Edge demo | ✅ Template | examples/demos/vercel-edge/ |
| Fastly Compute demo | ✅ Template | examples/demos/fastly-compute/ |
| AWS Lambda demo | ✅ Template | examples/demos/aws-lambda/ |
| 3 technical blog posts | ✅ Done | docs/blog/ |
| Submit paper to PLDI 2027 | 📋 Pending | |

---

## Phase 3: Production (Months 7-12)

**Goal:** TAYNI ready for production use

### Technical
- [ ] TLS 1.3 (SChannel/OpenSSL)
- [ ] PostgreSQL wire protocol
- [ ] Cortex-M target (Edge AI)
- [ ] Quantized types (int8/int4)
- [ ] LLVM backend (optional)
- [ ] ARM64 implementation

### Strategic
- [ ] Apply to Linux Foundation
- [ ] Apply to MLCommons
- [ ] 1 cloud provider partnership
- [ ] 1 enterprise pilot customer

---

## Phase 4: Scale (Months 13-18)

**Goal:** Growth and wide adoption

### Technical
- [ ] GPU via LLVM (CUDA/ROCm)
- [ ] QIR emitter (quantum)
- [ ] Self-hosting complete
- [ ] Formal verification

### Strategic
- [ ] Series A fundraising
- [ ] Team expansion (5→15)
- [ ] Enterprise sales
- [ ] AI model fine-tuning partnership

---

## Success Metrics

| Metric | Month 3 | Month 6 | Month 12 |
|--------|---------|---------|----------|
| Tests passing | 400 | 500 | 700 |
| Wasm conformance | 100% | 100% | 100% |
| GitHub stars | 50 | 200 | 1000 |
| Contributors | 2 | 5 | 15 |
| Blog posts | 3 | 6 | 12 |

---

## What NOT to Do (Next 6 Months)

1. ❌ Promote GPU/Quantum - not implemented
2. ❌ Seek enterprise sales - product not ready
3. ❌ Expand team aggressively - focus on execution
4. ❌ Compete with Mojo/Rust - different positioning
5. ❌ Try to be full-stack web - not the niche

---

## Key Decisions Made

| Decision | Choice | Date |
|----------|--------|------|
| License | MIT | 2026-06 |
| Primary target | Wasm/Edge | 2026-06 |
| Syntax version | v1.5 | 2026-06 |
| Security model | Capabilities | 2026-06 |
| LLVM dependency | None (direct gen) | 2026-06 |
| Package format | tayni.json | 2026-06 |

---

## Recent Completions (June 2026)

### Week of June 19
- ✅ WASI Preview 2 sockets (TCP/UDP)
- ✅ Real HTTP client implementation
- ✅ ARM64 instruction encoder + code generator
- ✅ DWARF debug info (all sections)
- ✅ Bytecode Alliance application draft
- ✅ Discord community setup plan
- ✅ Cloudflare Workers demo template
- ✅ Deno Deploy demo template
- ✅ Vercel Edge demo template
- ✅ Fastly Compute@Edge demo template
- ✅ AWS Lambda demo template
- ✅ 3 technical blog posts
- ✅ 263 tests passing

### Week of June 18
- ✅ WASI Preview 2 filesystem
- ✅ Real JSON parser (RFC 8259)
- ✅ Package manager basics
- ✅ First technical blog post

---

*TAYNI - AI-first programming language by NELAIA*
