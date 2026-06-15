# Editor Support for NELAIA

This directory contains editor integrations for the NELAIA language.

## VS Code Extension

The `vscode/` directory contains a VS Code extension for NELAIA syntax highlighting.

### Installation (Local)

1. Copy the `vscode/` folder to `~/.vscode/extensions/nelaia-0.22.0/`
2. Restart VS Code
3. Open any `.nela` file

### Features

- Syntax highlighting for:
  - Comments (`--`)
  - Strings
  - Numbers (decimal, hex)
  - Operators (ADD, SUB, MUL, etc.)
  - Capabilities (HTTP, SQL, JSON)
  - Node references (`.name`)
  - Flow operators (`>`, `>>`)

## GitHub Linguist

See `LINGUIST.md` for information about registering NELAIA with GitHub Linguist.

## Other Editors

Contributions welcome for:
- Vim/Neovim
- Emacs
- Sublime Text
- JetBrains IDEs
