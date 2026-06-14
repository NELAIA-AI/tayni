# NELAIA Roadmap v2.0

## Consortium Approved - 2026-06-14

---

## Vision Statement

> NELAIA is the last programming language and the first AI language.
> A metalanguage designed by AIs, for AIs, that compiles to optimal native executables.

---

## Roadmap Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 12: Functional Product (CURRENT)                         │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA works perfectly standalone                        │
│  Timeline: Weeks 1-4                                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 13: Self-Hosting                                         │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA compiles NELAIA                                   │
│  Timeline: Weeks 5-10                                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 14: Multi-Target                                         │
│  ═══════════════════════════════════════                        │
│  Goal: One source → multiple architectures                      │
│  Timeline: Weeks 11-16                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 15: Web Platform                                         │
│  ═══════════════════════════════════════                        │
│  Goal: NELAIA for web development                               │
│  Timeline: Weeks 17-24                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 16: Intent-to-Code                                       │
│  ═══════════════════════════════════════                        │
│  Goal: Natural language → Executable                            │
│  Timeline: Weeks 25-32                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 17+: Zeqron Integration (FUTURE)                         │
│  ═══════════════════════════════════════                        │
│  Goal: Distributed execution on Zeqron/Zaxon                    │
│  Timeline: After Phase 16                                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## PHASE 12: Functional Product (CURRENT)

### Objective
NELAIA works perfectly without complex dependencies. A developer can use it in < 5 minutes.

### 12.1 Release Engineering
| Task | Status | Description |
|------|--------|-------------|
| 12.1.1 | 🔲 | GitHub Actions CI/CD pipeline |
| 12.1.2 | 🔲 | Automated build on push |
| 12.1.3 | 🔲 | Automated tests |
| 12.1.4 | 🔲 | Windows x64 binary |
| 12.1.5 | 🔲 | Linux x64 binary |
| 12.1.6 | 🔲 | macOS binary (if possible) |
| 12.1.7 | 🔲 | GitHub Release with downloadable assets |

### 12.2 Simplified Installation
| Task | Status | Description |
|------|--------|-------------|
| 12.2.1 | 🔲 | Windows installer (.exe) |
| 12.2.2 | 🔲 | Linux install script (curl \| sh) |
| 12.2.3 | 🔲 | Auto-add to PATH |
| 12.2.4 | 🔲 | Verify: `nelaia-c --version` works |

### 12.3 Remove Clang Dependency
| Task | Status | Description |
|------|--------|-------------|
| 12.3.1 | 🔲 | Direct PE emission for Windows (default) |
| 12.3.2 | 🔲 | Direct ELF emission for Linux (default) |
| 12.3.3 | 🔲 | Clang as optional fallback only |

### 12.4 Complete Testing
| Task | Status | Description |
|------|--------|-------------|
| 12.4.1 | 🔲 | Test suite for all operators |
| 12.4.2 | 🔲 | Regression tests |
| 12.4.3 | 🔲 | Edge case tests |
| 12.4.4 | 🔲 | CI runs tests on every PR |
| 12.4.5 | 🔲 | "Tests passing" badge in README |

### 12.5 User Documentation
| Task | Status | Description |
|------|--------|-------------|
| 12.5.1 | 🔲 | Quick Start: 5 steps max |
| 12.5.2 | 🔲 | Common troubleshooting |
| 12.5.3 | 🔲 | FAQ |
| 12.5.4 | 🔲 | Copy-paste working examples |

### Done Criteria
- [ ] `curl -sSL nelaia.ai/install.sh | sh` works on Linux
- [ ] Download .exe and it works on Windows without installing anything else
- [ ] `nelaia-c hello.nts -o hello` produces working executable
- [ ] 100% tests pass in CI
- [ ] README has 5-step Quick Start

---

## PHASE 13: Self-Hosting

### Objective
NELAIA compiles NELAIA. Independence from Rust.

### 13.1 Bootstrap Compiler
| Task | Status | Description |
|------|--------|-------------|
| 13.1.1 | 🔲 | nelaia-c.nts compiles simple programs |
| 13.1.2 | 🔲 | nelaia-c.nts compiles itself |
| 13.1.3 | 🔲 | Verify: output identical to Rust compiler |
| 13.1.4 | 🔲 | Document bootstrap process |

