# Wave Loop 879 — Plan

| Field | Value |
|-------|-------|
| Wave | 879 |
| Issue | #1708 |
| Branch | `wave-loop-879` |
| Base | `wave-loop-878` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[577][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,928 elements × 32 bits = 1,181,696 bits (~1.128 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[577][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus V13.0 packed-vector
   improvements, Vericert v2.0.0 / PLDI 2024 hyperblock scheduling, Graphiti
   ASPLOS 2026, FPGA Roofline/BRAM bandwidth) and confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w878.py` → `scripts/gen_w879.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w879_bench_module_577x2p6_aos_var_call_write.t27`
   - module header f-string → `w879_bench_module_...`
   - `OUTER = 577`, `MID_IDX = 288`
3. **Spec** — run `python3 scripts/gen_w879.py` to produce
   `specs/scratch/w879_bench_module_577x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w879_bench_module_577x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W878 (339/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W879_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1708`, push branch, open PR.

## Constants for generator

```python
OUTER = 577
TOTAL = OUTER * 2 ** 6          # 36,928
LAST_IDX = OUTER - 1            # 576
MID_IDX = OUTER // 2            # 288
```

## Cooperation variants for W880

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[579][2]^6 Pt` | 579 | `[2]^6` | 37,056 | 1,185,792 | ~1.131 | Continue mechanical `outer += 2` ladder. |
| **B** | `[577][3]^6 Pt` | 577 | `[3]^6` | 55,296 | 1,769,472 | ~1.687 | Grow second inner dimension, stress stride scaling. |
| **C** | `[577][2]^6 Pt` with negative-index writes | 577 | `[2]^6` | 36,928 | 1,181,696 | ~1.128 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
