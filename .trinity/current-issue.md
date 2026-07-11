# Wave Loop 492 — Next wave (to be selected from cooperation plan)

**Issue:** #1462 (to create)  
**Branch:** `wave-loop-492`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selection

Choose one of the three W492 variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md`:

- **Variant A (default) — Extend the Lean 4 lowerability proof.** Prove soundness
  of the Rust classifier with respect to emitted Verilog, and completeness for
  the current Icarus-passing corpus.
- **Variant B — Continue gen-verilog struct/call lowering hardening.** Close
  nested struct-return field access, module-scope AOS constants from imported
  calls, and host-only propagation across import boundaries.
- **Variant C — FPGA live cold-POR / SPI flash boot evidence.** Collect physical
  boot traces on the QMTech Wukong V1 / XC7A100T-FGG676 and compare against the
  formal PVT envelope.

## Common gate

Whatever variant is chosen, keep:

- 691 / 691 non-smoke PASS (681 base + 6 W490 + 4 W491 scratch witnesses).
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable`: zero
  disagreements.

The W491 branch must land before W492 work begins.

---

*φ² + φ⁻² = 3 | TRINITY*
