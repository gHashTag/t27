# NOW — a flag that was a banner (2026-08-23)

`tri gates mutate --invert` shipped, merged, and published a number. It never ran the invert operator.

- `Direction::Invert` was declared and documented. `invert_sites()` was written and unit-tested. Nothing joined them — `mutate()` chose `if loud { Loud } else { Silent }`, so the flag printed an invert banner over a silent run.
- Proven two ways before touching it: statically (zero callers, variant never constructed) and empirically (`--invert` and the default printed byte-identical rows).
- All ten existing tests passed with the bug present. Each exercised one function or the other; none crossed between them. That is the defect this command exists to find — function covered, wiring to the answer not — arriving one level up, in the auditor.
- What made it durable: the answer was *plausible*. "One survivor, the same branch both other operators leave" was true in every part, and the last clause was true **by construction** — it *was* the other operator.
- Fixed, and the retraction is written into the entry that carried the number rather than beside it.

**First honest measurement:** 33 invert mutants across 13 gates, 33 killed, no survivors. `--all` now prints all three operators as columns, and they disagree where they must.

**Check added to the skill (§36):** for any new mode or flag, assert it produces something the other modes do not, on one fixture where all of them have a site. Equality across modes is not symmetry — it is the signature of a fall-through.
