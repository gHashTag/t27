# NOW -- The serde gate never travelled from gen_struct to gen_enum (2026-09-05)

The first compiler change this loop has shipped. Found by fanning four agents across
the four backend generate-to-accept gaps, then having two independent verifiers try
to refute each finding.

## The defect (Closes #3208)

- `gen_struct` writes the serde derive behind `#[cfg_attr(feature = "serde", ...)]` at compiler.rs:23726; `gen_enum` writes `serde::Serialize, serde::Deserialize` into a plain derive list 43 lines below
- the acceptance recipe compiles `--crate-type lib` with no `--extern serde`, so every spec declaring an enum stopped at `error[E0433]: failed to resolve: use of undeclared crate serde` before any of its own code was read
- the comment above the struct-side gate already names the defect it was written for: "measured as the first error in 23 of 38 sampled specs -- the largest single cause in the Rust column"
- same remedy applied to the enum path, same words in the comment

## Measured, 650 corpus specs, before and after by name (Closes #3208)

- rustc accepts 224 -> **237**; **+13 specs unblocked, 0 regressions**
- specs still failing on `serde` after the change: **0**, down from 84
- **84 was a first-error count; 13 is an unblocking count** -- errors queue, so removing the first cause from 84 specs clears 13 outright and leaves 71 reporting whatever stood behind it
- quoting 84 as the yield would have been wrong, and the two numbers answer different questions

## What the fan-out found in the other three columns (Refs #3208)

- Zig, 118 specs pass `zig build-obj` but fail `zig test --test-no-exec`: 102 are bodies t27c stubs with `@compileError("not yet implemented")` that the spec's own tests then call -- build-obj never analyses the callee, `zig test` does. Upheld by an independent census.
- Verilog, 306 of the 380 iverilog-accepted modules have no data port: the entry point is chosen by exact function NAME, and a spec declaring neither `fn on_comb` nor `fn on_clock` gets all three port sources empty. The contingency table over the 380 has both off-diagonals at **0**. One verifier upheld the mechanism, the other refused the framing of the denominator.
- C, 291 rejections: the largest class is an unresolved name emitted with no declaration against a fixed include set. The class survived; its COUNT did not -- re-drawing the within-file rule moves 113 to 215 or to 18, so only the class is reported here.
- a class and its count are separate claims, and three of the six verdicts refuted a count while leaving the mechanism standing

## Seal (Closes #3208)

- `bootstrap/src/compiler.rs` is under the M5 freeze, so `bootstrap/stage0/FROZEN_HASH` moves in this same commit, per FROZEN.md §5 step 3
- the ceremony's step 2 asks for a `[GOLD-RING]` intent; the practice in this tree is otherwise -- 1 of the last 30 commits touching that file says GOLD-RING, and the rest carry `fix(...)` with the measured delta in the subject, which is what this one does
