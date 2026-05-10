// Signature help service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{
    ParameterInformation, Position, Range, SignatureHelp, SignatureInformation,
};

/// Signature help service
pub struct SignatureHelpService;

impl SignatureHelpService {
    /// Get signature help at a position
    pub fn signature_help(doc: &Document, position: Position) -> Option<SignatureHelp> {
        // Find function call at position
        let (function_name, active_parameter) = Self::find_function_call_at_position(doc, position)?;

        // Get signature information for the function
        let signature_info = Self::get_signature_info(&function_name)?;

        Some(SignatureHelp {
            signatures: vec![signature_info],
            active_signature: Some(0),
            active_parameter: Some(active_parameter),
        })
    }

    /// Find function call at a position
    fn find_function_call_at_position(doc: &Document, position: Position) -> Option<(String, u32)> {
        let lines: Vec<&str> = doc.text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let cursor_char = position.character as usize;
        if cursor_char > line.len() {
            return None;
        }

        // Look backwards from cursor to find function name
        let text_before_cursor = &line[..cursor_char];

        // Find opening parenthesis of function call
        let paren_pos = text_before_cursor.rfind('(')?;
        let after_paren = &text_before_cursor[paren_pos + 1..];

        // Count commas to determine active parameter
        let active_parameter = after_paren.chars().filter(|&c| c == ',').count() as u32;

        // Extract function name
        let before_paren = &text_before_cursor[..paren_pos];
        let function_name = Self::extract_identifier(before_paren)?;

        Some((function_name, active_parameter))
    }

    /// Extract identifier from text
    fn extract_identifier(text: &str) -> Option<String> {
        // Find the last opening parenthesis and extract identifier before it
        if let Some(paren_pos) = text.rfind('(') {
            // Get text before parenthesis and trim
            let before_paren = text[..paren_pos].trim();
            if before_paren.is_empty() {
                None
            } else {
                Some(before_paren.to_string())
            }
        } else {
            // No parenthesis found, take the whole trimmed string
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }

    /// Get signature information for a function
    fn get_signature_info(function_name: &str) -> Option<SignatureInformation> {
        // Built-in functions with their signatures
        let signature = match function_name {
            "test" => Self::make_signature(
                "test(name: string, fn: () => void)",
                "Define a test case",
                &["name", "fn"],
            ),
            "invariant" => Self::make_signature(
                "invariant(name: string, expr: boolean)",
                "Define an invariant",
                &["name", "expr"],
            ),
            "bench" => Self::make_signature(
                "bench(name: string, fn: () => void)",
                "Define a benchmark",
                &["name", "fn"],
            ),
            "let" => Self::make_signature(
                "let(name: string, value: expr)",
                "Declare a variable",
                &["name", "value"],
            ),
            "const" => Self::make_signature(
                "const(name: string, type: type, value: expr)",
                "Declare a constant",
                &["name", "type", "value"],
            ),
            "fn" => Self::make_signature(
                "fn(name: string, params: [param], return: type)",
                "Define a function",
                &["name", "params", "return"],
            ),
            "type" => Self::make_signature(
                "type(name: string, fields: [field])",
                "Define a type/struct",
                &["name", "fields"],
            ),
            "import" => Self::make_signature(
                "import(path: string)",
                "Import another t27 file",
                &["path"],
            ),
            _ => {
                // Look for user-defined functions
                return None;
            }
        };

        Some(signature)
    }

    /// Create a signature information object
    fn make_signature(label: &str, doc: &str, params: &[&str]) -> SignatureInformation {
        let parameters = params
            .iter()
            .map(|&p| ParameterInformation {
                label: tower_lsp::lsp_types::ParameterLabel::Simple(p.to_string()),
                documentation: None,
            })
            .collect();

        SignatureInformation {
            label: label.to_string(),
            documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: format!("{}\n\n```t27\n{}\n```", doc, label),
                },
            )),
            parameters: Some(parameters),
            active_parameter: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn test_extract_identifier() {
        assert_eq!(
            SignatureHelpService::extract_identifier("test"),
            Some("test".to_string())
        );
        assert_eq!(
            SignatureHelpService::extract_identifier("test(   "),
            Some("test".to_string())
        );
        assert_eq!(
            SignatureHelpService::extract_identifier("  test  ("),
            Some("test".to_string())
        );
        assert_eq!(SignatureHelpService::extract_identifier(""), None);
    }

    #[test]
    fn test_find_function_call_at_position() {
        let doc = Document::new(
            Url::parse("file:///test.t27").unwrap(),
            "test(\"hello\", () => {}".to_string(),
        );

        // At the opening parenthesis
        let pos = Position::new(0, 5);
        assert_eq!(
            SignatureHelpService::find_function_call_at_position(&doc, pos),
            Some(("test".to_string(), 0))
        );

        // After the first parameter
        let pos = Position::new(0, 13);
        assert_eq!(
            SignatureHelpService::find_function_call_at_position(&doc, pos),
            Some(("test".to_string(), 1))
        );
    }

    #[test]
    fn test_get_signature_info() {
        let sig = SignatureHelpService::get_signature_info("test");
        assert!(sig.is_some());
        let sig = sig.unwrap();
        assert!(sig.label.contains("test"));
        assert!(sig.parameters.is_some());
        assert_eq!(sig.parameters.unwrap().len(), 2);
    }
}
