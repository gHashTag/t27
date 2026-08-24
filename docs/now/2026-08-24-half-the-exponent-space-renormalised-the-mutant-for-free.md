# NOW -- Half the exponent space renormalised the mutant for free (2026-08-24)

## Half the exponent space renormalised the mutant for free

- Refs #2161. `enc`, `_magmul`, `_magadd` and `_magsub` all end in `if mant >= 512: mant = 0; <exponent> += 1`, and all four survived the `>=` -> `>` boundary mutant. ONE mechanism for all four: the mutant leaves `mant == 512`, and `(off << 9) | 512` is IDENTICAL to `((off + 1) << 9)` whenever `off` is EVEN, because `512 == 1 << 9` is the low bit of the exponent field. The stuck mantissa renormalises the value by accident across half the exponent space
- Three probes in a row read "no difference" for reasons about the PROBE, not the subject: a sweep confined to one binade (`off = 40`, even); a sweep of exactly-representable values (`mant == 512` is reached ONLY by rounding up from 511, so the branch is never entered); and a stale `__pycache__` -- the mutant changes one character, so the file SIZE is unchanged and Python reused the previous build bytecode
- The stale-bytecode run is the one worth keeping: all four sites reported the SAME difference count at the SAME position, inside a function three of them do not touch. An IMPOSSIBLE result is the cheap signal; a merely wrong one would have shipped
- In `_magmul` the carry path cannot reach `mant == 512` in principle -- the largest product `1023 * 1023 == 1046529` gives `q == 1022`, `mant == 510`. Only the no-carry path distinguishes that site. My hand-derivation said `q == 1021`; the computation said 1022
- Four assertions, each checked both ways: holds clean, fails when its own site is mutated, and catches no other site mutant -- so all four are load-bearing rather than three plus a duplicate. Boundary column for this file, measured: killed 8/31 -> 12/31
