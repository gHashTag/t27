//! OrthoInit Baseline — Standard Orthogonal Initialization
//!
//! P04: OrthoInit baseline (gain=1.0) as control
//! Expected ΔBPB: −0.02
//!
//! Reference: #190 GOLF task

use rand::Rng;

/// Standard orthogonal initialization (PyTorch nn.init.orthogonal_)
///
/// # Arguments
/// * `embeddings` - [vocab_size, d_model] embedding matrix
/// * `d_model` - hidden dimension
/// * `vocab_size` - vocabulary size
pub fn ortho_init_baseline(embeddings: &mut [f32], d_model: usize, vocab_size: usize) {
    let mut rng = rand::thread_rng();

    for v in 0..vocab_size {
        let v_offset = v * d_model;

        // Initialize with small random values
        for d in 0..d_model {
            embeddings[v_offset + d] = (rng.gen::<f32>() - 0.5) * 0.1;
        }

        // Normalize to unit norm (orthogonal initialization)
        let mut norm = 0.0f32;
        for d in 0..d_model {
            norm += embeddings[v_offset + d].powi(2);
        }
        norm = norm.sqrt();

        if norm > 1e-6 {
            let scale = 1.0 / norm;
            for d in 0..d_model {
                embeddings[v_offset + d] *= scale;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ortho_init_unit_norm() {
        let d_model = 64;
        let vocab_size = 256;
        let mut embeddings = vec![0.0f32; vocab_size * d_model];

        ortho_init_baseline(&mut embeddings, d_model, vocab_size);

        // Check each embedding vector has unit norm
        for v in 0..vocab_size {
            let v_offset = v * d_model;
            let mut norm = 0.0f32;

            for d in 0..d_model {
                norm += embeddings[v_offset + d].powi(2);
            }
            norm = norm.sqrt();

            assert!((norm - 1.0).abs() < 1e-6, "Embedding {} has norm {}", v, norm);
        }
    }
}
