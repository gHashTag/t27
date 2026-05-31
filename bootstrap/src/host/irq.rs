// ============================================================================
// Host IRQ-handler harness (Wave 40, R-HS-2, Closes #786)
//
// Builds on top of `host::BitnetDriver` (W39) to provide an interrupt-driven
// completion path that mirrors what a real PS-side firmware would do against
// the W36d AXI-Lite slave + W36f `interrupt_controller`:
//
//   1. Host enables IRQs via `IRQ_EN` and arms the engine via `CTRL.start`.
//   2. Hardware latches `IRQ_STAT` bits as events occur (sticky).
//   3. Host CPU receives an interrupt, calls `IrqHandler::service()`, which:
//        a. reads IRQ_STAT,
//        b. dispatches a callback per latched source,
//        c. relies on the read-to-clear semantics of the W36d slave
//           (the single IRQ_STAT read clears all sticky latches).  W57.
//   4. `IrqDrivenDriver::wait_done_irq` loops over `service()` until
//      `InferenceDone` fires or a budget is exhausted.
//
// `MockMmio` is reused unchanged; tests inject pending IRQs via
// `MockMmio::latch_irq` between service rounds.
// ============================================================================

use super::csr_map;
use super::driver::{BitnetDriver, DriverError};
use super::mmio::Mmio;

/// Logical IRQ sources exposed by the W36f interrupt controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqSource {
    /// Inference finished (STATUS.done was latched into IRQ_STAT[0]).
    InferenceDone,
    /// DMA transfer finished (IRQ_STAT[1]).
    DmaDone,
    /// Error condition latched (IRQ_STAT[2]).
    Error,
}

impl IrqSource {
    /// Decode the IRQ_STAT bit mask for this source.
    pub fn mask(self) -> u32 {
        match self {
            IrqSource::InferenceDone => csr_map::IRQ_INFERENCE_DONE_MASK,
            IrqSource::DmaDone => csr_map::IRQ_DMA_DONE_MASK,
            IrqSource::Error => csr_map::IRQ_ERROR_MASK,
        }
    }

    /// Iterate every known source in stable (bit-position) order.
    pub fn all() -> [IrqSource; 3] {
        [
            IrqSource::InferenceDone,
            IrqSource::DmaDone,
            IrqSource::Error,
        ]
    }
}

/// Result of one `IrqHandler::service` round.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ServiceReport {
    /// Raw IRQ_STAT value read at the top of the round.
    pub raw_status: u32,
    /// Number of callbacks dispatched (= number of latched bits that had
    /// a registered handler).
    pub dispatched: u32,
    /// True if `InferenceDone` was among the dispatched sources.
    pub inference_done: bool,
    /// True if `Error` was among the dispatched sources.
    pub error: bool,
}

/// Type of a callback bound to a specific `IrqSource`.
///
/// Callbacks take no state and return nothing; they are intended to perform
/// either side-effect bookkeeping (e.g. logging) or to flip a flag the host
/// loop can poll between service rounds.
pub type IrqCallback = fn();

/// Per-source counters used by the default `IrqHandler::with_counters()` setup.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IrqCounters {
    /// How many times an `InferenceDone` callback fired.
    pub inference_done: u32,
    /// How many times a `DmaDone` callback fired.
    pub dma_done: u32,
    /// How many times an `Error` callback fired.
    pub error: u32,
}

/// Callback registry keyed by `IrqSource`.
///
/// Borrows the MMIO backend mutably during `service()` so it can read /
/// rely on the read-to-clear semantics of IRQ_STAT (the read inside
/// `service` clears all sticky latches).  Callbacks are `fn()` (stateless) by design --
/// stateful tracking is done via the borrowed `IrqCounters` returned by
/// `take_counters` after every service round.
pub struct IrqHandler {
    inference_done: Option<IrqCallback>,
    dma_done: Option<IrqCallback>,
    error: Option<IrqCallback>,
}

impl IrqHandler {
    /// Empty registry; no callbacks registered.
    pub fn new() -> Self {
        Self {
            inference_done: None,
            dma_done: None,
            error: None,
        }
    }

    /// Register (or replace) a callback for the given source.
    pub fn register(&mut self, src: IrqSource, cb: IrqCallback) {
        match src {
            IrqSource::InferenceDone => self.inference_done = Some(cb),
            IrqSource::DmaDone => self.dma_done = Some(cb),
            IrqSource::Error => self.error = Some(cb),
        }
    }

    /// Remove a callback for the given source.
    pub fn unregister(&mut self, src: IrqSource) {
        match src {
            IrqSource::InferenceDone => self.inference_done = None,
            IrqSource::DmaDone => self.dma_done = None,
            IrqSource::Error => self.error = None,
        }
    }

