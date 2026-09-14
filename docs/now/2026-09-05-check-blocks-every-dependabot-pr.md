# NOW -- check blocks every Dependabot PR (2026-09-05)

## check blocks every Dependabot PR (Refs #3335)

- The required context `check` has no trusted-bot bypass: IS_BOT appears 0 times and no
  step is conditional. Its two neighbours have 6 and 3.
- #1081 gave those two the bypass on 2026-06-14, and stated why it is a no-op PASS step
  rather than a skipped job: a skipped required check never satisfies branch protection.
- This job was exempt only because its whole body was an `echo`. Its blindness WAS the
  bypass, and when #2756 gave it real work the exemption went with the blindness.
- Measured: eight open Dependabot pull requests. The three opened after the change are RED
  on `check` and green on the other three required contexts. The five opened before are
  green only because their last run predates it; a synchronize or a title edit flips them.
- The ruleset is not editable and there are no bypass_actors, so red `check` means never
  merges.
- The failure text asserts what is false on this very population: "NOW Sync Gate should
  have caught that first" -- on a bot pull request NOW Sync Gate deliberately does not look.
- Ported character for character; the expression is now identical in all three files,
  checked rather than assumed.
- Found by an adversarial pass over the four required contexts: 8 candidates, 6 survived
  refutation, 2 refuted. The other five are recorded for the next pass.
