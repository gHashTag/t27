// Symbol representation for t27 Language Server

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{
    DocumentSymbol, Location, Range, SymbolInformation, SymbolKind as LspSymbolKind,
    Url,
};

/// t27 Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl From<SymbolKind> for LspSymbolKind {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Module => LspSymbolKind::MODULE,
            SymbolKind::Function => LspSymbolKind::FUNCTION,
            SymbolKind::Variable => LspSymbolKind::VARIABLE,
            SymbolKind::Constant => LspSymbolKind::CONSTANT,
            SymbolKind::Type => LspSymbolKind::STRUCT,
            SymbolKind::Test => LspSymbolKind::FUNCTION,
            SymbolKind::Invariant => LspSymbolKind::INTERFACE,
            SymbolKind::Bench => LspSymbolKind::FUNCTION,
            SymbolKind::Import => LspSymbolKind::NAMESPACE,
            SymbolKind::Unknown => LspSymbolKind::VARIABLE,
        }
    }
}

/// Symbol representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub uri: Url,
    pub range: Range,
    pub selection_range: Range,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub children: Vec<Symbol>,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, uri: Url, range: Range) -> Self {
        Self {
            name,
            kind,
            uri,
            selection_range: range,
            range,
            detail: None,
            documentation: None,
            children: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn with_documentation(mut self, doc: String) -> Self {
        self.documentation = Some(doc);
        self
    }

    pub fn with_children(mut self, children: Vec<Symbol>) -> Self {
        self.children = children;
        self
    }

    /// Convert to LSP DocumentSymbol (for outline view)
    pub fn to_document_symbol(&self) -> DocumentSymbol {
        DocumentSymbol {
            name: self.name.clone(),
            detail: self.detail.clone(),
            kind: self.kind.into(),
            tags: None,
            deprecated: Some(false),
            range: self.range,
            selection_range: self.selection_range,
            children: Some(
                self.children
                    .iter()
                    .map(|s| s.to_document_symbol())
                    .collect(),
            ),
        }
    }

    /// Convert to LSP SymbolInformation (for workspace symbols)
    pub fn to_symbol_information(&self) -> SymbolInformation {
        SymbolInformation {
            name: self.name.clone(),
            kind: self.kind.into(),
            tags: None,
            deprecated: Some(false),
            location: Location {
                uri: self.uri.clone(),
                range: self.range,
            },
            container_name: None,
        }
    }
}
