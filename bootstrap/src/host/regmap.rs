use super::csr_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtrlReg(u32);

impl CtrlReg {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn start(self) -> bool {
        (self.0 & csr_map::CTRL_START_MASK) != 0
    }

    pub const fn with_start(mut self) -> Self {
        self.0 |= csr_map::CTRL_START_MASK;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusReg(u32);

impl StatusReg {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn busy(self) -> bool {
        (self.0 & csr_map::STATUS_BUSY_MASK) != 0
    }

    pub const fn done(self) -> bool {
        (self.0 & csr_map::STATUS_DONE_MASK) != 0
    }

    pub const fn error(self) -> bool {
        (self.0 & csr_map::STATUS_ERROR_MASK) != 0
    }

    pub const fn is_idle(self) -> bool {
        !self.busy() && !self.done() && !self.error()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrqEnReg(u32);

impl IrqEnReg {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn inference_done(self) -> bool {
        (self.0 & csr_map::IRQ_INFERENCE_DONE_MASK) != 0
    }

    pub const fn dma_done(self) -> bool {
        (self.0 & csr_map::IRQ_DMA_DONE_MASK) != 0
    }

    pub const fn error(self) -> bool {
        (self.0 & csr_map::IRQ_ERROR_MASK) != 0
    }

    pub const fn with_inference_done(mut self) -> Self {
        self.0 |= csr_map::IRQ_INFERENCE_DONE_MASK;
        self
    }

    pub const fn with_dma_done(mut self) -> Self {
        self.0 |= csr_map::IRQ_DMA_DONE_MASK;
        self
    }

    pub const fn with_error(mut self) -> Self {
        self.0 |= csr_map::IRQ_ERROR_MASK;
        self
    }

    pub const fn with_all(mut self) -> Self {
        self.0 |= csr_map::IRQ_ALL_MASK;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrqStatReg(u32);

impl IrqStatReg {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn inference_done(self) -> bool {
        (self.0 & csr_map::IRQ_INFERENCE_DONE_MASK) != 0
    }

    pub const fn dma_done(self) -> bool {
        (self.0 & csr_map::IRQ_DMA_DONE_MASK) != 0
    }

    pub const fn error(self) -> bool {
        (self.0 & csr_map::IRQ_ERROR_MASK) != 0
    }

    pub const fn has_any(self) -> bool {
        (self.0 & csr_map::IRQ_ALL_MASK) != 0
    }

    pub const fn clear_inference_done(mut self) -> Self {
        self.0 |= csr_map::IRQ_INFERENCE_DONE_MASK;
        self
    }

    pub const fn clear_dma_done(mut self) -> Self {
        self.0 |= csr_map::IRQ_DMA_DONE_MASK;
        self
    }

    pub const fn clear_error(mut self) -> Self {
        self.0 |= csr_map::IRQ_ERROR_MASK;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightAddr {
    pub lo: u32,
    pub hi: u32,
}

impl WeightAddr {
    pub const fn new(addr: u64) -> Self {
        Self {
            lo: addr as u32,
            hi: (addr >> 32) as u32,
        }
    }

    pub const fn addr(self) -> u64 {
        (self.hi as u64) << 32 | self.lo as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigSnapshot {
    pub num_layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub threshold: u32,
    pub weight_addr: WeightAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullSnapshot {
    pub ctrl: CtrlReg,
    pub status: StatusReg,
    pub irq_en: IrqEnReg,
    pub irq_stat: IrqStatReg,
    pub config: ConfigSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_new_is_zero() {
        assert_eq!(CtrlReg::new().raw(), 0);
        assert!(!CtrlReg::new().start());
    }

    #[test]
    fn ctrl_with_start() {
        let c = CtrlReg::new().with_start();
        assert!(c.start());
        assert_eq!(c.raw(), csr_map::CTRL_START_MASK);
    }

    #[test]
    fn ctrl_from_raw() {
        let c = CtrlReg::from_raw(0x1);
        assert!(c.start());
    }

    #[test]
    fn status_busy() {
        let s = StatusReg::from_raw(csr_map::STATUS_BUSY_MASK);
        assert!(s.busy());
        assert!(!s.done());
        assert!(!s.error());
    }

    #[test]
    fn status_done() {
        let s = StatusReg::from_raw(csr_map::STATUS_DONE_MASK);
        assert!(!s.busy());
        assert!(s.done());
    }

    #[test]
    fn status_error() {
        let s = StatusReg::from_raw(csr_map::STATUS_ERROR_MASK);
        assert!(s.error());
    }

    #[test]
    fn status_idle() {
        let s = StatusReg::from_raw(0);
        assert!(s.is_idle());
        let s2 = StatusReg::from_raw(csr_map::STATUS_BUSY_MASK);
        assert!(!s2.is_idle());
    }

    #[test]
    fn status_multiple_bits() {
        let s = StatusReg::from_raw(csr_map::STATUS_BUSY_MASK | csr_map::STATUS_ERROR_MASK);
        assert!(s.busy());
        assert!(s.error());
        assert!(!s.done());
    }

    #[test]
    fn irq_en_build() {
        let e = IrqEnReg::new().with_inference_done().with_error();
        assert!(e.inference_done());
        assert!(!e_dma_done(e));
        assert!(e.error());
    }

    fn e_dma_done(e: IrqEnReg) -> bool {
        e.dma_done()
    }

    #[test]
    fn irq_en_all() {
        let e = IrqEnReg::new().with_all();
        assert!(e.inference_done());
        assert!(e.dma_done());
        assert!(e.error());
        assert_eq!(e.raw(), csr_map::IRQ_ALL_MASK);
    }

    #[test]
    fn irq_stat_has_any() {
        let s = IrqStatReg::from_raw(0);
        assert!(!s.has_any());
        let s2 = IrqStatReg::from_raw(csr_map::IRQ_INFERENCE_DONE_MASK);
        assert!(s2.has_any());
    }

    #[test]
    fn irq_stat_individual() {
        let s = IrqStatReg::from_raw(csr_map::IRQ_DMA_DONE_MASK | csr_map::IRQ_ERROR_MASK);
        assert!(!s.inference_done());
        assert!(s.dma_done());
        assert!(s.error());
    }

    #[test]
    fn weight_addr_roundtrip() {
        let a = WeightAddr::new(0x1234_5678_9ABC_DEF0);
        assert_eq!(a.lo, 0x9ABC_DEF0);
        assert_eq!(a.hi, 0x1234_5678);
        assert_eq!(a.addr(), 0x1234_5678_9ABC_DEF0);
    }

    #[test]
    fn weight_addr_zero() {
        let a = WeightAddr::new(0);
        assert_eq!(a.addr(), 0);
    }

    #[test]
    fn config_snapshot_fields() {
        let c = ConfigSnapshot {
            num_layers: 4,
            neurons: 128,
            chunks: 16,
            threshold: 0x00FF,
            weight_addr: WeightAddr::new(0x1000_0000),
        };
        assert_eq!(c.num_layers, 4);
        assert_eq!(c.neurons, 128);
        assert_eq!(c.weight_addr.addr(), 0x1000_0000);
    }

    #[test]
    fn full_snapshot() {
        let snap = FullSnapshot {
            ctrl: CtrlReg::new(),
            status: StatusReg::from_raw(csr_map::STATUS_DONE_MASK),
            irq_en: IrqEnReg::new().with_all(),
            irq_stat: IrqStatReg::from_raw(0),
            config: ConfigSnapshot {
                num_layers: 8,
                neurons: 256,
                chunks: 32,
                threshold: 0,
                weight_addr: WeightAddr::new(0),
            },
        };
        assert!(snap.status.done());
        assert!(!snap.status.busy());
        assert_eq!(snap.config.num_layers, 8);
    }
}
