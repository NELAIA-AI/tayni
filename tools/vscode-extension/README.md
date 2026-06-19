# TAYNI VS Code Extension

Language support for TAYNI - the AI-first programming language.

## Features

- **Syntax Highlighting** - Full support for TAYNI v1.0 and v1.5 syntax
- **Diagnostics** - Real-time error detection
- **Hover Information** - Documentation on hover for keywords and operations
- **Auto-completion** - Intelligent code completion
- **Go to Definition** - Navigate to variable and function definitions

## Installation

### From VSIX (Recommended)

1. Download the latest `.vsix` file from releases
2. In VS Code: Extensions → ... → Install from VSIX
3. Select the downloaded file

### From Source

```bash
cd tools/vscode-extension
npm install
npm run compile
# Then press F5 to launch Extension Development Host
```

## Building the LSP Server

The extension works best with the TAYNI Language Server:

```bash
cd tools/lsp
cargo build --release
# Copy target/release/tayni-lsp to your PATH
```

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `tayni.lsp.enabled` | `true` | Enable/disable the language server |
| `tayni.lsp.path` | `""` | Custom path to tayni-lsp executable |

## Syntax Examples

### TAYNI v1.0 (Assembly-like)

```tayni
LET x = 42
LET y = 10
ADD x y
PRT x
```

### TAYNI v1.5 (Modern)

```tayni
cap net:tcp

fn main() -> i32 {
    let message = "Hello, TAYNI!"
    print(message)
    0
}
```

## License

MIT License - See LICENSE file in repository root.
