# TAYNI v0.22 - Structured Reference for AIs

This document is optimized for an AI to quickly learn TAYNI syntax and semantics.

## Complete EBNF Grammar

```ebnf
program        = { node | flow | comment } [ "!" ] ;

node           = "." identifier ":" ( literal | operation | reference ) ;
literal        = number | string ;
number         = [ "-" ] digit { digit } [ "." digit { digit } ] ;
string         = '"' { character } '"' ;
reference      = "." identifier ;

operation      = operator { argument } ;
argument       = "." identifier | literal | "(" operation ")" ;

flow           = "." identifier ">" ( "." identifier | effect ) ;
cyclic_flow    = "." identifier ">>" "." identifier ;

effect         = "PRT" argument argument
               | "FOP" argument argument
               | "FRD" argument argument argument
               | "FWR" argument argument argument
               | "FCL" argument
               | io_effect ;

io_effect      = "TCP" | "BND" argument argument | "LST" argument argument
               | "ACC" argument | "XMT" argument argument argument
               | "RCV" argument argument argument | "CLS" argument ;

operator       = arith_op | cmp_op | logic_op | mem_op | cap_op | contract_op
               | test_op | cache_op | sen_op ;

arith_op       = "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "NEG" ;
cmp_op         = "EQ" | "NE" | "LT" | "GT" | "LE" | "GE" ;
logic_op       = "AND" | "OR" | "NOT" ;
mem_op         = "ALC" | "FRE" | "PUT" | "GET" | "CPY" | "SLN" ;

cap_op         = "REQUIRES" | http_op | sql_op | json_op ;
http_op        = "HTTP.LISTEN" | "HTTP.ACCEPT" | "HTTP.RESPOND"
               | "HTTP.METHOD" | "HTTP.PATH" | "HTTP.BODY"
               | "HTTP.GET" | "HTTP.POST" ;
sql_op         = "SQL.CONNECT" | "SQL.QUERY" | "SQL.EXEC"
               | "SQL.NEXT" | "SQL.GET" | "SQL.CLOSE" ;
json_op        = "JSON.PARSE" | "JSON.ENCODE" | "JSON.GET" | "JSON.SET" ;

contract_op    = "CONTRACT" | "GUARANTEE" | "LIMIT" | "SANDBOX"
               | "PROVIDES" | "NEGOTIATE" | "BIND"
               | "DEFCAP" | "EXTCAP" | "COMPOSE" ;

test_op        = "PROPERTY" | "GENTESTS" | "VERIFY" ;

cache_op       = "HASH" | "CACHE_GET" | "CACHE_PUT"
               | "CACHE_VERIFY" | "CACHE_INVALIDATE" ;

sen_op         = "DISCOVER" | "CAPABILITY_INFO" | "CAPABILITY_COST"
               | "PUBLISH" | "CAPABILITY_AVAILABLE"
               | "CAPABILITY_VERSION" | "CAPABILITY_DEPS" ;

identifier     = letter { letter | digit | "_" } ;
comment        = "--" { character } newline ;
```

---

## Operator Tables

### Arithmetic
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| ADD | `ADD .a .b` | a + b | `.sum: ADD .x .y` |
| SUB | `SUB .a .b` | a - b | `.dif: SUB .x .y` |
| MUL | `MUL .a .b` | a × b | `.pro: MUL .x .y` |
| DIV | `DIV .a .b` | a ÷ b | `.div: DIV .x .y` |
| MOD | `MOD .a .b` | a mod b | `.mod: MOD .x .y` |
| NEG | `NEG .a` | -a | `.neg: NEG .x` |

### Comparison (returns 0 or 1)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| EQ | `EQ .a .b` | a == b | `.eq: EQ .x .y` |
| NE | `NE .a .b` | a != b | `.ne: NE .x .y` |
| LT | `LT .a .b` | a < b | `.lt: LT .x .y` |
| GT | `GT .a .b` | a > b | `.gt: GT .x .y` |
| LE | `LE .a .b` | a <= b | `.le: LE .x .y` |
| GE | `GE .a .b` | a >= b | `.ge: GE .x .y` |

### Logic
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| AND | `AND .a .b` | a ∧ b | `.and: AND .x .y` |
| OR | `OR .a .b` | a ∨ b | `.or: OR .x .y` |
| NOT | `NOT .a` | ¬a | `.not: NOT .x` |

### Memory
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| ALC | `ALC size` | Allocate bytes | `.ptr: ALC 1024` |
| FRE | `FRE .ptr` | Free memory | `.f: FRE .ptr` |
| PUT | `PUT .ptr offset byte` | Write byte | `.p: PUT .ptr 0 65` |
| GET | `GET .ptr offset` | Read byte | `.b: GET .ptr 0` |
| CPY | `CPY .dst .src len` | Copy bytes | `.c: CPY .d .s 100` |
| SLN | `SLN .str` | String length | `.l: SLN .msg` |

