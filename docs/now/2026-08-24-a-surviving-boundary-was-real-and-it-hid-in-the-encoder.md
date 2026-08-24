# NOW -- A surviving boundary was real, and it hid in the encoder (2026-08-24)

## A surviving boundary was real, and it hid in the encoder (Closes #2161)

- if off < 0: return 0 in the GF-T encoder survived the boundary operator. Reading the source beside the line number — added yesterday — showed it is not a loop bound: off = e + 40 is zero in the smallest normal binade [2^-40, 2^-39), and the mutant encodes values there as 0, which is the zero sentinel.
- The exact power 2^-40 is not a witness: its mantissa is zero so it already encodes as 0 and the mutant changes nothing. A witness needs a non-zero mantissa — 1.5·2^-40 encodes 256 and would become 0 [measured].
- Three assertions pin the binade. Negative control: with the mutant planted the tool exits 1 on the new assertion; restored, it exits 0.
- And the classification corrects yesterday's generalization. Of four 'if x < 0' sites, only the find-idiom one is equivalent; the other three are real boundaries where zero is a valid value.
