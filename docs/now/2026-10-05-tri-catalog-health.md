# NOW -- tri catalog health, decided in t27 (2026-10-05)

## specs/tri/catalog/health.t27 and tri catalog health (Closes #6436, Refs #6434)

- The SPECS catalog counts every warn and fail alike. `specs/tri/catalog/health.t27` holds the split the t27b loop needs: negative fixtures (`bootstrap/tests/fixtures/**`, `neg_*`, `damage_*`, `eof_*`, as str consts) versus real work, parse-fail (no AST nodes) versus backend-only, and the change of one spec between two catalog builds.
- `tri catalog health` reads https://t27.ai/t27/manifest.json and decides only through `gen/c/tri/catalog/health.c` (gen-c on the t27c lab at master 682564f48); flags `--repo`, `--list`, `--json`, `--diff OLD`.
- On the live catalog: ok 1370, warn 232, fail 176; broken 408, of which 24 are negative fixtures, 146 real parse-fail and 238 real backend-only. The catalog was built from bf7d8eea9, 503 commits behind origin/master.
- `tri t27b gaps` was not added: `tri t27b next` already prints the sole/first blocker counts among specs where the reference passes and t27b is blocked.
