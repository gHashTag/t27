use super::csr_map;
use super::driver::{BitnetDriver, DriverError};
use super::mmio::Mmio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqSource {
    InferenceDone,
    DmaDone,
    Error,
}

impl IrqSource {
    pub fn mask(self) -> u32 {
        match self {
            IrqSource::InferenceDone => csr_map::IRQ_INFERENCE_DONE_MASK,
            IrqSource::DmaDone => csr_map::IRQ_DMA_DONE_MASK,
            IrqSource::Error => csr_map::IRQ_ERROR_MASK,
        }
    }

    pub fn from_mask(mask: u32) -> Vec<IrqSource> {
        let mut sources = Vec::new();
        if mask & csr_map::IRQ_INFERENCE_DONE_MASK != 0 {
            sources.push(IrqSource::InferenceDone);
        }
        if mask & csr_map::IRQ_DMA_DONE_MASK != 0 {
            sources.push(IrqSource::DmaDone);
        }
        if mask & csr_map::IRQ_ERROR_MASK != 0 {
            sources.push(IrqSource::Error);
        }
        sources
    }
}

type IrqCallback = fn(IrqSource);

pub struct IrqHandler<M: Mmio> {
    driver: BitnetDriver<M>,
    callbacks: [Option<IrqCallback>; 3],
}

impl<M: Mmio> IrqHandler<M> {
    pub fn new(driver: BitnetDriver<M>) -> Self {
        Self {
            driver,
            callbacks: [None, None, None],
        }
    }

    pub fn register(&mut self, source: IrqSource, cb: IrqCallback) {
        let idx = match source {
            IrqSource::InferenceDone => 0,
            IrqSource::DmaDone => 1,
            IrqSource::Error => 2,
        };
        self.callbacks[idx] = Some(cb);
    }

    pub fn service(&mut self) -> u32 {
        let stat = self.driver.read_irq_status();
        if stat == 0 {
            return 0;
        }
        let sources = IrqSource::from_mask(stat);
        for src in sources {
            let idx = match src {
                IrqSource::InferenceDone => 0,
                IrqSource::DmaDone => 1,
                IrqSource::Error => 2,
            };
            if let Some(cb) = self.callbacks[idx] {
                cb(src);
            }
        }
        self.driver.clear_irq(stat);
        stat
    }

    pub fn driver(&self) -> &BitnetDriver<M> {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut BitnetDriver<M> {
        &mut self.driver
    }
}

pub struct IrqDrivenDriver<M: Mmio> {
    handler: IrqHandler<M>,
}

impl<M: Mmio> IrqDrivenDriver<M> {
    pub fn new(driver: BitnetDriver<M>) -> Self {
        Self {
            handler: IrqHandler::new(driver),
        }
    }

    pub fn register(&mut self, source: IrqSource, cb: IrqCallback) {
        self.handler.register(source, cb);
    }

