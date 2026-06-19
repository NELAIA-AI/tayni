# TAYNI Crypto Implementation Specification

## Current State (Stubs)

The current crypto implementations are simplified stubs:
- `AES.ENCRYPT/DECRYPT`: XOR-based cipher (NOT secure)
- `RSA.ENCRYPT/DECRYPT`: Not implemented (returns mock data)
- `SHA1/MD5`: Simplified hash (NOT cryptographically secure)

## Required for Production

### AES-256-GCM Implementation

AES-256-GCM requires:
1. **AES-NI Instructions** (x86-64):
   - `AESENC` - AES encryption round
   - `AESENCLAST` - AES final encryption round
   - `AESKEYGENASSIST` - Key expansion
   - `PCLMULQDQ` - Carry-less multiplication for GCM

2. **Key Expansion** (256-bit key → 15 round keys)
3. **GCM Mode**:
   - Counter mode encryption
   - GHASH authentication
   - 96-bit nonce
   - 128-bit authentication tag

### RSA-OAEP Implementation

RSA-OAEP requires:
1. **Big Integer Arithmetic** (2048+ bit numbers)
2. **Modular Exponentiation** (Montgomery multiplication)
3. **OAEP Padding** (SHA-256 based)
4. **Key Generation** (Miller-Rabin primality test)

### Recommended Approach

For production use, consider:
1. **Windows CNG API** (bcrypt.dll):
   - `BCryptEncrypt/BCryptDecrypt`
   - `BCryptGenerateKeyPair`
   - Hardware-accelerated, FIPS-compliant

2. **Native Implementation** (for zero-dependency goal):
   - Use AES-NI for AES operations
   - Implement GCM using PCLMULQDQ
   - ~2000 lines of x86-64 assembly

## Implementation Priority

1. **Phase 1**: Use Windows CNG via IAT (quick, secure)
2. **Phase 2**: Native AES-NI implementation (zero-deps)
3. **Phase 3**: Native RSA implementation (complex)

## Files to Modify

- `pe_gen.rs`: `emit_aes_encrypt`, `emit_aes_decrypt`
- `pe/imports.rs`: Add bcrypt.dll imports
- `pe/constants.rs`: Add crypto constants

## Test Cases

```tayni
-- AES-256-GCM test
.key: ALC 32
.nonce: ALC 12
.plaintext: "Hello, World!"
.ciphertext: ALC 256
.tag: ALC 16

@.enc_len: AES.GCM_ENCRYPT .ciphertext .tag .key .nonce .plaintext 13
@.dec_len: AES.GCM_DECRYPT .decrypted .key .nonce .ciphertext .enc_len .tag
```
