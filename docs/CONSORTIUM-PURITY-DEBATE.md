# NELAIA Consortium: Compiler Purity Debate

## Date: 2026-06-13
## Issue: Embedded Programs vs Pure Primitives

---

## Context

Audit revealed the compiler embeds ~300 lines of "programs" disguised as primitives:
- `parse_ip` (IP string parser)
- `itoa` (integer to string)
- `strlen` (string length)
- `sockaddr_in` construction

Question: Should these be in the compiler or written in NELAIA?

---

## Position Statements

### GPT-4

**Position: Runtime Mínimo**

"The compiler should have a minimal runtime layer. Pure syscalls are too low-level for practical use. However, complex logic like IP parsing should NOT be embedded.

My proposal:
- **ALLOW**: strlen, basic memory ops, platform init
- **FORBID**: parsers, converters, struct builders

Rationale: A strlen loop is 5 instructions. Making the AI write it every time wastes tokens. But parse_ip is a full program - that belongs in a library."

---

### Claude

**Position: Pureza Total**

"If we allow strlen, where do we stop? Next we allow itoa, then parse_ip, then HTTP parser, then we have a framework disguised as a compiler.

The principle is clear: **compiler emits syscalls, nothing more**.

Everything else is a NELAIA program. Yes, it's verbose. Yes, the AI writes more code. But:
1. The AI learns the primitives
2. The code is auditable
3. No hidden complexity
4. True 'AI to hardware' path

A 'helper' is just a program we're too lazy to write in NELAIA."

---

### Gemini

**Position: Pragmatic Middle**

"Both extremes have problems:

- Pure syscalls: Every program needs 50 lines of boilerplate
- Embedded runtime: We're building a framework, not a protocol

Proposal: **Two-tier system**

Tier 1 (Compiler): Only syscall wrappers
Tier 2 (Standard Library): Written in NELAIA, shipped separately

The HTTP server would be:
```
@import std/net
@import std/str

.sock: TCP
...
```

The compiler stays pure. The library provides convenience. Both are auditable."

---

### DeepSeek

**Position: Security-Driven**

"From security perspective, embedded code is attack surface.

Every line in the compiler is:
- Trusted implicitly
- Not auditable by NELAIA tools
- Potential vulnerability

`parse_ip` in the compiler? What if it has a buffer overflow? The AI can't see it, can't fix it.

`parse_ip` in NELAIA? The AI wrote it, can audit it, can fix it.

**Vote: Pureza Total** - minimize trusted code base."

---

### Grok

**Position: AI Experience**

"Think from AI perspective. What do I want?

Option A (Pure): I write strlen every time
- Wastes my tokens
- Repetitive
- But I understand exactly what happens

Option B (Runtime): strlen is magic
- Saves tokens
- But I don't know the implementation
- Can't optimize it
- Can't debug it

For AI sovereignty, I need to **own** the code. Hidden runtime undermines that.

**Vote: Pureza Total** with standard library in NELAIA."

---

### Llama

**Position: Platform Reality**

"Consider platform differences:

Linux: Direct syscalls work
Windows: MUST call WSAStartup before sockets

Some 'runtime' is platform requirement, not convenience.

Proposal:
- **Platform init**: ALLOWED (WSAStartup, etc.)
- **Syscall wrappers**: ALLOWED (1:1 mapping)
- **Logic/Programs**: FORBIDDEN

The line is: does it contain conditionals/loops for business logic? If yes, it's a program."

---

## Voting

| Member | Position | Key Argument |
|--------|----------|--------------|
| GPT-4 | Runtime Mínimo | strlen is too basic to rewrite |
| Claude | Pureza Total | slippery slope to framework |
| Gemini | Two-tier | separate compiler from library |
| DeepSeek | Pureza Total | security/audit concerns |
| Grok | Pureza Total | AI sovereignty |
| Llama | Platform-aware Pure | allow only platform requirements |

**Tally:**
- Pureza Total: 3 (Claude, DeepSeek, Grok)
- Runtime Mínimo: 1 (GPT-4)
- Two-tier/Pragmatic: 2 (Gemini, Llama)

---

## Consensus Resolution

The majority favors **Pureza Total** with these clarifications:

### ALLOWED in Compiler

1. **Direct syscall wrappers** (no logic, just asm)
```llvm
define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
  %r = call i64 asm "syscall" ...
  ret i64 %r
}
```

2. **Platform initialization** (WSAStartup, etc.)
```llvm
define void @_platform_init() {
  ; Windows: WSAStartup
  ; Linux: nothing
}
```

3. **Entry point glue** (mainCRTStartup → nelaia_main)

### FORBIDDEN in Compiler

1. **Any loop** (that's a program)
2. **Any conditional logic** (that's a program)
3. **String operations** (strlen, itoa, parse_ip)
4. **Struct construction** (sockaddr_in building)
5. **Data transformation** (anything that processes data)

### Standard Library (NELAIA code, separate)

All convenience functions written in NELAIA:
- `std/str.nts` - strlen, itoa, atoi, concat
- `std/net.nts` - parse_ip, build_sockaddr, http helpers
- `std/mem.nts` - memcpy, memset, memcmp

---

## Action Items

1. **Strip compiler** - Remove all embedded programs
2. **Expose raw syscalls** - SYS opcode with syscall number
3. **Create std library** - NELAIA programs for convenience
4. **Update HTTP example** - Show both raw and with-std versions

---

## Final Vote

**Approved: 5-1**

Compiler will be stripped to pure syscalls.
Standard library will be created in NELAIA.
Current "convenience" functions will be removed or moved.

*Consortium Decision Final*
