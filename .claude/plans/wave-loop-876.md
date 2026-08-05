# Wave Loop 876 — Plan

| Field | Value |
|-------|-------|
| Wave | 876 |
| Issue | #1701 |
| Branch | `wave-loop-876` |
| Base | `wave-loop-875` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[571][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,544 elements × 32 bits = 1,169,408 bits (~1.116 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[571][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W875.
2. **Generator** — copy `scripts/gen_w875.py` → `scripts/gen_w876.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27`
   - module header f-string → `w876_bench_module_...`
   - `OUTER = 571`, `MID_IDX = 285`
3. **Spec** — run `python3 scripts/gen_w876.py` to produce
   `specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w876_bench_module_571x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W875 (336/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W876_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1701`, push branch, open PR.

## Constants for generator

```python
OUTER = 571
TOTAL = OUTER * 2 ** 6          # 36,544
LAST_IDX = OUTER - 1            # 570
MID_IDX = OUTER // 2            # 285
```

## Cooperation variants for W877

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[573][2]^6 Pt` | 573 | `[2]^6` | 36,672 | 1,173,504 | ~1.120 | Continue mechanical `outer += 2` ladder. |
| **B** | `[571][3]^6 Pt` | 571 | `[3]^6` | 54,816 | 1,754,112 | ~1.674 | Grow second inner dimension, stress stride scaling. |
| **C** | `[571][2]^6 Pt` with negative-index writes | 571 | `[2]^6` | 36,544 | 1,169,408 | ~1.116 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
