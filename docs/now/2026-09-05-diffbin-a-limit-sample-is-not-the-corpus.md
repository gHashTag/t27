# NOW -- diffbin: a --limit sample is not the corpus (2026-09-05)

## diffbin: a --limit sample is not the corpus (Refs #3195)

- diffbin.py took its coverage denominator from the list AFTER files = files[:limit], so --limit 10 over specs printed "corpus: 10 specs under specs" while 650 .t27 files were present, and a 2 percent sample could print 100 percent coverage under a paragraph saying coverage below 100 percent bounds what the run can claim
- the corpus size is captured before truncation; a sampled run now prints "sample: 10 of 650" and names the coverage denominator "of the 10 compared" plus an explicit THIS IS A SAMPLE block; an untruncated run is unchanged
- scripts/ci/test_a_sample_is_not_the_corpus.py builds its own 25-file fixture and needs no compiler; exit 1 against the pre-fix file, exit 0 after, and moving the capture below the truncation kills it
- tri topic printed "rows searched 759 (open PRs, open issues, last 40 commits, every SKILL.md section)" while its gh reads carried caps of 100 and 200; measured 12 open PRs (slack) and 509 open issues (binding, 309 never read). Raising both to 800 took the same command to 1068 rows and 569 matches. A cap that binds is now named with a LOWER BOUND marker; one that does not bind is not mentioned
