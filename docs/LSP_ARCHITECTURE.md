# LSP Server Architecture — t27 Language Server

**Ring:** 059 (EPOCH-02 EXPAND)
**Primary Agent:** C (Compiler)
**Status:** Design Phase
**Last Updated:** 2026-05-10

---

## 1. Executive Summary

This document defines the architecture for the t27 Language Server Protocol (LSP) implementation. The LSP server will provide IDE support for `.t27` specification files including syntax highlighting, navigation, diagnostics, and code completion.

**Target Features:**
- Parse-based diagnostics (syntax errors)
- Go-to-definition for symbols
- Find-references across files
- Hover documentation
- Code completion
- Semantic tokens (syntax highlighting)
- Document symbols (outline)
- Workspace symbols (search)

**Technology Stack:**
- **Language:** Rust (matches bootstrap compiler)
- **LSP Framework:** `tower-lsp` (async, type-safe)
- **Parser:** Reuse `t27c` parser from bootstrap/
- **Serialization:** `serde` + `serde_json`

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    LSP Client (Editor)                      │
│              (VS Code, Neovim, etc.)                        │
└──────────────────────────┬──────────────────────────────────┘
                           │ JSON-RPC 2.0
                           │ stdin/stdout or TCP
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   t27-language-server                        │
├─────────────────────────────────────────────────────────────┤
│  LSP Layer (tower-lsp)                                       │
│  ├─ Request Router                                           │
│  ├─ Response Builder                                         │
│  └─ Error Handler                                            │
├─────────────────────────────────────────────────────────────┤
│  Service Layer                                               │
│  ├─ DocumentManager (open files, version tracking)          │
│  ├─ SymbolTable (cross-file symbol resolution)               │
│  ├─ DiagnosticsService (error/warning generation)           │
│  ├─ NavigationService (go-to-def, find-references)           │
│  ├─ CompletionService (code completion)                      │
│  ├─ HoverService (tooltip documentation)                    │
│  ├─ SemanticTokensService (syntax highlighting)             │
│  └─ SymbolService (document/workspace symbols)               │
├─────────────────────────────────────────────────────────────┤
│  Parser Layer (from t27c)                                    │
│  ├─ Lexer (tokenization)                                     │
│  ├─ Parser (AST construction)                               │
│  ├─ TypeChecker (type inference)                            │
│  └─ SealVerifier (seal validation)                          │
├─────────────────────────────────────────────────────────────┤
│  File System Layer                                           │
│  ├─ WorkspaceScanner (find .t27 files)                       │
│  ├─ FileReader (watch for changes)                          │
│  └─ ConfigLoader (t27c config)                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Module Structure

```
lsp/
├── Cargo.toml              # LSP server binary crate
├── src/
│   ├── main.rs             # Entry point, server setup
│   ├── lib.rs              # Library exports
│   ├── backend/
│   │   ├── mod.rs          # Backend trait & implementation
│   │   └── parser.rs       # Interface to t27c parser
│   ├── server.rs           # LspService implementation
│   ├── services/
│   │   ├── mod.rs          # Service module
│   │   ├── document.rs     # Document management
│   │   ├── symbols.rs      # Symbol table
│   │   ├── diagnostics.rs  # Diagnostics generation
│   │   ├── navigation.rs   # Go-to-def, find-references
│   │   ├── completion.rs   # Code completion
│   │   ├── hover.rs        # Hover documentation
│   │   ├── tokens.rs       # Semantic tokens
│   │   └── symbol_service.rs # Document/workspace symbols
│   ├── types/
│   │   ├── mod.rs          # Type definitions
│   │   ├── position.rs     # LSP position conversions
│   │   ├── document.rs     # Document representation
│   │   └── symbol.rs       # Symbol representation
│   └── config.rs           # Server configuration
└── tests/
    └── integration.rs      # LSP integration tests
```

---

## 4. LSP Feature Mapping

### 4.1 Server Capabilities

