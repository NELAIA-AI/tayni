# NELAIA Runtime Architecture
## Direct Kernel Interface - No External Dependencies

**Version:** 0.4  
**Status:** Consortium Approved  
**Date:** 2026-06-13

---

## 1. Core Principle

> "NELAIA does not use libc. NELAIA does not use external libraries. NELAIA speaks directly to the kernel. The binary produced by NELAIA has zero runtime dependencies."

---

## 2. Architecture Overview

### 2.1 Linux x86_64 (Primary Target)

```
┌─────────────────────────────────────────┐
│           NELAIA Graph Code             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         NELAIA Compiler (Rust)          │
│    Emits LLVM IR with inline syscalls   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│            LLVM IR Output               │
│   syscall instructions embedded         │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              clang -nostdlib            │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│     Native Binary (NO libc linked)      │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│            Linux Kernel                 │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              Hardware                   │
└─────────────────────────────────────────┘
```

### 2.2 Windows x64 (Secondary Target)

```
┌─────────────────────────────────────────┐
│           NELAIA Graph Code             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         NELAIA Compiler (Rust)          │
│    Emits LLVM IR with ntdll calls       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│            LLVM IR Output               │
│   Dynamic calls to ntdll.dll            │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              clang / lld                │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│     Native Binary (NO CRT linked)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│    ntdll.dll (Part of Windows OS)       │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│             NT Kernel                   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              Hardware                   │
└─────────────────────────────────────────┘
```

---

## 3. Why This Matters

### 3.1 What We Avoid

| Dependency | Why It's Bad |
|------------|--------------|
| libc (glibc, musl) | External library, version conflicts, bloat |
| MSVC CRT | External runtime, must be installed |
| OpenSSL | External library, security updates needed |
| Any .so/.dll | Deployment complexity |

### 3.2 What We Achieve

| Benefit | Description |
|---------|-------------|
| Zero dependencies | Binary runs on bare OS |
| Minimal size | No library bloat |
| Predictable behavior | No library version differences |
| True portability | Same binary, same behavior |
| Security | Smaller attack surface |

---

## 4. Linux Syscalls Reference

### 4.1 Syscall Convention (x86_64)

```
Registers:
  rax = syscall number
  rdi = arg1
  rsi = arg2
  rdx = arg3
  r10 = arg4
  r8  = arg5
  r9  = arg6

Return:
  rax = result (negative = error)

Instruction:
  syscall
```

### 4.2 Required Syscalls

| Syscall | Number | Signature | Purpose |
|---------|--------|-----------|---------|
| `read` | 0 | `read(fd, buf, count)` | Read from fd |
| `write` | 1 | `write(fd, buf, count)` | Write to fd |
| `open` | 2 | `open(path, flags, mode)` | Open file |
| `close` | 3 | `close(fd)` | Close fd |
| `mmap` | 9 | `mmap(addr, len, prot, flags, fd, off)` | Allocate memory |
| `munmap` | 11 | `munmap(addr, len)` | Free memory |
| `socket` | 41 | `socket(domain, type, protocol)` | Create socket |
| `accept` | 43 | `accept(fd, addr, addrlen)` | Accept connection |
| `bind` | 49 | `bind(fd, addr, addrlen)` | Bind to address |
| `listen` | 50 | `listen(fd, backlog)` | Listen for connections |
| `exit` | 60 | `exit(code)` | Exit process |

### 4.3 LLVM IR for Syscalls

```llvm
; Example: write(1, "Hello", 5) - print to stdout
define i64 @nelaia_sys_write(i32 %fd, i8* %buf, i64 %len) {
entry:
  %fd64 = sext i32 %fd to i64
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 1,        ; syscall number (write)
      i64 %fd64,    ; fd
      i8* %buf,     ; buffer
      i64 %len      ; length
    )
  ret i64 %result
}

; Example: socket(AF_INET, SOCK_STREAM, 0)
define i64 @nelaia_sys_socket(i32 %domain, i32 %type, i32 %protocol) {
entry:
  %d64 = sext i32 %domain to i64
  %t64 = sext i32 %type to i64
  %p64 = sext i32 %protocol to i64
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 41,       ; syscall number (socket)
      i64 %d64,     ; domain (AF_INET = 2)
      i64 %t64,     ; type (SOCK_STREAM = 1)
      i64 %p64      ; protocol (0)
    )
  ret i64 %result
}
```

---

## 5. Windows ntdll Reference

### 5.1 Why ntdll.dll

- ntdll.dll is loaded into EVERY Windows process automatically
- It is the lowest user-mode interface to the kernel
- Its API is stable (unlike raw syscall numbers)
- It is NOT an external dependency - it IS Windows

### 5.2 Required Functions

| Function | Purpose |
|----------|---------|
| `NtWriteFile` | Write to file/socket/console |
| `NtReadFile` | Read from file/socket/console |
| `NtCreateFile` | Open/create file |
| `NtClose` | Close handle |
| `NtAllocateVirtualMemory` | Allocate memory |
| `NtFreeVirtualMemory` | Free memory |
| `NtTerminateProcess` | Exit process |

### 5.3 Socket Operations

Windows sockets require talking to AFD (Ancillary Function Driver):

