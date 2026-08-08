# Wave Loop 549 Plan — IGLA CODER + IGLA RACE, and getting onto real silicon

**Date:** 2026-08-09
**Predecessor:** issue [#1951](https://github.com/gHashTag/t27/issues/1951) — "Wave Loop 548 — positioning audit … three W549 variants" (2026-08-08)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

> **Numbering note.** Two wave-loop counters run in this repo and they have
> collided at 549. The **codegen/Icarus track** already used W549–W550 in
> July (`FPGA_LOOP_CLOSEOUT_W549_2026-07-16.md`, `.claude/plans/wave-loop-549.md`,
> `.trinity/ring-549.md`) for 3-D primitive array returns. This document
> belongs to the **issue track**, which reached W548 as issue #1951 on
> 2026-08-08 and is continued here. The two are unrelated; the date suffix on
> the July files is the reliable discriminator. Merging the counters is
> proposed as W550 Variant C.

---

## 0. Method

This wave follows the IGLA loop's own question from
[`docs/nona-03-manifest/IGLA_IMPROVEMENT_LOOP.md`](../nona-03-manifest/IGLA_IMPROVEMENT_LOOP.md):

> *What is the smallest fix that removes the biggest lie?*

A "lie" is a place where the repository claims an invariant the code does not
enforce. Every finding below was **measured on this host on 2026-08-09**, and
each is paired with the command that reproduces it. Nothing is carried forward
from an earlier report without re-measuring.

W548 chose its variants against a stale premise — its Variant A ("58 of 101
conformance files are empty") was invalidated by the later commit
`e5b171e7 fix(suite): the conformance corpus was never hollow -- the validator
was blind`. That is the failure mode this plan is written to avoid: measure
first, then choose.

---

## 1. Weak points found (measured)

| # | Weak point | Evidence | Severity |
|---|-----------|----------|----------|
| **W1** | `cargo build --release` **fails on stable Rust**. `bootstrap/Cargo.toml` pulled `rusqlite 0.40` → `libsqlite3-sys 0.38.1`, whose build script uses the nightly-only `cfg_select!` macro. `rustc 1.94.1` → `error[E0658]`. | build log | **blocker** — a clean checkout does not build |
| **W2** | `rusqlite` has **zero references** in `bootstrap/src/` and `bootstrap/tests/`. It is a dead dependency that also forced a bundled SQLite C compile. | `grep -rl rusqlite bootstrap/src bootstrap/tests` → 0 | high |
| **W3** | The documented binary path `./bootstrap/target/release/t27c` **does not exist**. `bootstrap/` is a workspace member, so the artifact lands at the workspace root `./target/release/t27c`. 67 occurrences repo-wide, including `SOUL.md`, `CANON.md`, `docs/T27-CONSTITUTION.md` and `CLAUDE.md` itself. | `ls bootstrap/target/release/` | **blocker for agents** — the onboarding command in the project's own instructions fails |
| **W4** | `CANON.md` §8 pointed the full sweep at `bash tests/run_all.sh`, which **does not exist** and contradicts both L7 UNITY and `CLAUDE.md`'s "no shell test harness under `tests/`". | `ls tests/run_all.sh` | medium |
| **W5** | **IGLA spec vacuity.** `2160 / 3788` (**57.0 %**) of `test`/`bench` blocks under `specs/igla/**` contain nothing but `assert true`; `1917 / 1931` (**99.3 %**) of invariants are the literal tautology `true`. IGLA is 2160 of the 2164 vacuous tests in the entire `specs/` tree (1063 specs). Every wave loop appends the same 2 tests + 1 invariant to every IGLA spec. | vacuity scan | high — L4 TESTABILITY is satisfied in letter, void in spirit |
| **W6** | `t27c fpga-flash` **did not exist**, yet `TASK.md` line 90 marked it "Done" and `QMTECH_A100T_SMOKE.md` step 1 instructed operators to run it. | `t27c fpga-flash` → "unrecognized subcommand" | **blocker for the FPGA goal** |
| **W7** | **The ternary MAC demo cannot demonstrate that the MAC works.** Ring-oscillator clock (unconstrainable), LEDs driven from `acc_out[0..1]` at ~10⁸ Hz (invisible), `w_code` tied to `+1` and `acc_in` tied to `0` (accumulate path and minus/zero decode never exercised, free to be constant-folded). | reading `ternary_mac_demo_top.v` against `ternary_mac_synth.v` | **critical** — flashing it proves nothing |
| **W8** | `COMPETITORS.md` names competitors for the **silicon/format** line and **none** for IGLA CODER or IGLA RACE — the two active model tracks. | `grep -i "verilogeval\|rtlcoder\|finn\|hls4ml" COMPETITORS.md` → no hits | medium |
| **W9** | `t27c fpga-build --device` defaults to `xc7a100tcsg324-1` — the **Arty A7** package — while `HARDWARE_SSOT.md` explicitly forbids mixing `csg324` into Wukong flows. | `main.rs` default_value | medium (deferred, see W550-A) |

---

## 2. Competitive landscape (see [`COMPETITORS.md`](../../COMPETITORS.md) §4)

Star counts read from the GitHub API 2026-08-09.

**IGLA CODER competes in LLM-for-RTL**, a field with an established benchmark
culture it has not entered: [VerilogEval](https://github.com/NVlabs/verilog-eval)
(458★, NVIDIA — the measuring stick),
[RTL-Coder](https://github.com/hkust-zhiyao/RTL-Coder) (317★, open weights with
published results), plus VeriGen / ChipNeMo / BetterV / CodeV / OriGen.
**IGLA CODER has no published score on any of them.** Its only defensible
differentiator is generating from a *typed, sealed specification* with
mechanically checkable conformance — which W5 currently undermines.

**IGLA RACE competes in two fields at once.** For training efficiency:
[modded-nanogpt](https://github.com/KellerJordan/modded-nanogpt) (5,648★) and
[nanoGPT](https://github.com/karpathy/nanoGPT) (61,983★) — but IGLA RACE's
BPB = 2.21 is measured on `tiny_shakespeare`, so the numbers are **not
comparable in either direction**. For low-bit FPGA inference the incumbents are
[FINN](https://github.com/Xilinx/finn) (1,038★) + [Brevitas](https://github.com/Xilinx/brevitas)
(1,562★), [hls4ml](https://github.com/fastmachinelearning/hls4ml) (2,092★),
[T-MAC](https://github.com/microsoft/T-MAC) (981★) and
[BitNet](https://github.com/microsoft/BitNet) (39,838★). FINN and hls4ml
produce working FPGA accelerators today; IGLA RACE has one MAC cell that has
never run on a board.

---

## 3. Decomposition

Ordered by leverage: unblock the build, then unblock the hardware path, then
tell the truth about both.

### Track A — Unblock the build *(W1, W2, W3, W4)*

- **A1** Remove the dead `rusqlite` dependency from `bootstrap/Cargo.toml`.
- **A2** Rebuild and confirm `cargo build --release -p t27c` is green on stable.
- **A3** Correct the binary path in the normative docs (`CLAUDE.md`, `SOUL.md`,
  `CANON.md`, `docs/T27-CONSTITUTION.md`, `tests/OWNERS.md`). Leave the
  historical `.claude/plans/wave-loop-*.md` journals alone — they are records
  of past runs, not instructions.
- **A4** Fix `CANON.md` §8 to point at the Rust runner instead of a nonexistent
  shell script.

### Track B — Make the FPGA path real *(W6, W7)*

- **B1** Implement `t27c fpga-flash` with board profiles taken from
  `fpga/HARDWARE_SSOT.md`, pre-flight checks (bitstream present and non-empty,
  loader on `PATH`, programmer actually attached), `--mode sram|flash`, and a
  `--dry-run` that is fully exercisable with no hardware.
- **B2** Write `ternary_mac_demo_top_v2.v`: `STARTUPE2`/`CFGMCLK` clock,
  24-bit prescaler for a visible step rate, weight sequence `{+1, 0, −1, 0}`
  covering both zero encodings, and `acc_out → acc_in` feedback so the
  accumulator actually accumulates.
- **B3** Write a self-checking testbench that proves accumulation, proves every
  weight encoding is applied, and proves the LEDs are not stuck.
- **B4** Constrain the design (`ternary_mac_demo_top_v2.xdc`) — a real clock
  primitive means a real timing constraint, unlike the ring oscillator.
- **B5** Write the staged launch plan with per-gate pass criteria, and label
  what is blocked on toolchain versus blocked on hardware.

### Track C — Correct the record *(W5, W8, and the docs behind W6)*

- **C1** Add `COMPETITORS.md` §4 naming the real IGLA CODER and IGLA RACE
  fields, with five new "we do not claim" entries.
- **C2** Publish the vacuity measurement as claim 10 in that section, rather
  than leaving inflated test counts to speak for themselves.
- **C3** Correct `QMTECH_A100T_SMOKE.md`: wrong board (100T vs 200T), a
  `fpga-flash` invocation that never existed, and a UART loopback that cannot
  run on a host with no serial node.

### Deferred, with reason

- **W9** (`fpga-build` device default) — a one-line change, but it alters the
  default output of a build command; it belongs in a wave that can rebuild and
  compare bitstreams. Carried to W550 Variant A.
- **Vacuity gate in CI** — the measurement (C2) must land and be agreed before
  a gate can be set at a threshold anyone will accept. Carried to W550
  Variant B.

---

## 4. Validation contract

| Check | Required outcome |
|-------|------------------|
| `cargo build --release -p t27c` | green on stable rustc 1.94.1 |
| `t27c fpga-flash --dry-run` | passes pre-flight, reports `BLOCKED` (no cable), exit 0 |
| `t27c fpga-flash` with no cable | refuses with an actionable message, exit 1 |
| `iverilog` + `vvp` on the v2 demo | all self-checks pass |
| `yosys synth_xilinx -abc9 -nocarry -arch xc7` on v2 | clean, resources reported |
| `t27c suite --repo-root .` | no regression against the pre-wave baseline |

---

*φ² + φ⁻² = 3 | TRINITY*