### I/O
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| PRT | `PRT .buf .len` | Print | `.o: PRT .msg 14` |
| INP | `INP` | Read stdin | `.i: INP` |
| ERR | `ERR` | Error code | `.e: ERR` |
| FOP | `FOP .path mode` | Open file | `.f: FOP "a.txt" 0` |
| FRD | `FRD .h .buf .len` | Read file | `.n: FRD .f .b 1024` |
| FWR | `FWR .h .buf .len` | Write file | `.w: FWR .f .b .n` |
| FCL | `FCL .h` | Close file | `.c: FCL .f` |

### Network
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| TCP | `TCP` | Create socket | `.s: TCP` |
| BND | `BND .sock .addr` | Bind | `.b: BND .s .a` |
| LST | `LST .sock backlog` | Listen | `.l: LST .s 10` |
| ACC | `ACC .sock` | Accept | `.c: ACC .s` |
| XMT | `XMT .sock .buf .len` | Send | `.x: XMT .c .b .n` |
| RCV | `RCV .sock .buf .len` | Receive | `.r: RCV .c .b 1024` |
| CLS | `CLS .sock` | Close | `.cl: CLS .c` |

### Capabilities (Phase 7)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| REQUIRES | `REQUIRES { caps }` | Declare caps | `.c: REQUIRES { http }` |
| HTTP.LISTEN | `HTTP.LISTEN port` | HTTP server | `.s: HTTP.LISTEN 8080` |
| HTTP.ACCEPT | `HTTP.ACCEPT .server` | Accept request | `.r: HTTP.ACCEPT .s` |
| HTTP.RESPOND | `HTTP.RESPOND .req code body` | Respond | `.o: HTTP.RESPOND .r 200 "OK"` |
| HTTP.GET | `HTTP.GET url` | GET request | `.d: HTTP.GET "http://..."` |
| HTTP.POST | `HTTP.POST url body` | POST request | `.p: HTTP.POST "..." .b` |
| SQL.CONNECT | `SQL.CONNECT connstr` | Connect DB | `.c: SQL.CONNECT "..."` |
| SQL.QUERY | `SQL.QUERY .conn sql` | Execute query | `.r: SQL.QUERY .c "SELECT..."` |
| SQL.EXEC | `SQL.EXEC .conn sql` | Execute no result | `.e: SQL.EXEC .c "INSERT..."` |
| SQL.NEXT | `SQL.NEXT .result` | Next row | `.n: SQL.NEXT .r` |
| SQL.GET | `SQL.GET .result col` | Get column | `.v: SQL.GET .r 0` |
| SQL.CLOSE | `SQL.CLOSE .conn` | Close connection | `.cl: SQL.CLOSE .c` |
| JSON.PARSE | `JSON.PARSE .str` | Parse JSON | `.o: JSON.PARSE .s` |
| JSON.ENCODE | `JSON.ENCODE .obj` | Serialize JSON | `.s: JSON.ENCODE .o` |
| JSON.GET | `JSON.GET .obj key` | Get value | `.v: JSON.GET .o "key"` |
| JSON.SET | `JSON.SET .obj key val` | Set value | `.n: JSON.SET .o "k" "v"` |

### Contracts (Phase 8)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| CONTRACT | `CONTRACT id` | Create contract | `.c: CONTRACT 0` |
| GUARANTEE | `GUARANTEE type val` | Declare guarantee | `.g: GUARANTEE 1 0` |
| LIMIT | `LIMIT type max` | Set limit | `.l: LIMIT 1 1000000` |
| SANDBOX | `SANDBOX .code .contract` | Execute sandboxed | `.r: SANDBOX .c .ct` |
| PROVIDES | `PROVIDES id` | Offer capability | `.o: PROVIDES 0` |
| NEGOTIATE | `NEGOTIATE .offer .need` | Negotiate | `.b: NEGOTIATE .o .n` |
| BIND | `BIND .offer .need` | Create binding | `.l: BIND .o .n` |
| DEFCAP | `DEFCAP id` | Define capability | `.c: DEFCAP 0` |
| EXTCAP | `EXTCAP .base` | Extend capability | `.e: EXTCAP .b` |
| COMPOSE | `COMPOSE id` | Compose caps | `.c: COMPOSE 0` |

