# NOW -- The cost, not the count (2026-08-29)

## The cost, not the count (Refs #2774)

- Three censuses reported HOW MANY quantified clauses are walkable. None
  reported what walking them would cost, which is the number a ceiling has to
  be chosen against. `tri quantifiers report` now prints it: **2 768 791
  evaluations** over the 119 walkable clauses.
- The distribution is not a distribution, it is a cliff. **42 clauses (35.3%)
  carry 2 752 512 evaluations -- 99.41% of the total.** All 42 sit at exactly
  2^16; 40 of them in `specs/igla/race/`, binders named `angle`, `psum`, `acc`.
- The default ceiling of 65536 sits ONE UNIT above a plateau that spans
  256..65535. That unit buys +42 clauses for +2 752 512 evaluations: a 170x
  multiplication of cost for a 35% increase in coverage. The default is not
  changed here -- the sweep is printed and the choice is the owner's.
- The sweep is DERIVED, not sampled. A ceiling only matters where a domain size
  sits, so the plateau tops are the distinct sizes: **413 finite clauses occupy
  exactly 19 of them**, and every other ceiling is a synonym. An earlier sampled
  sweep called 256..65535 the widest flat region; it is not, 2^48..2^64-1 is,
  256x wider, and the sampling had never looked inside it.
- 82% of the finite domains are exact powers of 256. This is a hardware-spec
  corpus and its quantifiers run over register widths, so the sweep prints
  `2^16` rather than `65 536`.
- Stated in the report, not left for a reader to assume: the sum is an
  ITERATION COUNT, not a cost. No backend executes an enumerated quantifier
  today and the number is blind to body weight. It must not be quoted as
  seconds.
- Every count in the sweep is a LOWER BOUND twice over: the last row is
  saturated (`u128::MAX`, `saturating_mul`), and **56 unbounded clauses are
  unbounded only because a struct name has more than one definition** (63 touch
  one). Resolving those can only move clauses inward.
- Two source lines that misreported the tool's own output are fixed: a doc
  comment saying 50 conflicted names while the binary printed 80, and `--help`
  calling the default "deliberately small". A truncated list of 8 of 80 now
  says so.
- ci-gates 226-229. 266 tests pass; report runs in 0.50 s.
