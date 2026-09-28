# iteration 02 — forall preservation: 13,198 tokens stop being silently dropped

2026-09-29, ~01:15–01:50. Target: the silent-discard family (plan item 2).

## The change (bootstrap/src/compiler.rs)

One arm of `parse_invariant_clause` used to see `forall` and let
`restore_bdd_fallback` truncate the block — the measured dominant discard
channel: by that arm alone, 581 whole-block fallbacks across 35 specs,
19,939 of 23,831 amnestied tokens (84%) corpus-wide rode `bdd-block-fallback`.

Replaced discard with **preservation** (wave-697 lineage, third option beside
inventing semantics and discarding): the quantified statement is captured
verbatim into the block's `value`, the block is marked `partial` (the
emitter's NOT CHECKED notice already keys on that), tokens are CONSUMED not
counted as dropped, children stay empty (emitted bytes unchanged — no
committed seal moves), and the text survives in the AST for #2774's lowering.

New `capture_to_next_top_level`: the preserving twin of `skip_to_next_top_level`.

**The bug I shipped first, and its fix:** v1 adjusted paren/bracket/brace
depth BEFORE the boundary check; because `is_top_level_start` includes RBrace,
a struct-literal closing brace inside the predicate (depth 1→0) read as a
boundary, truncated the capture mid-statement, and turned ternary_inference's
parses-with-discard into a HARD parse error. v2 checks the boundary BEFORE
depth adjustment (strict `== 0` on all three depths). Caught by the loop's
own gate before anything was committed — the gate exists for exactly this.

## Measured (t27c suite, full tree + corpus ratchet)

- Discarded top-level tokens corpus-wide: **27,562 → 14,364 (−13,198, −48%)**
  from the single arm.
- Worst spec `ternary_inference.t27`: 1813 → 87.
- Corpus PRIMARY reds: 148 → 148 (no spec fully clean YET — residual channels
  remain; next is the second forall site in `parse_bdd_clauses`, 256 events /
  29 specs). BLOCKED (gated upstream): 292 → 289.
- Gate diff vs baseline: failing-spec set IDENTICAL, zero new reds.
- `no-vacuous-invariant` count UNCHANGED — honest: preservation ≠ lowering;
  the statements are now kept and marked NOT CHECKED instead of vanished, but
  they still check nothing until #2774 decides lowering.

## Ledger re-bless (docs/reports/suite_expectations.json)

`--ratchet --corpus-only --bless-expectations`: **RATCHET: CLEAN** after
re-bless. `discard_tokens` sum re-pinned 23,831 → 14,364 — the win is now
LOCKED (any regression re-fails the ratchet). `max_entries` 152 → 148.

**Anomaly found and resolved (the self-critique the owner asked for):** the
bless didn't just shrink — it reshuffled the population: 52 entries removed,
47 different paths added, old phase split (parse 68 / no-discard 76 /
typecheck 7) → (parse 23 / no-discard 111 / typecheck 9 / vacuous 3 /
verilog-kw 1 / gen-verilog 1). Master's ledger had drifted from master's
suite reality: 52 amnestied specs no longer fail anything, and 47
currently-failing corpus specs were never tracked. Verified against the
untouched baseline before accepting: **every added path was already failing
at baseline** (0 exceptions) — my compiler change introduced none of them.
The bless resynced the ledger to the measured corpus reds (148 entries = 148
measured), which is what the file is for, but the PR must say so plainly:
this commit both locks the forall win AND republishes the honest corpus-red
inventory.

## Self-critique

- The v1 boundary bug reached a green build before the suite caught it; a
  single-spec parse check after editing would have caught it in seconds
  instead of a full suite run. Protocol updated: parse-check the touched
  spec immediately after any parser edit.
- Corpus red count did not move; only volume did. If the loop's goal is
  stated as "fewer red specs," iteration 2 did not achieve it — the honest
  statement is "13,198 fewer silently-dropped tokens, zero new reds, debt
  re-pinned lower."
- The ledger reshuffle means the pre-existing desync is now mixed into this
  PR's diff; reviewer burden is real. Mitigation: this report + the ledger
  diff is machine-checkable (`/tmp/ledger_before.json` preserved path-by-path).
- One suite run per verdict still; determinism of discard accounting across
  two back-to-back runs is asserted by ratchet CLEAN on re-run, not by a
  dedicated double-run.
