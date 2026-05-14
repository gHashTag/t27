# Physics review protocol

**Purpose:** Decide which statements require **external theoretical physics review** vs **internal engineering review** vs **exploratory appendix only**.

---

## Tiers

| Tier | Content | Review |
|------|---------|--------|
| **A — Core language** | Syntax, types, codegen contracts, conformance | PL / compiler reviewers; **no physics gate**. |
| **B — Reference numerics** | CODATA/NIST constants as data in specs | Verify sources and uncertainty budgets; cite official values. |
| **C — Empirical phi models** | Fits tying constants to phi-scaled templates | **Label as empirical**; statistician / metrologist-friendly appendix; optional external physics consult. |
| **D — Speculative unified claims** | “Everything reduces to φ” style | **Not** allowed in core language claims; only research track + clear disclaimer. |

---

## Checklist before claiming “derived”

- [ ] Is the statement an **algebraic identity** in a formal model?  
- [ ] Or a **fit** with residuals and dataset version pinned?  
- [ ] Or a **conjecture** with falsification experiment defined?

If none of the above, **downgrade the wording** or move to Tier D.

---

## Publication gate

Papers mixing **B** and **C** must **separate** sections: “Reference data” vs “Empirical model” vs “Conjecture” so reviewers cannot confuse them.

---

*Core t27 credibility must not depend on Tier D.*
