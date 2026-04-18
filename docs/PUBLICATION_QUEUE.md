# Publication queue (t27 + Trinity programme)

**Canonical tables:** [`docs/PUBLICATION_AUDIT.md`](docs/PUBLICATION_AUDIT.md) (readiness) and [`publications/README.md`](publications/README.md) (DOI index).

This file is the **human-facing queue**: what should go out **next**, and which **GitHub issue** tracks it.

---

## Queue (edit as you open issues)

| Priority | Artifact | Tracker issue | DOI status | Next action |
|----------|----------|---------------|------------|-------------|
| P0 | First `gHashTag/t27` GitHub Release + Zenodo | *open `publication-task`* | none | Enable Zenodo on repo; tag `v0.x.y` |
| P1 | Conformance corpus as dataset | *open `publication-task`* | none | Checksum manifest; `conformance/README.md` done |
| P1 | GoldenFloat validation CSV bundle | *open `benchmark-task` + `publication-task`* | none | Fill `NUMERICS_VALIDATION.md` §5 |
| P2 | LANGUAGE_SPEC v1 snapshot | *open `publication-task`* | none | Complete `docs/LANGUAGE_SPEC.md` |

---

## Rule

Each row **must** have a **living issue** (`publication-task`, `benchmark-task`, or `audit-task`). Close the issue with the **Zenodo version DOI** when published.

---

*Queue without issues is a wishlist, not a programme.*
