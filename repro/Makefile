# Reproducibility targets — run: make -C repro <target>
.PHONY: repro-smoke repro-language repro-numerics repro-ar repro-paper-figures

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST)))/..)
T27C := $(ROOT)/bootstrap/target/release/t27c

repro-smoke:
	cd "$(ROOT)/bootstrap" && cargo build --release
	cd "$(ROOT)" && bash tests/run_all.sh && bash tests/validate_conformance.sh && bash tests/validate_gen_headers.sh

repro-language:
	cd "$(ROOT)/bootstrap" && cargo build --release
	cd "$(ROOT)" && "$(T27C)" compile-all
	cd "$(ROOT)" && bash tests/validate_gen_headers.sh

repro-numerics:
	cd "$(ROOT)" && bash tests/validate_conformance.sh
	@echo "Numeric vectors: conformance/gf*_vectors.json, sacred_physics*.json (see module field in each JSON)."

repro-ar:
	cd "$(ROOT)" && bash tests/validate_conformance.sh
	@echo "AR vectors: conformance/ar_*.json"

repro-paper-figures:
	@echo "P2: add pinned scripts/notebooks under repro/paper/ and wire this target."
	@true
