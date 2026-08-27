# NOW -- Four defects, and not one test that would have caught them (2026-08-28)

## Four defects, and not one test that would have caught them (Refs #2161)

- Refs #2161. All four backend defects fixed this session emitted a green exit over output that was wrong or absent, and all four were invisible to the 1600-test suite -- because those tests read the emitted TEXT and not one of them hands it to a compiler. The Verilog backend has had iverilog targets in bootstrap/tests/ for a long time; C and Rust had nothing
- Seven behavioural tests added: generate, compile with the real toolchain, RUN, check the printed answer. Range loop prints 6 not 1; the inclusive range runs three times; a wide un-annotated local prints 4294967297 in both C and Rust AND the two are asserted equal to each other; a module-level var reaches 2 after two calls; the switch arms are in the Rust output
- A test that cannot find its compiler SKIPS LOUDLY rather than passing quietly. An absent tool is not a passing test, and this file exists precisely because silence looked like success
- Verified load-bearing rather than assumed: reverting the __auto_type fix fails the agreement test with "C truncated a u64 to int"; reverting the no_range fix fails two of the loop tests; restoring both returns 7/7. A green test suite proves nothing until you have watched it go red
