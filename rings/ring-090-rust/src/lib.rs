//! ring-090 -- **HIR cycle-accurate simulator primitives**.
//!
//! Wave 17 (2026-05-22, Closes #721): the third honestly-imported Wave-11
//! crate. Wave 15 landed ring-088 (GF16 MAC); Wave 16 landed ring-089
//! (TNN ISA); Wave 17 lands ring-090: the simulator data-model + helper
//! functions defined by [`specs/fpga/simulator.t27`].
//!
//! ## Scope
//!
//! Pure data types and helpers, faithful to the spec:
//!
//! * [`SimState`] -- 5-variant simulation state (idle/running/paused/done/error).
//! * [`SimConfig`] -- simulator configuration (name, max cycles, clock
//!   frequency, trace flags, VCD path).
//! * [`SimResult`] -- post-run report (cycles, terminal state, error counts,
//!   assertions fired, coverage points).
//! * [`ProbePoint`] -- a single named signal probe with width and signedness.
//! * [`TraceEntry`] -- a single `(cycle, signal, value)` trace record.
//! * Constructor helpers ([`sim_config`], [`sim_config_with_trace`],
//!   [`sim_ok`], [`sim_error`], [`probe`], [`trace_entry`]).
//! * Query predicates ([`is_idle`], [`is_done`], [`is_error`],
//!   [`has_errors`], [`passed`]).
//! * Cycle <-> time conversions ([`sim_time_ns`], [`sim_time_us`],
//!   [`sim_time_ms`], [`cycles_for_time_ns`]).
//! * Validation ([`validate_sim_config`]).
//! * [`identity_witness`] returning `phi^2 + 1/phi^2 == 3` (universal anchor).
//!
//! ## Honest scope (R5-HONEST)
//!
//! * **No scheduler, no VCD writer, no event queue, no clock-domain
//!   crossing logic, no RTL execution.** Those layers belong to adjacent
//!   specs (`vcd_trace.t27`, `clock_domain.t27`, `formal.t27`) and are
//!   out of scope for Wave 17.
//! * **No new spec.** Enum tags, struct field order, default values, and
//!   formula shapes mirror [`specs/fpga/simulator.t27`] byte-for-byte
//!   (L6 CEILING).
//! * **Time-conversion overflow note:** the source spec uses `u32` for
//!   `sim_time_ns(cfg, cycles) = cycles * 1_000_000_000 / clock_freq_hz`.
//!   For the spec's own canonical case (`clock_freq_hz = 100_000_000`,
//!   `cycles = 100`) the intermediate `100 * 1_000_000_000 = 1e11`
//!   exceeds `u32::MAX`. We faithfully implement the formula with a
//!   `u64` intermediate and then narrow back to `u32`; the public
//!   signature stays `u32 -> u32` exactly as in the spec, but the
//!   intermediate arithmetic is the minimum width needed to make the
//!   spec's own assertion `sim_time_ns(_, 100) == 1000` evaluate
//!   correctly. This is a faithful reading, not a spec change.
//! * **`#![no_std]`** with zero external dependencies; test cfg pulls
//!   `std` only for the test harness.
//!
//! Anchor: `phi^2 + 1/phi^2 = 3`.
//!
//! [`specs/fpga/simulator.t27`]: ../../specs/fpga/simulator.t27

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// SimState -- mirrors `enum(i8) SimState` in specs/fpga/simulator.t27
// ---------------------------------------------------------------------------

/// Numeric tag for [`SimState::Idle`] (matches the spec).
pub const SIM_STATE_IDLE: i8 = 0;
/// Numeric tag for [`SimState::Running`].
pub const SIM_STATE_RUNNING: i8 = 1;
/// Numeric tag for [`SimState::Paused`].
pub const SIM_STATE_PAUSED: i8 = 2;
/// Numeric tag for [`SimState::Done`].
pub const SIM_STATE_DONE: i8 = 3;
/// Numeric tag for [`SimState::Error`].
pub const SIM_STATE_ERROR: i8 = 4;

/// Lifecycle state of a simulator instance.
///
/// Tag values match `specs/fpga/simulator.t27` byte-for-byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimState {
    /// Configured but not started.
    Idle,
    /// Currently executing cycles.
    Running,
    /// Suspended; can resume.
    Paused,
    /// Reached a clean terminal state.
    Done,
    /// Terminated with one or more errors.
    Error,
}

impl SimState {
    /// Numeric tag (same as the spec's `i8` representation).
    pub const fn tag(self) -> i8 {
        match self {
            SimState::Idle => SIM_STATE_IDLE,
            SimState::Running => SIM_STATE_RUNNING,
            SimState::Paused => SIM_STATE_PAUSED,
            SimState::Done => SIM_STATE_DONE,
            SimState::Error => SIM_STATE_ERROR,
        }
    }

