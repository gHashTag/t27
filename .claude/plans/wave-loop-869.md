# Wave Loop 869 — Plan

| Field | Value |
|-------|-------|
| Wave | 869 |
| Issue | #1686 (expected) |
| Branch | `wave-loop-869` |
| Base | `wave-loop-868` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[557][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,648 elements × 32 bits = 1,140,736 bits (~1.088 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[557][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W868.
2. **Generator** — copy `scripts/gen_w868.py` → `scripts/gen_w869.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w869_bench_module_557x2p6_aos_var_call_write.t27`
   - module header f-string → `w869_bench_module_...`
   - `OUTER = 557`, `MID_IDX = 278`
3. **Spec** — run `python3 scripts/gen_w869.py` to produce
   `specs/scratch/w869_bench_module_557x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w869_bench_module_557x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W868 (329/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W869_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1686`, push branch, open PR.

## Constants for generator

```python
OUTER = 557
TOTAL = OUTER * 2 ** 6          # 35,648
LAST_IDX = OUTER - 1            # 556
MID_IDX = OUTER // 2            # 278
```

## Cooperation variants for W870

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[559][2]^6 Pt` | 559 | `[2]^6` | 35,776 | 1,144,832 | ~1.092 | Continue mechanical `outer += 2` ladder. |
| **B** | `[557][3]^6 Pt` | 557 | `[3]^6` | 53,376 | 1,708,032 | ~1.630 | Grow second inner dimension, stress stride scaling. |
| **C** | `[557][2]^6 Pt` (neg-index writes) | 557 | `[2]^6` | 35,648 | 1,140,736 | ~1.088 | Negative-index / wrap-around writes. |

*φ² + φ⁻² = 3 | TRINITY*
