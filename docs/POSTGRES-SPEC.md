# TAYNI PostgreSQL Wire Protocol Specification

## Overview

PostgreSQL wire protocol v3.0 implementation for TAYNI.

## Dependencies

| Dependency | Tier | Status | Required For |
|------------|------|--------|--------------|
| tcp | 0 | ✅ DONE | Transport |
| hash (MD5) | 1 | ⚠️ STUB | MD5 auth |
| hash (SHA256) | 1 | ⚠️ STUB | SCRAM-SHA-256 auth |
| tls | 2 | ⚠️ STUB | SSL connections (optional) |

## Protocol Messages

### Startup

```
┌────────────────────────────────────────┐
│ StartupMessage                         │
├────────────────────────────────────────┤
│ Int32  │ Length (including self)       │
│ Int32  │ Protocol version (196608)     │
│ String │ "user"                        │
│ String │ username                      │
│ String │ "database"                    │
│ String │ database_name                 │
│ Byte   │ 0 (terminator)                │
└────────────────────────────────────────┘
```

### Authentication

```
┌────────────────────────────────────────┐
│ AuthenticationMD5Password ('R')        │
├────────────────────────────────────────┤
│ Byte   │ 'R' (82)                      │
│ Int32  │ 12 (length)                   │
│ Int32  │ 5 (MD5 auth type)             │
│ Byte[4]│ Salt                          │
└────────────────────────────────────────┘

MD5 password = "md5" + md5(md5(password + user) + salt)
```

### Query

```
┌────────────────────────────────────────┐
│ Query ('Q')                            │
├────────────────────────────────────────┤
│ Byte   │ 'Q' (81)                      │
│ Int32  │ Length                        │
│ String │ SQL query (null-terminated)   │
└────────────────────────────────────────┘
```

### Response Messages

```
RowDescription ('T')  - Column metadata
DataRow ('D')         - Row data
CommandComplete ('C') - Query finished
ReadyForQuery ('Z')   - Ready for next query
ErrorResponse ('E')   - Error occurred
```

## TAYNI API

### PG.CONNECT
```tayni
.host: "localhost"
.port: 5432
.user: "postgres"
.pass: "password"
.db: "mydb"

@.conn: PG.CONNECT .host .port .user .pass .db
```

### PG.QUERY
```tayni
.sql: "SELECT id, name FROM users WHERE active = true"
.result: ALC 65536

@.rows: PG.QUERY .conn .result .sql
```

### PG.EXEC
```tayni
.sql: "INSERT INTO users (name) VALUES ('Alice')"
@.affected: PG.EXEC .conn .sql
```

### PG.CLOSE
```tayni
@.closed: PG.CLOSE .conn
```

## Implementation

### Connection State Machine

```
┌─────────┐     StartupMessage      ┌──────────────┐
│ Initial │ ───────────────────────►│ Authenticating│
└─────────┘                         └──────────────┘
                                           │
                    AuthenticationOk       │
                    ◄──────────────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    Ready    │◄────────────┐
                    └─────────────┘             │
                           │                   │
                    Query  │                   │ ReadyForQuery
                           ▼                   │
                    ┌─────────────┐            │
                    │  Executing  │────────────┘
                    └─────────────┘
```

### Message Parsing

```rust
fn parse_message(data: &[u8]) -> Message {
    let msg_type = data[0] as char;
    let length = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
    let payload = &data[5..5 + length as usize - 4];
    
    match msg_type {
        'R' => parse_auth(payload),
        'T' => parse_row_description(payload),
        'D' => parse_data_row(payload),
        'C' => parse_command_complete(payload),
        'Z' => parse_ready_for_query(payload),
        'E' => parse_error(payload),
        _ => Message::Unknown(msg_type),
    }
}
```

## Example

```tayni
-- PostgreSQL example
REQUIRES { postgres }

-- Connect
.host: "localhost"
.port: 5432
.user: "postgres"
.pass: "secret"
.db: "testdb"

@.conn: PG.CONNECT .host .port .user .pass .db
@.check: EQ .conn 0
@.j: JNZ :error .check

-- Query
.sql: "SELECT * FROM users LIMIT 10"
.result: ALC 65536
@.rows: PG.QUERY .conn .result .sql

-- Print result
@.out: PRT .result .rows

-- Close
@.closed: PG.CLOSE .conn
!

:error
.err: "Connection failed\n"
@.out: PRT .err 18
!
```

## Testing

1. Start PostgreSQL locally
2. Create test database and user
3. Run TAYNI test program
4. Verify query results
