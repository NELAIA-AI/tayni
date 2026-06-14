# NELAIA Pure Build System

## Overview

NELAIA compiles to native binaries with **zero external dependencies**. No libc. No runtime. Just your code and the kernel.

## Requirements

- `clang` (LLVM 14+)
- Linux x86_64 (for now)

## Building Examples

### Hello World

```bash
cd examples/pure
../../scripts/build_pure.sh hello.ll hello
./hello
```

Output:
```
Hello World
```

### HTTP Server

```bash
cd examples/pure
../../scripts/build_pure.sh http_server.ll server
./server &
curl http://localhost:8080
```

Output:
```
OK from NELAIA
```

## Verifying Purity

```bash
# Check that binary has no dynamic dependencies
ldd ./hello
# Output: "not a dynamic executable"

# Check binary size (should be tiny)
ls -la ./hello
# Output: ~8KB

# Check file type
file ./hello
# Output: ELF 64-bit LSB executable, x86-64, statically linked
```

## How It Works

1. **Syscall Layer** (`src/syscalls/linux_x86_64.ll`)
   - Defines direct syscall wrappers
   - No libc calls, just `syscall` instruction
   - Provides: print, read, memory allocation, TCP sockets

2. **Your Program** (`.ll` file)
   - Defines `@nelaia_main()` function
   - Uses syscall layer functions
   - Gets linked with syscall layer

3. **Build** (`scripts/build_pure.sh`)
   - Concatenates syscall layer + your program
   - Compiles with `clang -nostdlib -static`
   - Produces standalone binary

## Syscall Layer API

### Console I/O
```llvm
declare i64 @nelaia_print(i8* %str)      ; Print string to stdout
declare i64 @nelaia_println(i8* %str)    ; Print string + newline
```

### Memory
```llvm
declare i8* @nelaia_alloc(i64 %size)           ; Allocate memory
declare void @nelaia_free(i8* %ptr, i64 %size) ; Free memory
```

### TCP Networking
```llvm
declare i64 @nelaia_tcp_listen(i32 %port)                    ; Create server socket
declare i64 @nelaia_tcp_accept(i32 %server_fd)               ; Accept connection
declare i64 @nelaia_tcp_write(i32 %fd, i8* %buf, i64 %len)   ; Write to socket
declare i64 @nelaia_tcp_read(i32 %fd, i8* %buf, i64 %len)    ; Read from socket
declare void @nelaia_tcp_close(i32 %fd)                      ; Close socket
```

### Low-level Syscalls
```llvm
declare i64 @sys_write(i32 %fd, i8* %buf, i64 %count)
declare i64 @sys_read(i32 %fd, i8* %buf, i64 %count)
declare i64 @sys_open(i8* %path, i32 %flags, i32 %mode)
declare i64 @sys_close(i32 %fd)
declare i8* @sys_mmap(i8* %addr, i64 %len, i32 %prot, i32 %flags, i32 %fd, i64 %off)
declare i64 @sys_munmap(i8* %addr, i64 %len)
declare i64 @sys_socket(i32 %domain, i32 %type, i32 %protocol)
declare i64 @sys_bind(i32 %fd, i8* %addr, i32 %addrlen)
declare i64 @sys_listen(i32 %fd, i32 %backlog)
declare i64 @sys_accept(i32 %fd, i8* %addr, i32* %addrlen)
declare void @sys_exit(i32 %code)
```

## Next Steps

The NELAIA compiler will generate these `.ll` files from graph source code (`.nts` files). For now, you can write LLVM IR directly to test the syscall layer.
