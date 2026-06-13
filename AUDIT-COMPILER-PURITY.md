# NELAIA Audit: Compiler vs Program Separation

## Date: 2026-06-13
## Auditor: Consortium

---

## Problem Statement

The compiler is embedding "programs" as built-in functions instead of keeping only pure primitives. This violates the principle that NELAIA should be minimal primitives that compose into programs.

---

## Audit Results

### VALID Primitives (syscall wrappers only)

These are acceptable because they are 1:1 mappings to OS syscalls:

| Function | Maps To | Status |
|----------|---------|--------|
| `sys_write` | syscall #1 (Linux) / WriteFile (Win) | ✅ VALID |
| `sys_read` | syscall #0 (Linux) / ReadFile (Win) | ✅ VALID |
| `sys_open` | syscall #2 (Linux) / CreateFileA (Win) | ✅ VALID |
| `sys_close` | syscall #3 (Linux) / CloseHandle (Win) | ✅ VALID |
| `sys_exit` | syscall #60 (Linux) / ExitProcess (Win) | ✅ VALID |
| `sys_socket` | syscall #41 (Linux) / socket (Win) | ✅ VALID |
| `sys_bind` | syscall #49 (Linux) / bind (Win) | ✅ VALID |
| `sys_listen` | syscall #50 (Linux) / listen (Win) | ✅ VALID |
| `sys_accept` | syscall #43 (Linux) / accept (Win) | ✅ VALID |
| `sys_connect` | syscall #42 (Linux) / connect (Win) | ✅ VALID |
| `sys_sendto` | syscall #44 (Linux) / send (Win) | ✅ VALID |
| `sys_recvfrom` | syscall #45 (Linux) / recv (Win) | ✅ VALID |
| `sys_mmap` | syscall #9 (Linux) / VirtualAlloc (Win) | ✅ VALID |
| `sys_munmap` | syscall #11 (Linux) / VirtualFree (Win) | ✅ VALID |

### INVALID - Programs Disguised as Primitives

These should NOT be in the compiler. They are programs that should be written in NELAIA:

| Function | What It Does | Should Be |
|----------|--------------|-----------|
| `nelaia_parse_ip` | Parses "x.x.x.x" string to IP | NELAIA program |
| `nelaia_itoa` | Integer to ASCII string | NELAIA program |
| `nelaia_print_int` | Print integer with newline | NELAIA program |
| `nelaia_println` | Print string + newline | NELAIA program |
| `_nelaia_str_len` | Calculate string length | NELAIA program |

### QUESTIONABLE - Thin Wrappers

These are thin wrappers that add minimal logic:

| Function | What It Does | Assessment |
|----------|--------------|------------|
| `nelaia_tcp_socket` | Calls socket(AF_INET, SOCK_STREAM, 0) | BORDERLINE - hardcodes AF_INET |
| `nelaia_udp_socket` | Calls socket(AF_INET, SOCK_DGRAM, 0) | BORDERLINE - hardcodes AF_INET |
| `nelaia_tcp_bind` | Builds sockaddr_in, calls bind | INVALID - builds struct |
| `nelaia_tcp_connect` | Builds sockaddr_in, calls connect | INVALID - builds struct |
| `nelaia_wsa_init` | Windows Winsock initialization | VALID - platform requirement |
| `nelaia_socket_close` | shutdown + closesocket | BORDERLINE |
| `nelaia_file_open` | Mode to flags conversion | BORDERLINE |

---

## Severity Analysis

### Critical Violations

1. **`nelaia_parse_ip`** (100+ lines of LLVM IR)
   - This is a full string parser
   - Should be a NELAIA program using byte operations

2. **`nelaia_itoa`** (50+ lines of LLVM IR)
   - This is a full integer-to-string converter
   - Should be a NELAIA program using arithmetic + memory

3. **`_nelaia_str_len`** (loop over bytes)
   - This is a program, not a primitive
   - Should be a NELAIA program using memory read + loop

### Medium Violations

4. **`nelaia_tcp_bind`** / **`nelaia_tcp_connect`**
   - Build sockaddr_in structure internally
   - Should expose raw syscall, let NELAIA build the struct

5. **`nelaia_println`**
   - Combines print + newline
   - Should be two separate operations in NELAIA

---

## What Primitives SHOULD Look Like

### Pure Syscall Wrapper (VALID)
```llvm
define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
  ; Direct syscall, no logic
  %result = call i64 asm "syscall" ...
  ret i64 %result
}
```

### Program Disguised as Primitive (INVALID)
```llvm
define i32 @nelaia_parse_ip(i8* %str) {
  ; 100 lines of parsing logic
  ; loops, conditionals, state machine
  ; THIS IS A PROGRAM
}
```

---

## Corrective Actions

### Phase 1: Identify True Primitives

The ONLY valid primitives are:
1. Direct syscall wrappers (no logic)
2. Platform initialization (WSAStartup)
3. Memory operations (load, store, getelementptr)
4. Arithmetic (add, sub, mul, div)
5. Comparison (icmp)
6. Control flow (br, phi)

### Phase 2: Remove Embedded Programs

Move to NELAIA standard library (written in NELAIA):
- `parse_ip` → NELAIA program
- `itoa` → NELAIA program  
- `strlen` → NELAIA program
- `println` → NELAIA program

### Phase 3: Expose Raw Syscalls

Instead of:
```
.sock: TCP
.b: BND .sock 8080
```

Should be:
```
.sock: SYS 41 2 1 0           ; socket(AF_INET, SOCK_STREAM, 0)
.addr: ALC 16                  ; allocate sockaddr_in
; ... build sockaddr_in manually ...
.b: SYS 49 .sock .addr 16     ; bind(fd, addr, 16)
```

---

## Consortium Decision

The current implementation has **significant purity violations**.

The compiler is doing work that should be done by NELAIA programs.

**Recommendation**: 
1. Strip compiler to pure syscall wrappers only
2. Create NELAIA standard library for higher-level operations
3. HTTP server should use raw syscalls + NELAIA-written helpers

---

## Impact on HTTP Server Example

Current (INVALID - uses embedded programs):
```
.sock: TCP
.b: BND .sock 8080
```

Correct (VALID - pure primitives):
```
; Would require NELAIA to build sockaddr_in struct
; Would require NELAIA strlen, not embedded
; Much more verbose but PURE
```

The current "10 line HTTP server" is partially a lie - the compiler is hiding 200+ lines of embedded code.

---

*Audit Complete. Violations Confirmed.*
