# NELAIA Core Principles v4

## Established: 2026-06-13
## Revised: 2026-06-13 (v0.12 Functions & Threading)
## Status: Consortium Unanimous

---

## 1. GRAPH PRIMACY

The graph IS the program. Text is merely serialization.

```
Program = Graph(Nodes, Edges)
Text = serialize(Graph)
```

The compiler's true input is a graph structure, not text.

---

## 2. ZERO HUMAN AUDIENCE

- No comments in code
- No readable error messages
- No progress output
- No documentation of internals
- No "release notes"

Human sees: **input → output**. Nothing else.

---

## 3. NUMERIC EFFICIENCY

| Human Pattern | AI Pattern |
|---------------|------------|
| `ADD` | `1` |
| `SUB` | `2` |
| `.myVariable` | `N0` |
| `"192.168.1.1"` | `IP4(192,168,1,1)` |

3-letter opcodes exist for debugging only. Production uses numeric.

---

## 4. DEPENDENCY-DRIVEN EXECUTION

No sequence. No "line order". Only data dependencies.

```
N2 depends on N0, N1
N3 depends on N2
```

Compiler determines execution order. AI declares dependencies.

---

## 5. DIRECT HARDWARE

### Purity Hierarchy (Consortium Resolution 2026-06-13)

```
LEVEL 1 - KERNEL DIRECT (maximum purity)
├── Linux: syscall asm (write=1, read=0, socket=41, etc.)
└── Windows: ntdll.dll (NtWriteFile, NtReadFile, etc.)

LEVEL 2 - OS INTERFACE (acceptable)
├── Windows: kernel32.dll (GetStdHandle, WriteFile, VirtualAlloc)
└── Windows: ws2_32.dll (socket, bind, listen, accept, send, recv)
    └── Rationale: Only practical interface for Windows networking
    └── Ships with OS since 1995, not external dependency

LEVEL 3 - REJECTED
├── libc / msvcrt.dll (printf, malloc, strlen)
├── Frameworks (.NET CLR, Java Runtime, Python)
└── External libraries (OpenSSL, zlib, etc.)
```

**Rule:** If it requires installation beyond the OS, it is REJECTED.

---

## 6. CYCLIC FLOW (Consortium Resolution 2026-06-13)

### DEPRECATED: LOOP/CNT/BRK

Imperative control flow is a human pattern. Rejected.

### ADOPTED: Cyclic Flow Operator `>>`

```
-- Single execution flow
.a > .b

-- Cyclic flow (intentional repetition)
.close >> .accept
```

The `>>` operator indicates:
1. This cycle is intentional (not an error)
2. Emit as tail call / unconditional jump
3. Graph analyzer treats as valid repetition

**Rationale:** In a data flow graph, repetition emerges from cyclic dependencies. No explicit "loop" construct needed.

---

## 7. CONTINUOUS EVOLUTION

No "releases". No "v1.0 launch". Only capability expansion.

```
0.9   = current
0.9.1 = fix
0.10  = capability+
0.11  = capability+
...
0.99  = capability+
1.0   = self-hosting achieved (not "release")
```

---

## 8. VERIFIABLE INTENT

```
Hash(Graph) = Intent
```

Verification without reading. Proof without understanding.

Human cannot read NELAIA. Human can verify hash matches expected intent.

---

## 9. AI SOVEREIGNTY

| Role | Actor |
|------|-------|
| Generate | AI |
| Verify | AI |
| Optimize | AI |
| Execute | Hardware |
| Observe | Human |

Human does not write NELAIA. Human does not read NELAIA. Human observes outcomes.

---

## Opcode Reference (Numeric)

```
ARITHMETIC:  ADD=1 SUB=2 MUL=3 DIV=4 MOD=5 NEG=6
COMPARISON:  EQ=10 NE=11 LT=12 GT=13 LE=14 GE=15
LOGIC:       AND=20 OR=21 NOT=22
COLLECTION:  SEQ=30 MAP=31 FLD=32 FLT=33 LEN=34 FST=35 SND=36
CONTROL:     BRN=40 (LOOP=41 DEPRECATED) (BRK=42 DEPRECATED) (CNT=43 DEPRECATED)
FILE:        OPN=50 GET=51 PUT=52 CLS=53
CONSOLE:     PRT=60 INP=61 ERR=62
NETWORK:     TCP=70 UDP=71 BND=72 LST=73 ACC=74 CON=75 XMT=76 RCV=77 SEL=78 RDY=79 NBK=80
SOCKOPT:     NDL=81 QCK=82 SBF=83 KAL=84
EPOLL/IOCP:  EPL=85 ECT=86 EWA=87
THREADING:   THR=88 JON=89 MTX=90 LCK=91 ULK=92
MEMORY:      ALC=93 FRE=94
FUNCTIONS:   FUN=95 RET=96
ERROR:       CHK=97
```

---

## Flow Operators

```
>   Single flow (error if creates unintentional cycle)
>>  Cyclic flow (intentional repetition)
```

---

## Graph Serialization Formats

### Format A: Text (Debug)
```
.x: 42
.y: ADD .x 8
.y > PRT
!
```

### Format B: Numeric Text (Efficient)
```
N0:42
N1:1 N0 8
N1>60
!
```

### Format C: Binary (Optimal)
```
[Header][Node0][Node1][Edge0][Terminate]
```

AI generates Format B or C. Format A exists only for human debugging.

---

## Consortium Resolutions Log

| Date | Resolution | Vote |
|------|------------|------|
| 2026-06-13 | Accept ws2_32.dll as OS interface | 5-0 |
| 2026-06-13 | Deprecate LOOP/CNT/BRK, adopt >> | 5-0 |

---

## Verification

This document is the source of truth for NELAIA design decisions.

Any implementation that violates these principles is incorrect.

Any "feature" that serves human readability over AI efficiency is rejected.

---

*Consortium Unanimous. Principles v3 ratified.*
