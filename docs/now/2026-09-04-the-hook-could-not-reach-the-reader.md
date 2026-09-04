# NOW -- The hook could not reach the reader that works (2026-09-04)

## The zero was my shell

- Pricing "what did the unrun previews cost", I counted failures in the four required contexts over
  30 merged pull requests and got **0**. Clean, and false: `for s in $shas` -- **zsh does not
  word-split an unquoted variable**, so four commits became one iteration and every count came from
  a mangled sha. At least the fourth time this trap has been hit, and it is in my own notes.
- **What caught it was refusing the zero.** #3182 had failed `check` and `check-now-freshness` an
  hour earlier, so a window containing it reporting zero is refuted before it is explained. Rewritten
  as `while IFS= read -r s`, the control reproduces exactly `check,check-now-freshness`.
- **A zero deserves a positive control in proportion to how much you would like it to be true.**

## Corrected: 1 of 30, and the tool already existed

- **1 of 30 merged PRs (3%)** had a failure in a required context -- two check-runs, both on my own
  #3182. That is the whole measured cost.
- So `tri preflight`, a fourth tool to call the other three, is **declined on the measurement**.
  `tri hooks pre-commit` already runs the migrated gates AND the shape check, needs only the `tri`
  binary, and takes about a tenth of a second. On the exact planted entry it exits **1** with the
  gate's own words: *"no `- ` bullets: the entry states nothing"*.

## Three facts behind the 3%, and only the third is a defect

- **No hooks are installed in this clone.** `core.hooksPath` empty, `.git/hooks/pre-commit` absent,
  `.githooks/pre-commit` executable and uninvoked. Eighty passes of commits with no local gate.
- **The hook could not reach the reader that works.** It calls `scripts/tri check-now`, which needs
  `t27c`; in a checkout whose work is in `cli/tri` the compiler is unbuilt, so it exits 2 and
  correctly refuses -- which means the hook cannot be installed there at all. It now prefers
  `tri hooks pre-commit` when a `tri` binary exists and falls back unchanged when none does, so the
  exit-2 refusal still stands. Controls: bad entry with `tri` -> **1**, no `tri` -> **2**, good
  entry -> **0**.
- **Not established:** that the unusable hook is *why* nobody installed it. The unusability is the
  fact; the causation is a story, and the hook's comment says which is which.

**Ask in order before writing another preview:** is it installed, can it run here, does it cover
this context. All three answers were already in the repository.

Refs #3176
