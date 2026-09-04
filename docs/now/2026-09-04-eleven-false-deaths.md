# NOW -- Eleven false deaths, and the gate that already said why (2026-09-04)

## `tri` resolves through four surfaces; I asked one (Refs #2994)

- audited this file's own claims: which `tri <verb>` spellings still exist. Built the list from `./target/debug/tri --help` on a binary rebuilt at master (48 verbs) and found **11** the file names that are absent. **All eleven exist. The number is entirely mine**
- `tri` resolves through **four** surfaces: bash case arms in `scripts/tri`, `scripts/tri_loop/*.py` (dispatched BEFORE the binary), the Rust binary, and `t27c` via the forward-anything fallthrough
- `./scripts/tri claims` prints `names carrying a strong word: 18`; `./scripts/tri damage specs` prints `files scanned: 650`, and `loop-tools-gate.yml` **runs it in CI**
- **the repository already gates this** on every PR and on master, and its header says: *"Missing the last one alone would report 155 false deaths, so the checker refuses rather than guessing"*. `tools/check_documented_commands_exist.py` reads `README.md`, `docs/**` and `.claude/skills/**` -- this file -- so my sweep was already running, wider, with the refusal I lacked
- **second time in one pass:** the poll-and-merge loop was a worse copy of `tri pr ready --wait --merge`, dropping its refusal to score a check with `conclusion: null` AND `state: null`. **Both rewrites dropped the enumeration of sources** -- one field or four surfaces, I asked one and read its answer as the whole
- **two other classes clean, numbers recorded so nobody re-runs them:** 170 distinct file paths across 143 sections, **0** dangling (8 abbreviations, 17 unresolved are examples, a quoted wrong path, another repository, or the subject itself); 7 "`tri X` exits N" claims, **0** false, each dated to its own repair
- **nothing shipped, and that is the result** -- a gate over a population measured at zero watches an empty set
