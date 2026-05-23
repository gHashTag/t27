use super::csr_map;
use super::driver::{BitnetDriver, DriverError};
use super::irq::IrqDrivenDriver;
use super::mmio::MockMmio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferenceReport {
    pub total_layers: u32,
    pub layers_completed: u32,
    pub error_layer: Option<u32>,
    pub total_writes: usize,
    pub total_reads: usize,
}

pub struct InferenceEngine {
    driver: IrqDrivenDriver<MockMmio>,
    num_layers: u32,
    neurons: u32,
    chunks: u32,
    threshold: u32,
    weight_addr: u64,
}

impl InferenceEngine {
    pub fn new(driver: BitnetDriver<MockMmio>) -> Self {
        Self {
            driver: IrqDrivenDriver::new(driver),
            num_layers: 2,
            neurons: 16,
            chunks: 4,
            threshold: 1,
            weight_addr: 0,
        }
    }

    pub fn configure(
        &mut self,
        num_layers: u32,
        neurons: u32,
        chunks: u32,
        threshold: u32,
        weight_addr: u64,
    ) -> Result<(), DriverError> {
        if num_layers == 0 || neurons == 0 || chunks == 0 {
            return Err(DriverError::InvalidConfig);
        }
        self.num_layers = num_layers;
        self.neurons = neurons;
        self.chunks = chunks;
        self.threshold = threshold;
        self.weight_addr = weight_addr;
        Ok(())
    }

    pub fn run(&mut self, max_rounds_per_stage: u32) -> Result<InferenceReport, DriverError> {
        self.driver
            .handler_mut()
            .driver_mut()
            .configure(
                self.num_layers,
                self.neurons,
                self.chunks,
                self.threshold,
                self.weight_addr,
            )?;
        self.driver
            .handler_mut()
            .driver_mut()
            .enable_irqs(csr_map::IRQ_ALL_MASK);

        let mut layers_completed: u32 = 0;
        let mut error_layer: Option<u32> = None;

        for layer in 0..self.num_layers {
            let layer_weight_addr = self.weight_addr.wrapping_add(
                (layer as u64) * 0x1_0000 * (self.neurons as u64) * (self.chunks as u64),
            );
            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .poke(csr_map::WEIGHT_ADDR_LO, layer_weight_addr as u32);
            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .poke(csr_map::WEIGHT_ADDR_HI, (layer_weight_addr >> 32) as u32);

            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .latch_irq(csr_map::IRQ_DMA_DONE_MASK);
            if let Err(e) = self.wait_dma_done(max_rounds_per_stage) {
                error_layer = Some(layer);
                if e == DriverError::EngineError {
                    break;
                }
                return Err(e);
            }

            self.driver.handler_mut().driver_mut().start();
            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .set_done(true);
            if let Err(e) = self.wait_inference_done(max_rounds_per_stage) {
                error_layer = Some(layer);
                if e == DriverError::EngineError {
                    break;
                }
                return Err(e);
            }

            self.driver
                .handler_mut()
                .driver_mut()
                .mmio_mut()
                .latch_irq(csr_map::IRQ_DMA_DONE_MASK);
            if let Err(e) = self.wait_dma_done(max_rounds_per_stage) {
                error_layer = Some(layer);
                if e == DriverError::EngineError {
                    break;
                }
                return Err(e);
            }

            layers_completed += 1;
        }

        let total_writes = self.driver.handler().driver().mmio().write_count();
        let total_reads = self.driver.handler().driver().mmio().read_count();

        Ok(InferenceReport {
            total_layers: self.num_layers,
            layers_completed,
            error_layer,
            total_writes,
            total_reads,
        })
    }

    fn wait_dma_done(&mut self, max_rounds: u32) -> Result<(), DriverError> {
        self.driver.wait_irq_mask(csr_map::IRQ_DMA_DONE_MASK, max_rounds)
    }

    fn wait_inference_done(&mut self, max_rounds: u32) -> Result<(), DriverError> {
        self.driver.wait_irq_mask(csr_map::IRQ_INFERENCE_DONE_MASK, max_rounds)
    }

