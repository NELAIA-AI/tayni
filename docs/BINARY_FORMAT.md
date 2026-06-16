# TAYNI Binary Format (.nbin)

## Purpose
Direct graph serialization for AI-to-AI communication.
No text parsing needed - graphs are loaded directly into memory.

## Format

```
HEADER (16 bytes):
  [0-3]   Magic: "NBIN" (4 bytes)
  [4-5]   Version: u16 (currently 1)
  [6-7]   Flags: u16 (reserved)
  [8-11]  Node count: u32
  [12-15] String table offset: u32

NODES (variable):
  For each node:
    [0]     Node type: u8
              0 = Literal
              1 = Operation
              2 = Flow
              3 = Function
    [1]     Op code: u8 (for Operation nodes)
    [2-3]   Arg count: u16
    [4-7]   ID string index: u32
    [8+]    Args (variable)

ARG FORMAT:
  [0]     Arg type: u8
            0 = Reference (string index follows)
            1 = Integer (i64 follows)
            2 = String (string index follows)
  [1+]    Value (4 or 8 bytes)

STRING TABLE:
  [0-3]   String count: u32
  For each string:
    [0-1]   Length: u16
    [2+]    UTF-8 bytes (no null terminator)
```

## Op Codes (u8)

```
0x00-0x0F: Arithmetic
  0x00 ADD, 0x01 SUB, 0x02 MUL, 0x03 DIV, 0x04 MOD

0x10-0x1F: Comparison
  0x10 EQ, 0x11 NE, 0x12 LT, 0x13 GT, 0x14 LE, 0x15 GE

0x20-0x2F: Memory
  0x20 ALC, 0x21 FRE, 0x22 PUT, 0x23 GET
  0x24 CPY, 0x25 CMP, 0x26 FND, 0x27 SLN

0x30-0x3F: I/O
  0x30 PRT, 0x31 FOP, 0x32 FRD, 0x33 FWR, 0x34 FCL

0x40-0x4F: Network
  0x40 TCP, 0x41 UDP, 0x42 BND, 0x43 LST
  0x44 ACC, 0x45 CON, 0x46 XMT, 0x47 RCV, 0x48 CLS

0x50-0x5F: Threading
  0x50 THR, 0x51 JON, 0x52 MTX, 0x53 LCK, 0x54 ULK
  0x55 QUE, 0x56 PSH, 0x57 POP

0x60-0x6F: Vectors
  0x60 VEC, 0x61 VPH, 0x62 VGT, 0x63 VST, 0x64 VLN, 0x65 VCP

0x70-0x7F: Control
  0x70 BRN, 0x71 RET

0x80-0x8F: GUI
  0x80 WIN, 0x81 SHW, 0x82 BTN, 0x83 LBL, 0x84 TXB
```

## Usage

```
-- Compile to binary
TAYNI-c program.tyn --emit-bin program.nbin

-- Load and execute binary
TAYNI-c program.nbin output
```

## Benefits

1. **No parsing** - Direct memory load
2. **Compact** - ~50% smaller than text
3. **Fast** - O(1) node access
4. **AI-native** - Graphs, not text
