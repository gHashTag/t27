// ============================================================================
// MMIO abstraction for the host-side BitNet driver (Wave 39, R-HS-1, Closes #784)
//
// The `Mmio` trait provides a minimal 32-bit aligned read/write interface to a
// CSR aperture.  In production a thin wrapper around `read_volatile` /
// `write_volatile` on `*mut u32` would implement this trait.  For host-side
// unit testing we provide `MockMmio`, a deterministic in-memory backend that
// (a) stores CSR values in a `BTreeMap<u32, u32>`,
// (b) records every transaction in a `Vec<MmioOp>` for assertion,
// (c) honours the W36d AXI-Lite slave contract on unmapped reads
//     (returns `csr_map::UNMAPPED_READ_VALUE`),
// (d) exposes hooks (`set_busy`, `set_done`, `set_error`, `latch_irq`) that
//     simulate side effects the real BitNet engine would produce in hardware.
//
// All addresses are byte offsets within the CSR aperture base; the driver
// never adds a separate base address (the trait implementation owns the base).
// ============================================================================

use std::collections::BTreeMap;

use super::csr_map;

/// Kind of an MMIO transaction recorded by `MockMmio`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioOp {
    /// 32-bit read at the given byte offset.
    Read,
    /// 32-bit write at the given byte offset.
    Write,
}

/// One recorded MMIO transaction: kind, byte offset, observed/written value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MmioRecord {
    /// Kind of transaction.
    pub op: MmioOp,
    /// Byte offset within the CSR aperture.
    pub addr: u32,
    /// Value read or written.
    pub value: u32,
}

/// Minimal MMIO interface used by the host driver.
///
/// Implementors must guarantee 32-bit aligned, word-sized accesses.
/// Unaligned accesses are a programming error and may panic in debug builds.
pub trait Mmio {
    /// Read a 32-bit word from `addr`.
    fn read32(&mut self, addr: u32) -> u32;

    /// Write a 32-bit word to `addr`.
    fn write32(&mut self, addr: u32, value: u32);
}

/// In-memory mock implementing `Mmio` for host-side unit tests.
///
/// Stores CSR values in a sparse map.  Unmapped reads return
/// `csr_map::UNMAPPED_READ_VALUE` (mirroring the W36d AXI-Lite slave).
/// Every transaction is appended to an internal log for later inspection.
#[derive(Debug, Default, Clone)]
pub struct MockMmio {
    /// Backing store for CSR values, keyed by byte offset.
    regs: BTreeMap<u32, u32>,
    /// Ordered log of every read/write that has occurred.
    log: Vec<MmioRecord>,
}

impl MockMmio {
    /// Construct an empty mock with no registers initialised.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a mock pre-populated with the canonical CSR offsets zeroed.
    pub fn with_csrs_zeroed() -> Self {
        let mut m = Self::new();
        for off in csr_map::CSR_OFFSETS.iter() {
            m.regs.insert(*off, 0);
        }
        m
    }

    /// Borrow the recorded transaction log.
    pub fn log(&self) -> &[MmioRecord] {
        &self.log
    }

    /// Clear the recorded transaction log.  Register state is untouched.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    /// Number of recorded reads.
    pub fn read_count(&self) -> usize {
        self.log.iter().filter(|r| r.op == MmioOp::Read).count()
    }

    /// Number of recorded writes.
    pub fn write_count(&self) -> usize {
        self.log.iter().filter(|r| r.op == MmioOp::Write).count()
    }

    /// Force the busy bit of the STATUS register.
    pub fn set_busy(&mut self, busy: bool) {
        self.set_status_bit(csr_map::STATUS_BUSY_MASK, busy);
    }

    /// Force the done bit of the STATUS register.
    pub fn set_done(&mut self, done: bool) {
        self.set_status_bit(csr_map::STATUS_DONE_MASK, done);
    }

    /// Force the error bit of the STATUS register.
    pub fn set_error(&mut self, err: bool) {
        self.set_status_bit(csr_map::STATUS_ERROR_MASK, err);
    }

