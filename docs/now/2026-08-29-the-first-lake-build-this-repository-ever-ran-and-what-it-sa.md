# NOW -- The first lake build this repository ever ran, and what it said (2026-08-29)

## The first lake build this repository ever ran, and what it said (Refs #2747)

- 28 errors on the first run, all real: definitions over the reals with no noncomputable marker, tactics that could not unfold a def, and one proof whose bullets did not match its lemma
- 28 to 2 across six dispatches; the two that remain are true with room to spare (0.1695 against 0.1-1, and 0.2562 against 1)
- stopped guessing mathlib lemma names down a four-minute CI loop and wrote down what each needs instead -- half a proof in the tree is worse than one named failure
- lt_div_iff and div_lt_iff do not resolve in this revision; the zero-suffixed spellings do
