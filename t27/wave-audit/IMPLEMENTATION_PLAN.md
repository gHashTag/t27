# Implementation Plan — t27/wave-audit

Skill run: `skill/t27/wave-audit.run` (event 01M4341JQS23EYYDNBJR2KYGAD), issue gHashTag/t27#5985.
Tags: audit, issues, plan

## Measurement context

- SHA at measurement: `53dcd80f4c53b6a5c82d804a41ac7d2787f99402` (git rev-parse HEAD, worktree queen-5985)
- Date: 2026-10-04
- Inputs: WEAKNESS_ASSESSMENT.md (W-A…W-G), GITHUB_ISSUES_ANALYSIS.md (159 open issues)

## Milestones (decomposed, ordered by dependency)

### M1 — Correct the audit artifacts (this boundary; DONE this run)

- Scope: `t27/wave-audit/` only — replace placeholder SHAs with the real measurement SHA, correct the date to 2026-10-04, retract the false "fixed specs/*.tri" claims, write the four reports with measured values (command + sha + date each).
- Acceptance: every measured value in `t27/wave-audit/` carries a command, a sha, and a date; CLOSING_COMMENT.md answers the three criteria.
- Status: done in this commit (see IMPLEMENTATION_SUMMARY.md).

### M2 — Restore honest compile measurement (prerequisite for verifying every fix below)

- Scope: machine, not repo. Install `rustc`, `cc`, `iverilog` (corpus refused with exit 2, all 3512 pairs tool-absent), then re-run `t27c corpus --limit 0` and `t27c backlog` to get honest per-backend accept/reject counts and the real defect backlog.
- Acceptance: `t27c corpus` exits 0 with non-constant columns; the backlog's UNRESOLVED count drops to ~0 and DEFECT specs becomes measurable.
- Owner: operator (machine changes are outside this assistant's boundary).

### M3 — Gen-backend correctness defects (upstream issues; verification blocked until M2)

- Order by blast radius:
  1. #5984 gen-zig deletes top-level `_ = call(args);` (silently drops effects)
  2. #5973 + #5974 gen emits plain `%` for signed/float modulo (Zig and C UB) — one fix class, two issues
  3. #5987 gen-rust `assert((cond))` macro parsing bug
  4. #5986 gen-rust named integral array repetition → E0308
  5. #5966 gen-verilog output refused by iverilog/verilator (also visible as the 19 "does not generate Verilog" specs in `t27c backlog`)
- Acceptance per item: repro via the issue's own command; `t27c parse` and `t27c typecheck` on changed `.t27` files; generated artefact compiles under the target toolchain (M2).

### M4 — Parser honesty defects (upstream issues)

- Order:
  1. #5949 match block lowered to nothing while typecheck accepts (typecheck must refuse when lowering drops the block)
  2. #5968 six silent misreads (each gets a refusal, not a guess)
  3. #5978 dotted module name / use path leaves stray top-level expression in 12 specs
- Acceptance: each misread becomes a parse/typecheck error with a precise location; no spec regresses (guard: `t27c backlog` DEFECT count must not rise).

### M5 — t27b subset coverage and ledger hygiene (upstream issues)

- #5988 orphan ledger sections (correctness, small fix first), then #5977 coverage (StructDecl 393 and string literal 288 first-rejection classes are the two largest levers per the issue's own stats).
- Acceptance: t27b ledger matches emitted code; t27b corpus reject count drops measurably.

### M6 — Self-host epic (upstream epics #5980, #5981)

- Only after M2–M4: MVP compiler core in t27, byte-identical to gen-c output, per #5981's own acceptance criteria.

## Explicitly out of scope (boundary)

- Rewriting `.tri` design documents as t27 modules (W-B) — the repo's own docs prescribe visual audit for them; no unrequested adjacent work.
- Any change outside `t27/wave-audit/` — this run's commit touches only that directory.
- Installing toolchains, pushing, sealing, `docs/now/` entries — operator's responsibility.
