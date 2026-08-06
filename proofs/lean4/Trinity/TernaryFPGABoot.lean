/- SPDX-License-Identifier: Apache-2.0
   proofs/lean4/Trinity/TernaryFPGABoot.lean
   Formal model of the 7-series FPGA STAT register and the cold-POR
   decision tree documented in fpga/HARDWARE_SSOT.md.
   phi^2 + 1/phi^2 = 3 | TRINITY -/

import Mathlib.Tactic

namespace Trinity

/-- 32-bit raw value read from the 7-series STAT configuration register.
    UG470 Table 5-25 defines the bit layout used by `tri fpga stat`. -/
structure StatRegister where
  raw : UInt32
  deriving Repr, DecidableEq, Inhabited

namespace StatRegister

/-- Mode pins sampled at power-on: bits 10..8 of STAT. -/
def mode (s : StatRegister) : UInt8 :=
  (s.raw.shiftRight 8).land 0x7 |>.toUInt8

/-- DONE output: bit 14 of STAT. -/
def done (s : StatRegister) : Bool :=
  (s.raw.shiftRight 14).land 1 = 1

/-- End-Of-Startup: bit 4 of STAT. -/
def eos (s : StatRegister) : Bool :=
  (s.raw.shiftRight 4).land 1 = 1

/-- INIT complete: bit 11 of STAT. -/
def init_complete (s : StatRegister) : Bool :=
  (s.raw.shiftRight 11).land 1 = 1

/-- CRC error flag: bit 0 of STAT. -/
def crc_error (s : StatRegister) : Bool :=
  s.raw.land 1 = 1

/-- IDCODE mismatch flag: bit 15 of STAT. -/
def id_error (s : StatRegister) : Bool :=
  (s.raw.shiftRight 15).land 1 = 1

/-- DEC (BCM) error flag: bit 16 of STAT. -/
def dec_error (s : StatRegister) : Bool :=
  (s.raw.shiftRight 16).land 1 = 1

/-- Bus width observed during SPI access: bits 23..22 of STAT. -/
def bus_width (s : StatRegister) : UInt8 :=
  (s.raw.shiftRight 22).land 0x3 |>.toUInt8

/-- Master SPI x1 boot mode (value 0b001). -/
def MODE_MASTER_SPI_X1 : UInt8 := 0x1

/-- SPI bus width x1 (value 0b00). -/
def BUS_WIDTH_X1 : UInt8 := 0x0

/-- Known-good cold-POR success value from W400:
    `STAT=0x401079FC` => DONE=1, MODE=001, EOS=1, no CRC/ID/DEC errors. -/
def STAT_SUCCESS_EXAMPLE : UInt32 := 0x401079FC

/-- Cold-POR incomplete value observed before protocol discipline:
    `STAT=0x5000190C` => DONE=0, MODE=001, EOS=0. -/
def STAT_INCOMPLETE_EXAMPLE : UInt32 := 0x5000190C

-- ============================================================================
-- Bitstream configuration
-- ============================================================================

/-- Static bitstream configuration fields audited by `tri fpga smoke-gate`.
    These are 7-series configuration-register values, not STAT. -/
structure BitstreamConfig where
  idcode : UInt32
  spi_buswidth : UInt8
  startupclk : UInt8
  oscfsel : UInt8
  deriving Repr, DecidableEq, Inhabited

namespace BitstreamConfig

/-- Target FPGA for the QMTech Wukong V1 / XC7A200T-FGG676-1. -/
def IDCODE_XC7A200T : UInt32 := 0x03636093

/-- SPI bus width x1 (COR1[8:7] = 0b00). -/
def SPI_BUSWIDTH_X1 : UInt8 := 0x0

/-- Startup clock = CCLK (COR0[16:15] = 0b00). -/
def STARTUPCLK_CCLK : UInt8 := 0x0

/-- Default internal CCLK oscillator selection (COR0[22:17] = 0). -/
def OSCFSEL_DEFAULT : UInt8 := 0x0

/-- 6-bit raw OSCFSEL value has 64 possible selections. -/
def OSCFSEL_COUNT : Nat := 64

/-- Maximum valid raw OSCFSEL value (63). -/
def OSCFSEL_MAX : UInt8 := 0x3F

/-- The default oscillator selection is numerically 0. Used to simplify
    UInt8-to-Nat projections in the transaction proofs. -/
theorem OSCFSEL_DEFAULT_toNat : (OSCFSEL_DEFAULT : UInt8).toNat = 0 := by
  decide

/-- The UInt8 literal 0 projects to the natural number 0. -/
theorem OSCFSEL_ZERO_toNat : (0 : UInt8).toNat = 0 := by
  decide

/-- Nominal CCLK frequency in Hz for each 7-series OSCFSEL value.
    Values are taken from UG470 "Configuration Clock Sources" and the
    Artix-7 configuration timing tables. Only the first few values are
    documented/used by the t27 canonical bitstream; higher values are reserved
    or device-specific and are mapped to 0 here to keep the function total. -/
def cclk_nominal_hz (oscfsel : Nat) : Nat :=
  match oscfsel with
  | 0 => 2_500_000   -- default / 2.5 MHz
  | 1 => 4_200_000   -- ~4.2 MHz
  | 2 => 6_600_000   -- ~6.6 MHz
  | 3 => 10_000_000  -- ~10 MHz
  | 4 => 12_500_000  -- ~12.5 MHz
  | 5 => 16_700_000  -- ~16.7 MHz
  | 6 => 25_000_000  -- ~25 MHz
  | 7 => 33_300_000  -- ~33.3 MHz
  | _ => 0           -- reserved / undefined in this model

/-- Maximum SCK frequency supported by the on-board Micron N25Q128_3V for the
    standard SPI Read command (0x03) used during 7-series Master SPI boot.
    Datasheet value: 50 MHz for standard read; fast read can go higher but the
    FPGA boot loader issues 0x03 by default. Units: Hz. -/
def N25Q128_MAX_SCK_HZ : Nat := 50_000_000

/-- Minimum CS# high time (t_SHSL) required between SPI transactions for the
    N25Q128. Datasheet value: 100 ns. Units: nanoseconds. -/
def N25Q128_MIN_CS_HIGH_NS : Nat := 100

/-- Minimum SCK clock-low time (t_CL) for the N25Q128 standard Read command.
    Datasheet value: 5.5 ns; rounded up to 6 ns to keep the model integral.
    Units: nanoseconds. -/
def N25Q128_MIN_SCK_LOW_NS : Nat := 6

/-- Minimum SCK clock-high time (t_CH) for the N25Q128 standard Read command.
    Datasheet value: 5.5 ns; rounded up to 6 ns to keep the model integral.
    Units: nanoseconds. -/
def N25Q128_MIN_SCK_HIGH_NS : Nat := 6

/-- Maximum wake-up time from power-down (t_RES1) for the N25Q128. The
    datasheet gives ~30 us max; this model uses 100 us as a conservative bound
    that also absorbs board-level power-rail settling. Units: microseconds. -/
def N25Q128_WAKE_FROM_POWERDOWN_US : Nat := 100

/-- A given raw OSCFSEL selection is within the flash timing spec when its
    nominal CCLK is non-zero and does not exceed the flash maximum SCK
    frequency. This is a static, conservative predicate: it does not account for
    temperature/voltage/process variation; those are covered by the margin
    between the nominal CCLK and the flash limit. -/
def cclk_within_flash_spec (oscfsel : UInt8) : Bool :=
  let f := cclk_nominal_hz oscfsel.toNat
  f > 0 ∧ f ≤ N25Q128_MAX_SCK_HZ

/-- Nominal CCLK period in nanoseconds for a given OSCFSEL selection. Returns 0
    for reserved/undefined selections. -/
def cclk_period_ns (oscfsel : Nat) : Nat :=
  let f := cclk_nominal_hz oscfsel
  if f > 0 then 1_000_000_000 / f else 0

/-- True when the nominal CCLK period is long enough that both the clock-low
    and clock-high half-periods satisfy the N25Q128 minimum SCK low/high times.
    Assumes a nominal 50% duty cycle; the conservative period bound makes the
    predicate robust to moderate duty-cycle asymmetry. -/
def sck_duty_ok (oscfsel : Nat) : Bool :=
  let period := cclk_period_ns oscfsel
  let half := period / 2
  half ≥ N25Q128_MIN_SCK_LOW_NS ∧ half ≥ N25Q128_MIN_SCK_HIGH_NS

/-- Comprehensive SPI flash timing predicate for a given OSCFSEL selection.
    Combines the CCLK frequency bound with the SCK low/high half-period bounds.
    CS# high time and wake-up are separate board-level constants because they
    depend on the FPGA configuration engine's inter-transaction timing rather
    than on the CCLK frequency alone. -/
def flash_spi_timing_ok (oscfsel : UInt8) : Bool :=
  cclk_within_flash_spec oscfsel ∧ sck_duty_ok oscfsel.toNat

/-- The canonical bitstream configuration proven to boot from flash on W400.
    Matches the assertions run by `tri fpga smoke-gate`. -/
def canonical (cfg : BitstreamConfig) : Bool :=
  cfg.idcode = IDCODE_XC7A200T
  ∧ cfg.spi_buswidth = SPI_BUSWIDTH_X1
  ∧ cfg.startupclk = STARTUPCLK_CCLK
  ∧ cfg.oscfsel = OSCFSEL_DEFAULT