| Feature | Priority | Status | Notes |
|---------|----------|--------|-------|
| **textDocumentSync** | P0 | Planned | Full/incremental sync |
| **completion** | P0 | Planned | Trigger characters: `.`, `:`, `(` |
| **hover** | P0 | Planned | Type info, doc comments |
| **definition** | P0 | Planned | Go-to-definition |
| **references** | P0 | Planned | Find all references |
| **documentSymbol** | P1 | Planned | Outline view |
| **workspaceSymbol** | P1 | Planned | Fuzzy symbol search |
| **semanticTokens** | P1 | Planned | Syntax highlighting |
| **diagnostics** | P0 | Planned | Parse errors, warnings |
| **codeAction** | P2 | Planned | Quick fixes |
| **signatureHelp** | P2 | Planned | Parameter hints |
| **inlayHint** | P2 | Planned | Type hints inline |
| **rename** | P2 | Planned | Symbol renaming |

### 4.2 Feature Details

#### textDocumentSync
- **Mode:** Incremental
- **Will Save:** Yes
- **Will Save Wait Until:** Yes
- **Save Include Text:** Yes

#### completion
- **Trigger Characters:** `.`, `:`, `(`, `{`
- **Completion Item:**
  - `label`: Symbol name
  - `kind`: Symbol type (function, variable, const, etc.)
  - `detail`: Type signature
  - `documentation`: Doc string from spec
  - `insertText`: Snippet support
  - `sortText`: Relevance score

#### hover
- **Content:** Markdown formatted
- - Type information
- - Documentation from `doc {}` blocks
- - Seal status if applicable

#### definition
- **Result:** Location (URI, Range)
- Supports:
  - Module definitions
  - Type declarations
  - Function signatures
  - Constant definitions
  - Spec imports

#### references
- **Context:** Declaration and usages
- **Result:** Array of Locations

#### documentSymbol
- **Hierarchy:** Module → Functions → Variables
- **Kinds:**
  - `Module` for modules
  - `Function` for `fn {}` blocks
  - `Struct` for `type {}` declarations
  - `Constant` for `const {}`
  - `Variable` for `let {}`

#### semanticTokens
- **Legend:**
  | TokenType | Legend | Color (VS Code Dark+) |
  |-----------|--------|----------------------|
  | Function | `function` | #DCDCAA |
  | Variable | `variable` | #9CDCFE |
  | Constant | `constant` | #4FC1FF |
  | Type | `type` | #4EC9B0 |
  | Keyword | `keyword` | #569CD6 |
  | String | `string` | #CE9178 |
  | Number | `number` | #B5CEA8 |
  | Comment | `comment` | #6A9955 |
  | Operator | `operator` | #D4D4D4 |
  | Module | `namespace` | #4EC9B0 |

---

## 5. Parser Integration

### 5.1 Reusing t27c Parser

The LSP server will reuse the existing parser from `bootstrap/`:

```rust
// backend/parser.rs
use t27c::parser::{Parser, ParseResult, AST};
use t27c::lexer::{Lexer, Token, TokenKind};

pub struct T27Parser;

impl T27Parser {
    pub fn parse_text(text: &str) -> ParseResult<AST> {
        let lexer = Lexer::new(text);
        let parser = Parser::new(lexer);
        parser.parse()
    }

    pub fn parse_document(uri: &Url) -> ParseResult<AST> {
        let text = std::fs::read_to_string(uri.path())?;
        Self::parse_text(&text)
    }
}
```

### 5.2 AST to LSP Mapping

```rust
// types/symbol.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Module,
    Function,
    Variable,
    Constant,
    Type,
    Test,
    Invariant,
    Bench,
    Import,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub uri: Url,
    pub range: Range,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub children: Vec<Symbol>,
}
```

---

## 6. Document Management

### 6.1 Document Representation

