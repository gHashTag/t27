# NOW -- Two predictions wrong, and the detector right both times (2026-09-04)

## Recording the expected movement first is what made this a check

Before landing five pull requests I wrote down what each should move, so the verification could
fail. Two did.

- **#3158 was predicted to take `prints what it got` from 5 to 3.** It merged, and the figure
  stayed at 5. `baseline_phrase` made the *sentence* honest -- it says how many merged PRs were
  actually compared -- but it did not remove the *bound*. Saying how many you read is not saying
  whether more existed.
- **So I added `page_was_full`, and predicted 5 to 3 again.** Still 5.
- **Then I read the classifier instead of guessing a third time.** `classify_fetch` recognises
  three guard names -- `read_is_complete`, `is_lower_bound`, `total_count` -- and mine is none of
  them. And even a recognised guard would not have helped: `fn ready` holds **two** bounded
  fetches, so one guard lands in `GuardAmbiguous`, a bucket the tool already explains as
  *"a guard, but two fetches -- which one does it cover?"*

**The detector is right and I was wrong twice.** Renaming `page_was_full` to `is_lower_bound`
would turn the number green while covering only one of the two fetches. **Making a detector go
quiet is not the same as fixing the defect**, and this is the first time in this loop that the
cheap way to a green number was a rename.

## What shipped anyway

`page_was_full` and `baseline_phrase_bounded` are a real improvement to the sentence: a shortfall
now says *"the fetch page was FULL, so more may exist beyond it"*, and says nothing of the kind
when the page was short -- because a quiet week returning fewer merged PRs is the true answer, not
a truncation. Five tests, including that distinction, which is the one a careless version gets
backwards.

`fn ready` still carries two bounded fetches and still reads as unguarded. That is a correct
reading of code I have not fixed.

Refs #3157
