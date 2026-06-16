# TAYNI Common Mistakes

**READ THIS FIRST** - These are the most common errors when generating TAYNI code.

---

## 1. Server Without Loop (CRITICAL)

### WRONG - Server exits after one request:
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.req: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .req 200 "OK"
-- Server exits here!
```

### CORRECT - Server stays running:
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.loop: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .loop 200 "OK"
.loop >> .loop  -- THIS IS REQUIRED for persistent server
```

**Rule:** Any server (HTTP, TCP, etc.) needs `.loop >> .loop` to handle multiple requests.

---

## 2. PRT Missing Length (CRITICAL)

### WRONG - Silent failure, prints nothing:
```TAYNI
.msg: "Hello!"
.out: PRT .msg
```

### CORRECT - Prints "Hello!":
```TAYNI
.msg: "Hello!"
.len: 6
.out: PRT .msg .len
```

**Rule:** `PRT` requires TWO arguments: buffer AND length.

---

## 3. Wrong String Length

### WRONG - Truncated output:
```TAYNI
.msg: "Hello World!\n"
.len: 11  -- Wrong! Doesn't count \n
.out: PRT .msg .len
```

### CORRECT - Full output:
```TAYNI
.msg: "Hello World!\n"
.len: 13  -- "Hello World!" (12) + "\n" (1) = 13
.out: PRT .msg .len
```

**Rule:** Count ALL characters including `\n` (newline = 1 char).

---

## 4. Missing Capability Declaration

### WRONG - Will fail:
```TAYNI
.server: HTTP.LISTEN 8080
```

### CORRECT:
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
```

**Rule:** Always declare capabilities with `REQUIRES { }` before using them.

---

## 5. Missing Node Prefix

### WRONG - Invalid syntax:
```TAYNI
x: 42
y: 10
sum: ADD x y
```

### CORRECT:
```TAYNI
.x: 42
.y: 10
.sum: ADD .x .y
```

**Rule:** All node names must start with `.` (dot).

---

## 6. Resource Leaks

### WRONG - File handle leak:
```TAYNI
.file: FOP .path 0
.n: FRD .file .buf 1024
-- Forgot to close!
```

### CORRECT:
```TAYNI
.file: FOP .path 0
.n: FRD .file .buf 1024
.close: FCL .file
```

**Rule:** Always close files (FCL), sockets (CLS), and database connections (SQL.CLOSE).

---

## Quick Reference: Loop Syntax

```TAYNI
-- The >> operator creates a loop
.loop_start: OPERATION .args
-- ... more operations ...
.loop_start >> .loop_start  -- Jump back to loop_start
```

Without `>>`, code executes once and exits. With `>>`, it loops forever.

---

*This document is optimized for AI code generation. Read CRITICAL-PATTERNS.jsonl for structured data.*
