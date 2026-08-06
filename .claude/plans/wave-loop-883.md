# Wave Loop 883 — Plan

| Field | Value |
|-------|-------|
| Wave | 883 |
| Issue | TBD (next available GitHub issue) |
| Branch | `wave-loop-883` |
| Base | `wave-loop-882` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[585][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,440 elements × 32 bits = 1,198,080 bits (~1.143 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[585][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background and confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w882.py` → `scripts/gen_w883.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w883_bench_module_585x2p6_aos_var_call_write.t27`
   - module header f-string → `w883_bench_module_...`
   - `OUTER = 585`, `MID_IDX = 292`
3. **Spec** — run `python3 scripts/gen_w883.py` to produce
   `specs/scratch/w883_bench_module_585x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w883_bench_module_585x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W882 (343/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W883_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #{issue}`, push branch, open PR.

## Constants for generator

```python
OUTER = 585
TOTAL = OUTER * 2 ** 6          # 37,440
LAST_IDX = OUTER - 1            # 584
MID_IDX = OUTER // 2            # 292
```

## Cooperation variants for W884

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[587][2]^6 Pt` | 587 | `[2]^6` | 37,568 | 1,202,176 | ~1.147 | Continue mechanical `outer += 2` ladder. |
| **B** | `[585][3]^6 Pt` | 585 | `[3]^6` | 426,465 | 13,646,880 | ~13.01 | Grow second inner dimension, stress stride scaling. |
| **C** | `[585][2]^6 Pt` with negative-index writes | 585 | `[2]^6` | 37,440 | 1,198,080 | ~1.143 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
