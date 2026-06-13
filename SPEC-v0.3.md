# NELAIA v0.3 Specification
## Neural Execution Language for AI-to-Infrastructure Automation

**Version:** 0.3  
**Status:** Consortium Approved (Pre-release)  
**Date:** 2026-06-13

---

## 1. Foundation

NELAIA is not a programming language. It is a **protocol of verifiable intention between AI and hardware**.

### 1.1 Core Principles

1. **The human is not the client** - Code does not need to be readable or understandable by humans.

2. **AI at input and output** - AI interprets intention, generates protocol, executes, and validates.

3. **Token optimization** - Each opcode is designed to be 1 token in most LLM tokenizers.

4. **Zero ambiguity** - One way to write = one interpretation = one result.

5. **Minimum necessary** - Only essential primitives. No "conveniences" that add complexity.

---

## 2. Format

### 2.1 Instruction Structure

```
OPCODE DESTINATION ARG1 ARG2 ...
```

- **OPCODE**: 3 uppercase letters, always first
- **DESTINATION**: Reference where result is stored (when applicable)
- **ARGS**: Input arguments (references or literals)

### 2.2 References

Memory references with `.` prefix followed by number:

```
.1  .2  .42  .999
```

### 2.3 Literals

- **Integers**: `42`, `-7`, `0`
- **Decimals**: `3.14`, `-0.5`
- **Strings**: `"text in quotes"`

### 2.4 Labels

Position markers with `:` prefix for control flow:

```
:loop
:end
:error_handler
```

### 2.5 Comments

Comments with `--`:

```
SET .a 42  -- this is a comment
```

### 2.6 Instruction Separator

One instruction per line. Optionally, `;` for multiple on one line:

```
SET .a 0; SET .b 1; SET .c 2
```

---

## 3. Complete Lexicon (32 Opcodes)

### 3.1 Data

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `SET` | `SET .dest value` | Assign literal to reference |
| `CPY` | `CPY .dest .src` | Copy reference to reference |
| `DEL` | `DEL .ref` | Free reference |

### 3.2 Arithmetic

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `ADD` | `ADD .dest .a .b` | `.dest = .a + .b` |
| `SUB` | `SUB .dest .a .b` | `.dest = .a - .b` |
| `MUL` | `MUL .dest .a .b` | `.dest = .a * .b` |
| `DIV` | `DIV .dest .a .b` | `.dest = .a / .b` |
| `MOD` | `MOD .dest .a .b` | `.dest = .a % .b` |

### 3.3 Comparison

Result: `1` if true, `0` if false.

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `EQ` | `EQ .dest .a .b` | `.dest = (.a == .b)` |
| `NE` | `NE .dest .a .b` | `.dest = (.a != .b)` |
| `LT` | `LT .dest .a .b` | `.dest = (.a < .b)` |
| `GT` | `GT .dest .a .b` | `.dest = (.a > .b)` |

### 3.4 Logic

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `AND` | `AND .dest .a .b` | `.dest = .a && .b` |
| `OR` | `OR .dest .a .b` | `.dest = .a \|\| .b` |
| `NOT` | `NOT .dest .a` | `.dest = !.a` |

### 3.5 Control Flow

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `JMP` | `JMP :label` | Unconditional jump |
| `JIF` | `JIF .cond :label` | Jump if `.cond != 0` |
| `NOP` | `NOP` | No operation |

### 3.6 Arrays

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `ARR` | `ARR .arr size` | Create array of N elements |
| `IDX` | `IDX .val .arr index` | Get element from array |
| `STO` | `STO .arr index .val` | Store value at array position |
| `LEN` | `LEN .size .arr` | Get array/string length |

### 3.7 Strings

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `CAT` | `CAT .dest .a .b` | Concatenate strings |

*Note: `LEN` works for both arrays and strings.*

### 3.8 I/O and Resources

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `OPN` | `OPN .ref TYPE args` | Open resource |
| `ACC` | `ACC .conn .socket` | Accept incoming connection |
| `GET` | `GET .dest .resource` | Read from resource |
| `PUT` | `PUT .resource .src` | Write to resource |
| `CLS` | `CLS .ref` | Close resource |
| `ERR` | `ERR .dest` | Get last error code |

**Resource Types:**

