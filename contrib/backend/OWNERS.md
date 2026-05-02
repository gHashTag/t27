# OWNERS — contrib/backend/

## Primary

**B-Builder** — build, deploy, and service images (API, agent-runner, sandbox).

## Dependencies

- `specs/`, `conformance/` — behavior SSOT for product semantics (read-only from this tree).
- `.github/workflows/deploy-api.yml`, `agent-runner-docker.yml`, `sandbox-docker.yml`.

## Outputs

- Container images (GHCR), Railway-compatible API bundle under `api/`.

## Related agents

**R-Reasoning** may coordinate CLARA / AR flows that call these services; **E-Evidence** owns conformance vectors consumed by runners.
