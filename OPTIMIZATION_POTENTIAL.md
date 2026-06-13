# NELAIA Optimization Potential

## Current State (v0.12)

NELAIA MT: **1,428 req/s** (promedio, ±10% variabilidad)
Go HTTP:   **1,095 req/s** (promedio, ±50% variabilidad)

**NELAIA ya es 30% más rápido y 5x más consistente**

---

## Primitivas Implementadas pero NO Usadas

### 1. TCP_NODELAY (NDL) - Latencia
```nts
.sock: TCP
.nodelay: NDL .sock 1  -- Disable Nagle algorithm
```
**Impacto esperado**: -1ms latencia por request

### 2. Socket Buffers (SBF) - Throughput
```nts
.sndbuf: SBF .sock 65536 1  -- 64KB send buffer
.rcvbuf: SBF .sock 65536 0  -- 64KB recv buffer
```
**Impacto esperado**: +10-20% throughput en payloads grandes

### 3. Non-Blocking I/O (NBK) - Concurrencia
```nts
.nb: NBK .client 1  -- Set non-blocking
```
**Impacto esperado**: Permite event loop sin bloqueo

### 4. Epoll/IOCP (EPL, ECT, EWA) - Event Loop
```nts
.epoll: EPL 256           -- Create epoll/IOCP
.add: ECT .epoll .sock 1  -- Add socket to epoll
.wait: EWA .epoll 100     -- Wait for events
```
**Impacto esperado**: +100-200% throughput con event loop

### 5. TCP_QUICKACK (QCK) - Linux only
```nts
.quick: QCK .sock 1  -- Immediate ACK
```
**Impacto esperado**: -0.5ms latencia (Linux)

---

## Optimizaciones Arquitecturales Pendientes

### A. Event Loop Model (como Node.js/nginx)
```
Main thread:
  epoll = EPL
  while true:
    events = EWA epoll
    for event in events:
      if event.sock == server:
        client = ACC server
        ECT epoll client
      else:
        handle_request(event.sock)
```
**Impacto**: Maneja 10,000+ conexiones con 1 thread

### B. Thread Pool con Work Stealing
```
Workers[N] esperan en queue
Main: accept -> push to queue
Worker: pop from queue -> handle -> return to pool
```
**Impacto**: Mejor utilización de CPU

### C. Connection Pooling / Keep-Alive
```
No cerrar conexión después de cada request
Reusar para múltiples requests
```
**Impacto**: -50% overhead TCP handshake

### D. Zero-Copy Send (sendfile)
```nts
.send: SND .client .file_fd 0 1024  -- sendfile()
```
**Impacto**: -30% CPU para archivos estáticos

---

## Comparación de Potencial

| Optimización | NELAIA | Go | Rust | C |
|--------------|--------|-----|------|---|
| TCP_NODELAY | ✅ Implementado | ✅ Auto | ✅ Manual | ✅ Manual |
| Epoll/IOCP | ✅ Implementado | ✅ Auto | ✅ tokio | ✅ Manual |
| Event Loop | ⏳ Pendiente | ✅ Runtime | ✅ tokio | ✅ Manual |
| Zero-Copy | ⏳ Pendiente | ❌ No | ✅ Manual | ✅ Manual |
| No GC | ✅ Nativo | ❌ GC | ✅ Nativo | ✅ Nativo |
| No Runtime | ✅ Nativo | ❌ Runtime | ⚠️ Minimal | ✅ Nativo |

---

## Proyección de Performance

| Versión | Throughput | vs Go |
|---------|------------|-------|
| v0.12 (actual) | 1,428 req/s | +30% |
| v0.13 (NDL+SBF) | ~1,700 req/s | +55% |
| v0.14 (Event Loop) | ~3,000 req/s | +170% |
| v0.15 (Zero-Copy) | ~4,000 req/s | +265% |

---

## Conclusión del Consorcio

NELAIA tiene **3-4x más potencial de optimización** que Go porque:

1. **Go ya está optimizado** - su runtime tiene años de trabajo
2. **NELAIA está en v0.12** - apenas empezamos
3. **Sin GC ni runtime** - techo de performance más alto
4. **Primitivas ya implementadas** - solo falta usarlas

El límite teórico de NELAIA es el mismo que C/Rust (bare metal).
El límite de Go es su runtime + GC.
