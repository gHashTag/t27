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
    DONE=HIGH, EOS=HIGH, mode=Master SPI x1, no CRC/ID/DEC errors. -/
def boot_success (s : StatRegister) : Bool :=
  s.done ∧ s.eos ∧ s.mode_master_spi_x1 ∧ ¬s.crc_error ∧ ¬s.id_error ∧ ¬s.dec_error

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
-- Lemmas
-- ============================================================================

/-- If boot succeeded, the mode must be Master SPI x1. -/
theorem boot_success_implies_mode_master_spi_x1 (s : StatRegister) :
  s.boot_success → s.mode_master_spi_x1 := by
  intro h
  simp [boot_success] at h
  rcases h with ⟨_, _, h_mode, _, _, _⟩
  exact h_mode

/-- If boot succeeded, no fatal error bit is set. -/
theorem boot_success_implies_no_fatal_error (s : StatRegister) :
  s.boot_success → ¬s.fatal_error := by
  intro h
  simp [boot_success, fatal_error] at h ⊢
  rcases h with ⟨_, _, _, h_crc, h_id, h_dec⟩
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
  rcases h_fatal with (h | h | h)
  all_goals
    rcases h_boot with ⟨_, _, _, h_crc, h_id, h_dec⟩
    simp [h_crc, h_id, h_dec] at h

/-- Mode mismatch prevents boot success. -/
theorem mode_mismatch_implies_not_boot_success (s : StatRegister) :
  s.mode_mismatch → ¬s.boot_success := by
  intro h_mismatch h_boot
  simp [mode_mismatch, boot_success, mode_master_spi_x1] at h_mismatch h_boot
  rcases h_boot with ⟨_, _, h_mode, _, _, _⟩
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
  rcases h with ⟨⟨h_done_t, _, _, _, _, _⟩, ⟨h_done_f, _, _, _⟩⟩
  rw [h_done_t] at h_done_f
  contradiction

end StatRegister

end Trinity
