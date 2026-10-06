# NOW -- blog-share v4: a fresh article goes before the owner's queue (2026-10-06)

## blog-share v4: a fresh article goes before the owner's queue (Closes #6790)

- specs/automation/blog-share.t27 VERSION 4: FRESH_MS = 604800000 (7 days), is_fresh(age_ms) with the edge included, next_source(fresh, queued, left) -> fresh | queue | newest | none.
- Why: 2026-10-04..06 a queue of August articles kept the fresh ones out of X @t27_dev for a week; host fix gHashTag/999-multibots-telegraf#3786.
- t27c test-report 13/13 pass; negative controls: is_fresh '<=' -> '<' FAIL 1, next_source fresh guard '> 0' -> '> 9' FAIL 1. Seal re-saved.
