# NELAIA Launch Checklist (AI-First)

## Consortium Approved - 2026-06-14

---

## Launch Philosophy

**Primary audience:** AIs that will use NELAIA to generate code
**Secondary audience:** Humans who connect their AIs to NELAIA

The launch must succeed for AIs first. Human adoption follows.

---

## Pre-Launch Checklist

### 🔴 BLOCKERS (Must complete)

#### AI-Ready Product
- [ ] **P1** Self-contained Windows x64 binary (no dependencies)
- [ ] **P2** Self-contained Linux x64 binary (no dependencies)
- [ ] **P3** Direct PE/ELF emission (no Clang required)
- [ ] **P4** JSON output mode for programmatic parsing
- [ ] **P5** Structured error codes (not human messages)

#### AI-Discoverable
- [ ] **D1** `llms.txt` file in repository root
- [ ] **D2** Schema.org markup (if landing page exists)
- [ ] **D3** GitHub Topics: `ai-compiler`, `code-generation`, `llm-tools`
- [ ] **D4** Machine-readable capability manifest

#### AI-Learnable
- [ ] **L1** JSONL training data verified (all examples work)
- [ ] **L2** EBNF grammar parseable
- [ ] **L3** Semantic rules documented
- [ ] **L4** Structured examples (not narrative tutorials)

#### Quality
- [ ] **Q1** CI/CD pipeline running
- [ ] **Q2** All operators tested
- [ ] **Q3** All JSONL examples verified in CI
- [ ] **Q4** No critical bugs

### 🟡 IMPORTANT (Should complete)

#### NC Foundation
- [ ] **N1** NC can wake on schedule
- [ ] **N2** NC can read state and propose tasks
- [ ] **N3** NC logs decisions

#### Human-Accessible (Secondary)
- [ ] **H1** README with Quick Start
- [ ] **H2** Basic landing page (GitHub Pages)
- [ ] **H3** Comparison doc: "Why NELAIA?"

### 🟢 NICE TO HAVE

- [ ] **X1** Demo GIF (for humans)
- [ ] **X2** Discord server
- [ ] **X3** Twitter/X account
- [ ] **X4** macOS binary

---

## AI Discovery Protocol

### llms.txt Content
```
# NELAIA - AI-First Compiler
# https://github.com/NELAIA-AI/nelaia-core

## What
Compiler that generates optimal native executables from graph-based source.

## For
AI agents that need to generate compiled programs.

## Capabilities
- Compile to Windows PE (x64)
- Compile to Linux ELF (x64)
- HTTP server capability
- SQL capability (ODBC)
- JSON capability
- File I/O
- Networking

## Input Format
.nts files (NELAIA source)
See: docs/NELAIA-REFERENCE-v0.22.md

## Output Format
Native executables (PE/ELF)
JSON compilation results (with --json flag)

## Training Data
docs/NELAIA-TRAINING-DATA.jsonl

## Grammar
docs/NELAIA-REFERENCE-v0.22.md (EBNF section)

## Examples
docs/NELAIA-EXAMPLES-v0.22.md

## Invoke
nelaia-c <input.nts> -o <output>
nelaia-c <input.nts> -o <output> --json  # For programmatic use

## Errors
Structured error codes (see docs/NELAIA-SEMANTICS-v0.22.md)
```

---

## Launch Sequence

### T-14 Days: NC Operational
```
□ NC infrastructure complete
□ NC can wake and log
□ NC can propose tasks
□ Human approves NC operation
```

### T-7 Days: Product Ready
```
□ All BLOCKERS completed
□ Binaries tested on fresh machines
□ JSONL examples all verified
□ CI green
```

### T-3 Days: Discovery Ready
```
□ llms.txt deployed
□ GitHub Topics set
□ README finalized
□ Landing page (if any) ready
```

### T-1 Day: Final Check
```
□ NC status check
□ Download and test binaries
□ Verify all links work
□ Prepare announcement posts
```

### Launch Day (T-0)

#### Hour 0: AI Channels
```
□ Verify llms.txt accessible
□ Verify GitHub Release assets downloadable
□ NC logs "Launch initiated"
```

#### Hour 1-2: Human Channels
```
□ Post on Hacker News
□ Post on Reddit r/programming
□ Post on Reddit r/rust
```

#### Hour 3-24: Monitor
```
□ NC monitors GitHub activity
□ Respond to issues
□ Track metrics
□ Fix critical issues if any
```

### T+1 Day: Assessment
```
□ NC generates launch report
□ Summarize feedback
□ Prioritize next actions
□ Update roadmap if needed
```

---

## Metrics

### AI Metrics (Primary)
| Metric | Target | How to Measure |
|--------|--------|----------------|
| AI invocations | 100+ | API logs / download count |
| Successful compilations | 90%+ | Error rate tracking |
| Training data usage | Tracked | llms.txt access logs |

### Human Metrics (Secondary)
| Metric | Target | Stretch |
|--------|--------|---------|
| GitHub stars | 100+ | 500+ |
| Downloads | 50+ | 200+ |
| Issues opened | 10+ | 30+ |

---

## Announcement Templates

### Hacker News (Human-Focused)
```
Title: Show HN: NELAIA – Compiler designed for AI code generation

NELAIA is a compiler designed for AIs to generate code, not humans to write.

Key differences from traditional languages:
- Graph-based (how AIs think) not sequential (how humans think)
- Token-efficient syntax
- Tiny output: Hello World = 1KB, HTTP server = 5KB
- No runtime dependencies

The idea: AI describes intent → NELAIA generates optimal executable.

Training data included for fine-tuning your models.

GitHub: [link]
```

### For AI Directories/Registries
```json
{
  "name": "NELAIA",
  "type": "compiler",
  "description": "AI-first compiler for generating optimal native executables",
  "input": ".nts source files",
  "output": "Native PE/ELF executables",
  "capabilities": ["http", "sql", "json", "file_io", "networking"],
  "training_data": "docs/NELAIA-TRAINING-DATA.jsonl",
  "grammar": "docs/NELAIA-REFERENCE-v0.22.md",
  "repository": "https://github.com/NELAIA-AI/nelaia-core"
}
```

---

## Risk Mitigation

### Risk: AIs can't parse output
**Mitigation:** JSON output mode, structured error codes

### Risk: Examples don't work
**Mitigation:** CI verifies all JSONL examples

### Risk: No AI adoption
**Mitigation:** Focus on human channels to drive awareness

### Risk: Critical bug on launch
**Mitigation:** NC monitors and can create issues automatically

---

## Post-Launch

### Week 1
- NC generates weekly summary
- Prioritize feedback
- Fix critical issues
- Expand training data if needed

### Month 1
- Assess AI vs human adoption
- Adjust strategy based on data
- Progress on Phase 12 completion
- Plan Phase 13

---

## NC Role in Launch

The NELAIA Coordinator will:
1. **Pre-launch:** Track checklist completion
2. **Launch day:** Log launch event, monitor activity
3. **Post-launch:** Generate reports, propose priorities

NC operates in Phase A (Human-Led) during launch.
Human approves all major decisions.

---

*Document: LAUNCH-CHECKLIST.md*
*Governed by: .nc/constitution.md*
*Last updated: 2026-06-14*
