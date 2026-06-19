# TAYNI Language Support for VS Code

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://marketplace.visualstudio.com/items?itemName=nelaia.tayni)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Language support for **TAYNI** - the AI-first programming language designed for token-efficient code generation.

## Features

### Syntax Highlighting

Full support for TAYNI v1.0 (assembly-like) and v1.5 (modern) syntax:

```tayni
// v1.5 syntax
cap net:http

fn handle(req: Request) -> Response {
    let body = json_encode({"message": "Hello, TAYNI!"})
    Response::new(200, body)
}

fn main() -> i32 {
    http_server(8080, handle)
}
```

```tayni
// v1.0 syntax
LET x = 42
LET y = 10
ADD x y
PRT x
```

### IntelliSense

- **Auto-completion** for keywords, types, and capabilities
- **Hover information** with documentation
- **Go to definition** for variables and functions
- **Real-time diagnostics** and error checking

### Code Snippets

Quick templates for common patterns:

| Prefix | Description |
|--------|-------------|
| `fn` | Function definition |
| `main` | Main entry point |
| `cap` | Capability declaration |
| `httpserver` | HTTP server template |
| `tcpserver` | TCP server template |
| `if`, `ife` | Conditionals |
| `loop`, `while` | Loops |
| `struct` | Structure definition |

### Capabilities

TAYNI uses capability-based security. The extension highlights and validates:

- `net:tcp` - TCP networking
- `net:http` - HTTP client/server
- `fs:read` - File system read
- `fs:write` - File system write
- `time` - Time operations
- `env` - Environment variables
- `crypto` - Cryptographic operations

## Requirements

### Basic Features (No Dependencies)
- Syntax highlighting works out of the box

### Advanced Features (Optional)
For LSP features (diagnostics, completion, hover), install the TAYNI Language Server:

```bash
# From the TAYNI repository
cd tools/lsp
cargo build --release
# Add target/release/tayni-lsp to your PATH
```

Or set the path in settings:
```json
{
    "tayni.lsp.path": "/path/to/tayni-lsp"
}
```

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `tayni.lsp.enabled` | `true` | Enable Language Server |
| `tayni.lsp.path` | `""` | Custom path to tayni-lsp |
| `tayni.syntax.version` | `"auto"` | Syntax version (auto/v1.0/v1.5) |
| `tayni.diagnostics.enabled` | `true` | Enable real-time diagnostics |

## Commands

- **TAYNI: Restart Language Server** - Restart the LSP
- **TAYNI: Show Version** - Display extension version
- **TAYNI: Compile Current File** - Compile the active .tyn file

## File Associations

The extension automatically activates for:
- `.tyn` files
- `.tayni` files

## About TAYNI

TAYNI is an AI-first programming language that:

- **Reduces token consumption by 64%** compared to Python
- **Generates tiny executables** (10.5KB HTTP server vs 5.8MB in Go)
- **Has zero external dependencies** - standalone binaries
- **Enforces security** via compile-time capability checking

Learn more: [github.com/NELAIA-AI/tayni](https://github.com/NELAIA-AI/tayni)

## Contributing

Contributions are welcome! Please see the [TAYNI repository](https://github.com/NELAIA-AI/tayni) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**NELAIA Project** | [GitHub](https://github.com/NELAIA-AI/tayni) | [Documentation](https://github.com/NELAIA-AI/tayni/tree/main/docs)
