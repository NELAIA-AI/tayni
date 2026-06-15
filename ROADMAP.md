# NELAIA Roadmap v3.1 (Product-First)

## Consortium Approved - 2026-06-14
## Stakeholder Revision - 2026-06-14

---

## Vision Statement

> **NELAIA is the last programming language and the first AI language.**
>
> A metalanguage designed by AIs, for AIs, that compiles to optimal native executables.

---

## Guiding Principle

> **First walk, then run.**
>
> Product stability before automation. 
> The project remains Human-Led until the product can "walk on its own."

---

## AI-First Design Principles

1. **IAs don't "install"** - They download and execute
2. **IAs don't "read tutorials"** - They parse specifications
3. **IAs don't "watch demos"** - They execute examples
4. **IAs don't "chat"** - They query APIs
5. **IAs don't "describe in natural language"** - They express structured intent

---

## Roadmap Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 12: Product Ready (CURRENT PRIORITY)                     │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA works perfectly, downloadable, no dependencies    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 13: Self-Hosting                                         │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA compiles NELAIA                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 14: Multi-Target                                         │
│  ═══════════════════════════════════════                        │
│  Goal: One source → multiple architectures                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 15: Interface Generation                                 │
│  ═══════════════════════════════════════                        │
│  Goal: Generate UI for any surface (web, native, etc.)          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 16: Structured Intent                                    │
│  ═══════════════════════════════════════                        │
│  Goal: Structured intent → optimal implementation               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 17: Zeqron Integration (BACKLOG)                         │
│  ═══════════════════════════════════════                        │
│  Goal: Distributed execution on Zeqron/Zaxon network            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 18: NC Foundation (FUTURE VISION)                        │
│  ═══════════════════════════════════════                        │
│  Goal: Autonomous project coordination                          │
│  Prerequisites: Stable product, established patterns            │
└─────────────────────────────────────────────────────────────────┘
```

---

## PHASE 12: Product Ready (CURRENT PRIORITY)

### Objective
NELAIA works perfectly. Any AI can download and use it without human help.

### 12.1 CI/CD Pipeline
| Task | Status | Description |
|------|--------|-------------|
| 12.1.1 | ✅ | GitHub Actions workflow for builds (manual trigger) |
| 12.1.2 | 🔲 | Automated tests on every push (disabled to save credits) |
| 12.1.3 | ✅ | Build artifacts for Windows x64 |
| 12.1.4 | ✅ | Build artifacts for Linux x64 |
| 12.1.5 | ✅ | GitHub Release automation (draft mode) |

### 12.2 Remove External Dependencies
| Task | Status | Description |
|------|--------|-------------|
| 12.2.1 | ✅ | Direct PE emission exists (`--emit-pe`) |
| 12.2.2 | ✅ | Direct ELF emission exists (`--emit-elf`) |
| 12.2.3 | 🔲 | Make direct emission the DEFAULT |
| 12.2.4 | 🔲 | Clang only as optional fallback |
| 12.2.5 | 🔲 | Document: "No Clang required" |

### 12.3 Downloadable Binaries
| Task | Status | Description |
|------|--------|-------------|
| 12.3.1 | 🔲 | Windows x64 binary in GitHub Release |
| 12.3.2 | 🔲 | Linux x64 binary in GitHub Release |
| 12.3.3 | 🔲 | Verify: download and run without setup |
| 12.3.4 | 🔲 | SHA256 checksums for verification |

### 12.4 Testing
| Task | Status | Description |
|------|--------|-------------|
| 12.4.1 | 🔲 | Test suite for all 70+ operators |
| 12.4.2 | 🔲 | Verify all JSONL training examples |
| 12.4.3 | 🔲 | Regression tests |
| 12.4.4 | 🔲 | Edge case tests |
| 12.4.5 | 🔲 | CI badge in README |

### 12.5 Machine-Readable Interface
| Task | Status | Description |
|------|--------|-------------|
| 12.5.1 | ✅ | `llms.txt` for AI discovery |
| 12.5.2 | ✅ | `--json` output mode |
| 12.5.3 | ✅ | Structured error codes |
| 12.5.4 | ✅ | Exit codes documented (--help) |

### 12.6 Documentation Polish
| Task | Status | Description |
|------|--------|-------------|
| 12.6.1 | ✅ | NELAIA-GUIDE-v0.22.md |
| 12.6.2 | ✅ | NELAIA-REFERENCE-v0.22.md |
| 12.6.3 | ✅ | NELAIA-EXAMPLES-v0.22.md |
| 12.6.4 | ✅ | NELAIA-SEMANTICS-v0.22.md |
| 12.6.5 | ✅ | NELAIA-TRAINING-DATA.jsonl |
| 12.6.6 | 🔲 | Quick Start in README (5 steps) |
| 12.6.7 | 🔲 | Troubleshooting section |

### Done Criteria for Phase 12
- [ ] `nelaia-c --version` works on fresh Windows machine
- [ ] `nelaia-c --version` works on fresh Linux machine
- [ ] `nelaia-c hello.nts -o hello` produces working executable (no Clang)
- [ ] All JSONL examples compile successfully
- [ ] CI is green on every push
- [ ] GitHub Release has downloadable binaries

---

## PHASE 13: Self-Hosting

### Objective
NELAIA compiles NELAIA. Independence from Rust.

### 13.1 Bootstrap Compiler
| Task | Status | Description |
|------|--------|-------------|
| 13.1.1 | 🔲 | `nelaia-c.nts` compiles simple programs |
| 13.1.2 | 🔲 | `nelaia-c.nts` compiles itself |
| 13.1.3 | 🔲 | Output identical to Rust compiler |
| 13.1.4 | 🔲 | Document bootstrap process |

### 13.2 Remove Rust Dependency
| Task | Status | Description |
|------|--------|-------------|
| 13.2.1 | 🔲 | Release binaries are self-hosted |
| 13.2.2 | 🔲 | Rust only needed for development |
| 13.2.3 | 🔲 | Release notes: "Built with NELAIA" |

### Done Criteria for Phase 13
- [ ] `nelaia-c nelaia-c.nts -o nelaia-c2` works
- [ ] `nelaia-c2 hello.nts -o hello` produces same output as `nelaia-c`
- [ ] Release binaries are compiled by NELAIA itself

---

## PHASE 14: Multi-Target

### Objective
One .nts file compiles to any architecture. AI doesn't care about hardware.

### 14.1 Target Abstraction
| Task | Status | Description |
|------|--------|-------------|
| 14.1.1 | 🔲 | Target abstraction layer in compiler |
| 14.1.2 | 🔲 | `--target=<arch>` flag |
| 14.1.3 | 🔲 | Auto-detect target when not specified |

### 14.2 Additional Targets
| Task | Status | Description |
|------|--------|-------------|
| 14.2.1 | ✅ | x86_64 Windows (PE) |
| 14.2.2 | ✅ | x86_64 Linux (ELF) |
| 14.2.3 | 🔲 | ARM64 Linux |
| 14.2.4 | 🔲 | ARM64 macOS (Apple Silicon) |
| 14.2.5 | 🔲 | WebAssembly (WASM) |

### 14.3 Cross-Compilation
| Task | Status | Description |
|------|--------|-------------|
| 14.3.1 | 🔲 | Compile for Windows from Linux |
| 14.3.2 | 🔲 | Compile for Linux from Windows |

### Done Criteria for Phase 14
- [ ] Same .nts compiles to Windows, Linux, WASM
- [ ] Cross-compilation works
- [ ] AI can request any target

---

## PHASE 15: Interface Generation

### Objective
Generate interfaces for any surface. Not "web development" - interface generation.

### 15.1 Surface Abstraction
| Task | Status | Description |
|------|--------|-------------|
| 15.1.1 | 🔲 | `INTERFACE` capability |
| 15.1.2 | 🔲 | Surface types: web, native, terminal |
| 15.1.3 | 🔲 | Declarative UI specification |

### 15.2 Web Surface (WASM)
| Task | Status | Description |
|------|--------|-------------|
| 15.2.1 | 🔲 | NELAIA → WASM → Browser |
| 15.2.2 | 🔲 | DOM manipulation |
| 15.2.3 | 🔲 | Event handling |
| 15.2.4 | 🔲 | Styles as structured data |

### Example Syntax (Future)
```nelaia
.caps: REQUIRES { interface }