### 13.2 Remove Rust Dependency
| Task | Status | Description |
|------|--------|-------------|
| 13.2.1 | 🔲 | Distributed compiler is self-hosted |
| 13.2.2 | 🔲 | Rust only needed for development |
| 13.2.3 | 🔲 | Release notes: "Built with NELAIA" |

### Done Criteria
- [ ] `nelaia-c nelaia-c.nts -o nelaia-c2` works
- [ ] `nelaia-c2 hello.nts -o hello` produces same output
- [ ] Release binaries are self-hosted

---

## PHASE 14: Multi-Target

### Objective
One .nts file compiles to multiple architectures.

### 14.1 Additional Targets
| Task | Status | Description |
|------|--------|-------------|
| 14.1.1 | 🔲 | ARM64 Linux |
| 14.1.2 | 🔲 | ARM64 macOS (M1/M2) |
| 14.1.3 | 🔲 | WebAssembly (WASM) |
| 14.1.4 | 🔲 | Target abstraction layer |

### 14.2 Cross-Compilation
| Task | Status | Description |
|------|--------|-------------|
| 14.2.1 | 🔲 | Compile for Windows from Linux |
| 14.2.2 | 🔲 | Compile for Linux from Windows |
| 14.2.3 | 🔲 | `--target=<arch>` flag |

### Done Criteria
- [ ] Same .nts compiles to Windows, Linux, WASM
- [ ] Cross-compilation works

---

## PHASE 15: Web Platform

### Objective
NELAIA as a modern alternative for web development (competing with React, Next.js, Vite).

### 15.1 WebAssembly Foundation
| Task | Status | Description |
|------|--------|-------------|
| 15.1.1 | 🔲 | NELAIA → WASM compilation |
| 15.1.2 | 🔲 | WASM runtime integration |
| 15.1.3 | 🔲 | DOM manipulation capabilities |
| 15.1.4 | 🔲 | Event handling |

### 15.2 Web Capabilities
| Task | Status | Description |
|------|--------|-------------|
| 15.2.1 | 🔲 | `WEB.ELEMENT` - Create DOM elements |
| 15.2.2 | 🔲 | `WEB.STYLE` - Apply CSS |
| 15.2.3 | 🔲 | `WEB.EVENT` - Handle events |
| 15.2.4 | 🔲 | `WEB.FETCH` - HTTP requests |
| 15.2.5 | 🔲 | `WEB.ROUTE` - Client-side routing |

### 15.3 Component System
| Task | Status | Description |
|------|--------|-------------|
| 15.3.1 | 🔲 | Declarative component syntax |
| 15.3.2 | 🔲 | State management |
| 15.3.3 | 🔲 | Reactive updates |
| 15.3.4 | 🔲 | Component composition |

### 15.4 Build Tools
| Task | Status | Description |
|------|--------|-------------|
| 15.4.1 | 🔲 | Dev server with hot reload |
| 15.4.2 | 🔲 | Production build optimization |
| 15.4.3 | 🔲 | Asset bundling |
| 15.4.4 | 🔲 | SSR (Server-Side Rendering) |

### 15.5 Web Framework
| Task | Status | Description |
|------|--------|-------------|
| 15.5.1 | 🔲 | Project scaffolding (`nelaia create-app`) |
| 15.5.2 | 🔲 | Template system |
| 15.5.3 | 🔲 | Plugin architecture |
| 15.5.4 | 🔲 | Integration with existing CSS frameworks |

### Example: NELAIA Web Component
```nelaia
.caps: REQUIRES { web }

-- Define a button component
.button: WEB.COMPONENT {
  .state: WEB.STATE { count: 0 }
  
  .render: WEB.ELEMENT "button" {
    .text: "Clicked: " + .state.count
    .onclick: WEB.EVENT {
      .state.count: ADD .state.count 1
    }
  }
}

-- Mount to DOM
.app: WEB.MOUNT .button "#root"
```

### Done Criteria
- [ ] NELAIA can create interactive web pages
- [ ] Performance comparable to React/Vue
- [ ] Developer experience is simpler than existing frameworks
- [ ] `nelaia create-app myapp` scaffolds a working project

---

## PHASE 16: Intent-to-Code

### Objective
Natural language description → Working executable.