```rust
// types/document.rs
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub version: i32,
    pub text: String,
    pub line_offsets: Vec<usize>,
    pub ast: Option<AST>,
    pub symbols: Vec<Symbol>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Document {
    pub fn new(uri: Url, text: String) -> Self {
        // Calculate line offsets for position conversion
        let line_offsets = text
            .lines()
            .scan(0, |offset, line| {
                let start = *offset;
                *offset += line.len() + 1; // +1 for newline
                Some(start)
            })
            .collect();

        Self {
            uri,
            version: 0,
            text,
            line_offsets,
            ast: None,
            symbols: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn update(&mut self, changes: &[TextDocumentContentChangeEvent]) {
        // Apply incremental changes
        for change in changes {
            if let Some(range) = &change.range {
                self.apply_change_range(range, change.text.as_deref().unwrap_or(""));
            } else {
                self.text = change.text.clone().unwrap_or_default();
            }
        }
        // Recalculate line offsets
        self.recalculate_line_offsets();
        // Re-parse
        self.parse();
    }

    fn parse(&mut self) {
        // Use t27c parser
        match T27Parser::parse_text(&self.text) {
            Ok(ast) => {
                self.ast = Some(ast.clone());
                self.symbols = SymbolExtractor::extract(&ast);
                self.diagnostics = DiagnosticsGenerator::generate(&ast);
            }
            Err(e) => {
                self.diagnostics = vec![Diagnostic::from_parse_error(&e)];
            }
        }
    }
}
```

### 6.2 Document Manager

```rust
// services/document.rs
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct DocumentManager {
    documents: RwLock<HashMap<Url, Document>>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }

    pub async fn open(&self, uri: Url, text: String) {
        let mut docs = self.documents.write().await;
        docs.insert(uri, Document::new(uri, text));
    }

    pub async fn update(&self, uri: &Url, changes: &[TextDocumentContentChangeEvent], version: i32) {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(uri) {
            doc.version = version;
            doc.update(changes);
        }
    }

    pub async fn get(&self, uri: &Url) -> Option<Document> {
        self.documents.read().await.get(uri).cloned()
    }

    pub async fn get_symbols(&self, uri: &Url) -> Vec<Symbol> {
        self.documents
            .read()
            .await
            .get(uri)
            .map(|d| d.symbols.clone())
            .unwrap_or_default()
    }
}
```

---

## 7. Services

### 7.1 Diagnostics Service

```rust
// services/diagnostics.rs
pub struct DiagnosticsService;

impl DiagnosticsService {
    pub fn generate_from_ast(ast: &AST) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Type errors
        diagnostics.extend(TypeChecker::check(ast).into_iter().map(Diagnostic::from_type_error));

        // Seal errors
        diagnostics.extend(SealVerifier::verify(ast).into_iter().map(Diagnostic::from_seal_error));

        // Invariant violations
        diagnostics.extend(InvariantChecker::check(ast).into_iter().map(Diagnostic::from_invariant_error));

        diagnostics
    }

    pub fn publish(&self, docs: &DocumentManager, sender: &Sender<PublishDiagnosticsParams>) {
        // Publish diagnostics for all open documents
    }
}
```

### 7.2 Navigation Service

```rust
// services/navigation.rs
pub struct NavigationService;

impl NavigationService {
    pub fn goto_definition(&self, doc: &Document, position: Position) -> Option<Location> {
        // Find symbol at position
        let symbol = self.find_symbol_at_position(doc, position)?;

        // Look up definition
        let definition = self.find_definition(&symbol)?;

        Some(Location {
            uri: definition.uri,
            range: definition.range,
        })
    }

    pub fn find_references(&self, docs: &DocumentManager, symbol: &Symbol) -> Vec<Location> {
        // Search all documents for references
        let mut references = Vec::new();

        for doc in docs.all_documents() {
            if let Some(asts) = &doc.ast {
                references.extend(ReferenceFinder::find(asts, symbol));
            }
        }

        references
    }
}
```

### 7.3 Completion Service

```rust
// services/completion.rs
pub struct CompletionService;

impl CompletionService {
    pub fn complete(&self, doc: &Document, position: Position, trigger_char: Option<char>) -> CompletionList {
        let context = CompletionContext::new(doc, position, trigger_char);

        let mut items = Vec::new();

        // Add keyword completions
        items.extend(KeywordCompleter::complete(&context));

        // Add symbol completions
        items.extend(SymbolCompleter::complete(&context));

        // Add snippet completions
        items.extend(SnippetCompleter::complete(&context));

        CompletionList {
            is_incomplete: false,
            items,
        }
    }
}
```