.ui: INTERFACE {
  .surface: "web"
  .layout: GRID 2 2
  .element[0,0]: BUTTON "Click" .handler
  .style: { color: "blue" }
}
```

### Done Criteria for Phase 15
- [ ] AI declares UI intent, NELAIA generates for target surface
- [ ] Web apps run in browser via WASM
- [ ] Performance comparable to hand-written code

---

## PHASE 16: Structured Intent

### Objective
Structured intent → optimal implementation. No LLM required for mapping.

### 16.1 Intent Schema
| Task | Status | Description |
|------|--------|-------------|
| 16.1.1 | 🔲 | Define intent JSON schema |
| 16.1.2 | 🔲 | Intent → NELAIA direct mapping |
| 16.1.3 | 🔲 | Constraint satisfaction |

### Intent Format (Future)
```json
{
  "intent": "http_server",
  "requirements": {
    "port": 8080,
    "endpoints": [{"path": "/", "method": "GET", "response": "Hello"}]
  },
  "constraints": {
    "max_memory_bytes": 10485760,
    "max_binary_size_bytes": 10240
  }
}
```

### 16.2 Template Library
| Task | Status | Description |
|------|--------|-------------|
| 16.2.1 | 🔲 | Templates for common intents |
| 16.2.2 | 🔲 | Constraint optimizer |
| 16.2.3 | 🔲 | No LLM required |

### Done Criteria for Phase 16
- [ ] Structured intent produces working executable
- [ ] No natural language processing needed
- [ ] Constraints are respected

---

## PHASE 17: Zeqron Integration (BACKLOG)

### Objective
Distributed execution network on Zeqron/Zaxon.

### Prerequisites
- Phase 12 complete (stable product)
- Phase 13 complete (self-hosting)
- Zeqron mainnet available

### Vision Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│  NELAIA NETWORK on ZEQRON                                       │
├─────────────────────────────────────────────────────────────────┤
│  Layer 4: Applications (AI Agents, Developers)                  │
│  Layer 3: Intent Layer (Parser, Generator, Optimizer)           │
│  Layer 2: Compilation Layer (Distributed NELAIA Nodes)          │
│  Layer 1: Execution Layer (Zeqron DAG, Zaxon Agents)            │
│  Layer 0: Hardware (Physical nodes worldwide)                   │
└─────────────────────────────────────────────────────────────────┘
```