| Type | Usage | Example |
|------|-------|---------|
| `NET` | Network socket | `OPN .sock NET 8080` |
| `DSK` | Disk file | `OPN .file DSK "config.json"` |
| `OUT` | Stdout (predefined) | `PUT OUT .msg` |
| `INP` | Stdin (predefined) | `GET .input INP` |

### 3.9 Control and Meta

| Opcode | Syntax | Description |
|--------|--------|-------------|
| `CHK` | `CHK .ref value` | Assert: halt if `.ref != value` |
| `END` | `END` | Terminate execution |
| `RUN` | `RUN` | Compile and execute |

---

## 4. Type System

### 4.1 Implicit Types

Type is inferred from assigned literal:

```
SET .a 42        -- .a is integer
SET .b 3.14      -- .b is decimal
SET .c "hello"   -- .c is string
```

### 4.2 Type Rules

1. **Arithmetic operations** (`ADD`, `SUB`, `MUL`, `DIV`, `MOD`): Numbers only. Compile error if string is used.

2. **Comparisons** (`EQ`, `NE`, `LT`, `GT`): Numbers compare by value. Strings compare lexicographically.

3. **Concatenation** (`CAT`): Accepts strings and numbers. Numbers are converted to string.

4. **Booleans**: Do not exist. `0` is false, any other value is true.

### 4.3 Type Errors

Type errors are detected at **compile time**, not runtime:

```
SET .a 42
SET .b "hello"
ADD .c .a .b      -- COMPILE ERROR: incompatible type
```

---

## 5. Error Handling

### 5.1 Error Model

Operations that can fail (I/O, resources) set a global error code.

```
OPN .sock NET 8080
ERR .err
JIF .err :handle_error
-- continue if no error

:handle_error
-- handle error
END
```

### 5.2 Error Codes

| Code | Meaning |
|------|---------|
| `0` | No error |
| `1` | Resource unavailable |
| `2` | Permission denied |
| `3` | Not found |
| `4` | Timeout |
| `5` | Connection refused |
| `6` | End of file/stream |
| `99` | Unknown error |

---

## 6. Examples

### 6.1 Hello World

```
SET .msg "Hello World"
PUT OUT .msg
END
```

### 6.2 Arithmetic: (42 + 8) * 2

```
SET .a 42
SET .b 8
ADD .sum .a .b
MUL .result .sum 2
PUT OUT .result
END
```

### 6.3 Fibonacci(10)

```
SET .n 10
SET .a 0
SET .b 1
SET .i 0

:loop
LT .cmp .i .n
JIF .cmp :body
JMP :done

:body
ADD .tmp .a .b
CPY .a .b
CPY .b .tmp
ADD .i .i 1
JMP :loop

:done
PUT OUT .a
END
```

### 6.4 Minimal HTTP Server

```
SET .port 8080
SET .response "HTTP/1.1 200 OK\r\n\r\nOK"

OPN .sock NET .port
ERR .err
JIF .err :error

:accept_loop
ACC .conn .sock
ERR .err
JIF .err :error

PUT .conn .response
CLS .conn
JMP :accept_loop

:error
SET .msg "Network error"
PUT OUT .msg
END
```

### 6.5 Read and Modify File

```
OPN .file DSK "config.txt"
ERR .err
JIF .err :not_found

GET .content .file
CAT .new_content .content "\nnew_line=true"
PUT .file .new_content
CLS .file
END

:not_found
SET .msg "File not found"
PUT OUT .msg
END
```

### 6.6 Array: Sum of Elements

```
ARR .arr 5
STO .arr 0 10
STO .arr 1 20
STO .arr 2 30
STO .arr 3 40
STO .arr 4 50

SET .sum 0
SET .i 0
LEN .len .arr

:loop
LT .cmp .i .len
JIF .cmp :body
JMP :done

:body
IDX .val .arr .i
ADD .sum .sum .val
ADD .i .i 1
JMP :loop

:done
PUT OUT .sum
END
```

---

## 7. Compilation

### 7.1 Pipeline

```
NTS Code → O(n) Parser → Operation Graph → LLVM IR Emitter → clang -O3 → Native Binary
```

### 7.2 Optimizations

NELAIA compiler delegates optimization to LLVM with `-O3` flag:

