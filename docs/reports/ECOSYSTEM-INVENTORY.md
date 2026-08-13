# Ecosystem inventory and unification plan

**Generated:** 2026-08-13 (W655) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 0. Two corrections to this project's own reports, before anything else

**(a) `gHashTag` is a User account, not an Organization.**

```
$ gh api users/gHashTag --jq .type
User
```

Earlier reports in this session — including `docs/reports/ISSUE-REGISTRY.md`,
committed one hour before this file — say "across the gHashTag **organisation**".
That is wrong. It affects tooling: org-scoped API endpoints, team permissions and
`--owner` semantics differ for a User.

**(b) The account holds 219 repositories, not 100.**

```
$ gh repo list gHashTag --limit 100  | count -> 100     <- exactly the limit
$ gh repo list gHashTag --limit 200  | count -> 200     <- exactly the limit
$ gh repo list gHashTag --limit 1000 | count -> 219     <- the answer
$ gh api users/gHashTag --jq .public_repos     -> 188 public (+31 private)
```

**This session ran the `--limit 100` form and reported "100 repos".** That was the
limit, not the count — and it is precisely **T90**, which was written *after* the
error and without noticing the error had been made. The reconnaissance made the
same mistake at `--limit 200` and caught it; this document is the correction of
both.

> Every figure below comes from the `--limit 1000` set.

---

## 1. Shape of the account

| class | repos | disk |
|---|---:|---:|
| FORK (upstream, not ours) | 36 | 5.12 GiB |
| DEAD (untouched > 6 months, superseded) | 86 | 3.52 GiB |
| OTHER (retired product lines) | 42 | 4.22 GiB |
| **TOOLING** | 18 | 3.00 GiB |
| **CORE-HW** | 10 | 0.69 GiB |
| **CORE-NUMERIC** | 8 | — |
| **CORE-NET** | 7 | — |
| **PAPER** | 6 | — |
| **CORE-MODEL** | 5 | — |
| **total** | **218–219** | **17.92 GiB** |

**The ternary ecosystem is roughly a fifth of the account by count and a quarter
by disk.** The rest is a decade of retired product lines plus upstream forks.

**Merge candidates: 39 repos, 4.38 GiB** — verified by two independent scripts
returning the same `n=39, KB=4,589,509`.

---

## 2. ⚠ CRITICAL — `trinity` and `trinity-fpga` are one codebase with two live heads

Verified here by a route the reconnaissance did not use (HEAD cross-probe rather
than 100-commits-back):

```
shared root      bfd4d06ada47  2026-01-31T06:54:10Z
                 "Initial release: Trinity VSA library v0.1.0"

trinity      HEAD fa66dcf70850  ->  in trinity-fpga:  HTTP 422 "No commit found"
trinity-fpga HEAD f4e361a3da1d  ->  in trinity:       HTTP 422 "No commit found"

commits          trinity 5,801        trinity-fpga 6,771
disk             723,851 KB           713,442 KB
last push        2026-08-13 13:25     2026-08-13 10:02
```

**Neither head is reachable from the other, and both are still being pushed to
today.** `trinity-fpga` has ~970 more commits while being ~10 MB smaller on disk.

> This is an **active data-loss hazard**, not a tidiness problem. Work landing in
> one will never reach the other, and the divergence grows every day. Any future
> reconciliation is a manual merge of two 6,000-commit histories with a shared
> root and no common ancestor after it.

**This must be resolved before any monorepo work begins.** Unifying a codebase
that has two disagreeing definitions of itself would silently pick one.

---

## 3. Other operational hazards found

| finding | why it matters |
|---|---|
| **11 repos have ZERO commits** — `trios-t27`, `mcp-dev`, `zig-vsa`, `zig-half-f16`, `zig-half-rust`, `zig-half-base`, `zig-half-lib-new`, `zig-half-lib-v1`, `zig-half-lib`, `go-half-rust`, `go-half-lib` | eight are near-duplicate names of live repos; a search by name finds the empty one |
| **`tri` is a Tailscale Funnel CLI**, not the `./scripts/tri` pipeline CLI that `CLAUDE.md` and **L7** mandate | an agent told to "use `tri`" can install or invoke the wrong binary |
| `tri` and `tri-tunnel` are **~84% the same file** (44 changed lines of 270), pushed 2.5 h apart, with no marker saying which is canonical | no way to tell which to use |
| **`trinity-agents` and `trinity-physics` are README-only stubs** whose descriptions promise implementations that do not exist | the real physics is in `zig-physics` (588 MB); the stub is found first by name |
| **`openFPGALoader` is forked into the account** while `CLAUDE.md` forbids using it | the SSOT *requires* it for the measured `0403:6014` cables — the fork invites the mistake `CLAUDE.md` bans, and `CLAUDE.md` is the stale doc |
| **The GoldenFloat result is spread across 5 publication repos and 4 implementations** | no single source of truth for the project's headline numeric claim |

