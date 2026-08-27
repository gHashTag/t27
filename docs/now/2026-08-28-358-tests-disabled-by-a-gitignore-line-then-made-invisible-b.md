# NOW -- 358 tests disabled by a gitignore line, then made invisible by a ledger (2026-08-28)

## 358 tests disabled by a gitignore line, then made invisible by a ledger (Refs #2161)

- Refs #2161. specs/scratch/ is gitignored -- 578 MB of generated drafts, re-derivable from 343 committed generators. In a fresh clone the whole icarus_lowerable target panicked on read_dir, and that failure was then recorded in the CI baseline as "known failing": 358 tests, 15% of the suite, disabled by a .gitignore line and made invisible by a ledger
- A missing INPUT is not a failing test. It is also not a passing one, which is why the guard PRINTS rather than returning quietly -- a silent skip is how the count reached 358. Each skip names the test, the directory, and how to restore it
- Before: 358 failing, all for the same absent directory. After: 357 skipping loudly, 1 failing. That one is the point -- corpus_classifier_matches_lean_completeness fails on a REAL disagreement (specs/api/tri_net_api.t27: Rust says not lowerable, the Lean theorem says lowerable), identically before and after, checked. It was the only true failure in the target and it sat among 357 false ones
- The baseline recorded all 358 as the same thing. That is what a ledger does to a population it cannot distinguish: it makes the one that matters cost exactly as much attention as the 357 that do not
