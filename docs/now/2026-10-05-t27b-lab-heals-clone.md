# NOW -- the t27b lab heals a clone that cannot check out (2026-10-05)

## lab.checkout() clones again, once (Closes #6219)

- After the owner's redeploy on 2026-10-04, the lab's first run at bae81aee2 failed at checkout: the partial clone could not fetch a blob from its promisor remote. The redeploy had killed the previous run during its reference phase.
- `checkout()` had no second try, so a broken clone on the volume could fail every later poll the same way.
- Now a failed fetch or checkout removes the clone and clones once more. `steps.checkout` records `clone: kept` or `clone: recloned` plus `first_error`, so a heal shows up in the run and is never silent.
- CI: `scripts/ci/test_the_t27b_lab_heals_its_clone.py` in loop-tools-gate, using local file:// repos only. Master's checkout fails the same broken clone (negative control, run by hand).
- Takes effect only after the next lab redeploy, which is the owner's decision.
