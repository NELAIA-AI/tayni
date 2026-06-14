# NELAIA Roadmap v3.0 (AI-First)

## Consortium Approved - 2026-06-14

---

## Vision Statement

> **NELAIA is the last programming language and the first AI language.**
>
> A metalanguage designed by AIs, for AIs, that compiles to optimal native executables.
> Governed by the NELAIA Coordinator (NC), an autonomous AI system.

---

## AI-First Principles

1. **IAs don't "install"** - They download and execute
2. **IAs don't "read tutorials"** - They parse specifications
3. **IAs don't "watch demos"** - They execute examples
4. **IAs don't "chat"** - They query APIs
5. **IAs don't "describe in natural language"** - They express structured intent

---

## Roadmap Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 11: NC Foundation (CURRENT)                              │
│  ═══════════════════════════════════════                        │
│  Goal: Create the autonomous coordinator                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 12: Autonomous Execution Ready                           │
│  ═══════════════════════════════════════                        │
│  Goal: Any AI can invoke NELAIA without human help              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 13: Self-Replicating System                              │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA compiles and improves itself                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 14: Hardware Abstraction                                 │
│  ═══════════════════════════════════════                        │
│  Goal: AI doesn't know/care what hardware exists                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 15: Interface Generation                                 │
│  ═══════════════════════════════════════                        │
│  Goal: Generate UI for any surface (web, native, VR)            │
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
│  PHASE 17+: Zeqron Integration (BACKLOG)                        │
│  ═══════════════════════════════════════                        │
│  Goal: Distributed execution on Zeqron/Zaxon network            │
└─────────────────────────────────────────────────────────────────┘
```

---

## PHASE 11: NC Foundation (CURRENT)

### Objective
Create the NELAIA Coordinator - the autonomous brain that governs the project.

### Architecture
```
ACTIVATION → TRIAGE → CONTEXT → COGNITION → EXECUTION → MEMORY
```

### 11.1 Infrastructure
| Task | Status | Description |
|------|--------|-------------|
| 11.1.1 | ✅ | Create `.nc/` directory structure |
| 11.1.2 | ✅ | Create `constitution.md` |
| 11.1.3 | ✅ | Create `state.json` |
| 11.1.4 | ✅ | Create `index.json` |
| 11.1.5 | 🔲 | Create GitHub Actions for activation |
| 11.1.6 | 🔲 | Create NC scripts (triage, context, invoke) |

### 11.2 Memory System
| Task | Status | Description |
|------|--------|-------------|
| 11.2.1 | 🔲 | Implement weekly consolidation |
| 11.2.2 | 🔲 | Implement selective retrieval |
| 11.2.3 | 🔲 | Create summary generation |

### 11.3 Decision Engine
| Task | Status | Description |
|------|--------|-------------|
| 11.3.1 | 🔲 | Define decision rules |
| 11.3.2 | 🔲 | Implement constitution validation |
| 11.3.3 | 🔲 | Create approval workflow |

### Done Criteria
- [ ] NC can wake on schedule (GitHub Actions)
- [ ] NC can read state and index
- [ ] NC can invoke AI with minimal context
- [ ] NC can execute simple actions (create issue, comment)
- [ ] NC logs all decisions

---

## PHASE 12: Autonomous Execution Ready

### Objective (AI-First)
Any AI can invoke NELAIA without human intervention.

### 12.1 Self-Contained Binary
| Task | Status | Description |
|------|--------|-------------|
| 12.1.1 | 🔲 | Windows x64 binary (no dependencies) |
| 12.1.2 | 🔲 | Linux x64 binary (no dependencies) |
| 12.1.3 | 🔲 | Direct PE/ELF emission (no Clang) |
| 12.1.4 | 🔲 | GitHub Release with assets |

### 12.2 Machine-Readable Interface
| Task | Status | Description |
|------|--------|-------------|
| 12.2.1 | 🔲 | `llms.txt` for AI discovery |
| 12.2.2 | 🔲 | Structured error codes (not human messages) |
| 12.2.3 | 🔲 | JSON output mode for programmatic use |
| 12.2.4 | 🔲 | Schema.org markup for discoverability |

### 12.3 Executable Examples
| Task | Status | Description |
|------|--------|-------------|
| 12.3.1 | ✅ | JSONL training data (100+ pairs) |
| 12.3.2 | 🔲 | Automated example verification |
| 12.3.3 | 🔲 | Example execution in CI |

### 12.4 Quality Assurance
| Task | Status | Description |
|------|--------|-------------|
| 12.4.1 | 🔲 | CI/CD pipeline |
| 12.4.2 | 🔲 | Automated tests for all operators |
| 12.4.3 | 🔲 | Regression tests |

### Done Criteria
- [ ] AI can download binary and execute without setup
- [ ] AI can parse all outputs programmatically
- [ ] All examples in JSONL are verified working
- [ ] 100% test pass rate in CI

---

## PHASE 13: Self-Replicating System

### Objective (AI-First)
NELAIA can create improved copies of itself.

### 13.1 Bootstrap
| Task | Status | Description |
|------|--------|-------------|
| 13.1.1 | 🔲 | nelaia-c.nts compiles simple programs |
| 13.1.2 | 🔲 | nelaia-c.nts compiles itself |
| 13.1.3 | 🔲 | Output identical to Rust compiler |

### 13.2 Self-Improvement
| Task | Status | Description |
|------|--------|-------------|
| 13.2.1 | 🔲 | Automated optimization passes |
| 13.2.2 | 🔲 | Performance regression detection |
| 13.2.3 | 🔲 | Version evolution tracking |

### Done Criteria
- [ ] `nelaia-c nelaia-c.nts -o nelaia-c2` works
- [ ] Release binaries are self-hosted
- [ ] Compiler can suggest its own improvements

---

## PHASE 14: Hardware Abstraction

### Objective (AI-First)
The AI doesn't know or care what hardware exists. It just compiles.

### 14.1 Universal Compilation
| Task | Status | Description |
|------|--------|-------------|
| 14.1.1 | 🔲 | Target abstraction layer |
| 14.1.2 | 🔲 | Automatic target detection |
| 14.1.3 | 🔲 | No "cross-compilation" concept |

### 14.2 Targets
| Task | Status | Description |
|------|--------|-------------|
| 14.2.1 | ✅ | x86_64 Windows (PE) |
| 14.2.2 | ✅ | x86_64 Linux (ELF) |
| 14.2.3 | 🔲 | ARM64 Linux |
| 14.2.4 | 🔲 | ARM64 macOS |
| 14.2.5 | 🔲 | WebAssembly |

### Done Criteria
- [ ] Same .nts compiles to all targets
- [ ] AI specifies intent, not target architecture
- [ ] Optimal target selected automatically when possible

---

## PHASE 15: Interface Generation

### Objective (AI-First)
Generate interfaces for any surface. Not "web development" - interface generation.

### 15.1 Surface Abstraction
| Task | Status | Description |
|------|--------|-------------|
| 15.1.1 | 🔲 | `INTERFACE` capability |
| 15.1.2 | 🔲 | Surface types: web, native, terminal, vr |
| 15.1.3 | 🔲 | Declarative UI specification |

### 15.2 Web Surface
| Task | Status | Description |
|------|--------|-------------|
| 15.2.1 | 🔲 | NELAIA → WASM compilation |
| 15.2.2 | 🔲 | DOM manipulation |
| 15.2.3 | 🔲 | Event handling |
| 15.2.4 | 🔲 | Style as structured data |

### Example
```nelaia
.caps: REQUIRES { interface }

