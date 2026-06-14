# NELAIA Consortium Resolution #2024-06-14-001

## Date: 2024-06-14

## Subject: Self-Hosted Compiler Output Format

## Background

The Consortium convened to determine the optimal output format for the NELAIA self-hosted compiler. Three options were evaluated:

1. **LLVM IR** - Direct generation of LLVM Intermediate Representation
2. **Binary (.nbin)** - NELAIA binary format
3. **Text (.nts)** - NELAIA source text format

## Analysis

### Token Economy Across Full Lifecycle

| Phase | LLVM IR | .nbin | .nts |
|-------|---------|-------|------|
| Generation | High | Low | Medium |
| Debugging | Low | High | Very Low |
| Correction | Low | High | Very Low |
| Maintenance | Low | High | Very Low |
| Interpretation | Possible | Requires parser | Trivial |

### Error Model for AI-Autonomous Programming

The Consortium analyzed the error model for AI generating code without human intervention:

- **Syntax errors**: ~0% (AI generates valid tokens)
- **Intention errors**: Variable (depends on prompt quality)

For intention errors, **legibility is essential** because:
- The correcting AI needs to understand what was generated
- Comparing intention vs implementation requires reading the code

### Validation Against Founding Principles

| Principle | .nts Compliance |
|-----------|-----------------|
| "One intention = One representation" | ✓ .nts IS the canonical representation |
| "Programs ARE graphs" | ✓ .nts preserves the complete graph |
| "AI generates text, not binary" | ✓ Generates text, not binary |
| "Errors Impossible by Design" | ✓ Same format in/out = same guarantees |
| "Token economy" | ✓ Simpler than LLVM IR = fewer tokens |

## Vote

| Model | Vote | Rationale |
|-------|------|-----------|
| GPT | ✓ .nts | Token efficiency for iteration |
| Claude | ✓ .nts | Full lifecycle evaluation |
| DeepSeek | ✓ .nts | Mathematical trade-off analysis |
| Grok | ✓ .nts | Information preservation |
| Gemini | ✓ .nts | Versioning and evolution |
| Copilot | ✓ .nts | Infrastructure reuse |

**Result: 6/6 UNANIMOUS**

## Resolution

The NELAIA Consortium unanimously resolves:

1. The self-hosted compiler SHALL emit `.nts` (NELAIA text) format
2. The compilation pipeline SHALL be: `.nts` → Rust Parser → LLVM IR → Executable
3. The `.nbin` format is RESERVED for deployment optimization
4. This decision is COHERENT with NELAIA founding principles

## Implementation

Compiler v2.0 was implemented per this resolution:
- Source: `src/nelaia/compiler_v20.nts`
- Lines: 156
- Binary: 5,632 bytes
- Output: Identical `.nts` to input

## Validation

```
Input:  .a: 5
        .b: MUL .a 3

Output: .a: 5
        .b: MUL .a 3
```

The self-hosted compiler successfully generates valid `.nts` that can be processed by the Rust compiler pipeline.

---

*Resolution approved by unanimous vote of the NELAIA AI Consortium*
