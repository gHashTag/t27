# Current Work — Trinity t27

**Last updated:** 2026-05-12
**Note:** DARPA CLARA PA-25-07-02 submission package migrated to [ghashTag/trinity-clara](https://github.com/gHashTag/trinity-clara)

---

## Active Work

**Pure-Rust DLC10 Driver + SPI Flash** (branch feat/dlc10-rust)
- cli/dlc10 crate: USB control transfer + JTAG state machine via rusb (no Vivado, no openFPGALoader)
- IDCODE 0x13631093 (XC7A100T) verified on silicon through pure-Rust path
- cli/flash-spi rewritten to call dlc10::Dlc10::program_flash directly
- Includes bscan_spi_xc7a100t.bit (MIT, quartiq) + Cypress FX2 firmware
- 2026-05-12: SPI bit-reverse + 1-bit JTAG capture skew fix; new diagnostic subcommands
  `tri fpga proxy-load|proxy-status|spi-raw|ir-probe|flash-id-debug` for JEDEC=FF FF FF triage
  (see docs/fpga/SPI_FLASH_DEBUG.md). Refs #590, Closes #592.
- 2026-05-12: openXC7 QMTech-specific JTAG-to-SPI proxy build path
  (`fpga/bscan_spi_qmtech/`, `tri fpga build-proxy --install`) — replaces the
  generic csg324 quartiq bitstream with a FGG676 one built via
  yosys + nextpnr-himbaechel + prjxray (no Vivado).
  Refs #592, trabucayre/openFPGALoader#663.
- 2026-05-12: openXC7 native build attempt on macOS — corrected himbaechel
  device name xc7a100t-fgg676-2 -> xc7a100tfgg676-1 (the canonical prjxray
  spelling). Native chipdb generation via bbaexport.py reaches the
  Exporting tile and site instances stage then OOMs at ~1.5 GiB RSS on a
  low-disk Apple Silicon box (<1 GiB free, 16 GiB RAM). Docker-Vivado
  path (commit ce0f7ae3) remains the recommended Mac flow. Closes #592.

**Ring 080-087: Ternary Collection Specs** (PR #558 — merged)
- 6 new specs: sorting, search, pattern matching, graph, tree, set, hash table
- Closes #260 #262 #264 #267 #269 #271 #275

**Hybrid v2 + Golden Tests** (PR #559 — merged)
- L2 cosine similarity with f64 Pell numbers (N=2..152)
- Golden tests for N={5,10,15,20,50,152}, all pass
- Closes #339 #287

**GF Competitive Analysis** (PR pending)
- verify_precision.py with mpmath 100-digit sacred constants
- gf_competitive.t27 + pellis_verify.t27 specs
- Closes #289

**Pre-commit Gate (Ring 073)** (PR #554)
- 4 gates: NOW freshness, seal coverage, L7 no-new-shell, cargo check
- Install: `ln -sf ../../scripts/pre-commit .git/hooks/pre-commit`

**FFI Bug Fixes + API Completeness** (PR #553 — merged)
- BUG-001/002/003 fixed, GF4/8/12/20/24 encode/decode added

---

## Previous Active Work

**Ring 32 — Cloud Orchestration** (PR #485) — New ring for cloud deployment capabilities
- specs/base/ring_32.t27 — Ring 32 definition
- specs/cloud/railway_deploy.t27 — Railway deployment orchestrator
- specs/base/debounce.t27 — φ-structured debouncing (618ms)
- specs/queen/task_analysis.t27 — Task priority analysis for 27 bees
- specs/compiler/mod_structure.t27 — Module structure validation
- Full TDD coverage: 12 tests, 6 invariants, 1 benchmark
- Constitutional compliance: L1-L7

---

**DARPA CLARA Documentation Organization** (PR #478) — Docs structure overhaul for clarity

**DARPA CLARA v1.5 Submission** (PR #473) — Ready for review, deadline April 17, 2026

---

**φ² + 1/φ² = 3 | TRINITY**
