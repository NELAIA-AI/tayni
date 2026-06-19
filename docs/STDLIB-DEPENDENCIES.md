# TAYNI Stdlib Dependency Graph

## Tier Structure

```
Tier 0 (Essential)     Tier 1 (Common)        Tier 2 (Specialized)
─────────────────      ───────────────        ────────────────────
json                   hash ←──────────────── crypto (AES, RSA)
string                 time                   tls ←── crypto, hash
base64 ←───────────────────────────────────── tls, websocket
url                    regex                  websocket ←── base64, hash
http                   format                 postgres ←── tls (optional)
args                   uuid                   redis ←── tcp
random ←───────────────────────────────────── crypto, tls
log                    validation             sql
router                 test                   grpc ←── http, tls
env                    jwt ←── json, hash     yaml
                       async                  csv
                       timeout                xml
                       path                   pqc ←── crypto
                                              cors
                                              cookie
                                              gzip
                                              retry
```

## TLS 1.3 Dependencies

TLS 1.3 requires these components (in order of implementation):

### 1. Tier 1: hash (SHA-256, SHA-384)
```
Location: stdlib/tier1/hash.tyn
Ops: HASH.SHA256, HASH.SHA384, HASH.SHA1
Status: STUB - needs real implementation
Dependencies: None
```

### 2. Tier 2: crypto (AES-256-GCM, RSA, ECDHE)
```
Location: stdlib/tier2/crypto.tyn
Ops: AES.ENCRYPT, AES.DECRYPT, RSA.ENCRYPT, RSA.DECRYPT, ECDHE.KEYGEN
Status: STUB (XOR cipher) - needs real implementation
Dependencies: hash (for key derivation)
```

### 3. Tier 2: tls
```
Location: stdlib/tier2/tls.tyn
Ops: TLS.CONNECT, TLS.ACCEPT, TLS.SEND, TLS.RECV, TLS.CLOSE
Status: STUB - needs full implementation
Dependencies: crypto, hash, random, base64
```

## PostgreSQL Dependencies

PostgreSQL wire protocol requires:

### 1. Tier 0: tcp (already implemented)
```
Ops: TCP, BND, LST, ACC, XMT, RCV, CLS
Status: IMPLEMENTED
```

### 2. Tier 1: hash (MD5 for auth, SHA-256 for SCRAM)
```
Ops: HASH.MD5, HASH.SHA256
Status: STUB
Dependencies: None
```

### 3. Tier 2: tls (optional, for SSL connections)
```
Status: STUB
Dependencies: crypto, hash
```

### 4. Tier 2: postgres
```
Location: stdlib/tier2/postgres.tyn
Ops: PG.CONNECT, PG.QUERY, PG.EXEC, PG.CLOSE
Status: STUB
Dependencies: tcp, hash, tls (optional)
```

## Redis Dependencies

Redis RESP protocol requires:

### 1. Tier 0: tcp (already implemented)
```
Status: IMPLEMENTED
```

### 2. Tier 2: redis
```
Location: stdlib/tier2/redis.tyn
Ops: REDIS.CONNECT, REDIS.GET, REDIS.SET, REDIS.DEL, REDIS.CLOSE
Status: STUB (in-memory mock)
Dependencies: tcp only (no TLS required for basic)
```

## Implementation Priority

Based on dependencies, the implementation order should be:

1. **hash (Tier 1)** - No dependencies, needed by crypto and tls
2. **crypto (Tier 2)** - Depends on hash, needed by tls
3. **tls (Tier 2)** - Depends on crypto, hash, random, base64
4. **redis (Tier 2)** - Only needs tcp (already done)
5. **postgres (Tier 2)** - Needs tcp, hash, optionally tls

## Current Implementation Status

| Module | Tier | Status | Dependencies Met |
|--------|------|--------|------------------|
| tcp | 0 | ✅ REAL | N/A |
| json | 0 | ✅ REAL | N/A |
| string | 0 | ✅ REAL | N/A |
| base64 | 0 | ⚠️ STUB | N/A |
| random | 0 | ⚠️ STUB | N/A |
| hash | 1 | ⚠️ STUB | N/A |
| crypto | 2 | ⚠️ STUB | ❌ hash |
| tls | 2 | ⚠️ STUB | ❌ crypto, hash |
| websocket | 2 | ⚠️ PARTIAL | ⚠️ base64, hash |
| redis | 2 | ⚠️ STUB | ✅ tcp |
| postgres | 2 | ⚠️ STUB | ⚠️ tcp, hash, tls |

## Next Steps

1. Implement real `hash` module (SHA-256, MD5)
2. Implement real `base64` module
3. Implement real `random` module (CSPRNG)
4. Implement `crypto` using Windows CNG (bcrypt.dll)
5. Implement `tls` using Windows SChannel or crypto primitives
6. Implement `redis` RESP protocol (simple, tcp-only)
7. Implement `postgres` wire protocol
