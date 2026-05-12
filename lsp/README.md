# t27 Language Server

**Status:** ✅ Production Ready (Rings 059-118)

## Features

### Language Server Protocol (LSP)
- **Completion**: Code completion with trigger characters
- **Hover**: Hover tooltips showing type information and documentation
- **Go-to-Definition**: Navigate to symbol definitions
- **Find References**: Find all references to a symbol (including declarations)
- **Document Symbols**: Outline view for open documents (functions, types, constants)
- **Workspace Symbols**: Search across all `.t27` files in workspace
- **Diagnostics**: Real-time syntax errors and warnings
- **Semantic Tokens**: Full syntax highlighting
- **Code Actions**: Quick fixes for diagnostics
- **Signature Help**: Parameter hints for function calls

### Supported File Types
- `.t27` - Main t27 language file
- `.tri` - Trinity language files (trios-cli, trios-kg, trios-crypto)

## Installation

### Build
```bash
cd lsp
cargo build --release
```

The `t27-language-server` binary will be built in `target/release/t27-language-server`.

### Usage with VS Code

1. Install the extension:
   ```bash
   code --install-extension t27-lsp
   ```

2. Open a `.t27` file to activate LSP features

### Usage with Neovim

1. Install the Neovim plugin:
   ```bash
   cd trios-kg/neovim-plugin
   npm install
   ```

2. Add to your Neovim configuration:
   ```lua
   require('neovim-plugin').setup({
     cmd = { "t27-language-server" }
   })
   ```

### Usage with Other Editors

Any editor that supports the Language Server Protocol can use `t27-language-server`. You'll need to configure:
- The LSP server command (e.g., `t27-language-server -- --stdio`)
- File associations for `.t27` files
- `languageId: "t27"`

## Configuration

The LSP server can be configured in multiple ways:

### Workspace Configuration (`.t27-lsp.json`)

Create a `.t27-lsp.json` file in your project root:

```json
{
  "server": {
    "path": "cargo run --package t27-language-server",
    "args": ["--stdio"]
  },
  "maxWorkspaceFiles": 1000,
  "diagnostics": {
    "enable_type_errors": true,
    "enable_seal_errors": true,
    "enable_warnings": true
  },
  "completion": {
    "enable_snippets": true,
    "trigger_characters": [".", "(", "{", " ", ":"]
  },
  "semanticTokens": {
    "enabled": true
  },
  "workspace": {
    "symbolSearch": {
      "enabled": true,
      "max_items": 100
    }
  }
}
```

### Environment Variables

- `T27_LOG_LEVEL`: Logging level (error, warn, info, debug, trace)
- `T27_MAX_PROBLEMS`: Maximum number of problems to show (default: 100)
- `T27_SERVER_PATH`: Path to the t27 LSP server binary

## Development

### Running Tests
```bash
cd lsp
cargo test
```

### Running the LSP Server

For development, you can run the LSP server in stdio mode:

```bash
cargo run --bin t27-language-server -- --stdio
```

Or with debug logging:

```bash
RUST_LOG=t27=debug cargo run --bin t27-language-server -- --stdio
```

### Adding a New Feature

1. Implement the service in `src/services/`
2. Add the service module to `src/services/mod.rs`
3. Implement the LSP method in `src/backend/mod.rs`
4. Register the capability in `initialize()` method
5. Add tests for the new service

### Architecture

```
lsp/
├── Cargo.toml              # LSP server manifest
├── src/
│   ├── main.rs               # Binary entry point
│   ├── backend/              # LSP implementation
│   │   ├── mod.rs           # Backend struct and LanguageServer trait
│   │   ├── parser/           # t27 parser (Tri)
│   │   └── utils/
│   └── services/            # LSP feature services
│       ├── completion.rs        # Code completion
│       ├── hover.rs             # Hover information
│       ├── navigation.rs        # Go-to-definition, find references
│       ├── symbols.rs           # Document/workspace symbols
│       ├── tokens.rs             # Semantic tokens
│       ├── code_actions.rs     # Quick fixes
│       ├── signature_help.rs   # Parameter hints
│       ├── document_colors.rs # Document syntax highlighting
│       └── formatting.rs        # Document formatting
└── types/                    # Shared types
    ├── document.rs             # Document type
    ├── symbol.rs               # Symbol type
    └── ...
```

## Performance

The LSP server is optimized for fast response times:
- **Target**: < 100ms for most operations
- **Semantic Tokens**: Full document processing with caching
- **Workspace Symbols**: Lazy loading of all documents
- **Diagnostics**: Incremental updates

## Troubleshooting

### LSP Server Not Starting

Check that:
1. The LSP server binary exists: `cargo build --release`
2. The path to the server is correctly configured in your editor
3. The file associations are set for `.t27` files

### Features Not Available

If certain LSP features aren't available in your editor, check:
1. The editor supports LSP
2. The LSP capabilities are properly registered
3. The file is recognized as a `.t27` file

### Performance Issues

If you experience slow responses:
1. Disable semantic tokens in `.t27-lsp.json`
2. Reduce `maxWorkspaceFiles` limit
3. Check system resources

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

[MIT](../LICENSE)

---

**Ring Numbers:** Rings 059-118 (EPOCH-01: Complete, EPOCH-02: In Progress)
