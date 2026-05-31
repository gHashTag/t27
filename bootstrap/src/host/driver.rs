// ============================================================================
// Host-side BitNet driver (Wave 39, R-HS-1, Closes #784)
//
// `BitnetDriver<M: Mmio>` is a thin, allocator-free, no_std-friendly wrapper
// around an MMIO aperture that mirrors the W36d AXI-Lite slave CSR map.
// It provides a small, well-typed surface that host firmware (or unit tests)
// can use to:
//
//   * configure the inference engine (layers, neurons, chunks, threshold,
//     and 64-bit weight base address),
//   * pulse the start bit,
//   * poll the status register for busy/done/error,
//   * enable / clear interrupts in the W36f interrupt controller,
//   * dump a snapshot of all CSR values for debugging.
//
// The driver itself does not own the MMIO backend: it borrows it mutably so
// the same backend can be inspected from tests (e.g. `MockMmio::log()`).
// ============================================================================

use super::csr_map;
use super::mmio::Mmio;

/// Driver-level error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// Engine reported its `error` STATUS bit during polling.
    EngineError,
    /// Polling budget exhausted before `done` asserted.
    Timeout,
    /// Caller passed a zero or otherwise invalid configuration value.
    InvalidConfig,
}

/// Snapshot of every mapped CSR, captured in one `dump()` pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsrSnapshot {
    /// CTRL register value.
    pub ctrl: u32,
    /// STATUS register value.
    pub status: u32,
    /// IRQ_EN register value.
    pub irq_en: u32,
    /// IRQ_STAT register value.
    pub irq_stat: u32,
    /// NUM_LAYERS register value.
    pub num_layers: u32,
    /// NEURONS register value.
    pub neurons: u32,
    /// CHUNKS register value.
    pub chunks: u32,
    /// THRESHOLD register value.
    pub threshold: u32,
    /// WEIGHT_ADDR_LO register value.
    pub weight_addr_lo: u32,
    /// WEIGHT_ADDR_HI register value.
    pub weight_addr_hi: u32,
}

impl CsrSnapshot {
    /// Reassemble the 64-bit weight base address.
    pub fn weight_addr_64(&self) -> u64 {
        ((self.weight_addr_hi as u64) << 32) | (self.weight_addr_lo as u64)
    }
}

/// Host-side driver for the BitNet AXI-Lite CSR aperture.
pub struct BitnetDriver<M: Mmio> {
    mmio: M,
}

impl<M: Mmio> BitnetDriver<M> {
    /// Construct a driver around the given MMIO backend.
    pub fn new(mmio: M) -> Self {
        Self { mmio }
    }

    /// Borrow the underlying MMIO backend (for test introspection).
    pub fn mmio(&self) -> &M {
        &self.mmio
    }

    /// Borrow the underlying MMIO backend mutably (for test scaffolding).
    pub fn mmio_mut(&mut self) -> &mut M {
        &mut self.mmio
    }

    /// Consume the driver and return the backend.
    pub fn into_mmio(self) -> M {
        self.mmio
    }

    /// Program the inference parameters.  The caller is responsible for
    /// ensuring values fit the underlying CSR widths; this routine only
    /// rejects zero counts via `DriverError::InvalidConfig`.
    pub fn configure(
        &mut self,
        num_layers: u32,
        neurons: u32,
        chunks: u32,
        threshold: u32,
        weight_addr_64: u64,
    ) -> Result<(), DriverError> {
        if num_layers == 0 || neurons == 0 || chunks == 0 {
            return Err(DriverError::InvalidConfig);
        }
        self.mmio.write32(csr_map::NUM_LAYERS, num_layers);
        self.mmio.write32(csr_map::NEURONS, neurons);
        self.mmio.write32(csr_map::CHUNKS, chunks);
        self.mmio.write32(csr_map::THRESHOLD, threshold);
        self.mmio
            .write32(csr_map::WEIGHT_ADDR_LO, weight_addr_64 as u32);
        self.mmio
            .write32(csr_map::WEIGHT_ADDR_HI, (weight_addr_64 >> 32) as u32);
        Ok(())
    }

    /// Pulse the CTRL start bit (write CTRL_START_MASK to CTRL).
    pub fn start(&mut self) {
        self.mmio.write32(csr_map::CTRL, csr_map::CTRL_START_MASK);
    }

    /// Read the STATUS register and test the `busy` bit.
    pub fn is_busy(&mut self) -> bool {
        self.mmio.read32(csr_map::STATUS) & csr_map::STATUS_BUSY_MASK != 0
    }

    /// Read the STATUS register and test the `done` bit.
    pub fn is_done(&mut self) -> bool {
        self.mmio.read32(csr_map::STATUS) & csr_map::STATUS_DONE_MASK != 0
    }

    /// Read the STATUS register and test the `error` bit.
    pub fn has_error(&mut self) -> bool {
        self.mmio.read32(csr_map::STATUS) & csr_map::STATUS_ERROR_MASK != 0
    }

    /// Poll STATUS up to `max_polls` times waiting for `done`.
    ///
    /// Returns:
    ///   * `Ok(())` once `done` is observed,
    ///   * `Err(EngineError)` if `error` asserts before `done`,
    ///   * `Err(Timeout)` if neither asserts within the budget.
    pub fn wait_done(&mut self, max_polls: u32) -> Result<(), DriverError> {
        for _ in 0..max_polls {
            let s = self.mmio.read32(csr_map::STATUS);
            if s & csr_map::STATUS_ERROR_MASK != 0 {
                return Err(DriverError::EngineError);
            }
            if s & csr_map::STATUS_DONE_MASK != 0 {
                return Ok(());
            }
        }
        Err(DriverError::Timeout)
    }

