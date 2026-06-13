# NELAIA v0.7 Release Notes

## Overview

NELAIA v0.7 introduces **Network I/O** (TCP sockets) and **Memory Allocation** capabilities, completing the foundation for building networked applications entirely in NELAIA without any external dependencies.

## New Features

### 1. Network Operations (TCP)

New opcodes for TCP socket programming:

| Opcode | Operation | Syntax | Description |
|--------|-----------|--------|-------------|
| `TCP` | Create socket | `.sock: TCP` | Creates a TCP socket |
| `BND` | Bind | `.r: BND .sock port` | Binds socket to port |
| `LST` | Listen | `.r: LST .sock backlog` | Starts listening |
| `ACC` | Accept | `.client: ACC .sock` | Accepts connection |
| `CON` | Connect | `.r: CON .sock "host" port` | Connects to server |
| `XMT` | Transmit | `.r: XMT .sock "data"` | Sends data |
| `RCV` | Receive | `.r: RCV .sock .buf maxlen` | Receives data |

### 2. Memory Operations

New opcodes for dynamic memory allocation:

| Opcode | Operation | Syntax | Description |
|--------|-----------|--------|-------------|
| `ALC` | Allocate | `.ptr: ALC size` | Allocates memory |
| `FRE` | Free | `.r: FRE .ptr` | Frees memory |

### 3. Platform Implementation

#### Linux
- **Network**: Direct syscalls (`socket`, `bind`, `listen`, `accept`, `connect`, `sendto`, `recvfrom`)
- **Memory**: `mmap`/`munmap` syscalls (syscall #9, #11)

#### Windows
- **Network**: Winsock2 via `ws2_32.dll` (`WSAStartup`, `socket`, `bind`, `listen`, `accept`, `connect`, `send`, `recv`)
- **Memory**: `VirtualAlloc`/`VirtualFree` from `kernel32.dll`

## Test Results

### Memory Allocation Test
```
$ .\test_memory.exe
Memory allocated: 1024 bytes
Memory freed
```

### TCP Server Test
```
$ .\test_tcp_server.exe
TCP Server listening on port 8080...
Client connected!
Server closed.
```

### TCP Client Test
```
$ .\test_tcp_client.exe
Connected to server!
Message sent!
Client closed.
```

## Architecture

### Syscall Numbers (Linux x86_64)

| Syscall | Number | Purpose |
|---------|--------|---------|
| `socket` | 41 | Create socket |
| `connect` | 42 | Connect to server |
| `accept` | 43 | Accept connection |
| `sendto` | 44 | Send data |
| `recvfrom` | 45 | Receive data |
| `bind` | 49 | Bind to address |
| `listen` | 50 | Listen for connections |
| `mmap` | 9 | Allocate memory |
| `munmap` | 11 | Free memory |

### Windows DLL Functions

**ws2_32.dll** (Winsock2):
- `WSAStartup`, `socket`, `bind`, `listen`, `accept`, `connect`, `send`, `recv`, `closesocket`

**kernel32.dll** (Memory):
- `VirtualAlloc`, `VirtualFree`

## Opcode Summary (v0.7)

Total opcodes: **40**

| Category | Opcodes |
|----------|---------|
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEG` |
| Comparison | `EQ`, `NE`, `LT`, `GT`, `LE`, `GE` |
| Logic | `AND`, `OR`, `NOT` |
| Collections | `SEQ`, `MAP`, `FLD`, `FLT`, `LEN`, `FST`, `SND` |
| Control | `BRN`, `LOOP` |
| File I/O | `OPN`, `GET`, `PUT`, `CLS` |
| Console I/O | `PRT`, `INP`, `ERR` |
| Network | `TCP`, `BND`, `LST`, `ACC`, `CON`, `XMT`, `RCV` |
| Memory | `ALC`, `FRE` |

## Known Limitations

1. **IP Address Parsing**: Currently hardcoded to `127.0.0.1` for `CON` operation
2. **Memory Size Tracking**: `FRE` uses fixed page size (4096 bytes) for `munmap`
3. **Buffer Management**: No automatic buffer size tracking for `RCV`
4. **Error Handling**: Network errors return negative values but no detailed error codes

## Purity Status

NELAIA v0.7 maintains **100% purity**:
- **Linux**: Direct syscalls only (no libc)
- **Windows**: kernel32.dll + ws2_32.dll only (OS-provided, no external dependencies)

The compiled binaries have **ZERO external library dependencies** beyond the operating system itself.

## Next Steps (v0.8 Roadmap)

1. **IP Address Parsing**: Parse "x.x.x.x" format for `CON`
2. **DNS Resolution**: Hostname to IP resolution
3. **UDP Support**: Datagram sockets
4. **Error Codes**: Detailed error reporting
5. **Buffer Utilities**: String/buffer manipulation opcodes
6. **Loop Implementation**: Full `LOOP` opcode support

## Consortium Approval

Pending review by the AI Consortium.
