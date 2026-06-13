# NELAIA Comprehensive Architecture Specification
**Version:** 1.1 (Absolute Purity Edition)
**Status:** Foundational Baseline
**Paradigm:** Declarative Resource Intent Model (DRIM)
**Backend:** Pure LLVM Intermediate Representation (IR)

---

## 1. Executive Summary & Context
If this document is being read after a total loss of context, start here.

**The Problem:** Modern Artificial Intelligence (LLMs) is forced to generate code in languages designed for human cognitive limitations (e.g., Python, C++, Rust). These languages require complex Abstract Syntax Trees (AST), nested curly braces, indentation rules, and procedural loops. For an AI, this causes "Inference Bottlenecks": excessive token usage, high hallucination rates on formatting, and massive latency.

**The Solution:** NELAIA is an AI-Native Metalanguage and Compiler. It is not designed for humans to write. It is designed for LLMs to generate instantly. It uses a "Zero-Syntax" approach: no brackets, no nesting, just a linear stream of compressed tokens. The NELAIA compiler (written in Rust) ingests these tokens in O(n) time and emits highly optimized LLVM Intermediate Representation (IR) that compiles directly to native machine code (`.exe` / `.elf`).

---

## 2. Core Architectural Philosophies

### 2.1 The Principle of Absolute Purity
NELAIA must never use "crutches". It does not transpile to Rust or C. It does not use external C libraries to fake complex behavior. The compiler emits raw LLVM IR instructions natively. If an operation requires interacting with the Operating System, NELAIA natively links to the OS's base `libc` (e.g., `msvcrt.dll` on Windows) via low-level Foreign Function Interfaces (FFI).

### 2.2 The Law of Lexical Compression
LLMs bill and process by "tokens". A word like `STATIC_LOAD` splits into three tokens for an LLM. To optimize generation speed and cost, NELAIA enforces the **3-Letter Lexicon Rule**. All operational commands are compressed into a single token (e.g., `STR`, `OUT`, `EXE`).

---

## 3. The NELAIA Token Stream (NTS) Language Spec

NTS files (`.nts`) are parsed linearly by spaces. The language is structured around mathematical reference IDs (e.g., `#1`, `#2`) to track data without human variable names.

### Current v1.1 Lexicon (Purist Core)

1. **`STR <ref_id> <string_literal>`**
   - **Purpose:** Allocates a static string in memory.
   - **Example:** `STR #1 "Hello AI"`
   - **Internal Mapping:** Maps to LLVM `private unnamed_addr constant`.

2. **`OUT <buffer_ref>`**
   - **Purpose:** Prints the referenced buffer to standard output.
   - **Example:** `OUT #1`
   - **Internal Mapping:** Maps to a native call to `puts` from the OS `libc`.

3. **`EXE`**
   - **Purpose:** Marks the end of the execution graph and triggers the build.
   - **Example:** `EXE`

---

## 4. Compiler Codebase Architecture (Rust)

The compiler `nelaia-c` is written in Rust for memory safety during parsing. It is divided into 4 tightly coupled modules:

### 4.1 `main.rs` (The Orchestrator)
- Reads the `.nts` file from the disk.
- Invokes the Parser.
- Invokes the Emitter to generate the `.ll` file.
- Handles standard I/O logging for the compilation process.

### 4.2 `ir.rs` (Intermediate Representation)
- Defines the `Opcode` Enum representing the 3-letter lexicon (`STR`, `OUT`, `EXE`).
- Defines the `NelaiaGraph` struct, which holds a linear `Vec<Opcode>`. This graph replaces complex nested ASTs used by traditional compilers.

### 4.3 `parser.rs` (The Zero-Syntax Ingestor)
- Reads the file line by line.
- Splits tokens by the `"` character to safely extract string literals.
- Extracts mathematical references by parsing `#` (e.g., `#1` -> `1`).
- Instantly matches the 3-letter tokens and pushes them to the `NelaiaGraph`.
- **Complexity:** O(n). Fastest possible ingestion.

### 4.4 `emitter.rs` (The LLVM Backend & Optimizer)
This is the heart of the "Absolute Purity" engine.
1. **IR Generation:** It iterates over the `NelaiaGraph` and constructs a `.ll` string.
2. **Global String Allocation:** For every `STR`, it calculates the length (including the null terminator `\00`) and declares a global constant (e.g., `@str.1 = private unnamed_addr constant [9 x i8] c"Hello AI\00"`).
3. **Native Invocation:** For every `OUT`, it calls `@puts(ptr @str.1)`.
4. **Assembly (Clang):** It invokes the system's `clang` as an assembler.
   - **CRITICAL:** It passes the `-O3` flag. This engages the LLVM V8 Optimization engine, performing loop unrolling, vectorization, and dead-code elimination.
   - It passes `-w` to silence warnings, keeping the AI's output clean.

---

## 5. End-to-End Pipeline Example

**Input (`pureza_opt.nts`):**
```text
STR #1 "NELAIA Optimizacion Extrema"
OUT #1
EXE
```

**Intermediate LLVM IR Generated (`nelaia_temp_emit.ll`):**
```llvm
; NELAIA AUTO-GENERATED LLVM IR (V8 OPTIMIZED)
declare i32 @puts(ptr)
@str.1 = private unnamed_addr constant [28 x i8] c"NELAIA Optimizacion Extrema\00"

define i32 @main() {
entry:
  %call1 = call i32 @puts(ptr @str.1)
  ret i32 0
}
```

**Final Output:**
A heavily optimized `nelaia_artifact.exe` that natively executes the code at peak CPU efficiency.

---

## 6. Future Roadmap: The Path to v2.0

If reconstructing this project, the immediate next architectural phases are:

1. **Basic Arithmetic (ALU Integration):** Add 3-letter tokens for math (`ADD`, `SUB`, `MUL`, `DIV`) mapping directly to LLVM's `add`, `sub`, etc.
2. **Conditional Branching (Mathematical Jumps):** Implement zero-syntax branching using reference evaluation (e.g., `JMP #target #condition`).
3. **The Bootstrapping Phase (Direct Syscalls):** Once NELAIA is Turing complete via basic math and pointers, write the Standard Library (Network, File I/O) purely in NTS using raw LLVM IR `syscall` instructions, completely deprecating the reliance on `libc`'s `puts`.
