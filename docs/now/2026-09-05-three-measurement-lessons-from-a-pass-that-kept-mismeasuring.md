# NOW -- Three measurement lessons from a pass that kept mismeasuring itself (2026-09-05)

The Rust column moved 224 -> 333 across twelve fixes today, and every one of them was
the same defect class: a rule that existed in one place and did not travel. That story
is already in the notes. These three are about how I measured, which went wrong more
often than the fixes did.

## Pin the instrument before any parallel measurement (§573)

- launched twelve agents over one corpus, wrote into every brief "do not rebuild, it would silently change the very thing you are measuring", and then rebuilt it twice myself while they ran
- one reported it verbatim -- **"THE RULER MOVED UNDER ME"** -- naming the byte size it started against and the size it later saw; a second said its control figure did not reproduce
- the run was stopped and its counts discarded; the repair is `cp target/release/t27c /tmp/t27c-pinned` and giving agents only that path, with its `shasum` printed in the brief
- on the re-run every control came back **0 occurrences among the passing population** -- total discrimination, which is what a fixed ruler buys
- a structural observation survives a contaminated run and a COUNT does not, so the discarded run was still worth reading for its mechanisms

## A defect you merged reads as pre-existing on every later branch (§574)

- a required gate went red on master; I read three consecutive red runs, and `tri pr ready`'s "also failing in 4 other place(s) -- pre-existing", as evidence it was not mine
- my pull request merged at **02:21:25Z** and the gate's first master failure is at **02:21:28Z** -- the run its own merge triggered. Seven passes before, five failures after
- "failing elsewhere too" answers *is this unique to this branch* and never *did you cause it*, and I substituted the second answer for the first question because it exonerated me
- the separating question is one command -- when did this gate last pass, and what landed between -- and it found a SECOND breakage of mine the same day, also three seconds after my own merge

## A declined probe measures where its rule ends (§575)

- I probed a type mapping at **+1 / -1**, declined it as net zero, and recorded it as priced-and-declined
- hours later a fan-out proposed the same mapping and it measured **+1 with no regression**; the difference was one word
- my version keyed on the element type and caught both `[]const u8` and `[]u8`; the regression was a spec whose `[]u8` is indexed as `chain[(idx) as usize]` and cannot be a string. Keying on the `const` QUALIFIER leaves the mutable buffer alone
- the regression was never a refutation -- it was a measurement of where the rule ends, naming the exact case the rule must not cover
- a refusal recorded as a bare net-zero reads later as a closed class, and the next reader is you: write the distinguishing word into the refusal
