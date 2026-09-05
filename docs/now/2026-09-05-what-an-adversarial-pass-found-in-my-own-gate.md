# NOW -- What an adversarial pass found in my own gate (2026-09-05)

## What an adversarial pass found in my own gate (Refs #3292)

- Five agents with distinct lenses attacked the gate before it went into a REQUIRED
  context. It had already passed unit tests, three killed mutations, a 498-commit
  historical sweep with one refusal, and a two-reader agreement check with zero
  disagreements. It still had five defects.
- Four categories of legitimate work the extension whitelist would have refused: 14 `.xdc`
  and 4 `.tcl`, the actual deliverable of timing work under `fix(verilog)`; 43 `.toml`,
  where a build breakage lives; 72 `.lean` formalising the compiler's own lowering; and
  every extensionless path -- `Makefile`, `Dockerfile`, `scripts/tri` -- that no extension
  entry can ever match. The list was never going to close.
- The inversion that replaced it had its own version of the same error: it called anything
  under `docs/` prose, and `docs/` holds 11 `.py` and 4 `.sh`. A directory prefix is not a
  claim about content. The prose test is now document FORMATS only.
- `on: pull_request` defaults to `[opened, synchronize, reopened]`. `edited` is not in it,
  and the gate reads the pull request TITLE. So a benign title could be renamed to
  `fix(rust)` after the last green run and never be re-read -- and an author following the
  failure message's own advice, "name that scope instead", would edit the title and watch
  the required context stay red with no way to re-run it but pushing a commit.
- GitHub's squash defaults the commit message to the pull request title only when the
  branch has MORE THAN ONE commit. With exactly one it uses that commit's message. A
  title-only reading therefore misses a single-commit `fix(rust)` under a benign title.
  The gate now takes the union of claims -- title and every commit subject -- against the
  union of the diff, which costs nothing: a branch may claim a fix in one commit and carry
  its source in the next.
- A prose comment inside the `SOURCE_SCOPES` brackets silently widened the rule, because
  the extractor could not tell a definition from a sentence about it. Line comments are
  stripped before parsing now.
- `git diff --name-only` C-quotes any path with a non-ASCII character, so the extension
  test would have read a trailing quote as part of the extension. `-z` and NUL splitting.
- Re-measured after every change: 498 commits, 1 refusal, and the refusal set is identical
  to the first form's. Two readers, 0 disagreements, 0 could-not-run.