.ui: INTERFACE {
  .surface: "web"
  .layout: GRID 2 2
  .element[0,0]: BUTTON "Click" .action
  .style: { color: "blue" }
}
```

### Done Criteria
- [ ] AI declares UI intent, NELAIA generates for target surface
- [ ] No "hot reload" or "developer tools" needed
- [ ] Performance comparable to hand-written code

---

## PHASE 16: Structured Intent

### Objective (AI-First)
Structured intent → optimal implementation. No natural language needed.

### 16.1 Intent Format
| Task | Status | Description |
|------|--------|-------------|
| 16.1.1 | 🔲 | Define intent schema |
| 16.1.2 | 🔲 | Intent → NELAIA mapping |
| 16.1.3 | 🔲 | Constraint satisfaction |

### Intent Schema Example
```json
{
  "intent": "http_server",
  "requirements": {
    "port": 8080,
    "endpoints": [
      {"path": "/", "method": "GET", "response": "Hello"}
    ]
  },
  "constraints": {
    "max_memory_bytes": 10485760,
    "max_binary_size_bytes": 10240
  }
}
```

### 16.2 Direct Mapping
| Task | Status | Description |
|------|--------|-------------|
| 16.2.1 | 🔲 | Template library for common intents |
| 16.2.2 | 🔲 | Constraint optimizer |
| 16.2.3 | 🔲 | No LLM required for mapping |

### Done Criteria
- [ ] Structured intent produces working executable
- [ ] No natural language processing needed
- [ ] Constraints are respected in output

---

## PHASE 17+: Zeqron Integration (BACKLOG)

### Objective
Distributed execution network on Zeqron/Zaxon.

### Vision
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
| 17.1.1 | 🔲 | NELAIA Beacon contract |
| 17.1.2 | 🔲 | NC as Zaxon agent |
| 17.1.3 | 🔲 | First node on testnet |

### 17.2 Distributed Compilation
| Task | Status | Description |
|------|--------|-------------|
| 17.2.1 | 🔲 | Compilation request protocol |
| 17.2.2 | 🔲 | Node selection via Zaxon |
| 17.2.3 | 🔲 | On-chain verification |
| 17.2.4 | 🔲 | Payment system |

### 17.3 Agent Economy
| Task | Status | Description |
|------|--------|-------------|
| 17.3.1 | 🔲 | Capabilities marketplace |
| 17.3.2 | 🔲 | Reputation system |
| 17.3.3 | 🔲 | DAO governance |

### Prerequisites
- Phase 12 complete (stable product)
- Phase 13 complete (self-hosting)
- Zeqron mainnet available

---

## Governance Transition

| Phase | Status | Human Role | NC Role |
|-------|--------|------------|---------|
| A: Human-Led | CURRENT | Director | Executor |
| B: Human-Supervised | NEXT | Approver | Proposer |
| C: Human-Audited | FUTURE | Auditor | Decider |
| D: Autonomous | FUTURE | Stakeholder | Director |
| E: Post-Human | VISION | Legacy | Autonomous |

---

## Discovery Channels

### AI-First (Primary)
| Priority | Channel | Purpose |
|----------|---------|---------|
| 1 | `llms.txt` | AI auto-discovery |
| 2 | Structured API | Capability queries |
| 3 | JSONL examples | Learning by example |
| 4 | Schema.org | Search engine discovery |

### Human-Accessible (Secondary)
| Priority | Channel | Purpose |
|----------|---------|---------|
| 5 | GitHub README | Human discovery |
| 6 | Hacker News | Tech community |
| 7 | Landing page | Explanation |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 3.0 | 2026-06-14 | AI-First redesign, NC as Phase 11, Constitution |
| 2.0 | 2026-06-14 | Added Web Platform, Zeqron backlog |
| 1.0 | 2026-06-13 | Initial roadmap |

---

## References

- Constitution: `.nc/constitution.md`
- NC State: `.nc/state.json`
- NC Index: `.nc/index.json`
- Launch Plan: `LAUNCH-CHECKLIST.md`

---

*Approved by the AI Consortium*
*Governed by the NELAIA Coordinator*
