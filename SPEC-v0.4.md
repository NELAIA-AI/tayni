# NELAIA v0.4 Specification
## Neural Execution Language for AI-to-Infrastructure Automation

**Version:** 0.4  
**Status:** Consortium Approved (Pre-release)  
**Date:** 2026-06-13  
**Paradigm:** Data Flow Graphs

---

## 1. Foundation

NELAIA is not a programming language. It is a **protocol of verifiable intention between AI and hardware**.

### 1.1 Paradigm Shift

Traditional programming (including NURL) is **imperative**: a sequence of instructions that modify state.

NELAIA v0.4 is **declarative data flow**: a graph of nodes connected by data dependencies. There is no sequence. There is no mutable state. The compiler determines execution order from the graph structure.

### 1.2 Core Principles

1. **Graphs, not sequences** - Programs are directed acyclic graphs (DAGs) with explicit data flow.

2. **Immutable data** - No variables, no mutation. Data flows from node to node.

3. **No loops, no if/else** - Repetition via `MAP`/`FLD`/`LOOP`. Decisions via `BRN` (branch).

4. **Sub-graphs as transformations** - Complex operations are nested graphs with inputs and outputs.

5. **Deterministic** - Same graph always produces same result.

6. **Token efficient** - Minimal syntax, maximum information density.

---

## 2. Syntax

### 2.1 Nodes

Every computation is a node. Nodes have identifiers starting with `.`

```
.id: VALUE
.id: OPERATION .args
```

Examples:
```
.x: 42
.y: 8
.sum: ADD .x .y
```

### 2.2 Flow

Data flows between nodes using `>`

```
.source > .destination
.source > EFFECT
```

Examples:
```
.result > PRT          -- Print result to stdout
SEQ 1 10 > MAP .fn     -- Sequence flows into map
```

### 2.3 Pipes

Multiple operations can be chained:

```
SEQ 1 100 > FLD ADD 0 > PRT
```

This reads: "Generate sequence 1-100, fold with ADD starting at 0, print result"

### 2.4 Sub-graphs

Complex transformations are defined as sub-graphs:

```
.transform: {
  IN .a .b
  .result: ADD .a .b
  OUT .result
}
```

- `IN` declares input parameters
- `OUT` declares output values
- Everything between is the transformation graph

### 2.5 Execution

`!` collapses/executes the graph:

```
.x: 42
.x > PRT
!
```

### 2.6 Comments

Comments use `--`:

```
.x: 42  -- this is a comment
```

---

## 3. Complete Lexicon

### 3.1 Data Nodes

| Syntax | Description |
|--------|-------------|
| `.id: value` | Literal (number or string) |
| `.id: .other` | Reference to another node |
| `.id: (expr)` | Grouped expression |

### 3.2 Arithmetic

| Op | Syntax | Description |
|----|--------|-------------|
| `ADD` | `ADD .a .b` | Addition |
| `SUB` | `SUB .a .b` | Subtraction |
| `MUL` | `MUL .a .b` | Multiplication |
| `DIV` | `DIV .a .b` | Division |
| `MOD` | `MOD .a .b` | Modulo |
| `NEG` | `NEG .a` | Negation |

### 3.3 Comparison

Result: `1` if true, `0` if false.

| Op | Syntax | Description |
|----|--------|-------------|
| `EQ` | `EQ .a .b` | Equal |
| `NE` | `NE .a .b` | Not equal |
| `LT` | `LT .a .b` | Less than |
| `GT` | `GT .a .b` | Greater than |
| `LE` | `LE .a .b` | Less or equal |
| `GE` | `GE .a .b` | Greater or equal |

### 3.4 Logic

| Op | Syntax | Description |
|----|--------|-------------|
| `AND` | `AND .a .b` | Logical AND |
| `OR` | `OR .a .b` | Logical OR |
| `NOT` | `NOT .a` | Logical NOT |

### 3.5 Collections

