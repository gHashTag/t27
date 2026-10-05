# NOW -- L2 reads a regenerated gen/ file as a regeneration (2026-10-05)

## A modified gen/ file passes when t27c reproduces it byte for byte (Closes #6226)

- `.github/workflows/l1-traceability.yml` failed every `M gen/...` line, so #6218 and #6246 (spec changed, `gen/c/tri/t27b/steward.c` regenerated with `t27c gen-c`) went red and L2 was being read as noise.
- New `tools/l2_regen_check.py`: for each modified `gen/<backend>/<path>.<ext>` it runs `t27c gen-<backend> specs/<path>.t27` and compares bytes. Equal passes; different bytes, no spec, unknown backend or a t27c error fail. No t27c prints "not checked" and fails.
- The L2 step now only lists modified gen/ files. A cargo cache and `cargo build --release -p t27c` run only when that list is non-empty, so a PR that does not touch gen/ pays nothing.
- Mutation control (local t27c, scratch worktree): spec changed + regenerated -> pass; the same plus one hand-edited byte -> fail at byte 108; a code-byte hand edit with the spec untouched -> fail; no t27c -> not checked, fail; `gen/c/vsa/core.c` (no `specs/vsa/core.t27`) -> fail.
- Census re-blessed in the same commit: shell `run: steps` 286 -> 287 (the new build-and-compare step); quiet `failure branch passes` 15 -> 14, because the old `git diff ... 2>/dev/null | grep ... || echo ""` that passed when git diff failed is gone.
- Second defect, same step: the list was `git diff origin/master..<PR head>` (two dots), so a branch cut before master changed a gen/ file was charged with master's change (#6247 failed on c5406e6b8 while touching no gen/ file). It is now `origin/master...<PR head>`, from the merge base. Control: a stale branch touching no gen/ file lists `M gen/c/tri/t27b/steward.c` with two dots and passes with three.
