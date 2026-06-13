# NELAIA v0.11 - Ultra Low Latency Primitives

## New Opcodes Added

### Socket Options (Ultra-Low-Latency)

| Opcode | Name | Description | Linux | Windows |
|--------|------|-------------|-------|---------|
| NDL | TCP_NODELAY | Disable Nagle algorithm | setsockopt | setsockopt |
| QCK | TCP_QUICKACK | Immediate ACK | setsockopt | N/A (no-op) |
| SBF | SO_SNDBUF/RCVBUF | Set buffer sizes | setsockopt | setsockopt |
| KAL | SO_KEEPALIVE | Keep connection alive | setsockopt | setsockopt |

### High-Performance I/O

| Opcode | Name | Description | Linux | Windows |
|--------|------|-------------|-------|---------|
| EPL | epoll_create | Create event multiplexer | epoll_create1 | CreateIoCompletionPort |
| ECT | epoll_ctl | Add/modify/delete fd | epoll_ctl | CreateIoCompletionPort |
| EWA | epoll_wait | Wait for events | epoll_wait | GetQueuedCompletionStatus |

## Usage Examples

### TCP_NODELAY (Disable Nagle)
```nts
.sock: TCP
.nodelay: NDL .sock 1
```

### Large Buffers
```nts
.sock: TCP
.bufsize: SBF .sock 65536
```

### epoll Event Loop (Linux)
```nts
.epfd: EPL
.add: ECT .epfd 1 .sock 1
.wait: EWA .epfd .buf 64 1000
```

## Benchmark Results

Test: 1000 requests, 20 concurrent connections

| Server | Req/s | P50 ms | P99 ms | Binary |
|--------|-------|--------|--------|--------|
| Go 1.22 | 723 | 1.92 | 25.29 | 8.4 MB |
| **NELAIA Ultra** | **604** | 20.31 | 29.57 | **5.6 KB** |
| NELAIA Opt | 602 | 14.79 | 28.93 | 5.6 KB |
| NELAIA Fair | 590 | 10.41 | 30.08 | 5.6 KB |
| Rust 1.78 | 489 | 12.28 | 28.89 | 139 KB |
| C (Clang) | 425 | 13.26 | 41.97 | 113 KB |

## Key Findings

1. **NELAIA ranks 2nd** - Only Go is faster
2. **NELAIA beats Rust and C** - Despite being a new language
3. **Binary size unchanged** - 5.6 KB with all optimizations
4. **Zero errors** - 1000/1000 successful requests

## Compiler Version

Updated to v0.11 with ultra-low-latency primitives.
