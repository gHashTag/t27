// Symbol service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{DocumentSymbol, SymbolInformation, Url};

/// Symbol service
pub struct SymbolService;

impl SymbolService {
    /// Get document symbols (for outline view)
    pub fn document_symbols(doc: &Document) -> Vec<DocumentSymbol> {
        doc.symbols
            .iter()
            .map(|s| s.to_document_symbol())
            .collect()
    }

    /// Get workspace symbols (for fuzzy search)
    pub fn workspace_symbols(
        documents: &[Document],
        query: Option<String>,
    ) -> Vec<SymbolInformation> {
        let query = query.unwrap_or_default().to_lowercase();
        let is_empty = query.is_empty();

        let mut symbols = Vec::new();

        for doc in documents {
            for symbol in &doc.symbols {
                if is_empty || symbol.name.to_lowercase().contains(&query) {
                    symbols.push(symbol.to_symbol_information());
                }
            }
        }

        symbols
    }

    /// Filter symbols by kind
    pub fn filter_by_kind(
        documents: &[Document],
        kind: crate::types::SymbolKind,
    ) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();

        for doc in documents {
            for symbol in &doc.symbols {
                if symbol.kind == kind {
                    symbols.push(symbol.to_symbol_information());
                }
                // Also check children
                Self::filter_children_by_kind(symbol, kind, &mut symbols);
            }
        }

        symbols
    }

    fn filter_children_by_kind(
        symbol: &crate::types::Symbol,
        kind: crate::types::SymbolKind,
        results: &mut Vec<SymbolInformation>,
    ) {
        for child in &symbol.children {
            if child.kind == kind {
                results.push(child.to_symbol_information());
            }
            Self::filter_children_by_kind(child, kind, results);
        }
    }
}