    /// OR a mask into the IRQ_STAT register (simulates an HW latch).
    pub fn latch_irq(&mut self, mask: u32) {
        let cur = *self.regs.get(&csr_map::IRQ_STAT).unwrap_or(&0);
        self.regs.insert(csr_map::IRQ_STAT, cur | mask);
    }

    /// Inspect the current value at `addr` without recording a transaction.
    pub fn peek(&self, addr: u32) -> u32 {
        *self.regs.get(&addr).unwrap_or(&csr_map::UNMAPPED_READ_VALUE)
    }

    /// Overwrite a register without recording a transaction (test scaffolding).
    pub fn poke(&mut self, addr: u32, value: u32) {
        self.regs.insert(addr, value);
    }

    fn set_status_bit(&mut self, mask: u32, on: bool) {
        let cur = *self.regs.get(&csr_map::STATUS).unwrap_or(&0);
        let next = if on { cur | mask } else { cur & !mask };
        self.regs.insert(csr_map::STATUS, next);
    }
}

impl Mmio for MockMmio {
    fn read32(&mut self, addr: u32) -> u32 {
        debug_assert_eq!(addr % 4, 0, "MMIO read addr {addr:#x} not word-aligned");
        let val = match self.regs.get(&addr) {
            Some(v) => *v,
            None => csr_map::UNMAPPED_READ_VALUE,
        };
        self.log.push(MmioRecord {
            op: MmioOp::Read,
            addr,
            value: val,
        });
        if addr == csr_map::IRQ_STAT {
            // Mirror the W36d RTL: `bitnet_irq` clears ALL `irq_status`
            // bits on the `status_read` pulse generated by an IRQ_STAT
            // read (see `build_interrupt_controller` in
            // bootstrap/src/bitnet_irq.rs). The mock must model the same
            // destructive read so host code exercised against MockMmio
            // behaves identically on silicon.
            //
            // Bound on that "identically": since W555 the RTL OR's the
            // interrupt sources on top of the cleared value, so an event
            // raised in the SAME cycle as the acknowledging read survives
            // it. This mock is untimed -- it has no notion of a cycle, and
            // so no way to raise an event concurrently with a read -- and
            // therefore cannot exhibit that case either way. It models the
            // no-concurrent-event behaviour, which is unchanged.
            self.regs.insert(addr, 0);
        }
        val
    }

