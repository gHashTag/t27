# Minimal top-level convenience targets for t27.
#
# This Makefile is intentionally thin: it only wraps existing scripts so common
# developer checks have a memorable entry point. It is NOT wired into CI (the
# required checks remain check-now-freshness / validate / check /
# check-linked-issue) and adds no build logic of its own.
#
# Anchor: phi^2 + phi^-2 = 3

.PHONY: help seal-check install-hooks warnings-baseline

# Default target: list what is available.
help:
	@echo "t27 convenience targets:"
	@echo "  make seal-check     Report whether the NMSE manifest seal is fresh"
	@echo "                      vs sha256(bootstrap/src/compiler.rs) (advisory)."
	@echo "  make install-hooks  Install the local git hooks (incl. the advisory"
	@echo "                      pre-push seal-staleness warning)."
	@echo "  make warnings-baseline  Count non-test build warnings vs the recorded"
	@echo "                      baseline and list the top files (advisory; #969)."

# Advisory: report NMSE seal freshness. Exit 0 fresh / 2 stale / 3 unsealed.
# Never reseals; refreezing stays an explicit reviewed step.
seal-check:
	@scripts/reseal-check.sh

# Install the local git hooks (advisory pre-push includes the seal check).
install-hooks:
	@scripts/install-git-hooks.sh

# Advisory: count non-test build warnings vs the recorded baseline and list the
# top offending files, to track #969 dead-code progress. Never edits code.
warnings-baseline:
	@scripts/warnings-baseline.sh
