/- SPDX-License-Identifier: Apache-2.0
   proofs/lean4/Trinity/TernaryFPGABoot.lean
   Formal model of the 7-series FPGA STAT register and the cold-POR
   decision tree documented in fpga/HARDWARE_SSOT.md.
   phi^2 + 1/phi^2 = 3 | TRINITY -/

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

/-- A given raw OSCFSEL selection is within the flash timing spec when its
    nominal CCLK is non-zero and does not exceed the flash maximum SCK
    frequency. This is a static, conservative predicate: it does not account for
    temperature/voltage/process variation; those are covered by the margin
    between the nominal CCLK and the flash limit. -/
def cclk_within_flash_spec (oscfsel : UInt8) : Bool :=
  let f := cclk_nominal_hz oscfsel.toNat
  f > 0 ∧ f ≤ N25Q128_MAX_SCK_HZ

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
  ∧ BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel
  ∧ s.mode_master_spi_x1 ∧ ¬s.fatal_error

/-- If the static preconditions hold then the oscillator selection is within
    the flash timing spec. This is the formal link between the cold-POR
    predicate and the CCLK timing bounds. -/
theorem cold_por_implies_cclk_within_flash_spec
  (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel := by
  intro h
  simp [cold_por_spi_flash_pred] at h
  rcases h with ⟨_, _, _, h_cclk, _, _⟩
  exact h_cclk

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