    /// Write the IRQ enable mask.
    pub fn enable_irqs(&mut self, mask: u32) {
        self.mmio.write32(csr_map::IRQ_EN, mask & csr_map::IRQ_ALL_MASK);
    }

    /// Read the sticky IRQ status latch.
    ///
    /// **DESTRUCTIVE on hardware (W57).** The W36d RTL slave generates a
    /// `status_read` pulse on every AXI read of offset 0x0C, and the
    /// `bitnet_irq` module clears *all* `irq_status` bits on that pulse
    /// (see `bootstrap/src/bitnet_irq.rs` around line 103). The host
    /// therefore cannot inspect IRQ_STAT without also clearing every
    /// latched bit. Callers must either consume the returned value as the
    /// authoritative snapshot of which IRQs were pending at the moment of
    /// the read, or buffer it themselves -- the latches are gone after
    /// the read returns. `MockMmio` mirrors this destructive behaviour.
    pub fn read_irq_status(&mut self) -> u32 {
        self.mmio.read32(csr_map::IRQ_STAT)
    }

    /// Deprecated W57: `clear_irq` is a no-op on hardware.
    ///
    /// The W36d AXI slave has no write case for offset 0x0C
    /// (`bootstrap/src/bitnet_axi.rs`), so writes to IRQ_STAT are silently
    /// dropped. Bits are cleared by READING IRQ_STAT (read-to-clear) via
    /// [`Self::read_irq_status`]. This method is retained as a no-op for
    /// source compatibility with pre-W57 callers; new code must use
    /// `read_irq_status` for both inspection and acknowledgement.
    #[deprecated(
        since = "0.1.0",
        note = "W57: IRQ_STAT is read-to-clear; a write is a no-op on hardware. Use read_irq_status() for both inspection and acknowledgement."
    )]
    pub fn clear_irq(&mut self, _mask: u32) {
        // Intentionally empty: a write to IRQ_STAT is dropped by the AXI
        // slave. We do not even emit the transaction so the host write
        // log no longer carries the misleading W1C record.
    }

    /// Capture a snapshot of all 10 CSRs (10 reads).
    pub fn dump(&mut self) -> CsrSnapshot {
        CsrSnapshot {
            ctrl: self.mmio.read32(csr_map::CTRL),
            status: self.mmio.read32(csr_map::STATUS),
            irq_en: self.mmio.read32(csr_map::IRQ_EN),
            irq_stat: self.mmio.read32(csr_map::IRQ_STAT),
            num_layers: self.mmio.read32(csr_map::NUM_LAYERS),
            neurons: self.mmio.read32(csr_map::NEURONS),
            chunks: self.mmio.read32(csr_map::CHUNKS),
            threshold: self.mmio.read32(csr_map::THRESHOLD),
            weight_addr_lo: self.mmio.read32(csr_map::WEIGHT_ADDR_LO),
            weight_addr_hi: self.mmio.read32(csr_map::WEIGHT_ADDR_HI),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::mmio::MockMmio;

    fn fresh() -> BitnetDriver<MockMmio> {
        BitnetDriver::new(MockMmio::with_csrs_zeroed())
    }

    #[test]
    fn configure_writes_six_registers() {
        let mut d = fresh();
        d.mmio_mut().clear_log();
        d.configure(2, 64, 8, 5, 0x1_0000_2000).unwrap();
        assert_eq!(d.mmio().write_count(), 6);
    }

    #[test]
    fn configure_rejects_zero_layers() {
        let mut d = fresh();
        assert_eq!(d.configure(0, 1, 1, 0, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn configure_rejects_zero_neurons() {
        let mut d = fresh();
        assert_eq!(d.configure(1, 0, 1, 0, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn configure_rejects_zero_chunks() {
        let mut d = fresh();
        assert_eq!(d.configure(1, 1, 0, 0, 0), Err(DriverError::InvalidConfig));
    }

    #[test]
    fn start_pulses_ctrl_start_bit() {
        let mut d = fresh();
        d.start();
        assert_eq!(d.mmio().peek(csr_map::CTRL), csr_map::CTRL_START_MASK);
    }

    #[test]
    fn is_busy_reads_status_bit() {
        let mut d = fresh();
        d.mmio_mut().set_busy(true);
        assert!(d.is_busy());
        d.mmio_mut().set_busy(false);
        assert!(!d.is_busy());
    }

    #[test]
    fn wait_done_succeeds_on_done() {
        let mut d = fresh();
        d.mmio_mut().set_done(true);
        assert_eq!(d.wait_done(8), Ok(()));
    }

    #[test]
    fn wait_done_returns_engine_error() {
        let mut d = fresh();
        d.mmio_mut().set_error(true);
        assert_eq!(d.wait_done(8), Err(DriverError::EngineError));
    }

    #[test]
    fn wait_done_times_out_when_idle() {
        let mut d = fresh();
        assert_eq!(d.wait_done(4), Err(DriverError::Timeout));
    }

    #[test]
    fn enable_irqs_masks_extra_bits() {
        let mut d = fresh();
        d.enable_irqs(0xFFFF_FFFF);
        assert_eq!(d.mmio().peek(csr_map::IRQ_EN), csr_map::IRQ_ALL_MASK);
    }

    #[test]
    fn dump_returns_full_snapshot() {
        let mut d = fresh();
        d.configure(3, 16, 4, 1, 0x2000_0000_1000_0000).unwrap();
        let snap = d.dump();
        assert_eq!(snap.num_layers, 3);
        assert_eq!(snap.neurons, 16);
        assert_eq!(snap.chunks, 4);
        assert_eq!(snap.threshold, 1);
        assert_eq!(snap.weight_addr_64(), 0x2000_0000_1000_0000);
    }
}