    /// Inverse of [`SimState::tag`]. Returns `None` for unknown tags.
    pub const fn from_tag(tag: i8) -> Option<SimState> {
        match tag {
            SIM_STATE_IDLE => Some(SimState::Idle),
            SIM_STATE_RUNNING => Some(SimState::Running),
            SIM_STATE_PAUSED => Some(SimState::Paused),
            SIM_STATE_DONE => Some(SimState::Done),
            SIM_STATE_ERROR => Some(SimState::Error),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// SimConfig
// ---------------------------------------------------------------------------

/// Default clock frequency baked into the spec's `sim_config` constructor.
///
/// The spec hard-codes `100_000_000` Hz (100 MHz).
pub const DEFAULT_CLOCK_FREQ_HZ: u32 = 100_000_000;

/// Simulator configuration -- field order matches the spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimConfig {
    /// Human-readable simulator name.
    pub name: &'static str,
    /// Hard upper bound on simulated cycles.
    pub max_cycles: u32,
    /// Simulated clock frequency in Hz.
    pub clock_freq_hz: u32,
    /// Whether per-cycle tracing is enabled.
    pub trace_enabled: bool,
    /// Whether VCD output is emitted.
    pub vcd_output: bool,
    /// Whether the run aborts on the first error.
    pub break_on_error: bool,
    /// File-system path for VCD output (empty string when disabled).
    pub vcd_path: &'static str,
}

/// Construct a default `SimConfig` (matches the spec's `sim_config`).
///
/// Defaults: 100 MHz clock, tracing off, VCD off, break-on-error on,
/// empty VCD path.
pub const fn sim_config(name: &'static str, max_cycles: u32) -> SimConfig {
    SimConfig {
        name,
        max_cycles,
        clock_freq_hz: DEFAULT_CLOCK_FREQ_HZ,
        trace_enabled: false,
        vcd_output: false,
        break_on_error: true,
        vcd_path: "",
    }
}

/// Construct a tracing `SimConfig` (matches the spec's `sim_config_with_trace`).
pub const fn sim_config_with_trace(
    name: &'static str,
    max_cycles: u32,
    vcd_path: &'static str,
) -> SimConfig {
    SimConfig {
        name,
        max_cycles,
        clock_freq_hz: DEFAULT_CLOCK_FREQ_HZ,
        trace_enabled: true,
        vcd_output: true,
        break_on_error: true,
        vcd_path,
    }
}

// ---------------------------------------------------------------------------
// SimResult
// ---------------------------------------------------------------------------

/// Post-run simulation report -- field order matches the spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimResult {
    /// Number of cycles actually simulated.
    pub cycles: u32,
    /// Terminal state tag (use [`SimState::from_tag`] to decode).
    pub state: i8,
    /// Total errors encountered.
    pub errors: u32,
    /// Number of assertion checks that fired.
    pub assertions_fired: u32,
    /// Number of coverage points hit.
    pub coverage_points: u32,
}

/// Construct a "ok / done" `SimResult` (matches the spec's `sim_ok`).
pub const fn sim_ok(cycles: u32, coverage: u32) -> SimResult {
    SimResult {
        cycles,
        state: SIM_STATE_DONE,
        errors: 0,
        assertions_fired: 0,
        coverage_points: coverage,
    }
}

/// Construct an "error" `SimResult` (matches the spec's `sim_error`).
pub const fn sim_error(cycles: u32, errors: u32) -> SimResult {
    SimResult {
        cycles,
        state: SIM_STATE_ERROR,
        errors,
        assertions_fired: 0,
        coverage_points: 0,
    }
}

// ---------------------------------------------------------------------------
// ProbePoint + TraceEntry
// ---------------------------------------------------------------------------

/// A single named signal probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbePoint {
    /// Probe display name.
    pub name: &'static str,
    /// Signal path (e.g. `top.uart.tx_data`).
    pub signal: &'static str,
    /// Signal width in bits.
    pub width: u32,
    /// Whether the signal is interpreted as signed.
    pub is_signed: bool,
}

/// Construct an unsigned `ProbePoint` (matches the spec's `probe`).
pub const fn probe(name: &'static str, signal: &'static str, width: u32) -> ProbePoint {
    ProbePoint {
        name,
        signal,
        width,
        is_signed: false,
    }
}

/// One row of a VCD-like trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceEntry {
    /// Cycle number this entry was sampled at.
    pub cycle: u32,
    /// Signal name (must match a `ProbePoint::signal` to be meaningful).
    pub signal: &'static str,
    /// Sampled value (u32 width matches the spec).
    pub value: u32,
}

