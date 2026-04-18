# What remains speculative — and why this is not numerology

**Audience:** Reviewers who see **phi**, **ternary**, and **“sacred”** labels and need a **clear boundary** between **engineering** and **exploratory physics narrative**.

For a dedicated “not numerology” argument, see [`docs/WHY_THIS_IS_NOT_NUMEROLOGY.md`](WHY_THIS_IS_NOT_NUMEROLOGY.md).

---

## Not numerology

The project uses **φ** and ternary structure as **engineering constraints** where they:

- Define **numeric formats** (GoldenFloat family) with stated bit layouts (`docs/NUMERIC-STANDARD-001.md`).  
- Define **logic** interfaces (e.g. K3-style unknowns in AR specs) as **specified** behavior, not mysticism.  
- Enforce **reproducibility** (CI, seals, conformance) so claims are **testable**.

**Numerology** would mean: claiming physical truth from aesthetic coincidence **without** measurement, uncertainty, or falsification. This repo **rejects** that standard for **core compiler/language claims**.

---

## What is still speculative or empirical

| Area | Nature | Required honesty |
|------|--------|-------------------|
| Phi-linked **physical constant** relations in `specs/math/**` | Often **empirical fits** or approximations | Label each relation: `exact identity`, `empirical fit`, `within CODATA uncertainty`, `conjectural`. |
| “Sacred physics” as **fundamental law** | **Not** claimed for the whole language | Physics overlays are **domain specs**; the **t27 core** is definable without them. |
| GoldenFloat vs IEEE / posits | **Engineering hypothesis** | Needs benchmarks + error envelopes (`docs/NUMERICS_VALIDATION.md`). |
| Full AR soundness | **Research** | Bounded traces and restraint are **specified**; complete proofs are **work in progress**. |

---

## Separation rule (P0)

**Core language + compiler correctness obligations** must be explainable **without** adopting any controversial physics interpretation. Anything else lives in **labeled research specs** and `docs/RESEARCH_CLAIMS.md`.

---

## Related

- `docs/PHYSICS_REVIEW_PROTOCOL.md` — when external physics review is required.  
- `docs/RESEARCH_CLAIMS.md` — claim status and falsification.  
- `docs/REPOSITORY_EXCELLENCE_PROGRAM.md` — hardening roadmap.
