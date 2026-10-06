# NOW -- t27b lab on Railway, JIT on arm64 Linux (2026-10-04)

The owner's Mac runs at load 600-850, and a local cargo build of t27b takes
over ten minutes. t27b corpus runs move to a Railway service.

## t27b lab: the corpus on Railway under qemu-user (Closes #6071)

- `cli/t27b/src/jit.rs`: the JIT runs on arm64 Linux too. The region is
  mapped RW, written, flushed (`dc cvau` / `ic ivau` with line sizes from
  CTR_EL0, `dsb ish; isb`) and switched to RX with `mprotect`. macOS keeps
  the same MAP_JIT / `pthread_jit_write_protect_np` /
  `sys_icache_invalidate` sequence, moved into `map_code` unchanged.
- `tests/differential.rs` now runs on arm64 Linux as well as arm64 macOS.
- `t27b corpus --json <path>`: the text summary's totals plus one record per
  file (`file`, `reference`, `t27b`, `tests`, `invariants`, `blockers`,
  `detail`). `corpus` does not run the reference path, so `reference` is
  `skip` there; the lab fills it in.
- `t27b corpus --runner "<cmd> [args]"`: each per-file `t27b test` is
  started through `<cmd>`. A container has no binfmt_misc, so a corpus
  driver under qemu-user cannot exec its own aarch64 binary without it.
- `contrib/railway/t27b-lab/`: a Dockerfile (rust:1-bookworm, the aarch64
  cross gcc, qemu-user, Zig 0.16.0 as in CI) and `lab.py`. On start and
  every 30 minutes if `T27_REF` moved, it builds t27c natively,
  cross-builds t27b for aarch64-unknown-linux-gnu, runs `t27b corpus specs`
  under qemu-user, runs the reference path (`t27c test-report <file>`)
  natively per file, runs `cargo test --release -p t27b` under qemu-user,
  and serves `/latest.json`, `/runs/<sha>.json` and `/runs/<sha>.log`.
- No secrets: the repository is cloned anonymously and the service never
  writes to GitHub. The only variable set on it is `T27_REF`.
- First full run (commit 9d87d1406, 1190 files under `specs/`): reference
  646 pass, 25 fail, 516 blocked, 3 timeout; t27b 47 pass, all 47 among the
  reference passes, 0 mismatch, 0 crash, 0 timeout; `cargo test -p t27b`
  under qemu-user 18 passed, 0 failed. The JIT runs under qemu-user.
- The first deploy failed the reference step with `can't start new thread`:
  the container sees 48 CPUs, has a 24-CPU quota and `pids.max` 1000, and
  each `zig test` starts one thread per visible CPU. Workers are now sized
  from `cpu.max` and `pids.max`.
- Epic: #6063.
