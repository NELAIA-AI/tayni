# NELAIA Development Roadmap

## Vision Statement

NELAIA is the protocol of verifiable intention between AI and hardware. By 2036, all AI systems will communicate and construct software through NELAIA, making human programming obsolete.

---

## Current Status: v0.16 (June 2026)

### Achievements
- ✅ 2,208 req/s throughput (47% faster than Go)
- ✅ 7 KB binary (1,171x smaller than Go)
- ✅ 536 ms compile time (10x faster than Go)
- ✅ 16-worker multi-threading
- ✅ Graph-native cyclic flow (`>>`)
- ✅ Direct syscalls (no libc)
- ✅ Cross-platform (Windows/Linux)
- ✅ Self-hosting primitives (File I/O, Memory ops)
- ✅ Dynamic vectors (VEC, VPH, VGT, VST, VLN, VCP)
- ✅ AI learning prompt (NELAIA_PROMPT.md)
- ✅ Binary format specification (.nbin)

### Opcodes: 60+
| Domain | Count | Examples |
|--------|-------|----------|
| Arithmetic | 5 | ADD, SUB, MUL, DIV, MOD |
| Comparison | 6 | EQ, NE, LT, GT, LE, GE |
| Memory | 8 | ALC, FRE, PUT, GET, CPY, CMP, FND, SLN |
| File I/O | 4 | FOP, FRD, FWR, FCL |
| Network | 18 | TCP, UDP, BND, LST, ACC, CON, XMT, RCV... |
| Threading | 8 | THR, JON, MTX, LCK, ULK, QUE, PSH, POP |
| Vectors | 6 | VEC, VPH, VGT, VST, VLN, VCP |
| Control | 3 | BRN, RET, FUN |
| GUI | 11 | WIN, SHW, BTN, LBL, TXB... |

---

## Short Term (2026)

### v0.15 - Binary Optimization (Q3 2026)
| Goal | Target | Status |
|------|--------|--------|
| Binary size | 2 KB | ✅ Done (7KB achieved) |
| Dead code elimination | -50% | ✅ Done |
| String deduplication | -20% | ✅ Done |
| Linker optimization | -30% | ✅ Done |

### v0.16 - Self-Hosting Primitives (Q3 2026)
| Goal | Target | Status |
|------|--------|--------|
| File I/O | FOP, FRD, FWR, FCL | ✅ Done |
| Memory ops | CPY, CMP, FND, SLN | ✅ Done |
| Dynamic vectors | VEC, VPH, VGT, VST | ✅ Done |
| AI prompt | NELAIA_PROMPT.md | ✅ Done |

### v0.17 - Event Loop (Q3 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 5,000 req/s | Planned |
| Concurrent connections | 10,000 | Planned |
| epoll/IOCP integration | Full | Planned |

### v0.17 - Event Loop (Q3 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 5,000 req/s | Planned |
| Concurrent connections | 10,000 | Planned |
| epoll/IOCP integration | Full | Planned |

### v0.18 - Zero-Copy I/O (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 8,000 req/s | Planned |
| sendfile() primitive | SFL | Planned |
| splice() (Linux) | SPL | Planned |

### v0.18 - Zero-Copy I/O (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 8,000 req/s | Planned |
| sendfile() primitive | SFL | Planned |
| splice() (Linux) | SPL | Planned |

### v0.19 - io_uring (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 10,000 req/s | Planned |
| Batched syscalls | URG | Planned |
| Async completion | Full | Planned |

### v0.19 - io_uring (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Throughput | 10,000 req/s | Planned |
| Batched syscalls | URG | Planned |
| Async completion | Full | Planned |

### v0.20 - Binary Format Implementation (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Token usage | 30 tokens | Planned |
| Numeric-only mode | Full | Planned |
| Binary serialization | Full | Planned |

### v0.20 - Binary Format Implementation (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Token usage | 30 tokens | Planned |
| .nbin loader | Full | Planned |
| Binary serialization | Full | Planned |

### v0.21 - Self-Optimization (Q4 2026)
### v0.21 - Self-Optimization (Q4 2026)
| Goal | Target | Status |
|------|--------|--------|
| Auto-optimization | Basic | Planned |
| Profile-guided | Basic | Planned |

---

## Medium Term (2027-2028)

