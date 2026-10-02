# NOW -- t27c 0.4.0 — the registry crate predates its seventh backend (2026-09-21)

## t27c 0.4.0 — the registry crate predates its seventh backend (Closes #4503)

- bootstrap/Cargo.toml and .zenodo.json both move to 0.4.0, because the release pipeline's VERSION TRUTH gate refuses a tag whose manifests do not already say what the tag says — a half-bumped pair fails the tag, not the PR.
- Minor, not patch: gen-ts is added functionality and the six existing targets are untouched.
- ZENODO_DEPOSITION_T27C stays unset, so zenodo-publish.yml skips and mints no DOI.