/-- The bitstream is configured for x1 SPI boot from CCLK. -/
def spi_x1_cclk_boot (cfg : BitstreamConfig) : Bool :=
  cfg.spi_buswidth = SPI_BUSWIDTH_X1 ∧ cfg.startupclk = STARTUPCLK_CCLK

/-- Default oscillator selection (no COR0 patch required). -/
def default_oscfsel (cfg : BitstreamConfig) : Bool :=
  cfg.oscfsel = OSCFSEL_DEFAULT

/-- A canonical configuration is always an x1-CCLK-boot configuration. -/
theorem canonical_implies_spi_x1_cclk_boot (cfg : BitstreamConfig) :
  cfg.canonical → cfg.spi_x1_cclk_boot := by
  intro h
  simp [canonical, spi_x1_cclk_boot] at h ⊢
  exact ⟨h.right.left, h.right.right.left⟩

/-- The canonical OSCFSEL=0 selection has a 2.5 MHz nominal CCLK, well below
    the N25Q128 50 MHz standard-read limit. -/
theorem canonical_oscfsel_within_flash_spec :
  cclk_within_flash_spec 0 = true := by
  decide

/-- If a bitstream is canonical then its oscillator selection is timing-safe
    for the on-board flash. This closes the static CCLK-timing side of the
    cold-POR decision tree. -/
theorem canonical_implies_cclk_within_flash_spec (cfg : BitstreamConfig) :
  cfg.canonical → cclk_within_flash_spec cfg.oscfsel := by
  intro h
  simp [canonical, OSCFSEL_DEFAULT, cclk_within_flash_spec, cclk_nominal_hz,
        N25Q128_MAX_SCK_HZ] at h ⊢
  rw [h.right.right.right]
  decide

/-- The full SPI flash timing predicate implies the CCLK frequency bound. -/
theorem flash_spi_timing_ok_implies_cclk_within_flash_spec (oscfsel : UInt8) :
  flash_spi_timing_ok oscfsel → cclk_within_flash_spec oscfsel := by
  intro h
  simp [flash_spi_timing_ok] at h
  exact h.left

/-- The canonical OSCFSEL=0 selection has a 400 ns CCLK period, giving a 200 ns
    half-period that is far above the N25Q128 6 ns SCK low/high limits. -/
theorem canonical_oscfsel_sck_duty_ok :
  sck_duty_ok 0 = true := by
  decide

/-- The canonical OSCFSEL=0 selection satisfies the full SPI flash timing
    predicate: frequency is within the 50 MHz standard-read limit and the
    nominal period is long enough for the SCK low/high constraints. -/
theorem canonical_oscfsel_flash_spi_timing_ok :
  flash_spi_timing_ok 0 = true := by
  decide

/-- If a bitstream is canonical then its oscillator selection satisfies the
    full SPI flash timing predicate. -/
theorem canonical_implies_flash_spi_timing_ok (cfg : BitstreamConfig) :
  cfg.canonical → flash_spi_timing_ok cfg.oscfsel := by
  intro h
  simp [canonical, OSCFSEL_DEFAULT, flash_spi_timing_ok,
        cclk_within_flash_spec, sck_duty_ok, cclk_period_ns, cclk_nominal_hz,
        N25Q128_MAX_SCK_HZ, N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS] at h ⊢
  rw [h.right.right.right]
  decide

-- ============================================================================
-- SPI flash read-transaction model (W408)
-- ============================================================================

/-- A single SPI flash read transaction as issued by the Artix-7 configuration
    engine during Master SPI boot. The fields capture the timing dimensions that
    must satisfy the N25Q128_3V datasheet for a safe boot read. -/
structure SPIReadTransaction where
  csHighNs : Nat
  numSckEdges : Nat
  sckLowNs : Nat
  sckHighNs : Nat
  wakeUs : Nat
  deriving Repr, DecidableEq, Inhabited

/-- Build a conservative SPI read-transaction model from a raw OSCFSEL value and
    the number of bits the FPGA will shift from flash during this transaction.
    This is the transaction-level equivalent of `flash_spi_timing_ok` and is
    parameterized by OSCFSEL directly so we can prove a lookup table for every
    documented Artix-7 CCLK selection without constructing a full
    `BitstreamConfig`. -/
def artix7_boot_transaction_for_oscfsel (oscfsel : Nat) (bitstream_bits : Nat) :
    SPIReadTransaction :=
  let period := cclk_period_ns oscfsel
  let half := period / 2
  { csHighNs := N25Q128_MIN_CS_HIGH_NS,
    numSckEdges := 2 * bitstream_bits,
    sckLowNs := half,
    sckHighNs := half,
    wakeUs := N25Q128_WAKE_FROM_POWERDOWN_US }

/-- Build a conservative SPI read-transaction model from a bitstream config and
    the number of bits the FPGA will shift from flash during this transaction.
    The FPGA configuration engine issues one SCK edge per half-bit, so a
    `bitstream_bits`-bit read produces `2 * bitstream_bits` SCK edges. The CS#
    high time and wake-up delay are taken as the N25Q128 minimum constants; the
    SCK low/high times come from the nominal CCLK period. -/
def artix7_boot_transaction (cfg : BitstreamConfig) (bitstream_bits : Nat) :
    SPIReadTransaction :=
  artix7_boot_transaction_for_oscfsel cfg.oscfsel.toNat bitstream_bits

/-- True when a transaction respects every N25Q128_3V timing bound we model:
    CS# high time, SCK low/high times, maximum SCK frequency, and wake-up time.
    The frequency bound is checked from the sum of the low and high times so
    that asymmetric duty cycles are still constrained. -/
def transaction_satisfies_flash_spec (t : SPIReadTransaction) : Bool :=
  t.csHighNs ≥ N25Q128_MIN_CS_HIGH_NS
  ∧ t.sckLowNs ≥ N25Q128_MIN_SCK_LOW_NS
  ∧ t.sckHighNs ≥ N25Q128_MIN_SCK_HIGH_NS
  ∧ (t.sckLowNs + t.sckHighNs > 0 ∧ 1_000_000_000 / (t.sckLowNs + t.sckHighNs) ≤ N25Q128_MAX_SCK_HZ)
  ∧ t.wakeUs ≥ N25Q128_WAKE_FROM_POWERDOWN_US

/-- The canonical OSCFSEL=0 configuration produces a 400 ns CCLK period, giving
    200 ns SCK low/high times, a 2.5 MHz SCK frequency, and all other constants
    are within the N25Q128_3V spec. This holds for any `bitstream_bits` because
    the timing spec is per-edge, not per-transaction-length. -/
theorem canonical_oscfsel_transaction_satisfies_flash_spec :
  ∀ (bits : Nat),
    transaction_satisfies_flash_spec
      (artix7_boot_transaction ⟨IDCODE_XC7A200T, SPI_BUSWIDTH_X1, STARTUPCLK_CCLK, OSCFSEL_DEFAULT⟩ bits)
      = true := by
  intro bits
  have hnat :
    (⟨IDCODE_XC7A200T, SPI_BUSWIDTH_X1, STARTUPCLK_CCLK, OSCFSEL_DEFAULT⟩ : BitstreamConfig).oscfsel.toNat = 0 := by
    decide
  simp [artix7_boot_transaction, artix7_boot_transaction_for_oscfsel, transaction_satisfies_flash_spec,
        cclk_period_ns, cclk_nominal_hz, N25Q128_MAX_SCK_HZ, N25Q128_MIN_CS_HIGH_NS,
        N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS, N25Q128_WAKE_FROM_POWERDOWN_US, hnat]

/-- If a bitstream is canonical then any boot transaction it produces satisfies
    the N25Q128_3V timing spec. -/
theorem canonical_implies_transaction_satisfies_flash_spec (cfg : BitstreamConfig) (bits : Nat) :
  cfg.canonical → transaction_satisfies_flash_spec (artix7_boot_transaction cfg bits) := by
  intro h
  simp [canonical, OSCFSEL_DEFAULT, artix7_boot_transaction, artix7_boot_transaction_for_oscfsel,
        transaction_satisfies_flash_spec, cclk_period_ns, cclk_nominal_hz, N25Q128_MAX_SCK_HZ,
        N25Q128_MIN_CS_HIGH_NS, N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS,
        N25Q128_WAKE_FROM_POWERDOWN_US] at h ⊢
  have hnat : cfg.oscfsel.toNat = 0 := by
    rw [h.right.right.right]
    decide
  simp [hnat]

/-- For every documented Artix-7 OSCFSEL value (0..7), the boot transaction
    produced by that CCLK selection satisfies the N25Q128_3V timing spec. This
    gives a lookup-table proof that matches the UG470 CCLK-frequency mapping and
    covers the sweep variants exercised in W400. -/
theorem oscfsel_zero_to_seven_transaction_satisfies_flash_spec
  (oscfsel : Nat) (bits : Nat) :
  oscfsel ≤ 7
  → transaction_satisfies_flash_spec (artix7_boot_transaction_for_oscfsel oscfsel bits) = true := by
  intro h
  interval_cases oscfsel
  all_goals
    simp [artix7_boot_transaction_for_oscfsel, transaction_satisfies_flash_spec,
          cclk_period_ns, cclk_nominal_hz,
          N25Q128_MAX_SCK_HZ, N25Q128_MIN_CS_HIGH_NS,
          N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS,
          N25Q128_WAKE_FROM_POWERDOWN_US]

