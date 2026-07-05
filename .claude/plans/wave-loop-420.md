# Wave Loop 420 — Decomposed Plan

**Issue:** #1361  
**Branch:** `wave-loop-420`  
**Default variant:** C (physical bench still blocked: DLC10 cable not found, P12 unwired, no relay)

---

## 1. Weak points identified

1. **Missing VCD `$comment` exact-token terminator.** The W419 report claimed this
   hardening landed, but the merged commit (`101fd0748`) did not touch the VCD
   parser. The current code still uses `ends_with("$end")` / `contains(" $end")`
   heuristics, so a `$comment` block containing the literal substring `$end`
   terminates early and corrupts the signal dictionary.
2. **Real-valued VCD nets require an explicit `--vcd-threshold-v`.** There is no
   auto-threshold fallback for analog VCD exports; users must know the voltage
   swing up-front.
3. **PVT half-period bound lacks a process-corner monotonicity lemma.** We have
   temperature monotonicity and VCCINT antitonicity from W419, but no formal
   statement that `ff ≤ tt ≤ ss` is preserved by the half-period bound itself.

---

## 2. Competitor landscape (high-level)

The closest formal-HDL competition is **Sparkle / Verilean** (Lean 4 native,
binary RTL). Other credible players: **Clash** (Haskell, external formal),
**Chisel/FIRRTL/CIRCT** (mainstream, SVA/model-checking oriented),
**Bluespec** (rule-based, Coq bridge via Kami), **Coq Kami / Silver Oak**
(dependent-type hardware), **ACL2** (specification/proof layer only).

None combine Lean 4 native proof, a ternary compute stack, spec-first sealed
`*.t27 → gen/` code generation, and physical boot-evidence instrumentation.
That intersection is the gap t27 fills.

---

## 3. Implementation steps

| Step | Files | Deliverable | Acceptance |
|------|-------|-------------|------------|
| 3.1 | `cli/tri/src/fpga.rs` | Replace VCD `$date`/`$version`/`$comment` terminator heuristics with exact-token check; clear state immediately on same-line termination. | `test_parse_vcd_comment_with_embedded_end_token` PASS |
| 3.2 | `cli/tri/src/fpga.rs` | Add VCD auto-threshold for real-valued nets: compute `50% (vmin + vmax)` when `--vcd-threshold-v` is omitted; warn user. | `test_parse_vcd_real_auto_threshold` PASS |
| 3.3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Add `pvt_half_ns_monotone_in_process_corner` lemma: `c1.worse_than c2 → bound c1 ≤ bound c2`. | `lake build` PASS |
| 3.4 | `cli/tri/src/fpga.rs` | Add Rust test `test_pvt_half_ns_monotone_in_process_corner`. | `cargo test` PASS |
| 3.5 | `fpga/HARDWARE_SSOT.md` | Document VCD `$comment` exact-terminator behavior and real-net auto-threshold. | docs render clean |
| 3.6 | `.trinity/experience.md` | Capture W420 learnings. | committed |
| 3.7 | `docs/reports/*` | W420 report, evidence, W421 cooperation variants. | committed |

---

## 4. Verification plan

- `cargo test -p tri vcd` — expect 12 tests (was 11).
- `cargo test -p tri pvt` — expect 10 tests (was 9).
- `cargo test -p tri fpga::tests` — expect 47 tests (was 45).
- `lake build Trinity.TernaryFPGABoot` — must PASS.
- `./scripts/tri test` — must PASS except 16 pre-existing gen-verilog yosys failures.

---

## 5. Land

- Commit with `Closes #1361`.
- Open PR #? from `wave-loop-420` → `master`.
- After merge, create issue #? and branch `wave-loop-421`.

*φ² + φ⁻² = 3 | TRINITY*
