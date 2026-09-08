## a trigger that omits its own file cannot verify its own repair

`coq-proofs.yml` listed itself in `push:`:

```yaml
  push:
    paths:
      - 'proofs/trinity/**.v'
      - '.github/workflows/coq-proofs.yml'
  pull_request:
    paths:
      - 'proofs/trinity/**.v'          # <-- and NOT itself
```

The two triggers disagreed, and only the `pull_request` side matters to someone
fixing it: **a PR that repairs this workflow does not run it.** Any repair
therefore ships unverified, is reported as done, and the workflow stays broken.
That is a plausible account of how this one stayed red from August.

Adding the file to `pull_request` paths is what made the next PR self-checking —
and it immediately earned its keep by proving the first repair incomplete rather
than letting it merge as finished.

This is the same shape as a gate whose `paths:` is narrower than its subject,
with one extra turn: the subject here is *the workflow itself*, so the omission
specifically disables the case where someone is trying to fix it. The failure
mode is silent and self-perpetuating.

Check both triggers list the workflow file whenever either does. A disagreement
between `push:` and `pull_request:` paths is the signature.
