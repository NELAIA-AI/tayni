# TAYNI v0.22 - Semantic Rules and Type System

This document defines the semantic constraints and type rules for TAYNI.

---

## 1. Type System

### 1.1 Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `int` | 64-bit signed integer | `42`, `-10`, `0` |
| `ptr` | Memory pointer (64-bit address) | Result of `ALC` |
| `str` | String literal (ptr + implicit length) | `"Hello"` |
| `bool` | Boolean (0 = false, non-zero = true) | `0`, `1` |
| `handle` | OS resource handle | Result of `FOP`, `TCP` |
| `void` | No return value | Result of `PRT`, `FCL` |

### 1.2 Composite Types

| Type | Description | Examples |
|------|-------------|----------|
| `json_obj` | JSON object | Result of `JSON.PARSE` |
| `sql_conn` | SQL connection | Result of `SQL.CONNECT` |
| `sql_result` | SQL result set | Result of `SQL.QUERY` |
| `http_server` | HTTP server handle | Result of `HTTP.LISTEN` |
| `http_request` | HTTP request object | Result of `HTTP.ACCEPT` |
| `contract` | Resource contract | Result of `CONTRACT` |
| `capability` | Capability reference | Result of `DEFCAP` |

---

## 2. Operator Type Signatures

### 2.1 Arithmetic Operators

```
ADD : (int, int) → int
SUB : (int, int) → int
MUL : (int, int) → int
DIV : (int, int) → int      -- Precondition: b ≠ 0
MOD : (int, int) → int      -- Precondition: b ≠ 0
NEG : (int) → int
```

**Type Errors:**
```
ADD .str .int    -- ERROR: Cannot add string to integer
MUL .ptr .int    -- ERROR: Cannot multiply pointer
DIV .a 0         -- ERROR: Division by zero
```

### 2.2 Comparison Operators

```
EQ : (T, T) → bool          -- T must be same type
NE : (T, T) → bool
LT : (int, int) → bool
GT : (int, int) → bool
LE : (int, int) → bool
GE : (int, int) → bool
```

**Type Errors:**
```
EQ .int .str     -- ERROR: Cannot compare int with string
LT .ptr .ptr     -- ERROR: Pointer comparison not allowed
```

### 2.3 Logic Operators

```
AND : (bool, bool) → bool
OR  : (bool, bool) → bool
NOT : (bool) → bool
```

**Note:** Any non-zero value is truthy.

### 2.4 Memory Operators

```
ALC : (int) → ptr           -- Precondition: size > 0
FRE : (ptr) → void
PUT : (ptr, int, int) → void -- (ptr, offset, byte)
GET : (ptr, int) → int      -- (ptr, offset) → byte value
CPY : (ptr, ptr, int) → void -- (dst, src, len)
SLN : (str) → int           -- String length
CMP : (ptr, ptr, int) → int -- Compare bytes, 0 if equal
```

**Preconditions:**
```
ALC n where n > 0           -- Cannot allocate 0 or negative
PUT p o b where 0 ≤ o < size(p) and 0 ≤ b ≤ 255
GET p o where 0 ≤ o < size(p)
```

**Type Errors:**
```
ALC -100         -- ERROR: Negative allocation size
PUT .int 0 65    -- ERROR: First arg must be ptr
GET .str 0       -- OK: str is implicitly ptr
```

### 2.5 I/O Operators

```
PRT : (ptr, int) → void     -- (buffer, length)
INP : () → str
ERR : () → int              -- Last error code
```

**Preconditions:**
```
PRT buf len where len ≥ 0 and len ≤ size(buf)
```

### 2.6 File Operators

```
FOP : (str, int) → handle   -- (path, mode) mode: 0=read, 1=write
FRD : (handle, ptr, int) → int -- Returns bytes read
FWR : (handle, ptr, int) → int -- Returns bytes written
FCL : (handle) → void
```

**Type Errors:**
```
FOP .int 0       -- ERROR: Path must be string
FRD .ptr .buf 10 -- ERROR: First arg must be handle
```

### 2.7 Network Operators

```
TCP : () → handle
BND : (handle, ptr) → int   -- Returns 0 on success
LST : (handle, int) → int   -- (socket, backlog)
ACC : (handle) → handle     -- Returns client socket
XMT : (handle, ptr, int) → int -- Returns bytes sent
RCV : (handle, ptr, int) → int -- Returns bytes received
CLS : (handle) → void
```

