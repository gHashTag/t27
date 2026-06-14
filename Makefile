# Minimal top-level convenience targets for t27.
#
# This Makefile is intentionally thin: it only wraps existing scripts so common
# developer checks have a memorable entry point. It is NOT wired into CI (the
# required checks remain check-now-freshness / validate / check /
# check-linked-issue) and adds no build logic of its own.
#
# Anchor: phi^2 + phi^-2 = 3

.PHONY: help verify seal-check seal install-hooks warnings-baseline

# Default target: list what is available.
help:
	@echo "t27 convenience targets:"
	@echo "  make seal-check     Report whether the NMSE manifest seal is fresh"
	@echo "                      vs sha256(bootstrap/src/compiler.rs) (advisory)."
	@echo "  make seal           Recertify the NMSE seal against the current"
	@echo "                      compiler.rs -- explicit, prompts for confirmation."
	@echo "  make install-hooks  Install the local git hooks (incl. the advisory"
	@echo "                      pre-push seal-staleness warning)."
	@echo "  make warnings-baseline  Count non-test build warnings vs the recorded"
	@echo "                      baseline and list the top files (advisory; #969)."
	@echo "  make verify         Run all advisory pre-PR checks at once (seal-check"
	@echo "                      + warnings-baseline + quick test + gate-preview"
	@echo "                      + reseal-preview); never blocks. Skip flags:"
	@echo "                      VERIFY_SKIP_TEST / VERIFY_SKIP_GATES /"
	@echo "                      VERIFY_SKIP_RESEAL =1."

# Advisory umbrella: run the seal-freshness, warnings-baseline, and a quick test
# back-to-back and print one compact summary. Never edits code, never reseals,
# always exits 0 -- a convenience entry point for a pre-PR glance. The real gate
# stays the four required CI checks.
verify:
	@scripts/verify.sh

# Advisory: report NMSE seal freshness. Exit 0 fresh / 2 stale / 3 unsealed.
# Never reseals; refreezing stays an explicit reviewed step.
seal-check:
	@scripts/reseal-check.sh

# Explicit, reviewed reseal: recertify the NMSE seal against the current
# compiler.rs. Prompts for confirmation (or RESEAL_YES=1); never automatic.
seal:
	@scripts/reseal-apply.sh

# Install the local git hooks (advisory pre-push includes the seal check).
install-hooks:
	@scripts/install-git-hooks.sh

# Advisory: count non-test build warnings vs the recorded baseline and list the
# top offending files, to track #969 dead-code progress. Never edits code.
warnings-baseline:
	@scripts/warnings-baseline.sh
