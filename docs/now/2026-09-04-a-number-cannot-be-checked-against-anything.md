# NOW -- A number is the one part of a pull request that cannot be checked (2026-09-04)

## The lander was handed a stranger and would have merged it

- A serial lander was queued with **#3160** where `gh pr create` had printed **#3161**. #3160 is
  *"feat(tri): tri merging"* -- **another session's work in the same repository** -- and it was
  queued to merge on a green verdict. Caught by hand, before it ran.
- **The obvious guard does not work here.** An author check catches nothing: every session
  authenticates as the same GitHub user, so both pull requests read as mine. What differs is the
  **head branch**, and the caller always knows which branch it just pushed.
- `tri pr ready --expect-branch <name>` refuses **before reading a single check**, with exit **6**,
  naming both branches. Reproduced against the real near-miss:

      gHashTag/t27#3160 — NOT the pull request you meant.
        expected head branch: w-workflow-listing
        this PR's head branch: loop/merge-in-flight

- Opt-in: without the flag nothing changes, and a test asserts that absence is not a refusal.
  Five tests, three mutants, three dead -- including one that accepts a prefix, because `w-tri`
  must not match `w-tri-status`.

**A PR number is an identifier, not a computed value.** In a repository several sessions are
pushing to, the number next to yours belongs to somebody else, and a lander that merges on a green
verdict will merge whatever number it is handed.

Refs #3157
