# Iteration 11 — remaining not-lowerable rows (and/when/assert/given): 3 rewrites
# (lotus ×2 blocks, sac_critic ×1) + the rest confirmed documented leaves

## Sequencing discovery first

The planned candidate (declare the undeclared test helpers —
approximately_equal ×14 etc.) was DEFERRED with a reason measured from
the ledger: positional_enc and multi_head_attn are pinned red at
parse-no-discard (their #2774 quantifier-tail invariants still
discard), so typecheck-phase fixes in those specs are INVISIBLE in the
corpus count until #2774 lands. Parse-level rows first — they can
actually green specs.

## Decomposition

| spec | tokens | root construct | verdict |
|---|---|---|---|
| lotus | 48 (2 blocks) | block A: `and then policy_state[5] == 42` — a stray `then` after `and` (typo); block B: `record_episode(CycleResult{...})` — literal ellipsis sketch-placeholder | **both rewritten**: assertion moved to its own `then` clause; CycleResult constructed with `undefined` for the four unconstructed structs (corpus idiom, mime/html/xml) + real `outcome`/`total_time_ms` — record_episode only reads `.context.timestamp` (garbage tolerated, the test asserts only the id increment) |
| sac_critic | 49 | multi-line Zig-style clause bodies (`given\n    x = ...` with no `and`), clause-level `const` inside `when`, `[]f32 { ... }` spaced pseudo-literal, void-call binding | **rewritten**: native and-chain, indexed-element array literal `[]f32{target_params[0], ...}`, bare `when f(...)`, `then true` kept verbatim (the original claim was execution-survival) |
| filesystem | 38 | `join(allocator, &[_][]const u8{...})` + `std.mem.eql` — allocator family | **left** (allocator, same class as circular_buffer) |
| async_stream | 213 | `from(items, std.testing.allocator)` + `map(stream, fn(x) ...)` closure + generator API | **left** (allocator + closure + generator; biggest single leave) |
| io | 112 | `map(io, f(x) -> int { x + 1 })` arrow-lambda literals | **left** (closure family) |
| ternary_add | 96 | `assert ... in {-1,0,+1} for all Trit a,b,c` ×2 (finite 27-combo but set-membership), `implies ... for all i8 s` ×3 (infinite) | **left** (#2774; hand-unrolling 27 combos × set-membership duplicates the future lowering and bloats the spec — scale, not kind, separates this from iteration-9's 4-combo writhe unroll) |
| sgd + reader | ~195 | whole-module sketches | **left** (iteration-8 decision stands) |

## Probe-first (held)

/tmp/t27_hyg/iter11_probe.t27 proved three NEW shapes in one file —
unbound void call as `and` continuation (`and rec(CR{...})`), struct
literal with `undefined` fields as a call argument, indexed-element
array literals, bare void `when` + `then true`: nothing discarded,
typecheck 0 errors. lotus and sac_critic were only touched after that.

## Measured

- lotus 48→**0** (nothing discarded), sac_critic 49→**0**; both
  typecheck 0 errors (warnings pre-existing in fn bodies).
- and-row 5/3 → 4/2; when-row 4/2 → 2/1; assert/given rows unchanged
  (their specs are the documented leaves).
- corpus discard sum: 5,792 → **5,695 tokens** (cumulative −79.3%);
  specs with discard 90 → 88.

## Self-critique

- lotus-B's `undefined` context means the emitted check reads
  `undefined.timestamp` — legal, garbage-valued, and irrelevant to the
  asserted id-increment, but it IS a read of unconstructed memory in a
  corpus test. The alternative (constructing real Context/Evaluation/
  Plan/Action literals) would need four more struct definitions read
  and mirrored; the idiom precedent says this is acceptable. Recorded
  so the next loop can revisit if execution phase ever complains.
- ternary_add's two finite-domain asserts were left on a scale
  argument. That argument is honest but soft: if #2774 stalls, a
  follow-up could generate the 27-combo unroll mechanically (a
  generator script, like the wave-loop generators) instead of by
  hand. Not done here — one dialect fix per spec per iteration keeps
  the diffs reviewable.
- The helper-declaration candidate stays parked with its measured
  reason (phase domination), not forgotten: it is first in state.md's
  next-up for whenever #2774 moves.
