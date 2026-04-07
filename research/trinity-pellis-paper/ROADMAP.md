# Trinity x Pellis — roadmap (one glance)

PR [#280](https://github.com/gHashTag/t27/pull/280) merged; [issue #277](https://github.com/gHashTag/t27/issues/277) closed via L1.

```text
Done          Merge PR #280 → master
  │
  ├── ~1 d    Issue: expand FORMULA_TABLE 13 → 152 (SSOT: sacred_verification.t27 + future JSON; see FORMULA_TABLE.md)
  ├── ~2 d    Outreach: Pellis runs `tri math compare --pellis --hybrid --sensitivity` (repro joint observable)
  ├── ~3 d    hybrid-conjecture.md: Conjecture H1 + test/falsify protocol (this file + hybrid-conjecture.md)
  ├── ~1 w    Outreach: Olsen — concrete ask (historical section outline for preprint)
  └── ~2 w    Zenodo preprint bump (v0.2+) once PDF + author list ready
```

**Central empirical question:** under a *defined* extension of constants and a *fixed* hybrid map, does the hybrid score **stabilize** or **drift**? Answer drives the paper; the current CLI score (~0.564) is **version 1** of the map — renormalization may change the target value (see `hybrid-conjecture.md`).
