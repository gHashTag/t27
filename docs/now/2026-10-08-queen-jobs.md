# NOW -- a multi-step job (a release) runs in the swarm, with no session open

## specs/queen/jobs.t27 and specs/jobs/release_t27c.t27 (Closes #7676)

- `jobs.t27` defines step kinds CHECK, WAIT and EFFECT, and job states RUNNING, DONE, FAILED and
  CANCELLED.
  - No person is waited for: a wait or block ends after one day, and a failure is retried at most 3
    times.
  - An irreversible effect runs only at a pinned subject that every check passed at, and never in a
    rehearsal.
  - Only one running job per card is allowed.
- `release_t27c.t27` is the t27c release as constants: version-truth, tag-absent, github-release,
  release-workflow, crate-published. These are the steps a laptop session ran by hand for 0.5.0.
- Tests: jobs.t27 passes 6 of 6, with 41 of 41 mutants killed; the release card passes 1 of 1. The
  supervisor's vendored compiler wasm typechecks both with 0 errors.
- Not established here: the executor. gHashTag/BrowserOS wires it into the Queen's round.
