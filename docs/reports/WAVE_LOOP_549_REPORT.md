# Wave Loop 549 Report — IGLA CODER + IGLA RACE, and the road to silicon

**Date:** 2026-08-09
**Plan:** [`WAVE_LOOP_549_PLAN.md`](WAVE_LOOP_549_PLAN.md)
**Next-wave variants:** [`WAVE_LOOP_549_COOPERATION.md`](WAVE_LOOP_549_COOPERATION.md)
**Launch plan:** [`docs/fpga/IGLA_FPGA_LAUNCH_PLAN.md`](../fpga/IGLA_FPGA_LAUNCH_PLAN.md)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

The wave set out to plan IGLA's route onto real FPGA hardware. It found that
the route was blocked in three places nobody had checked, and that the demo
waiting at the end of it could not have proved anything even if it had been
flashed.

1. **The repository did not build on stable Rust.** A dead `rusqlite`
   dependency dragged in `libsqlite3-sys 0.38.1`, whose build script uses the
   nightly-only `cfg_select!` macro. Removed — build is green, and 1m58s
   faster for not compiling a bundled SQLite.
2. **The build command in the project's own agent instructions was wrong.**
   `./bootstrap/target/release/t27c` does not exist; `bootstrap/` is a
   workspace member, so the binary lands at the workspace root. 67 occurrences
   repo-wide, including the constitution.
3. **`t27c fpga-flash` never existed**, despite `TASK.md` marking it "Done"
   and the operator smoke-test doc instructing people to run it. Implemented.
4. **The ternary MAC demo was unfalsifiable.** Ring-oscillator clock, LEDs
   driven at ~10⁸ Hz, and a datapath wired so that neither accumulation nor
   the minus/zero weight decode was ever exercised. Replaced with a version
   that is observable and self-checking.
5. **57 % of IGLA's tests assert nothing.** Measured: 2,160 of 3,788
   `test`/`bench` blocks under `specs/igla/**` contain only `assert true`, and
   1,917 of 1,931 invariants are the literal tautology `true`.

---

## 1. Weak points, measured

Full table in the [plan](WAVE_LOOP_549_PLAN.md) §1. Reproduction commands are
given there for each. Summary of disposition:

| # | Weak point | Disposition |
|---|-----------|-------------|
| W1 | `cargo build --release` fails on stable rustc 1.94.1 (`E0658`, `cfg_select!`) | **fixed** |
| W2 | `rusqlite` dead dependency, 0 references in `bootstrap/` | **fixed** |
| W3 | `./bootstrap/target/release/t27c` does not exist (67 refs) | **fixed** in normative docs |
| W4 | `CANON.md` §8 pointed at nonexistent `tests/run_all.sh` | **fixed** |
| W5 | IGLA spec vacuity: 57.0 % of tests, 99.3 % of invariants | **measured and published**; gate deferred to W550-B |
| W6 | `t27c fpga-flash` documented but absent | **fixed** (implemented) |
| W7 | Ternary MAC demo proves nothing on silicon | **fixed** (v2 written, simulated, synthesized) |
| W8 | No competitors named for IGLA CODER / IGLA RACE | **fixed** (`COMPETITORS.md` §4) |
| W9 | `fpga-build --device` defaults to the Arty A7 package | **deferred** to W550-A, with reason |

### On W5, stated plainly

Every wave loop has been appending the same two `assert true` tests and one
`invariant …: true` to every IGLA spec. The counts are uniform to the file —
exactly 80 vacuous tests and 71 vacuous invariants each — which is the
signature of a mechanical appender, not of engineering. IGLA accounts for
**2,160 of the 2,164** vacuous tests in the whole 1,063-spec tree.

The consequence is that a headline like "340 tests in `ternary_mac.t27`"
overstates real coverage by roughly a factor of two, and the invariant count
is very nearly pure noise. This is now recorded as claim 10 in
[`COMPETITORS.md`](../../COMPETITORS.md) §4.3 rather than left to imply
coverage we do not have.

### On W7, in detail

`ternary_mac_demo_top.v` had three independent defects, each sufficient on its
own to make a successful flash indistinguishable from a broken one:

- **Unconstrainable clock** — a 20-stage `LUT1` ring oscillator closed through
  the fabric with `ALLOW_COMBINATORIAL_LOOPS TRUE`. No Fmax, no timing claim.
- **Invisible output** — `led_r23 = ~acc_out[0]`, `led_t23 = ~acc_out[1]`,
  toggling at `f_osc/2` and `f_osc/4`. Both LEDs sit at ~50 % brightness; the
  eye cannot separate "working" from "dead".
- **Dead datapath** — `w_code` tied to `2'b01`, `acc_in` tied to `0`. The
  minus-weight and zero-decode branches never activate and the accumulator
  never accumulates; synthesis may constant-fold both away.

---

## 2. Delivered

### Build and documentation

- `bootstrap/Cargo.toml` — removed the dead `rusqlite` dependency.
- `CLAUDE.md`, `SOUL.md`, `CANON.md`, `docs/T27-CONSTITUTION.md`,
  `tests/OWNERS.md` — corrected the binary path to `./target/release/t27c` and
  the build command to `cargo build --release -p t27c`. Historical
  `.claude/plans/wave-loop-*.md` journals were deliberately left untouched:
  they are records of past runs, not instructions.
- `CANON.md` §8 — cheat sheet now runs from the repo root and points the full
  sweep at the Rust runner, consistent with L7 UNITY.

### New CLI: `t27c fpga-flash`

