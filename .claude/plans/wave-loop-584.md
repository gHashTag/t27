# Wave Loop 584 Plan — 17-D array-of-struct return call deduplication

## Issue

Closes #1555.

## Context

Wave Loops 566–582 scaled function-local / call-return array-of-struct
packed-vector lowering from 2-D to 16-D. W583 took a deliberate detour to
module scope and fixed an indefinite-width concatenation bug for non-literal
expressions. The recommended next step in the W583 closeout report was to
resume rank scaling with 17-D if CI budget allowed.

## Weak-spot analysis

1. **Time budget at 4 MiBit:** W582 16-D direct simulation took ~4 minutes;
   W584 17-D direct simulation took ~22.5 minutes on the same host. The
   witness file also doubled to ~22 MB / ~1.18 M lines. This approaches the
   practical timeout boundary for a single `./scripts/tri test` gate.
2. **Signed i16 field range shrinks relative addressable space:** at rank 17,
   even `[0][0][1]...` reaches element 32768, beyond the `2*e+1 ≤ 32767`
   limit. Probes need three leading zeros to stay safe, reducing the effective
   coverage of indexed access to the lower half of the address space.
3. **Icarus memory/formatter stress:** 4,194,304-bit packed vectors still pass
   Icarus 12.0 when the local-`expected` workaround is used, but the VPI
   `$display` path emits many probe lines and the simulator runtime scales
   non-linearly.
4. **No new compiler-path coverage:** the same rank-agnostic machinery that
   handled 16-D handles 17-D; this wave is a stress test, not a feature
   extension.

## Scientific / engineering precedents

- IEEE Std 1800-2017 §7.4.1 packed-array minimum = 65,536 bits. W584 tests a
  4,194,304-bit vector — 64× the LRM minimum.
- Icarus maintainer caryr (GitHub #1171, Sep 2024): standard suggests 2^16
  packed-dimension floor; Icarus does not hard-code it but very large vectors
  can exhaust memory or time.
- Icarus `vpi/sys_display.c`: decimal string size grows linearly with vector
  width (`calc_dec_size`). The W573–W582 workaround of binding the wide
  literal to a local variable before `$display` remains effective.
- Electronics StackExchange / EDA Playground reports: some simulators segfault
  on 500 kbit packed vectors, while 50 kbit works; limits are
  implementation-defined.

## Variant choice

Select **Variant A — 17-D array-of-struct return call deduplication**
(`[2]^17 Pt`, 4,194,304 bits, 131,072 elements). This is the recommended
continuation of the rank-scaling sequence and provides a clean data point on
the practical boundary of Icarus runtime for wide packed vectors.

Variant B and C are documented as alternatives for Wave Loop 585.

## Risk assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Direct simulation exceeds CI timeout | High | Use `--fast` tri gate; direct simulation was run once manually to create baseline; note wall-clock in closeout. |
| Signed i16 probes overflow | Medium | Generator enforces `e ≤ 16383`; three leading zeros used. |
| Icarus oom/crash at 4 MiBit | Low | Witness passed manual simulation; local-`expected` workaround used. |
| File size breaks tooling | Low | 22 MB / 1.18 M lines under git; no shell scripts involved. |
| No compiler change needed | Expected | Confirms rank-agnostic paths; record as zero-code-change stress test. |

## Implementation steps

1. Generate deterministic witness `specs/scratch/w584_bench_17d_aos_call_dedup.t27`
   with a Python script mirroring W582 but for rank 17, with safe indexed
   probes (`e ≤ 16383`).
2. Parse, typecheck, and check Icarus lowerability.
3. Run direct `t27c icarus-simulate` once to create the baseline and confirm
   PASS; note the ~22.5 min wall-clock.
4. Run direct `t27c icarus-cocotb` once to confirm cocotb cross-check PASS.
5. Save seal and Icarus baseline.
6. Add integration test `accepts_w584_bench_17d_aos_call_dedup` to
   `bootstrap/tests/icarus_lowerable.rs`.
7. Run `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
   and verify zero new failures and zero seal mismatches.
8. Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W584_2026-07-07.md`.
9. Update `.trinity/current-issue.md` with Wave Loop 585 variants.
10. Update `.trinity/experience.md` with W584 learnings.
11. Save persistent memory `~/.claude/projects/-Users-playra-t27/memory/wave-loop-584.md`
    and `MEMORY.md` index.
12. Commit with `Closes #1555` and create `wave-loop-585` branch.

## Verification criteria

- `cargo test -p t27c --test icarus_lowerable` includes the new test and
  passes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus PASS count >= 73, cocotb PASS count >= 73, 0 seal mismatches, 24
  pre-existing yosys smoke baselines unchanged.
- Direct `t27c icarus-simulate` W584: PASS.
- Direct `t27c icarus-cocotb` W584: PASS.

## Three cooperation variants for Wave Loop 585

### Variant A — 18-D rank scaling
`[2]^18 Pt` (8,388,608 bits, 262,144 elements). The next doubling. Risk:
witness ~44 MB / ~2.4 M lines; direct simulation likely 40+ min, may exceed
practical CI budget.

### Variant B — Non-power-of-two at rank 17
`[3][2]^17 Pt` (6,291,456 bits, 393,216 elements). Tests product-based
width/index arithmetic at the boundary while staying within the same rank
class.

### Variant C — Large module-scope multi-D AoS variable
A module `var` of type `[2][2][2][2][2][2][2]Pt` (7-D, 16,384 elements,
524,288 bits) initialized from a function call and used in multiple bench/test
sites. Combines W583 module-scope learning with W557 call-array CSE while
keeping file size small.
