use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HostSmokeJson {
    pub ok: bool,
    pub writes: usize,
    pub reads: usize,
    pub layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub threshold: u32,
    pub weight_addr: String,
    pub irq_stat: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostPollVsIrqJson {
    pub ok: bool,
    pub poll_writes: usize,
    pub poll_reads: usize,
    pub irq_writes: usize,
    pub irq_reads: usize,
    pub writes_match: bool,
    pub irq_stat_poll: String,
    pub irq_stat_irq: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostInferenceJson {
    pub ok: bool,
    pub total_layers: u32,
    pub layers_completed: u32,
    pub error_layer: Option<u32>,
    pub total_writes: usize,
    pub total_reads: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostPerfJson {
    pub ok: bool,
    pub layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub total_cycles: u32,
    pub total_weight_words: u32,
    pub total_weight_bytes: u32,
    pub bram_utilization_pct: f64,
    pub total_dma_beats: u32,
    pub throughput_inf_per_sec: f64,
    pub clock_mhz: f64,
}

pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let s = serde_json::to_string(value)
        .map_err(|e| anyhow::anyhow!("JSON serialization failed: {}", e))?;
    println!("{}", s);
    Ok(())
}
