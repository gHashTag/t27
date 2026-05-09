// Navigation service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{Location, Position, Range};

/// Navigation service
pub struct NavigationService;

impl NavigationService {
    /// Go to definition at a position
    pub fn goto_definition(doc: &Document, position: Position) -> Option<Location> {
        // Find symbol at position
        let symbol = Self::find_symbol_at_position(doc, position)?;

        // Look up definition
        Some(Location {
            uri: symbol.uri.clone(),
            range: symbol.range,
        })
    }

    /// Find all references to a symbol
    pub fn find_references(
        documents: &[Document],
        position: Position,
        include_declaration: bool,
    ) -> Vec<Location> {
        // First, find the symbol at position
        let first_doc = match documents.first() {
            Some(d) => d,
            None => return Vec::new(),
        };

        let symbol = match Self::find_symbol_at_position(first_doc, position) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut references = Vec::new();

        for doc in documents {
            // Find all references to this symbol in the document
            for found_symbol in Self::find_symbol_references_in_doc(doc, symbol) {
                if include_declaration || found_symbol.range != symbol.range {
                    references.push(Location {
                        uri: doc.uri.clone(),
                        range: found_symbol.range,
                    });
                }
            }
        }

        references
    }

    /// Find all implementations of a symbol
    pub fn find_implementations(
        documents: &[Document],
        position: Position,
    ) -> Vec<Location> {
        // Find symbol at position
        let first_doc = match documents.first() {
            Some(d) => d,
            None => return Vec::new(),
        };

        let _symbol = match Self::find_symbol_at_position(first_doc, position) {
            Some(s) => s,
            None => return Vec::new(),
        };

        // Find implementations (types that implement this, etc.)
        let mut implementations = Vec::new();

        for _doc in documents {
            // TODO: Check if found_symbol implements symbol
            // For now, return empty
        }

        implementations
    }

    /// Find type definition at position
    pub fn goto_type_definition(doc: &Document, position: Position) -> Option<Location> {
        // Find symbol at position
        let _symbol = Self::find_symbol_at_position(doc, position)?;

        // Find the type of the symbol
        // TODO: Implement type resolution
        None
    }

    fn find_symbol_at_position(doc: &Document, position: Position) -> Option<&crate::types::Symbol> {
        // Find the innermost symbol containing the position
        doc.symbols
            .iter()
            .find(|s| Self::position_in_range(position, &s.range))
            .or_else(|| Self::find_symbol_in_children_recursive(&doc.symbols, position))
    }

    fn find_symbol_in_children_recursive<'a>(
        symbols: &'a [crate::types::Symbol],
        position: Position,
    ) -> Option<&'a crate::types::Symbol> {
        for symbol in symbols {
            if Self::position_in_range(position, &symbol.range) {
                return Some(symbol);
            }
            if let Some(found) = Self::find_symbol_in_children_recursive(&symbol.children, position) {
                return Some(found);
            }
        }
        None
    }

    fn find_symbol_references_in_doc<'a>(
        doc: &'a Document,
        symbol: &'a crate::types::Symbol,
    ) -> Vec<&'a crate::types::Symbol> {
        let mut references = Vec::new();

        // Add the declaration itself
        references.push(symbol);

        // Find usages (TODO: implement actual reference finding)
        // This requires parsing the document to find identifier references

        references
    }

    fn position_in_range(
        position: Position,
        range: &Range,
    ) -> bool {
        position >= range.start && position <= range.end
    }
}