```
NtCreateFile → open \\Device\\Afd\\Endpoint
NtDeviceIoControlFile → IOCTL_AFD_BIND, IOCTL_AFD_LISTEN, etc.
```

This is more complex than Linux but achievable.

### 5.4 Dynamic Loading Without kernel32

```llvm
; Get ntdll base from PEB (Process Environment Block)
; PEB is at gs:[0x60] on x64 Windows
; PEB->Ldr->InMemoryOrderModuleList contains loaded modules
; ntdll is always the second module

define i8* @get_ntdll_base() {
entry:
  ; Read PEB from gs:[0x60]
  %peb = call i8* asm sideeffect "mov %gs:0x60, $0", "=r"()
  ; Navigate to module list and find ntdll...
  ; (implementation details)
  ret i8* %ntdll_base
}
```

---

## 6. Compilation Flags

### 6.1 Linux

```bash
# Compile LLVM IR to object
clang -c program.ll -o program.o

# Link without libc
ld -nostdlib -static -o program program.o

# Or with clang
clang -nostdlib -static -o program program.ll
```

### 6.2 Windows

```bash
# Compile LLVM IR to object
clang -c program.ll -o program.obj

# Link without CRT
lld-link /nodefaultlib /entry:_start program.obj

# Or with clang
clang -nostdlib -o program.exe program.ll -Wl,/nodefaultlib,/entry:_start
```

---

## 7. Memory Management

### 7.1 Without malloc/free

We implement our own allocator using:
- Linux: `mmap`/`munmap` syscalls
- Windows: `NtAllocateVirtualMemory`/`NtFreeVirtualMemory`

### 7.2 Simple Bump Allocator

```llvm
@heap_base = global i8* null
@heap_current = global i8* null
@heap_end = global i8* null

define i8* @nelaia_alloc(i64 %size) {
entry:
  ; If heap not initialized, mmap a chunk
  ; Bump heap_current by size
  ; Return old heap_current
  ; If out of space, mmap more
}

define void @nelaia_free(i8* %ptr) {
entry:
  ; For bump allocator: no-op
  ; For real allocator: mark as free
  ret void
}
```

---

## 8. String Handling

### 8.1 Without libc string functions

We implement:
- `nelaia_strlen` - count bytes until null
- `nelaia_strcpy` - copy bytes until null
- `nelaia_strcat` - concatenate (allocate new)
- `nelaia_strcmp` - compare byte by byte

All in pure LLVM IR, no external calls.

---

## 9. Entry Point

### 9.1 Linux

```llvm
; Linux entry point (no _start from libc)
define void @_start() {
entry:
  ; Call our main
  %result = call i32 @nelaia_main()
  
  ; Exit syscall
  call void asm sideeffect "syscall",
    "{rax},{rdi}"(i64 60, i32 %result)
  
  unreachable
}
```

### 9.2 Windows

```llvm
; Windows entry point (no mainCRTStartup)
define void @_start() {
entry:
  ; Call our main
  %result = call i32 @nelaia_main()
  
  ; NtTerminateProcess
  %ntdll = call i8* @get_ntdll_base()
  %terminate = call i8* @get_proc_address(i8* %ntdll, "NtTerminateProcess")
  call void %terminate(i8* inttoptr(i64 -1 to i8*), i32 %result)
  
  unreachable
}
```

---

## 10. Purity Levels

| Level | Linux | Windows | Status |
|-------|-------|---------|--------|
| 0 | libc | MSVC CRT | ❌ Rejected |
| 1 | libc (static) | UCRT (static) | ❌ Rejected |
| 2 | syscalls | ntdll.dll | ✅ Approved |
| 3 | syscalls | raw syscalls | ⚠️ Unstable on Windows |

**We implement Level 2: Maximum purity that is stable and practical.**

---

## 11. Roadmap

### Phase 1: Linux x86_64 (v0.4-v0.5)
- [ ] Syscall wrappers in LLVM IR
- [ ] Basic I/O (print, read)
- [ ] Memory allocation (mmap)
- [ ] File operations
- [ ] TCP sockets

### Phase 2: Windows x64 (v0.6)
- [ ] ntdll dynamic resolution
- [ ] Basic I/O via NtWriteFile/NtReadFile
- [ ] Memory via NtAllocateVirtualMemory
- [ ] File operations via NtCreateFile
- [ ] TCP via AFD driver

### Phase 3: Optimization (v0.7+)
- [ ] Better memory allocator
- [ ] String interning
- [ ] Connection pooling

---

## 12. Consortium Decision Record

**Question:** Should NELAIA use libc/runtime.c or direct syscalls?

**Vote:** 6-0 in favor of direct syscalls

**Rationale:**
- "If we use libc, we're just another language with a C runtime"
- "The differentiator is not just the graph paradigm, it's execution purity"
- "AI to Hardware means AI to Hardware, not AI to libc to Hardware"

**Windows Addendum:**
- ntdll.dll is accepted as "kernel interface" not "external dependency"
- It is always present, always loaded, part of the OS itself
- This is as pure as Windows allows

---

*Document generated by the AI Consortium*
*"No libc. No excuses. Direct to kernel."*