Board profiles are taken from `fpga/HARDWARE_SSOT.md` (`wukong-a200t` default,
`wukong-a100t` legacy, `arty-a7`), each carrying part string, expected JTAG
IDCODE, openFPGALoader cable profile, and a canonical bitstream. Pre-flight
checks: bitstream exists and is non-empty, loader is on `PATH`, a programmer
is actually attached. `--mode sram|flash`, and `--dry-run` performs every check
except programming — so the whole command is exercisable with no hardware.

### New RTL: `ternary_mac_demo_top_v2`

- `fpga/verilog/ternary_mac_demo_top_v2.v` — `STARTUPE2`/`CFGMCLK` clock,
  24-bit prescaler (≈3.9 steps/s), weight sequence `{+1, 0, −1, 0}` covering
  both zero encodings, and `acc_out → acc_in` feedback so the accumulator
  genuinely accumulates.
- `fpga/verilog/tb_ternary_mac_demo_v2.v` — self-checking, with a `STARTUPE2`
  behavioural stub for Icarus. Asserts the accumulator walk, that every weight
  encoding is actually applied, that the sign LED stays dark, and that the
  activity LED is not stuck.
- `fpga/verilog/ternary_mac_demo_top_v2.xdc` — a real clock constraint on a
  real clock net; none of v1's `ALLOW_COMBINATORIAL_LOOPS` /
  `CLOCK_DEDICATED_ROUTE FALSE` escapes are needed.

### New documentation

- `docs/fpga/IGLA_FPGA_LAUNCH_PLAN.md` — gates G0–G4 with per-gate commands
  and pass criteria, and an explicit split between "blocked on toolchain" and
  "blocked on hardware".
- `COMPETITORS.md` §4 — names the LLM-for-RTL and low-bit-FPGA fields IGLA
  actually competes in, with five new "we do not claim" entries.
- `docs/fpga/QMTECH_A100T_SMOKE.md` — correction banner: wrong board (100T vs
  the 200T actually connected), a `fpga-flash` invocation that never existed,
  and a UART loopback that cannot run on a host with no serial node.

---

## 3. Validation

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` (stable rustc 1.94.1) | **green**, 1m58s — was a hard `E0658` failure before |
| `t27c fpga-flash --dry-run` | pre-flight passes, correctly reports `BLOCKED` (no cable), exit 0 |
| `t27c fpga-flash` with no cable | refuses with an actionable message, **exit 1** |
| `t27c fpga-flash --board nope` | rejected, lists valid boards |
| `t27c fpga-flash --mode zap` | rejected |
| `t27c fpga-flash --bitstream <missing>` | rejected with the underlying OS error |
| `iverilog -g2005` + `vvp` on v2 demo | **12 / 12 PASS** |
| `yosys synth_xilinx -abc9 -nocarry -arch xc7` on v2 | **clean** |
| v2 resources | 113 LUT (3/21/19/24/19/27 across LUT1–LUT6), 60 FF (32 FDCE + 28 FDRE), 1 STARTUPE2, 1 BUFG, 2 OBUF; 190 cells |
| `t27c suite --repo-root .` | **still running at report time** — see §5 |

Reproduce the RTL result:

```bash
cd fpga/verilog && iverilog -g2005 -o /tmp/tb_v2.vvp tb_ternary_mac_demo_v2.v ternary_mac_demo_top_v2.v ternary_mac_synth.v && vvp /tmp/tb_v2.vvp
```

---

## 4. What this wave did *not* achieve

Stated plainly, because the point of the exercise is to stop overclaiming:

1. **Nothing ran on hardware.** `openFPGALoader --scan-usb` reports "No USB
   devices found" on this host. Gates G2–G4 are untouched.
2. **v2 has no bitstream.** `nextpnr-xilinx` is not installed locally, so
   place-and-route (gate G1) did not run. The design is simulated and
   synthesized, not routed.
3. **The vacuity finding is measured but not gated.** Nothing yet prevents the
   next wave from appending another 34 `assert true` blocks.
4. **No IGLA CODER benchmark number exists.** The competitor section names the
   benchmarks; it does not report a score, because there is none.
5. **W9 is open.** `fpga-build --device` still defaults to the wrong package.

---

## 5. Note on the suite run

`t27c suite --repo-root .` was started against the post-fix tree and had not
completed when this report was written (≈16 min elapsed, CPU accumulating,
spawning per-spec child processes across 1,063 specs × 3 backends). It is
reported here as **incomplete, not as passing**. The compiler changes in this
wave are additive — one new `clap` subcommand and one new handler function,
with no edits to `compiler.rs` or any code path the suite exercises — so a
regression would be surprising; but "surprising" is not "verified", and the
next wave should confirm the baseline before building on it.

---

## 6. Method note for the next agent

Two process lessons from this wave, recorded in `.trinity/experience.md` and
the wave-loop skill:

1. **Re-measure the previous wave's premise before adopting its variant.**
   W548's recommended Variant A rested on "58 of 101 conformance files are
   empty", which the later commit `e5b171e7` had already invalidated
   ("the conformance corpus was never hollow — the validator was blind").
   Choosing it would have burned a wave on a solved problem.
2. **Verify the shell's working directory before concluding a file is
   missing.** Midway through this wave a persisted `cd bootstrap` made
   `bootstrap/Cargo.toml` appear absent and produced a confident, wrong
   conclusion that the compiler source had been deleted from master. `pwd`,
   or an absolute path, would have caught it immediately. Prefer absolute
   paths in audit commands.

---

*φ² + φ⁻² = 3 | TRINITY*
