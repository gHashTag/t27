# OWNERS — scripts/

## Primary

**B-Builder** / **Q-QA** — repo automation, CI helpers, language checks.

## Dependencies

- `docs/`, `specs/` — paths scanned by quality scripts.
- Optional: `verify_precision.py` + `requirements-verify-precision.txt` (mpmath); `print_pellis_seal_decimal.py` (stdlib `Decimal`) — research / digit dumps only; not release gates.

## Outputs

Shell scripts invoked locally and from GitHub Actions.