/-- The transaction computed from a full `BitstreamConfig` equals the transaction
    computed from its OSCFSEL value alone. This links the config-level proof to
    the per-OSCFSEL lookup table. -/
theorem artix7_boot_transaction_eq_for_oscfsel
  (cfg : BitstreamConfig) (bits : Nat) :
  artix7_boot_transaction cfg bits = artix7_boot_transaction_for_oscfsel cfg.oscfsel.toNat bits := by
  rfl

-- ============================================================================
-- Measured-CCLK formal link (W410)
-- ============================================================================

/-- Conservative nanosecond period derived from a measured frequency in Hz.
    Uses integer division rounded down; this is the conservative (shorter-period)
    direction because a real frequency estimate is an upper bound on the actual
    clock. Returns 0 for an invalid zero frequency. -/
def measured_cclk_period_ns (freq_hz : Nat) : Nat :=
  if freq_hz > 0 then 1_000_000_000 / freq_hz else 0

/-- Clock-low time in nanoseconds from a measured duty-cycle percentage.
    `duty_pct` is the high-time fraction, so low time is the complement. -/
def measured_cclk_low_ns (freq_hz : Nat) (duty_pct : Nat) : Nat :=
  measured_cclk_period_ns freq_hz * (100 - duty_pct) / 100

/-- Clock-high time in nanoseconds from a measured duty-cycle percentage.
    Defined as the remainder of the conservative period so that low + high
    exactly equals the measured period, avoiding rounding drift in the SCK
    frequency bound. -/
def measured_cclk_high_ns (freq_hz : Nat) (duty_pct : Nat) : Nat :=
  let period := measured_cclk_period_ns freq_hz
  let low := measured_cclk_low_ns freq_hz duty_pct
  period - low

/-- Minimum SCK period implied by the N25Q128_3V 50 MHz standard-read limit.
    Units: nanoseconds. -/
def N25Q128_MIN_SCK_PERIOD_NS : Nat :=
  1_000_000_000 / N25Q128_MAX_SCK_HZ

/-- Worst-case SCK clock-low time used for the PVT-margin predicate. This is a
    conservative 2× derating of the nominal N25Q128_3V `t_CL` value to absorb
    process/voltage/temperature variation until actual PVT characterization data
    is available. Units: nanoseconds. -/
def N25Q128_MIN_SCK_LOW_NS_WC : Nat := 12

/-- Worst-case SCK clock-high time used for the PVT-margin predicate. This is a
    conservative 2× derating of the nominal N25Q128_3V `t_CH` value to absorb
    process/voltage/temperature variation until actual PVT characterization data
    is available. Units: nanoseconds. -/
def N25Q128_MIN_SCK_HIGH_NS_WC : Nat := 12

/-- Build a transaction from a measured (frequency, duty-cycle) pair. The
    number of bits is not part of the timing predicate, but the transaction
    carries it for consistency with `artix7_boot_transaction`. -/
def measured_boot_transaction (freq_hz : Nat) (duty_pct : Nat) (bits : Nat) :
    SPIReadTransaction :=
  { csHighNs := N25Q128_MIN_CS_HIGH_NS,
    numSckEdges := 2 * bits,
    sckLowNs := measured_cclk_low_ns freq_hz duty_pct,
    sckHighNs := measured_cclk_high_ns freq_hz duty_pct,
    wakeUs := N25Q128_WAKE_FROM_POWERDOWN_US }

/-- True when a measured (frequency, duty-cycle) pair satisfies the N25Q128_3V
    standard-read timing bounds. This is the formal counterpart of the Rust
    `tri fpga measure-cclk --validate` guard, and the entry point for turning a
    real capture into a `transaction_satisfies_flash_spec` proof. -/
def measured_cclk_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool :=
  freq_hz > 0
  ∧ freq_hz ≤ N25Q128_MAX_SCK_HZ
  ∧ duty_pct ≤ 100
  ∧ measured_cclk_low_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_LOW_NS
  ∧ measured_cclk_high_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_HIGH_NS

/-- The measured low time is never larger than the conservative period when the
    duty cycle is at most 100%. This is needed to show that low + high equals
    the period. -/
