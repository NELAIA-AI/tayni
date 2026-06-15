# Copilot Instructions for NELAIA

When generating NELAIA code (.nela files), follow these rules:

## Critical Rules

1. **All nodes start with dot**: `.name: value`
2. **PRT needs TWO arguments**: `.out: PRT .msg .len` (buffer AND length)
3. **Servers need loops**: Add `.loop >> .loop` at the end for persistent servers
4. **Declare capabilities**: `REQUIRES { http }` before using HTTP operations

## Correct HTTP Server (Persistent)

```nelaia
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.loop: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .loop 200 "Hello!"
.loop >> .loop
```

## Correct Hello World

```nelaia
.msg: "Hello World!\n"
.len: 13
.out: PRT .msg .len
```

## Common Mistakes to Avoid

- ❌ `PRT .msg` - Missing length argument
- ❌ Server without `.loop >> .loop` - Exits after one request
- ❌ `x: 42` - Missing dot prefix, should be `.x: 42`
- ❌ Using HTTP without `REQUIRES { http }`

## String Length Calculation

Count ALL characters including `\n`:
- `"Hello!\n"` = 7 characters (6 + newline)
- `"Hello World!\n"` = 13 characters

## Documentation

- Full reference: docs/NELAIA-REFERENCE-v0.22.md
- Examples: docs/NELAIA-EXAMPLES-v0.22.md
- Common mistakes: docs/COMMON-MISTAKES.md
