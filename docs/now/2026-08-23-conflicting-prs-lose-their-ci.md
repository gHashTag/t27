# NOW — a conflicting pull request loses its path-filtered CI (2026-08-23)

Yesterday left one pull request's missing CI undiagnosed. Checking every open one found the mechanism — and then the mechanism corrected itself.

- **Three more pull requests have the same shape.** Two change `bootstrap/src/compiler.rs` and got **three checks**, every one path-less. That file's gate carries the comment *"Without `bootstrap/**` here, a PR that rewrites the C emitter merges with the cross-target proof never running."* The path was added; **the gate is defeated by a merge conflict instead.**
- **The mechanism:** a pull request that is CONFLICTING when an event fires cannot have its merge diff computed, so `paths:` filters cannot be evaluated and only path-less workflows run. The remaining checks are green — they never look at the diff — which reads exactly like a passing pull request.

## The rule I first wrote was wrong

The correlation looked exact: four conflicting PRs with 3, 3, 9, 7 checks; everything else 21–35. I wrote *"a CONFLICTING pull request loses most of its checks"* into the tool's documentation.

**An hour later two of those four reported 21 and 26.** They had been mergeable when their events fired, kept those results, and only conflicted afterwards. **A conflict does not retract past runs.**

The detectable shape is not the state — it is *conflicting **and** a check list far below its siblings'*. `tri gates prs` now computes a reference from the non-conflicting pull requests. Three flagged, two correctly excluded.

**Caught by re-running the command on fresher data**, not by thinking harder — which is the argument for putting a finding into a tool rather than a document. A sentence cannot disagree with tomorrow's data; a command can.

And it crashed on its first run, slicing a title by byte index in the middle of an em dash — in a pull request this campaign had opened.
