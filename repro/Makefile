# Reproducibility targets — run: make -C repro <target>
.PHONY: repro-smoke repro-language repro-numerics repro-numerics-certify repro-ar repro-paper-figures

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

# Validate the committed certifying manifest against schemas/nmse-protocol-v1.json.
# Re-validation only (no measurement); the measured run is the L4/L5 oracle below.
repro-numerics-certify:
	cd "$(ROOT)" && python3 repro/numerics/validate_manifest.py
	@echo "Certifying manifest conforms to schemas/nmse-protocol-v1.json (seal_hash honesty enforced)."

repro-ar:
	cd "$(ROOT)" && bash tests/validate_conformance.sh
	@echo "AR vectors: conformance/ar_*.json"

repro-paper-figures:
	@echo "P2: add pinned scripts/notebooks under repro/paper/ and wire this target."
	@true
