# NOW -- The shape reader listed the index and judged the working tree (2026-09-05)

## The shape reader listed the index and judged the working tree (Refs #3303)

- Reproduced before acting. Stage a MALFORMED docs/now entry, then fix the working copy:
  the gate prints `ok <path>` and exits 0, and `git commit` takes the malformed one.
- The population came from `git diff --cached`; the content came from disk. Listing one
  operand and judging another is the same structural error found in four other gates today.
- `--from-index` added: when the paths came from the index, the bytes come from
  `git show :<path>`. Verified: the same probe now exits 1 naming `'garbage,` -- the
  INDEX's first line, not the working tree's corrected one.
- `-z` added to both path listings, in the Rust reader and the Python one. `--name-only`
  C-quotes any path with a non-ASCII character, and the quoted form does not end in `.md`,
  so such an entry left the population silently and the gate reported that the change adds
  none.
- The control for that one has to be read carefully. A Cyrillic filename is refused either
  way, because `FILENAME` requires an ASCII slug and always did. What changed is that the
  entry is now SEEN and judged by the repository's own rule, where before it was invisible
  and the gate said "no entry was added" while passing at exit 0. Visibility was the
  defect; the naming rule was never in question.
- Third time today a probe measured the wrong tree: a scratch worktree created from HEAD
  does not carry uncommitted edits, so it ran the old script and reported the fix as
  failing. Copying the edited files in is what settled it.
