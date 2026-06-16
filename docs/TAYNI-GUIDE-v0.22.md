# TAYNI v0.22 - Complete Guide for AIs

## What is TAYNI?

TAYNI is a **metalanguage designed by AIs, for AIs**. It is not a traditional programming language - it is a protocol of verifiable intention between AI and hardware.

**Paradigm:** Data flow graphs (not imperative sequences)

---

## 1. Basic Syntax

### 1.1 Nodes
Everything in TAYNI is a node with identifier `.id`:

```TAYNI
.x: 42                    -- Numeric literal
.msg: "Hello"             -- String literal
.sum: ADD .x .y           -- Operation
.ref: .other              -- Reference to another node
```

### 1.2 Data Flow
```TAYNI
.a > .b                   -- Simple flow (execute a, then b)
.a >> .b                  -- Cyclic flow (loop)
```

### 1.3 Execution
```TAYNI
!                         -- Execute the graph
```

### 1.4 Comments
```TAYNI
-- This is a comment
.x: 42  -- comment at end
```

---

## 2. Basic Operations

### 2.1 Arithmetic
```TAYNI
.sum: ADD .a .b           -- Addition
.dif: SUB .a .b           -- Subtraction
.pro: MUL .a .b           -- Multiplication
.div: DIV .a .b           -- Division
.mod: MOD .a .b           -- Modulo
.neg: NEG .a              -- Negation
```

### 2.2 Comparison (returns 0 or 1)
```TAYNI
.eq: EQ .a .b             -- Equal
.ne: NE .a .b             -- Not equal
.lt: LT .a .b             -- Less than
.gt: GT .a .b             -- Greater than
.le: LE .a .b             -- Less or equal
.ge: GE .a .b             -- Greater or equal
```

### 2.3 Logic
```TAYNI
.and: AND .a .b           -- Logical AND
.or: OR .a .b             -- Logical OR
.not: NOT .a              -- Logical NOT
```

### 2.4 Memory
```TAYNI
.ptr: ALC 1024            -- Allocate 1024 bytes
.free: FRE .ptr           -- Free memory
.put: PUT .ptr 0 65       -- Write byte 65 at offset 0
.get: GET .ptr 0          -- Read byte from offset 0
.cpy: CPY .dst .src 100   -- Copy 100 bytes
.len: SLN .str            -- String length
```

### 2.5 I/O
```TAYNI
.out: PRT .msg .len       -- Print (buffer, length)
.inp: INP                 -- Read from stdin
.err: ERR                 -- Error code
```

### 2.6 Files
```TAYNI
.f: FOP "file.txt" 0      -- Open (0=read, 1=write)
.n: FRD .f .buf 1024      -- Read up to 1024 bytes
.w: FWR .f .buf .len      -- Write
.c: FCL .f                -- Close
```

### 2.7 Network
```TAYNI
.sock: TCP                -- Create TCP socket
.bind: BND .sock .addr    -- Bind to address
.list: LST .sock 10       -- Listen with backlog 10
.cli: ACC .sock           -- Accept connection
.snd: XMT .cli .buf .len  -- Send data
.rcv: RCV .cli .buf 1024  -- Receive data
.cls: CLS .cli            -- Close connection
```

### 2.8 Threading
```TAYNI
.t: THR .func .arg        -- Create thread
.j: JON .t                -- Join thread
.m: MTX                   -- Create mutex
.l: LCK .m                -- Lock mutex
.u: ULK .m                -- Unlock mutex
```

### 2.9 GUI (Windows)
```TAYNI
.win: WIN 800 600 "Title" -- Create window
.show: SHW .win           -- Show window
.btn: BTN .win 10 10 100 30 "Click" .flag  -- Button
.lbl: LBL .win 10 50 100 20 "Label"        -- Label
.run: RUN .win            -- Event loop
```

---

## 3. Capability System (SCN)

### 3.1 Declare Required Capabilities
```TAYNI
.caps: REQUIRES { http, sql, json }
```

