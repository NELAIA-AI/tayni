# TAYNI WebSocket Implementation (RFC 6455)

## Current State

The WebSocket implementation has:
- Frame encoding (WS.SEND) - Basic text frames
- Frame decoding (WS.RECV) - Basic parsing
- Stubs for WS.CONNECT, WS.ACCEPT, WS.CLOSE

## RFC 6455 Requirements

### Opening Handshake (Client)
```
GET /path HTTP/1.1
Host: server.example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```

### Opening Handshake (Server Response)
```
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

### Sec-WebSocket-Accept Calculation
```
accept = base64(SHA1(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"))
```

### Frame Format
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-------+-+-------------+-------------------------------+
|F|R|R|R| opcode|M| Payload len |    Extended payload length    |
|I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
|N|V|V|V|       |S|             |   (if payload len==126/127)   |
| |1|2|3|       |K|             |                               |
+-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
|     Extended payload length continued, if payload len == 127  |
+ - - - - - - - - - - - - - - - +-------------------------------+
|                               |Masking-key, if MASK set to 1  |
+-------------------------------+-------------------------------+
| Masking-key (continued)       |          Payload Data         |
+-------------------------------- - - - - - - - - - - - - - - - +
:                     Payload Data continued ...                :
+ - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
|                     Payload Data continued ...                |
+---------------------------------------------------------------+
```

### Opcodes
- 0x0: Continuation
- 0x1: Text
- 0x2: Binary
- 0x8: Close
- 0x9: Ping
- 0xA: Pong

## Implementation Plan

### Phase 1: Frame Encoding/Decoding (DONE)
- [x] Basic text frame encoding
- [x] Basic frame decoding
- [ ] Extended length support (126, 127)
- [ ] Masking support (client->server)
- [ ] Binary frames
- [ ] Control frames (ping/pong/close)

### Phase 2: Handshake
- [ ] HTTP upgrade request generation
- [ ] Sec-WebSocket-Key generation (random base64)
- [ ] Sec-WebSocket-Accept validation (SHA1 + base64)
- [ ] Server-side handshake

### Phase 3: Full Protocol
- [ ] Fragmentation support
- [ ] Close handshake
- [ ] Ping/pong heartbeat
- [ ] Error handling

## TAYNI API

```tayni
-- Client connection
.url: "ws://localhost:8080/chat"
@.ws: WS.CONNECT .url 25

-- Send text message
.msg: "Hello, WebSocket!"
.frame: ALC 256
@.frame_len: WS.SEND .frame .ws .msg 17

-- Receive message
.recv_buf: ALC 1024
@.recv_len: WS.RECV .recv_buf .ws .frame .frame_len

-- Close connection
@.closed: WS.CLOSE .ws
```

## Dependencies

- TCP socket operations (already implemented)
- SHA1 hash (for handshake)
- Base64 encoding (for handshake)
- Random number generation (for masking key)