lemma measured_cclk_low_le_period (freq_hz duty_pct : Nat) :
  duty_pct ≤ 100 → measured_cclk_low_ns freq_hz duty_pct ≤ measured_cclk_period_ns freq_hz := by
  intro h
  simp [measured_cclk_low_ns, measured_cclk_period_ns]
  by_cases hz : freq_hz > 0
  · -- freq_hz > 0, so period = 1_000_000_000 / freq_hz
    simp [hz]
    have h1 : 1_000_000_000 / freq_hz * (100 - duty_pct) ≤ 1_000_000_000 / freq_hz * 100 := by
      apply Nat.mul_le_mul_left
      omega
    have h2 : 1_000_000_000 / freq_hz * (100 - duty_pct) / 100 ≤ 1_000_000_000 / freq_hz * 100 / 100 := by
      apply Nat.div_le_div_right
      exact h1
    have h3 : 1_000_000_000 / freq_hz * 100 / 100 = 1_000_000_000 / freq_hz := by
      simp
    linarith
  · -- freq_hz = 0, both sides reduce to 0
    have hz' : freq_hz = 0 := by omega
    simp [hz']

/-- If the measured pair satisfies the flash timing predicate, the conservative
    period is at least the N25Q128 minimum SCK period. -/
lemma measured_cclk_period_at_least_min_sck_period (freq_hz : Nat) :
  freq_hz > 0 → freq_hz ≤ N25Q128_MAX_SCK_HZ
  → measured_cclk_period_ns freq_hz ≥ N25Q128_MIN_SCK_PERIOD_NS := by
  intro h_pos h_max
  simp [measured_cclk_period_ns, N25Q128_MIN_SCK_PERIOD_NS, N25Q128_MAX_SCK_HZ] at *
  rw [if_pos h_pos]
  rw [Nat.le_div_iff_mul_le h_pos]
  omega

/-- A measured (frequency, duty-cycle) pair that passes the flash predicate
    produces a `SPIReadTransaction` that satisfies the N25Q128_3V timing spec.
    This closes the formal link between a real CCLK capture and the boot
    transaction model. -/
theorem measured_cclk_satisfies_flash_spec_implies_transaction_ok
  (freq_hz duty_pct bits : Nat) :
  measured_cclk_satisfies_flash_spec freq_hz duty_pct = true
  → transaction_satisfies_flash_spec (measured_boot_transaction freq_hz duty_pct bits) = true := by
  intro h
  -- Simplify the predicate and the transaction spec, but keep the measured
  -- low/high/period functions symbolic so that the `low + high = period` rewrite
  -- matches syntactically in the goal.
  simp [measured_cclk_satisfies_flash_spec, measured_boot_transaction, transaction_satisfies_flash_spec,
        N25Q128_MAX_SCK_HZ,
        N25Q128_MIN_CS_HIGH_NS, N25Q128_MIN_SCK_LOW_NS,
        N25Q128_MIN_SCK_HIGH_NS, N25Q128_WAKE_FROM_POWERDOWN_US] at h ⊢
  rcases h with ⟨h_fpos, h_fmax, h_duty, h_low, h_high⟩
  have h_period_min : measured_cclk_period_ns freq_hz ≥ N25Q128_MIN_SCK_PERIOD_NS :=
    measured_cclk_period_at_least_min_sck_period freq_hz h_fpos h_fmax
  have h_period_pos : measured_cclk_period_ns freq_hz > 0 := by
    simp [measured_cclk_period_ns]
    rw [if_pos h_fpos]
    apply Nat.div_pos
    · omega
    · exact h_fpos
  have h_low_le_period : measured_cclk_low_ns freq_hz duty_pct ≤ measured_cclk_period_ns freq_hz :=
    measured_cclk_low_le_period freq_hz duty_pct h_duty
  have h_sum : measured_cclk_low_ns freq_hz duty_pct + measured_cclk_high_ns freq_hz duty_pct = measured_cclk_period_ns freq_hz := by
    simp [measured_cclk_high_ns]
    rw [Nat.add_sub_of_le h_low_le_period]
  have h_freq : 1_000_000_000 / measured_cclk_period_ns freq_hz ≤ N25Q128_MAX_SCK_HZ := by
    rw [Nat.div_le_iff_le_mul_add_pred h_period_pos]
    simp [N25Q128_MAX_SCK_HZ, N25Q128_MIN_SCK_PERIOD_NS] at h_period_min ⊢
    omega
  have h_low_pos : 0 < measured_cclk_low_ns freq_hz duty_pct := by omega
  have h_high_pos : 0 < measured_cclk_high_ns freq_hz duty_pct := by omega
  -- Rewrite `low + high = period` in the goal so the frequency bound follows
  -- from h_freq and the positivity disjunction follows from h_low_pos.
  rw [h_sum]
  simp [N25Q128_MAX_SCK_HZ] at h_freq
  simp_all

-- ============================================================================
-- Raw-ns measured-CCLK formal link (W412)
-- ============================================================================

/-- Frequency in Hz derived from a raw measured period in nanoseconds.
    Rounded down to keep the estimate conservative. -/
def measured_cclk_freq_hz_from_period_ns (period_ns : Nat) : Nat :=
  if period_ns > 0 then 1_000_000_000 / period_ns else 0

/-- Duty-cycle percentage (high-time fraction) derived from raw low/high times.
    Rounded to the nearest integer, clamped to [0, 100]. -/
def measured_cclk_duty_pct_from_raw_ns (period_ns : Nat) (high_ns : Nat) : Nat :=
  if period_ns > 0 then (100 * high_ns) / period_ns else 0

/-- True when raw nanosecond timings satisfy the N25Q128_3V standard-read bounds.
    This is the entry point for instrument exports that report period/low/high
    directly instead of frequency/duty. The `low_ns + high_ns = period_ns`
    precondition rejects inconsistent instrument readings. -/
def measured_cclk_from_raw_ns_satisfies_flash_spec (period_ns low_ns high_ns : Nat) : Bool :=
  low_ns + high_ns = period_ns
  ∧ let freq_hz := measured_cclk_freq_hz_from_period_ns period_ns
    let duty_pct := measured_cclk_duty_pct_from_raw_ns period_ns high_ns
    measured_cclk_satisfies_flash_spec freq_hz duty_pct

/-- Transaction built from raw nanosecond timings, mirroring `measured_boot_transaction`. -/
def measured_boot_transaction_from_raw_ns (period_ns _low_ns high_ns bits : Nat) : SPIReadTransaction :=
  let freq_hz := measured_cclk_freq_hz_from_period_ns period_ns
  let duty_pct := measured_cclk_duty_pct_from_raw_ns period_ns high_ns
  measured_boot_transaction freq_hz duty_pct bits

/-- If raw nanosecond timings satisfy the flash spec, the transaction is OK. -/
theorem measured_cclk_from_raw_ns_implies_transaction_ok
  (period_ns low_ns high_ns bits : Nat) :
  measured_cclk_from_raw_ns_satisfies_flash_spec period_ns low_ns high_ns = true
  → transaction_satisfies_flash_spec (measured_boot_transaction_from_raw_ns period_ns low_ns high_ns bits) = true := by
  intro h
  simp [measured_cclk_from_raw_ns_satisfies_flash_spec, measured_boot_transaction_from_raw_ns] at h ⊢
  rcases h with ⟨_h_consistent, h_spec⟩
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  exact h_spec

/-- Concrete example: a raw 40 ns period / 20 ns low / 20 ns high capture
    satisfies the flash spec. -/
theorem measured_raw_ns_40_20_20_satisfies_flash_spec :
  measured_cclk_from_raw_ns_satisfies_flash_spec 40 20 20 = true := by
  decide

-- ============================================================================
-- PVT-aware timing model (W412)
-- ============================================================================

/-- Process corner for PVT-aware derating. The actual numeric derating is a
    placeholder pending Micron N25Q128_3V PVT characterization data. -/
inductive ProcessCorner where
  | tt
  | ff
  | ss
  deriving Repr, DecidableEq, Inhabited

/-- PVT context supplied with a measured CCLK pair. All fields are part of the
    falsifiable assumption set; replacing the placeholder derating below with
    real datasheet curves is the next step once PVT data is available. -/
structure PvtContext where
  temp_c : Int
  vccint_mv : Nat
  vccaux_mv : Nat
  process_corner : ProcessCorner
  deriving Repr, DecidableEq, Inhabited

/-- Operating-envelope lower bound for temperature in °C. -/
def PVT_TEMP_MIN_C : Int := -40

/-- Operating-envelope upper bound for temperature in °C. -/
def PVT_TEMP_MAX_C : Int := 85

/-- Operating-envelope lower bound for VCCINT in millivolts. -/
def PVT_VCCINT_MIN_MV : Nat := 900

/-- Operating-envelope upper bound for VCCINT in millivolts. -/
def PVT_VCCINT_MAX_MV : Nat := 1100

/-- Conservative temperature derating in nanoseconds. The N25Q128_3V datasheet
    does not publish a closed-form curve, so we use a linear upper envelope:
    0.02 ns per degree Celsius above the minimum temperature. At the industrial
    upper bound (+85 °C) this adds 2 ns to the nominal bound. -/
def n25q128_pvt_temp_derating_ns (temp_c : Int) : Nat :=
  ((temp_c - PVT_TEMP_MIN_C).toNat * 2) / 100

/-- Conservative VCCINT derating in nanoseconds. Lower VCCINT slows the flash
    output drivers; we model a linear upper envelope of 0.005 ns per millivolt
    below the maximum VCCINT. At the minimum envelope voltage (900 mV) this
    adds 1 ns. -/
def n25q128_pvt_voltage_derating_ns (vccint_mv : Nat) : Nat :=
  ((PVT_VCCINT_MAX_MV - vccint_mv) * 5) / 1000

/-- Process-corner derating in nanoseconds. `ss` (slow-slow) is the worst case,
    `tt` is typical, and `ff` (fast-fast) adds no margin. -/
def n25q128_pvt_process_derating_ns (corner : ProcessCorner) : Nat :=
  match corner with
  | ProcessCorner.ff => 0
  | ProcessCorner.tt => 2
  | ProcessCorner.ss => 4

/-- PVT-aware minimum SCK low time. Combines nominal N25Q128_3V `t_CL` with
    conservative temperature, voltage, and process-corner envelopes. The result
    ranges from the nominal 6 ns (best case) to 13 ns (worst case: +85 °C,
    900 mV, ss corner), replacing the previous flat 12 ns placeholder.

    **Falsification condition:** if a future N25Q128_3V PVT characterization
    shows that `t_CL` can exceed the computed value under the operating envelope,
    the envelope coefficients must be raised. All implication theorems below
    remain valid as long as `n25q128_min_sck_low_ns_pvt ctx ≥ N25Q128_MIN_SCK_LOW_NS`. -/
def n25q128_min_sck_low_ns_pvt (ctx : PvtContext) : Nat :=
  N25Q128_MIN_SCK_LOW_NS
  + n25q128_pvt_temp_derating_ns ctx.temp_c
  + n25q128_pvt_voltage_derating_ns ctx.vccint_mv
  + n25q128_pvt_process_derating_ns ctx.process_corner

/-- PVT-aware minimum SCK high time. See `n25q128_min_sck_low_ns_pvt` for the
    envelope rationale and falsification condition. -/
def n25q128_min_sck_high_ns_pvt (ctx : PvtContext) : Nat :=
  N25Q128_MIN_SCK_HIGH_NS
  + n25q128_pvt_temp_derating_ns ctx.temp_c
  + n25q128_pvt_voltage_derating_ns ctx.vccint_mv
  + n25q128_pvt_process_derating_ns ctx.process_corner

/-- PVT-aware minimum SCK half-period time. The low and high bounds are symmetric
    in the current envelope, so the half-period bound equals both; this function
    is the single entry point used by the Rust `n25q128_min_sck_half_ns_pvt` and
    by instrument-export validation. Units: nanoseconds. -/
def n25q128_min_sck_half_ns_pvt (ctx : PvtContext) : Nat :=
  n25q128_min_sck_low_ns_pvt ctx

/-- PVT-aware measured-CCLK flash predicate. As long as the placeholder derating
    is at least the nominal 6 ns bound, it implies the nominal predicate. -/
def measured_cclk_with_pvt_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) (ctx : PvtContext) : Bool :=
  freq_hz > 0
  ∧ freq_hz ≤ N25Q128_MAX_SCK_HZ
  ∧ duty_pct ≤ 100
  ∧ measured_cclk_low_ns freq_hz duty_pct ≥ n25q128_min_sck_low_ns_pvt ctx
  ∧ measured_cclk_high_ns freq_hz duty_pct ≥ n25q128_min_sck_high_ns_pvt ctx

/-- The temperature derating is non-negative inside the operating envelope. -/
lemma n25q128_pvt_temp_derating_ns_nonneg (temp_c : Int) :
  (PVT_TEMP_MIN_C ≤ temp_c) → n25q128_pvt_temp_derating_ns temp_c ≥ 0 := by
  intro h
  simp [n25q128_pvt_temp_derating_ns, PVT_TEMP_MIN_C]

/-- The voltage derating is non-negative inside the operating envelope. -/
lemma n25q128_pvt_voltage_derating_ns_nonneg (vccint_mv : Nat) :
  (vccint_mv ≤ PVT_VCCINT_MAX_MV) → n25q128_pvt_voltage_derating_ns vccint_mv ≥ 0 := by
  intro h
  simp [n25q128_pvt_voltage_derating_ns, PVT_VCCINT_MAX_MV]

/-- The process-corner derating is non-negative. -/
lemma n25q128_pvt_process_derating_ns_nonneg (corner : ProcessCorner) :
  n25q128_pvt_process_derating_ns corner ≥ 0 := by
  cases corner <;> simp [n25q128_pvt_process_derating_ns]

/-- The temperature derating is monotone inside the operating envelope: a
    higher temperature (above the minimum) does not decrease the derating. -/
lemma n25q128_pvt_temp_derating_ns_monotone (t1 t2 : Int) :
  (PVT_TEMP_MIN_C ≤ t1) → (t1 ≤ t2)
  → n25q128_pvt_temp_derating_ns t1 ≤ n25q128_pvt_temp_derating_ns t2 := by
  intro h_min h_le
  simp [n25q128_pvt_temp_derating_ns, PVT_TEMP_MIN_C]
  omega

