//! Code completion for t27

use tower_lsp::lsp_types::*;
use crate::parser::{T27SyntaxKind, SymbolKind};

/// Completion item with additional context
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
    pub filter_text: Option<String>,
}

/// Context for completion
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub uri: String,
    pub line: String,
    pub column: usize,
    pub symbols: Vec<Symbol>,
}

/// Get completions based on context
pub fn get_completions(context: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let prefix = &context.line[..context.column.min(context.line.len())];

    // Keywords
    let keywords = vec![
        ("module", CompletionItemKind::KEYWORD, "Define a module", "module MyModule { ... }"),
        ("fn", CompletionItemKind::KEYWORD, "Define a function", "fn my_name() -> type { ... }"),
        ("const", CompletionItemKind::KEYWORD, "Define a constant", "const PHI: phi = 1.618;"),
        ("let", CompletionItemKind::KEYWORD, "Define a variable", "let x: type = value;"),
        ("if", CompletionItemKind::KEYWORD, "Conditional", "if condition { ... }"),
        ("else", CompletionItemKind::KEYWORD, "Alternative branch", "else { ... }"),
        ("return", CompletionItemKind::KEYWORD, "Return value", "return value;"),
        ("test", CompletionItemKind::KEYWORD, "Test case", "test \"name\" { given; then; expect; }"),
        ("invariant", CompletionItemKind::KEYWORD, "Invariant check", "invariant: expression == expected;"),
        ("bench", CompletionItemKind::KEYWORD, "Benchmark", "bench \"name\" { ... }"),
        ("given", CompletionItemKind::KEYWORD, "Test setup", "given { setup }"),
        ("then", CompletionItemKind::KEYWORD, "Test action", "then { action }"),
        ("expect", CompletionItemKind::KEYWORD, "Test assertion", "expect { assertion }"),
    ];

    for (kw, kind, detail, example) in keywords {
        if kw.starts_with(prefix) && kw.len() > prefix.len() {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind,
                detail: Some(detail.to_string()),
                documentation: Some(format!("```t27\n{}\n```", example)),
                insert_text: kw.to_string(),
                filter_text: None,
            });
        }
    }

    // Types
    let types = vec![
        ("phi", CompletionItemKind::TYPE, "Golden ratio (GF16)", "φ ≈ 1.618034"),
        ("gf4", CompletionItemKind::TYPE, "4-bit GoldenFloat", "4-bit format"),
        ("gf8", CompletionItemKind::TYPE, "8-bit GoldenFloat", "8-bit format"),
        ("gf12", CompletionItemKind::TYPE, "12-bit GoldenFloat", "12-bit format"),
        ("gf16", CompletionItemKind::TYPE, "16-bit GoldenFloat", "Primary type, 6+9 bits"),
        ("gf20", CompletionItemKind::TYPE, "20-bit GoldenFloat", "Training format"),
        ("gf24", CompletionItemKind::TYPE, "24-bit GoldenFloat", "High precision"),
        ("gf32", CompletionItemKind::TYPE, "32-bit GoldenFloat", "Best δ"),
        ("u8", CompletionItemKind::TYPE, "8-bit unsigned", "0 to 255"),
        ("u16", CompletionItemKind::TYPE, "16-bit unsigned", "0 to 65535"),
        ("u32", CompletionItemKind::TYPE, "32-bit unsigned", "0 to 4294967295"),
        ("u64", CompletionItemKind::TYPE, "64-bit unsigned", "0 to 18446744073709551615"),
        ("i8", CompletionItemKind::TYPE, "8-bit signed", "-128 to 127"),
        ("i16", CompletionItemKind::TYPE, "16-bit signed", "-32768 to 32767"),
        ("i32", CompletionItemKind::TYPE, "32-bit signed", "-2147483648 to 2147483647"),
        ("i64", CompletionItemKind::TYPE, "64-bit signed", "-2^63 to 2^63-1"),
        ("f32", CompletionItemKind::TYPE, "32-bit float", "IEEE 754"),
        ("f64", CompletionItemKind::TYPE, "64-bit float", "IEEE 754"),
        ("bool", CompletionItemKind::TYPE, "Boolean", "true or false"),
        ("str", CompletionItemKind::TYPE, "String", "Text"),
        ("vec", CompletionItemKind::TYPE, "Vector", "Dynamic array"),
        ("array", CompletionItemKind::TYPE, "Array", "Fixed-size array"),
        ("option", CompletionItemKind::TYPE, "Optional", "May contain value"),
        ("result", CompletionItemKind::TYPE, "Result", "Success or error"),
    ];

    for (t, kind, detail, desc) in types {
        if t.starts_with(prefix) && t.len() > prefix.len() {
            items.push(CompletionItem {
                label: t.to_string(),
                kind,
                detail: Some(detail.to_string()),
                documentation: Some(desc.to_string()),
                insert_text: t.to_string(),
                filter_text: None,
            });
        }
    }

    // Built-in symbols
    let builtins = vec![
        ("PHI", CompletionItemKind::CONSTANT, "Golden ratio constant", "1.618033988749895"),
        ("PI", CompletionItemKind::CONSTANT, "π constant", "3.141592653589793"),
        ("E", CompletionItemKind::CONSTANT, "e constant", "2.718281828459045"),
    ];

    for (name, kind, detail, value) in builtins {
        if name.starts_with(prefix) {
            items.push(CompletionItem {
                label: name.to_string(),
                kind,
                detail: Some(detail.to_string()),
                documentation: Some(value.to_string()),
                insert_text: name.to_string(),
                filter_text: None,
            });
        }
    }

    // Module and function symbols from context
    for symbol in &context.symbols {
        if symbol.name.starts_with(prefix) {
            let kind = match symbol.kind {
                SymbolKind::Module => CompletionItemKind::MODULE,
                SymbolKind::Function => CompletionItemKind::FUNCTION,
                SymbolKind::Const => CompletionItemKind::CONSTANT,
                SymbolKind::Variable => CompletionItemKind::VARIABLE,
                SymbolKind::Type => CompletionItemKind::STRUCT,
                SymbolKind::Test => CompletionItemKind::INTERFACE,
                SymbolKind::Invariant => CompletionItemKind::INTERFACE,
                SymbolKind::Benchmark => CompletionItemKind::INTERFACE,
            };
            items.push(CompletionItem {
                label: symbol.name.clone(),
                kind,
                detail: symbol.detail.clone(),
                documentation: None,
                insert_text: symbol.name.clone(),
                filter_text: None,
            });
        }
    }

    items
}

/// Get completion for type annotation (after colon)
pub fn get_type_completions(context: &CompletionContext) -> Vec<CompletionItem> {
    get_completions(context)
        .into_iter()
        .filter(|item| matches!(item.kind, CompletionItemKind::TYPE | CompletionItemKind::MODULE))
        .collect()
}

/// Get completion after "fn" keyword
pub fn get_function_completion(context: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Function template
    items.push(CompletionItem {
        label: "function_template".to_string(),
        kind: CompletionItemKind::SNIPPET,
        detail: Some("Function template".to_string()),
        documentation: Some("Create a new function".to_string()),
        insert_text: "${1:name}(${2:params}) -> ${3:return_type} {\n\t$0\n}".to_string(),
        filter_text: None,
    });

    items
}