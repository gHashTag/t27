//! FineWeb Dataloader

pub struct FineWebBatch {
    pub tokens: Vec<u32>,
}

impl FineWebBatch {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
        }
    }
}