    pub fn handler(&self) -> &IrqHandler<M> {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut IrqHandler<M> {
        &mut self.handler
    }

    pub fn wait_done_irq(&mut self, max_service_rounds: u32) -> Result<(), DriverError> {
        self.wait_irq_mask(csr_map::IRQ_INFERENCE_DONE_MASK, max_service_rounds)
    }

    pub fn wait_irq_mask(&mut self, mask: u32, max_service_rounds: u32) -> Result<(), DriverError> {
        for _ in 0..max_service_rounds {
            let serviced = self.handler.service();
            if serviced & csr_map::IRQ_ERROR_MASK != 0 {
                return Err(DriverError::EngineError);
            }
            if serviced & mask != 0 {
                return Ok(());
            }
            if mask == csr_map::IRQ_INFERENCE_DONE_MASK
                && self.handler.driver_mut().is_done()
            {
                return Ok(());
            }
        }
        Err(DriverError::Timeout)
    }

    pub fn into_handler(self) -> IrqHandler<M> {
        self.handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::mmio::MockMmio;

    thread_local! {
        static FIRED: std::cell::RefCell<Vec<IrqSource>> = std::cell::RefCell::new(Vec::new());
    }

    fn record_cb(src: IrqSource) {
        FIRED.with(|f| f.borrow_mut().push(src));
    }

    fn fired() -> Vec<IrqSource> {
        FIRED.with(|f| f.borrow().clone())
    }

    fn clear_fired() {
        FIRED.with(|f| f.borrow_mut().clear());
    }

    fn fresh_handler() -> IrqHandler<MockMmio> {
        IrqHandler::new(BitnetDriver::new(MockMmio::with_csrs_zeroed()))
    }

    fn fresh_irq_driver() -> IrqDrivenDriver<MockMmio> {
        IrqDrivenDriver::new(BitnetDriver::new(MockMmio::with_csrs_zeroed()))
    }

    #[test]
    fn irq_source_mask_roundtrip() {
        assert_eq!(IrqSource::InferenceDone.mask(), csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(IrqSource::DmaDone.mask(), csr_map::IRQ_DMA_DONE_MASK);
        assert_eq!(IrqSource::Error.mask(), csr_map::IRQ_ERROR_MASK);
    }

    #[test]
    fn from_mask_empty() {
        assert!(IrqSource::from_mask(0).is_empty());
    }

    #[test]
    fn from_mask_all_three() {
        let sources = IrqSource::from_mask(csr_map::IRQ_ALL_MASK);
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn handler_service_no_irqs_returns_zero() {
        let mut h = fresh_handler();
        assert_eq!(h.service(), 0);
    }

    #[test]
    fn handler_service_dispatches_callback() {
        clear_fired();
        let mut h = fresh_handler();
        h.register(IrqSource::InferenceDone, record_cb);
        h.driver_mut().mmio_mut().latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        let serviced = h.service();
        assert_eq!(serviced, csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(fired(), vec![IrqSource::InferenceDone]);
    }

    #[test]
    fn handler_service_calls_clear_irq() {
        let mut h = fresh_handler();
        h.driver_mut().mmio_mut().latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        h.service();
        let log = h.driver().mmio().log();
        let last = log.last().unwrap();
        assert_eq!(last.op, super::super::mmio::MmioOp::Write);
        assert_eq!(last.addr, csr_map::IRQ_STAT);
        assert_eq!(last.value, csr_map::IRQ_INFERENCE_DONE_MASK);
    }

    #[test]
    fn handler_multiple_sources() {
        clear_fired();
        let mut h = fresh_handler();
        h.register(IrqSource::InferenceDone, record_cb);
        h.register(IrqSource::DmaDone, record_cb);
        h.driver_mut().mmio_mut().latch_irq(
            csr_map::IRQ_INFERENCE_DONE_MASK | csr_map::IRQ_DMA_DONE_MASK,
        );
        let serviced = h.service();
        assert_eq!(serviced, csr_map::IRQ_INFERENCE_DONE_MASK | csr_map::IRQ_DMA_DONE_MASK);
        let f = fired();
        assert!(f.contains(&IrqSource::InferenceDone));
        assert!(f.contains(&IrqSource::DmaDone));
    }

    #[test]
    fn irq_driver_wait_done_succeeds_on_inference_done() {
        let mut d = fresh_irq_driver();
        d.handler_mut().driver_mut().mmio_mut().latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(d.wait_done_irq(4), Ok(()));
    }

    #[test]
    fn irq_driver_wait_done_returns_error_on_irq_error() {
        let mut d = fresh_irq_driver();
        d.handler_mut().driver_mut().mmio_mut().latch_irq(csr_map::IRQ_ERROR_MASK);
        assert_eq!(d.wait_done_irq(4), Err(DriverError::EngineError));
    }

    #[test]
    fn irq_driver_wait_done_times_out() {
        let mut d = fresh_irq_driver();
        assert_eq!(d.wait_done_irq(2), Err(DriverError::Timeout));
    }

    #[test]
    fn irq_driver_wait_done_falls_back_to_done_bit() {
        let mut d = fresh_irq_driver();
        d.handler_mut().driver_mut().mmio_mut().set_done(true);
        assert_eq!(d.wait_done_irq(4), Ok(()));
    }
}