    fn write32(&mut self, addr: u32, value: u32) {
        debug_assert_eq!(addr % 4, 0, "MMIO write addr {addr:#x} not word-aligned");
        if addr == csr_map::IRQ_STAT {
            // Mirror the W36d AXI slave: there is no write case for offset
            // 0x0C (see bootstrap/src/bitnet_axi.rs), so writes to IRQ_STAT
            // are silently dropped on hardware. The mock matches by
            // recording the transaction in the log but leaving the
            // sticky-latch register untouched -- bits are cleared by a
            // READ via the read-to-clear path, not by a write.
        } else {
            self.regs.insert(addr, value);
        }
        self.log.push(MmioRecord {
            op: MmioOp::Write,
            addr,
            value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mock_is_empty() {
        let m = MockMmio::new();
        assert!(m.log().is_empty());
        assert_eq!(m.read_count(), 0);
        assert_eq!(m.write_count(), 0);
    }

    #[test]
    fn unmapped_read_returns_sentinel() {
        let mut m = MockMmio::new();
        assert_eq!(m.read32(0x80), csr_map::UNMAPPED_READ_VALUE);
    }

    #[test]
    fn write_then_read_roundtrip() {
        let mut m = MockMmio::new();
        m.write32(csr_map::NUM_LAYERS, 7);
        assert_eq!(m.read32(csr_map::NUM_LAYERS), 7);
    }

    #[test]
    fn log_records_read_and_write_in_order() {
        let mut m = MockMmio::new();
        m.write32(csr_map::CTRL, 1);
        let _ = m.read32(csr_map::STATUS);
        let log = m.log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].op, MmioOp::Write);
        assert_eq!(log[0].addr, csr_map::CTRL);
        assert_eq!(log[0].value, 1);
        assert_eq!(log[1].op, MmioOp::Read);
        assert_eq!(log[1].addr, csr_map::STATUS);
    }

    #[test]
    fn clear_log_does_not_clear_regs() {
        let mut m = MockMmio::new();
        m.write32(csr_map::NEURONS, 42);
        m.clear_log();
        assert!(m.log().is_empty());
        assert_eq!(m.read32(csr_map::NEURONS), 42);
    }

    #[test]
    fn with_csrs_zeroed_initialises_all_offsets() {
        let m = MockMmio::with_csrs_zeroed();
        for off in csr_map::CSR_OFFSETS.iter() {
            assert_eq!(m.peek(*off), 0, "offset {off:#x} not zeroed");
        }
    }

    #[test]
    fn set_status_bits_compose() {
        let mut m = MockMmio::with_csrs_zeroed();
        m.set_busy(true);
        m.set_done(true);
        let s = m.peek(csr_map::STATUS);
        assert_ne!(s & csr_map::STATUS_BUSY_MASK, 0);
        assert_ne!(s & csr_map::STATUS_DONE_MASK, 0);
        assert_eq!(s & csr_map::STATUS_ERROR_MASK, 0);
    }

    #[test]
    fn latch_irq_is_sticky_or() {
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        m.latch_irq(csr_map::IRQ_DMA_DONE_MASK);
        let s = m.peek(csr_map::IRQ_STAT);
        assert_eq!(
            s,
            csr_map::IRQ_INFERENCE_DONE_MASK | csr_map::IRQ_DMA_DONE_MASK
        );
    }

    #[test]
    fn read_write_counts_match_log() {
        let mut m = MockMmio::new();
        m.write32(0x00, 1);
        m.write32(0x10, 2);
        m.write32(0x14, 3);
        let _ = m.read32(0x04);
        let _ = m.read32(0x0C);
        assert_eq!(m.write_count(), 3);
        assert_eq!(m.read_count(), 2);
        assert_eq!(m.log().len(), 5);
    }

    #[test]
    fn peek_does_not_record_transaction() {
        let mut m = MockMmio::with_csrs_zeroed();
        let _ = m.peek(csr_map::STATUS);
        assert!(m.log().is_empty());
    }

    #[test]
    fn read_irq_stat_is_destructive_w57() {
        // W57: the W36d RTL slave clears ALL irq_status bits on the
        // status_read pulse generated by an AXI read of offset 0x0C.
        // The mock must mirror this so host code exercised against
        // MockMmio behaves identically on silicon.
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_ALL_MASK);
        assert_eq!(m.peek(csr_map::IRQ_STAT), csr_map::IRQ_ALL_MASK);
        // First read returns the latched value.
        assert_eq!(m.read32(csr_map::IRQ_STAT), csr_map::IRQ_ALL_MASK);
        // The read cleared every sticky bit.
        assert_eq!(m.peek(csr_map::IRQ_STAT), 0);
        // A second read returns 0 and is non-destructive (already 0).
        assert_eq!(m.read32(csr_map::IRQ_STAT), 0);
    }

    #[test]
    fn write_irq_stat_is_dropped_w57() {
        // W57: the AXI slave has no write case for offset 0x0C, so any
        // write to IRQ_STAT is silently dropped on hardware. The mock
        // matches: the transaction is recorded in the log (so host
        // bookkeeping still sees the bus activity), but the sticky-latch
        // register is left untouched. The previous W1C model was wrong.
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(m.peek(csr_map::IRQ_STAT), csr_map::IRQ_INFERENCE_DONE_MASK);
        // Even writing the bit-to-clear leaves the latch in place.
        m.write32(csr_map::IRQ_STAT, csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(m.peek(csr_map::IRQ_STAT), csr_map::IRQ_INFERENCE_DONE_MASK);
        // The transaction was still logged.
        assert_eq!(m.write_count(), 1);
    }
}