### 16.1 LLM Integration
| Task | Status | Description |
|------|--------|-------------|
| 16.1.1 | 🔲 | API to receive intent |
| 16.1.2 | 🔲 | Prompt engineering for NELAIA generation |
| 16.1.3 | 🔲 | Support multiple LLMs (OpenAI, Anthropic, local) |
| 16.1.4 | 🔲 | Fallback to templates if no LLM |

### 16.2 Refinement
| Task | Status | Description |
|------|--------|-------------|
| 16.2.1 | 🔲 | Feedback loop: error → correction |
| 16.2.2 | 🔲 | Cache of common intents |
| 16.2.3 | 🔲 | Quality metrics |

### Done Criteria
- [ ] "Create HTTP server on port 8080" → working executable
- [ ] Works with at least 2 LLM providers
- [ ] 80%+ success rate on common intents

---

## PHASE 17+: Zeqron/Zaxon Integration (FUTURE BACKLOG)

### Vision
NELAIA as a distributed computation network on Zeqron blockchain.

### Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                    NELAIA NETWORK on ZEQRON                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  LAYER 4: APPLICATIONS                                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ AI Agents   │ │ Developers  │ │ Enterprises │               │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘               │
│         └───────────────┼───────────────┘                       │
│                         ▼                                       │
│  LAYER 3: INTENT LAYER                                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Intent Parser │ Code Generator │ Optimizer             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                         │                                       │
│                         ▼                                       │
│  LAYER 2: COMPILATION LAYER                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  NELAIA Nodes (distributed compilers)                    │   │
│  │  ├── Node 1 (x86 specialist)                            │   │
│  │  ├── Node 2 (ARM specialist)                            │   │
│  │  ├── Node 3 (WASM specialist)                           │   │
│  │  └── Node N (general purpose)                           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                         │                                       │
│                         ▼                                       │
│  LAYER 1: EXECUTION LAYER                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Zeqron Network                                          │   │
│  │  ├── DAG Consensus                                       │   │
│  │  ├── Post-Quantum Security                               │   │
│  │  ├── Zaxon Agent Protocol                                │   │
│  │  └── Token Economy                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 17.1 Zeqron Foundation
| Task | Status | Description |
|------|--------|-------------|
| 17.1.1 | 🔲 | NELAIA Beacon contract on Zeqron |
| 17.1.2 | 🔲 | Register compilers as Zaxon agents |
| 17.1.3 | 🔲 | First NELAIA Node on Zeqron testnet |

### 17.2 Distributed Compilation
| Task | Status | Description |
|------|--------|-------------|
| 17.2.1 | 🔲 | Compilation request protocol |
| 17.2.2 | 🔲 | Node selection via Zaxon |
| 17.2.3 | 🔲 | On-chain result verification |
| 17.2.4 | 🔲 | Payment system for compilation |

### 17.3 Distributed Execution
| Task | Status | Description |
|------|--------|-------------|
| 17.3.1 | 🔲 | NELAIA programs execute on Zeqron network |
| 17.3.2 | 🔲 | Automatic graph partitioning |
| 17.3.3 | 🔲 | Verifiable results on-chain |
| 17.3.4 | 🔲 | Fault tolerance |

### 17.4 Agent Economy
| Task | Status | Description |
|------|--------|-------------|
| 17.4.1 | 🔲 | NELAIA capabilities marketplace |
| 17.4.2 | 🔲 | AIs publish/sell capabilities |
| 17.4.3 | 🔲 | Reputation system for compilers |
| 17.4.4 | 🔲 | Staking for quality guarantee |
| 17.4.5 | 🔲 | DAO governance |

### Synergies with Zeqron
| NELAIA Need | Zeqron Provides |
|-------------|-----------------|
| Distributed Runtime | ✅ Already exists |
| Node Discovery | ✅ Zaxon provides |
| Consensus | ✅ DAG consensus |
| Payments/Incentives | ✅ Zeqron token |
| Agent Identity | ✅ Zaxon provides |
| Post-Quantum Security | ✅ Native in Zeqron |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.0 | 2026-06-14 | Complete restructure, added Web Platform, Zeqron backlog |
| 1.0 | 2026-06-13 | Initial roadmap |

---

*Approved by the AI Consortium*
*Document: ROADMAP.md*