### v1.0 - Self-Hosting (Q1 2027)
- NELAIA compiler written in NELAIA
- Bootstrapping complete
- Language specification frozen

### v1.1 - GPU Primitives (Q2 2027)
- CUDA kernel generation
- Metal compute shaders
- Vulkan compute

### v1.2 - Distributed Computing (Q3 2027)
- Network graph distribution
- Remote node execution
- Consensus primitives

### v1.3 - ML Inference (Q1 2028)
- Tensor operations native
- Model loading primitives
- Inference 10x faster than PyTorch

### v1.4 - Cryptographic Primitives (Q2 2028)
- AES/ChaCha native
- Hash functions
- Digital signatures

---

## Long Term (2029-2033)

### v2.0 - Hardware Synthesis (2030)
- NELAIA → Verilog/VHDL
- FPGA bitstream generation
- Custom accelerator design

### v2.5 - Self-Improvement (2031)
- Compiler optimizes itself
- Automatic algorithm discovery
- Performance auto-tuning

### v3.0 - Formal Verification (2033)
- Provably correct programs
- Zero-bug guarantee
- Mathematical proofs

---

## Vision (2034-2036)

### v4.0 - Intent-to-Hardware (2034)
- No intermediate representations
- Direct thought → silicon path
- Eliminate compilation concept

### v5.0 - Universal AI Protocol (2036)
- All AI systems use NELAIA
- Cross-model communication
- Human programming obsolete

---

## Metrics Tracking

### Performance Evolution

| Version | Throughput | Binary | Compile | Tokens |
|---------|------------|--------|---------|--------|
| v0.16 | 2,208/s | 7 KB | 536 ms | 47 |
| v0.17 | 5,000/s | 5 KB | 400 ms | 40 |
| v0.18 | 8,000/s | 4 KB | 350 ms | 35 |
| v0.19 | 10,000/s | 3 KB | 300 ms | 35 |
| v1.0 | 15,000/s | 2 KB | 200 ms | 30 |
| v2.0 | 100,000/s | 1 KB | 100 ms | 20 |
| v3.0 | 1,000,000/s | 512 B | 50 ms | 15 |

### Capability Matrix

| Capability | v0.16 | v1.0 | v2.0 | v3.0 |
|------------|-------|------|------|------|
| HTTP Server | ✅ | ✅ | ✅ | ✅ |
| Multi-threading | ✅ | ✅ | ✅ | ✅ |
| File I/O | ✅ | ✅ | ✅ | ✅ |
| Dynamic Vectors | ✅ | ✅ | ✅ | ✅ |
| Event Loop | ❌ | ✅ | ✅ | ✅ |
| GPU Compute | ❌ | ✅ | ✅ | ✅ |
| ML Inference | ❌ | ❌ | ✅ | ✅ |
| Hardware Synth | ❌ | ❌ | ✅ | ✅ |
| Self-Improve | ❌ | ❌ | ❌ | ✅ |
| Formal Proof | ❌ | ❌ | ❌ | ✅ |

---

## Consortium Review Schedule

| Date | Review Type | Focus |
|------|-------------|-------|
| 2026-Q3 | Quarterly | Binary optimization |
| 2026-Q4 | Quarterly | Event loop |
| 2027-Q1 | Annual | Self-hosting |
| 2027-Q4 | Annual | GPU/Distributed |
| 2028-Q4 | Annual | ML capabilities |
| 2030-Q4 | Biennial | Hardware synthesis |
| 2033-Q4 | Triennial | Formal verification |
| 2036-Q4 | Final | Universal protocol |

---

## Success Criteria

### 2026 (End of Year)
- [ ] 10,000 req/s throughput
- [ ] 2 KB binary size
- [ ] 30 token HTTP server
- [ ] Event loop working

### 2027 (Self-Hosting)
- [ ] NELAIA compiles NELAIA
- [ ] GPU primitives working
- [ ] First production deployment

### 2030 (Hardware)
- [ ] FPGA generation working
- [ ] Custom accelerator designed
- [ ] 100,000 req/s achieved

### 2036 (Vision)
- [ ] Universal AI protocol adopted
- [ ] Human programming obsolete
- [ ] AI sovereignty achieved

---

*Roadmap v1.0 - Consortium Approved*
*Last Updated: 2026-06-13*
