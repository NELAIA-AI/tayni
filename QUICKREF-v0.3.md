# NELAIA v0.3 Quick Reference Card

## Opcodes (32 total)

```
┌─────────────────────────────────────────────────────────────────┐
│ DATA                                                            │
├─────────────────────────────────────────────────────────────────┤
│ SET .dest value        Assign literal                           │
│ CPY .dest .src         Copy reference                           │
│ DEL .ref               Free reference                           │
├─────────────────────────────────────────────────────────────────┤
│ ARITHMETIC                                                      │
├─────────────────────────────────────────────────────────────────┤
│ ADD .dest .a .b        Addition                                 │
│ SUB .dest .a .b        Subtraction                              │
│ MUL .dest .a .b        Multiplication                           │
│ DIV .dest .a .b        Division                                 │
│ MOD .dest .a .b        Modulo                                   │
├─────────────────────────────────────────────────────────────────┤
│ COMPARISON (result: 0 or 1)                                     │
├─────────────────────────────────────────────────────────────────┤
│ EQ .dest .a .b         Equal                                    │
│ NE .dest .a .b         Not equal                                │
│ LT .dest .a .b         Less than                                │
│ GT .dest .a .b         Greater than                             │
├─────────────────────────────────────────────────────────────────┤
│ LOGIC                                                           │
├─────────────────────────────────────────────────────────────────┤
│ AND .dest .a .b        Logical AND                              │
│ OR .dest .a .b         Logical OR                               │
│ NOT .dest .a           Logical NOT                              │
├─────────────────────────────────────────────────────────────────┤
│ CONTROL                                                         │
├─────────────────────────────────────────────────────────────────┤
│ JMP :label             Unconditional jump                       │
│ JIF .cond :label       Jump if cond != 0                        │
│ NOP                    No operation                             │
├─────────────────────────────────────────────────────────────────┤
│ ARRAYS                                                          │
├─────────────────────────────────────────────────────────────────┤
│ ARR .arr size          Create array                             │
│ IDX .val .arr index    Get element                              │
│ STO .arr index .val    Store element                            │
│ LEN .size .arr         Get length (also for strings)            │
├─────────────────────────────────────────────────────────────────┤
│ STRINGS                                                         │
├─────────────────────────────────────────────────────────────────┤
│ CAT .dest .a .b        Concatenate                              │
├─────────────────────────────────────────────────────────────────┤
│ I/O AND RESOURCES                                               │
├─────────────────────────────────────────────────────────────────┤
│ OPN .ref TYPE args     Open resource                            │
│ ACC .conn .socket      Accept connection                        │
│ GET .dest .resource    Read                                     │
│ PUT .resource .src     Write                                    │
│ CLS .ref               Close                                    │
│ ERR .dest              Last error                               │
├─────────────────────────────────────────────────────────────────┤
│ META                                                            │
├─────────────────────────────────────────────────────────────────┤
│ CHK .ref value         Assert                                   │
│ END                    Terminate                                │
│ RUN                    Compile/execute                          │
└─────────────────────────────────────────────────────────────────┘
```

## Syntax

```
.1 .2 .99        References (. prefix)
:loop :end       Labels (: prefix)
42 3.14          Numeric literals
"text"           String literals
-- comment       Comments
;                Instruction separator (optional)
```

## Resources

```
OUT              Stdout (predefined)
INP              Stdin (predefined)
NET port         Network socket
DSK "path"       Disk file
```

## Error Codes

```
0   No error
1   Resource unavailable
2   Permission denied
3   Not found
4   Timeout
5   Connection refused
6   End of file
99  Unknown error
```

## Minimal Examples

```nts
-- Hello World
SET .1 "Hello World"
PUT OUT .1
END

-- Addition
ADD .1 42 8
PUT OUT .1
END

-- Loop
SET .i 0
:loop
LT .cmp .i 10
JIF .cmp :body
JMP :end
:body
PUT OUT .i
ADD .i .i 1
JMP :loop
:end
END
```

## Pipeline

```
file.nts → nelaia-c → LLVM IR → clang -O3 → native binary
```

---
*NELAIA v0.3 - AI Consortium Approved (Pre-release)*
