# NOW -- X of Y where X can exceed Y, in the other half of a function I had already fixed (2026-09-05)

## X of Y where X can exceed Y, in the other half of a function I had already fixed (Refs #3195)

- tri gates mutate prints N of M equivalence claims contradicted, where M counts distinct claim lines from a union over all directions and N is extended once PER DIRECTION -- so a claim that is a site under two operators and dies under both makes N exceed M. Under --all the command can print 2 of 1
- measured not hypothesised: of the eight mutant-equivalent markers in tools, exactly one is a site in two directions -- gft_backprop_microcode.py line 742, an assert with a comparison, which is both an assert site and a boundary site
- fixed by counting each half in its own unit rather than deleting a number: the ratio now speaks about claims and the per-direction rows are counted as rows and printed with their unit, because which operator killed a claim is the useful half
- SKILL 542 fixed this same function denominator two passes ago and the numerator sat four lines below in the same println. The fix did not travel four lines. Tenth pass running that the wiring outlived the function: three fresh unit tests on distinct_claims all passed against the reverted call site
