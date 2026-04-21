pub fn spectral_init_embedding(vocab_size: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut embeddings = Vec::with_capacity(vocab_size);

    for i in 0..vocab_size {
        let mut row = vec![0.0f32; dim];
        let freq_base = 1.0 / (1.618033988749895_f64).powi(i as i32 % 10 + 1);

        for (d, item) in row.iter_mut().enumerate() {
            let freq = freq_base / ((d + 1) as f64);
            let phase = (i as f64 * 0.618) * std::f64::consts::PI;
            let spectral_val = (2.0 * std::f64::consts::PI * freq + phase).sin();
            *item = (spectral_val * 0.02) as f32;
        }

        let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        for x in row.iter_mut() {
            *x /= norm;
        }

        embeddings.push(row);
    }

    embeddings
}

pub fn spectral_init_linear(rows: usize, cols: usize) -> Vec<f32> {
    let mut weights = Vec::with_capacity(rows * cols);
    let scale = (2.0_f32 / rows as f32).sqrt();

    for i in 0..rows {
        for j in 0..cols {
            let freq = 1.0 / 1.618_034_f32.powi(((i + j) % 7 + 1) as i32);
            let phase = (i as f32 * 0.618) * std::f32::consts::PI;
            let val = (2.0 * std::f32::consts::PI * freq * j as f32 + phase).sin();
            weights.push(val * scale * 0.02);
        }
    }

    weights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectral_embedding_shape() {
        let emb = spectral_init_embedding(100, 64);
        assert_eq!(emb.len(), 100);
        assert_eq!(emb[0].len(), 64);
    }

    #[test]
    fn spectral_embedding_normalized() {
        let emb = spectral_init_embedding(10, 32);
        for row in &emb {
            let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01, "norm = {}", norm);
        }
    }

    #[test]
    fn spectral_embedding_non_zero() {
        let emb = spectral_init_embedding(10, 32);
        let sum: f32 = emb.iter().flat_map(|r| r.iter()).sum();
        assert!(sum.abs() > 0.0, "embeddings should not all be zero");
    }

    #[test]
    fn spectral_linear_shape() {
        let w = spectral_init_linear(64, 32);
        assert_eq!(w.len(), 64 * 32);
    }

    #[test]
    fn spectral_embedding_diverse() {
        let emb = spectral_init_embedding(100, 64);
        let dot: f32 = emb[0].iter().zip(emb[1].iter()).map(|(a, b)| a * b).sum();
        assert!(
            dot.abs() < 0.99,
            "different rows should not be identical, dot={}",
            dot
        );
    }
}
