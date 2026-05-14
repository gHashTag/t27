# Reproducibility entrypoints

One-command targets for reviewers and CI spot-checks. Run from repository root:

```bash
make -C repro repro-smoke
```

| Target | Intent |
|--------|--------|
| `repro-smoke` | Bootstrap build + full `tests/run_all.sh` + conformance JSON sanity + gen header check |
| `repro-language` | `cargo build --release` + `t27c compile-all` (canonical `gen/zig`) + gen headers |
| `repro-numerics` | Conformance validation + pointer to `conformance/gf*_vectors.json` |
| `repro-ar` | Same conformance gate + pointer to `conformance/ar_*.json` |
| `repro-paper-figures` | Placeholder until paper figure scripts are pinned under `repro/paper/` |

**Toolchain:** Pin Rust via `bootstrap/rust-toolchain.toml` (if present) and document OS in `docs/STATE_OF_THE_PROJECT.md`. Full container digest matrix is **P1** in `docs/REPOSITORY_EXCELLENCE_PROGRAM.md`.

See also `docs/EXTERNAL_AUDIT_PACKAGE.md` and `docs/RESEARCH_CLAIMS.md`.