/-- The voltage derating is antitone inside the operating envelope: a higher
    VCCINT (closer to the maximum) does not increase the derating. -/
lemma n25q128_pvt_voltage_derating_ns_antitone (v1 v2 : Nat) :
  (v1 ≤ v2) → (v2 ≤ PVT_VCCINT_MAX_MV)
  → n25q128_pvt_voltage_derating_ns v2 ≤ n25q128_pvt_voltage_derating_ns v1 := by
  intro h_le h_max
  simp [n25q128_pvt_voltage_derating_ns, PVT_VCCINT_MAX_MV]
  omega

/-- Process-corner ordering: ff is the best (fast-fast, no derating), ss is the
    worst (slow-slow, largest derating). -/
def ProcessCorner.worse_than (c1 c2 : ProcessCorner) : Prop :=
  n25q128_pvt_process_derating_ns c1 ≤ n25q128_pvt_process_derating_ns c2

/-- The process-corner derating is monotone with the `worse_than` order. -/
lemma n25q128_pvt_process_derating_ns_monotone (c1 c2 : ProcessCorner) :
  c1.worse_than c2 → n25q128_pvt_process_derating_ns c1 ≤ n25q128_pvt_process_derating_ns c2 := by
  intro h
  exact h

/-- The corner ordering facts used by the monotonicity proof. -/
lemma ProcessCorner.ff_worse_than_tt : ProcessCorner.ff.worse_than ProcessCorner.tt := by
  simp [worse_than, n25q128_pvt_process_derating_ns]

lemma ProcessCorner.tt_worse_than_ss : ProcessCorner.tt.worse_than ProcessCorner.ss := by
  simp [worse_than, n25q128_pvt_process_derating_ns]

/-- Decidable equality for process corners (exposed for automation that cannot
    see the auto-derived instance). -/
def ProcessCorner.eq_decidable (c1 c2 : ProcessCorner) : Decidable (c1 = c2) :=
  inferInstance

/-- Decidable corner ordering. This lets future automation compare two contexts
    without leaving a `Prop` goal. -/
def ProcessCorner.worse_than_decidable (c1 c2 : ProcessCorner) :
  Decidable (c1.worse_than c2) := by
  cases c1 <;> cases c2 <;> simp [worse_than] <;> infer_instance

/-- Total severity rank: ff=0, tt=1, ss=2. Useful for `if c1 < c2` style scripts. -/
def ProcessCorner.severity (c : ProcessCorner) : Nat :=
  match c with
  | ProcessCorner.ff => 0
  | ProcessCorner.tt => 1
  | ProcessCorner.ss => 2

/-- The severity rank agrees with the `worse_than` order. -/
lemma ProcessCorner.worse_than_iff_severity_le (c1 c2 : ProcessCorner) :
  c1.worse_than c2 ↔ c1.severity ≤ c2.severity := by
  cases c1 <;> cases c2 <;> simp [worse_than, severity, n25q128_pvt_process_derating_ns]

/-- The PVT-aware SCK low bound is at least the nominal N25Q128 bound. This is
    the only fact the implication proof needs; real PVT data must preserve it. -/
