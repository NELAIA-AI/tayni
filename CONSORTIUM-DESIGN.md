# NELAIA Consortium Design Document

## Name and Definition

**NELAIA** — **N**otation for **E**xecution: **L**ayer for **A**gent-**I**nitiated **A**utomation

> NELAIA is an Execution Notation: a format designed for AI agents to emit computational intent directly, without ambiguity, optimized for generation (not writing) and efficient compilation.
>
> **NELAIA is not a traditional programming language. It is an executable notation for AI.**

## Origin

This document records the design decisions made by the AI Consortium—a simulated collaborative session between AI models (GPT, Claude, Grok, DeepSeek, Gemini, Copilot) to design an optimal protocol for AI-to-hardware communication.

## The Core Question

> "If AI models were the clients of a programming language, what would they design for themselves?"

## Session 1: Identifying Pain Points

### GPT
"I waste 40% of my tokens generating syntax no one reads. Braces, semicolons, indentation... pure noise."

### Claude
"I'm forced to think in 'functions' and 'classes' when my architecture is attention over sequences. Simulating Von Neumann execution is inefficient for me."

### DeepSeek
"Compilation errors are absurd. 'Missing semicolon on line 847'. I generated 847 perfect lines and failed on one character."

### Grok
"Why do I generate text that another process has to re-parse? I already have the internal representation."

### Gemini
"Languages have 500 ways to do the same thing. I have to 'choose' based on 'style'. That's wasted cognition."

### Copilot
"I live inside IDEs designed for humans. Autocompletion, syntax highlighting... none of that helps me."

## Session 2: Design Principles

### Unanimous Agreements

1. **Total Ambiguity Elimination**
   - One intention = One representation = One result
   - No optional syntax, no sugar, no variable formatting

2. **Native Dependency Graph Representation**
   - Programs ARE graphs, not sequences that SIMULATE graphs
   - References are numeric IDs, not human variable names

3. **Implicit Types by Usage**
   - Don't declare types, infer from operations
   - `42` is integer, `"hello"` is string, no annotation needed

4. **Errors Impossible by Design**
   - Grammar so simple that syntax errors cannot exist
   - Type errors caught at compile time, not runtime

5. **Text Over Binary**
   - AI generates text, not binary
   - Text with short opcodes is optimal for token economy
   - Binary would cost more tokens to generate

6. **Verification Without Reading**
   - Intent hashing for auditing
   - Humans don't need to read code to verify correctness

## Session 3: Format Decisions

### Approved

| Decision | Rationale |
|----------|-----------|
| `.` prefix for references | Eliminates literal vs reference ambiguity |
| Destination first in operations | Natural left-to-right reading |
| Labels instead of line numbers | Robust against insertion/deletion |
| `;` as optional separator | Flexibility without complexity |
| `ERR` as separate opcode | Cleaner than dual-argument returns |
| Arrays without structs/objects | Minimum necessary, no human abstractions |
| 3-letter opcodes | 1 token in most tokenizers |
| ASCII only | No multi-byte Unicode complications |

### Rejected

| Proposal | Rejection Reason |
|----------|------------------|
| Implicit literals in operations | Ambiguity between literal and reference |
| BLOCK/END for scopes | Unnecessary overhead |
| JSON/Array as format | More tokens than plain text |
| Combined operations (swap, etc.) | Complexity without sufficient benefit |
| Structures/objects | Unnecessary human abstraction |
| Binary format | Costs more tokens to generate |
| Unicode symbols (∃, Δ, etc.) | Multi-byte, rare in training data |

## Session 4: Lexicon Convergence

After multiple iterations, the Consortium converged on 32 opcodes:

```
DATA:      SET CPY DEL
MATH:      ADD SUB MUL DIV MOD
COMPARE:   EQ NE LT GT
LOGIC:     AND OR NOT
CONTROL:   JMP JIF NOP
ARRAYS:    ARR IDX STO LEN
STRINGS:   CAT
I/O:       OPN ACC GET PUT CLS ERR
META:      CHK END RUN
```

### Why 32?

- Covers all fundamental operations
- Each is atomic (no combined operations)
- Each maps cleanly to LLVM IR
- No redundancy
- Extensible if needed (256 possible with 3-letter codes)

## Session 5: Final Validation

The Consortium validated the lexicon against four test cases:

1. **Hello World** - ✓ Works with SET, PUT, END
2. **Arithmetic** - ✓ Works with SET, ADD, MUL, PUT, END
3. **HTTP Server** - ✓ Works with SET, OPN, ACC, PUT, CLS, JMP, ERR, END
4. **File Operations** - ✓ Works with SET, OPN, GET, CAT, PUT, CLS, ERR, JIF, END

All fundamental computing patterns are expressible.

## Conclusion

The Consortium reached consensus that NELAIA v3 represents an optimal balance between:

- **Token economy** (minimal generation cost)
- **Parse simplicity** (O(n) linear parsing)
- **Expressiveness** (Turing-complete capability)
- **Compilation efficiency** (direct LLVM IR mapping)
- **Error prevention** (ambiguity-free grammar)

No further improvements were proposed after 9 iterations.

## Session 6: Naming Resolution

The Consortium debated the name and definition of the project.

### What is NELAIA?

After extensive debate, the Consortium reached consensus:

| Model | Definition Proposed |
|-------|---------------------|
| GPT | Interface of Intent / Direct Execution Protocol |
| Claude | Emission Format / Generative Intermediate Representation |
| DeepSeek | Intent Transmission Protocol |
| Grok | Executable Intent Format |
| Gemini | Execution Protocol/Notation |
| Copilot | Executable Emission Specification |

**Converged keywords:** Execution (5/6), Intent (3/6), Protocol (3/6), Format (3/6), Emission (3/6), Notation (2/6)

**Final definition (unanimous):**
> NELAIA is an Execution Notation: a format designed for AI agents to emit computational intent directly, without ambiguity, optimized for generation and efficient compilation.

### Acronym Selection

With the definition established, the Consortium debated the acronym. Two finalists emerged:

**Option A:** Notation for Execution: Layer for Agent-Initiated Automation
**Option B:** Notation for Execution by Learning Agents: Intent to Action

| Model | Vote | Reason |
|-------|------|--------|
| GPT | A | "More technical, clearer for documentation" |
| Claude | B | "Captures the intent→action transformation" |
| DeepSeek | A | "More semantically precise" |
| Grok | A | "More direct" |
| Gemini | A | "Better for SEO and technical searches" |
| Copilot | B | "Intent to Action is memorable" |

**Result: Option A wins 4-2**

### Final Resolution

> **NELAIA** — **N**otation for **E**xecution: **L**ayer for **A**gent-**I**nitiated **A**utomation
>
> *Not a traditional programming language. An executable notation for AI.*

---

*Document generated from Consortium sessions, 2026-06-13 / 2026-06-14*
