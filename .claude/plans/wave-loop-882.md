# Wave Loop 882 — Plan

| Field | Value |
|-------|-------|
| Wave | 882 |
| Issue | TBD (next available GitHub issue) |
| Branch | `wave-loop-882` |
| Base | `wave-loop-881` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[583][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,312 elements × 32 bits = 1,193,984 bits (~1.139 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[583][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus V13.0 packed-vector
   improvements, Vericert/Graphiti verified HLS, FPGA Roofline/BRAM bandwidth) and
   confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w881.py` → `scripts/gen_w882.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w882_bench_module_583x2p6_aos_var_call_write.t27`
   - module header f-string → `w882_bench_module_...`
   - `OUTER = 583`, `MID_IDX = 291`
3. **Spec** — run `python3 scripts/gen_w882.py` to produce
   `specs/scratch/w882_bench_module_583x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w882_bench_module_583x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W881 (342/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W882_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #{issue}`, push branch, open PR.

## Constants for generator

```python
OUTER = 583
TOTAL = OUTER * 2 ** 6          # 37,312
LAST_IDX = OUTER - 1            # 582
MID_IDX = OUTER // 2            # 291
```

## Cooperation variants for W883

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[585][2]^6 Pt` | 585 | `[2]^6` | 37,440 | 1,198,080 | ~1.143 | Continue mechanical `outer += 2` ladder. |
| **B** | `[583][3]^6 Pt` | 583 | `[3]^6` | 424,683 | 13,589,856 | ~12.96 | Grow second inner dimension, stress stride scaling. |
| **C** | `[583][2]^6 Pt` with negative-index writes | 583 | `[2]^6` | 37,312 | 1,193,984 | ~1.139 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
