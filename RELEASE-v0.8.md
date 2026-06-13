# NELAIA v0.8 Release Notes

## Overview

NELAIA v0.8 introduces **IP Address Parsing**, **UDP Sockets**, **Error Checking**, and **Loop Control Flow**, completing the foundation for building robust networked applications.

## New Features

### 1. IP Address Parsing

The `CON` (connect) operation now parses IP addresses in "x.x.x.x" format:

```
.sock: TCP
.conn: CON .sock "192.168.1.100" 8080
```

Previously, only `127.0.0.1` was supported (hardcoded). Now any valid IPv4 address works.

**Implementation**: Pure LLVM IR function `nelaia_parse_ip` that:
- Iterates through the string character by character
- Accumulates digits for each octet
- Handles '.' as octet separator
- Returns 32-bit IP in network byte order

### 2. UDP Socket Support

New `UDP` opcode for datagram sockets:

```
.sock: UDP
.sent: XMT .sock "Hello" 5
.c1: CLS .sock
```

| Platform | Implementation |
|----------|----------------|
| Linux | `socket(AF_INET, SOCK_DGRAM, 0)` via syscall #41 |
| Windows | `socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)` via ws2_32.dll |

### 3. Error Checking (CHK)

New `CHK` opcode for error handling:

```
.result: CON .sock "192.168.1.1" 80
.checked: CHK .result -1
```

`CHK value error_code` returns:
- `error_code` if `value < 0`
- `value` otherwise

This enables simple error checking patterns without complex branching.

### 4. Loop Control Flow

New opcodes for iteration:

| Opcode | Description |
|--------|-------------|
| `LOOP n` | Loop up to n iterations |
| `BRK` | Break out of loop |
| `CNT` | Continue to next iteration |

```
.i: LOOP 10
-- loop body here
```

## Test Results

### UDP Socket Test
```
$ .\test_udp.exe
UDP socket created
UDP socket closed
```

### IP Parsing Test
```
$ .\test_ip_parse.exe
TCP socket created
Connection attempted (expected to fail)
Socket closed
```

### Loop Test
```
$ .\test_loop.exe
Starting loop test...
Loop iteration
Loop complete!
```

## Opcode Summary (v0.8)

Total opcodes: **45**

| Category | Opcodes |
|----------|---------|
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEG` |
| Comparison | `EQ`, `NE`, `LT`, `GT`, `LE`, `GE` |
| Logic | `AND`, `OR`, `NOT` |
| Collections | `SEQ`, `MAP`, `FLD`, `FLT`, `LEN`, `FST`, `SND` |
| Control | `BRN`, `LOOP`, `BRK`, `CNT` |
| File I/O | `OPN`, `GET`, `PUT`, `CLS` |
| Console I/O | `PRT`, `INP`, `ERR` |
| Network | `TCP`, `UDP`, `BND`, `LST`, `ACC`, `CON`, `XMT`, `RCV` |
| Memory | `ALC`, `FRE` |
| Error | `CHK` |

## Architecture Changes

### IP Parsing Function (LLVM IR)

```llvm
define i32 @nelaia_parse_ip(i8* %str) {
  ; Parse "x.x.x.x" string to 32-bit network byte order
  ; Uses state machine: accumulate digits, switch on '.'
  ; Returns combined octets: o1 | (o2<<8) | (o3<<16) | (o4<<24)
}
```

### UDP Socket Creation

```llvm
; Linux
define i64 @nelaia_udp_socket() {
  %fd = call i64 @sys_socket(i64 2, i64 2, i64 0)  ; AF_INET, SOCK_DGRAM
  ret i64 %fd
}

; Windows
define i64 @nelaia_udp_socket() {
  call void @nelaia_wsa_init()
  %fd = call i64 @socket(i32 2, i32 2, i32 17)  ; AF_INET, SOCK_DGRAM, IPPROTO_UDP
  ret i64 %fd
}
```

## Purity Status

NELAIA v0.8 maintains **100% purity**:
- All new features implemented in pure LLVM IR
- No new external dependencies
- IP parsing done without any library calls

## Known Limitations

1. **LOOP**: Current implementation is basic; full iteration with body execution requires sub-graph integration
2. **IPv6**: Not yet supported (IPv4 only)
3. **DNS**: Hostnames not resolved (IP addresses only)

## Next Steps (v0.9 Roadmap)

1. DNS resolution (hostname to IP)
2. Socket options (SO_REUSEADDR, TCP_NODELAY)
3. String manipulation opcodes (CAT, CPY, CMP)
4. Buffer operations (SET, ZRO)

## Consortium Approval

Pending review.