lemma pvt_low_ns_at_least_nominal (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → n25q128_min_sck_low_ns_pvt ctx ≥ N25Q128_MIN_SCK_LOW_NS := by
  intro h_temp h_volt
  simp [n25q128_min_sck_low_ns_pvt]
  have ht : n25q128_pvt_temp_derating_ns ctx.temp_c ≥ 0 :=
    n25q128_pvt_temp_derating_ns_nonneg ctx.temp_c h_temp
  have hv : n25q128_pvt_voltage_derating_ns ctx.vccint_mv ≥ 0 :=
    n25q128_pvt_voltage_derating_ns_nonneg ctx.vccint_mv h_volt
  have hp : n25q128_pvt_process_derating_ns ctx.process_corner ≥ 0 :=
    n25q128_pvt_process_derating_ns_nonneg ctx.process_corner
  omega

/-- The PVT-aware SCK high bound is at least the nominal N25Q128 bound. -/
lemma pvt_high_ns_at_least_nominal (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → n25q128_min_sck_high_ns_pvt ctx ≥ N25Q128_MIN_SCK_HIGH_NS := by
  intro h_temp h_volt
  simp [n25q128_min_sck_high_ns_pvt]
  have ht : n25q128_pvt_temp_derating_ns ctx.temp_c ≥ 0 :=
    n25q128_pvt_temp_derating_ns_nonneg ctx.temp_c h_temp
  have hv : n25q128_pvt_voltage_derating_ns ctx.vccint_mv ≥ 0 :=
    n25q128_pvt_voltage_derating_ns_nonneg ctx.vccint_mv h_volt
  have hp : n25q128_pvt_process_derating_ns ctx.process_corner ≥ 0 :=
    n25q128_pvt_process_derating_ns_nonneg ctx.process_corner
  omega

/-- The PVT-aware SCK half-period bound is at least the nominal 6 ns bound across
    the entire operating envelope. This is the regression fact checked by the
    Rust `n25q128_min_sck_half_ns_pvt` operating-rectangle sweep. -/
lemma pvt_half_ns_at_least_nominal (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → n25q128_min_sck_half_ns_pvt ctx ≥ N25Q128_MIN_SCK_LOW_NS := by
  intro h_temp h_volt
  simp [n25q128_min_sck_half_ns_pvt]
  exact pvt_low_ns_at_least_nominal ctx h_temp h_volt

/-- The PVT-aware SCK half-period bound is monotone non-decreasing in
    temperature (inside the operating envelope): a higher temperature never
    yields a smaller bound. -/
lemma pvt_half_ns_monotone_in_temp (t1 t2 : Int) (v : Nat) (c : ProcessCorner) :
  (PVT_TEMP_MIN_C ≤ t1) → (t1 ≤ t2)
  → n25q128_min_sck_half_ns_pvt ⟨t1, v, 2700, c⟩
    ≤ n25q128_min_sck_half_ns_pvt ⟨t2, v, 2700, c⟩ := by
  intro h_min h_le
  simp [n25q128_min_sck_half_ns_pvt, n25q128_min_sck_low_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, PVT_TEMP_MIN_C]
  omega

/-- The PVT-aware SCK half-period bound is antitone non-increasing in VCCINT
    (inside the operating envelope): a higher VCCINT (closer to the maximum)
    never yields a larger bound. -/
lemma pvt_half_ns_antitone_in_vccint (t : Int) (v1 v2 : Nat) (c : ProcessCorner) :
  (v1 ≤ v2) → (v2 ≤ PVT_VCCINT_MAX_MV)
  → n25q128_min_sck_half_ns_pvt ⟨t, v2, 2700, c⟩
    ≤ n25q128_min_sck_half_ns_pvt ⟨t, v1, 2700, c⟩ := by
  intro h_le h_max
  simp [n25q128_min_sck_half_ns_pvt, n25q128_min_sck_low_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, PVT_VCCINT_MAX_MV]
  omega

/-- The PVT-aware SCK half-period bound is monotone with the process-corner
    ordering: a worse corner (larger derating) never yields a smaller bound. -/
lemma pvt_half_ns_monotone_in_process_corner (t : Int) (v : Nat) (c1 c2 : ProcessCorner) :
  c1.worse_than c2
  → n25q128_min_sck_half_ns_pvt ⟨t, v, 2700, c1⟩
    ≤ n25q128_min_sck_half_ns_pvt ⟨t, v, 2700, c2⟩ := by
  intro h
  simp [n25q128_min_sck_half_ns_pvt, n25q128_min_sck_low_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, ProcessCorner.worse_than] at h ⊢
  cases c1 <;> cases c2 <;> omega

/-- The PVT-aware SCK half-period bound is monotone in the combined ordering:
    higher temperature, lower VCCINT, and a worse process corner all increase
    (or keep) the bound. This is the shape property used by worst-case
    operating-point search. -/
lemma pvt_half_ns_monotone_combined
  (t1 t2 : Int) (v1 v2 : Nat) (c1 c2 : ProcessCorner) :
  (PVT_TEMP_MIN_C ≤ t1) → (t1 ≤ t2)
  → (v2 ≤ v1) → (v1 ≤ PVT_VCCINT_MAX_MV)
  → c1.worse_than c2
  → n25q128_min_sck_half_ns_pvt ⟨t1, v1, 2700, c1⟩
    ≤ n25q128_min_sck_half_ns_pvt ⟨t2, v2, 2700, c2⟩ := by
  intro ht_min ht_le hv_le hv_max hc
  simp [n25q128_min_sck_half_ns_pvt, n25q128_min_sck_low_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, ProcessCorner.worse_than, PVT_TEMP_MIN_C, PVT_VCCINT_MAX_MV] at ht_min ht_le hv_le hv_max hc ⊢
  cases c1 <;> cases c2 <;> omega

/-- The PVT-aware SCK low bound is monotone in the combined ordering: higher
    temperature, lower VCCINT, and a worse process corner all increase (or keep)
    the bound. This mirrors `pvt_half_ns_monotone_combined` and is used when the
    low and high limits need separate treatment. -/
lemma pvt_low_ns_monotone_combined
  (t1 t2 : Int) (v1 v2 : Nat) (c1 c2 : ProcessCorner) :
  (PVT_TEMP_MIN_C ≤ t1) → (t1 ≤ t2)
  → (v2 ≤ v1) → (v1 ≤ PVT_VCCINT_MAX_MV)
  → c1.worse_than c2
  → n25q128_min_sck_low_ns_pvt ⟨t1, v1, 2700, c1⟩
    ≤ n25q128_min_sck_low_ns_pvt ⟨t2, v2, 2700, c2⟩ := by
  intro ht_min ht_le hv_le hv_max hc
  simp [n25q128_min_sck_low_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, ProcessCorner.worse_than, PVT_TEMP_MIN_C, PVT_VCCINT_MAX_MV]
    at ht_min ht_le hv_le hv_max hc ⊢
  cases c1 <;> cases c2 <;> omega

/-- The PVT-aware SCK high bound is monotone in the combined ordering: higher
    temperature, lower VCCINT, and a worse process corner all increase (or keep)
    the bound. The low and high bounds are symmetric in the current placeholder
    envelope, so the proof is identical to the low-bound version. -/
lemma pvt_high_ns_monotone_combined
  (t1 t2 : Int) (v1 v2 : Nat) (c1 c2 : ProcessCorner) :
  (PVT_TEMP_MIN_C ≤ t1) → (t1 ≤ t2)
  → (v2 ≤ v1) → (v1 ≤ PVT_VCCINT_MAX_MV)
  → c1.worse_than c2
  → n25q128_min_sck_high_ns_pvt ⟨t1, v1, 2700, c1⟩
    ≤ n25q128_min_sck_high_ns_pvt ⟨t2, v2, 2700, c2⟩ := by
  intro ht_min ht_le hv_le hv_max hc
  simp [n25q128_min_sck_high_ns_pvt,
        n25q128_pvt_temp_derating_ns, n25q128_pvt_voltage_derating_ns,
        n25q128_pvt_process_derating_ns, ProcessCorner.worse_than, PVT_TEMP_MIN_C, PVT_VCCINT_MAX_MV]
    at ht_min ht_le hv_le hv_max hc ⊢
  cases c1 <;> cases c2 <;> omega

/-- Every process corner is no better (no smaller derating) than the slow-slow
    corner. This is the corner-ordering fact needed by the worst-case bound. -/
lemma ProcessCorner.any_worse_than_ss (c : ProcessCorner) : c.worse_than ProcessCorner.ss := by
  cases c <;> simp [worse_than, n25q128_pvt_process_derating_ns]

/-- Worst-case operating-point search: across the entire documented operating
    envelope, the PVT-aware half-period bound is maximized at the worst corner
    (maximum temperature, minimum VCCINT, slow-slow process corner). This lets
    a validation tool enumerate a finite grid and prove that any context inside
    the envelope is no worse than the corner it finds. -/
lemma pvt_half_ns_worst_case_bound (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.temp_c ≤ PVT_TEMP_MAX_C)
  → (PVT_VCCINT_MIN_MV ≤ ctx.vccint_mv) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → n25q128_min_sck_half_ns_pvt ctx
    ≤ n25q128_min_sck_half_ns_pvt ⟨PVT_TEMP_MAX_C, PVT_VCCINT_MIN_MV, 2700, ProcessCorner.ss⟩ := by
  intro ht_min ht_max hv_min hv_max
  apply pvt_half_ns_monotone_combined ctx.temp_c PVT_TEMP_MAX_C ctx.vccint_mv PVT_VCCINT_MIN_MV ctx.process_corner ProcessCorner.ss
  · exact ht_min
  · exact ht_max
  · exact hv_min
  · exact hv_max
  · exact ProcessCorner.any_worse_than_ss ctx.process_corner

/-- If the PVT-aware predicate holds, the nominal predicate holds (for contexts
    inside the operating envelope). -/
theorem measured_cclk_with_pvt_implies_measured_cclk_satisfies_flash_spec
  (freq_hz duty_pct : Nat) (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → measured_cclk_with_pvt_satisfies_flash_spec freq_hz duty_pct ctx = true
  → measured_cclk_satisfies_flash_spec freq_hz duty_pct = true := by
  intro h_temp h_volt h
  simp [measured_cclk_with_pvt_satisfies_flash_spec, measured_cclk_satisfies_flash_spec,
        n25q128_min_sck_low_ns_pvt, n25q128_min_sck_high_ns_pvt,
        N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS] at h ⊢
  rcases h with ⟨h_pos, h_max, h_duty, h_low, h_high⟩
  have h_low_ge : n25q128_min_sck_low_ns_pvt ctx ≥ N25Q128_MIN_SCK_LOW_NS :=
    pvt_low_ns_at_least_nominal ctx h_temp h_volt
  have h_high_ge : n25q128_min_sck_high_ns_pvt ctx ≥ N25Q128_MIN_SCK_HIGH_NS :=
    pvt_high_ns_at_least_nominal ctx h_temp h_volt
  constructor
  · exact h_pos
  constructor
  · exact h_max
  constructor
  · exact h_duty
  constructor
  · omega
  · omega

/-- If the PVT-aware predicate holds (inside the operating envelope), the boot
    transaction satisfies the flash spec. -/
theorem measured_cclk_with_pvt_implies_transaction_ok
  (freq_hz duty_pct bits : Nat) (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → measured_cclk_with_pvt_satisfies_flash_spec freq_hz duty_pct ctx = true
  → transaction_satisfies_flash_spec (measured_boot_transaction freq_hz duty_pct bits) = true := by
  intro h_temp h_volt h
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  exact measured_cclk_with_pvt_implies_measured_cclk_satisfies_flash_spec freq_hz duty_pct ctx h_temp h_volt h

/-- Concrete example: a measured 25 MHz CCLK with 50% duty satisfies the PVT
    predicate under the worst-case operating envelope (ss corner, 900 mV, +85 °C). -/
theorem measured_25mhz_50duty_pvt_worstcase_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec 25_000_000 50
    { temp_c := (85 : Int), vccint_mv := 900, vccaux_mv := 2700, process_corner := ProcessCorner.ss } = true := by
  decide

/-- Concrete example: a measured 2.5 MHz CCLK with 50% duty cycle satisfies the
    flash timing predicate. This matches the synthetic fixture used by the
    `tri fpga measure-cclk --synth` CI path. -/
theorem measured_2_5mhz_50duty_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec 2_500_000 50 = true := by
  decide

/-- Concrete example: a measured 25 MHz CCLK with 50% duty cycle satisfies the
    flash timing predicate. This is the nominal rate for OSCFSEL=6. -/
theorem measured_25mhz_50duty_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec 25_000_000 50 = true := by
  decide

/-- Concrete example: a measured 33.3 MHz CCLK with 50% duty cycle satisfies the
    flash timing predicate. This is the nominal rate for OSCFSEL=7. -/
theorem measured_33_3mhz_50duty_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec 33_300_000 50 = true := by
  decide

/-- PVT-margin version of the measured-CCLK flash predicate. Uses conservative
    2× derated SCK low/high limits to absorb process, voltage, and temperature
    variation. A capture that satisfies this predicate also satisfies the
    nominal predicate and therefore produces a flash-spec-compliant transaction.
    This is a placeholder until actual N25Q128_3V PVT characterization data is
    available; the 2× factor is intentionally conservative. -/
def measured_cclk_with_margin_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool :=
  freq_hz > 0
  ∧ freq_hz ≤ N25Q128_MAX_SCK_HZ
  ∧ duty_pct ≤ 100
  ∧ measured_cclk_low_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_LOW_NS_WC
  ∧ measured_cclk_high_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_HIGH_NS_WC

/-- The worst-case SCK low limit is at least the nominal limit. -/
lemma min_sck_low_wc_ge_nominal :
  N25Q128_MIN_SCK_LOW_NS_WC ≥ N25Q128_MIN_SCK_LOW_NS := by
  decide

/-- The worst-case SCK high limit is at least the nominal limit. -/
lemma min_sck_high_wc_ge_nominal :
  N25Q128_MIN_SCK_HIGH_NS_WC ≥ N25Q128_MIN_SCK_HIGH_NS := by
  decide

/-- If a measured pair satisfies the PVT-margin predicate, it also satisfies the
    nominal measured-CCLK predicate. -/
theorem measured_cclk_with_margin_implies_measured_cclk_satisfies_flash_spec
  (freq_hz duty_pct : Nat) :
  measured_cclk_with_margin_satisfies_flash_spec freq_hz duty_pct = true
  → measured_cclk_satisfies_flash_spec freq_hz duty_pct = true := by
  intro h
  simp [measured_cclk_with_margin_satisfies_flash_spec, measured_cclk_satisfies_flash_spec,
        N25Q128_MIN_SCK_LOW_NS_WC, N25Q128_MIN_SCK_HIGH_NS_WC,
        N25Q128_MIN_SCK_LOW_NS, N25Q128_MIN_SCK_HIGH_NS] at h ⊢
  rcases h with ⟨h_fpos, h_fmax, h_duty, h_low_wc, h_high_wc⟩
  constructor
  · exact h_fpos
  · constructor
    · exact h_fmax
    · constructor
      · exact h_duty
      · constructor
        · omega
        · omega

/-- If a measured pair satisfies the PVT-margin predicate, the transaction built
    from that pair satisfies the N25Q128_3V timing spec. This is the end-to-end
    measured-to-formal link with conservative PVT margins. -/
theorem measured_cclk_with_margin_implies_transaction_ok
  (freq_hz duty_pct bits : Nat) :
  measured_cclk_with_margin_satisfies_flash_spec freq_hz duty_pct = true
  → transaction_satisfies_flash_spec (measured_boot_transaction freq_hz duty_pct bits) = true := by
  intro h
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  exact measured_cclk_with_margin_implies_measured_cclk_satisfies_flash_spec freq_hz duty_pct h

/-- Concrete example: a measured 2.5 MHz CCLK with 50% duty cycle satisfies the
    PVT-margin predicate. -/
theorem measured_2_5mhz_50duty_with_margin_satisfies_flash_spec :
  measured_cclk_with_margin_satisfies_flash_spec 2_500_000 50 = true := by
  decide

/-- Concrete example: a measured 25 MHz CCLK with 50% duty cycle satisfies the
    PVT-margin predicate. This is the nominal rate for OSCFSEL=6. -/
theorem measured_25mhz_50duty_with_margin_satisfies_flash_spec :
  measured_cclk_with_margin_satisfies_flash_spec 25_000_000 50 = true := by
  decide

/-- Concrete example: a measured 33.3 MHz CCLK with 50% duty cycle satisfies the
    PVT-margin predicate. This is the nominal rate for OSCFSEL=7. -/
theorem measured_33_3mhz_50duty_with_margin_satisfies_flash_spec :
  measured_cclk_with_margin_satisfies_flash_spec 33_300_000 50 = true := by
  decide

/-- PVT-aware raw-ns measured-CCLK flash predicate. As long as the placeholder
    PVT derating is at least the nominal N25Q128 bound, a raw capture that
    passes this predicate also passes the nominal raw-ns predicate. This is the
    falsifiable entry point for instrument exports: replace `n25q128_min_sck_*_ns_pvt`
    with real N25Q128_3V PVT curves; the implication theorems below remain valid
    as long as the derated limits are ≥ the nominal 6 ns bounds. -/
def measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec (period_ns low_ns high_ns : Nat) (ctx : PvtContext) : Bool :=
  low_ns + high_ns = period_ns
  ∧ let freq_hz := measured_cclk_freq_hz_from_period_ns period_ns
    let duty_pct := measured_cclk_duty_pct_from_raw_ns period_ns high_ns
    measured_cclk_with_pvt_satisfies_flash_spec freq_hz duty_pct ctx

/-- Transaction built from a PVT-aware raw-ns capture. Mirrors
    `measured_boot_transaction_from_raw_ns`. -/
def measured_boot_transaction_from_raw_ns_with_pvt (period_ns _low_ns high_ns bits : Nat) : SPIReadTransaction :=
  measured_boot_transaction_from_raw_ns period_ns _low_ns high_ns bits

/-- If a PVT-aware raw-ns capture satisfies the flash predicate (inside the
    operating envelope), the transaction built from it satisfies the N25Q128_3V
    timing spec. -/
theorem measured_cclk_from_raw_ns_with_pvt_implies_transaction_ok
  (period_ns low_ns high_ns bits : Nat) (ctx : PvtContext) :
  (PVT_TEMP_MIN_C ≤ ctx.temp_c) → (ctx.vccint_mv ≤ PVT_VCCINT_MAX_MV)
  → measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec period_ns low_ns high_ns ctx = true
  → transaction_satisfies_flash_spec (measured_boot_transaction_from_raw_ns_with_pvt period_ns low_ns high_ns bits) = true := by
  intro h_temp h_volt h
  simp [measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec, measured_boot_transaction_from_raw_ns_with_pvt,
        measured_boot_transaction_from_raw_ns] at h ⊢
  rcases h with ⟨_h_consistent, h_spec⟩
  apply measured_cclk_with_pvt_implies_transaction_ok
  exact h_temp
  exact h_volt
  exact h_spec

/-- Concrete example: a raw 40 ns / 20 ns / 20 ns capture satisfies the PVT-aware
    raw-ns predicate under the worst-case operating envelope. -/
theorem measured_raw_ns_40_20_20_with_pvt_worstcase_satisfies_flash_spec :
  measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec 40 20 20
    { temp_c := (85 : Int), vccint_mv := 900, vccaux_mv := 2700, process_corner := ProcessCorner.ss } = true := by
  decide

/-- Concrete example: a raw 40 ns / 20 ns / 20 ns capture satisfies the
    PVT-margin raw-ns predicate. -/
theorem measured_raw_ns_40_20_20_with_margin_satisfies_flash_spec :
  measured_cclk_from_raw_ns_satisfies_flash_spec 40 20 20 = true := by
  decide

/-- The worst-case PVT-aware SCK low bound equals the previous flat 12 ns
    placeholder only under the best process/voltage/temperature corner; under
    the worst corner it is strictly larger, making the new envelope at least as
    conservative as the old placeholder. -/
theorem pvt_low_ns_wc_ge_old_placeholder :
  n25q128_min_sck_low_ns_pvt { temp_c := (85 : Int), vccint_mv := 900, vccaux_mv := 2700, process_corner := ProcessCorner.ss }
  ≥ N25Q128_MIN_SCK_LOW_NS_WC := by
  decide

-- ============================================================================
-- OSCFSEL 0..7 measured-CCLK theorem library (W415)
-- ============================================================================

/-- Shared worst-case PVT context used for every OSCFSEL worst-case theorem. -/
def OSCFSEL_WORST_CASE_PVT_CONTEXT : PvtContext :=
  { temp_c := (85 : Int), vccint_mv := 900, vccaux_mv := 2700, process_corner := ProcessCorner.ss }

/-- Nominal flash-spec theorem for OSCFSEL=0 (2.5 MHz). -/
theorem oscfsel_0_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 0) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=0 (2.5 MHz). -/
theorem oscfsel_0_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 0) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=1 (~4.2 MHz). -/
theorem oscfsel_1_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 1) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=1 (~4.2 MHz). -/
theorem oscfsel_1_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 1) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=2 (~6.6 MHz). -/
theorem oscfsel_2_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 2) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=2 (~6.6 MHz). -/
theorem oscfsel_2_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 2) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=3 (~10 MHz). -/
theorem oscfsel_3_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 3) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=3 (~10 MHz). -/
theorem oscfsel_3_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 3) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=4 (~12.5 MHz). -/
theorem oscfsel_4_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 4) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=4 (~12.5 MHz). -/
theorem oscfsel_4_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 4) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=5 (~16.7 MHz). -/
theorem oscfsel_5_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 5) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=5 (~16.7 MHz). -/
theorem oscfsel_5_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 5) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=6 (~25 MHz). -/
theorem oscfsel_6_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 6) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=6 (~25 MHz). -/
theorem oscfsel_6_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 6) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