### 2.8 Capability Operators

```
REQUIRES : (capability_set) → void  -- Compile-time declaration

HTTP.LISTEN  : (int) → http_server
HTTP.ACCEPT  : (http_server) → http_request
HTTP.METHOD  : (http_request) → str
HTTP.PATH    : (http_request) → str
HTTP.BODY    : (http_request) → str
HTTP.RESPOND : (http_request, int, str) → void
HTTP.GET     : (str) → str
HTTP.POST    : (str, str) → str

SQL.CONNECT : (str) → sql_conn
SQL.QUERY   : (sql_conn, str) → sql_result
SQL.EXEC    : (sql_conn, str) → int
SQL.NEXT    : (sql_result) → bool
SQL.GET     : (sql_result, int) → str
SQL.CLOSE   : (sql_conn) → void

JSON.PARSE  : (str) → json_obj
JSON.ENCODE : (json_obj) → str
JSON.GET    : (json_obj, str) → str
JSON.SET    : (json_obj, str, str) → json_obj
```

### 2.9 Contract Operators

```
CONTRACT  : (int) → contract
GUARANTEE : (int, int) → void
LIMIT     : (int, int) → void
SANDBOX   : (ptr, contract) → int
PROVIDES  : (int) → capability
NEGOTIATE : (capability, capability) → bool
BIND      : (capability, capability) → void
DEFCAP    : (int) → capability
EXTCAP    : (capability) → capability
COMPOSE   : (int) → capability
```

### 2.10 Testing Operators

```
PROPERTY : (int) → property
GENTESTS : (property, int) → void
VERIFY   : (property) → bool
```

### 2.11 Cache Operators

```
HASH             : (any) → int
CACHE_GET        : (int) → ptr | null
CACHE_PUT        : (int, ptr) → void
CACHE_VERIFY     : (int) → bool
CACHE_INVALIDATE : (int) → void
```

### 2.12 SEN Operators

```
DISCOVER           : (str) → capability_list
CAPABILITY_INFO    : (str) → metadata
CAPABILITY_COST    : (str) → cost_struct
PUBLISH            : (capability) → void
CAPABILITY_AVAILABLE : (str, str) → bool
CAPABILITY_VERSION : (str) → str
CAPABILITY_DEPS    : (str) → capability_list
```

---

## 3. Semantic Rules

### 3.1 Node Definition Rules

**Rule 1: Every node must have an identifier**
```
VALID:   .x: 42
INVALID: 42              -- Missing .id:
```

**Rule 2: Identifiers must be unique within scope**
```
INVALID:
.x: 42
.x: 10                   -- Duplicate identifier!
```

**Rule 3: References must be defined before use**
```
INVALID:
.sum: ADD .a .b          -- .a and .b not defined yet!

VALID:
.a: 10
.b: 20
.sum: ADD .a .b
```

### 3.2 Dependency Rules

**Rule 4: No circular dependencies**
```
INVALID:
.a: ADD .b 1
.b: ADD .a 1             -- Circular dependency!

VALID:
.a: 1
.b: ADD .a 1
.c: ADD .b 1
```

**Rule 5: Execution order follows dependency graph**
```
.c: ADD .a .b            -- Executes third
.a: 42                   -- Executes first (no deps)
.b: 8                    -- Executes first (no deps)

Execution order: {.a, .b} → .c
```

### 3.3 Flow Rules

**Rule 6: Flow operator creates execution dependency**
```
.a > .b                  -- .b executes after .a
```

**Rule 7: Cyclic flow creates loop**
```
.loop: ACC .sock
...
.loop >> .loop           -- Returns to .loop after completion
```

### 3.4 Capability Rules

**Rule 8: Capabilities must be declared before use**
```
INVALID:
.server: HTTP.LISTEN 8080  -- http capability not declared!

VALID:
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
```

**Rule 9: REQUIRES is compile-time only**
```
-- REQUIRES does not generate runtime code
-- It only validates capability usage at compile time
```

### 3.5 Contract Rules

**Rule 10: Contracts are immutable after creation**
```
.contract: CONTRACT 0
.limit: LIMIT 1 1000000    -- Adds to contract
-- Cannot modify contract after SANDBOX uses it
```

