# NOW -- review_log.t27 tests pin str_starts_with as a prefix check (2026-10-07)

## review_log.t27 tests pin str_starts_with as a prefix check (Closes #7396)

- Three test blocks whose answer differs between a prefix check and a substring check: a line carrying 'reviewer: gh pr list' or 'gh pr list -R: transient failure' after offset 0 classifies as OTHER, the prefixed line keeps LISTING_ERROR_504 / LISTING_ERROR_OTHER / RETRY, plus direct str_starts_with assertions.
- The 14 existing tests passed with str_starts_with written either way, so the #7359 fix had no witness. Negative control: with the body replaced by str_contains, test-report shows 14 pass, FAIL 3.
- t27c test-report: 17 tests, 17 pass, FAIL 0, 0 vacuous; seal refreshed.
