# Iteration 12 — stopped-mid-clause rows, and the wall gets a name

Date: 2026-09-29 (loop auto-2026-09-29, iter 12). Branch `loop/auto-2026-09-29`, PR #5084.

Goal: the "stopped mid-clause" census rows — specs whose fallback event names the
token the parser choked on (`.` / `true` / `^`), the biggest unexamined mass left
after iteration 11 closed the then-row and and/when rows.

## Decomposition first (the whole blocks, printed before diagnosing)

| Spec | Events | Root construct | Action |
|---|---|---|---|
| `tri/crypto/base64.t27` | 6 | placeholder bodies: `test "name"` then a bare `true` on the next line — the `true` is a statement, not a clause | **REWROTE**: `then true` ×6; quoted names kept — they parse natively (probe `iter12_probe.t27`) |
| `igla/coder/tokenizer.t27` | 17 | (a) stray top-level `}` (wave-loop generator artifact, old line 1384); (b) two invariants with bare-expression bodies | **REWROTE**: `}` deleted with comment; `then encode_keyword("module") == 256 and decode_keyword(256) == "module"` (+ endmodule/257 twin) |
| `port/fpga/vivado/gf16_matmul_top.t27` | 5 | test names contain `^`: `after_tick_2^19`, `at_counter_2^19_minus_1` — `^` is not an identifier character | **RENAMED** to `2_pow_19` / `2_pow_19_minus_1` |
| `port/hdl/vcd_conformance_compare.t27` | 6 | stopped on `.` → `record_pass(r)` receivers are `fn record_pass(r: *CompareResult)` | WALL — pointer-receiver API |
| `ml/recurrent/gla.t27` | — | `gla_state_update(hs: *GlaHeadState, …)`, `gla_state_read(hs: *GlaHeadState, …)` | WALL — pointer receiver |
| `tools/tri_to_t27_converter.t27` | — | string escapes `\"1.0\"` (lines 403/408) | **LEXER GAP** — the only language-side fault left in the census |
| `ml/packed_vsa.t27` | — | `packedCosineSimilarity(&vec, &vec)` | WALL — pointer args |
| `port/peripheral/spi.t27` | — | method-call object style `spi.transfer(0xAA)` | WALL — no method-call spelling |
| `core/channel.t27` | — | `Maybe(T)::None` | WALL — generics `::` |

## The wall, named

After twelve iterations the remaining parse-level census is no longer "dialect
drift" — every large family has been either rewritten or measured to a named
construct with no t27 spelling:

1. **Pointer / allocator receivers** (`*T` params, `&x` call sites,
   `std.mem.Allocator`) — tests cannot CALL these APIs natively; each spec needs
   an API-redesign decision (value-returning wrappers or a `^mut` native
   spelling), not a text rewrite. Specs: circular_buffer, kd_tree, segment_tree,
   async_stream, filesystem, vcd_conformance_compare, gla, packed_vsa,
   io (mixed with lambdas).
2. **Method-call object style** (`obj.method(args)`) — spi; possibly a parser
   feature (desugar to `method(obj, args)`) rather than a spec rewrite.
3. **Generics with `::`** (`Maybe(T)::None`) — channel.
4. **#2774 quantifier tails** (`for i in …:` invariant bodies, English
   `implies` prose) — the parse-no-discard pins on positional_enc / mha /
   ternary_add; dominates any typecheck-level fix (iteration-11 finding).
5. **One lexer gap**: string escapes `\"` (tri_to_t27_converter). The only item
   here that is a *language fault* rather than a dialect or design choice —
   candidate for a one-line lexer fix or a filed issue.

Everything else in the 5,667 remaining tokens sits behind these five doors.

## Measured

- Tokens: **5,695 → 5,667** (−79.4% cumulative from 27,562), 85 specs.
- Corpus: **131 → 129** — base64 and gf16_matmul_top fully green; tokenizer's
  ledger entry moved parse-no-discard → **no-vacuous-invariant** (see below).
- Typecheck after edits: 0 errors on all three touched specs.
- 2 runtime checks resurrected in tokenizer (keyword roundtrip ×2 — the
  invariants now carry `encode_keyword == 256 and decode_keyword == "module"`
  pairs instead of bare expressions).

## The tokenizer phase-move (iteration-7 discipline, applied)

Gate returned `UNEXPECTED FAILURES: 1 — specs/igla/coder/tokenizer.t27
[no-vacuous-invariant]`. Per the iteration-7 rule this was investigated BEFORE
blessing:

- The failing phase is the phase AFTER the one my edit cleared. tokenizer was
  pinned at parse-no-discard; the rewrite (stray `}` deletion + two native
  invariants) cleared parse, exposing the next phase.
- That phase fails on the **pre-existing** `invariant tokenizer_wNNN_depth_NNN:
  true` wave-loop placeholder family — vacuous **by design** (depth markers),
  present in the file for hundreds of waves, previously masked by the parse red.
- My diff cannot be the cause: it *added* non-vacuous invariants and deleted a
  stray brace; it did not touch any `wNNN` marker.

Verdict: honest phase-move, same mechanic as iteration 9's DISCARD-WORSENED
rows. Blessed with tokenizer pinned at no-vacuous-invariant — the honest state
until the wave-loop marker family gets a real predicate or an exemption rule.

## New probe-proven native shapes (cumulative list grows)

- `test "quoted name"` parses natively (no identifier restriction).
- `then true` as the ONLY clause of a test is native and lowerable.
- Invariant bodies: bare expressions are unmodelled; `then … and …` chains at
  column 0 are native.

## Self-critique

- The `^`-in-name fix is cosmetic (renames) but it was measured, not guessed:
  the census event named the exact character, and the probe confirmed quoted
  names as the alternative had the spec wanted to keep `2^19` verbatim.
- I did NOT attempt the lexer gap fix in this iteration: a lexer change moves
  FROZEN_HASH and touches every spec's seal; that is a compiler-commit
  decision, not a spec-dialect iteration. Filed as the next candidate with the
  measured blast radius (one spec, 2 sites).
- The wall specs were printed and LEFT, not probed-then-rewritten: probing a
  pointer receiver would just confirm the wall. The decomposition table above
  is the deliverable for those rows.

Phase complete: iteration 12 (stopped-mid-clause rows)
→ next: the wall (per-door decision), lexer gap, or #2774
