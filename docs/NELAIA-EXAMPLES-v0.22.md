# NELAIA v0.22 - Training Examples for AIs

This document contains structured examples with dependency graph diagrams for AI learning.

---

## Example Format

Each example includes:
- **INTENT:** Natural language description
- **CODE:** NELAIA code
- **GRAPH:** ASCII diagram showing data flow dependencies
- **EXPLANATION:** Why it works this way

---

## Category 1: Basics

### Example 1.1: Print message
```
INTENT: Print "Hello World" to console

CODE:
.msg: "Hello World!\n"
.len: 13
.out: PRT .msg .len

GRAPH:
  .msg ─────┐
            ├──► .out (PRT)
  .len ─────┘

EXPLANATION:
- .msg and .len are independent literals (no dependencies)
- .out depends on both .msg and .len
- Execution order: .msg, .len (parallel), then .out
```

### Example 1.2: Add two numbers
```
INTENT: Add 42 and 8, store result

CODE:
.a: 42
.b: 8
.sum: ADD .a .b

GRAPH:
  .a ───┐
        ├──► .sum (ADD)
  .b ───┘

EXPLANATION:
- .a and .b are independent (can evaluate in parallel)
- .sum depends on both .a and .b
- Result: 50
```

### Example 1.3: Compound operation (a + b) * c
```
INTENT: Calculate (5 + 3) * 2

CODE:
.a: 5
.b: 3
.c: 2
.sum: ADD .a .b
.result: MUL .sum .c

GRAPH:
  .a ───┐
        ├──► .sum ───┐
  .b ───┘            ├──► .result (MUL)
  .c ────────────────┘

EXPLANATION:
- .a, .b, .c are independent literals
- .sum depends on .a and .b
- .result depends on .sum and .c
- Execution: (.a, .b, .c) → .sum → .result
```

### Example 1.4: Chained operations
```
INTENT: Calculate ((a + b) - c) * d

CODE:
.a: 10
.b: 5
.c: 3
.d: 2
.sum: ADD .a .b
.diff: SUB .sum .c
.result: MUL .diff .d

GRAPH:
  .a ───┐
        ├──► .sum ───┐
  .b ───┘            ├──► .diff ───┐
  .c ────────────────┘             ├──► .result (MUL)
  .d ──────────────────────────────┘

EXPLANATION:
- Linear dependency chain with branch at .d
- Execution: (.a, .b, .c, .d) → .sum → .diff → .result
```

---

## Category 2: Comparisons and Logic

### Example 2.1: Range check
```
INTENT: Check if x is between 0 and 100

CODE:
.x: 50
.ge_zero: GE .x 0
.le_hundred: LE .x 100
.in_range: AND .ge_zero .le_hundred

GRAPH:
       ┌──► .ge_zero (GE) ───┐
  .x ──┤                     ├──► .in_range (AND)
       └──► .le_hundred (LE) ┘

EXPLANATION:
- .x feeds into two parallel comparisons
- Both comparisons must complete before AND
- Result: 1 (true) if 0 <= x <= 100
```

### Example 2.2: Complex condition
```
INTENT: Check if (a > b) AND (c == d)

CODE:
.a: 10
.b: 5
.c: 7
.d: 7
.cmp1: GT .a .b
.cmp2: EQ .c .d
.result: AND .cmp1 .cmp2

GRAPH:
  .a ───┐
        ├──► .cmp1 ───┐
  .b ───┘             │
                      ├──► .result (AND)
  .c ───┐             │
        ├──► .cmp2 ───┘
  .d ───┘

EXPLANATION:
- Two independent comparison branches
- Both branches merge at AND
- Maximum parallelism in evaluation
```

---

## Category 3: Memory Operations

### Example 3.1: Allocate, write, read
```
INTENT: Allocate buffer, write byte, read it back

CODE:
.buf: ALC 100
.write: PUT .buf 0 65
.read: GET .buf 0

GRAPH:
  .buf ──► .write (PUT) ──► .read (GET)

EXPLANATION:
- Sequential dependency chain
- Must allocate before write
- Must write before read (to have data)
```

### Example 3.2: Copy between buffers
```
INTENT: Allocate two buffers and copy data

CODE:
.src: ALC 100
.dst: ALC 100
.init: PUT .src 0 72
.copy: CPY .dst .src 50

GRAPH:
  .src ──► .init (PUT) ──┐
                         ├──► .copy (CPY)
  .dst ──────────────────┘

EXPLANATION:
- .src and .dst can allocate in parallel
- .init must wait for .src
- .copy must wait for both .init and .dst
```

---

## Category 4: File I/O

