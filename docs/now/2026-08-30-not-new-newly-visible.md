# NOW -- Not new, newly visible (2026-08-30)

## The corpus ratchet was red on master, and it was right (Refs #2852)

- UNEXPECTED FAILURES: 3 -- `fpga/bridge.t27`, `numeric/gf16.t27`, `pins/emitter_xdc.t27`, all `[typecheck]`
- bisected across six binaries: none of MY branches fail; only builds from master do, and the first is #2854
- #2854 made `t27c typecheck` carry its printed verdict in the exit code. The previous binary printed `Typecheck FAILED (6 errors)` and exited 0
- so the three were always failing and the ratchet could not see it: not a regression, a measurement that started working
- the correct response to that is a re-bless with a stated reason, not a revert
- running the ratchet locally needed a second fix first: `reg_decl` computes `width - 1` guarded by `width == 1`, and a zero width panics the whole suite in a debug build
- in release the same line emits `reg [18446744073709551615:0]`, so the guard was wrong in both profiles and loud in only one
- `--bless-expectations` writes `reason: unclassified` and does NOT raise `max_entries`, so a blessed ledger still fails on the cap; both were set by hand
