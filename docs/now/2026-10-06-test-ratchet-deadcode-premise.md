# NOW -- test-ratchet: a test whose premise the corpus outgrew (2026-10-06)

## the_dead_code_census_names_what_it_skipped (Closes #6687)

- `Bootstrap Test Ratchet` on master reported two newly failing tests. This entry is about one of them; the other, `corpus_classifier_matches_lean_completeness` (`ar_ternary_logic`), is handled by PR #6672 for #6658.
- `the_dead_code_census_names_what_it_skipped` asserted that the live tree holds at least one spec that does not parse. Master now parses every spec: `t27c deadcode --repo` on edb537227 reads 1317 walked, 0 did not parse, 1317 counted (Railway t27c lab).
- The counter is wired: a scratch tree with one good and one broken spec reads 2 walked, 1 did not parse, 1 counted.
- The test is entered in `scripts/ci/test-baseline.txt` as a KNOWN GAP with a comment that points to #6687. The real fix is a Rust test change (a scratch-tree witness), proposed in prose in #6687; it needs the owner's `owner-approved-foreign` label and must remove the baseline line.
