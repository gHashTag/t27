# Ring 1 test fixtures (planned)

This directory is reserved for **first-class `.t27` fixtures** executed by the **Rust runner** once `tri run` / `t27c run` (or equivalent) exists.

- Charter and issue spine: **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`** (Stage 1, #11–#25).  
- Oracle vocabulary: **`docs/nona-03-manifest/GOLDEN-CHAIN-TESTING-ATLAS.md`**.

Until the runner lands, keep fixtures out or add only files that are **explicitly consumed** by `t27c suite` / codegen paths. Do not reintroduce shell harnesses under `tests/`.