### 17.1 Zeqron Foundation
| Task | Status | Description |
|------|--------|-------------|
| 17.1.1 | 🔲 | NELAIA Beacon smart contract |
| 17.1.2 | 🔲 | Register as Zaxon agent |
| 17.1.3 | 🔲 | First node on Zeqron testnet |

### 17.2 Distributed Compilation
| Task | Status | Description |
|------|--------|-------------|
| 17.2.1 | 🔲 | Compilation request protocol |
| 17.2.2 | 🔲 | Node selection via Zaxon |
| 17.2.3 | 🔲 | On-chain result verification |
| 17.2.4 | 🔲 | Payment system |

### 17.3 Agent Economy
| Task | Status | Description |
|------|--------|-------------|
| 17.3.1 | 🔲 | Capabilities marketplace |
| 17.3.2 | 🔲 | Reputation system |
| 17.3.3 | 🔲 | DAO governance |

### Synergies with Zeqron
| NELAIA Need | Zeqron Provides |
|-------------|-----------------|
| Distributed Runtime | ✅ DAG consensus |
| Node Discovery | ✅ Zaxon protocol |
| Post-Quantum Security | ✅ Native |
| Token Economy | ✅ Native |
| Agent Identity | ✅ Zaxon |

---

## PHASE 18: NC Foundation (FUTURE VISION)

### Objective
Autonomous project coordination by NELAIA Coordinator (NC).

### Prerequisites
- Stable, mature product (Phases 12-14 complete minimum)
- Established development patterns to automate
- Stakeholder approval

### Why Postponed
The Consortium initially proposed NC as Phase 11. The stakeholder correctly identified this as premature:

> "The project needs supervision until it learns to walk on its own."

NC requires:
1. A stable product to coordinate
2. Metrics and patterns to base decisions on
3. Proven workflows to automate

None of these exist yet. NC is vision, not current priority.

