## a repair reveals the defect standing behind it

`coq-proofs.yml` had never passed. The cause was exact and the log stated it:
`options: --user root` runs the container as root, the coqorg image initialises
opam for the `coq` user under `/home/coq/.opam`, so `OPAMROOT` defaulted to
`/root/.opam` and opam reported `[ERROR] Opam has not been initialised`, exit 20.

Setting `OPAMROOT` fixed that. The job then failed anyway, in the same step, on
something else entirely:

```
[ERROR] Package conflict!
  * Missing dependency:
    - coq-interval = 4.9.0 -> coq < 8.19~
No solution found, exiting
```

The pin asks for a package requiring a Coq **older than the image it runs on**
(`coqorg/coq:8.19`). It could never have installed. But opam never got as far as
computing a solution, so the conflict had never once been printed in this
repository's history.

The rule: **a workflow that fails early hides every defect after the first**, and
the count of defects is not knowable until the first is repaired. "Fixed" and
"green" are different claims. Reporting the first as the second is how a repair
gets recorded as done while the job stays red.

The corollary for effort: the second failure is not evidence the first diagnosis
was wrong. Here the first fix demonstrably worked — opam went from refusing to
start to synchronising both repositories — and the reward for that was a new,
truer error message.

Related: [[a-trigger-that-omits-its-own-file-cannot-verify-its-own-repair]].
