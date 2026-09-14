## A private target dir hides the binary from the hooks that guard the commit

Two loop ticks were sharing one checkout and one `target/`. The repair -- a private `git worktree`
with its own `CARGO_TARGET_DIR` -- fixed the flip-flopping readings and broke committing.

`git commit` printed `t27c is not built ... Nothing about your change was checked` and exited
non-zero. **The commit did not form.** The next `git push` reported success: it pushed the
unchanged `HEAD`, publishing master's commit under my branch name.

`.githooks/pre-commit:52` and `.githooks/commit-msg:14`:

```
for cand in "$ROOT/target/debug/tri" "$ROOT/target/release/tri"; do
```

The hooks probe the DEFAULT build directory. Isolation moves the binary out of their view.

- Fix without a rebuild: `mkdir -p target/debug && ln -sf "$W/target-s2/debug/tri" target/debug/tri`
  (`target/` is gitignored -- confirm with `git check-ignore -q target`).
- **Check that the commit formed**, never that the command printed something:
  `[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/master)" ]`. This is the second distinct
  cause of "commit did not form while push said pushed"; the causes differ, the shape repeats.
- The hooks also warn that `target/debug/tri` is older than `cli/tri/src`. That means the gates run
  are the ones the OLD binary carries -- a stale ruler, reported as a pass.