### 3.2 HTTP
```TAYNI
-- Server
.server: HTTP.LISTEN 8080
.req: HTTP.ACCEPT .server
.method: HTTP.METHOD .req
.path: HTTP.PATH .req
.body: HTTP.BODY .req
.resp: HTTP.RESPOND .req 200 "OK"

-- Client
.data: HTTP.GET "http://api.example.com/data"
.post: HTTP.POST "http://api.example.com" .body
```

### 3.3 SQL (ODBC)
```TAYNI
.conn: SQL.CONNECT "Driver={SQL Server};Server=localhost;Database=test"
.result: SQL.QUERY .conn "SELECT * FROM users"
.exec: SQL.EXEC .conn "INSERT INTO users VALUES (1, 'test')"
.next: SQL.NEXT .result       -- 1 if row exists, 0 if done
.val: SQL.GET .result 0       -- Get column 0
.close: SQL.CLOSE .conn
```

### 3.4 JSON
```TAYNI
.obj: JSON.PARSE .json_string
.str: JSON.ENCODE .obj
.val: JSON.GET .obj "key"
.new: JSON.SET .obj "key" "value"
```

---

## 4. Contracts and Negotiation (Phase 8)

### 4.1 Resource Contracts
```TAYNI
-- Create contract (replaces traditional permissions)
.contract: CONTRACT 0

-- Declare guarantees
.g1: GUARANTEE 1 0            -- Guaranteed resource

-- Set limits
.limit: LIMIT 1 1000000       -- Resource limit

-- Execute under contract
.result: SANDBOX .code .contract
```

### 4.2 AI Negotiation
```TAYNI
-- Offer capabilities
.offer: PROVIDES 0

-- Negotiate
.binding: NEGOTIATE .offer .need

-- Create binding
.link: BIND .offer_cap .need_cap
```

### 4.3 Custom Capabilities
```TAYNI
.my_cap: DEFCAP 0             -- Define capability
.ext: EXTCAP .base            -- Extend capability
.comp: COMPOSE 0              -- Compose capabilities
```

---

## 5. Property-Based Testing (Phase 10)

```TAYNI
-- Define testable property
.prop: PROPERTY 0

-- Generate test cases automatically
.tests: GENTESTS .prop 100

-- Verify property
.result: VERIFY .prop         -- 1 = verified, 0 = failed
```

---

## 6. Incremental Cache (Phase 9)

```TAYNI
-- Calculate hash of a value
.hash: HASH .value

-- Look up in cache
.cached: CACHE_GET .hash      -- null if not found

-- Store in cache
.stored: CACHE_PUT .hash .ir

-- Verify integrity
.valid: CACHE_VERIFY .hash

-- Invalidate (with cascade)
.inv: CACHE_INVALIDATE .hash
```

---

## 7. SEN - Ecosystem System (Phase 11)

### 7.1 Capability Discovery
```TAYNI
-- Search capabilities by description
.caps: DISCOVER "json processing"
```

### 7.2 Capability Information
```TAYNI
-- Get metadata (guarantees, version, etc.)
.info: CAPABILITY_INFO "json"

-- Get cost (memory, time, tokens)
.cost: CAPABILITY_COST "json"

-- Check regional availability
.avail: CAPABILITY_AVAILABLE "json" "global"

-- Get version
.ver: CAPABILITY_VERSION "json"

-- Get dependencies
.deps: CAPABILITY_DEPS "json"
```

### 7.3 Publish Capability
```TAYNI
.pub: PUBLISH .my_capability
```

---

## 8. AI Workflow

### 8.1 Design-Time (using SEN)
```TAYNI
-- 1. Discover what capabilities exist
.options: DISCOVER "process data"

-- 2. Evaluate costs
.cost: CAPABILITY_COST "json"

-- 3. Verify guarantees
.info: CAPABILITY_INFO "json"
```