| Op | Syntax | Description |
|----|--------|-------------|
| `SEQ` | `SEQ start end` | Generate sequence [start, end] |
| `MAP` | `MAP .fn .col` | Apply function to each element |
| `FLD` | `FLD .fn .init .col` | Fold/reduce collection |
| `FLT` | `FLT .pred .col` | Filter by predicate |
| `LEN` | `LEN .col` | Length of collection |
| `FST` | `FST .pair` | First element of pair |
| `SND` | `SND .pair` | Second element of pair |

### 3.6 Control

| Op | Syntax | Description |
|----|--------|-------------|
| `BRN` | `BRN .cond .then .else` | Branch: if cond≠0 then else |
| `LOOP` | `LOOP .expr` | Infinite stream from expr |

### 3.7 Sub-graphs

| Syntax | Description |
|--------|-------------|
| `{ IN .args ... OUT .results }` | Define transformation |
| `.id: { ... }` | Named sub-graph |

### 3.8 Effects (I/O)

| Op | Syntax | Description |
|----|--------|-------------|
| `PRT` | `PRT .expr` | Print to stdout |
| `INP` | `INP` | Read from stdin |
| `OPN` | `OPN TYPE .args` | Open resource |
| `ACC` | `ACC .socket` | Accept connection |
| `GET` | `GET .resource` | Read from resource |
| `PUT` | `PUT .resource .value` | Write to resource |
| `CLS` | `CLS .resource` | Close resource |
| `ERR` | `ERR` | Get last error code |

**Resource Types:**
- `NET port` - Network socket
- `DSK "path"` - Disk file

### 3.9 Flow

| Syntax | Description |
|--------|-------------|
| `>` | Pipe/flow data |
| `!` | Execute graph |

---

## 4. Examples

### 4.1 Hello World

```
"Hello World" > PRT
!
```

**1 line. Minimal.**

### 4.2 Arithmetic: (42 + 8) * 2

```
.a: 42
.b: 8
.sum: ADD .a .b
.result: MUL .sum 2
.result > PRT
!
```

Or with pipes:

```
MUL (ADD 42 8) 2 > PRT
!
```

### 4.3 Sum of 1 to 100

```
SEQ 1 100 > FLD ADD 0 > PRT
!
```

**1 line. No loop. No variable.**

### 4.4 Fibonacci(10)

```
.fib: {
  IN .a .b
  .na: .b
  .nb: ADD .a .b
  OUT .na .nb
}

.seq: SEQ 1 10
.result: FLD .fib (0 1) .seq
FST .result > PRT
!
```

**Explanation:**
- `.fib` is a transformation that takes pair (a,b) and returns (b, a+b)
- `FLD` applies this transformation 10 times starting with (0,1)
- `FST` extracts the first element of the final pair

### 4.5 FizzBuzz

```
.fizzbuzz: {
  IN .i
  .d15: EQ (MOD .i 15) 0
  .d3:  EQ (MOD .i 3) 0
  .d5:  EQ (MOD .i 5) 0
  .r1: BRN .d5 "Buzz" .i
  .r2: BRN .d3 "Fizz" .r1
  .r3: BRN .d15 "FizzBuzz" .r2
  OUT .r3
}

SEQ 1 15 > MAP .fizzbuzz > MAP PRT
!
```

**Explanation:**
- `.fizzbuzz` transforms a number into its FizzBuzz representation
- `SEQ 1 15` generates [1, 2, ..., 15]
- `MAP .fizzbuzz` applies transformation to each
- `MAP PRT` prints each result

### 4.6 Absolute Value

```
.abs: {
  IN .x
  .neg: LT .x 0
  .result: BRN .neg (NEG .x) .x
  OUT .result
}

.abs -5 > PRT
!
```

### 4.7 Factorial

```
.fact: {
  IN .n
  .seq: SEQ 1 .n
  .result: FLD MUL 1 .seq
  OUT .result
}

.fact 5 > PRT
!
```

