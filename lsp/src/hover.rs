//! Hover documentation for t27

/// Hover content for symbols
pub struct HoverContent {
    pub title: String,
    pub content: String,
    pub kind: HoverKind,
}

pub enum HoverKind {
    Keyword,
    Type,
    Function,
    Constant,
    Variable,
}

/// Get hover documentation for a word
pub fn get_hover_documentation(word: &str) -> Option<String> {
    match word {
        "module" => Some("**module**\n\nDefines a named module for organizing code.\n\n```t27\nmodule ModuleName {\n    // contents\n}\n```"),
        "fn" => Some("**fn**\n\nDefines a function.\n\n```t27\nfn name(params) -> return_type {\n    // body\n}\n```"),
        "const" => Some("**const**\n\nDefines a constant value.\n\n```t27\nconst NAME: type = value;\n```"),
        "phi" => Some("**phi (φ)**\n\nGolden ratio type (GF16 format).\n\nValue: ≈ 1.618033988749895\n\nProperties:\n- φ² = φ + 1\n- φ² + φ⁻² = 3\n- Self-similarity principle"),
        "gf16" => Some("**gf16**\n\n16-bit GoldenFloat format.\n\nStructure:\n- 1 bit sign\n- 6 bits exponent\n- 9 bits mantissa\n\nPerfect for ML model parameters."),
        "gf32" => Some("**gf32**\n\n32-bit GoldenFloat format.\n\nStructure:\n- 1 bit sign\n- 12 bits exponent\n- 19 bits mantissa\n\nBest δ (minimum φ-distance)."),
        "test" => Some("**test**\n\nDefines a test case.\n\nStructure:\n```t27\ntest \"name\" {\n    given { setup }\n    then { action }\n    expect { assertion }\n}\n```"),
        "invariant" => Some("**invariant**\n\nDefines an invariant that must always hold true.\n\n```t27\ninvariant: expression == expected;\n```"),
        "bench" => Some("**bench**\n\nDefines a benchmark.\n\n```t27\nbench \"name\" {\n    // benchmark code\n}\n```"),
        _ => None,
    }
}

/// Format hover content as Markdown
pub fn format_hover(content: &HoverContent) -> String {
    match content.kind {
        HoverKind::Keyword => format!("### {}\n{}", content.title, content.content),
        HoverKind::Type => format!("### {}\n``t27\n{}\n```", content.title, content.content),
        HoverKind::Function => format!("### {}\n```t27\n{}\n```", content.title, content.content),
        HoverKind::Constant => format!("### {}\n**Value:** {}", content.title, content.content),
        HoverKind::Variable => format!("### {}\n**Type:** {}", content.title, content.content),
    }
}