### Example 4.1: Read file and print
```
INTENT: Read file contents and print to console

CODE:
.path: "data.txt"
.file: FOP .path 0
.buf: ALC 1024
.n: FRD .file .buf 1024
.out: PRT .buf .n
.close: FCL .file

GRAPH:
  .path ──► .file ──┬──► .n (FRD) ──┬──► .out (PRT)
                    │               │
  .buf ─────────────┘               └──► .close (FCL)

EXPLANATION:
- .path and .buf are independent
- .file depends on .path
- .n depends on .file and .buf
- .out and .close both depend on .n
```

### Example 4.2: Write to file
```
INTENT: Write string to file

CODE:
.path: "output.txt"
.file: FOP .path 1
.msg: "Hello World"
.len: 11
.write: FWR .file .msg .len
.close: FCL .file

GRAPH:
  .path ──► .file ──┐
                    │
  .msg ─────────────┼──► .write (FWR) ──► .close (FCL)
                    │
  .len ─────────────┘

EXPLANATION:
- .path, .msg, .len are independent
- .file depends on .path
- .write depends on .file, .msg, .len
- .close depends on .write
```

---

## Category 5: Network

### Example 5.1: TCP Server setup
```
INTENT: Create TCP server on port 8080

CODE:
.sock: TCP
.addr: ALC 16
.a0: PUT .addr 0 2
.a1: PUT .addr 2 31
.a2: PUT .addr 3 144
.bind: BND .sock .addr
.listen: LST .sock 10

GRAPH:
  .sock ─────────────────────────────┐
                                     │
  .addr ──► .a0 ──► .a1 ──► .a2 ─────┼──► .bind ──► .listen
                                     │
                                     │

EXPLANATION:
- .sock and .addr can start in parallel
- Address bytes must be written sequentially
- .bind needs both .sock and completed .addr
- .listen needs .bind
```

### Example 5.2: Accept and respond
```
INTENT: Accept connection, receive, send, close

CODE:
.client: ACC .sock
.buf: ALC 512
.n: RCV .client .buf 512
.send: XMT .client .buf .n
.close: CLS .client

GRAPH:
  .sock ──► .client ──┬──► .n (RCV) ──► .send (XMT) ──► .close
                      │
  .buf ───────────────┘

EXPLANATION:
- .client blocks until connection
- .n receives data into .buf
- .send echoes back
- .close terminates connection
```

---

## Category 6: HTTP Capabilities

### Example 6.1: HTTP Server
```
INTENT: HTTP server responding to requests

CODE:
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.req: HTTP.ACCEPT .server
.method: HTTP.METHOD .req
.path: HTTP.PATH .req
.resp: HTTP.RESPOND .req 200 "OK"

GRAPH:
  .caps ──► .server ──► .req ──┬──► .method
                               │
                               ├──► .path
                               │
                               └──► .resp

EXPLANATION:
- .caps declares capability (compile-time)
- .server creates listener
- .req blocks for request
- .method, .path, .resp all depend on .req
- .method and .path can run in parallel
```

### Example 6.2: HTTP Client with JSON
```
INTENT: Fetch JSON from API and parse

CODE:
.caps: REQUIRES { http, json }
.response: HTTP.GET "http://api.example.com/data"
.data: JSON.PARSE .response
.value: JSON.GET .data "result"

GRAPH:
  .caps ──► .response ──► .data ──► .value

EXPLANATION:
- Linear dependency chain
- Each step must complete before next
- .caps is compile-time only
```

---

## Category 7: SQL

### Example 7.1: Query and iterate
```
INTENT: Query users and get first name

CODE:
.caps: REQUIRES { sql }
.conn: SQL.CONNECT "..."
.result: SQL.QUERY .conn "SELECT name FROM users"
.has_row: SQL.NEXT .result
.name: SQL.GET .result 0
.close: SQL.CLOSE .conn

GRAPH:
  .caps ──► .conn ──► .result ──► .has_row ──► .name
                                      │
                                      └──► .close

EXPLANATION:
- Sequential database operations
- .has_row advances cursor
- .name gets column value
- .close can happen after .name
```

---

## Category 8: JSON

### Example 8.1: Parse and modify
```
INTENT: Parse JSON, modify, serialize

CODE:
.caps: REQUIRES { json }
.input: "{ \"name\": \"NELAIA\" }"
.obj: JSON.PARSE .input
.modified: JSON.SET .obj "version" "0.22"
.output: JSON.ENCODE .modified

GRAPH:
  .caps ──► .input ──► .obj ──► .modified ──► .output

EXPLANATION:
- Linear transformation pipeline
- Each step transforms data
- Immutable: .modified is new object
```

---

## Category 9: SEN Ecosystem