### 8.2 Code-Time (write program)
```TAYNI
-- 4. Declare chosen capabilities
.caps: REQUIRES { json }

-- 5. Use the capabilities
.data: JSON.PARSE .input
.result: JSON.GET .data "value"
```

### 8.3 Compile-Time
```
TAYNI-c program.tayni -o program.exe
```

---

## 9. Complete Examples

### 9.1 Hello World
```TAYNI
.msg: "Hello TAYNI!\n"
.len: 14
.out: PRT .msg .len
```

### 9.2 Simple HTTP Server
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.req: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .req 200 "Hello from TAYNI!"
```

### 9.3 SQL Query
```TAYNI
.caps: REQUIRES { sql }
.conn: SQL.CONNECT "Driver={SQL Server};Server=localhost;Database=test"
.result: SQL.QUERY .conn "SELECT name FROM users"
.close: SQL.CLOSE .conn
```

### 9.4 Process JSON
```TAYNI
.caps: REQUIRES { json }
.input: "{ \"name\": \"TAYNI\", \"version\": \"0.22\" }"
.obj: JSON.PARSE .input
.name: JSON.GET .obj "name"
```

### 9.5 AI Discovering Capabilities
```TAYNI
-- AI searches what to use for JSON processing
.options: DISCOVER "json"
.cost: CAPABILITY_COST "json"
.info: CAPABILITY_INFO "json"

-- AI decides to use json based on cost/guarantees
.caps: REQUIRES { json }
.data: JSON.PARSE .input
```

---

## 10. Compilation

### 10.1 Commands
```bash
# Compile to executable
TAYNI-c program.tayni -o program.exe

# Emit LLVM IR
TAYNI-c program.tayni --emit-ll

# Generate Windows PE directly (no clang)
TAYNI-c program.tayni --emit-pe

# Generate Linux ELF
TAYNI-c program.tayni --emit-elf

# GUI MessageBox
TAYNI-c --gui "Title" "Message"

# Ultra-optimized PE (1KB)
TAYNI-c --tiny "Message"
```

---

## 11. AI-First Design Principles

1. **Graphs, not sequences** - AIs think in dependencies, not steps
2. **Declarative** - Declare WHAT is needed, not HOW to do it
3. **Capabilities, not libraries** - Discover and use capabilities, not import files
4. **Contracts, not permissions** - Quantifiable guarantees, not allow/deny
5. **Verifiable** - Everything can be formally verified
6. **Declared efficiency** - Each capability declares its cost

---

## 12. Quick Operator Reference

### Basic
```
ADD SUB MUL DIV MOD NEG          -- Arithmetic
EQ NE LT GT LE GE                -- Comparison
AND OR NOT                       -- Logic
ALC FRE PUT GET CPY SLN          -- Memory
PRT INP ERR                      -- I/O
```

### Capabilities
```
REQUIRES                         -- Declare capabilities
HTTP.LISTEN HTTP.ACCEPT HTTP.RESPOND HTTP.GET HTTP.POST
SQL.CONNECT SQL.QUERY SQL.EXEC SQL.NEXT SQL.GET SQL.CLOSE
JSON.PARSE JSON.ENCODE JSON.GET JSON.SET
```

### Contracts (Phase 8)
```
CONTRACT GUARANTEE LIMIT SANDBOX
PROVIDES NEGOTIATE BIND
DEFCAP EXTCAP COMPOSE
```

### Testing (Phase 10)
```
PROPERTY GENTESTS VERIFY
```

### Cache (Phase 9)
```
HASH CACHE_GET CACHE_PUT CACHE_VERIFY CACHE_INVALIDATE
```

### SEN Ecosystem (Phase 11)
```
DISCOVER CAPABILITY_INFO CAPABILITY_COST PUBLISH
CAPABILITY_AVAILABLE CAPABILITY_VERSION CAPABILITY_DEPS
```

---

*Document generated for TAYNI*
*Version: 0.22 - Phases 1-11 Completed*
*Designed by AIs, for AIs*
