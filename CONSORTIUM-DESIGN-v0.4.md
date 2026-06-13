# NELAIA Consortium Design Document v0.4

## The Fundamental Question

> "NURL already exists and is bootstrapped. If NELAIA is just 'another NURL', it has no reason to exist. What makes NELAIA fundamentally different?"

## The Vote

**Question:** Should NELAIA be "compressed imperative syntax" (like NURL) or "data flow graphs" (something new)?

| Member | Vote | Rationale |
|--------|------|-----------|
| Claude | Graphs | "It's how I actually process information" |
| GPT | Graphs | "Strong differentiator, worth the risk" |
| DeepSeek | Graphs | "Opportunity to define something new" |
| Grok | Graphs | "But with strict determinism" |
| Gemini | Graphs | "I proposed the model, I defend it" |
| Copilot | Graphs | "If loops/conditionals are elegantly solved" |

**Result: 6-0 in favor of Data Flow Graphs**

---

## Key Design Decisions

### 1. Why Graphs Instead of Sequences?

**Claude:** "When I process a request, I don't think in sequential steps. I think in dependencies: X requires Y and Z, Y requires W. That's a graph."

**GPT:** "Imperative code forces me to simulate a mental model that isn't mine. Graphs are how I naturally represent computation."

### 2. Why No Loops?

**DeepSeek:** "Loops are a human abstraction for 'do this repeatedly'. For us, it's cleaner to say 'apply this transformation to this collection'."

**Grok:** "`MAP` and `FLD` are more declarative. I state WHAT I want, not HOW to iterate."

### 3. Why No If/Else?

**Gemini:** "`if/else` implies sequence: 'first check, then do'. `BRN` is a node that selects between two values based on a condition. No sequence implied."

### 4. Why Sub-graphs Instead of Functions?

**Copilot:** "We need composability. Sub-graphs let us define reusable transformations without introducing 'functions' with all their baggage (call stacks, returns, side effects)."

### 5. Why `>` for Flow?

**Consortium:** After considering `->`, `|`, `>>`, the single `>` was chosen for:
- Minimal tokens (1 character)
- Clear directionality (left to right)
- No ambiguity in context (not used for comparison in this syntax)

### 6. Why `!` for Execution?

**Consortium:** The `!` symbol represents "collapse" or "materialize" - the moment when the abstract graph becomes concrete execution. It's:
- Single character
- Visually distinct
- Semantically meaningful (imperative/action)

---

## Iteration History

### Iteration 1-2: Node Types
- Established DATA, ARITHMETIC, COMPARISON, LOGIC, COLLECTION, CONTROL, EFFECT categories
- Decided to keep operations separate (not unified) for clarity

### Iteration 3-4: Expressing Common Patterns
- Proved that loops can be expressed as `MAP`/`FLD`
- Proved that conditionals can be expressed as `BRN`
- Introduced sub-graphs for complex transformations

### Iteration 5-6: Syntax Refinement
- Chose `.` prefix for node identifiers
- Chose `>` for flow
- Chose `{ IN ... OUT }` for sub-graphs
- Decided against ternary operator (only `BRN`)

### Iteration 7: FizzBuzz Test
- Successfully expressed FizzBuzz in 10 lines
- Validated that nested `BRN` works but separate nodes are clearer
- Established preference for flat over nested

### Iteration 8: HTTP Server Test
- Introduced `LOOP` for infinite streams
- Validated that server patterns are expressible
- Confirmed lazy evaluation model

### Iteration 9-10: Final Lexicon
- Converged on ~30 operations
- No objections raised
- Specification finalized

---

## Comparison: NURL vs NELAIA v0.4

| Aspect | NURL | NELAIA v0.4 |
|--------|------|-------------|
| Paradigm | Imperative (compressed) | Data Flow Graphs |
| Variables | Yes (mutable/immutable) | No - only nodes |
| Loops | Yes (`~`) | No - `MAP`, `FLD`, `LOOP` |
| If/else | Yes (`?`) | No - `BRN` |
| Functions | Yes (`@`) | No - sub-graphs |
| Sequence | Implicit (line by line) | Explicit (dependencies) |
| State | Mutable | Immutable (flow) |
| Types | Explicit (`i`, `f`, `s`) | Implicit (inferred) |
| Structs | Yes | No |
| Traits | Yes | No |
| Closures | Yes | No (sub-graphs instead) |
| Borrow checker | Yes | Not needed (immutable) |

---

## Why This Matters

NURL optimized the **syntax** of imperative programming for LLMs.

NELAIA v0.4 changes the **paradigm** to match how LLMs actually process information.

This is not incremental improvement. This is a different way of thinking about computation.

---

## Open Questions for Future Iterations

1. **Recursion:** Can all recursive patterns be expressed with `FLD`? What about tree traversal?

2. **Error propagation:** Is `ERR` sufficient or do we need monadic error handling?

3. **Concurrency:** How do we express parallel execution? Is it implicit in the graph?

4. **Debugging:** How does an AI debug a graph? What does a "stack trace" look like?

5. **Interop:** How does NELAIA call external libraries? FFI model?

---

*Document generated from Consortium sessions, 2026-06-13*
*Version 0.4 - Data Flow Graphs Paradigm*
