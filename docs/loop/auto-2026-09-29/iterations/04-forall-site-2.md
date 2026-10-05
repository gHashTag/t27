# iterations/04-forall-site-2.md — the second quantified-invariant site + tri loop state

Iteration 4 had two targets (state.md, 2026-09-29 ~02:20); both landed.

## 1. `tri loop state` (commit 88e3d4e05, pushed)

Reads and validates the state file every firing reads FIRST. Exit 0/1/2:
0 = parsed and tree matches, 1 = state-vs-tree divergence (wrong checkout),
2 = could-not-run. Five tests, all green (`cargo test -p tri` full run
started fresh after the `trim()` fix — loopclaim 5/5 targeted already).
Live self-check on the loop's own worktree caught a `.gitattributes`
side-write recurrence and the unset upstream — exactly the failure class
it was written for.

## 2. forall site 2 — `parse_bdd_clauses`, the LED shape (this commit)

Census before (iterations/02 + this loop's re-measure): the largest
remaining whole-block-fallback row was **256 events across 29 specs** —
BARE `invariant NAME` whose body LEADS with its binder:

```
invariant sum_bounds:
    forall acc : i32, a : i8
    ...
```

The clause walker hit `forall`, did not recognize it, and the whole
remaining body was one fallback event — every token after it counted as
silently discarded.

**Fix** — same preservation as site 1 (a976e540c), reusing the same
walker `capture_to_next_top_level`: when a `forall` arrives at the head
of a not-yet-lowered block (`block.children.len() == start_children`),
capture verbatim into `block.value`, mark `extra_field = "partial"`
(emitter prints NOT CHECKED), consume the tokens (they stop counting as
dropped), children stay empty so emitted bytes and committed seals do
not move, and #2774 keeps owning what forall MEANS at lowering.

Only the LED case: a forall arriving after lowered clauses (mid-block)
keeps its existing handling, whose trailing clauses still lower today.

### Measured effect

| metric | before | after |
|---|---|---|
| discarded tokens (corpus sum) | 14,364 | **8,559** (−5,805) |
| cumulative vs baseline 27,562 | −48% | **−69%** |
| forall fallback row | 256 events / 29 specs | **gone from census** |
| ternary_mac.t27 fallback events | — | 0 |
| corpus reds (PRIMARY) | 147 | 147 (preservation = volume, not count) |
| gate diff vs baseline | — | rc=0, no new reds |

Ledger re-blessed after: entries 147 / cap 147, RATCHET CLEAN,
`added entries not failing at baseline: []` — nothing this loop added.

FROZEN_HASH: 5bd6c20baf5e4e79 → d925e634aa4cfbca (`tri reseal write`;
build.rs contract — compiler's own seal, not a corpus reseal).

### What remains (next loop's map, measured post-fix)

`stopped mid-clause / for` — 49 events / 12 specs — is the new largest
row; then `clause not lowerable` given/then/assert (19/15/10). All are
smaller and more varied than the forall family: no single 256-event row
remains.

## Premises falsified by measurement (plan items 6/7 — recorded, not "fixed")

- **Item 6** ("`t27c gen --out/` silently swallows bad state"): the
  `--out/` directory no longer exists anywhere; `t27c` rejects unknown
  flags with rc=2. The premise described a state that has since been
  fixed. My own first test of it was `... | head -4; echo rc=$?` —
  rc=0 was head's, not t27c's (W846 hazard); unpiped, the true rc=2.
- **Item 7** ("19 `.tri` files parse silently to garbage"): all 19 fail
  LOUDLY at parse today. The silent-garbage measurement in
  specs/mcp/server_registry.t27's header (file now only in git history,
  f4d3f8338) describes a changed state.

Both are recorded here and in seal-drift-2026-09-29.md's spirit: a
recommendation built on an unmeasured premise is a recommendation about
an imaginary repo. Items 6/7 get no fake fixes.

## Self-critique

- Site 2's guard is `children.len() == start_children` — the LED shape.
  The mid-block forall shape (lowered clauses, then forall, then more
  clauses) was NOT separately measured after the fix; the claim that its
  tail "still lowers today" is inherited from site-1 analysis, not
  re-proven on this pass. The 8,559 remaining discarded tokens include
  whatever that shape drops.
- The −69% is tokens, not specs: which specs still discard heavily is
  visible in the ledger's per-entry `discard_tokens`, not summarized
  here.
- `tri loop state`'s branch check reads `git rev-parse --abbrev-ref HEAD`
  — on a detached HEAD this prints `HEAD`, which fails the check loudly.
  Correct behavior, but untested (no test covers detached HEAD).
