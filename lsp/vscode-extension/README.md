# VS Code Extension for t27 Language Server

**Ring:** 071 (EPOCH-02 EXPAND)

## Features

### Language Server Support
- **Syntax Highlighting**: Comprehensive syntax highlighting for `.t27` files
  - Keywords: `test`, `invariant`, `bench`, `const`, `type`, `fn`, `import`, `let`, `return`, `if`, `else`, `match`
  - Types: `GF16`, `GF32`, `GF64`, `GF128`, `GF256`, `GF512`, `GF1024`, `TF3`, `TF16`, `Boolean`, `String`, `Number`, `Integer`, `Float`, `Option`, `Result`, `Vec`, `Box`, `Arc`, `Rc`, `BTreeSet`, `HashMap`
  - String literals: Quoted strings
  - Comments: Single-line and block comments
  - Identifiers: Function names, type names, variable names, constants

### LSP Capabilities
- ✅ **Completion**: Code completion with trigger characters `.`, `:`, `(`, `{`
- ✅ **Hover**: Hover tooltips showing type information and documentation
- ✅ **Go-to-Definition**: Navigate to symbol definitions
- ✅ **References**: Find all references to a symbol (including declarations)
- ✅ **Document Symbols**: Outline view for open documents (functions, types, constants)
- ✅ **Workspace Symbols**: Search across all `.t27` files in workspace
- ✅ **Diagnostics**: Real-time syntax errors and warnings
- ✅ **Semantic Tokens**: Full syntax highlighting
- 🚧 **Code Actions**: Quick fixes for diagnostics (stub, TODO: Implement)
- 🚧 **Signature Help**: Parameter hints for function calls (stub, TODO: Implement)

### Configuration
- **t27ServerPath**: Path to `t27-language-server` binary (default: `cargo run --package t27-language-server`)
- **maxProblems**: Maximum number of problems to report (default: 100)

### Commands
- **Open t27 Language Server Log**: Opens output channel for debugging server messages

## Installation

### Development
1. **Install dependencies:**
   ```bash
   cd lsp/vscode-extension
   npm install
   ```

2. **Package the extension:**
   ```bash
   npm run package
   ```

3. **Test the extension:**
   - Press F5 in VS Code
   - Select "Extensions: Install from VSIX..."
   - Select the generated `.vsix` file
   - Open a `.t27` file to test highlighting and LSP features

## File Structure
```
lsp/vscode-extension/
├── package.json              # Extension manifest
├── index.js                 # Extension entry point
├── language-t27-configuration.json  # Default configuration
└── syntaxes/
    └── t27.tmLanguage.json   # TextMate grammar for syntax highlighting
```

## Language Configuration

The extension supports `language-t27-configuration.json` with the following settings:

| Property | Type | Default | Description |
|-----------|------|---------|-------------|
| t27ServerPath | string | `cargo run --package t27-language-server` | Path to the t27 LSP server binary |
| maxProblems | number | 100 | Maximum number of problems to show in the Problems panel |

## Troubleshooting

### LSP Server Not Starting
- Check that the `t27-language-server` package is built: `cargo build --package t27-language-server`
- Verify the path to the server binary in configuration
- Check the output channel: Command Palette → "Open t27 Language Server Log"

### Syntax Highlighting Not Working
- Ensure the file has the `.t27` extension
- Check that VS Code has associated the file with the "t27" language
- Reload the window: Command Palette → "Developer: Reload Window"

### LSP Features Not Available
- Verify that the LSP server is running and the extension is activated
- Check the Output channel for errors
- Ensure the `.t27` file is recognized by the extension

## Development Notes

### LSP Communication
- The extension uses **stdio** for LSP communication with the server
- This is the recommended approach for local development and testing

### Performance
- Max problems is set to 100 by default to avoid overwhelming the Problems panel
- The server should respond within 100ms for most operations

### TextMate Grammar
- Uses regular expressions for pattern matching
- Scopes: `source.t27`
- Grammar files are easy to extend for new constructs

## Future Work

### Ring 072: Neovim Plugin (Next Ring)
- Create a similar extension structure for Neovim
- Neovim uses LSP client for language server communication
- Configuration in Neovim's LSP settings

### Ring 073-074: Performance Optimization
- Implement incremental parsing for large files
- Add caching for frequently accessed symbols
- Optimize semantic tokens computation

### Ring 075: VS Code Extension Polish
- Add code actions with actual implementations
- Implement signature help with real parameter documentation
- Add rich hover documentation with examples
- Add inlay hints for type annotations

---

**Status:** ✅ Complete (Base implementation)
**Next:** Rings 072-075 - Advanced LSP features and integration
