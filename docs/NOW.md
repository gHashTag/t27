# Current Work — Trinity t27

**Last updated:** 2026-05-24 (V2+codegen synced to master via PR #774)
**Note:** DARPA CLARA PA-25-07-02 submission package migrated to [ghashTag/trinity-clara](https://github.com/gHashTag/trinity-clara)

---

## Active Work

**L-TRI-3 Week 2: V2 (SHA256) Integration** — complete
- 2026-05-23: **prove.rs V2 path integrated** — `ProveRequest.version` field (default 0=V1),
  V2 verification: `derive_phi_challenge_v2` + `verify_phi_response_v2` (SHA256 of 16x16
  GF16 matmul with CHAMPION_WEIGHTS). `verify_ed25519_signature` branches on version
  (TRI_PROVE_V1 vs TRI_PROVE_V2 domain). `epoch_hash` and `next_challenge` both V2-aware.
  2 new E2E tests (valid + wrong response). **33/33 tri tests pass** (17 phi_challenge + 8
  prove + 4 hooks + 4 merkle). Files: `cli/tri/src/depin/{types,prove}.rs`.
- 2026-05-23: **Solana Anchor `submit_proof_v2` instruction** — separate instruction with
  32-byte phi_response. `NodeProofV2` account (160 bytes), `SubmitProofV2` context with
  `proof-v2` PDA seed. On-chain V2 challenge derivation matches off-chain canon: 16 rows
  of SHA256("TRI_PHI_CHALLENGE_V2" || epoch_le8 || node_id || row_index), high nibble of
  bytes[0..16]. Response = SHA256(pack(gf16_matmul(CHAMPION_WEIGHTS, challenge))).
  CHAMPION_WEIGHTS const identical to `phi_challenge.rs`. **Compiles clean, 1/1 test pass**.
  File: `contrib/solana/programs/tri-mining/src/lib.rs`.

**Pure-Rust DLC10 Driver + FPGA Silicon Verified** (branch feat/dlc10-rust)
- 2026-05-23: **LEDs BLINKING ON SILICON** — gf16_heartbeat_top.bit loaded via
  `dlc10 sram`, DONE=1, EOS=1, D5/D6 LEDs blink in phi-pattern. STARTUPE2.CFGMCLK
  as clock source, GF16 dot4 logic, LEDs on R23/T23 (active-LOW). This closes the
  3-month hardware bringup: DLC10 Rust driver → JTAG SRAM → working bitstream.
  SPI flash boot still blocked (BOOTSTS=0 after power-cycle, board-level mode pin
  or flash wiring issue — not a software bug). SRAM boot sufficient for development.
  Added CLI subcommands: flash-read (diagnostic), reload (JPROGRAM+BOOTSTS), improved
  bitswap (per-byte bit-reversal for Master SPI boot convention). Remaining: FT232RL
  UART connection for tok/s measurement, L-TRI-3 Week 2 V2 integration. Updates #592.
- 2026-05-13: **DONE=HIGH ACHIEVED** after 3 months blocked.
  Three root causes fixed in one session:
  1. `cli/dlc10::program_sram_verbose`: replaced broken IR-capture INIT_B polling
     with blind 50ms sleep + 12×10k RTI clocks (DLC10 FX2 firmware does not
     propagate TDO during Shift-IR, so `shift_ir_capture` always returns 0).
     JSHUTDOWN removed. JSTART startup clocks raised from 24 to 2000 per UG470 §6.3.
  2. `cli/dlc10::read_cfg_reg_raw_n`: replaced 5 separate `shift_dr_small` packet
     transfers (which TLR-reset config FSM in between) with one unbroken TMS/TDI
     vector — TLR → RTI → CFG_IN IR → 160-bit packet DR (5×32, packets 0..3 in
     Shift-DR, packet 4 last bit in Exit1-DR) → SELECT_IR → CFG_OUT IR → DR read,
     dispatched as one `do_shift_with_read` call. Matches openFPGALoader
     `Xilinx::dumpRegister` exactly. `tri fpga idcode-cfg` now returns 0x13631093.
  3. `spiOverJtag/constr_xc7a_fgg676.xdc` (openFPGALoader fork): added
     `set_property BITSTREAM.STARTUP.STARTUPCLK JTAGCLK [current_design]`.
     Without it the startup FSM never sees clocks when loading over JTAG
     (default CFGCLK=CCLK only runs during SelectMAP/SPI). STAT was stuck at
     0x4000190C (INIT_COMPL=1, MMCM_LOCK=1, CRC=0, ID_ERROR=0, EOS=0). Rebuilt
     via CI (run 25763758480, sha 800b4dbe...), STAT now 0x401079FC (DONE=1,
     EOS=1). Remaining work: `tri fpga flash-id` returns FF FF FE (floating
     MISO) — bridge wire protocol / CS_N routing needs separate triage.
  Updates #590, Closes #592 (partial: DONE=HIGH; flash-id is follow-up).
- 2026-05-13 (later): **JEDEC=20BA17 read** — fourth and final root cause was
  the wire protocol on top of USER1. The current `spiOverJtag_core.v` from
  openFPGALoader uses an FSM `IDLE → RECV_HEADER1 [→ RECV_HEADER2] → XFER →
  WAIT_END` and requires a leading start-bit + header byte(s) encoding
  mode and transfer length, NOT raw SPI bytes like the older quartiq/
  bscan_spi bridge. Without the header `csn` never asserts and MISO
  floats (FF FF FE). Ported `Xilinx::spi_put_v2` from openFPGALoader
  src/xilinx.cpp:2278 as `Dlc10::spi_xfer_v2`. Added primitives
  `shift_dr_read_bytes` and `go_test_logic_reset` to support it. The
  v2 packet for READ_ID is [0x23, 0xF9, 0x00, 0x00, 0x00, 0x00] over
  48 bits with single Shift-DR scan, ending in TLR. `read_flash_id`
  switched to v2 for all SPI ops (READ_ID, RELEASE_PD, RESET_ENABLE,
  RESET_DEVICE). One implementation gotcha caught by the subagent:
  `real_len = max(tx.len(), rx_len) + 1`, not `tx.len() + 1` — the
  packet must reserve space for the RX bytes that the bridge clocks
  out during XFER. Result: `tri fpga flash-id` returns `20 BA 17`
  (N25Q064A, 8 Mbit). Three-month blocker fully resolved.
- 2026-05-13: **BLOCKER-2 (flash verify) — fixed `spi_xfer_v2` for tx-then-rx flow.**
  When porting `Xilinx::spi_put_v2`, READ_ID (tx empty, rx=3) worked
  because `data_len = max(tx.len(), rx_len)` happened to equal `rx_len`.
  But READ_DATA / PAGE_PROGRAM use a TX phase (3 address bytes) **followed**
  by an RX phase — the two are sequential on the wire, not overlapping.
  The packet payload must be `cmd + tx_bytes + zero_pad(rx_len)`, so
  `data_len = tx.len() + rx_len` (not `max`), and the RX-reconstruction
  index must skip the address-phase echo: `idx = idx_base + tx.len()`.
  Also switched `program_flash` (SECTOR_ERASE, PAGE_PROGRAM, READ_DATA,
  WREN, READ_STATUS) and the spi_wait_wip / spi_write_enable helpers
  to use `spi_xfer_v2`, since the legacy `spi_xfer` only reads floating
  `FF FF...FE FF` through the new `spiOverJtag_core.v` FSM bridge.
  Fixed spi_xfer_v2 RX byte alignment for tx-then-rx flow (READ_DATA + flash verify).
  Verified on real DLC10 hardware via end-to-end `tri fpga program`.
  Updates #590.
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
- 2026-05-12: openXC7 native build re-attempt with 29 GiB free disk.
  bbaexport.py completes (71s real, 2.1 GiB peak RSS, .bba=462 MB).
  bbasm assembles xc7a100tfgg676-1 chipdb (158 MB .bin) in ~6s. nextpnr-xilinx
  routes a user-pin variant of the bridge cleanly (Fmax 254 MHz, post-routing
  legalisation OK). **Blocker:** routing onto the dedicated configuration
  pins (FCS_B=C8, MOSI=B19, MISO=A18) triggers `dict::at()` abort during
  `Preparing clocking...` after IOB placement on `OPAD_X0Y10`
  (GTP_CHANNEL_1_X130Y173). The proxy by design must drive these dedicated
  pins via STARTUPE2+USRCCLKO, which openXC7 does not yet model.
  See docs/fpga/OPENXC7_FGG676_STATUS.md. Docker-Vivado (ce0f7ae3) remains
  the only path to a functional `bscan_spi_xc7a100tfgg676.bit`. Closes #592.
- 2026-05-12: Docker-Vivado recipe refreshed for the actually-on-disk
  installer: `FPGAs_AdaptiveSoCs_Unified_SDI_2025.2_1114_2157_Lin64.bin`
  (web installer stub, 363 MiB). `docker/Dockerfile.vivado` now targets
  Vivado ML Standard 2025.2, drives the unattended `xsetup -b AuthTokenGen`
  via `expect`, and accepts either a pre-baked
  `docker/wi_authentication_key` (Variant A — recommended) or
  `--secret id=xilinx_user / xilinx_pass` (Variant B).
  `docker/install_config.txt` selects only Artix-7 + Spartan-7 modules
  (~10 GiB post-install vs ~96 GiB full archive). Auth token generated
  via expect-driven `xsetup -b AuthTokenGen` (valid until 2026-05-19),
  saved to `docker/wi_authentication_key` (gitignored — see
  `.gitignore`). See docs/fpga/DOCKER_VIVADO_STATUS.md. Refs #592.

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

- 2026-05-13: **CI win** — GitHub Actions builds spiOverJtag_xc7a100tfgg676.bit
  successfully via Vivado 2024.2 on ubuntu-24.04 runner (workflow run 25753882084,
  commit f44f5af3). Root cause of prior route_design failures: the previous
  constr_xc7a_fgg676.xdc tried to use dedicated configuration bank pins
  (C8/B19/A18/B18/A19) which are GT terminals on FGG676. Corrected pinout
  P18/R14/R15/P14/N14 sourced from QMTECH_XC7A75T_100T_200T-CORE-BOARD
  schematic (Bank 14 dual-purpose D00..D03 + FCS_B). New bitstream
  407,262 bytes, sha256 bf5be125e9098d61b4855c599b19a5c90c360592991b7b9b7835af02e605cad2,
  contains "7a100tfgg676" device string. Deployed to fpga/tools/bscan_spi_xc7a100t.bit
  and re-embedded into cli/dlc10 via include_bytes!. Runtime status:
  STAT=0x00000000 after proxy-load — DONE never goes HIGH for both the new and the
  pre-existing fgg676 bitstreams, so the remaining blocker is in the JTAG transport
  layer (cli/dlc10 program_sram path), not the bitstream itself. On-board flash is
  N25Q064A 3V (JEDEC 0x20BA17), not MT25QL128. Closes #592 (CI side); follow-up
  issue needed for proxy-load DONE=LOW. See docs/fpga/SPI_FLASH_DEBUG.md.

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