---

## 4. Error Codes

### 4.1 Compile-Time Errors

| Code | Message | Cause |
|------|---------|-------|
| E:UNDEF | Undefined reference | Using node before definition |
| E:DUP | Duplicate identifier | Same .id used twice |
| E:CYCLE | Circular dependency | Nodes depend on each other |
| E:TYPE | Type mismatch | Wrong argument types |
| E:CAP | Missing capability | Using capability without REQUIRES |
| E:SYNTAX | Syntax error | Invalid TAYNI syntax |

### 4.2 Runtime Errors

| Code | Value | Meaning |
|------|-------|---------|
| 0 | Success | No error |
| 1 | E_UNAVAIL | Resource unavailable |
| 2 | E_PERM | Permission denied |
| 3 | E_NOTFOUND | Not found |
| 4 | E_TIMEOUT | Operation timed out |
| 5 | E_CONNREF | Connection refused |
| 6 | E_EOF | End of stream |
| 99 | E_UNKNOWN | Unknown error |

---

## 5. Formal Semantics

### 5.1 Evaluation Rules

**Literals:**
```
eval(.id: n) = n                    where n is numeric
eval(.id: "s") = ptr_to("s")        where "s" is string
```

**Arithmetic:**
```
eval(ADD .a .b) = eval(.a) + eval(.b)
eval(SUB .a .b) = eval(.a) - eval(.b)
eval(MUL .a .b) = eval(.a) × eval(.b)
eval(DIV .a .b) = eval(.a) ÷ eval(.b)    if eval(.b) ≠ 0
eval(MOD .a .b) = eval(.a) mod eval(.b)  if eval(.b) ≠ 0
eval(NEG .a) = -eval(.a)
```

**Comparison:**
```
eval(EQ .a .b) = 1 if eval(.a) = eval(.b), else 0
eval(NE .a .b) = 1 if eval(.a) ≠ eval(.b), else 0
eval(LT .a .b) = 1 if eval(.a) < eval(.b), else 0
eval(GT .a .b) = 1 if eval(.a) > eval(.b), else 0
eval(LE .a .b) = 1 if eval(.a) ≤ eval(.b), else 0
eval(GE .a .b) = 1 if eval(.a) ≥ eval(.b), else 0
```

**Logic:**
```
eval(AND .a .b) = 1 if eval(.a) ≠ 0 ∧ eval(.b) ≠ 0, else 0
eval(OR .a .b) = 1 if eval(.a) ≠ 0 ∨ eval(.b) ≠ 0, else 0
eval(NOT .a) = 1 if eval(.a) = 0, else 0
```

### 5.2 Algebraic Properties

**Commutativity:**
```
ADD .a .b ≡ ADD .b .a
MUL .a .b ≡ MUL .b .a
EQ .a .b ≡ EQ .b .a
AND .a .b ≡ AND .b .a
OR .a .b ≡ OR .b .a
```

**Associativity:**
```
ADD (ADD .a .b) .c ≡ ADD .a (ADD .b .c)
MUL (MUL .a .b) .c ≡ MUL .a (MUL .b .c)
```

**Identity:**
```
ADD .a 0 ≡ .a
MUL .a 1 ≡ .a
AND .a 1 ≡ .a
OR .a 0 ≡ .a
```

**Annihilation:**
```
MUL .a 0 ≡ 0
AND .a 0 ≡ 0
OR .a 1 ≡ 1
```

**Involution:**
```
NEG (NEG .a) ≡ .a
NOT (NOT .a) ≡ .a
```

---

## 6. Memory Model

### 6.1 Allocation

```
ALC n allocates n contiguous bytes
Returns ptr p where:
  - valid_range(p) = [0, n-1]
  - initial_value(p, i) = 0 for all i in valid_range
```

### 6.2 Access Rules

```
PUT p o v is valid iff:
  - p is valid ptr from ALC
  - 0 ≤ o < size(p)
  - 0 ≤ v ≤ 255

GET p o is valid iff:
  - p is valid ptr from ALC
  - 0 ≤ o < size(p)
```

### 6.3 Deallocation

```
FRE p invalidates ptr p
After FRE p:
  - PUT p o v is undefined behavior
  - GET p o is undefined behavior
```

---

*Semantic specification for TAYNI v0.22*
*Type safety verification reference*
