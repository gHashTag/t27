//! What a `.t27` file *is*, decided before the parser is asked.
//!
//! A `.t27` extension is a filename, not a type declaration. The corpus holds
//! Markdown documents, TRI-27 assembly listings and deliberately damaged test
//! fixtures under that extension, and a parser asked about any of them answers
//! the only way it can: this is not a module. Counting that answer as a broken
//! spec inflates every corpus ratio in a knowable direction.
//!
//! This rule used to live inside `run_classify` in `main.rs`, where it was
//! reachable from the CLI and from nothing else. The Spec Explorer needed the
//! same answer and could not have it, and re-deriving the rule in JavaScript
//! would have made a second, weaker implementation of something the compiler
//! already decides -- the failure this codebase keeps repeating. It is a module
//! now so that `run_classify` and `bindings/wasm-explorer` read one rule.
//!
//! It reads the OPENING of a file. Parsing is a different question about the
//! rest, and the two disagree in both directions: `parse-complete` measured
//! 2026-08-24 that 5 non-`Source` files parse anyway. Neither is a proxy for
//! the other.

/// What a `.t27` file is, judged from its text alone.
///
/// The order of the variants is the order the rule tries them, which is also
/// the order `run_classify` reported them in before the rule moved here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceKind {
    /// `module X { ... }` or `module X;` -- an ordinary compilation unit.
    Source,
    /// `spec X { ... }` -- the older surface, still in the corpus.
    AltSyntax,
    /// Markdown headings near the top and no `fn` anywhere: a document.
    NotCode,
    /// Markdown headings AND functions -- prose with code in it, which is
    /// neither a document nor a module and is not treated as either.
    Mixed,
    /// None of the above. This is a genuine grab-bag, not a synonym for prose:
    /// it holds damaged fixtures whose `module X` line lost its brace, TRI-27
    /// assembly whose every line opens with `;`, and files that begin at a
    /// `const`. Do not report it as "not code".
    Unclassified,
}

impl SourceKind {
    /// The stable machine name. This crosses the wasm boundary and lands in a
    /// catalog, so it is spelled out here rather than derived from `Debug`,
    /// which would rename every consumer the day a variant is renamed.
    pub fn slug(self) -> &'static str {
        match self {
            SourceKind::Source => "source",
            SourceKind::AltSyntax => "alt-syntax",
            SourceKind::NotCode => "not-code",
            SourceKind::Mixed => "mixed",
            SourceKind::Unclassified => "unclassified",
        }
    }

    /// The column `t27c classify` prints, padded as it has always been so the
    /// tool's output is unchanged by the extraction.
    pub fn label(self) -> &'static str {
        match self {
            SourceKind::Source => "SOURCE          module ...",
            SourceKind::AltSyntax => "ALT-SYNTAX      spec X { ... }",
            SourceKind::NotCode => "NOT-CODE        Markdown document",
            SourceKind::Mixed => "MIXED           Markdown + fn",
            SourceKind::Unclassified => "UNCLASSIFIED    neither module, spec nor Markdown",
        }
    }

    /// Whether this file is an ordinary compilation unit -- the honest
    /// denominator for any corpus ratio about compiling.
    pub fn is_source(self) -> bool {
        matches!(self, SourceKind::Source)
    }
}

/// Classify one file's text.
///
/// Deliberately text-matching and deliberately cheap: it must answer for a file
/// the lexer cannot get through, which rules out asking the compiler.
pub fn classify(text: &str) -> SourceKind {
    let has_module = text.lines().any(|l| {
        let l = l.trim_start().trim_start_matches("pub ").trim_start();
        l.starts_with("module ") && (l.ends_with(';') || l.contains('{'))
    });
    let has_spec = text
        .lines()
        .any(|l| l.trim_start().starts_with("spec ") && l.contains('{'));
    // A Markdown heading in the first 40 lines, `# ` or `## `, with no `fn`.
    let head: Vec<&str> = text.lines().take(40).collect();
    let md = head.iter().any(|l| {
        let l = l.trim_start();
        (l.starts_with("# ") || l.starts_with("## ") || l.starts_with("### ")) && l.len() > 3
    });
    let has_fn = text
        .lines()
        .any(|l| l.trim_start().trim_start_matches("pub ").starts_with("fn "));

    if has_module {
        SourceKind::Source
    } else if has_spec {
        SourceKind::AltSyntax
    } else if md && !has_fn {
        SourceKind::NotCode
    } else if md {
        SourceKind::Mixed
    } else {
        SourceKind::Unclassified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_module_is_source() {
        assert_eq!(classify("module m {\n}\n"), SourceKind::Source);
        assert_eq!(classify("pub module m;\n"), SourceKind::Source);
        // Prose above the declaration does not change what the file is.
        assert_eq!(classify("# Title\n\nmodule m {\n}\n"), SourceKind::Source);
    }

    #[test]
    fn a_markdown_document_is_not_code() {
        assert_eq!(classify("# Title\n\nSome prose.\n"), SourceKind::NotCode);
        // `#` alone is a heading marker with no heading.
        assert_eq!(classify("#\n\nprose\n"), SourceKind::Unclassified);
    }

    #[test]
    fn markdown_with_functions_is_mixed_not_a_document() {
        assert_eq!(classify("# Title\n\nfn f() {}\n"), SourceKind::Mixed);
    }

    #[test]
    fn the_older_spec_surface_is_alt_syntax() {
        assert_eq!(classify("spec S {\n}\n"), SourceKind::AltSyntax);
        // Without a brace it is not the alt surface either -- this is
        // `trinity-fpga/specs/physics/gamma_conjecture.t27`.
        assert_eq!(
            classify("spec GammaConjecture version 1.0.0\n"),
            SourceKind::Unclassified
        );
    }

    #[test]
    fn a_damaged_module_line_is_unclassified_not_source() {
        // `bootstrap/tests/fixtures/damage/damage_class_01.t27` opens with a
        // `module` line that lost its brace. It is a fixture that exists to be
        // broken, and it is NOT source -- which is the whole point of asking.
        assert_eq!(classify("module damage_class_01\n"), SourceKind::Unclassified);
    }

    #[test]
    fn tri27_assembly_is_unclassified() {
        // Every line opens with `;`, which t27 does not treat as a comment.
        assert_eq!(
            classify("; vsa_bind -- TRI-27\n; ---\nLOAD r1, r2\n"),
            SourceKind::Unclassified
        );
    }

    #[test]
    fn a_heading_deeper_than_the_window_does_not_count() {
        let mut t = String::new();
        for _ in 0..45 {
            t.push_str("prose\n");
        }
        t.push_str("# Title\n");
        assert_eq!(classify(&t), SourceKind::Unclassified);
    }

    #[test]
    fn slugs_are_distinct_and_stable() {
        let all = [
            SourceKind::Source,
            SourceKind::AltSyntax,
            SourceKind::NotCode,
            SourceKind::Mixed,
            SourceKind::Unclassified,
        ];
        let mut slugs: Vec<&str> = all.iter().map(|k| k.slug()).collect();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), all.len());
        assert!(SourceKind::Source.is_source());
        assert!(!SourceKind::Unclassified.is_source());
    }
}
