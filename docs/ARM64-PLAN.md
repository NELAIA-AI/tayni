# ARM64 Linux Backend - Technical Plan

> **Status:** Draft / Planning
> **Target:** Phase 2-3
> **Priority:** Medium

## Overview

This document outlines the plan for implementing ARM64 (AArch64) Linux support in TAYNI. ARM64 is increasingly important for:

- Cloud servers (AWS Graviton, Azure Ampere, GCP Tau T2A)
- Edge devices (Raspberry Pi 4/5, NVIDIA Jetson)
- Apple Silicon (via Linux VMs)

## Current State

TAYNI already has partial ARM64 support in `elf_arm64.rs`:

```rust
// Existing code structure
pub fn generate_arm64_elf(graph: &Graph) -> Vec<u8>
```

### What Works
- Basic ELF64 header generation for ARM64
- Simple syscall emission (write, exit)
- String literal handling

### What's Missing
- Full instruction encoding
- Function calls and stack frames
- Arithmetic operations
- Control flow (branches, loops)
- Memory operations
- TCP/HTTP networking

## Architecture

### ARM64 Instruction Encoding

ARM64 uses fixed 32-bit instructions. Key instruction formats:

```
Data Processing (Immediate):
  31 30 29 28-24 23-22 21-10  9-5  4-0
  sf op  S  10001  shift  imm12  Rn   Rd

Data Processing (Register):
  31 30 29 28-24 23-22 21  20-16 15-10 9-5  4-0
  sf op  S  01011  shift  0   Rm   imm6  Rn   Rd

Branch:
  31-26   25-0
  000101  imm26  (unconditional)
  
Load/Store:
  31-30 29-27 26 25-24 23-22 21  20-16 15-12 11-10 9-5  4-0
  size  111   V  00    opc   0   imm9  op2   Rn    Rt
```

### Register Allocation

ARM64 has 31 general-purpose registers (X0-X30):

| Register | Purpose | TAYNI Usage |
|----------|---------|-------------|
| X0-X7 | Arguments/Return | Function args, syscall args |
| X8 | Indirect result | Syscall number |
| X9-X15 | Temporaries | Scratch registers |
| X16-X17 | IP0/IP1 | Intra-procedure call |
| X18 | Platform | Reserved |
| X19-X28 | Callee-saved | Local variables |
| X29 | Frame pointer | Stack frame |
| X30 | Link register | Return address |
| SP | Stack pointer | Stack |

### Linux Syscalls

ARM64 Linux syscalls use:
- X8: syscall number
- X0-X5: arguments
- X0: return value
- Instruction: `SVC #0`

Key syscalls:
```
write:  64  (fd, buf, count)
exit:   93  (status)
socket: 198 (domain, type, protocol)
bind:   200 (fd, addr, addrlen)
listen: 201 (fd, backlog)
accept: 202 (fd, addr, addrlen)
connect:203 (fd, addr, addrlen)
```

## Implementation Plan

### Phase 1: Core Instructions (Week 1-2)

1. **Instruction Encoder Module**
   ```rust
   pub struct Arm64Encoder {
       code: Vec<u8>,
   }
   
   impl Arm64Encoder {
       pub fn mov_imm(&mut self, rd: u8, imm: u64);
       pub fn add_reg(&mut self, rd: u8, rn: u8, rm: u8);
       pub fn sub_reg(&mut self, rd: u8, rn: u8, rm: u8);
       pub fn mul_reg(&mut self, rd: u8, rn: u8, rm: u8);
       pub fn ldr(&mut self, rt: u8, rn: u8, offset: i16);
       pub fn str(&mut self, rt: u8, rn: u8, offset: i16);
       pub fn b(&mut self, offset: i32);
       pub fn bl(&mut self, offset: i32);
       pub fn ret(&mut self);
       pub fn svc(&mut self, imm: u16);
   }
   ```

2. **Basic Operations**
   - Integer arithmetic (ADD, SUB, MUL, DIV)
   - Comparisons (CMP, CSET)
   - Logical operations (AND, ORR, EOR)

### Phase 2: Control Flow (Week 3)

1. **Branches**
   - Conditional branches (B.EQ, B.NE, B.LT, etc.)
   - Unconditional branch (B)
   - Branch with link (BL)

2. **Loops**
   - While loops
   - For loops
   - Break/continue

### Phase 3: Functions (Week 4)

1. **Calling Convention**
   - Stack frame setup/teardown
   - Argument passing
   - Return values
   - Callee-saved register preservation

2. **Function Calls**
   - Direct calls
   - Indirect calls (function pointers)

### Phase 4: Networking (Week 5-6)

1. **TCP Sockets**
   - socket() syscall
   - bind() syscall
   - listen() syscall
   - accept() syscall
   - connect() syscall
   - read()/write() on sockets

2. **HTTP Server**
   - Port to ARM64 from x86-64 implementation
   - Verify with curl/wget

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_arm64_mov_imm() {
    let mut enc = Arm64Encoder::new();
    enc.mov_imm(0, 42);
    // Verify instruction encoding
}
```

### Integration Tests
- Compile simple programs
- Run on QEMU ARM64
- Run on real ARM64 hardware (Raspberry Pi, Graviton)

### CI/CD
```yaml
# GitHub Actions
jobs:
  test-arm64:
    runs-on: ubuntu-latest
    steps:
      - uses: docker/setup-qemu-action@v3
      - name: Test ARM64
        run: |
          cargo build --target aarch64-unknown-linux-gnu
          qemu-aarch64 ./target/aarch64-unknown-linux-gnu/debug/tayni
```

## Resources

### Documentation
- [ARM Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest)
- [ARM64 Linux ABI](https://github.com/ARM-software/abi-aa)
- [Linux syscall table](https://arm64.syscall.sh/)

### Tools
- QEMU for emulation
- GDB for debugging
- objdump for disassembly

## Timeline

| Week | Milestone |
|------|-----------|
| 1-2 | Core instruction encoding |
| 3 | Control flow |
| 4 | Functions |
| 5-6 | Networking |
| 7 | Testing & polish |

## Success Criteria

1. ✅ Hello World compiles and runs on ARM64 Linux
2. ✅ Arithmetic operations work correctly
3. ✅ Control flow (if/while) works
4. ✅ Functions with arguments work
5. ✅ TCP server runs on ARM64
6. ✅ HTTP server responds correctly
7. ✅ Binary size < 15KB for HTTP server

## Dependencies

- None (zero external dependencies maintained)
- Testing requires QEMU or ARM64 hardware

---

*Document created: 2026-06-19*
*Last updated: 2026-06-19*
