# NOW — the gate said "all cases passed" one line below its own MISMATCH

Last updated: 2026-08-22

## Add pipefail to the conformance execution step (Closes #2415)

- Branch: `fix/2415-conformance-pipefail`
- Issue: #2415 · lands on top of #2403

### Что легло

One line in `.github/workflows/fpga-build.yml` — `set -o pipefail` at the top of the
"Execute conformance vectors" step — plus the comment explaining why.

`if python3 tools/run_conformance_vvp.py ... | tee ...; then` tests **tee's** exit status
without it. A failing runner took the `then` branch, `fails` stayed 0, and the `exit 1`
below never fired. The step has no `shell: bash` and the workflow has no `defaults:` block,
so the default `bash -e {0}` applies and `-o pipefail` is not on.

**This is the defect #2242 removed from `fpga-formal`**, in the same file, 115 lines below
the corrected pattern at `:638-640`.

### Границы честности (BINDING)

- **The rest of #2403 is untouched and is good.** Registry-scoped execution with the
  remainder printed as visible debt is the honest shape. This changes one line.
- **This does not establish that the vectors pass.** It establishes that if they stop
  passing, the job will say so. Whether `fpga-conformance` is green for the right reason is
  now measurable; before it was not.
- **The planted-fault control in #2403 validated the runner, not the gate.** Different
  claims. The control should be re-run *through the workflow step* so it covers the gate;
  not done here.
- Raised on #2403 before it merged and merged unchanged. Context, not complaint — the
  review landed close to the merge.

### Evidence

Stub runner failing the way a real mismatch would:

```
echo "case 3: MISMATCH expected 0x2a got 0x00"; exit 1
```

As landed on master:

```
case 3: MISMATCH expected 0x2a got 0x00
| mac | EXECUTED, all cases passed |
fails=0                                    <- step exit 0, job green
```

With `set -o pipefail`, same runner:

```
| mac | FAILED |
fails=1
::error::conformance execution failed      <- step exit 1
```
