// Memory & Search — generated from specs/memory/*.t27
// DO NOT EDIT — edit specs instead (S³AI Law L2)
// phi^2 + 1/phi^2 = 3 | TRINITY

// ============================================================================
// Shared constants and utilities
// ============================================================================

/// PHI constant (golden ratio)
pub const PHI: f64 = 1.618033988749895;

/// Embedding dimension (TRINITY: 27 = 3^3)
pub const EMBEDDING_DIM: usize = 27;

/// Cosine similarity between two vectors
pub fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

// ============================================================================
// Include generated semantic search modules
// ============================================================================

// Note: These files are generated from .t27 specs via:
//   ./target/release/t27c gen-rust specs/memory/formula_embed.t27
//   ./target/release/t27c gen-rust specs/memory/semantic_search.t27

include!("../../../gen/rust/memory/formula_embed.rs");
include!("../../../gen/rust/memory/semantic_search.rs");
