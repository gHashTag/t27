// Document Colors Service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{Color, ColorPresentation, ColorInformation, Range, ServerCapabilities,
};

/// Document colors service
pub struct DocumentColorsService;

impl DocumentColorsService {
    pub fn new() -> Self {
        Self
    }

    /// Get document colors for a range
    pub fn get_document_colors(doc: &Document, range: Range) -> Vec<ColorInformation> {
        let mut colors = Vec::new();

        // Add color information for symbols
        for symbol in &doc.symbols {
            if Self::range_intersects(&symbol.range, range) {
                if let Some(color) = Self::get_symbol_color(&symbol.kind) {
                    colors.push(ColorInformation {
                        range: symbol.range,
                        color: color,
                    });
                }
            }
        }

        colors
    }

    /// Get color for a symbol kind
    fn get_symbol_color(kind: &crate::types::SymbolKind) -> Color {
        match kind {
            // Keywords - blue
            crate::types::SymbolKind::Keyword => Color {
                red: 0x42A5F5,
                green: 0x1E1E1,
                blue: 0x61BFFFF,
                alpha: 1.0,
            },
            // Functions - purple
            crate::types::SymbolKind::Function => Color {
                red: 0x6A5D1,
                green: 0xBDB76B,
                blue: 0x5697D9,
                alpha: 1.0,
            },
            // Variables/Constants - orange
            crate::types::SymbolKind::Variable | crate::types::SymbolKind::Constant => Color {
                red: 0xCE9178,
                green: 0x50FA7B,
                blue: 0x2478F8,
                alpha: 1.0,
            },
            // Types - yellow
            crate::types::SymbolKind::Type => Color {
                red: 0xCC7832,
                green: 0x19A979,
                blue: 0x1FA668,
                alpha: 1.0,
            },
            // Module - cyan
            crate::types::SymbolKind::Module => Color {
                red: 0x8F9FA2,
                green: 0xA1B4C7,
                blue: 0xD98035,
                alpha: 1.0,
            },
            _ => Color {
                red: 0x00,
                green: 0x00,
                blue: 0x00,
                alpha: 1.0,
            },
        }
    }

    /// Check if two ranges intersect
    fn range_intersects(range1: &Range, range2: &Range) -> bool {
        !(range1.end < range2.start || range1.start > range2.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_get_symbol_color() {
        let function_color = DocumentColorsService::get_symbol_color(&crate::types::SymbolKind::Function);
        assert_eq!(function_color.red, 0x6A5D1);
        assert_eq!(function_color.green, 0xBDB76B);
        assert_eq!(function_color.blue, 0x5697D9);
    }

    #[test]
    fn test_range_intersects() {
        let range1 = Range {
            start: Position::new(0, 5),
            end: Position::new(0, 10),
        };
        let range2 = Range {
            start: Position::new(0, 8),
            end: Position::new(0, 15),
        };

        assert!(DocumentColorsService::range_intersects(&range1, &range2));
    }
}