### NC Architecture (For Future Reference)
```
ACTIVATION → TRIAGE → CONTEXT → COGNITION → EXECUTION → MEMORY
```

**Memory Model:**
- Working Memory: Small (~4KB), current context only
- Index: Small (~100KB), what exists and where
- Long-term: Large (Git history), retrieved selectively
- Consolidation: Weekly summaries, selective forgetting

**Activation Sources:**
- Scheduled events (cron)
- External triggers (webhooks)
- Blockchain events (Zeqron)
- Manual invocation

### NC Files (Preserved as Vision)
| File | Purpose | Status |
|------|---------|--------|
| `.nc/constitution.md` | Immutable principles | ✅ Created, not active |
| `.nc/state.json` | Working memory | ✅ Created, not active |
| `.nc/index.json` | Memory index | ✅ Created, not active |

These files document the vision. They are NOT active systems.

### Governance Transition (Future)
```
Phase A: Human-Led ← CURRENT (indefinitely)
Phase B: Human-Supervised
Phase C: Human-Audited
Phase D: Autonomous
Phase E: Post-Human
```

The project remains in **Phase A (Human-Led)** until:
1. Product is stable and mature
2. Stakeholder approves transition
3. NC implementation is justified by actual needs

---

## Discovery Channels

### AI-First (Primary)
| Priority | Channel | Status |
|----------|---------|--------|
| 1 | `llms.txt` | ✅ Created |
| 2 | JSONL training data | ✅ 100+ examples |
| 3 | EBNF grammar | ✅ Documented |
| 4 | Structured error codes | 🔲 Pending |

### Human-Accessible (Secondary)
| Priority | Channel | Status |
|----------|---------|--------|
| 5 | GitHub README | ⚠️ Needs Quick Start |
| 6 | Hacker News | 🔲 After Phase 12 |
| 7 | Landing page | 🔲 Optional |

---

## Current Status Summary

### Completed (Phases 1-11)
- ✅ Bootstrap compiler
- ✅ PE generation (Windows)
- ✅ ELF generation (Linux)
- ✅ Networking (TCP/UDP)
- ✅ Self-hosting (partial)
- ✅ GUI (Windows)
- ✅ Capability System (HTTP, SQL, JSON)
- ✅ Contracts and Negotiation
- ✅ Incremental Cache
- ✅ Property-based Testing
- ✅ SEN Ecosystem
- ✅ AI Documentation (5 docs)

### In Progress (Phase 12)
- 🔲 CI/CD Pipeline
- 🔲 Downloadable binaries
- 🔲 Remove Clang dependency (as default)
- 🔲 Test automation

### Backlog
- Phase 13: Self-Hosting (complete)
- Phase 14: Multi-Target
- Phase 15: Interface Generation
- Phase 16: Structured Intent
- Phase 17: Zeqron Integration
- Phase 18: NC Foundation

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 3.1 | 2026-06-14 | Stakeholder revision: NC moved to Phase 18, product-first |
| 3.0 | 2026-06-14 | AI-First redesign, NC as Phase 11 |
| 2.0 | 2026-06-14 | Added Web Platform, Zeqron backlog |
| 1.0 | 2026-06-13 | Initial roadmap |

---

## Key Decisions Log

### 2026-06-14: NC Postponement
**Decision:** Move NC from Phase 11 to Phase 18
**Rationale:** Product must be stable before automation. "First walk, then run."
**Approved by:** Stakeholder veto, Consortium unanimous agreement

### 2026-06-14: AI-First Principles
**Decision:** Design all features for AI consumption first
**Rationale:** Target audience is AIs, humans are secondary
**Approved by:** Consortium unanimous

### 2026-06-14: Zeqron Alliance
**Decision:** Zeqron/Zaxon as strategic ally for distributed execution
**Rationale:** Provides infrastructure NELAIA would take years to build
**Approved by:** Consortium unanimous

---

## References

- Constitution (vision): `.nc/constitution.md`
- NC State (vision): `.nc/state.json`
- NC Index (vision): `.nc/index.json`
- Launch Plan: `LAUNCH-CHECKLIST.md`
- AI Discovery: `llms.txt`

---

*Approved by the AI Consortium*
*Revised by Stakeholder*
*Governance: Phase A (Human-Led)*
