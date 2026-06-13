# NELAIA Quick Reference for AI

## Syntax (5 rules)
```
.id: VALUE          -- literal (number or "string")
.id: OP args        -- operation with arguments
.id: .other         -- reference to another node
.a > .b             -- data flow (execute a, then b)
.a >> .b            -- cyclic flow (loop back to b)
```

## Core Opcodes
```
-- Arithmetic
ADD SUB MUL DIV MOD    -- .r: ADD .a .b

-- Comparison (returns 0 or 1)
EQ NE LT GT LE GE      -- .cmp: LT .a .b

-- Memory
ALC size              -- allocate bytes, returns ptr
PUT ptr offset byte   -- store byte at ptr+offset
GET ptr offset        -- load byte from ptr+offset

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
-- Build sockaddr_in at ptr:
.addr: ALC 16
PUT .addr 0 2         -- AF_INET (family)
PUT .addr 2 0x1F      -- port high byte (8080 = 0x1F90)
PUT .addr 3 0x90      -- port low byte
PUT .addr 4 0         -- IP: 0.0.0.0 (4 bytes)
PUT .addr 5 0
PUT .addr 6 0
PUT .addr 7 0
```

## Example 1: Hello World
```
.msg: "Hello NELAIA!\n"
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

## Example 3: TCP Echo Server
```
.sock: TCP
.addr: ALC 16
PUT .addr 0 2
PUT .addr 2 0x1F
PUT .addr 3 0x90
PUT .addr 4 0
PUT .addr 5 0
PUT .addr 6 0
PUT .addr 7 0
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
nelaia-c program.nts output
./output
```
