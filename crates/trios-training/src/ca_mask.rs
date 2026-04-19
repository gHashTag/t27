//! CA φ-Mask

pub struct CAMask {
    sparsity: f32,
}

impl CAMask {
    pub fn new(seq_len: usize) -> Self {
        let fib_dists = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];
        let visible = fib_dists.iter().filter(|&&d| d < seq_len).count();
        let sparsity = 1.0 - (visible as f32 / seq_len as f32);
        Self { sparsity }
    }
}
