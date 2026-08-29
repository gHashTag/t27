# NOW -- Every binder is a binder (2026-08-29)

## Every binder is a binder (Refs #2774)

- `tri quantifiers report` sized a clause's domain from its FIRST binder only.
  `parse_binders` did `upto.split_once(':')` -- one split, so the first colon
  ended the binder list. 297 of 924 clauses write a colon per binder and were
  read as though they bound one variable.
- Worst case, and the one to put in front of a reader:
  `specs/igla/race/cordic_top.t27:311` declares
  `forall clk : bool, rst_n : bool, angle : i16, valid_in : bool` and was
  printed as `walkable |D| = 2`. True domain 524 288 -- eight times over the
  ceiling, presented as the second-cheapest clause in the corpus.
- Measured from the binary after the fix: walkable **193 -> 119**,
  finite-over-ceiling **250 -> 294**, unbounded **456 -> 486**, no-binder 25
  unchanged. 924 clauses, buckets sum to 924. 302 rows gained a binder.
- The direction was provable before any code ran: a dropped binder can only
  make |D| smaller and `Unbounded` is absorbing, so no clause can move INTO
  walkable by this fix. The report has only ever over-promised what could be
  checked by brute force.
- Four distinct parse paths were wrong, not one: the colon list (297), a second
  `forall` inside the binder text (`systolic_array.t27:287`), space-separated
  names (`bench_proxy.t27:349`, filed as no-binder), and a colon meaning "such
  that" (`gf16_bfloat16_nmse.t27:89`) where the reader FABRICATED a binder
  `[D: F,]` whose name and type are both absent from the source.
- 69 clauses put the body after a comma (`forall p : T, body(p) >= 0`) and the
  old one-split was right about them by accident. The fix walks segments and
  STOPS at the first non-binder; a fix that kept walking would have corrupted
  those 69 to repair the 297.
- Caught by the monotonicity invariant, stated before the code: my first guard
  was an acceptor written from the types I had seen, and it called 19 real
  binders (`input : [u32]`, `assign : m.assigns`) "no binder". Inverted to a
  rejector. Eighteen green unit tests did not catch it; a one-line invariant did.
- Stale numbers corrected in place or bannered: two `docs/now/` entries,
  `.claude/skills/ci-gates/SKILL.md` §195, and a new comment on #2774 naming
  both prior comment URLs. ci-gates 216-218.
