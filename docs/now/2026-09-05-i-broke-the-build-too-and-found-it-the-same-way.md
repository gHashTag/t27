# NOW -- I broke the build too, and found it by asking the same question (2026-09-05)

Second gate broken by me today and read as someone else's. The question that found
both is one command: **when did this last pass, and what landed between?**

## The build (Refs #3225)

- `cli-tri` last passed on master at **01:52:40Z** and first failed at **02:08:28Z**
- my `tri misread` merged at **02:08:25Z** -- three seconds before, the run its own merge triggered
- the failing step names itself: **"No census moved without saying so"**, and the census is `fetches`: `files read 42 -> 43`
- the new file is `cli/tri/src/misread.rs`, which that census counts. The move is real and correct; what was missing is the sentence saying so
- the gate states its own remedy: re-bless in the SAME commit and say which number moved and why. That is this commit

## The other half is not mine, and the timing proves it (Refs #3225)

- master also fails `tri skill check`: **section 567 appears three times**, and the file reads out of order
- the three sections come from `838fb817b`, `d30acede3` and `10889fc78` -- three neighbour branches that each appended a section numbered 567 and each squash-merged
- they landed at **02:36:46Z** and **02:51:28Z**, AFTER the first build failure, so they are a second, independent breakage rather than the cause of the first
- I am not renumbering them: `tri skill renumber` moves sections YOU appended against a base, and pointing it at three other sessions' sections is how a resolver deletes work silently

## What I keep getting wrong (Refs #3225)

- earlier today I read three red master runs and `tri pr ready`'s "also failing in 4 other place(s) -- pre-existing" as evidence a gate was not mine. It was mine, merged 3 seconds before the first failure
- **a defect merged to master fails on every later pull request, which is exactly what pre-existing looks like** from a branch-versus-master comparison
- "failing elsewhere too" answers *is it unique to this branch*, never *did you cause it*
- both of today's breakages were found by the same one-line question, and neither would have been found by staring at the failure text
