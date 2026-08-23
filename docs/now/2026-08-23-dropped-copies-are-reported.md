# NOW -- a copy nobody compared was printed as a copy that agrees (2026-08-23)

Refs #2325.

- `check_duplicate_agreement.py` reported "one behaviour across 9 specs" for
  `dot27` while **13 specs define it**, and 7 for `quantize` while 9 do. Six of
  thirty-six spec-function pairs vanished at the `if d:` join with no output:
  four `dot27` copies emit `int8_t` against a hardcoded `int16_t` in the
  signature pattern, and two `quantize` copies take one argument while the
  harness calls with two.
- The counts in the docstring and in the workflow comment were the COMPARED
  counts printed as the DEFINED counts. Now 14 / 13 / 9, and the gate fails
  when `defined - compared` is non-empty, naming the specs.
- **The "9 specs" was a regex artefact, and the proof is a digest.** With the
  return type relaxed and `tp` added as a dependency, all 13 copies compare
  and produce `8f8e7503` -- byte-identical to the digest the gate already
  reported for 9. There was never a signature boundary there.
- Coverage 30/36 -> 36/36, and every function is still one behaviour: the drops
  were hiding no divergence, which is exactly why closing them was cheap.
  `tmul 91a68892` and `quantize2 1f7b9105` unchanged.
- A second control, `--self-check-drop`, exercises the DROP join rather than
  the digest split. Mutation-tested for orthogonality: neutering the drop
  accounting leaves `--self-check` green and takes the new one red; inverting
  the divergence verdict does the opposite. The assertion names the branch --
  "DIFFERENT behaviours" must be ABSENT -- because a fixture that started
  disagreeing would also exit 1, through the wrong branch.
- On the planted tree the old gate prints `OK tmul one behaviour across 1
  specs` and exits 0: a duplicate-agreement gate certifying agreement over a
  sample of one.
