# NOW -- "Corrected in place" was not (2026-09-04)

## A correction that shipped its announcement and not its edit

A sweep re-verified **158 prose counts** across `docs/`, `.github/` and `.claude/`, rebuilding each
sentence's command and running it at the commit that shipped the sentence. 27 mismatches proposed,
**9 survived** two adversarial lenses. All nine were wrong at their own commit.

- Two of the nine were already named in
  `docs/now/2026-09-04-the-repair-broke-the-gate-the-other-way.md`, which ends **"both corrected in
  place."** They were not. `git show --stat c3cbc25d6` shows `SKILL.md` with **45 insertions, zero
  deletions** -- a new section appended, both wrong sentences left standing -- and the third site
  was not in the commit at all.
- Now actually applied, at all three sites: *"all three have an `else`"* becomes two excluded by
  the `else` clause and one, `sign-release.yml:58`, by the exit clause, having no `else`;
  *"the 90 lines"* becomes **104** (`122 - 18`, not `122 - 32`).
- Separately, the devhome figures: `33 files / 51 occurrences / 28 executable` is dated
  2026-08-29 and kept, because the six-fold ratio is the argument -- but **"one of the 28 is the
  compiler" was false at ship time**, `bootstrap/src/service.rs` having been cleaned eleven minutes
  earlier. Present tense in the skill, in `secret-scan.yml`'s live step comment, and in the tool's
  docstring; the gate's own blessed ledger `tools/devhome_baseline.txt` says **30 / 38 / 25** and
  does not contain the compiler. All three now read past tense and point at the ledger.

**The defect is one level up from the one being corrected.** Re-deriving the arithmetic, naming the
file and giving the right replacement is the hard part, and it was done well. Then the note asserted
the edit had happened -- a claim about an *action*, published with exactly the discipline it was
written to condemn. A deletion count of zero on a file you say you corrected in place is the tell.

Refs #2994
