# `origin/master` was replaced by an orphan commit — evidence, W971 (2026-08-21)

Recorded because the local reflog that proves this **expires**, and after that the event
is unreconstructible from this clone.

## What happened

| | commit | commits in history | author | timestamp |
|---|---|---|---|---|
| **master, before** | `ca4234e20` | **2545** | Vasilev Dmitrii | 2026-08-21 18:00:32 +0700 |
| **master, now** | `fead099c2` | **1** (it is also the root) | Vasilev Dmitrii | 2026-08-21 18:48:37 +0700 |

`git merge-base origin/master HEAD` exits **1**: the two histories are **unrelated**. The new
master is a fresh orphan root, not a rewrite of the old one — its message is
`fix(coq): PhiFloat.v — bare B2R/B754_finite resolved to BinarySingleN`.

Detected by `git status` reporting divergence jump from *126 / 117* to *2556 / 1* between
W969 and W970.

## Nothing is lost

The previous 2545-commit history is **still on the remote**, reachable from
`origin/fix/coq-phifloat-binary64-name-collision` (2547 commits, contains `ca4234e20`).

This branch — `claude/igla-fpga-improvements-3f5e1a`, 2556 commits — is **intact and pushed**;
it diverges from its own `origin` ref by 0/0. No wave's work is affected. The upstream
research repository `gHashTag/trinity-fpga` is a different remote and is untouched.

## Recovery, for whoever decides to do it

This is a destructive operation on shared state and is **deliberately not performed by the
autonomous loop**. It also discards the new commit unless that is preserved first.

```bash
git fetch origin
git branch orphan-master-fead099c2 origin/master        # keep the new commit
git push origin ca4234e20:refs/heads/master --force-with-lease
```

`--force-with-lease` rather than `--force`: if anyone else has moved master since this file
was written, the push must fail rather than repeat the event.

## W972: restore attempted with the owner's authorisation, and refused by the repository

The owner authorised the restore. The order was: **preserve first**, then attempt.

1. `fead099c2` was pushed to **`origin/orphan-master-fead099c2`** and verified present —
   the new commit is now recoverable independently of `master`.
2. `git push origin ca4234e20:refs/heads/master --force-with-lease` was then **rejected**:

   ```
   ! [remote rejected]  ca4234e20 -> master (push declined due to repository rule violations)
   ```

**The repository's own protection rules forbid it, and the loop did not attempt to bypass
them.** The restore needs someone with bypass rights — via the GitHub UI, or by relaxing the
ruleset for one push.

Because preservation completed and was verified before the destructive step, the failed
attempt left the repository exactly as it was, plus one recoverable branch.

## Why the loop did not fix it unilaterally

Restoring a shared branch is outward-facing and irreversible for anyone who has already
fetched. The loop's part is to **notice, preserve the evidence, and name the exact remedy** —
which is what this file is.

---

*φ² + φ⁻² = 3 | TRINITY*