### Testing (Phase 10)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| PROPERTY | `PROPERTY id` | Define property | `.p: PROPERTY 0` |
| GENTESTS | `GENTESTS .prop count` | Generate tests | `.t: GENTESTS .p 100` |
| VERIFY | `VERIFY .prop` | Verify | `.v: VERIFY .p` |

### Cache (Phase 9)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| HASH | `HASH .value` | Calculate hash | `.h: HASH .v` |
| CACHE_GET | `CACHE_GET .hash` | Look up cache | `.c: CACHE_GET .h` |
| CACHE_PUT | `CACHE_PUT .hash .ir` | Store in cache | `.s: CACHE_PUT .h .ir` |
| CACHE_VERIFY | `CACHE_VERIFY .hash` | Verify integrity | `.v: CACHE_VERIFY .h` |
| CACHE_INVALIDATE | `CACHE_INVALIDATE .hash` | Invalidate | `.i: CACHE_INVALIDATE .h` |

### SEN Ecosystem (Phase 11)
| Operator | Syntax | Semantics | Example |
|----------|--------|-----------|---------|
| DISCOVER | `DISCOVER desc` | Search capabilities | `.c: DISCOVER "json"` |
| CAPABILITY_INFO | `CAPABILITY_INFO name` | Capability info | `.i: CAPABILITY_INFO "json"` |
| CAPABILITY_COST | `CAPABILITY_COST name` | Capability cost | `.c: CAPABILITY_COST "json"` |
| PUBLISH | `PUBLISH .cap` | Publish capability | `.p: PUBLISH .my_cap` |
| CAPABILITY_AVAILABLE | `CAPABILITY_AVAILABLE name region` | Availability | `.a: CAPABILITY_AVAILABLE "json" "global"` |
| CAPABILITY_VERSION | `CAPABILITY_VERSION name` | Version | `.v: CAPABILITY_VERSION "json"` |
| CAPABILITY_DEPS | `CAPABILITY_DEPS name` | Dependencies | `.d: CAPABILITY_DEPS "json"` |

---

## Common Patterns

### Pattern 1: Hello World
```TAYNI
.msg: "Hello TAYNI!\n"
.len: 14
.out: PRT .msg .len
```

### Pattern 2: HTTP Server (Single Request)
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.req: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .req 200 "OK"
```

### Pattern 2b: HTTP Server (Persistent with Loop)
```TAYNI
.caps: REQUIRES { http }
.server: HTTP.LISTEN 8080
.loop: HTTP.ACCEPT .server
.resp: HTTP.RESPOND .loop 200 "OK"
.loop >> .loop
```
**IMPORTANT:** Use `>>` to create a loop. Without `.loop >> .loop`, the server handles only one request and exits.

### Pattern 3: SQL Query
```TAYNI
.caps: REQUIRES { sql }
.conn: SQL.CONNECT "Driver={SQL Server};Server=localhost;Database=test"
.result: SQL.QUERY .conn "SELECT * FROM users"
.close: SQL.CLOSE .conn
```

### Pattern 4: Process JSON
```TAYNI
.caps: REQUIRES { json }
.obj: JSON.PARSE .input
.val: JSON.GET .obj "key"
```

### Pattern 5: TCP Loop
```TAYNI
.sock: TCP
.addr: ALC 16
.a0: PUT .addr 0 2
.a1: PUT .addr 2 31
.a2: PUT .addr 3 144
.bind: BND .sock .addr
.listen: LST .sock 10
.loop: ACC .sock
.buf: ALC 512
.n: RCV .loop .buf 512
.send: XMT .loop .buf .n
.cls: CLS .loop
.loop >> .loop
```

### Pattern 6: AI Discovering Capabilities
```TAYNI
-- Design-time: AI evaluates options
.options: DISCOVER "process data"
.cost: CAPABILITY_COST "json"
.info: CAPABILITY_INFO "json"

-- Code-time: AI uses chosen capability
.caps: REQUIRES { json }
.data: JSON.PARSE .input
```

---

## Semantic Rules

1. **Every node requires `.id:`** - No anonymous operations
2. **Data flow, not sequence** - Execution order determined by dependency graph
3. **Immutability** - Nodes are not modified after definition
4. **Declarative capabilities** - `REQUIRES` declares, does not import
5. **Quantifiable contracts** - Guarantees and limits are numeric
6. **SEN is design-time** - `DISCOVER` is for the AI, not for runtime

---

## Error Codes

| Code | Meaning |
|------|---------|
| 0 | No error |
| 1 | Resource unavailable |
| 2 | Permission denied |
| 3 | Not found |
| 4 | Timeout |
| 5 | Connection refused |
| 6 | End of stream |
| 99 | Unknown error |

---

*Version: 0.22 | Phases 1-11 Completed*
