# TRI-NET API for External Integration

> **Status:** SPEC (draft). Interface shape only. No runtime endpoint is
> hosted from this repo and none is promised. This document specifies the
> *shape* of an integration interface that external tools may rely on when
> they consume TRI-NET artefacts (specs, conformance vectors, NMSE
> manifests, format registry, seal hashes).
>
> **Source of truth:** `specs/api/tri_net_api.t27` and
> `schemas/tri-net-api-v1.json`. If this Markdown disagrees with either,
> the spec / schema wins.

---

## 1. What this API is, in one paragraph

TRI-NET exposes a small set of **read-only artefacts** that downstream
tools (chip-repo bring-up scripts, CI gates, third-party auditors) can
consume programmatically. The interface is **file-based, not network-based**:
the canonical "TRI-NET API" is a contract over the JSON files that live in
this repo (and in sibling chip repos that follow the same schema). There is
no proprietary REST surface, no SDK, and no service in scope here.

This is deliberate. R5-HONEST forbids us from describing a hosted runtime
we do not actually operate.

---

## 2. Versioning

- Schema version follows semver: `MAJOR.MINOR.PATCH`.
- `MAJOR` change is a breaking change; consumers must opt in.
- A consumer SHOULD reject an artefact whose schema major does not match
  what the consumer was written against.
- The current schema is `1.0` (see `schemas/tri-net-api-v1.json`).

---

## 3. Artefact families

Each family below has a canonical filename pattern and a canonical schema
fragment. A TRI-NET-conforming artefact lives at a path matching the
pattern and validates against the schema.

### 3.1 Format registry

- **Path:** `conformance/FORMAT-SPEC-001.json` (this repo is the canonical
  host of the registry; chip repos consume, do not republish).
- **Schema:** `schemas/numeric-format-v1.json` (existing, unchanged).
- **Stability:** L6 CEILING -- only changed via constitutional amendment.

### 3.2 NMSE protocol results

- **Path pattern:** `bench/results/nmse-<runner>-<seed>-<isodate>.json`.
- **Schema:** `schemas/nmse-protocol-v1.json` (new, see
  `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`).
- **Stability:** SPEC. A new minor field is non-breaking; renaming an
  existing field is breaking.

### 3.3 Toolchain seal hash

- **Path:** `bootstrap/stage0/FROZEN_HASH` (plain text, single SHA-256
  hex digest).
- **Stability:** GREEN -- defined by the constitutional stack
  (`AGENTS.md` section 3, item 6).

### 3.4 Readiness ladder

- **Path:** `STATUS.md` (human-readable) with table cells one of
  `SPEC`, `RTL`, `SIM`, `SYNTH`, `GDS/TAPEOUT`, `SILICON`.
- **Programmatic mirror:** OPTIONAL `bench/readiness.json` -- when present,
  conforms to schema fragment `tri-net-api-v1#/$defs/Readiness`.
- **Stability:** SPEC. Programmatic mirror is opt-in.

### 3.5 Conformance vectors

- **Path pattern:** `conformance/*.json`.
- **Schema:** family-specific (existing `gf*_vectors.json`, `ar_*.json`,
  `nn_*.json`, `sacred_physics*.json`).
- **Stability:** SPEC for new families; existing families are GREEN.

### 3.6 Cross-repo identity

- **Path:** `tri-net-identity.json` (optional, top-level of each repo in
  the line, including chip repos).
- **Schema:** `schemas/tri-net-api-v1.json#/$defs/RepoIdentity`.
- **Purpose:** lets a consumer enumerate the four products of the line
  without scraping documentation.

---

## 4. Consumer contract

A consumer that depends on the TRI-NET API:

1. MUST pin a schema major version.
2. MUST validate every artefact against the relevant schema fragment
   before treating its fields as authoritative.
3. MUST fail closed on schema violation; partial parsing is forbidden.
4. SHOULD verify `bootstrap/stage0/FROZEN_HASH` matches the seal recorded
   in any artefact whose `seal_hash` field is present.
5. SHOULD log the schema version in any output it produces.

---

## 5. Producer contract

A producer (e.g. a chip repo's CI job):

1. MUST emit only artefacts that validate against the active schema.
2. MUST include the schema version field at the artefact root.
3. MUST NOT add undocumented top-level fields; extensions live under a
   reserved `x_extension` object.
4. MUST cite the toolchain seal hash if numeric results are reported.
5. SHOULD include a `tri-net-identity.json` at repo root.

---

## 6. What this API is NOT

- It is not a hosted HTTP service.
- It is not an SDK with bindings shipped from this repo.
- It is not a real-time control plane for any chip.
- It does not expose any silicon-bring-up endpoint.

If a future hosted endpoint is added, it will be a separate document and a
separate contract.

---

## 7. Worked example: NMSE manifest consumer

A third-party auditor wants to verify a claimed GF16-vs-BF16 result:

1. Fetch the manifest at `bench/results/nmse-<runner>-<seed>-<date>.json`.
2. Validate against `schemas/nmse-protocol-v1.json`. Reject on failure.
3. Read `seal_hash` and compare against the repo's
   `bootstrap/stage0/FROZEN_HASH`. Reject on mismatch.
4. Read `protocol_version`. Reject if outside the accepted major range.
5. Read the per-distribution `nmse_gf16` / `nmse_bf16` pairs. Treat any
   distribution not listed in `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` section 4 as
   informational, not certifying.
6. Output is the validated, signed-by-seal table -- no further trust step
   needed.

---

## 8. Cross-links

- Whitepaper: `docs/TRI_NET_WHITEPAPER.md`
- NMSE protocol: `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`
- Numeric SSOT: `FORMAT_REGISTRY.md`, `conformance/FORMAT-SPEC-001.json`
- Readiness: `STATUS.md`
- Sibling chip repos: `LINEUP.md` (the four-product map)
- Chip-side D2D protocol (out of scope here, handled in
  `tt-trinity-euler` / `tt-trinity-gamma`).
- Roadmap: `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` -- OS-01 names the
  Python SDK target that consumes this API's schemas read-only.

---

## 9. R5-HONEST notes

- The API is file-based today; no hosted endpoint is implied.
- Schemas are conservative -- they describe SHAPE, not semantic guarantees
  about silicon behaviour.
- No throughput, latency, or energy field is part of the canonical
  surface. Those, if added later, will be under `x_extension` until
  promoted by an issue + PR + constitutional review.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
