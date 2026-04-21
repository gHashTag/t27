//! IGLA-GF16 Model

pub struct IGLAGF16Model {
    pub vocab_size: usize,
    pub d_model: usize,
    pub n_layers: usize,
}

impl IGLAGF16Model {
    pub fn new(vocab: usize, d_model: usize, n_layers: usize) -> Self {
        Self {
            vocab_size: vocab,
            d_model,
            n_layers,
        }
    }
}
