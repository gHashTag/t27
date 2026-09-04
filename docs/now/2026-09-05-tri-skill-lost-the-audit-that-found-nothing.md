# NOW -- tri skill lost: the audit that found nothing (2026-09-05)

## tri skill lost: the audit that found nothing (Refs #3195)

- walked 281 commits and 518 titles ever written to SKILL.md, looking for bodies that are a strict prefix of their first version -- the exact signature of the truncation that destroyed section 550
- 40 prefix hits, of which 38 differ by trailing blank lines alone; the 2 real cut tails are both unnumbered blocks that moved elsewhere and are still on master, and the 2 absent titles are one rewrite under a longer heading and one deliberate withdrawal whose commit says so
- zero unexplained losses: section 550 was the only one, caused by the tool in this session, and repaired. A clean audit is what makes that a closed incident rather than a sample of an unknown population
- eighth pass running whose surviving mutant was the wiring, and the first found by mutating the call site BEFORE writing any test for the helper -- the rule the previous seven produced
