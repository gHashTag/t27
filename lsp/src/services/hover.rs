// Hover service for t27 Language Server

use crate::types::Document;
use crate::types::symbol::SymbolKind;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

/// Hover service
pub struct HoverService;

impl HoverService {
    /// Generate hover information for a position in a document
    pub fn hover(doc: &Document, position: Position) -> Option<Hover> {
        // Find symbol at position
        let symbol = Self::find_symbol_at_position(doc, position)?;

        // Build hover content
        let content = Self::build_hover_content(&symbol);

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(symbol.range),
        })
    }

    fn find_symbol_at_position(doc: &Document, position: Position) -> Option<&crate::types::Symbol> {
        // Find the innermost symbol containing the position
        doc.symbols
            .iter()
            .find(|s| Self::position_in_range(position, &s.range))
            .or_else(|| {
                // Check children
                for symbol in &doc.symbols {
                    if let Some(child) = Self::find_symbol_in_children(symbol, position) {
                        return Some(child);
                    }
                }
                None
            })
    }

    fn find_symbol_in_children<'a>(
        symbol: &'a crate::types::Symbol,
        position: Position,
    ) -> Option<&'a crate::types::Symbol> {
        for child in &symbol.children {
            if Self::position_in_range(position, &child.range) {
                return Some(child);
            }
            if let Some(found) = Self::find_symbol_in_children(child, position) {
                return Some(found);
            }
        }
        None
    }

    fn position_in_range(
        position: Position,
        range: &tower_lsp::lsp_types::Range,
    ) -> bool {
        position >= range.start && position <= range.end
    }

    fn build_hover_content(symbol: &crate::types::Symbol) -> String {
        let mut content = String::new();

        // Add symbol kind
        content.push_str(Self::symbol_kind_to_emoji(symbol.kind));
        content.push(' ');
        content.push_str(&symbol.name);
        content.push('\n');

        // Add detail if available
        if let Some(detail) = &symbol.detail {
            content.push_str("\n```t27\n");
            content.push_str(detail);
            content.push_str("\n```\n");
        }

        // Add documentation if available
        if let Some(doc) = &symbol.documentation {
            content.push_str("\n---\n\n");
            content.push_str(doc);
        }

        content
    }

    fn symbol_kind_to_emoji(kind: SymbolKind) -> &'static str {
        match kind {
            SymbolKind::Module => "📦",
            SymbolKind::Function => "🔧",
            SymbolKind::Variable => "📝",
            SymbolKind::Constant => "🔒",
            SymbolKind::Type => "🏗️",
            SymbolKind::Test => "✅",
            SymbolKind::Invariant => "🔐",
            SymbolKind::Bench => "⏱️",
            SymbolKind::Import => "📥",
            SymbolKind::Unknown => "❓",
        }
    }
}