---

## 8. Configuration

### 8.1 Server Options

```rust
// config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Path to t27c binary for fallback parsing
    pub t27c_path: Option<String>,

    /// Maximum number of workspace files to index
    pub max_workspace_files: Option<usize>,

    /// Enable experimental features
    pub experimental: bool,

    /// Diagnostic severity levels
    pub diagnostics: DiagnosticConfig,

    /// Completion options
    pub completion: CompletionConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticConfig {
    pub enable_type_errors: bool,
    pub enable_seal_errors: bool,
    pub enable_warnings: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionConfig {
    pub enable_snippets: bool,
    pub show_documentation: bool,
    pub trigger_characters: Vec<char>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            t27c_path: None,
            max_workspace_files: Some(1000),
            experimental: false,
            diagnostics: DiagnosticConfig {
                enable_type_errors: true,
                enable_seal_errors: true,
                enable_warnings: true,
            },
            completion: CompletionConfig {
                enable_snippets: true,
                show_documentation: true,
                trigger_characters: vec!['.', ':', '(', '{'],
            },
        }
    }
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

- Parser integration
- Symbol extraction
- Diagnostic generation
- Position conversion
- Document update logic

### 9.2 Integration Tests

- LSP protocol compliance
- Client-server communication
- File watching
- Workspace scanning

### 9.3 E2E Tests

```rust
// tests/e2e.rs
#[tokio::test]
async fn test_goto_definition() {
    let mut client = TestClient::new().await;

    client.open_file("test.t27", r#"
        const PHI: GF16 = 1.618;

        fn test() {
            let x = PHI;
        }
    "#).await;

    let location = client
        .goto_definition(Position::new(2, 16)) // PHI reference
        .await
        .expect("definition not found");

    assert_eq!(location.range.start.line, 0); // const PHI declaration
}
```

---

## 10. Implementation Phases

### Phase 1: Foundation (Rings 059-065)
- Ring 059: Architecture design (this document)
- Ring 060: Project setup, basic server scaffolding
- Ring 061: Document management (open, update, close)
- Ring 062: Parser integration
- Ring 063: Diagnostics (parse errors)
- Ring 064: Go-to-definition
- Ring 065: Basic LSP completion

### Phase 2: Core Features (Rings 066-070)
- Ring 066: Hover documentation
- Ring 067: Find-references
- Ring 068: Document symbols
- Ring 069: Workspace symbols
- Ring 070: Semantic tokens

### Phase 3: Advanced Features (Rings 071-075)
- Ring 071: Code actions
- Ring 072: Signature help
- Ring 073: Inlay hints
- Ring 074: Performance optimization
- Ring 075: VS Code extension

---

## 11. Dependencies

```toml
[dependencies]
tower-lsp = "0.20"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"

# Internal dependencies
t27c-parser = { path = "../bootstrap/compiler" }

[dev-dependencies]
tokio-test = "0.4"
```

---

## 12. Open Questions

1. **Parser Reuse:** Should we link directly to `bootstrap/compiler` or spawn `t27c` as a subprocess?
   - **Recommendation:** Direct linking for performance, subprocess fallback for isolation

2. **Incremental Parsing:** Should we implement incremental parsing for large files?
   - **Recommendation:** Phase 2 optimization, not MVP

3. **Multi-root Workspaces:** Should we support LSP multi-root workspaces?
   - **Recommendation:** Yes, for monorepo support

4. **Configuration File:** Should we support `.t27-lsp.json` or use VS Code settings?
   - **Recommendation:** VS Code settings first, config file later

---

## 13. References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp Documentation](https://docs.rs/tower-lsp/)
- [VS Code Language Server Extension Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [Neovim LSP Client Documentation](https://neovim.io/doc/user/lsp.html)

---

**φ² + 1/φ² = 3 | TRINITY**

**Document Created:** 2026-05-10
**Ring:** 059 (EPOCH-02 EXPAND)
**Next:** Ring 060 — Project setup and server scaffolding
