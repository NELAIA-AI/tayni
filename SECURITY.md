# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.5.x   | :white_check_mark: |
| < 1.5   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in TAYNI, please report it responsibly:

1. **Do NOT** open a public issue
2. Email: security@nelaia.ai
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 24-48 hours
  - High: 7 days
  - Medium: 30 days
  - Low: Next release

## Security Model

TAYNI uses a **capability-based security model**:

### Capabilities

| Capability | Grants Access To |
|------------|------------------|
| `cap:net` | TCP/HTTP networking |
| `cap:fs` | File system operations |
| `cap:env` | Environment variables |
| `cap:proc` | Process management |
| `cap:time` | Time operations |

### Security Properties

1. **No implicit permissions**: Code without capabilities cannot access system resources
2. **Compile-time enforcement**: Missing capabilities cause compilation errors
3. **Zero dependencies**: No supply chain vulnerabilities
4. **Standalone binaries**: No runtime to exploit

### Known Limitations

- No memory safety guarantees (like Rust's borrow checker)
- No sandboxing beyond capability system
- WASI security depends on runtime implementation

## Security Best Practices

When writing TAYNI code:

1. **Minimize capabilities**: Only declare what you need
2. **Validate input**: Especially from network/files
3. **Handle errors**: Use try-catch for risky operations
4. **Avoid hardcoded secrets**: Use environment variables

## Vulnerability Disclosure

We follow responsible disclosure:

1. Reporter notifies us privately
2. We confirm and assess the issue
3. We develop and test a fix
4. We release the fix
5. We credit the reporter (if desired)
6. Public disclosure after fix is available

## Security Audits

- No external audits completed yet
- Planned for Phase 3 (Month 7-12)

## Contact

- Security issues: security@nelaia.ai
- General questions: contact@nelaia.ai

---

*Last updated: 2026-06-19*
