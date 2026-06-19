# TAYNI Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- ARM64 code generator with register allocation (`arm64_codegen.rs`)
- DWARF debug sections: aranges, str, pubnames, pubtypes, frame
- Vercel Edge Functions demo template
- Fastly Compute@Edge demo template
- AWS Lambda demo template
- 165 new tests (total: 263)

### Changed
- Updated README with accurate test count and project structure
- Updated ROADMAP with completed Phase 2 items

## [0.1.0] - 2026-06-19

### Added

#### Compiler Core
- Lexer with full v1.5 syntax support
- Parser generating typed AST
- Type checker with inference
- IR generation and optimization
- Multi-target code generation

#### Compilation Targets
- Windows PE x86-64 generator (`pe.rs`)
- Linux ELF x86-64 generator (`elf.rs`)
- Linux ELF ARM64 generator (`elf_arm64.rs`)
- WebAssembly generator (`wasm.rs`)
- WASI Preview 1 generator (`wasi.rs`)
- WASI Preview 2 generator (`wasi_p2.rs`)
- WASI HTTP generator (`wasi_http.rs`)

#### ARM64 Support
- Instruction encoder (`arm64.rs`)
  - Data processing (ADD, SUB, MUL, DIV, AND, OR, XOR)
  - Load/Store (LDR, STR, LDP, STP)
  - Branches (B, BL, BR, BLR, RET, B.cond, CBZ, CBNZ)
  - System (SVC, NOP)
  - Address generation (ADR, ADRP)
- Code generator (`arm64_codegen.rs`)
  - Register allocator
  - IR to machine code translation
  - Branch fixups

#### Debug Support
- DWARF generator (`dwarf.rs`)
  - `.debug_info` - Compilation units, types, functions
  - `.debug_abbrev` - Abbreviation tables
  - `.debug_line` - Line number information
  - `.debug_aranges` - Address ranges
  - `.debug_str` - String table
  - `.debug_pubnames` - Public names
  - `.debug_pubtypes` - Public types
  - `.debug_frame` - Call frame information

#### Standard Library
- JSON parser/encoder (`json.rs`) - RFC 8259 compliant
- HTTP/1.1 client (`http_client.rs`)
- Package manager (`pkg.rs`) - Semver, manifests, lockfiles

#### WASI Support
- Preview 1: fd_write, fd_read, proc_exit, args, environ
- Preview 2 Filesystem: open, read, write, close, stat, mkdir, remove
- Preview 2 Sockets: TCP (create, bind, listen, accept, connect)
- Preview 2 Sockets: UDP (create, bind, send, receive)
- WASI HTTP: incoming-handler, outgoing-handler

#### Tooling
- VS Code extension with syntax highlighting
- Language Server Protocol (LSP) implementation
- Diagnostics, hover, completion support

#### Documentation
- Technical architecture document
- arXiv paper draft
- 3 technical blog posts
- 32 example programs
- 5 serverless platform demos

#### Testing
- 263 tests passing
- 100% WebAssembly conformance (wasm-tools validated)

### Security
- Capability-based security model
- SECURITY.md with vulnerability reporting
- OWASP ASVS Level 1 compliance checklist
- Threat model documentation

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2026-06-19 | Initial release with multi-target support |

---

*TAYNI - AI-first programming language by NELAIA*