    /// Is a callback currently bound to `src`?
    pub fn is_registered(&self, src: IrqSource) -> bool {
        match src {
            IrqSource::InferenceDone => self.inference_done.is_some(),
            IrqSource::DmaDone => self.dma_done.is_some(),
            IrqSource::Error => self.error.is_some(),
        }
    }

    /// Read IRQ_STAT once and dispatch callbacks for every latched bit
    /// that has a registered handler.
    ///
    /// **W57 behaviour change.** IRQ_STAT is read-to-clear on hardware --
    /// the single read inside this function clears ALL sticky bits, not
    /// just the serviced ones. This includes any bit that is latched but
    /// has no registered handler: that event is consumed by the read and
    /// reported in `raw_status` so the caller can still observe it, but
    /// it cannot be re-read from IRQ_STAT afterwards. Previously this
    /// function also emitted a write-1-to-clear to IRQ_STAT after
    /// dispatch; that write was a no-op on hardware (the AXI slave has
    /// no write case for offset 0x0C) and only added a misleading entry
    /// to the MMIO log on the mock. It has been removed.
    pub fn service<M: Mmio>(&self, mmio: &mut M) -> ServiceReport {
        let raw = mmio.read32(csr_map::IRQ_STAT);
        let mut report = ServiceReport {
            raw_status: raw,
            ..Default::default()
        };
        for src in IrqSource::all() {
            let m = src.mask();
            if raw & m == 0 {
                continue;
            }
            let cb = match src {
                IrqSource::InferenceDone => self.inference_done,
                IrqSource::DmaDone => self.dma_done,
                IrqSource::Error => self.error,
            };
            if let Some(f) = cb {
                f();
                report.dispatched += 1;
                match src {
                    IrqSource::InferenceDone => report.inference_done = true,
                    IrqSource::Error => report.error = true,
                    _ => {}
                }
            }
        }
        report
    }
}

impl Default for IrqHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver wrapper that completes work via the IRQ path instead of polling.
///
/// `wait_done_irq` calls `IrqHandler::service` until either:
///   * the `InferenceDone` callback has fired (success), or
///   * the `Error` callback has fired (returns `DriverError::EngineError`), or
///   * the round budget is exhausted (returns `DriverError::Timeout`).
///
/// Tests advance time by latching pending IRQs into `MockMmio` between
/// service rounds; production code would attach to a real CPU IRQ line.
pub struct IrqDrivenDriver<M: Mmio> {
    inner: BitnetDriver<M>,
    handler: IrqHandler,
}

impl<M: Mmio> IrqDrivenDriver<M> {
    /// Construct from a base driver and an IRQ handler.
    pub fn new(inner: BitnetDriver<M>, handler: IrqHandler) -> Self {
        Self { inner, handler }
    }

    /// Borrow the inner driver.
    pub fn driver(&self) -> &BitnetDriver<M> {
        &self.inner
    }

    /// Borrow the inner driver mutably.
    pub fn driver_mut(&mut self) -> &mut BitnetDriver<M> {
        &mut self.inner
    }

    /// Borrow the IRQ handler mutably (for callback registration).
    pub fn handler_mut(&mut self) -> &mut IrqHandler {
        &mut self.handler
    }

    /// Consume the wrapper and return both halves.
    pub fn into_parts(self) -> (BitnetDriver<M>, IrqHandler) {
        (self.inner, self.handler)
    }

