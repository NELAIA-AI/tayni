# Contributing to NELAIA

Thank you for your interest in contributing to NELAIA.

## How to Contribute

### Reporting Issues

- Use the issue tracker to report bugs
- Include NELAIA version (`nelaia-c --version`)
- Include your OS and architecture
- Provide minimal reproduction code

### Code Contributions

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit with clear messages
6. Push to your fork
7. Open a Pull Request

### Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality

### NELAIA Language Contributions

- Follow the syntax defined in `docs/NELAIA-REFERENCE-v0.22.md`
- Add examples to `docs/NELAIA-EXAMPLES-v0.22.md`
- Update training data in `docs/NELAIA-TRAINING-DATA.jsonl`

## Development Setup

```bash
# Clone
git clone https://github.com/NELAIA-AI/nelaia.git
cd nelaia

# Build
cargo build

# Test
cargo test

# Run
./target/debug/nelaia-c --help
```

## Questions?

Open an issue with the "question" label.
