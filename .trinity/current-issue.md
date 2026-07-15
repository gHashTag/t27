# Wave Loop 544 — mutable module vars and test-block call assignments for independent VCD cross-check

**Issue:** #1515 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-544`  
**Source:** `docs/reports/FPGA_LOOP_COOPERATION_W544_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Enable independent cocotb VCD cross-checks for module-level mutable vars whose
initializer is a function call and for whole-struct assignments inside test
blocks whose RHS is a function call.  This closes the last mutable-state gap in
the call-evaluation path opened in Wave Loops 542 and 543.

## Scope

1. Extend `scripts/cocotb_ref_model.py` so that mutable module vars with
   lowerable function-call initializers are bound eagerly.
2. Verify that `_collect_assertions` already updates `ctx.vars[lhs]` for
   whole-struct assignments to mutable module vars; add a witness where the RHS
   of such an assignment is a function call.
3. Add scratch witnesses:
   - `w544_module_var_scalar_call_init.t27`
   - `w544_module_var_struct_call_assign.t27`
4. Optionally add Variant B adversarial witnesses for nested call initializers,
   initializer dependencies on other module consts, and scalar-array return values.
5. Seal witnesses, record Icarus baselines, and run the full validation matrix.

## Acceptance criteria

- New mutable-var witnesses pass with explicit VCD probe checks.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays at 0 cocotb failures
  and 0 seal mismatches.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and `lake build Trinity.IcarusLowerable.Soundness`
  remain green.

---

*Next: Wave Loop 545 cooperation variants will be proposed in the W544 closeout.*
