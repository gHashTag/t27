# Why this is not numerology

**Claim:** Use of **φ**, ternary structure, and “sacred” labels in t27 is **engineering and specification discipline**, not numerological proof of nature.

## Criteria we reject

Numerology asserts **hidden cosmic truth** from symbol patterns **without**:

- reproducible measurement,  
- stated uncertainty,  
- or a falsification experiment.

## What we do instead

1. **Specified formats** — GoldenFloat layouts and tolerances live in `.t27` + `conformance/*.json` (`docs/NUMERIC-STANDARD-001.md`).  
2. **Test hooks** — CI runs parse, codegen, conformance JSON checks, gen headers, seals (`tests/run_all.sh`, `repro/Makefile`).  
3. **Explicit epistemic labels** — Physics-flavored relations are marked **empirical / conjectural** where appropriate (`docs/RESEARCH_CLAIMS.md`, `docs/PHYSICS_REVIEW_PROTOCOL.md`).  
4. **Separation** — Core language/compiler claims do **not** depend on adopting speculative physics (`docs/WHAT_REMAINS_SPECULATIVE.md`).

## If a claim cannot pass the bar

It is downgraded to **research-only** documentation or labeled **untested** until evidence exists.

---

*Skepticism is welcome; the repo’s job is to route it to the right artifact.*
