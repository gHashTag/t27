# Security policy

## Supported versions

Security fixes are applied to the default branch (`master`) and released through normal repository workflow. There is no separate LTS line today.

## Reporting a vulnerability

Please **do not** open a public issue for undisclosed security problems.

1. Use **[GitHub Security Advisories](https://github.com/gHashTag/t27/security/advisories)** for this repository (preferred), or
2. Contact the maintainers through a private channel they publish on the org or repo profile.

Include:

- Description of the issue and impact
- Steps to reproduce (if possible)
- Affected components (e.g. `bootstrap/`, `tri` / `t27c serve`, CI)

We will acknowledge receipt as capacity allows and coordinate a fix and disclosure timeline.
## Secrets never enter the repository

No password, API key, token or credentials file is committed, not even in docs or examples. Read secrets from the environment or from a gitignored file.

The gate has three layers, all driven by [`.gitleaks.toml`](.gitleaks.toml):

1. **pre-commit** (lefthook) scans staged changes with gitleaks.
2. **pre-push** (lefthook) scans every commit that is not yet on a remote.
3. **CI** ([`secret-scan`](.github/workflows/secret-scan.yml)) scans the PR range, so `--no-verify` does not get a secret past it.

Set up once per clone: `brew install gitleaks lefthook && lefthook install`.

A secret that was ever pushed is compromised. Removing it from the tree does not unpublish it, so rotate it at the provider.