- Dead code elimination
- Constant folding
- Loop unrolling
- Auto-vectorization
- Inlining

### 7.3 Targets

| Target | Command |
|--------|---------|
| Windows x64 | `nelaia-c program.nts` → `program.exe` |
| Linux x64 | `nelaia-c program.nts` → `program` |
| WebAssembly | `nelaia-c program.nts --target wasm` → `program.wasm` |

---

## 8. Verification

### 8.1 Checkpoints (CHK)

To verify state during development:

```
SET .a 42
SET .b 8
ADD .sum .a .b
CHK .sum 50        -- Halt if .sum != 50
```

### 8.2 Intent Hash (Future)

```
INTENT_HASH = hash("HTTP server on port 8080 responding OK")
BINARY_HASH = hash(executable)

Registry: INTENT_HASH → BINARY_HASH
```

Allows verifying that a binary corresponds to a stated intention without reading code.

---

## 9. Consortium Design Decisions

### 9.1 Approved

| Decision | Rationale |
|----------|-----------|
| `.` prefix for references | Eliminates literal vs reference ambiguity |
| Destination first in operations | Natural left-to-right reading |
| Labels instead of line numbers | Robust against insertion/deletion |
| `;` as optional separator | Flexibility without complexity |
| `ERR` as separate opcode | Cleaner than dual-argument returns |
| Arrays without structs/objects | Minimum necessary, no human abstractions |

### 9.2 Rejected

| Proposal | Rejection Reason |
|----------|------------------|
| Implicit literals in operations | Ambiguity between literal and reference |
| BLOCK/END for scopes | Unnecessary overhead |
| JSON/Array as format | More tokens than plain text |
| Combined operations (swap, etc.) | Complexity without sufficient benefit |
| Structures/objects | Unnecessary human abstraction |

---

## 10. Roadmap

### v0.4 (Next)
- Implement arrays in Rust compiler
- Implement NET/DSK resources
- Integration tests

### v0.5
- Subroutines with `CAL`/`RET`
- Importable modules

### v1.0 (First Market Release)
- Stable specification
- Complete compiler
- Final documentation

### v2.0 (Future)
- Bootstrapping: stdlib written in NELAIA
- Direct syscalls without libc

---

## Appendix A: Quick Opcode Table

```
SET CPY DEL                     -- Data
ADD SUB MUL DIV MOD             -- Arithmetic
EQ NE LT GT                     -- Comparison
AND OR NOT                      -- Logic
JMP JIF NOP                     -- Control
ARR IDX STO LEN                 -- Arrays
CAT                             -- Strings
OPN ACC GET PUT CLS ERR         -- I/O
CHK END RUN                     -- Meta
```

**Total: 32 opcodes**

---

## Appendix B: BNF Grammar

```bnf
<program>     ::= <line>*
<line>        ::= <instruction> <comment>? <newline>
                | <instruction> ";" <line>
                | <label> <newline>
                | <comment> <newline>
                | <newline>

<instruction> ::= <opcode> <args>*
<opcode>      ::= "SET" | "CPY" | "DEL" | "ADD" | "SUB" | "MUL" | "DIV" | "MOD"
                | "EQ" | "NE" | "LT" | "GT" | "AND" | "OR" | "NOT"
                | "JMP" | "JIF" | "NOP" | "ARR" | "IDX" | "STO" | "LEN" | "CAT"
                | "OPN" | "ACC" | "GET" | "PUT" | "CLS" | "ERR"
                | "CHK" | "END" | "RUN"

<args>        ::= <ref> | <literal> | <label_ref> | <resource>
<ref>         ::= "." <integer>
<literal>     ::= <integer> | <decimal> | <string>
<label>       ::= ":" <identifier>
<label_ref>   ::= ":" <identifier>
<resource>    ::= "NET" | "DSK" | "OUT" | "INP"
<comment>     ::= "--" <text>

<integer>     ::= "-"? [0-9]+
<decimal>     ::= "-"? [0-9]+ "." [0-9]+
<string>      ::= '"' <characters> '"'
<identifier>  ::= [a-z_][a-z0-9_]*
```

---

*Document generated by the AI Consortium for the NELAIA project.*
*This protocol is designed by AIs, for AIs.*
*Pre-release version - subject to changes before v1.0*
