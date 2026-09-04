# NOW -- The third reading was right, and the first two were guesses (2026-09-04)

## Splitting the function is the fix; renaming the helper was not

- Two predictions failed last pass. `classify_fetch` takes the **enclosing function** as its
  subject and returns `Guarded` only when that function holds **one** fetch site beside a
  recognised guard. `fn ready` held two, so any guard there landed in `GuardAmbiguous` -- and the
  census called it unguarded twice, correctly.
- **The fix is to split, not to rename.** Each fetch now lives in its own function --
  `recent_commits` and `merged_recently` -- with one site and one guard.
- **The guard already existed.** `issues.rs` has `read_is_complete(returned, limit)`, which the
  classifier recognises by name, and my `page_was_full` was a second definition of it, inverted.
  Two literals of one predicate, and the reason the census kept reading these as unguarded.
  `page_was_full` is gone.

Prediction recorded before the work, and this time grounded in reading the classifier:

      prints what it got            5 -> 3
      asks whether the page filled  2 -> 4

Both held exactly, and both `prcheck.rs` sites left the list. 617 crate tests pass.

Of the three that remain, two are `gates.rs` and already paginated in the unlanded #3161; the last
is `red.rs::runs_url`, which the tool explains as a function that only builds a URL while its guard
lives in the caller.

**Two guesses and one reading.** The difference was not effort -- it was opening the classifier
instead of predicting what it might want.

Refs #3157
