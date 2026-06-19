# TAYNI Hash Implementation Specification

## Current State

The hash implementations are simplified stubs using XOR-based algorithms.
They produce deterministic output but are NOT cryptographically secure.

## Real Implementation Options

### Option 1: Windows CNG (bcrypt.dll) - RECOMMENDED

Uses Windows Cryptography API: Next Generation.

```c
// SHA-256 using BCrypt
BCRYPT_ALG_HANDLE hAlg;
BCRYPT_HASH_HANDLE hHash;
BYTE hash[32];
DWORD hashLen = 32;

BCryptOpenAlgorithmProvider(&hAlg, BCRYPT_SHA256_ALGORITHM, NULL, 0);
BCryptCreateHash(hAlg, &hHash, NULL, 0, NULL, 0, 0);
BCryptHashData(hHash, data, dataLen, 0);
BCryptFinishHash(hHash, hash, hashLen, 0);
BCryptDestroyHash(hHash);
BCryptCloseAlgorithmProvider(hAlg, 0);
```

**Pros:**
- Hardware accelerated (uses CPU SHA extensions)
- FIPS 140-2 compliant
- No external dependencies (built into Windows)

**Cons:**
- Windows-only
- Requires IAT entries for bcrypt.dll

### Option 2: Pure Assembly Implementation

Implement SHA-256 directly in x86-64 assembly.

**Pros:**
- Zero dependencies
- Cross-platform potential

**Cons:**
- ~2000 lines of assembly
- Complex to implement correctly
- No hardware acceleration

## BCrypt API Calls Required

Already added to `pe/imports.rs`:

```rust
ImportFunction { name: "BCryptOpenAlgorithmProvider", hint: 0 },
ImportFunction { name: "BCryptCloseAlgorithmProvider", hint: 1 },
ImportFunction { name: "BCryptCreateHash", hint: 10 },
ImportFunction { name: "BCryptHashData", hint: 11 },
ImportFunction { name: "BCryptFinishHash", hint: 12 },
ImportFunction { name: "BCryptDestroyHash", hint: 13 },
```

## Algorithm Identifiers

```rust
// Wide strings for BCrypt
pub const BCRYPT_SHA256_ALGORITHM: &[u8] = b"S\0H\0A\02\05\06\0\0\0";
pub const BCRYPT_SHA1_ALGORITHM: &[u8] = b"S\0H\0A\01\0\0\0";
pub const BCRYPT_MD5_ALGORITHM: &[u8] = b"M\0D\05\0\0\0";
```

## Implementation Plan

### Phase 1: BCrypt Integration (Current)
- [x] Add bcrypt.dll imports
- [x] Define algorithm constants
- [ ] Generate code to call BCryptHash

### Phase 2: Assembly Codegen
```asm
; SHA-256 using BCrypt
emit_hash_sha256_bcrypt:
    ; Allocate stack space
    sub rsp, 0x60
    
    ; BCryptOpenAlgorithmProvider(&hAlg, L"SHA256", NULL, 0)
    lea rcx, [rsp+0x40]     ; &hAlg
    lea rdx, [sha256_alg]   ; L"SHA256"
    xor r8, r8              ; NULL
    xor r9, r9              ; 0
    call [BCryptOpenAlgorithmProvider]
    
    ; BCryptHash(hAlg, NULL, 0, data, dataLen, hash, 32)
    mov rcx, [rsp+0x40]     ; hAlg
    xor rdx, rdx            ; NULL
    xor r8, r8              ; 0
    mov r9, rsi             ; data
    mov [rsp+0x20], dataLen ; dataLen
    lea rax, [rsp+0x50]     ; hash buffer
    mov [rsp+0x28], rax
    mov qword [rsp+0x30], 32 ; hashLen
    call [BCryptHash]
    
    ; BCryptCloseAlgorithmProvider(hAlg, 0)
    mov rcx, [rsp+0x40]
    xor rdx, rdx
    call [BCryptCloseAlgorithmProvider]
    
    ; Convert 32-byte hash to 64-char hex string
    ; ... (hex conversion code)
    
    add rsp, 0x60
    ret
```

## Testing

```tayni
-- Test SHA-256
.data: "Hello, World!"
.hash: ALC 65

@.len: HASH.SHA256 .hash .data 13
@.out: PRT .hash .len

-- Expected: dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f
```

## Hash Output Formats

| Algorithm | Binary Size | Hex String Size |
|-----------|-------------|-----------------|
| MD5 | 16 bytes | 32 chars |
| SHA-1 | 20 bytes | 40 chars |
| SHA-256 | 32 bytes | 64 chars |
| SHA-384 | 48 bytes | 96 chars |
| SHA-512 | 64 bytes | 128 chars |
