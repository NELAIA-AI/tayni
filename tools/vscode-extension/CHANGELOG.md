# Changelog

All notable changes to the TAYNI VS Code extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-18

### Added
- Initial release of TAYNI language support for VS Code
- Syntax highlighting for TAYNI v1.0 and v1.5 syntax
- Language Server Protocol (LSP) integration
  - Real-time diagnostics and error checking
  - Hover information for keywords and operations
  - Auto-completion for keywords, types, and capabilities
  - Go to definition for variables and functions
- Code snippets for common patterns
  - Functions, structs, control flow
  - HTTP/TCP server templates
  - File I/O patterns
  - JSON encoding/decoding
- Language configuration
  - Comment toggling (// and /* */)
  - Bracket matching and auto-closing
  - Indentation rules
- File associations for `.tyn` and `.tayni` extensions

### Known Issues
- LSP requires `tayni-lsp` binary to be installed separately
- Some v1.5 syntax features may not have full completion support yet

## [Unreleased]

### Planned
- Integrated debugger support
- Code formatting
- Refactoring tools (rename, extract function)
- Test runner integration
- WASM preview panel
