# Iteration 10 — the not-lowerable then-row: 3 rewrites + 6 documented leaves
# (15 events / 9 specs → measured below)

## Decomposition first (the state.md instruction, again)

The census said 15 events / 9 specs filed under "clause not lowerable
then". Printing whole blocks and grepping declarations split them:

| spec | tokens | root construct | verdict |
|---|---|---|---|
| attention_mechanism | 729 | three foreign habits per block: repeated `given` heads (native chains with `and`), python `var output = [0.0] * 12` (native `[0.0; 12]`), `then result == void` on a void fn (the bare `when f(...)` call IS the execution) | **rewritten** — 4 test blocks, biggest single mass of the whole family |
| positional_enc | 109 | `then not all_equal(a, b)` — **undeclared helper** (grep: the call site is the module's only occurrence) behind a `not` prefix | **rewritten** as element witnesses: position 1 rotates pair 0 by 1 rad, so elements 0/1 must differ from position 0's `[1.0, 0.0]` |
| multi_head_attn | 70 | three poisons: `sum(result)` nested inside `approximately_equal(...)` (two undeclared helpers), `sqrt(head_dim as gf16::GF16)` (as-cast), `approximately_equal(result, expected, 1e-6)` (undeclared) | **rewritten**: two-sided sum band (keeps 1e-4), `sqrt(64.0)` (= head_dim's value), `result > expected - 1e-6 and result < expected + 1e-6` (keeps 1e-6) |
| circular_buffer / kd_tree / segment_tree | ~197 | module API is Zig-allocator-shaped: `init(allocator: std.mem.Allocator, ...)`, `write(buf: *CircularBuffer, ...)` — no allocator VALUE and no `&x` call spelling exists in t27 clauses; rewriting tests would mean redesigning the modules' fn signatures | **left documented** (allocator-family, same class as iteration-8's reader/sgd sketches) |
| sdk | 56 | clause-level `const a = ...;` + C-style `for` with block body in then | **left** (#2774 quantifier family) |
| sigmoid_activation | 22 | `std.meta.fields` comptime reflection asserts | **left** (same class as config's @FieldType ×5) |

Two rewrites RESURRECT checks that never emitted: multi_head_attn's
softmax sum-to-one and phi-scale formula now emit real two-sided checks
(the formula one also clears its typecheck red — `approximately_equal`
was undeclared, and the band removes the call entirely).

## Probe-first (held again)

/tmp/t27_hyg/iter10_probe2.t27 proved all five target shapes in one
file — single-element `[0]` array as `[]u32` call arg, field+index
`!=` chains (`result1.rotated_q[0] != ...`), two-sided sums of indexed
reads, `sqrt(64.0)` in a when-binding, compound `1.0 / (sqrt(64.0) *
PHI)` expected: **nothing discarded, typecheck 0 errors / 0 warnings**.
No spec was touched before that file came back clean.

## Measured

- then-row: 15 events / 9 specs → **11 / 8**; the three remaining
  events in rewritten specs are exactly the documented #2774
  quantifier-tail invariants (`then for i in 1..len-1:` nested ranges,
  positional_enc 259-260, multi_head_attn 341-343) — verified by
  printing each spec's remaining discards.
- per-spec: attention_mechanism 729→**0** (nothing discarded, typecheck
  OK), positional_enc 109→34, multi_head_attn 70→45, config 41
  unchanged (the @FieldType leave).
- corpus discard sum: 6,621 → **5,792 tokens** (cumulative **−79.0%**
  from 27,562); specs with discard 91→90.

## Discovered for the next iteration

Grepping `approximately_equal` in positional_enc found **9 more call
sites across 8 blocks** — single-level calls that PARSE but fail
TYPECHECK on the undeclared symbol (two sites use a two-argument form
with no tolerance). Same family in multi_head_attn: `random_input`,
`any_mha_config`, `positive_u32`, `sum`. These are test helpers the
spec authors assumed. A candidate for iteration 11: declare the
helpers in the modules that call them (a legitimate spec-side fix —
the two-arg form needs either an overload or a tolerance at those
call sites).

## Self-critique

- The `not all_equal` rewrite asserts two specific differing elements
  — STRONGER than "not all equal" (implies it), and the comment in the
  spec says so. But the claim now depends on the concrete theta_base
  100 arithmetic (1 rad on pair 0) — if someone changes the test's
  q vector, the witnesses may need re-picking. The comment names the
  numbers.
- The 1e-6 band on the phi-scale formula preserves the ORIGINAL
  tolerance while the values are gf16 (~1e-3 relative precision). If
  the executed check fails at runtime, that is the spec's own claim
  failing honestly, not the rewrite weakening it — but I did not
  verify the runtime result; the gate will tell.
- circular_buffer/kd_tree/segment_tree joined reader/sgd as permanent
  ledger residents (~197 more tokens). The family is now 5 specs /
  ~392 tokens of "Zig-shaped module sketches the language cannot call
  natively". Recorded, not resolved — the honest fix is redesigning
  those modules' APIs to t27 shapes, which is feature work, not
  dialect repair.