    /// Wait for completion via the IRQ path.
    ///
    /// Each round invokes `handler.service(mmio)` once.  If
    /// `InferenceDone` fires, returns `Ok(())`.  If `Error` fires,
    /// returns `Err(EngineError)`.  Otherwise loops up to
    /// `max_service_rounds` times before returning `Err(Timeout)`.
    pub fn wait_done_irq(&mut self, max_service_rounds: u32) -> Result<(), DriverError> {
        for _ in 0..max_service_rounds {
            let rep = self.handler.service(self.inner.mmio_mut());
            if rep.error {
                return Err(DriverError::EngineError);
            }
            if rep.inference_done {
                return Ok(());
            }
        }
        Err(DriverError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::driver::BitnetDriver;
    use crate::host::mmio::MockMmio;

    fn noop() {}

    fn fresh_driver() -> BitnetDriver<MockMmio> {
        BitnetDriver::new(MockMmio::with_csrs_zeroed())
    }

    #[test]
    fn irq_source_masks_match_csr_map() {
        assert_eq!(
            IrqSource::InferenceDone.mask(),
            csr_map::IRQ_INFERENCE_DONE_MASK
        );
        assert_eq!(IrqSource::DmaDone.mask(), csr_map::IRQ_DMA_DONE_MASK);
        assert_eq!(IrqSource::Error.mask(), csr_map::IRQ_ERROR_MASK);
    }

    #[test]
    fn irq_source_all_returns_three_distinct() {
        let all = IrqSource::all();
        assert_eq!(all.len(), 3);
        assert_ne!(all[0], all[1]);
        assert_ne!(all[1], all[2]);
        assert_ne!(all[0], all[2]);
    }

    #[test]
    fn handler_new_is_empty() {
        let h = IrqHandler::new();
        for s in IrqSource::all() {
            assert!(!h.is_registered(s));
        }
    }

    #[test]
    fn handler_register_marks_source_registered() {
        let mut h = IrqHandler::new();
        h.register(IrqSource::InferenceDone, noop);
        assert!(h.is_registered(IrqSource::InferenceDone));
        assert!(!h.is_registered(IrqSource::DmaDone));
    }

    #[test]
    fn handler_unregister_clears_source() {
        let mut h = IrqHandler::new();
        h.register(IrqSource::Error, noop);
        h.unregister(IrqSource::Error);
        assert!(!h.is_registered(IrqSource::Error));
    }

    #[test]
    fn service_with_no_pending_irqs_dispatches_nothing() {
        let mut m = MockMmio::with_csrs_zeroed();
        let h = IrqHandler::new();
        let rep = h.service(&mut m);
        assert_eq!(rep.dispatched, 0);
        assert!(!rep.inference_done);
        assert!(!rep.error);
    }

    #[test]
    fn service_dispatches_registered_inference_done() {
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        let mut h = IrqHandler::new();
        h.register(IrqSource::InferenceDone, noop);
        let rep = h.service(&mut m);
        assert_eq!(rep.dispatched, 1);
        assert!(rep.inference_done);
        // Sticky bit was cleared by the read-to-clear inside service().
        assert_eq!(m.peek(csr_map::IRQ_STAT) & csr_map::IRQ_INFERENCE_DONE_MASK, 0);
    }

    #[test]
    fn service_skips_latched_but_unregistered_source_w57() {
        // W57: IRQ_STAT is read-to-clear on hardware. The single read
        // inside `service` clears ALL sticky bits, including the
        // unregistered one. The bit is reported in `raw_status` so the
        // caller can still observe the event, but it is not redispatched
        // and not visible in IRQ_STAT after service returns.
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_DMA_DONE_MASK);
        let h = IrqHandler::new(); // no callbacks
        let rep = h.service(&mut m);
        assert_eq!(rep.dispatched, 0);
        // raw_status captured the pre-clear value so the caller can see
        // the dropped event.
        assert_ne!(rep.raw_status & csr_map::IRQ_DMA_DONE_MASK, 0);
        // Latch was consumed by the read.
        assert_eq!(m.peek(csr_map::IRQ_STAT) & csr_map::IRQ_DMA_DONE_MASK, 0);
    }

    #[test]
    fn service_dispatches_three_sources_simultaneously() {
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_ALL_MASK);
        let mut h = IrqHandler::new();
        for s in IrqSource::all() {
            h.register(s, noop);
        }
        let rep = h.service(&mut m);
        assert_eq!(rep.dispatched, 3);
        assert!(rep.inference_done && rep.error);
        assert_eq!(m.peek(csr_map::IRQ_STAT), 0);
    }

    #[test]
    fn service_raw_status_reflects_irq_stat_pre_clear() {
        let mut m = MockMmio::with_csrs_zeroed();
        m.latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK | csr_map::IRQ_DMA_DONE_MASK);
        let mut h = IrqHandler::new();
        h.register(IrqSource::InferenceDone, noop);
        let rep = h.service(&mut m);
        assert_eq!(
            rep.raw_status,
            csr_map::IRQ_INFERENCE_DONE_MASK | csr_map::IRQ_DMA_DONE_MASK
        );
    }

    #[test]
    fn irq_driver_completes_when_inference_done_latched() {
        let mut d = IrqDrivenDriver::new(fresh_driver(), IrqHandler::new());
        d.handler_mut().register(IrqSource::InferenceDone, noop);
        d.driver_mut()
            .mmio_mut()
            .latch_irq(csr_map::IRQ_INFERENCE_DONE_MASK);
        assert_eq!(d.wait_done_irq(4), Ok(()));
    }

    #[test]
    fn irq_driver_returns_engine_error_when_error_latched() {
        let mut d = IrqDrivenDriver::new(fresh_driver(), IrqHandler::new());
        d.handler_mut().register(IrqSource::Error, noop);
        d.driver_mut().mmio_mut().latch_irq(csr_map::IRQ_ERROR_MASK);
        assert_eq!(d.wait_done_irq(4), Err(DriverError::EngineError));
    }

    #[test]
    fn irq_driver_times_out_when_no_irq_arrives() {
        let mut d = IrqDrivenDriver::new(fresh_driver(), IrqHandler::new());
        d.handler_mut().register(IrqSource::InferenceDone, noop);
        assert_eq!(d.wait_done_irq(3), Err(DriverError::Timeout));
    }
}
