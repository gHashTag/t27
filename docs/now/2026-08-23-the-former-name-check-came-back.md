# NOW -- a comment outlived the check it described (2026-08-23)

Refs #2325.

- Four lines of comment explained why the former name must stay searchable --
  research notes, prior branches, the author's profile, and every measurement
  against takum/tekum/posit recorded under the old label. The code beneath them
  tested for the golden-section rule, a different thing entirely. A later
  commit had replaced the check and left the reasoning behind.
- Measured: stripping all nine `former_name=` fields from the SSOT passed
  **both** catalog gates green. A repo-wide grep finds `former_name` nowhere
  outside `specs/` and two docs, so nothing else would have noticed either.
- Restored as its own rule: every `tnf<N>` rung must carry
  `former_name="GF-T<N>"`. The golden-section test stays, renumbered as rule 5,
  so both invariants now have a check AND a comment that describes it.
- Controls: stripping all nine names lists all nine by rung and exits 1;
  stripping exactly one names that one and exits 1; the clean tree exits 0.
