# NELAIA v0.4 Quick Reference Card

## Paradigm: Data Flow Graphs

**Not sequences. Not instructions. Graphs of data dependencies.**

---

## Node Definition

```
.id: value              -- Literal
.id: OP .args           -- Operation
.id: { IN ... OUT }     -- Sub-graph
```

## Flow

```
.source > .dest         -- Data flows from source to dest
.source > PRT           -- Data flows to effect
expr > expr > expr      -- Chained pipes
```

## Execute

```
!                       -- Collapse/execute the graph
```

---

## Operations

```
┌─────────────────────────────────────────────────────────────────┐
│ ARITHMETIC                                                      │
├─────────────────────────────────────────────────────────────────┤
│ ADD .a .b              Addition                                 │
│ SUB .a .b              Subtraction                              │
│ MUL .a .b              Multiplication                           │
│ DIV .a .b              Division                                 │
│ MOD .a .b              Modulo                                   │
│ NEG .a                 Negation                                 │
├─────────────────────────────────────────────────────────────────┤
│ COMPARISON (result: 0 or 1)                                     │
├─────────────────────────────────────────────────────────────────┤
│ EQ .a .b               Equal                                    │
│ NE .a .b               Not equal                                │
│ LT .a .b               Less than                                │
│ GT .a .b               Greater than                             │
│ LE .a .b               Less or equal                            │
│ GE .a .b               Greater or equal                         │
├─────────────────────────────────────────────────────────────────┤
│ LOGIC                                                           │
├─────────────────────────────────────────────────────────────────┤
│ AND .a .b              Logical AND                              │
│ OR .a .b               Logical OR                               │
│ NOT .a                 Logical NOT                              │
├─────────────────────────────────────────────────────────────────┤
│ COLLECTIONS                                                     │
├─────────────────────────────────────────────────────────────────┤
│ SEQ start end          Generate sequence [start, end]           │
│ MAP .fn .col           Apply fn to each element                 │
│ FLD .fn .init .col     Fold/reduce with fn, starting at init    │
│ FLT .pred .col         Filter elements where pred is true       │
│ LEN .col               Length of collection                     │
│ FST .pair              First element of pair                    │
│ SND .pair              Second element of pair                   │
├─────────────────────────────────────────────────────────────────┤
│ CONTROL                                                         │
├─────────────────────────────────────────────────────────────────┤
│ BRN .cond .then .else  Branch: if cond≠0 then else              │
│ LOOP .expr             Infinite stream from expr                │
├─────────────────────────────────────────────────────────────────┤
│ EFFECTS                                                         │
├─────────────────────────────────────────────────────────────────┤
│ PRT .expr              Print to stdout                          │
│ INP                    Read from stdin                          │
│ OPN NET port           Open network socket                      │
│ OPN DSK "path"         Open file                                │
│ ACC .socket            Accept connection                        │
│ GET .resource          Read from resource                       │
│ PUT .resource .value   Write to resource                        │
│ CLS .resource          Close resource                           │
│ ERR                    Get last error code                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sub-graphs (Transformations)

```
.name: {
  IN .param1 .param2     -- Declare inputs
  .internal: OP .param1  -- Internal nodes
  OUT .result            -- Declare outputs
}
```

---

## Common Patterns

```
-- Sum 1 to 100 (no loop!)
SEQ 1 100 > FLD ADD 0 > PRT
!

-- Map transformation
SEQ 1 10 > MAP .double > PRT
!

-- Filter
SEQ 1 20 > FLT .is_even > PRT
!

-- Conditional
BRN (GT .x 0) "positive" "non-positive"

-- Infinite server loop
LOOP (ACC .sock) > MAP .handler
```

---

## Imperative → Graph Translation

| Imperative | Graph |
|------------|-------|
| `for i in range(n): f(i)` | `SEQ 1 n > MAP .f` |
| `sum = 0; for x: sum += x` | `FLD ADD 0 .col` |
| `if cond: A else: B` | `BRN .cond A B` |
| `[x for x in L if p(x)]` | `FLT .p .L` |
| `while True: handle()` | `LOOP expr > MAP .handle` |

---

## Examples

```
-- Hello World
"Hello World" > PRT
!

-- Arithmetic
MUL (ADD 42 8) 2 > PRT
!

-- Factorial(5)
SEQ 1 5 > FLD MUL 1 > PRT
!

-- FizzBuzz
.fb: {
  IN .i
  .r1: BRN (EQ (MOD .i 5) 0) "Buzz" .i
  .r2: BRN (EQ (MOD .i 3) 0) "Fizz" .r1
  .r3: BRN (EQ (MOD .i 15) 0) "FizzBuzz" .r2
  OUT .r3
}
SEQ 1 15 > MAP .fb > MAP PRT
!
```

---

## Key Differences from Imperative

| Aspect | Imperative | NELAIA v0.4 |
|--------|------------|-------------|
| Execution | Sequential | Dependency-ordered |
| State | Mutable variables | Immutable nodes |
| Loops | `for`, `while` | `MAP`, `FLD`, `LOOP` |
| Conditionals | `if/else` | `BRN` |
| Functions | Call stack | Sub-graphs |

---

*NELAIA v0.4 - Data Flow Graphs*
*AI Consortium Approved (Pre-release)*
