# NOW -- The ruler was a binary on disk (2026-08-30)

## The ruler was a binary on disk (Closes #2851, Refs #2767)

- `Seal Coverage` had been red on master for seven runs. Run locally the same
  script said `OK: 1316 seals, 1222 hold` and exited 0.
- The script checks seals against **the built compiler**. Mine was six hours old,
  from before four `gen-c` fixes landed, so it produced the OLD output which
  matched the OLD seals. `cargo build --release` and the same script: **exit 1,
  134 gen-drift**. `_find_t27c` handles a MISSING binary explicitly -- "a missing
  binary is NOT a passing check" -- and a STALE one looks exactly like a healthy
  one.
- **Control before re-sealing**, because re-sealing is how a regression gets
  written into the record as truth: published baseline Zig 214 / rustc 214 /
  cc 163 / iverilog 373 / ALL FOUR 66; after the four fixes Zig 214 / rustc 214 /
  **cc 166** / iverilog 373 / ALL FOUR 66. cc gained three, everything else held
  exactly. The new output is better, not merely different.
- `t27c seal --save` writes ONE seal file, and 547 specs carry TWO under
  different names (#2767). So 62 re-seals took gen-drift 134 -> 61, and **60 of
  the 61 remaining were the twin case**. `tri seals sync-twins` -- built for this,
  and its own --help describes the scenario -- took it 61 -> 2, and an ordering
  mistake of mine accounted for the last 2.
- **Gate green: exit 0, 1222 hold, 94 known-broken -- the same 94.** No new debt
  entered `seal_baseline.txt`, no sealed hash is `none` (sync-twins refused 31
  specs for exactly that reason), and corpus acceptance is unmoved.
- Not fixed, and not mine to fix: `Seal Coverage` is not a required check
  (`required_status_checks.contexts` is `[]`), so seven red runs blocked nothing.
  #2851 stays open on that half.
- ci-gates 261-263.
