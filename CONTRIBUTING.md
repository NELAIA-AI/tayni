# Contributing to TAYNI

Thank you for your interest in contributing to TAYNI!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/tayni.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Commit: `git commit -m "Add your feature"`
7. Push: `git push origin feature/your-feature`
8. Open a Pull Request

## Development Setup

```bash
# Clone the repository
git clone https://github.com/NELAIA-AI/tayni.git
cd tayni/archive/rust-bootstrap

# Build the compiler
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_name
```

## Code Style

- Use `cargo fmt` before committing
- Use `cargo clippy` to check for issues
- Follow Rust naming conventions
- Add tests for new features

## Areas to Contribute

### High Priority
- [ ] WASI Preview 2 implementation
- [ ] More code examples
- [ ] Documentation improvements
- [ ] Bug fixes

### Medium Priority
- [ ] ARM64 backend
- [ ] macOS Mach-O verification
- [ ] LSP improvements
- [ ] VS Code extension features

### Low Priority (Future)
- [ ] GPU targets
- [ ] Quantum (QIR) targets
- [ ] Self-hosting compiler

## Reporting Issues

When reporting issues, please include:
- TAYNI version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages (if any)

## Pull Request Guidelines

1. Keep PRs focused on a single change
2. Update documentation if needed
3. Add tests for new features
4. Ensure all tests pass
5. Update CHANGELOG if applicable

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- Open an issue for questions
- Email: contact@nelaia.ai

---

*Thank you for helping make TAYNI better!*
