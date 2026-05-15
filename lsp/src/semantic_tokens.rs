//! Semantic tokens for t27

use tower_lsp::lsp_types::*;
use crate::parser::T27SyntaxKind;

/// Legend for semantic tokens
pub const TOKEN_LEGEND: SemanticTokensLegend = SemanticTokensLegend {
    token_types: vec![
        SemanticTokenType::NAMESPACE,
        SemanticTokenType::TYPE,
        SemanticTokenType::CLASS,
        SemanticTokenType::ENUM,
        SemanticTokenType::INTERFACE,
        SemanticTokenType::STRUCT,
        SemanticTokenType::TYPE_PARAMETER,
        SemanticTokenType::PARAMETER,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::PROPERTY,
        SemanticTokenType::ENUM_MEMBER,
        SemanticTokenType::EVENT,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::METHOD,
        SemanticTokenType::MACRO,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::MODIFIER,
        SemanticTokenType::COMMENT,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::REGEXP,
        SemanticTokenType::OPERATOR,
        SemanticTokenType::DECORATOR,
    ],
    token_modifiers: vec![],
};

/// Get semantic token type for a t27 token kind
pub fn get_token_type(kind: T27SyntaxKind) -> u32 {
    match kind {
        T27SyntaxKind::Module => 0, // NAMESPACE
        T27SyntaxKind::Phi | T27SyntaxKind::Gf4 | T27SyntaxKind::Gf8 | T27SyntaxKind::Gf12 | T27SyntaxKind::Gf16
        | T27SyntaxKind::Gf20 | T27SyntaxKind::Gf24 | T27SyntaxKind::Gf32 => 1, // TYPE
        T27SyntaxKind::Fn => 13, // FUNCTION
        T27SyntaxKind::Const => 3, // PROPERTY
        T27SyntaxKind::Let => 3, // PROPERTY
        T27SyntaxKind::Test | T27SyntaxKind::Invariant | T27SyntaxKind::Bench => 4, // INTERFACE
        T27SyntaxKind::Module => 0, // NAMESPACE (already handled above)
        T27SyntaxKind::U8 | T27SyntaxKind::U16 | T27SyntaxKind::U32 | T27SyntaxKind::U64 => 1, // TYPE
        T27SyntaxKind::I8 | T27SyntaxKind::I16 | T27SyntaxKind::I32 | T27SyntaxKind::I64 => 1, // TYPE
        T27SyntaxKind::F32 | T27SyntaxKind::F64 => 1, // TYPE
        T27SyntaxKind::Bool | T27SyntaxKind::Str => 1, // TYPE
        T27SyntaxKind::Vec | T27SyntaxKind::Array => 1, // TYPE
        T27SyntaxKind::Option | T27SyntaxKind::Result => 1, // TYPE
        T27SyntaxKind::If | T27SyntaxKind::Else | T27SyntaxKind::Return => 15, // KEYWORD
        T27SyntaxKind::Given | T27SyntaxKind::Then | T27SyntaxKind::Expect => 15, // KEYWORD
        T27SyntaxKind::LineComment | T27SyntaxKind::BlockComment => 18, // COMMENT
        T27SyntaxKind::StringLiteral => 19, // STRING
        T27SyntaxKind::PhiLiteral | T27SyntaxKind::NumberLiteral => 20, // NUMBER
        T27SyntaxKind::BooleanLiteral => 20, // NUMBER
        T27SyntaxKind::Plus | T27SyntaxKind::Minus | T27SyntaxKind::Star | T27SyntaxKind::Slash
        | T27SyntaxKind::Percent | T27SyntaxKind::Caret | T27SyntaxKind::Less | T27SyntaxKind::Greater
        | T27SyntaxKind::Equal | T27SyntaxKind::BangEqual | T27SyntaxKind::LessEqual | T27SyntaxKind::GreaterEqual
        | T27SyntaxKind::And | T27SyntaxKind::Or | T27SyntaxKind::Not => 22, // OPERATOR
        _ => 0, // NAMESPACE (default)
    }
}

/// Encode semantic tokens for a document
pub fn encode_semantic_tokens(tokens: &[crate::parser::Token]) -> Vec<SemanticToken> {
    tokens
        .iter()
        .filter_map(|token| {
            if token.kind == T27SyntaxKind::Whitespace || token.kind == T27SyntaxKind::Unknown {
                None
            } else {
                Some(SemanticToken {
                    token_type: get_token_type(token.kind),
                    token_modifiers_bitset: 0,
                    delta_line: token.end.line - token.start.line,
                    delta_start: if token.end.line > token.start.line {
                        token.end.character
                    } else {
                        token.end.character - token.start.character
                    },
                    length: (token.end.line - token.start.line) as u32
                        + if token.end.line == token.start.line {
                            token.end.character - token.start.character
                        } else {
                            token.end.character
                        },
                })
            }
        })
        .collect()
}