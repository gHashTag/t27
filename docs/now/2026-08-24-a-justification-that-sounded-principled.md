# NOW — a justification that sounded principled (2026-08-24)

Yesterday two gates reported `no success path to break` under the loud operator, and I documented that as an honest limit with a reason. The reason was wrong, and I had written it myself.

- **What the comment said.** *"A ternary can yield 0 on one arm and a verdict on the other; forcing the whole line to 1 there is the Silent operator's job seen backwards, and would be scored against the wrong control."*

- **It is not.** Silent forces the line to `0` — the gate never fails. Loud forces it to `1` — the gate always fails. They are two different mutants of the same line. Excluding one printed a clean-looking row for a gate whose every verdict is a ternary, and that row was an absence of measurement rather than a result.

- **Allowing ternaries immediately found two more survivors — and both were false.** `first_line()` returning `out.splitlines()[0][:88] if out else "(nothing)"`, and `_is_exact_zero()` returning `v == 0`. Helper functions returning values. My predicate scanned for a standalone digit **anywhere** in the expression, so an index and a comparison both read as verdicts.

- **The strict predicate.** A return is a verdict when the whole expression is a literal, or a ternary whose two arms are literals. Anything else is a value the caller decides about, and mutating it perturbs a helper rather than a failure path. The same weakness was in the silent predicate for `1..4`: `return v == 1` would have been taken as a verdict too.

- **The denominators moved, and downward.** `check_elab_ratchet` 10 → 6 sites, `wp18_conformance_gate` 6 → 2, `check_catalog_count` 4 → 3, `check_duplicate_agreement` 4 → 3. Those earlier kills were real — the controls did notice — but they were noticing mutations of helper functions. **A higher denominator looked like more thorough measurement and was measurement of the wrong thing.**

- **Where both operators stand now.** Loud: every gate all-killed except `wp18_conformance_gate`, whose success return is `return code` — a variable, correctly not a site, and the row says `no success path to break` for a reason that is now true. Silent: one survivor, the declared one.

- **Three of my own justifications have now been wrong in this campaign**, and this is the first that was wrong in a way that made a report look better. The other two produced missing coverage; this one produced a row that read as a score.

Refs #2492