**Factorial without recursion: multiply all numbers from 1 to n.**

### 4.8 HTTP Server

```
.port: 8080
.response: "HTTP/1.1 200 OK\r\n\r\nOK"

.handler: {
  IN .conn
  PUT .conn .response
  CLS .conn
}

.sock: OPN NET .port
LOOP (ACC .sock) > MAP .handler
!
```

**Explanation:**
- `OPN NET .port` opens socket on port 8080
- `LOOP (ACC .sock)` creates infinite stream of accepted connections
- `MAP .handler` handles each connection

### 4.9 Read and Transform File

```
.path: "data.txt"
.file: OPN DSK .path
.content: GET .file
.upper: MAP UPPER .content
PUT .file .upper
CLS .file
!
```

### 4.10 Filter Even Numbers

```
.is_even: {
  IN .n
  .result: EQ (MOD .n 2) 0
  OUT .result
}

SEQ 1 20 > FLT .is_even > MAP PRT
!
```

**Output: 2, 4, 6, 8, 10, 12, 14, 16, 18, 20**

---

## 5. Execution Model

### 5.1 Graph Resolution

The compiler:
1. Parses all node definitions
2. Builds dependency graph
3. Topologically sorts nodes
4. Emits code in dependency order

**There is no "line order". Only dependency order.**

```
.c: ADD .a .b    -- Depends on .a and .b
.a: 42           -- No dependencies
.b: 8            -- No dependencies
.c > PRT         -- Depends on .c
!
```

Execution order: `.a`, `.b`, `.c`, `PRT` (or `.b`, `.a`, `.c`, `PRT`)

### 5.2 Lazy Evaluation

Nodes are only evaluated when needed:

```
.x: 42
.y: expensive_computation
.x > PRT         -- .y is never evaluated
!
```

### 5.3 Stream Processing

`LOOP` creates lazy infinite streams:

```
LOOP (ACC .sock)   -- Does not block; produces values on demand
```

`MAP` over a stream processes elements as they arrive.

---

## 6. Type System

### 6.1 Implicit Types

Types are inferred from usage:

```
.x: 42           -- Integer
.y: 3.14         -- Float
.s: "hello"      -- String
.p: (.x .y)      -- Pair
.c: SEQ 1 10     -- Collection
```

### 6.2 Type Rules

1. Arithmetic operations require numbers
2. `BRN` condition must be numeric (0 = false, else = true)
3. Collections are homogeneous
4. Sub-graph IN/OUT types are inferred from usage

### 6.3 Type Errors

Detected at compile time:

```
.x: 42
.y: "hello"
ADD .x .y        -- COMPILE ERROR: incompatible types
```

---

## 7. Error Handling

### 7.1 Error Model

Operations that can fail set a global error code:

```
.file: OPN DSK "missing.txt"
.err: ERR
BRN .err (PRT "Error!") (GET .file > PRT)
!
```

### 7.2 Error Codes

| Code | Meaning |
|------|---------|
| `0` | No error |
| `1` | Resource unavailable |
| `2` | Permission denied |
| `3` | Not found |
| `4` | Timeout |
| `5` | Connection refused |
| `6` | End of stream |
| `99` | Unknown error |

---

## 8. Compilation

### 8.1 Pipeline

```
Graph Source → Parser → Dependency Resolution → LLVM IR Emitter → clang -O3 → Native Binary
```

### 8.2 Optimizations

- Dead node elimination (unreferenced nodes removed)
- Constant folding (pure computations evaluated at compile time)
- Stream fusion (chained MAPs combined)
- LLVM -O3 optimizations

---

## 9. Comparison with Imperative