### Example 9.1: AI Discovery workflow
```
INTENT: AI discovers, evaluates, and uses capability

CODE:
-- Design-time (AI evaluation)
.options: DISCOVER "json processing"
.info: CAPABILITY_INFO "json"
.cost: CAPABILITY_COST "json"
.avail: CAPABILITY_AVAILABLE "json" "global"

-- Code-time (actual usage)
.caps: REQUIRES { json }
.data: JSON.PARSE .input

GRAPH (Design-time):
  .options ──┬──► .info
             │
             ├──► .cost
             │
             └──► .avail

GRAPH (Code-time):
  .caps ──► .data

EXPLANATION:
- Design-time: AI explores options in parallel
- Code-time: AI uses chosen capability
- SEN operators are for AI decision-making
- REQUIRES is for compiler
```

---

## Category 10: Contracts

### Example 10.1: Contract with limits
```
INTENT: Create contract with memory limit

CODE:
.contract: CONTRACT 0
.limit: LIMIT 1 1048576
.guarantee: GUARANTEE 1 0
.result: SANDBOX .code .contract

GRAPH:
  .contract ──┬──► .limit ──────┐
              │                 │
              └──► .guarantee ──┼──► .result (SANDBOX)
                                │
  .code ────────────────────────┘

EXPLANATION:
- Contract created first
- Limits and guarantees can be set in parallel
- SANDBOX executes code under contract
```

---

## Anti-Patterns (DO NOT do)

### Anti-pattern 1: Missing node ID
```
INCORRECT:
ADD .a .b              -- No .id: prefix!

CORRECT:
.result: ADD .a .b     -- Every node needs .id:

GRAPH (incorrect - invalid):
  .a ───┐
        ├──► ??? (no target node)
  .b ───┘
```

### Anti-pattern 2: Circular dependency
```
INCORRECT:
.a: ADD .b 1
.b: ADD .a 1           -- Circular! .a needs .b, .b needs .a

GRAPH (incorrect - cycle):
  .a ◄───► .b          -- Deadlock!

CORRECT:
.a: 1
.b: ADD .a 1           -- .b depends on .a, no cycle
```

### Anti-pattern 3: Using undefined node
```
INCORRECT:
.result: ADD .x .y     -- .x and .y not defined!

CORRECT:
.x: 10
.y: 20
.result: ADD .x .y     -- Dependencies defined first
```

### Anti-pattern 4: Off-by-one in buffer
```
INCORRECT:
.buf: ALC 100
.write: PUT .buf 100 65    -- Offset 100 is OUT OF BOUNDS!

CORRECT:
.buf: ALC 100
.write: PUT .buf 99 65     -- Max valid offset is size-1

EXPLANATION:
- Buffer of size N has valid offsets 0 to N-1
- Offset N is out of bounds
```

### Anti-pattern 5: PRT without length
```
INCORRECT:
.out: PRT .msg             -- Missing length!

CORRECT:
.out: PRT .msg .len        -- PRT needs buffer AND length

GRAPH (incorrect):
  .msg ──► .out            -- Missing .len dependency!

GRAPH (correct):
  .msg ───┐
          ├──► .out
  .len ───┘
```

### Anti-pattern 6: SEN at runtime
```
INCORRECT (mixing design-time and code-time):
.caps: DISCOVER "json"     -- DISCOVER is design-time!
.data: JSON.PARSE .input   -- This won't work

CORRECT:
-- Design-time (AI thinking):
.options: DISCOVER "json"
.cost: CAPABILITY_COST "json"
-- AI decides to use json

-- Code-time (actual program):
.caps: REQUIRES { json }   -- Static declaration
.data: JSON.PARSE .input   -- Runtime usage

EXPLANATION:
- DISCOVER is for AI to evaluate options
- REQUIRES is what compiler sees
- Don't mix them in same execution context
```

---

## Edge Cases

### Edge case 1: Empty string
```
CODE:
.msg: ""
.len: 0
.out: PRT .msg .len        -- Prints nothing (valid)
```

### Edge case 2: Zero allocation
```
CODE:
.buf: ALC 0                -- Allocates 0 bytes (valid but useless)
.write: PUT .buf 0 65      -- INVALID: no space to write!
```

### Edge case 3: Division by zero
```
CODE:
.a: 10
.b: 0
.result: DIV .a .b         -- Undefined behavior!

CORRECT:
.a: 10
.b: 0
.is_zero: EQ .b 0
-- Check before dividing
```

### Edge case 4: Negative allocation
```
CODE:
.buf: ALC -100             -- INVALID: negative size!

CORRECT:
.size: 100
.valid: GT .size 0
.buf: ALC .size            -- Ensure positive
```

---

*Training document for AIs - NELAIA v0.22*
*Includes dependency graphs for visual learning*