/// Construct a `TraceEntry` (matches the spec's `trace_entry`).
pub const fn trace_entry(cycle: u32, signal: &'static str, value: u32) -> TraceEntry {
    TraceEntry {
        cycle,
        signal,
        value,
    }
}

// ---------------------------------------------------------------------------
// Query predicates
// ---------------------------------------------------------------------------

/// Returns `true` iff `r.state == Idle`.
pub const fn is_idle(r: SimResult) -> bool {
    r.state == SIM_STATE_IDLE
}

/// Returns `true` iff `r.state == Done`.
pub const fn is_done(r: SimResult) -> bool {
    r.state == SIM_STATE_DONE
}

/// Returns `true` iff `r.state == Error`.
pub const fn is_error(r: SimResult) -> bool {
    r.state == SIM_STATE_ERROR
}

/// Returns `true` iff `r.errors > 0`.
pub const fn has_errors(r: SimResult) -> bool {
    r.errors > 0
}

/// Returns `true` iff the run finished cleanly (Done + 0 errors).
pub const fn passed(r: SimResult) -> bool {
    is_done(r) && r.errors == 0
}

// ---------------------------------------------------------------------------
// Time conversions
// ---------------------------------------------------------------------------

/// `cycles -> nanoseconds`.
///
/// Matches the spec's `sim_time_ns`. The intermediate multiplication is
/// performed in `u64` to faithfully reproduce the spec's *intended*
/// arithmetic at the spec's own example values (`100 cycles @ 100 MHz =
/// 1000 ns` -- the literal `100 * 1_000_000_000` would overflow `u32`).
/// The result is narrowed back to `u32`; over-large results saturate.
pub const fn sim_time_ns(cfg: SimConfig, cycles: u32) -> u32 {
    if cfg.clock_freq_hz == 0 {
        return 0;
    }
    let ns = (cycles as u64) * 1_000_000_000u64 / (cfg.clock_freq_hz as u64);
    if ns > u32::MAX as u64 {
        u32::MAX
    } else {
        ns as u32
    }
}

/// `cycles -> microseconds`. Mirrors the spec's `sim_time_us`.
pub const fn sim_time_us(cfg: SimConfig, cycles: u32) -> u32 {
    sim_time_ns(cfg, cycles) / 1000
}

/// `cycles -> milliseconds`. Mirrors the spec's `sim_time_ms`.
pub const fn sim_time_ms(cfg: SimConfig, cycles: u32) -> u32 {
    sim_time_ns(cfg, cycles) / 1_000_000
}

