// Completion service for t27 Language Server

use crate::types::Document;
use crate::types::symbol::SymbolKind;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, Position, Range,
    TextEdit,
};

/// Completion service
pub struct CompletionService;

impl CompletionService {
    /// Generate completion items for a position in a document
    pub fn complete(doc: &Document, position: Position, trigger_char: Option<char>) -> CompletionList {
        let context = CompletionContext::new(doc, position, trigger_char);

        let mut items = Vec::new();

        // Add keyword completions
        items.extend(Self::keyword_completions(&context));

        // Add symbol completions
        items.extend(Self::symbol_completions(&context));

        // Add snippet completions
        if context.trigger_char == Some('{') {
            items.extend(Self::snippet_completions(&context));
        }

        CompletionList {
            is_incomplete: false,
            items,
        }
    }

    fn keyword_completions(_context: &CompletionContext) -> Vec<CompletionItem> {
        let keywords = [
            ("module", "module declaration"),
            ("import", "import statement"),
            ("export", "export statement"),
            ("pub", "public visibility"),
            ("const", "constant declaration"),
            ("type", "type declaration"),
            ("fn", "function declaration"),
            ("test", "test block"),
            ("invariant", "invariant block"),
            ("bench", "benchmark block"),
            ("let", "variable binding"),
            ("return", "return statement"),
            ("if", "conditional"),
            ("else", "alternative branch"),
            ("match", "pattern matching"),
            ("for", "loop"),
            ("while", "while loop"),
            ("break", "break statement"),
            ("continue", "continue statement"),
            ("true", "boolean true"),
            ("false", "boolean false"),
        ];

        keywords
            .iter()
            .map(|(keyword, detail)| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(detail.to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn symbol_completions(context: &CompletionContext) -> Vec<CompletionItem> {
        context
            .doc
            .symbols
            .iter()
            .filter(|s| Self::is_symbol_visible(s, context))
            .map(|symbol| Self::symbol_to_completion_item(symbol, context))
            .collect()
    }

    fn snippet_completions(context: &CompletionContext) -> Vec<CompletionItem> {
        let snippets = [
            ("fn", "function", "fn ${1:name}(${2:params}) {\n    ${3:body}\n}"),
            ("test", "test block", "test ${1:name} {\n    ${2:assertions}\n}"),
            ("invariant", "invariant block", "invariant ${1:name} {\n    ${2:assertion}\n}"),
            ("const", "constant", "const ${1:name}: ${2:type} = ${3:value};"),
            ("type", "type declaration", "type ${1:name} = ${2:definition};"),
        ];

        snippets
            .iter()
            .map(|(label, detail, snippet)| CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(detail.to_string()),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }

    fn is_symbol_visible(_symbol: &crate::types::Symbol, _context: &CompletionContext) -> bool {
        // TODO: Implement visibility checking
        true
    }

    fn symbol_to_completion_item(
        symbol: &crate::types::Symbol,
        context: &CompletionContext,
    ) -> CompletionItem {
        let kind = match symbol.kind {
            SymbolKind::Function => CompletionItemKind::FUNCTION,
            SymbolKind::Variable => CompletionItemKind::VARIABLE,
            SymbolKind::Constant => CompletionItemKind::CONSTANT,
            SymbolKind::Type => CompletionItemKind::STRUCT,
            SymbolKind::Module => CompletionItemKind::MODULE,
            _ => CompletionItemKind::VARIABLE,
        };

        CompletionItem {
            label: symbol.name.clone(),
            kind: Some(kind),
            detail: symbol.detail.clone(),
            documentation: symbol
                .documentation
                .clone()
                .map(tower_lsp::lsp_types::Documentation::String),
            ..Default::default()
        }
    }
}

/// Completion context
struct CompletionContext<'a> {
    doc: &'a Document,
    position: Position,
    trigger_char: Option<char>,
}

impl<'a> CompletionContext<'a> {
    fn new(doc: &'a Document, position: Position, trigger_char: Option<char>) -> Self {
        Self {
            doc,
            position,
            trigger_char,
        }
    }
}