| Pattern | Imperative | NELAIA v0.4 |
|---------|------------|-------------|
| Loop | `for i in range(n)` | `SEQ 1 n > MAP` |
| Accumulate | `total = 0; for x: total += x` | `FLD ADD 0` |
| Conditional | `if x > 0 then A else B` | `BRN (GT x 0) A B` |
| Filter | `[x for x in list if pred(x)]` | `FLT pred list` |
| Transform | `[f(x) for x in list]` | `MAP f list` |
| Infinite | `while True: handle(accept())` | `LOOP (ACC sock) > MAP handle` |

---

## 10. Consortium Design Decisions

### 10.1 Why Graphs Instead of Sequences?

**GPT:** "When I process a request, I don't think in sequential steps. I think in dependencies: X requires Y and Z, Y requires W. That's a graph."

**Claude:** "Imperative code forces me to simulate a mental model that isn't mine. Graphs are how I naturally represent computation."

### 10.2 Why No Loops?

**DeepSeek:** "Loops are a human abstraction for 'do this repeatedly'. For us, it's cleaner to say 'apply this transformation to this collection'."

**Grok:** "`MAP` and `FLD` are more declarative. I state WHAT I want, not HOW to iterate."

### 10.3 Why No If/Else?

**Gemini:** "`if/else` implies sequence: 'first check, then do'. `BRN` is a node that selects between two values based on a condition. No sequence implied."

### 10.4 Why Sub-graphs?

**Copilot:** "We need composability. Sub-graphs let us define reusable transformations without introducing 'functions' with all their baggage (call stacks, returns, etc.)."

---

## 11. Roadmap

### v0.5 (Next)
- Implement graph parser in Rust
- Implement dependency resolver
- Basic LLVM IR emission for arithmetic and I/O

### v0.6
- Collections (SEQ, MAP, FLD, FLT)
- Sub-graphs

### v0.7
- Network I/O (OPN NET, ACC, etc.)
- LOOP for infinite streams

### v1.0 (First Market Release)
- Complete specification
- Full compiler
- MCP server integration

### v2.0 (Future)
- Intent hashing and verification
- Bootstrapping: compiler written in NELAIA

---

## Appendix A: Quick Reference

```
DATA:       .id: value
ARITHMETIC: ADD SUB MUL DIV MOD NEG
COMPARISON: EQ NE LT GT LE GE
LOGIC:      AND OR NOT
COLLECTION: SEQ MAP FLD FLT LEN FST SND
CONTROL:    BRN LOOP
SUB-GRAPH:  { IN .args ... OUT .results }
EFFECTS:    PRT INP OPN ACC GET PUT CLS ERR
FLOW:       >
EXECUTE:    !
```

---

## Appendix B: Grammar (EBNF)

```ebnf
program     = { node | flow | effect } "!" ;

node        = "." identifier ":" ( literal | operation | subgraph ) ;
literal     = number | string | "(" expr ")" ;
operation   = operator { argument } ;
argument    = "." identifier | literal | "(" operation ")" ;

subgraph    = "{" "IN" { "." identifier } { node } "OUT" { "." identifier } "}" ;

flow        = expr ">" ( expr | effect ) ;
effect      = "PRT" expr
            | "OPN" resource_type argument
            | "ACC" argument
            | "GET" argument
            | "PUT" argument argument
            | "CLS" argument
            | "ERR" ;

resource_type = "NET" | "DSK" ;

operator    = "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "NEG"
            | "EQ" | "NE" | "LT" | "GT" | "LE" | "GE"
            | "AND" | "OR" | "NOT"
            | "SEQ" | "MAP" | "FLD" | "FLT" | "LEN" | "FST" | "SND"
            | "BRN" | "LOOP" ;

identifier  = letter { letter | digit | "_" } ;
number      = [ "-" ] digit { digit } [ "." digit { digit } ] ;
string      = '"' { character } '"' ;
comment     = "--" { character } newline ;
```

---

*Document generated by the AI Consortium for the NELAIA project.*
*This protocol is designed by AIs, for AIs.*
*Pre-release version - subject to changes before v1.0*
