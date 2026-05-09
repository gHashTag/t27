# t27 Language Server

Language Server Protocol (LSP) implementation for t27 specification files.

## Features

- ✅ Syntax highlighting (semantic tokens)
- ✅ Go-to-definition
- ✅ Find references
- ✅ Document symbols (outline)
- ✅ Workspace symbols
- ✅ Hover documentation
- ✅ Code completion
- ✅ Diagnostics (parse errors, type errors, seal errors)

## Installation

### From Source

```bash
cd lsp
cargo build --release
```

The binary will be available at `target/release/t27-language-server`.

## Configuration

Create a `.t27-lsp.json` file in your workspace root:

```json
{
  "max_workspace_files": 1000,
  "experimental": false,
  "diagnostics": {
    "enable_type_errors": true,
    "enable_seal_errors": true,
    "enable_warnings": true,
    "enable_semantic_errors": true
  },
  "completion": {
    "enable_snippets": true,
    "show_documentation": true,
    "trigger_characters": [".", ":", "(", "{"],
    "max_items": 100
  },
  "semantic_tokens": {
    "enabled": true,
    "modules": true,
    "functions": true,
    "types": true,
    "constants": true,
    "variables": true
  }
}
```

## Editor Integration

### VS Code

See `vscode-t27/` extension directory.

### Neovim

```lua
require('lspconfig').t27_language_server.setup {
  cmd = { "t27-language-server" },
  filetypes = { "t27" },
  root_dir = require('lspconfig').util.root_pattern(".t27-lsp.json", ".git"),
}
```

### Vim/Neovim (with coc.nvim)

```vim
" coc-settings.json
{
  "languageserver": {
    "t27": {
      "command": "t27-language-server",
      "filetypes": ["t27"],
      "rootPatterns": [".t27-lsp.json", ".git"]
    }
  }
}
```

## Development

### Running Tests

```bash
cargo test
```

### LSP Testing

For LSP testing, use `vscode-languageserver-node`'s test framework:

```bash
npm test
```

## Architecture

See `docs/LSP_ARCHITECTURE.md` for detailed architecture documentation.

## License

MIT