/// `nanoseconds -> cycles`. Mirrors the spec's `cycles_for_time_ns`.
///
/// Uses a `u64` intermediate to match the spec's intended semantics on
/// the canonical case (`1000 ns @ 100 MHz = 100 cycles`); see the
/// crate-level "time-conversion overflow note" for details.
pub const fn cycles_for_time_ns(cfg: SimConfig, ns: u32) -> u32 {
    if cfg.clock_freq_hz == 0 {
        return 0;
    }
    let c = (ns as u64) * (cfg.clock_freq_hz as u64) / 1_000_000_000u64;
    if c > u32::MAX as u64 {
        u32::MAX
    } else {
        c as u32
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Count configuration errors; matches the spec's `validate_sim_config`.
///
/// Returns the number of invalid fields:
/// * empty `name` -> +1
/// * zero `max_cycles` -> +1
/// * zero `clock_freq_hz` -> +1
pub const fn validate_sim_config(cfg: SimConfig) -> u32 {
    let mut errors: u32 = 0;
    if cfg.name.is_empty() {
        errors += 1;
    }
    if cfg.max_cycles == 0 {
        errors += 1;
    }
    if cfg.clock_freq_hz == 0 {
        errors += 1;
    }
    errors
}

// ---------------------------------------------------------------------------
// Identity witness (universal anchor)
// ---------------------------------------------------------------------------

/// Golden ratio (used by [`identity_witness`]).
pub const PHI: f64 = 1.618_033_988_749_894_8;

/// Returns `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
///
/// Required of every t27 ring crate.
pub fn identity_witness() -> bool {
    let phi2 = PHI * PHI;
    let inv_phi2 = 1.0 / phi2;
    let d = phi2 + inv_phi2 - 3.0;
    // no_std f64::abs() is not available without core::intrinsics; inline:
    let d_abs = if d < 0.0 { -d } else { d };
    d_abs < 1.0e-15
}

// ===========================================================================
// Tests -- 13 mirrored from specs/fpga/simulator.t27, 4 invariants, +1
// universal identity witness = 18 total. All asserts use the exact values
// the spec asserts. Test names follow the spec's `test <name>` blocks.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- mirrored spec tests (13) ---------------------------------------

    #[test]
    fn sim_config_creation() {
        let cfg = sim_config("uart_sim", 10_000);
        assert_eq!(cfg.max_cycles, 10_000);
        assert_eq!(cfg.trace_enabled, false);
    }

    #[test]
    fn sim_config_with_trace_creation() {
        let cfg = sim_config_with_trace("uart_sim", 10_000, "uart.vcd");
        assert_eq!(cfg.trace_enabled, true);
        assert_eq!(cfg.vcd_output, true);
        assert_eq!(cfg.vcd_path, "uart.vcd");
    }

    #[test]
    fn sim_ok_result() {
        let r = sim_ok(5000, 10);
        assert!(is_done(r));
        assert!(!is_error(r));
        assert!(passed(r));
        assert!(!has_errors(r));
        assert_eq!(r.cycles, 5000);
        assert_eq!(r.coverage_points, 10);
    }

    #[test]
    fn sim_error_result() {
        let r = sim_error(3000, 2);
        assert!(!is_done(r));
        assert!(is_error(r));
        assert!(!passed(r));
        assert!(has_errors(r));
        assert_eq!(r.errors, 2);
    }

    #[test]
    fn probe_creation() {
        let p = probe("clk_probe", "clk", 1);
        assert_eq!(p.name, "clk_probe");
        assert_eq!(p.signal, "clk");
        assert_eq!(p.width, 1);
    }

    #[test]
    fn trace_entry_creation() {
        let t = trace_entry(42, "counter", 27);
        assert_eq!(t.cycle, 42);
        assert_eq!(t.signal, "counter");
        assert_eq!(t.value, 27);
    }

    #[test]
    fn sim_time_ns_canonical() {
        let cfg = sim_config("sim", 10_000);
        assert_eq!(sim_time_ns(cfg, 100), 1000);
    }

    #[test]
    fn sim_time_us_canonical() {
        let cfg = sim_config("sim", 10_000);
        assert_eq!(sim_time_us(cfg, 100_000), 1000);
    }

    #[test]
    fn sim_time_ms_canonical() {
        let cfg = sim_config("sim", 10_000);
        assert_eq!(sim_time_ms(cfg, 100_000_000), 1000);
    }

    #[test]
    fn cycles_for_time_ns_canonical() {
        let cfg = sim_config("sim", 10_000);
        assert_eq!(cycles_for_time_ns(cfg, 1000), 100);
    }

    #[test]
    fn validate_config_ok() {
        let cfg = sim_config("sim", 10_000);
        assert_eq!(validate_sim_config(cfg), 0);
    }

    #[test]
    fn validate_config_empty_name() {
        let cfg = sim_config("", 10_000);
        assert!(validate_sim_config(cfg) > 0);
    }

    #[test]
    fn validate_config_zero_cycles() {
        let cfg = sim_config("sim", 0);
        assert!(validate_sim_config(cfg) > 0);
    }

    // ----- spec invariants (4) -------------------------------------------

    #[test]
    fn invariant_max_cycles_positive() {
        let cfg = sim_config("inv", 100);
        assert!(cfg.max_cycles > 0);
    }

    #[test]
    fn invariant_sim_time_positive() {
        let cfg = sim_config("inv", 100);
        assert!(sim_time_ns(cfg, 1) > 0);
    }

    #[test]
    fn invariant_cycles_for_time_positive() {
        let cfg = sim_config("inv", 100);
        assert!(cycles_for_time_ns(cfg, 10) > 0);
    }

    #[test]
    fn invariant_validate_non_negative() {
        // u32 is non-negative by construction; this test mirrors the spec
        // invariant `validate_sim_config(cfg) >= 0`. We assert the
        // type-witness AND that the canonical config has zero errors.
        let cfg = sim_config("inv", 100);
        let e = validate_sim_config(cfg);
        assert_eq!(e, 0);
    }

    // ----- universal anchor (1) ------------------------------------------

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    // ----- bonus sanity: SimState tag round-trip -------------------------

    #[test]
    fn sim_state_tag_roundtrip() {
        for &s in [
            SimState::Idle,
            SimState::Running,
            SimState::Paused,
            SimState::Done,
            SimState::Error,
        ]
        .iter()
        {
            assert_eq!(SimState::from_tag(s.tag()), Some(s));
        }
        assert_eq!(SimState::from_tag(-1), None);
        assert_eq!(SimState::from_tag(5), None);
    }
}
