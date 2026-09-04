# NOW -- tri mutate run was counting its own tests (2026-09-05)

## tri mutate run was counting its own tests (Refs #3195)

- find_mutants masks comments and strings and nothing else, so a literal inside a cfg(test) module is perturbed like any other; the test holding it fails and that red is reported as the checker noticing
- measured by the tool itself with --cmd true: 45 of the 59 sites it finds in red.rs are inside the test module (76 percent), 1545 of 3198 across the crate. Reproduced: perturbing render_headline(50, ...) in a test call fails the suite, over a number that exists only in that test
- sites inside a Rust cfg(test) module are now dropped by the same gates::test_module_lines rule used elsewhere, and the number dropped is PRINTED. Rust only: diffbin.py still reports all 61 of its literals with no skip line
