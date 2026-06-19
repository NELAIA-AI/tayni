# TAYNI Redis RESP Protocol Specification

## Overview

Redis Serialization Protocol (RESP) implementation for TAYNI.
This is the simplest of the database protocols - TCP only, no auth required for basic use.

## Dependencies

| Dependency | Tier | Status | Required For |
|------------|------|--------|--------------|
| tcp | 0 | ✅ DONE | Transport |

**Note**: Redis is the easiest to implement - only needs TCP!

## RESP Protocol

### Data Types

```
Simple String: +OK\r\n
Error:         -ERR message\r\n
Integer:       :1000\r\n
Bulk String:   $6\r\nfoobar\r\n
Array:         *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
Null:          $-1\r\n
```

### Command Format

Commands are sent as RESP arrays:
```
*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
```

Equivalent to: `SET key value`

## TAYNI API

### REDIS.CONNECT
```tayni
.host: "localhost"
.port: 6379
@.redis: REDIS.CONNECT .host .port
```

### REDIS.SET
```tayni
.key: "mykey"
.value: "myvalue"
@.ok: REDIS.SET .redis .key .value
```

### REDIS.GET
```tayni
.key: "mykey"
.buf: ALC 1024
@.len: REDIS.GET .redis .buf .key
```

### REDIS.DEL
```tayni
.key: "mykey"
@.deleted: REDIS.DEL .redis .key
```

### REDIS.CLOSE
```tayni
@.closed: REDIS.CLOSE .redis
```

## Implementation

### Building RESP Commands

```rust
fn build_command(args: &[&str]) -> Vec<u8> {
    let mut cmd = format!("*{}\r\n", args.len());
    for arg in args {
        cmd.push_str(&format!("${}\r\n{}\r\n", arg.len(), arg));
    }
    cmd.into_bytes()
}

// SET key value
let cmd = build_command(&["SET", "mykey", "myvalue"]);
// Result: *3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n
```

### Parsing RESP Responses

```rust
fn parse_response(data: &[u8]) -> Response {
    match data[0] {
        b'+' => Response::SimpleString(parse_line(&data[1..])),
        b'-' => Response::Error(parse_line(&data[1..])),
        b':' => Response::Integer(parse_integer(&data[1..])),
        b'$' => Response::BulkString(parse_bulk(&data[1..])),
        b'*' => Response::Array(parse_array(&data[1..])),
        _ => Response::Unknown,
    }
}
```

## x86-64 Assembly Implementation

### REDIS.SET Implementation

```asm
; Build SET command: *3\r\n$3\r\nSET\r\n$keylen\r\nkey\r\n$vallen\r\nvalue\r\n
redis_set:
    ; rdi = buffer, rsi = key, rdx = key_len, rcx = value, r8 = value_len
    
    ; Write "*3\r\n"
    mov byte [rdi], '*'
    mov byte [rdi+1], '3'
    mov byte [rdi+2], 0x0D  ; \r
    mov byte [rdi+3], 0x0A  ; \n
    add rdi, 4
    
    ; Write "$3\r\nSET\r\n"
    mov dword [rdi], '$3\r\n'
    add rdi, 4
    mov dword [rdi], 'SET\r'
    mov byte [rdi+4], 0x0A
    add rdi, 5
    
    ; Write key length and key
    ; ... (similar pattern)
    
    ; Send via TCP
    ; recv response
    ; Parse +OK\r\n
    ret
```

## Example

```tayni
-- Redis example
REQUIRES { redis }

-- Connect to Redis
.host: "127.0.0.1"
.port: 6379
@.redis: REDIS.CONNECT .host .port

-- Set a key
.key: "greeting"
.value: "Hello, Redis!"
@.set_ok: REDIS.SET .redis .key .value

-- Get the key
.buf: ALC 256
@.len: REDIS.GET .redis .buf .key

-- Print value
@.out: PRT .buf .len

-- Delete key
@.deleted: REDIS.DEL .redis .key

-- Close connection
@.closed: REDIS.CLOSE .redis
!
```

## Testing

```bash
# Start Redis
docker run -d -p 6379:6379 redis

# Or install locally
redis-server

# Test with redis-cli
redis-cli ping
# PONG
```

## Implementation Priority

Redis is **highest priority** for database support because:
1. Only needs TCP (already implemented)
2. Simple text protocol (RESP)
3. No authentication required for local
4. No TLS required for local
5. Widely used for caching/sessions