    pub fn driver(&self) -> &IrqDrivenDriver<MockMmio> {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut IrqDrivenDriver<MockMmio> {
        &mut self.driver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::mmio::MockMmio;

    fn fresh() -> InferenceEngine {
        InferenceEngine::new(BitnetDriver::new(MockMmio::with_csrs_zeroed()))
    }

    #[test]
    fn configure_rejects_zero_layers() {
        let mut e = fresh();
        assert_eq!(e.configure(0, 1, 1, 1, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn configure_rejects_zero_neurons() {
        let mut e = fresh();
        assert_eq!(e.configure(1, 0, 1, 1, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn configure_rejects_zero_chunks() {
        let mut e = fresh();
        assert_eq!(e.configure(1, 1, 0, 1, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn run_single_layer_succeeds() {
        let mut e = fresh();
        e.configure(1, 16, 4, 1, 0).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.total_layers, 1);
        assert_eq!(report.layers_completed, 1);
        assert_eq!(report.error_layer, None);
    }

    #[test]
    fn run_two_layers_succeeds() {
        let mut e = fresh();
        e.configure(2, 16, 4, 1, 0).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.total_layers, 2);
        assert_eq!(report.layers_completed, 2);
    }

    #[test]
    fn run_reports_writes_and_reads() {
        let mut e = fresh();
        e.configure(1, 16, 4, 1, 0).unwrap();
        let report = e.run(4).unwrap();
        assert!(report.total_writes > 0);
        assert!(report.total_reads > 0);
    }

    #[test]
    fn run_increases_writes_with_more_layers() {
        let mut e1 = fresh();
        e1.configure(1, 16, 4, 1, 0).unwrap();
        let r1 = e1.run(4).unwrap();

        let mut e2 = fresh();
        e2.configure(3, 16, 4, 1, 0).unwrap();
        let r2 = e2.run(4).unwrap();

        assert!(r2.total_writes > r1.total_writes);
    }

    #[test]
    fn run_error_on_irq_error_stops_early() {
        let mut e = fresh();
        e.configure(3, 16, 4, 1, 0).unwrap();
        e.driver_mut()
            .handler_mut()
            .driver_mut()
            .mmio_mut()
            .latch_irq(csr_map::IRQ_ERROR_MASK);
        let report = e.run(4).unwrap();
        assert!(report.layers_completed < report.total_layers);
        assert!(report.error_layer.is_some());
    }

    #[test]
    fn run_returns_error_on_zero_rounds() {
        let mut e = fresh();
        e.configure(1, 16, 4, 1, 0).unwrap();
        let result = e.run(0);
        assert_eq!(result, Err(DriverError::Timeout));
    }

    #[test]
    fn five_layers_all_complete() {
        let mut e = fresh();
        e.configure(5, 4, 2, 1, 0).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.layers_completed, 5);
        assert_eq!(report.error_layer, None);
    }

    #[test]
    fn one_layer_one_neuron_one_chunk() {
        let mut e = fresh();
        e.configure(1, 1, 1, 0, 0).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.layers_completed, 1);
    }

    #[test]
    fn threshold_zero_is_valid() {
        let mut e = fresh();
        e.configure(1, 4, 2, 0, 0).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.layers_completed, 1);
    }

    #[test]
    fn large_weight_addr_wraps() {
        let mut e = fresh();
        e.configure(1, 4, 2, 1, 0xFFFF_FFFF_FFFF_FFFF).unwrap();
        let report = e.run(4).unwrap();
        assert_eq!(report.layers_completed, 1);
    }

    #[test]
    fn read_count_positive_after_run() {
        let mut e = fresh();
        e.configure(1, 4, 4, 1, 0).unwrap();
        let report = e.run(4).unwrap();
        assert!(report.total_reads >= 1);
    }

    #[test]
    fn writes_increase_monotonically_with_layers() {
        let results: Vec<InferenceReport> = (1..=4)
            .map(|n| {
                let mut e = fresh();
                e.configure(n, 4, 2, 1, 0).unwrap();
                e.run(4).unwrap()
            })
            .collect();
        for w in results.windows(2) {
            assert!(w[1].total_writes > w[0].total_writes);
        }
    }

    #[test]
    fn configure_accepts_max_values() {
        let mut e = fresh();
        assert!(e.configure(u32::MAX, u32::MAX, u32::MAX, u32::MAX, u64::MAX).is_ok());
    }
}