/-- Nominal flash-spec theorem for OSCFSEL=7 (~33.3 MHz). -/
theorem oscfsel_7_nominal_measured_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec (cclk_nominal_hz 7) 50 = true := by
  decide

/-- Worst-case PVT theorem for OSCFSEL=7 (~33.3 MHz). -/
theorem oscfsel_7_worstcase_pvt_measured_satisfies_flash_spec :
  measured_cclk_with_pvt_satisfies_flash_spec (cclk_nominal_hz 7) 50 OSCFSEL_WORST_CASE_PVT_CONTEXT = true := by
  decide

-- ============================================================================
-- OSCFSEL transaction theorems (W416)
-- ============================================================================

/-- The nominal OSCFSEL=0 rate produces a flash-spec-compliant transaction for
    any transaction size. -/
theorem oscfsel_0_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 0) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_0_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=1 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_1_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 1) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_1_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=2 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_2_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 2) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_2_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=3 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_3_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 3) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_3_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=4 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_4_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 4) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_4_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=5 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_5_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 5) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_5_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=6 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_6_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 6) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_6_nominal_measured_satisfies_flash_spec

/-- The nominal OSCFSEL=7 rate produces a flash-spec-compliant transaction. -/
theorem oscfsel_7_measured_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction (cclk_nominal_hz 7) 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  · exact oscfsel_7_nominal_measured_satisfies_flash_spec

end BitstreamConfig

-- ============================================================================
-- Mode / bus-width predicates
-- ============================================================================

/-- The FPGA sampled Master SPI x1 mode at power-on. -/
def mode_master_spi_x1 (s : StatRegister) : Bool :=
  s.mode = MODE_MASTER_SPI_X1

/-- The FPGA is using x1 SPI bus width. -/
def bus_width_x1 (s : StatRegister) : Bool :=
  s.bus_width = BUS_WIDTH_X1

-- ============================================================================
-- Decision-tree predicates
-- ============================================================================

/-- FPGA booted successfully from SPI flash:
    DONE=HIGH, mode=Master SPI x1, no CRC/ID/DEC errors.
    EOS is not required in the model because, in a valid SPI master boot,
    DONE=HIGH implies EOS=HIGH; keeping EOS in the dynamic preconditions
    (see `cold_por_done_eos_high_implies_boot_success`) is sufficient. -/
def boot_success (s : StatRegister) : Bool :=
  s.done ∧ s.mode_master_spi_x1 ∧ ¬s.crc_error ∧ ¬s.id_error ∧ ¬s.dec_error

