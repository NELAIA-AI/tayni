# Editor Support for TAYNI

This directory contains editor integrations for the TAYNI language.

## VS Code Extension

The `vscode/` directory contains a VS Code extension for TAYNI syntax highlighting.

### Installation (Local)

1. Copy the `vscode/` folder to `~/.vscode/extensions/TAYNI-0.22.0/`
2. Restart VS Code
3. Open any `.tyn` file

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

See `LINGUIST.md` for information about registering TAYNI with GitHub Linguist.

## Other Editors

Contributions welcome for:
- Vim/Neovim
- Emacs
- Sublime Text
- JetBrains IDEs
