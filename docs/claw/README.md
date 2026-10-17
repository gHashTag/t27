# TRI CLAW

TRI CLAW — TRI-27 — Trinity S3AI: every CI gate of this repository, packaged
behind one front door, with the gate logic in `.t27` specs. Epic
[gHashTag/t27#9271](https://github.com/gHashTag/t27/issues/9271).

## What exists today

| Piece | Where | Status |
|---|---|---|
| Gate inventory (SSOT) | `specs/claw/gates.t27` | T1, merged |
| Drift audit law | `claw_status` in `specs/claw/gates.t27` | T2, this branch |
| Drift audit in CI | `gate-topology` workflow, "claw drift" step | T3, this branch |
| GitHub App manifest | release step, see `docs/claw/RELEASE.md` | T4, deferred to App setup |
| Service skeleton | Railway, thin I/O glue only | T5, merged (spec contract) |
| Positioning / pricing | `docs/claw/competitors.md` | T6, started |

## The inventory spec and how to regenerate it

`specs/claw/gates.t27` holds three parallel `[67]str`-shaped arrays — `NAMES`
(workflow file stems), `CLASSES` (0 blocking / 1 soft / 2 reporting), `EVENTS`
(trigger keys, comma-joined). Rows stay alphabetical and aligned; tests pin
the alignment, the census (62/3/2), pairwise name uniqueness, and the
merge-critical rows.

The class rule is mechanical, so the regeneration is too:

- **soft** = the workflow file contains `continue-on-error`
  (`grep -l 'continue-on-error' .github/workflows/*.yml`)
- **reporting** = the workflow only merges / opens / files work — today
  `auto-merge-ready-prs` and `queen-publish`, the same two files
  `scripts/ci/check_pr_branch_filters.py` records as NOT_MERGE_CRITICAL for
  exactly that reason
- **blocking** = everything else

Events extraction (run from the repository root):

```sh
cd .github/workflows && for f in *.yml; do
  n="${f%.yml}"
  ev=$(awk '/^on:/{p=1;next} p&&/^[A-Za-z_]+:/{p=0}
            p&&/^  [a-z_]+:/{gsub(/:.*/,"");gsub(/^  /,"");printf "%s,",$0}' "$f" | sed 's/,$//')
  printf '%s|%s\n' "$n" "${ev:-NONE}"
done
```

A workflow file that lands in `.github/workflows/` without its row in the
inventory (or vice versa) is an inconsistency by construction: `claw_status`
in the spec is lowered by `t27c gen-c` into `gen/c/claw/gates.c`, and the
claw-drift step of the `gate-topology` workflow compiles a four-line harness
around it that diffs `NAMES` against the directory and fails on any unknown
name — mirroring the ceiling discipline of
`scripts/ci/check_pr_branch_filters.py`. The law is the spec; the harness is
machinery (owner rule 2026-10-13: everything new is .t27).

## Why this shape (own language first)

The gate logic lives in `.t27` because that is this project's language and the
whole claim of the service: gates as a **tested spec**, not a vendor blacklist
or a foreign DSL. Everything else — the GitHub App wiring, the Railway
service, the CLI plumbing — is thin I/O glue around the spec. See the
`own-language-first` rule in the repository instructions.

S3AI is written "S3AI" here (ASCII, L3) — the wordmark with the superscript
lives where superscripts are safe.