---

## 4. What actually ports to `.t27`, and what must not

**Ports cleanly — this is the numeric layer, and it is what `.t27` exists for:**

- GoldenFloat's **GF4…GF1024 ladder** (`zig-golden-float`, 175 commits)
- the **GF16 dot-product / 4×4 matmul kernel** — the load-bearing hardware kernel,
  appearing in `trinity-fpga`, `tt-trinity-gf16`, `tt-trinity-max`, `tt-trinity-mini`
- **f16/bf16 ternary pack/unpack** (`zig-half`, `go-half`, `zig-half-rs`)
- `tt-lang-t27`'s conformance vectors and `tt-trinity-corona`'s **77-format ROM** —
  the natural first migrations, because **L6** already names
  `FORMAT-SPEC-001.json` + `gf16.t27` as numeric SSOT

**Must NOT be forced into `.t27`, and an attempt would be actively harmful:**

- **git topology and repo lineage** — the `trinity`/`trinity-fpga` divergence is a
  VCS fact, not a spec fact
- **the Tiny Tapeout submission contract** — `info.yaml` manifests, `gds.yaml`
  builds, sky130/GF180 PDK pinning and thermal gates are foundry-shuttle CI, and
  a spec cannot own them
- **LaTeX publication state**, especially `arith2027-goldenfloat`'s **double-blind
  pre-registration**, whose entire value is that it lives in an anonymous state a
  monorepo would destroy

> **The unification target is the numeric and RTL layer, not the account.**
> A monorepo that swallows the tapeout contracts or the blind submission destroys
> properties those artefacts exist to have.

---

## 5. Proposed layout

```
t27/                      <- the existing repo, already the spec SSOT
 ├─ specs/
 │   ├─ numeric/          <- gfternary.t27, tnf17.t27, gf*.t27  [GF ladder lands here]
 │   ├─ ternary/          <- gft_* datapath family
 │   ├─ igla/race/        <- phi_weights.t27, systolic, gemm
 │   ├─ igla/coder/
 │   └─ fpga/             <- ternary_link.t27, uart, boards
 ├─ conformance/          <- FORMAT-SPEC-001.json + vectors  [L6 SSOT]
 ├─ fpga/                 <- hand-written top wrappers, XDC, bitstreams
 └─ docs/theory/          <- IGLA-FORMAL-RESULTS.md, TNF_ARTICLE_RU.md
```

**Stays separate, deliberately:**

| repo | reason |
|---|---|
| `tt-trinity-*` (8 tapeouts) | each owns a foundry-shuttle CI contract |
| `arith2027-goldenfloat` | double-blind pre-registration |
| `trinity-papers-ru`, `phi-paper`, `goldenfloat-preprint` | publication state, own review cycles |
| all 36 forks | upstream, not ours |

**Order of operations, and the first step is not a migration:**

1. **Resolve `trinity` vs `trinity-fpga`.** Nothing else can start.
2. **Delete or mark the 11 empty repos** and the 2 stubs, so name searches stop
   finding them first.
3. **Migrate the GF ladder** into `specs/numeric/`, one rung at a time, each with
   the `test`/`invariant`/`bench` obligation **L4** requires.
4. **Point the conformance vectors at the migrated specs**, so `L6`'s SSOT claim
   becomes checkable rather than declared.
5. **Only then** consider the RTL cores.

---

## 6. The training-database question, stated rather than answered

The mission asks for the unified project to be *simultaneously* the training
database for IGLA CODER and IGLA RACE. **Those two purposes conflict**, and the
conflict should be named before it is designed around:

- A **monorepo** wants one history, one CI, one review gate, and small diffs.
- A **training corpus** wants many independent samples, deduplication guarantees,
  provenance per sample, and a licence that permits redistribution.

`dataset/igla-coder/v0.1/DECONTAM.md` already cites deduplication methodology,
which means the corpus side has started and the monorepo side has not.

> **The honest framing:** the monorepo is the *source*, and the training corpus is
> a *derived artefact built from it by a documented extraction*, with its own
> provenance stamps. Making them the same object means every training-set change
> is a source-tree change, and every source refactor silently rewrites the corpus.

---

## 7. What this document does not establish

- **The class assignments are machine-generated from names, descriptions and
  timestamps.** No repository was read. "DEAD" means untouched and superseded by
  those signals, not verified obsolete.
- **Disk figures are GitHub's `diskUsage`**, which includes history and is not the
  size of a checkout.
- **The 39 merge candidates were not tested for build compatibility**, licence
  compatibility, or duplicated symbol names.
- **No repository was modified.** This is an inventory and a proposal.

**φ² + φ⁻² = 3 | TRINITY**
