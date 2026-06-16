# TAYNI Quick Reference for AI

## Syntax (6 rules)
```
.id: VALUE          -- literal (number or "string")
.id: OP args        -- operation (EVERY line needs .id:)
.id: .other         -- reference to another node
.a > .b             -- data flow (execute a, then b)
.a >> .b            -- cyclic flow (loop back to b)
!GEN args           -- generate nodes (subgraph fusion)
```

## GEN - Graph Element Generators
```
-- Define generator (creates reusable subgraph pattern)
#NAME param1 param2:
  .internal: OP .param1 .param2
  .result: OP .internal
#END

-- Generate nodes (fuses subgraph into main graph)
!NAME .arg1 .arg2
-- Creates nodes: ._g0_name_internal, ._g0_name_result
```

## Important
- Every operation MUST have `.id:` prefix
- Numbers are decimal only (no 0x hex)
- Port 8080 = bytes 31, 144 (0x1F, 0x90)
- Generated nodes are prefixed: `._gN_name_nodeid`

## Control Flow
```
END cond            -- terminate if cond == 0, continue if != 0
BRN cond a b        -- select a if cond != 0, else b
```

## Core Opcodes
```
-- Arithmetic
ADD SUB MUL DIV MOD    -- .r: ADD .a .b

-- Comparison (returns 0 or 1)
EQ NE LT GT LE GE      -- .cmp: LT .a .b

-- Logic
AND OR NOT             -- .r: AND .a .b

-- Memory
ALC size              -- allocate bytes, returns ptr
FRE ptr               -- free memory
PUT ptr offset byte   -- store byte at ptr+offset
GET ptr offset        -- load byte from ptr+offset
CPY dst src len       -- copy len bytes from src to dst
CMP a b len           -- compare len bytes, returns 0 if equal
FND buf char len      -- find char in buf, returns offset or -1
SLN buf               -- string length (until null byte)

-- Vectors (dynamic arrays)
VEC cap               -- create vector with capacity
VPH vec val           -- push value to vector
VGT vec idx           -- get value at index
VST vec idx val       -- set value at index
VLN vec               -- get vector length
VCP vec               -- get vector capacity

-- HashMap
HMP cap               -- create hashmap with capacity
HPT map key val       -- put key-value pair
HGT map key           -- get value by key
HHS map key           -- check if key exists (0 or 1)

-- Strings
CAT dst s1 s2         -- concatenate s1+s2 into dst
CHR str idx           -- get char code at index
SBS dst src start len -- substring from src to dst
ITS buf num           -- integer to string (single digit)

-- I/O
PRT buf len           -- print buf (len bytes) to stdout
FOP path mode         -- open file (mode: 0=read, 1=write)
FRD handle buf len    -- read from file
FWR handle buf len    -- write to file
FCL handle            -- close file

-- Network
TCP                   -- create TCP socket
BND sock addr         -- bind socket to address
LST sock backlog      -- listen for connections
ACC sock              -- accept connection, returns client
XMT sock buf len      -- send data
RCV sock buf len      -- receive data
CLS sock              -- close socket

-- Threading
THR func arg          -- create thread running func(arg)
```

## Address Format (for BND)
```
-- Build sockaddr_in at ptr (port 8080 = 0x1F90):
.addr: ALC 16
.a0: PUT .addr 0 2         -- AF_INET (family)
.a1: PUT .addr 2 31        -- port high byte (8080 >> 8 = 31)
.a2: PUT .addr 3 144       -- port low byte (8080 & 255 = 144)
.a3: PUT .addr 4 0         -- IP: 0.0.0.0 (4 bytes)
.a4: PUT .addr 5 0
.a5: PUT .addr 6 0
.a6: PUT .addr 7 0
```

## Example 1: Hello World
```
.msg: "Hello TAYNI!\n"
.out: PRT .msg 14
!
```

## Example 2: Read File
```
.path: "input.txt"
.file: FOP .path 0
.buf: ALC 1024
.n: FRD .file .buf 1024
.out: PRT .buf .n
.close: FCL .file
!
```

## Example 3: TCP Echo Server (port 8080)
```
.sock: TCP
.addr: ALC 16
.a0: PUT .addr 0 2
.a1: PUT .addr 2 31
.a2: PUT .addr 3 144
.a3: PUT .addr 4 0
.a4: PUT .addr 5 0
.a5: PUT .addr 6 0
.a6: PUT .addr 7 0
.bind: BND .sock .addr
.listen: LST .sock 10
.loop: ACC .sock
.buf: ALC 512
.n: RCV .loop .buf 512
.send: XMT .loop .buf .n
.cls: CLS .loop
.loop >> .loop
!
```

## Patterns
```
-- Loop: use >> to cycle back
.start: OP ...
... more ops ...
.end >> .start

-- Function definition
.fname: FUN .param1 .param2
  ... body ...
!FUN

-- Thread worker
.worker: FUN .arg
  ... work ...
  RET 0
!FUN
.t: THR .worker .data
```

## Compile
```
TAYNI-c program.tayni output
./output
```
