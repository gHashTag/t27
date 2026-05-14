# Security policy

## Reporting vulnerabilities

Report sensitive issues **privately** to the maintainers (GitHub Security Advisories for this repository, or contact the **primary maintainer**: **Dmitrii Vasilev** — [ORCID 0009-0008-4294-6159](https://orcid.org/0009-0008-4294-6159), [github.com/gHashTag](https://github.com/gHashTag)). Please do **not** open public issues for undisclosed credential leaks.

## Compiler / CLI threat model (summary)

- **Input:** Untrusted `.t27` files and conformance JSON should be treated as **untrusted input** until parser hardening and fuzzing reach release-grade (see `docs/STATE_OF_THE_PROJECT.md`).
- **Output:** Generated Zig/C/Verilog must be reviewed before deployment in safety-critical or networked paths.
- **Secrets:** API keys and tokens belong in **local** `.env` (gitignored) or host secret stores — **never** in the git tree.

## Incident: committed `.env`

If `.env` was ever tracked with real keys, **rotate those credentials immediately**; git history may retain them until rewritten (e.g. `git filter-repo`). After rotation, use `.env.example` only as a **name template** without live values.

## Supply chain

Release artifacts should eventually publish SBOM and signed builds (see `docs/REPOSITORY_EXCELLENCE_PROGRAM.md`, P2). CI currently enforces build, parse, codegen, conformance, and header checks — not yet full SLSA.
