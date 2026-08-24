# NOW -- The multiplier had the same rounding gap, and my control silently did not apply (2026-08-24)

## The multiplier had the same rounding gap, and my control silently did not apply (Closes #2161)

- _magmul rounds with the identical round-half-to-even pair as _magadd, over a product rather than a sum, and its r > half survived for the same reason: the self-tests check accuracy and an optimiser absorbs a last-bit error in every multiply [measured].
- It has two paths with different half, so four cases are needed: only even-q rows distinguish >=, only odd-q rows distinguish a dead parity branch. Measured by planting each mutant — 20610 vs 20611, 20738 vs 20737, 20998 vs 20999, 21016 vs 21015.
- My first negative control reported the >= mutant as surviving. It had not been planted: the replacement string lost the leading 'if ', so the substitution was a no-op and the control tested nothing. A control that silently fails to apply reads as the strongest possible evidence for the wrong conclusion.
- Both controls now assert the anchor exists before replacing. With that, >= is caught by the even-q assertion and disabling parity by the odd-q one.
