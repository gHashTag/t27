# Wave Loop 885 — Issue TBD

**Branch:** `wave-loop-885` (to be created from `wave-loop-884` HEAD)
**Parent branch:** `wave-loop-884` HEAD
**Date:** 2026-08-06
**Issue:** TBD (create after W884 issue #1828 / PR #1829 lands)
**PR:** TBD (to open)
**Cooperation variant:** A (recommended)

## Goal

Select one of three W885 cooperation variants and close the wave with a green targeted
test, updated seals, and the standard close-out artifacts (report, evidence, next-wave
plan).

Close Wave Loop 885 by validating a module-scope `[589][2]^6 Pt` packed array-of-struct
variable initialized from a function call, with indexed signed field writes and `assert_eq`
read-back in a `bench` block. Earlier wave PRs (#1810 W881, #1813 W882, #1815 W883,
#1829 W884) remain open awaiting review, so W885 will be branched from `wave-loop-884`
HEAD to avoid blocking the sequence.

## Acceptance criteria

- [ ] Generator `scripts/gen_w885.py` with `OUTER = 589`, `MID_IDX = 294`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w885_bench_module_589x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w885_bench_module_589x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] Full `cargo test --release --test icarus_lowerable` passes at **345/0** (targeted test green; pre-existing classifier failure tracked separately).
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #<W885-issue>`, push branch, open PR to `master`.

## Notes

- Shape: `[589][2]^6 Pt` where `Pt = pub struct Pt { x : i16, y : i16 }`.
- Total elements: `589 x 64 = 37,696`.
- Packed vector width: `589 x 32 = 1,206,272` bits (~1.151 MiBit).
- `MID_IDX = 294`; frame-condition element `[294][1][0][0][0][0][0]` is element
  `294*64 + 32 = 18,848`.
- Generator script: `scripts/gen_w885.py` (copy from `scripts/gen_w884.py`, set
  `OUTER = 589` and `MID_IDX = 294`, fix module prefix).
- Use `assert_eq` checks on changed elements (Icarus simulation path does not emit `assert_ne`).
- Include `make_grid(32768)` period-identity check because `32768 == 0 (mod 32768)`.
- Zero compiler / reference-model / `FROZEN_HASH` changes expected for the witness.

---

- **Variant A (recommended):** continue the odd outer-dimension ladder with `[589][2]^6 Pt`.
- **Variant B:** keep width at ~1.151 MiBit but move the packed var to bench/function scope.
- **Variant C:** add `if`-guarded indexed signed field writes at the current width.

---

phi^2 + 1/phi^2 = 3 | TRINITY
