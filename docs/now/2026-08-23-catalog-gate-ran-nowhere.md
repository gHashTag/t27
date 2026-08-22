# NOW -- the only gate that catches an emptied catalog ran nowhere (2026-08-23)

Refs #2325. From the gate audit, each item reproduced before acting.

- `check_catalog_integrity.py` was invoked by **no workflow, no hook, no
  Makefile target** -- a whole-tree grep found it in `docs/NOW.md` and nowhere
  else. It is the only check that catches an emptied catalog: strip every
  `// CATALOG:` line and `check_catalog_count.py` prints
  `OK: SSOT == fresh regen == 0 (canonical)` and exits 0, while integrity exits
  1 with 9 problems. Now wired into `catalog-count-gate.yml`.
- The count gate gained a FLOOR. Equality alone is satisfied at zero, and both
  counters read the same file, so at zero the independent path collapses onto
  nothing. `MIN_ROWS = 109`; the ladder has only ever grown (83 -> 92 -> 109),
  so a drop is a deliberate act that lowers the constant in the same commit.
- The OVERLAP branch was dead code. `gf\d+`, `gft\d+`, `bnf\d+`, `tnf\d+` are
  pairwise disjoint under fullmatch -- brute-forced every string up to six
  characters over the alphabet, zero match two families. Replaced with the
  check its own comment describes: two rows naming the same spec file. Holds
  today (43 rows, 43 distinct specs, one to one); a planted alias fails.
- Docstring said "three phi-family neighbours" where the loop iterates five.

## the audit was wrong once, and it is worth recording

The audit proposed a `source=` UNIQUENESS check. `source=` is a citation --
"Alam 2021", "IEEE 754-2008" -- and 30 of 109 rows legitimately share one, so
that check would have failed on the clean tree the day it landed. Verified
before implementing; the invariant that does hold is per-SPEC, not per-source.
A refuted finding survives refutation only in the direction it was checked.
