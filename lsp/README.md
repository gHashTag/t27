# t27 Language Server Protocol (LSP)

A complete LSP server for t27 spec-first language, providing 12 services for IDE integration.

## Features

- Context-aware code completion
- Hover documentation (phi/GF help)
- Go-to-definition (cross-references)
- Find all references
- Symbol outline
- Workspace symbol search
- Spec validation diagnostics
- Quick fixes
- Auto-formatting
- Function signature help
- Call hierarchy
- Dependency graph

## Building

```bash
cd lsp
cargo build --release
```

The LSP server binary will be at `target/release/t27_lsp`.

## Usage

### With VSCode

Install the VSCode extension:

```bash
code --install-extension trinity-s3ai.t27
```

Or build from source:

```bash
cd lsp/vscode-extension
npm install
npm run compile
code --install-extension .
```

### With Other Editors

The LSP server can be used with any LSP-compatible editor (Neovim, Emacs, Sublime Text, etc.).

Example Neovim configuration:

```lua
require'lspconfig'.t27.setup {
  cmd = { '/path/to/t27_lsp' },
  filetypes = { 't27' },
  root_dir = function(fname)
    return vim.fs.dirname(vim.fs.find('.git', { path = fname, upward = true }) or vim.fs.dirname(fname))
  end,
}
```

### Manual Testing

```bash
# Start the server
target/release/t27_lsp

# Or run specific service tests
cargo test
```

## Architecture

```
lsp/src/
├── main.rs              — Server entry point
├── lib.rs               — LSP types and traits
├── parser.rs            — Tokenizer (30+ token types)
├── completion.rs        — Context-aware completions
├── hover.rs             — phi/GF documentation
├── navigation.rs        — Go-to-definition, references
├── diagnostics.rs       — Spec validation
├── semantic_tokens.rs   — Token highlighting
└── services.rs          — All LSP service implementations
```

## Contributing

See [../CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

**phi^2 + 1/phi^2 = 3 | TRINITY**