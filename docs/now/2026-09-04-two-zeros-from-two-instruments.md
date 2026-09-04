# NOW -- Two zeros from two instruments (2026-09-04)

## Both would have read as a clean tree

- **`git grep -E` knows no Perl escape.** `-cE '^\s*assert true\s*$'` returns **0**; the same
  population is **2247** under `-E '^[[:space:]]*assert true$'` and under `-P`. A section above
  says `\b` is not a word boundary there -- true, and too narrow. **A rule stated about one escape
  does not carry to its siblings.**
- **zsh eats `:t` out of a path.** `git show $c:tests/ring0_trivial.t27 | grep -c '^test'`
  returning `0 0 0 0 0` was offered as proof a file never had tests. In zsh `$c:t...` applies the
  tail modifier -- `c3356a4a6ests/...` -- `git rev-parse` fails, and the pipe renders the failure
  as `0`. bash expands it correctly. Quote it. **The shell is part of the instrument.**
- **The audit of our own gates found no hole, which is the result.** 20 `tools/check_*.py`, 10 of
  them matching text, **19 with a `--self-check`**. The one without it, `check_sync_repo_root.py`,
  is the best-defended: it **fails closed**, returning 2 when its matcher finds nothing. It now has
  a self-check too, for the matcher rather than for the corpus.
- The distinction a self-check does **not** confer: constructed inputs prove a matcher can
  distinguish; only failing closed proves it read anything. A ratchet gets that free -- a silently
  broken matcher collapses the count and the baseline reports a massive shrink.

Refs #3141
