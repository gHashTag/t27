# NOW -- export_push read the wrong field, and took 0 of 348 (2026-09-23)

## The queue was readable all along; the filter emptied it (Closes #4607)

- The secret landed today and the workflow proved it works on the first run: `waiting: 348 branch(es); taking 0`. The credential was never the last blocker - the field name was.
- `/queen/export` sends `commits` per branch (`queen-export.ts`). The filter read `ahead`, which is absent, defaults to zero, and zero is what it drops. Every one of the 348 was filtered out.
- The self-test did not catch it because its fixtures were written from the same assumption as the reader: both said `ahead`, so they agreed with each other and disagreed with production. A test whose fixtures come from the same hand as the code under test checks a guess against itself.
- Fixed to read `commits`, with `ahead` still accepted so a later route change cannot become a second silent zero. Two cases added: the old name still works, and an entry with neither name means nothing to take.
