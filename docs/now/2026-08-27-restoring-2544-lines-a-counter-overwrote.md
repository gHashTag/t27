# NOW -- Restoring 2544 lines a counter overwrote (2026-08-27)

## Restoring 2544 lines a counter overwrote (Refs #2161)

- Refs #2161, #2713. Commit 4639b38cd replaced the i-th non-ASCII character of each .t27 file with the ASCII digits of i. The transform is exactly reproducible from the pre-image, and reproducing it is what makes the repair safe: I verified the model reproduces the commit output BYTE FOR BYTE on specs/isa/ternary_gates.t27 before touching anything
- A line is restored ONLY when the line at HEAD is byte-identical to the transform of its pre-image line, so any line edited in the four months since is left alone. 141 files, 2544 lines
- Measured with the master binary so the restoration is isolated from the compiler fixes: specs that parse 603 -> 610, regressions 0. The 7 are exactly the "Expected LBrace, got Number" cluster -- they failed not because they LOST an arrow but because they GAINED an integer literal
- A destroyed invariant came back: specs/vsa/jones_polynomial.t27:335 read `assert jones_trefoil_at_phi() 1222 PHI + PHI * PHI within 0.1` and now reads `... ≈ PHI + PHI * PHI within 0.1`. The comparison operator was the 1222nd non-ASCII character of the file
- This takes the RESTORE branch of the question in #2713 rather than transliterating, on two grounds: the pre-image is exact and a transliteration is a new authoring decision, and the L3 ASCII rule is enforced by nothing today -- 320 of 747 files carry 88033 non-ASCII characters at HEAD and no gate objects. The repair script is committed, so the transliterate branch remains available