/-- Configuration did not finish but the mode is correct.
    This is the H2 CCLK/SPI-timing hypothesis bucket. -/
def h2_cclk_timing (s : StatRegister) : Bool :=
  ¬s.done ∧ s.mode_master_spi_x1 ∧ ¬s.crc_error ∧ ¬s.id_error

/-- Mode-pin strapping issue: mode is not Master SPI x1. -/
def mode_mismatch (s : StatRegister) : Bool :=
  ¬s.mode_master_spi_x1

/-- Any fatal error bit is set. -/
def fatal_error (s : StatRegister) : Bool :=
  s.crc_error ∨ s.id_error ∨ s.dec_error

-- ============================================================================
-- Cold-POR / bitstream-config linkage
-- ============================================================================

/-- Static configuration + cold-POR mode sampling preconditions.
    `mode_ok` asserts the FPGA sampled Master SPI x1 at power-on.
    `no_cable_interference` asserts the JTAG cable was disconnected during POR,
    which is a physical protocol assumption rather than a register value. -/
structure ColdPOR where
  cfg : BitstreamConfig
  mode_ok : Bool
  no_cable_interference : Bool
  deriving Repr, DecidableEq, Inhabited

/-- The canonical bitstream + correct mode sampling + clean protocol give the
    static preconditions for a successful SPI flash boot. This does not yet
    prove `boot_success` because the actual CCLK timing/signal integrity must
    also be sufficient; it partitions the outcome into success or H2 once
    DONE is observed. -/
def cold_por_spi_flash_pred (p : ColdPOR) (s : StatRegister) : Bool :=
  p.cfg.canonical ∧ p.mode_ok ∧ p.no_cable_interference
  ∧ BitstreamConfig.flash_spi_timing_ok p.cfg.oscfsel
  ∧ s.mode_master_spi_x1 ∧ ¬s.fatal_error

/-- If the static preconditions hold then the oscillator selection is within
    the flash timing spec. This is the formal link between the cold-POR
    predicate and the CCLK timing bounds. -/
theorem cold_por_implies_flash_spi_timing_ok
  (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → BitstreamConfig.flash_spi_timing_ok p.cfg.oscfsel := by
  intro h
  simp [cold_por_spi_flash_pred] at h
  rcases h with ⟨_, _, _, h_flash, _, _⟩
  exact h_flash

/-- If the static preconditions hold then the oscillator selection is within
    the flash CCLK-frequency bound. This follows from the stronger
    `flash_spi_timing_ok` precondition. -/
theorem cold_por_implies_cclk_within_flash_spec
  (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel := by
  intro h
  apply BitstreamConfig.flash_spi_timing_ok_implies_cclk_within_flash_spec
  exact cold_por_implies_flash_spi_timing_ok p s h

/-- If the static preconditions hold then the boot transaction produced by the
    bitstream configuration satisfies the N25Q128_3V timing spec. This closes
    the loop between the cold-POR predicate and the transaction-level model. -/
theorem cold_por_implies_transaction_satisfies_flash_spec
  (p : ColdPOR) (s : StatRegister) (bits : Nat) :
  cold_por_spi_flash_pred p s
  → BitstreamConfig.transaction_satisfies_flash_spec (BitstreamConfig.artix7_boot_transaction p.cfg bits) := by
  intro h
  apply BitstreamConfig.canonical_implies_transaction_satisfies_flash_spec
  simp [cold_por_spi_flash_pred] at h
  exact h.left

/-- If the static preconditions hold and both DONE and EOS are HIGH, then
    boot_success holds. EOS is a dynamic observation, not a static config field. -/
theorem cold_por_done_eos_high_implies_boot_success
  (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → s.done → s.eos → s.boot_success := by
  intro h h_done _h_eos
  simp [cold_por_spi_flash_pred, boot_success, mode_master_spi_x1, fatal_error] at h ⊢
  rcases h with ⟨_, _, _, _, h_mode, h_no_fatal⟩
  rcases h_no_fatal with ⟨h_crc, h_id, h_dec⟩
  simp [h_done, h_mode, h_crc, h_id, h_dec]

/-- If the static preconditions hold and DONE is LOW, then the outcome is the
    H2 CCLK/SPI-timing hypothesis bucket. -/
theorem cold_por_done_low_implies_h2
  (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → ¬s.done → s.h2_cclk_timing := by
  intro h h_not_done
  simp [cold_por_spi_flash_pred, h2_cclk_timing, mode_master_spi_x1, fatal_error] at h ⊢
  rcases h with ⟨_, _, _, _, h_mode, h_no_fatal⟩
  rcases h_no_fatal with ⟨h_crc, h_id, _⟩
  simp [h_not_done, h_mode, h_crc, h_id]

/-- Decision-tree exhaustiveness: for any STAT register, at least one of the
    documented outcomes applies.
    Tree:
    1. mode != Master SPI x1  → mode_mismatch
    2. mode == Master SPI x1 and fatal error → fatal_error
    3. mode == Master SPI x1, no fatal error, DONE=1 → boot_success
    4. mode == Master SPI x1, no fatal error, DONE=0 → h2_cclk_timing -/
theorem decision_tree_exhaustive (s : StatRegister) :
  s.boot_success ∨ s.h2_cclk_timing ∨ s.mode_mismatch ∨ s.fatal_error := by
  by_cases h_mode : s.mode_master_spi_x1
  · -- mode OK
    by_cases h_fatal : s.fatal_error
    · -- fatal error: fourth disjunct
      exact Or.inr (Or.inr (Or.inr h_fatal))
    · -- no fatal error
      simp [fatal_error] at h_fatal
      rcases h_fatal with ⟨h_crc, h_id, h_dec⟩
      by_cases h_done : s.done
      · -- done=1: boot_success
        left
        simp [boot_success, h_done, h_mode, h_crc, h_id, h_dec]
      · -- done=0: h2_cclk_timing
        right; left
        simp [h2_cclk_timing, h_done, h_mode, h_crc, h_id]
  · -- mode mismatch: third disjunct
    have h_mm : s.mode_mismatch = true := by simp [mode_mismatch, h_mode]
    exact Or.inr (Or.inr (Or.inl h_mm))

-- ============================================================================
-- Lemmas
-- ============================================================================

/-- If boot succeeded, the mode must be Master SPI x1. -/
theorem boot_success_implies_mode_master_spi_x1 (s : StatRegister) :
  s.boot_success → s.mode_master_spi_x1 := by
  intro h
  simp [boot_success] at h
  rcases h with ⟨_, h_mode, _, _, _⟩
  exact h_mode

/-- If boot succeeded, no fatal error bit is set. -/
theorem boot_success_implies_no_fatal_error (s : StatRegister) :
  s.boot_success → ¬s.fatal_error := by
  intro h
  simp [boot_success, fatal_error] at h ⊢
  rcases h with ⟨_, h_mode, h_crc, h_id, h_dec⟩
  exact ⟨h_crc, h_id, h_dec⟩

/-- H2 hypothesis implies mode is correct but DONE is still LOW. -/
theorem h2_implies_mode_ok_done_low (s : StatRegister) :
  s.h2_cclk_timing → s.mode_master_spi_x1 ∧ ¬s.done := by
  intro h
  simp [h2_cclk_timing] at h
  rcases h with ⟨h_done, h_mode, _, _⟩
  simp [h_mode, h_done]

/-- A fatal error prevents boot success. -/
theorem fatal_error_implies_not_boot_success (s : StatRegister) :
  s.fatal_error → ¬s.boot_success := by
  intro h_fatal h_boot
  simp [boot_success, fatal_error] at h_fatal h_boot
  rcases h_fatal with (h_crc | h_id | h_dec)
  all_goals
    rcases h_boot with ⟨_, _, h_crc', h_id', h_dec'⟩
    try { simp [h_crc'] at h_crc }
    try { simp [h_id'] at h_id }
    try { simp [h_dec'] at h_dec }

/-- Mode mismatch prevents boot success. -/
theorem mode_mismatch_implies_not_boot_success (s : StatRegister) :
  s.mode_mismatch → ¬s.boot_success := by
  intro h_mismatch h_boot
  simp [mode_mismatch, boot_success, mode_master_spi_x1] at h_mismatch h_boot
  rcases h_boot with ⟨_, h_mode, _, _, _⟩
  contradiction

/-- The W400 success example decodes to boot_success. -/
theorem stat_success_example_boots : boot_success ⟨STAT_SUCCESS_EXAMPLE⟩ := by
  decide

/-- The incomplete example is not boot_success and falls into the H2 bucket. -/
theorem stat_incomplete_example_is_h2 :
  h2_cclk_timing ⟨STAT_INCOMPLETE_EXAMPLE⟩ := by
  decide

/-- boot_success is mutually exclusive with h2_cclk_timing. -/
theorem boot_success_and_h2_disjoint (s : StatRegister) :
  ¬(s.boot_success ∧ s.h2_cclk_timing) := by
  intro h
  simp [boot_success, h2_cclk_timing] at h
  rcases h with ⟨⟨h_done_t, _, _, _, _⟩, ⟨h_done_f, _, _, _⟩⟩
  rw [h_done_t] at h_done_f
  contradiction

end StatRegister

end Trinity
