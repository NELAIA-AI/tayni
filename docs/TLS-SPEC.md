# TAYNI TLS 1.3 Implementation Specification

## Overview

TLS implementation using Windows SChannel (native, no external dependencies).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TAYNI TLS Module                         │
│                   (stdlib/tier2/tls)                        │
├─────────────────────────────────────────────────────────────┤
│  TLS.CONNECT  │  TLS.ACCEPT  │  TLS.SEND  │  TLS.RECV      │
├─────────────────────────────────────────────────────────────┤
│                  Windows SChannel API                        │
│              (secur32.dll / schannel.dll)                   │
├─────────────────────────────────────────────────────────────┤
│  AcquireCredentialsHandle  │  InitializeSecurityContext     │
│  EncryptMessage            │  DecryptMessage                │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Tier | Status | Required For |
|------------|------|--------|--------------|
| tcp | 0 | ✅ DONE | Transport layer |
| secur32.dll | - | ✅ IMPORTS ADDED | SChannel API |

## API

### TLS.CONNECT
```tayni
-- Connect to TLS server
.host: "example.com"
.port: 443
@.tls: TLS.CONNECT .host .port
```

### TLS.SEND / TLS.RECV
```tayni
-- Send encrypted data
.data: "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n"
@.sent: TLS.SEND .tls .data 40

-- Receive encrypted data
.buf: ALC 4096
@.received: TLS.RECV .tls .buf 4096
```

### TLS.CLOSE
```tayni
@.closed: TLS.CLOSE .tls
```

## SChannel Handshake Flow

```
Client                                    Server
  │                                         │
  │  1. AcquireCredentialsHandle            │
  │     (SECPKG_CRED_OUTBOUND)              │
  │                                         │
  │  2. InitializeSecurityContext ────────► │
  │     (sends ClientHello)                 │
  │                                         │
  │  ◄──────────────────────────────────────│
  │     (receives ServerHello, Cert, etc.)  │
  │                                         │
  │  3. InitializeSecurityContext ────────► │
  │     (sends ClientKeyExchange, etc.)     │
  │                                         │
  │  ◄──────────────────────────────────────│
  │     (receives Finished)                 │
  │                                         │
  │  4. Handshake complete                  │
  │     (SEC_E_OK)                          │
  │                                         │
  │  5. EncryptMessage / DecryptMessage     │
  │     (application data)                  │
  │                                         │
```

## Implementation Steps

### Step 1: Credential Acquisition
```c
SCHANNEL_CRED cred = {0};
cred.dwVersion = SCHANNEL_CRED_VERSION;
cred.grbitEnabledProtocols = SP_PROT_TLS1_2 | SP_PROT_TLS1_3;

AcquireCredentialsHandle(
    NULL,                    // principal
    UNISP_NAME,              // package
    SECPKG_CRED_OUTBOUND,    // usage
    NULL,                    // logon ID
    &cred,                   // auth data
    NULL,                    // get key fn
    NULL,                    // get key arg
    &hCred,                  // credential handle
    &tsExpiry                // expiry
);
```

### Step 2: Security Context Initialization
```c
SecBufferDesc outBufferDesc;
SecBuffer outBuffer;
outBuffer.BufferType = SECBUFFER_TOKEN;
outBuffer.cbBuffer = 0;
outBuffer.pvBuffer = NULL;

InitializeSecurityContext(
    &hCred,                  // credential
    NULL,                    // context (NULL for first call)
    hostname,                // target name
    ISC_REQ_FLAGS,           // requirements
    0,                       // reserved
    SECURITY_NATIVE_DREP,    // data representation
    NULL,                    // input (NULL for first call)
    0,                       // reserved
    &hContext,               // new context
    &outBufferDesc,          // output
    &contextAttr,            // attributes
    &tsExpiry                // expiry
);
```

### Step 3: Handshake Loop
```c
while (status == SEC_I_CONTINUE_NEEDED) {
    // Send output buffer to server
    send(socket, outBuffer.pvBuffer, outBuffer.cbBuffer, 0);
    
    // Receive server response
    recv(socket, inBuffer, sizeof(inBuffer), 0);
    
    // Continue handshake
    status = InitializeSecurityContext(...);
}
```

### Step 4: Encrypt/Decrypt
```c
// Encrypt
SecBuffer buffers[4];
buffers[0].BufferType = SECBUFFER_STREAM_HEADER;
buffers[1].BufferType = SECBUFFER_DATA;
buffers[2].BufferType = SECBUFFER_STREAM_TRAILER;
buffers[3].BufferType = SECBUFFER_EMPTY;

EncryptMessage(&hContext, 0, &bufferDesc, 0);

// Decrypt
DecryptMessage(&hContext, &bufferDesc, 0, NULL);
```

## Error Handling

| Error Code | Meaning | Action |
|------------|---------|--------|
| SEC_E_OK | Success | Continue |
| SEC_I_CONTINUE_NEEDED | More data needed | Continue handshake |
| SEC_E_INCOMPLETE_MESSAGE | Partial data | Read more |
| SEC_E_CERT_EXPIRED | Certificate expired | Fail |
| SEC_E_WRONG_PRINCIPAL | Hostname mismatch | Fail |

## Testing

```tayni
-- Test TLS connection to httpbin.org
REQUIRES { tls }

.host: "httpbin.org"
.port: 443

@.tls: TLS.CONNECT .host .port
@.check: EQ .tls 0
@.j: JNZ :error .check

.request: "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nConnection: close\r\n\r\n"
@.sent: TLS.SEND .tls .request 60

.response: ALC 4096
@.received: TLS.RECV .tls .response 4096

@.out: PRT .response .received
@.closed: TLS.CLOSE .tls
!

:error
.err: "TLS connection failed\n"
@.out: PRT .err 22
!
```
