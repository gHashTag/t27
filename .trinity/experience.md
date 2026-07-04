# t27 / Trinity Agent Experience Log

## 2026-07-04 — Wave Loop 409 (per-OSCFSEL transaction lookup + tighter duty bound)

### What worked
- Refactoring `artix7_boot_transaction` to call `artix7_boot_transaction_for_oscfsel`
  made the per-OSCFSEL lookup table trivial to state and prove. The equality
  theorem `artix7_boot_transaction_eq_for_oscfsel` preserves the link to the
  config-level API.
- Using `interval_cases` (from `Mathlib.Tactic`) on `oscfsel ≤ 7` let Lean
  enumerate the eight documented OSCFSEL values and discharge each branch with
  `simp` + the UG470 frequency table. This is a clean computational proof pattern
  for small finite lookup tables.
- Deriving the duty-cycle bound from the N25Q128 `t_CL` / `t_CH` limits and the
  measured frequency replaces the arbitrary 25%–75% placeholder with a bound that
  tightens automatically as frequency increases.
- Re-running the live P12 capture immediately confirmed the wiring blocker is
  unchanged, avoiding the temptation to claim Variant A succeeded.

### What changed behavior
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`: added
  `artix7_boot_transaction_for_oscfsel`,
  `oscfsel_zero_to_seven_transaction_satisfies_flash_spec`, and
  `artix7_boot_transaction_eq_for_oscfsel`; imported `Mathlib.Tactic`.
- `cli/tri/src/fpga.rs`: added `N25Q128_MIN_SCK_LOW_S` / `N25Q128_MIN_SCK_HIGH_S`
  and replaced the fixed 25%–75% duty guard with a frequency-derived bound
  clamped to 10%–90%.
- `fpga/HARDWARE_SSOT.md` §3.6.9: per-OSCFSEL transaction table and note that
  OSCFSEL 6/7 are model-only.
- Close-out artifacts: `docs/reports/WAVE_LOOP_409_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W409_2026-07-04.md`, and
  `docs/reports/FPGA_LOOP_COOPERATION_W410_2026-07-04.md`.

### Patterns to reuse
- For a finite lookup-table proof in Lean 4, import `Mathlib.Tactic` and use
  `interval_cases` followed by `simp` with the lookup function and constants.
- When replacing a placeholder constant with a computed bound, keep a small
  sensible clamp so pathological low-frequency captures are still rejected.
- Always re-run the physical gate that was blocked in the previous wave before
  claiming it is unblocked.

### Anti-patterns to avoid
- Do not add a new tactic import without checking that the file builds with it;
  `interval_cases` is not available in a bare Lean file.
- Do not change a definition used by existing theorems without updating their
  `simp` sets; `artix7_boot_transaction` now expands to
  `artix7_boot_transaction_for_oscfsel`, so the latter must be in the simp list.

## 2026-07-04 — Wave Loop 408 (SPI transaction model + real CCLK blocker)

### What worked
- Adding a `SPIReadTransaction` structure and `artix7_boot_transaction` function
  turned the static `flash_spi_timing_ok` predicate into a transaction-level
  model that captures CS# high time, SCK edges, SCK low/high times, and wake-up
  delay. This is a harder claim for competitors to reproduce than a single
  frequency bound.
- Proving `canonical_implies_transaction_satisfies_flash_spec` required dealing
  with `UInt8.toNat 0` carefully: compute the `cfg.oscfsel.toNat = 0` equality
  as a separate `have` and then use `simp` with that equality, rather than
  relying on `decide` with free variables.
- Attempting the real P12 capture immediately surfaced the missing wiring
  blocker. Recording the failed capture as evidence is better than pretending
  Variant A happened.
- Resealing all `.t27` specs with the freshly built `t27c` release binary
  brought the seal files back into sync with the compiler output.

### What changed behavior
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`: added `SPIReadTransaction`,
  `artix7_boot_transaction`, `transaction_satisfies_flash_spec`, and the
  theorems `canonical_oscfsel_transaction_satisfies_flash_spec`,
  `canonical_implies_transaction_satisfies_flash_spec`, and
  `cold_por_implies_transaction_satisfies_flash_spec`.
- `fpga/HARDWARE_SSOT.md` §3.6.8 documents the transaction model and the
  real-capture blocker.
- Close-out artifacts: `docs/reports/WAVE_LOOP_408_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W408_2026-07-04.md`, and
  `docs/reports/FPGA_LOOP_COOPERATION_W409_2026-07-04.md`.
- `docs/NOW.md` updated with W408 entry and `Last updated: 2026-07-04`.

### Patterns to reuse
- When a Lean proof involves a `UInt8` literal projected to `Nat`, compute the
  equality as a standalone `have` and feed it to `simp` instead of calling
  `decide` on a goal with free variables.
- When a real hardware step is blocked, run the command anyway, capture the
  output, and commit it as evidence. The blocker becomes a traceable
  acceptance-criterion item instead of an invisible gap.
- Before claiming `./scripts/tri test` passes, run it and reseal any stale
  seal files so the verification gate is grounded in the current compiler.

### Anti-patterns to avoid
- Do not write Lean proofs that rely on `decide` with free variables in the
  goal; use `intro` binders plus `exact rfl`, or compute the closed equality
  first and then simplify.
- Do not update only the report date; also update `docs/NOW.md` `Last updated:`
  or the suite check will block the build.
- Do not claim `./scripts/tri test` passes when a local phase (gen-verilog-yosys-smoke)
  has pre-existing failures; report the exact phase and the tracked defect file
  instead.
- When `gh` operations fail with `HTTP 401: Bad credentials`, check for a stale
  `GH_TOKEN` environment variable overriding the keyring credentials. Unset it
  (`unset GH_TOKEN`) so `gh` uses the active keyring account.

## 2026-07-13 — Wave Loop 407 close-out / Wave Loop 408 setup

### What worked
- Using `gh pr edit <n> --body-file /tmp/body.md` repaired a PR body that had
  been mangled by shell interpretation of backticks and newlines in an inline
  `--body` argument.
- Creating the W408 issue (#1318) and branch (`wave-loop-408`) immediately
  after the W407 commit keeps the loop boundary explicit and gives the next
  wave a clean starting point.
- Branching `wave-loop-408` from `wave-loop-407` carries the W407 timing-model
  changes while PR #1317 is still open; it can be rebased onto `master` once
  #1317 lands.

### Anti-patterns to avoid
- Never pass a `gh pr create --body` string that contains backticks or literal
  newlines; always write the body to a file and use `--body-file`.
- Do not assume the next PR/issue number matches the `Closes #N` reference;
  GitHub assigns the next available number independently.

## 2026-07-13 — Wave Loop 407 (Deeper SPI flash timing + synthetic CCLK fixture)

### What worked
- Extending the W406 formal model with additional N25Q128 timing constants
  (`MIN_SCK_LOW_NS`, `MIN_SCK_HIGH_NS`, `WAKE_FROM_POWERDOWN_US`) and a
  comprehensive `flash_spi_timing_ok` predicate made the CCLK bound a
  *component* of a fuller timing-safety argument rather than a one-off claim.
- Replacing `cclk_within_flash_spec` with `flash_spi_timing_ok` inside
  `cold_por_spi_flash_pred` keeps the cold-POR precondition as strong as
  possible while recovering the original frequency bound through a separate
  lemma (`flash_spi_timing_ok_implies_cclk_within_flash_spec`).
- Adding a `--synth` fixture to `tri fpga measure-cclk` gave the validation
  pipeline a CI-runnable path with no bench hardware, which is exactly the
  fallback needed when P12 is not wired.
- Unit tests for `is_logic_csv`, `parse_logic_csv`, and `generate_synth_cclk_csv`
  catch parser regressions before they reach the conformance suite.

### What changed behavior
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`: added `N25Q128_MIN_SCK_LOW_NS`,
  `N25Q128_MIN_SCK_HIGH_NS`, `N25Q128_WAKE_FROM_POWERDOWN_US`, `cclk_period_ns`,
  `sck_duty_ok`, and `flash_spi_timing_ok`. Proved
  `canonical_oscfsel_flash_spi_timing_ok`,
  `canonical_implies_flash_spi_timing_ok`,
  `cold_por_implies_flash_spi_timing_ok`, and
  `flash_spi_timing_ok_implies_cclk_within_flash_spec`.
  `cold_por_spi_flash_pred` now requires `flash_spi_timing_ok`.
- `cli/tri/src/fpga.rs`: `FpgaCmd::MeasureCclk` gained `--synth`. Added
  `generate_synth_cclk_csv`, duty-cycle constants, duty-cycle validation in
  `--validate`, and four new unit tests.
- `fpga/HARDWARE_SSOT.md` §3.6 expanded with N25Q128 SCK low/high / wake-up
  constants, `flash_spi_timing_ok` traceability, synthetic fixture instructions,
  and real-capture wiring checklist.
- `docs/NOW.md` updated with the W407 entry.
- Close-out artifacts: `docs/reports/WAVE_LOOP_407_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-13.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-13.md`.

### Patterns to reuse
- When a formal predicate can be strengthened without losing the old lemma,
  replace the old predicate in the main definition and re-prove the old lemma
  as a corollary. This keeps downstream proofs compact and the model auditable.
- For bench commands that depend on physical wiring, add a synthetic fixture
  path so CI can exercise the same parsing/validation code without the probe.

### Anti-patterns to avoid
- Do not conflate static config timing with dynamic STAT observations. The
  new `flash_spi_timing_ok` is a function of `OSCFSEL` only; the cold-POR
  predicate links it to the observed STAT outcome, not the other way around.

## 2026-07-12 — Wave Loop 406 (CCLK measurement + OSCFSEL/CCLK timing safety in Lean 4)

### What worked
- Adding an axiomatic `cclk_nominal_hz` lookup and `N25Q128_MAX_SCK_HZ` flash spec to
  `TernaryFPGABoot.lean` closed the quantitative gap in the cold-POR formal model.
  `cclk_within_flash_spec` now links `OSCFSEL` to the Micron standard-read timing
  bound (≤ 50 MHz) and is integrated into `cold_por_spi_flash_pred`.
- Extending `tri fpga measure-cclk` with a `--live` path that drives `sigrok-cli`
  and parses exported logic CSV gives a repeatable way to verify nominal CCLK
  against the same flash bound, not just a manual spreadsheet.
- Keeping the measurement command board-less (CSV) by default and opt-in (`--live`)
  preserves CI while enabling bench evidence when the P12 wiring is ready.
- The W405 `cclk_sweep` gate was already sufficient to prove cold-POR success; W406
  adds the *formal reason* the CCLK rate itself is safe, which is the remaining
  half of the boot-verification gap.

### What changed behavior
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`: added `OSCFSEL_COUNT`,
  `OSCFSEL_MAX`, `cclk_nominal_hz`, `N25Q128_MAX_SCK_HZ`,
  `N25Q128_MIN_CS_HIGH_NS`, and `cclk_within_flash_spec`. Three theorems connect
  the canonical config, any canonical config, and the cold-POR predicate to the
  flash spec.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`: `cold_por_spi_flash_pred` now
  requires `BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel`.
- `cli/tri/src/fpga.rs`: `FpgaCmd::MeasureCclk` now accepts `--live`, `--driver`,
  `--channel`, `--samplerate`, `--samples`, and `--validate`. Added live capture
  through `sigrok-cli`, logic CSV parsing, frequency/period estimation, and
  flash-spec validation.
- `fpga/HARDWARE_SSOT.md` §3.6 updated with nominal CCLK table, live-capture
  protocol, CSV parsing rules, and formal traceability to
  `BitstreamConfig.cclk_within_flash_spec`.
- `docs/NOW.md` updated with the W406 entry.
- Close-out artifacts: `docs/reports/WAVE_LOOP_406_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-12.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-12.md`.

### Patterns to reuse
- When a physical quantity is implicit in a formal model, expose it as a lookup
  table + spec constant + predicate; then prove that the concrete/default case
  satisfies the predicate. This makes the model auditable without over-fitting
  to one board.
- Bridge bench tooling and formal models through a single CLI subcommand that
  accepts both recorded CSV (offline) and live logic-analyzer capture (online)
  so the same validation predicate can be evaluated on either data source.
- Document the *blocking hardware precondition* (P12 → logic analyzer channel)
  explicitly in the report and cooperation variants rather than silently leaving
  the measurement at zero.

### Anti-patterns to avoid
- Do not let a CLI live-capture helper swallow the underlying tool error. The
  first implementation masked `sigrok-cli` failures; surfacing `stderr` in the
  `anyhow` error made the "no transitions" case immediately interpretable.
- Do not add new formal constants without a corresponding test/theorem; the
  `canonical_oscfsel_within_flash_spec` `decide` theorem catches lookup-table
  typos at build time.

## 2026-07-04 — Wave Loop 405 (Hardware smoke-gate `--flash-boot`)

### What worked
- Reusing the empirically-working `cclk_sweep` cold-POR path for the flash-boot
  smoke gate instead of writing a separate `program_flash` + `capture_stat`
  sequence. The first implementation produced `H2_CCLK_TIMING` (`STAT=0x5000190C`)
  repeatedly despite identical operator actions; delegating to `cclk_sweep`
  immediately reached `STAT=0x401079FC` and passed the gate.
- Returning `Vec<SweepResult>` from `cclk_sweep` let both the CLI and the
  smoke-gate caller inspect the outcome without parsing logs or side files.
- Keeping `--flash-boot` explicit (and implying `--require-cable`) preserves the
  existing SRAM smoke-gate path and the board-less default.
- Writing the W405 plan, NOW.md entry, and close-out reports in the same
  session keeps the traceability chain intact (issue -> branch -> implementation
  -> evidence -> next variants).

### What changed behavior
- `cli/tri/src/fpga.rs`: `FpgaCmd::SmokeGate` now accepts `--flash-boot` and
  `--wait-seconds`. When `--flash-boot` is set, `smoke_gate` calls
  `cclk_sweep` with a single `OSCFSEL=0` variant, verifies that at least one
  result has `done=true`, and prints the existing `boot_success` confirmation.
- `cclk_sweep` now returns `Result<Vec<SweepResult>>`; CLI dispatch bails if
  no variant reaches `DONE=HIGH`.
- `.claude/plans/wave-loop-405.md` acceptance criteria updated.
- `docs/NOW.md` updated with the W405 entry.
- Close-out artifacts: `docs/reports/WAVE_LOOP_405_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-10.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-10.md`.

### Patterns to reuse
- When a physical cold-POR code path is known to work, reuse it exactly rather
  than duplicating it with slightly different helper calls. Subtle differences
  in stdin timing, prompt text, or helper interaction can change bench
  behavior even when the openFPGALoader invocations look identical.
- Make command helpers return structured results so higher-level callers can
  assert on them without parsing text output.
- Keep hardware gates opt-in via explicit flags so CI and board-less runs are
  unaffected.

### Anti-patterns to avoid
- Do not assume two command implementations are equivalent just because they
  invoke the same binary with the same flags. Cold-POR state machines can be
  sensitive to timing and order that is not obvious in the code.

## 2026-07-06 — Wave Loop 404 (Hardware smoke-gate `--require-cable`)

### What worked
- Checking the bench before choosing the variant changed the wave outcome: the
  Digilent FTDI cable and XC7A200T board were reachable, so **Variant C**
  (hardware smoke gate) became feasible instead of another no-hardware formal
  extension.
- Keeping `--require-cable` as an **optional** flag preserved the board-less
  default path. CI without a cable still passes all static checks; a runner
  with hardware can opt into the SRAM load assertion.
- Reusing the existing `load_sram` and `capture_stat` helpers kept the change
  small and avoided duplicating openFPGALoader parsing logic.
- Asserting the same `boot_success` conditions used by the Lean model
  (`DONE=1`, `MODE=0b001`, no CRC/ID/DEC errors) links the hardware smoke gate
  directly to the formal predicates.
- On the bench: `openFPGALoader --detect` returned idcode `0x3636093`, SRAM
  load completed with `done 1`, and post-load STAT matched `0x401079FC`.
- Conformance suite: **576/576 PASS**.

### What changed behavior
- `cli/tri/src/fpga.rs`: `FpgaCmd::SmokeGate` now accepts `--require-cable`,
  `--cable`, and `--part`. When `--require-cable` is set, the gate runs
  `cable_detected`, `load_sram`, `capture_stat`, and `assert_stat_boot_success`
  before the existing board-less checks.
- `fpga/HARDWARE_SSOT.md` §3.2 now references the hardware smoke traceability.
- `docs/NOW.md` updated with the W404 entry.
- Close-out artifacts: `docs/reports/WAVE_LOOP_404_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-07.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-07.md`.

### Patterns to reuse
- Probe hardware availability at the start of a wave; it can change which
  variant is highest leverage.
- Add optional hardware gates as `--require-<resource>` flags so board-less CI
  stays green while physical evidence can be collected when a resource is present.
- Reuse existing command helpers (`load_sram`, `capture_stat`) instead of
  spawning openFPGALoader ad-hoc; this keeps parsing and error handling
  consistent.

### Anti-patterns to avoid
- Do not make a hardware gate mandatory unless the normal CI environment is
  guaranteed to have the resource. A broken cable should fail the specific
  check, not the whole pipeline.
- Do not skip the board-less path when adding hardware coverage; the static
  audit is still the regression barrier that runs on every PR.

## 2026-07-05 — Wave Loop 403 (Bitstream config linked to cold-POR decision tree)

### What worked
- Falling back to **Variant B** again (Lean 4 extension) let W403 close without
  bench hardware. The formal layer added value by connecting the `.bit`
  configuration audit to the STAT-register decision tree.
- Keeping the `BitstreamConfig` structure field names identical to the
  `tri fpga bit-config` output (`idcode`, `spi_buswidth`, `startupclk`,
  `oscfsel`) makes the formal model traceable to the CLI tool.
- The `ColdPOR` structure cleanly separates static bitstream facts from dynamic
  physical preconditions (`mode_ok`, `no_cable_interference`), matching the
  prose in `fpga/HARDWARE_SSOT.md`.
- Proving `decision_tree_exhaustive` by explicit `Or.inl` / `Or.inr`
  construction avoided fragile `tauto`/`rcases` behavior on `Bool` disjunctions
  defined via `decide`.
- Removing the unnecessary `eos` requirement from `boot_success` closed a
  logical gap and made the exhaustiveness theorem provable without inventing an
  unreachable "other" branch.
- Conformance suite: **576/576 PASS**; `lake build Trinity.TernaryFPGABoot` green.

### What changed behavior
- `proofs/lean4/Trinity/TernaryFPGABoot.lean` now contains:
  - `BitstreamConfig` and `BitstreamConfig.canonical`
  - `ColdPOR` and `cold_por_spi_flash_pred`
  - Linkage lemmas `cold_por_done_eos_high_implies_boot_success`,
    `cold_por_done_low_implies_h2`, and `decision_tree_exhaustive`
- `fpga/HARDWARE_SSOT.md` §3.2 now links the canonical bitstream config audit to
  the Lean 4 predicates and the exhaustive decision-tree theorem.
- `docs/NOW.md` updated with the W403 entry.
- Close-out artifacts: `docs/reports/WAVE_LOOP_403_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-06.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-06.md`.

### Patterns to reuse
- Extend a formal model one layer at a time: W402 formalized STAT decode and
  the decision tree; W403 formalized the static bitstream config that feeds the
  tree. Each layer is a small, reviewable diff.
- Use explicit disjunct construction in Lean 4 when working with `Bool`
  predicates that contain `decide` terms; automation is brittle there.
- Keep the physical-deferred AC explicit in the report and the next-loop
  cooperation variants so the work does not silently drop off the radar.

### Anti-patterns to avoid
- Do not require `eos` in a success predicate unless the exhaustiveness proof
  actually needs it. Unnecessary conjuncts create unreachable model corners.
- Do not rely on `tauto`/`rcases` to split `Bool` disjunctions that are not
  syntactic inductives; build the proof term explicitly instead.

## 2026-07-05 — Wave Loop 402 (Cold-POR decision tree formalized in Lean 4)

### What worked
- Defaulting to **Variant B** (Lean 4 formalization) when bench hardware was
  unavailable let W402 close cleanly. The physical CCLK capture tooling was
  already ready from W401; only the operator step was missing.
- Modeling the 7-series STAT register directly from the `cli/dlc10` bit layout
  kept the formal predicates faithful to the Rust tooling. Named field decoders
  (`mode`, `done`, `eos`, `crc_error`, `id_error`, `dec_error`, `bus_width`)
  make the Lean module readable next to `fpga/HARDWARE_SSOT.md`.
- Proving both the W400 success example (`0x401079FC`) and the incomplete
  example (`0x5000190C`) as concrete instances of `boot_success` and
  `h2_cclk_timing` ties the formal specification to real captured data.
- Squashing the W397-W401 wave sequence into a single mergeable commit was the
  only path through the L1 TRACEABILITY gate, because the long-lived
  `trinity-rust-rings` branch had accumulated commits without per-commit issue
  references.
- Resealing the three specs whose generated hashes shifted after the master
  gen-verilog backend (#1250) reached the branch kept the conformance gate green.
- Conformance suite: **576/576 PASS**.

### What changed behavior
- New Lean 4 module `proofs/lean4/Trinity/TernaryFPGABoot.lean` formalizes the
  cold-POR / CCLK decision tree.
- `proofs/lean4/Trinity.lean` imports the new module.
- `fpga/HARDWARE_SSOT.md` §3.2 now links the documented decision tree to the
  Lean predicates.
- `.trinity/current-issue.md` points to W402 issue #1305.
- `.claude/plans/wave-loop-402.md` records the weak-point + competitor analysis.
- Close-out artifacts: `docs/reports/WAVE_LOOP_402_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-05.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-05.md`.

### Patterns to reuse
- When a physical AC cannot be closed in a headless session, convert it into a
  formal or tooling AC that captures the same knowledge and can be verified
  board-less.
- Keep formal predicates adjacent to the operational prose that defines them;
  cross-linking the docs and the Lean module makes both easier to audit.
- Squash long-lived feature branches before opening a PR if earlier commits
  lack issue references; a single clean merge commit satisfies L1 TRACEABILITY.
- After any backend change reaches a working branch, run the seal gate and
  reseal affected specs before declaring the wave complete.

### Anti-patterns to avoid
- Do not let a long-lived branch accumulate commits without issue references;
  landing becomes painful when branch protection checks every commit.
- Do not assume the conformance suite count is static; backend improvements can
  change generated hashes and require resealing.
- Do not skip documenting the deferred physical AC; state explicitly what is
  blocked and what would unblock it.

## 2026-07-09 — Wave Loop 401 (Cold-POR protocol hardening & board-less CI guards)

### What worked
- Treating W401 as a **hardening** loop rather than another physical experiment
  let the work close cleanly without board access. The W400 physical result was
  already known; W401 made it regression-proof.
- Extending `scripts/dump_bit_config.py` with `--assert-oscfsel 0` and
  `--assert-no-crc-writes` gave `tri fpga smoke-gate` the exact assertions
  needed to protect the canonical default bitstream.
- Adding `tri fpga boot-protocol` (interactive and `--checklist` modes) makes
  the cold-POR steps explicit and printable, reducing operator error in future
  lab sessions.
- Auto-detecting DSView / PulseView / Saleae CSV headers in `measure-cclk --csv`
  removed the previous single-tool dependency; the same logic-analyser export can
  come from whichever tool is on the bench.
- Running the smoke-gate dry-run CCLK sweep inside `tri test` means the report
  pipeline is exercised on every CI run, even with no board connected.
- Conformance suite stayed at **575/575 PASS**.

### What changed behavior
- `tri fpga smoke-gate` now asserts `OSCFSEL=0` and no CRC writes in addition to
  the previous IDCODE / SPI-x1 / CCLK-startup checks.
- `tri fpga smoke-gate` also runs a board-less dry-run CCLK sweep and verifies
  `sweep-report` produces six variant rows.
- `tri fpga boot-protocol` is the canonical interactive / checklist command for
  cold-POR experiments.
- `tri fpga measure-cclk --csv` accepts any of the three common logic-analyser
  export formats and returns frequency / duty cycle.
- `fpga/HARDWARE_SSOT.md` documents the new commands, the CSV formats, and the
  dry-run CI guard.
- Close-out artifacts: `docs/reports/WAVE_LOOP_401_REPORT.md`,
  `FPGA_LOOP_EVIDENCE_2026-07-09.md`, and
  `FPGA_LOOP_COOPERATION_2026-07-09.md`.

### Patterns to reuse
- Turn a proven physical result into a static assertion set so CI protects it
  from silent regression.
- Provide both interactive and `--checklist` modes for any protocol that has a
  human-in-the-loop step; the checklist is reviewable in PRs and lab notebooks.
- Detect CSV formats by content rather than by filename or user flag; it makes
  the tool robust to whichever instrument happens to be available.
- Run the full dry-run path of a hardware workflow inside the normal test suite
  so report-generation logic is exercised board-less.

### Anti-patterns to avoid
- Do not let a successful physical experiment end without board-less guards;
  the next wave may not have hardware access.
- Do not assume one logic-analyser export format; DSView, PulseView, and Saleae
  all differ in header spelling and column naming.
- Do not add physical-only acceptance criteria to a loop that cannot access the
  bench; defer them explicitly and document the deferred state.

## 2026-07-08 — Wave Loop 400 (FPGA SPI boot root-cause closure — default bitstream boots from flash)

### What worked
- Running the automated `tri fpga cclk-sweep` on the physical board with `--wait-seconds 120` kept the protocol disciplined: disconnect cable, power-cycle, reconnect, press ENTER.
- Capturing `STAT` with `--pre-jtag-reset` (no JTAG reset / PROGRAM_B pulse) gave the true cold-POR state rather than a post-reset artifact.
- All six `OSCFSEL` variants (0..5) produced `STAT=0x401079FC` (`DONE=1`, `MODE=001`, `EOS=1`, no CRC/ID errors), so the default bitstream is verified to boot from flash.
- Archiving stale dry-run/partial JSON logs into `build/fpga/boot-log-archive/` kept the active `boot-log-*.json` directory clean for `sweep-report`.
- `sweep-report` correctly aggregated the six logs into `sweep-report-w400-clean.md`, confirming the first working value is `OSCFSEL=0`.
- `fpga/HARDWARE_SSOT.md` was updated to state that the canonical bitstream boots from flash and that earlier `DONE=0` observations were caused by incomplete cold-POR or JTAG-cable interference, not CCLK timing.

### What changed behavior
- `fpga/HARDWARE_SSOT.md` §3.3 now contains the W400 physical result box and declares the default `ternary_mac_demo_top_200t.bit` the working default.
- `docs/reports/WAVE_LOOP_400_REPORT.md`, `FPGA_LOOP_EVIDENCE_2026-07-08.md`, and `FPGA_LOOP_COOPERATION_2026-07-08.md` are the W400 close-out artifacts.
- The CCLK timing hypothesis (H2) is closed as a blocker; the remaining work is to measure the actual CCLK frequency for documentation.

### Patterns to reuse
- When a hardware experiment has many variants, script the entire sweep in one command that handles variant generation, programming, user prompting, STAT capture, and JSON logging.
- Use `--pre-jtag-reset` (or the tool's equivalent) when diagnosing cold-POR; a normal JTAG reset before `STAT` read destroys the evidence.
- When all variants pass, the default is the default — do not patch what already works.
- Keep raw logs and generated reports in version control so the evidence is reviewable without re-running the physical experiment.

### Anti-patterns to avoid
- Do not attribute `DONE=0` to CCLK timing before ruling out incomplete cold-POR and attached JTAG-cable interference.
- Do not leave stale dry-run logs in the active log directory; archive them so report generators do not mix real and synthetic data.
- Do not skip writing the close-out report because the physical result was unexpected; document the null result as strongly as a fix.

## 2026-07-05 — Wave Loop 399 (FPGA SPI boot cold-POR CCLK sweep automation)

### What worked
- Adding `tri fpga cclk-sweep` wrapped the entire W398 variant workflow into one
  command: generate variants, program flash, prompt for the physical power-cycle,
  capture STAT, and write JSON logs. This keeps the only manual step strictly the
  cable / power handling that software cannot perform.
- Adding `tri fpga sweep-report` turned the per-variant JSON logs into a single
  markdown evidence table, making it easy to identify the first working OSCFSEL
  value after a session.
- Adding `tri fpga measure-cclk` gives a concrete capture protocol (pin P12,
  DSLogic settings) and optional CSV parsing so frequency/duty cycle can be
  estimated from a logic-analyser export.
- A `--dry-run` mode let the sweep and report paths be tested board-less in CI.
- Conformance suite stayed at **575/575 PASS**; FPGA CLI changes remain isolated
  from the compiler path.

### What changed behavior
- `tri fpga cclk-sweep` is now the canonical way to run a cold-POR CCLK sweep.
- `tri fpga sweep-report` reads `build/fpga/boot-log-*.json` and produces a
  markdown report.
- `tri fpga measure-cclk` documents CCLK pin P12 and DSLogic settings and can
  parse DSView CSV exports.
- `fpga/HARDWARE_SSOT.md` §3.4 and §9 describe the automated sweep and measurement
  protocol.
- W399 closes with tooling complete; the physical board sweep is deferred to W400.

### Patterns to reuse
- When a physical action cannot be automated, wrap everything around it in a single
  command and make the manual step explicit in printed instructions.
- Persist every attempt in machine-readable JSON so a separate report command can
  summarise results without re-running the experiment.
- Provide a `--dry-run` mode for any hardware-dependent workflow so CI and review
  can exercise the logic without a board.
- Keep the report generator separate from the data collector; they evolve at
  different rates and may be run by different people.

### Anti-patterns to avoid
- Do not claim a CCLK timing fix is verified without a physical cold-POR
  measurement and an actual frequency reading.
- Do not mix data collection and report formatting in one function; separation
  makes both easier to test.
- Do not let a hardware-dependent command fail CI by lacking a board-less path.

## 2026-07-08 — Wave Loop 398 (FPGA SPI boot root-cause closure — CCLK variant tooling, H2 actionable)

### What worked
- Adding `tri fpga patch-cor0` and `tri fpga cclk-variants` made the H2 CCLK/SPI-startup hypothesis testable without regenerating the bitstream from openXC7, which has no `CONFIGRATE` knob.
- Extending `scripts/dump_bit_config.py` to decode `CTL0` and `BSPI` and to warn on `OSCFSEL=0` / CRC writes gives clearer diagnostics for both users and CI.
- Adding assertion flags to `bit-config` and wiring them into `tri fpga smoke-gate` turned the board-less smoke gate into a real regression catch for IDCODE/SPI width/startup clock.
- Instructing the user to **disconnect the JTAG cable during POR** in `tri fpga boot-log` addresses a known source of cold-POR corruption (AR66954 / XAPP1188).
- Writing a JSON log entry from `boot-log` lets multiple CCLK variants be compared after a sweep, even if the capturing session is interrupted.
- The conformance suite stayed at **575/575 PASS**; FPGA tooling changes remain isolated from the compiler path.

### What changed behavior
- `tri fpga bit-config` now prints warnings and supports CI assertions.
- `tri fpga smoke-gate` fails if the demo bitstream does not target `xc7a200tfgg676-1`, does not use SPI x1, or does not start up from CCLK.
- `tri fpga boot-log` now documents the JTAG-cable-disconnect step and persists results to JSON.
- `fpga/HARDWARE_SSOT.md` contains the H2 decision tree and the CCLK-variant protocol.
- W398 closes with H2 tooling complete; the actual cold-POR/CCLK sweep is deferred to W399.

### Patterns to reuse
- When a vendor bitstream field (e.g. `OSCFSEL`) is not publicly documented, provide a raw-value patch tool and a structured sweep protocol rather than guessing a MHz mapping.
- Capture every physical-diagnostic attempt in a machine-readable log (JSON) so that later waves can compare runs without re-running the experiment.
- Add explicit CI assertions for hardware-invariant register values (IDCODE, SPI width, startup clock) so regressions are caught board-less.
- When a physical action is unsafe or impossible to automate (disconnecting a cable), make the printed protocol the source of truth and record the user's follow-through in the log.

### Anti-patterns to avoid
- Do not claim a CCLK timing fix is verified without a physical cold-POR measurement; document the unknown MHz mapping and the required experiment.
- Do not silently patch a bitstream without warning about CRC invalidation; check for CRC register writes and surface the risk.
- Do not add new Python scripts on the verification critical path; extend existing helpers (`dump_bit_config.py`) and drive them through Rust CLI/tri.

## 2026-07-06 — Wave Loop 397 (FPGA SPI boot root-cause closure — boot-log, smoke gate, H1 likely ruled out)

### What worked
- Adding `tri fpga boot-log <bit>` kept the cold-POR experiment self-contained: it programs flash, prints the exact user-assisted power-cycle protocol, and runs `tri fpga stat --pre-jtag-reset` after the user presses ENTER.
- Adding `--repeat N` to `tri fpga stat` captured multiple consecutive STAT samples after power-on, making transient mode-bit or DONE behavior visible.
- Adding `tri fpga smoke-gate` and a Phase 3c in-runner check in `bootstrap/src/suite.rs` gives the FPGA path a board-less CI gate that runs `bit-config` and yosys synthesis on `fpga/verilog/ternary_mac_demo_top_200t.bit`.
- A controlled JTAG-reset experiment showed STAT=`0x5000190C` with `MODE=0b001` and `DONE=0`, strongly suggesting H1 (mode-pin sampling) is not the blocker.
- SRAM load of the same 200T bitstream reported `done 1`, confirming the bitstream itself is valid.
- Flash round-trip verify matched 9,730,548 bytes, confirming the write path is still bit-perfect.

### What changed behavior
- `tri fpga stat` now decodes and prints the `MODE` field so boot-mode diagnosis is explicit.
- `tri fpga boot-log` provides a reproducible cold-POR protocol and decision tree, removing the ambiguity of which commands to run in what order.
- The conformance suite now includes an FPGA board-less smoke gate; regressions in `tri fpga bit-config` or the demo Verilog will fail CI even without a physical board.
- `fpga/HARDWARE_SSOT.md` now contains the cold-POR decision tree, and `fpga/diagnostics/jtag_wiring.md` is explicitly deprecated.
- W397 closes with H1 likely ruled out and H2 (CCLK/SPI-startup timing or flash state after reset) as the leading hypothesis for W398.

### Patterns to reuse
- When a CLI command needs a physical user step (power-cycle), keep it interactive with clear printed instructions and a single keypress to continue; do not try to automate the unsafe physical action.
- Add a board-less smoke gate for every hardware-dependent feature so CI can catch regressions in generated artifacts even when the board is unavailable.
- Decode and print bit-field values (e.g. STAT `MODE`) explicitly; raw hex alone is not enough for root-cause diagnosis.
- After a JTAG reset fails with correct mode and no CRC/ID error, the next hypothesis is SPI/CCLK timing or flash wake-up state, not mode pins.

### Anti-patterns to avoid
- Do not claim a cold-POR experiment is complete without a true physical power-cycle; document the user-assisted step and the evidence that exists without it.
- Do not default the smoke gate to the smaller/older bitstream (`ternary_mac_demo_top.bit`) when the target board is the 200T; always use the part-matched artifact.
- Do not run yosys synthesis on a single demo file when the top module instantiates another local module; include all required Verilog sources in the smoke script.
- Do not leave stale docs with wrong IDCODEs and broken tool paths; either update them or add a prominent deprecation notice redirecting to the SSOT.

## 2026-07-06 — Wave Loop 396 (FPGA SPI boot debug — bit-config, round-trip verify, cold-POR diagnostics)

### What worked
- Implemented three CLI diagnostics in `cli/tri/src/fpga.rs` without touching the compiler: `--pre-jtag-reset` for `tri fpga stat`, `tri fpga bit-config <bit>`, and `tri fpga round-trip-verify <bit>`.
- Wrote `scripts/dump_bit_config.py` using the **prjxray Series-7 Type-1 packet layout** (register address in bits [26:13], word count in bits [10:0]) to decode COR0/COR1/IDCODE/CTL0/CTL1/BSPI and confirm the bitstream is SPI x1 with the correct IDCODE `0x03636093`.
- Used `openFPGALoader` with the Digilent FTDI cable (`digilent_hs2`) for program, dump, and STAT readback after discovering the Xilinx DLC10 cable (0x03FD) is not connected.
- Implemented `round-trip-verify` by aligning both the original .bit payload and the dumped flash payload at the sync word `0xAA995566`, accounting for the 7-series SPI preamble that openFPGALoader prepends.
- Cross-checked with the XC7A100T `blink_j26.bit` and observed `ID_ERROR=1` (STAT `0x5000890c`), confirming the FPGA does check IDCODE during flash boot and that the XC7A200T GF16 bitstream has the right IDCODE.

### What changed behavior
- `tri fpga stat` can now skip the openFPGALoader JTAG reset with `--pre-jtag-reset`, allowing a post-cold-POR STAT read before the FPGA is reset.
- `tri fpga bit-config` exposes 7-series configuration register values from any .bit file.
- `tri fpga round-trip-verify` gives a deterministic pass/fail for flash write-path integrity.
- `fpga/HARDWARE_SSOT.md` now states that FBG676 and FGG676 have identical pinout and documents the revised flash-boot diagnostic checklist.
- W396 closed as honest diagnostic gathering: H2 (bitstream config), H3 (round-trip corruption), and H4 (package chipdb) were ruled out; H1 (cold-POR mode sampling) remains unverified and requires a user-assisted physical power-cycle.

### Patterns to reuse
- When a 7-series .bit parser is needed, use the prjxray bit layout, not the higher-level UG470 register-field layout; the latter misplaces address/count bits and produces "no packets found".
- Align flash round-trip comparisons at the Xilinx sync word `0xAA995566`; openFPGALoader strips the ASCII header and inserts SPI preamble bytes.
- When the actual cable is an FTDI probe, treat openFPGALoader as the canonical tool and document that the DLC10 driver is not required.
- Record every physical measurement with a timestamp and power state, even if the result is "still no boot".

### Anti-patterns to avoid
- Do not compare flash dump bytes from offset 0 directly to .bit payload bytes from offset 0; the formats differ by header and preamble.
- Do not use `--enable-quad` / `--disable-quad` with the N25Q128 flash; it has no separate QE bit and openFPGALoader aborts.
- Do not write `Closes #NNNN` in a PR without first running `gh issue view NNNN`.
- Do not modify prjxray-db as a first diagnostic step when package pinout identity can be verified from primary Xilinx sources.

## 2026-07-05 — Wave Loop 394 (FPGA flash-boot diagnostics)

### What worked
- Adding `--enable-quad`, `--disable-quad`, and `--spi-buswidth` to `tri fpga program-flash` required only CLI plumbing in `cli/tri/src/fpga.rs`; no compiler changes.
- `tri fpga flash-status` was added as a best-effort diagnostic wrapper around `openFPGALoader -f --detect` because openFPGALoader does not expose a raw RDSR (0x05) read.
- Updating `fpga/HARDWARE_SSOT.md` to cover both mode-pin strapping and quad-mode / `SPI_BUSWIDTH` gives the user a clear checklist for the physical experiment.
- Conformance suite stayed at **575/575 PASS**; FPGA CLI changes do not affect the compiler conformance path.

### What changed behavior
- `tri fpga program-flash` now supports the openFPGALoader options that are most likely to fix the W393 boot-from-flash failure (quad-enable).
- The boot-from-flash root-cause hypothesis expanded from "mode pins only" to "mode pins OR quad-mode/SPI_BUSWIDTH mismatch".

### Patterns to reuse
- When a competitor (Sparkle/Verilean) has stronger formal verification, differentiate by closing the physical-demo loop: open-source toolchain → real board → non-volatile boot.
- When an external tool (openFPGALoader) lacks a needed subcommand, wrap the closest available command honestly and document the limitation instead of building a fragile workaround.
- Create the GitHub issue first, then `Closes #NNNN`.

## 2026-07-04 — Wave Loop 392 completion

### What worked
- Forward-appending W392 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSeventyPlus`, `AccumulateSixtyNineMinus`, `QuinquagintupleDuoCancellation`, `ZeroWeightTwentySevenPairClosure`) returned **575/575 PASS**.
- `t27c stats` now reports **13,939 tests**, **6,151 invariants**, and **1,010 benchmarks**.
- The IGLA CODER+RACE zero-failure streak advanced to **125 waves**.
- Created `docs/BRANCHING_MODEL.md` to document the three-tier branch model and opened master-alignment epic #1284 instead of starting a risky replay inside the wave-loop.
- Opened the real W392 issue (#1282) **before** writing `Closes #1282` in any commit or PR, following the W391 lesson.
- Squash-merged PR #1283 (`wave-loop-392` → `trinity-rust-rings`) without force-push; `origin/trinity-rust-rings` advanced cleanly to merge commit `66183ef23`.

### What changed behavior
- The `ternaryMac` generic ∀ count is now **312**.
- Pool A floor, CODER minimum, Pool B depth, and Integration depth each advanced by +1.
- `trinity-rust-rings` is now explicitly recognized as the long-lived IGLA integration branch; master-alignment is a separate epic requiring explicit approval.

### Patterns to reuse
- Create the GitHub issue first (`gh issue create`), capture the number, then write `Closes #NNNN`. This removes the risk of referencing a non-existent issue.
- For long-lived integration branches with large divergence from `master`, document the alignment as a separate epic rather than forcing it inside a routine wave-loop.
- Use squash-merge through GitHub UI/CLI as the normal update path for `trinity-rust-rings`; reserve force-push for emergency recovery only.

### Anti-patterns to avoid
- Do not start a `master`-alignment replay inside a wave-loop without explicit user approval, especially when hot `bootstrap` files have diverged.
- Do not force-push `trinity-rust-rings` as a routine workflow step.

## 2026-07-04 — Wave Loop 391 completion

### What worked
- Forward-appending W391 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyNinePlus`, `AccumulateSixtyEightMinus`, `QuinquagintupleUnoCancellation`, `ZeroWeightTwentySixPairClosure`) returned **575/575 PASS**.
- `t27c stats` now reports **13,885 tests**, **6,124 invariants**, and **1,010 benchmarks**.
- The IGLA CODER+RACE zero-failure streak advanced to **124 waves**.
- Completed W391 locally despite `gh` CLI being unauthenticated; documented the remote-cleanup debt in `docs/reports/WAVE_LOOP_391_SYNC_REPORT.md` instead of inventing issue numbers.

### What changed behavior
- The `ternaryMac` generic ∀ count is now **308**.
- Pool A floor, CODER minimum, Pool B depth, and Integration depth each advanced by +1.
- `.trinity/current-issue.md` now explicitly marks the W391 issue number as pending `gh` auth, replacing the incorrect #1290 reference.

### Patterns to reuse
- When GitHub API access is unavailable, continue the local wave work (proof/spec/seal/test/docs) but do **not** fabricate issue/PR numbers. Record the auth/cleanup debt for the next wave.
- The generator script pattern (`scripts/gen_wNNN.py` + `scripts/gen_wNNN_lean.py`) remains the fastest way to add a wave block and theorem set.

### Anti-patterns to avoid
- Do not write `Closes #NNNN` without verifying the issue exists via `gh issue view`.
- Do not stall an entire wave waiting for remote cleanup if the local proof/spec work can be completed and committed cleanly.
- Do not start new SPI flash attempts without first resolving the toolchain blocker.

## 2026-07-01 — Wave Loop 390 completion

### What worked
- Forward-appending W390 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyEightPlus`, `AccumulateSixtySevenMinus`, `QuinquagintupleCancellation`, `ZeroWeightTwentyFivePairClosure`) returned **575/575 PASS**.
- `t27c stats` now reports **13,831 tests**, **6,097 invariants**, and **1,010 benchmarks**.
- The IGLA CODER+RACE zero-failure streak advanced to **123 waves**.
- The W389 SPI flash workaround was verified to still function on this workstation (generic proxy copied to package-specific name), and the board's persistent bitstream remains valid.

### What changed behavior
- The `ternaryMac` generic ∀ count is now **304**.
- Pool A floor, CODER minimum, Pool B depth, and Integration depth each advanced by +1.
- SPI flash is operationally reproducible on this workstation but **not yet reproducible from a clean environment** because no package-specific `spiOverJtag_xc7a200tfgg676.bit.gz` proxy exists.

### Patterns to reuse
- When a multi-path task (build proxy) is blocked by missing toolchain artifacts, document each attempted path, the exact missing dependency, and the fallback workaround so the next wave can pick the cheapest unblocked entry point.
- Keep the proof-lattice momentum with 4 generic theorems per wave; cancellation RHS follows depth parity (identity for even depths, residual `.plus` for odd depths).

### Anti-patterns to avoid
- Do not let a blocked hardware subtask delay closing the wave; record the blocker and land the completed proof/spec work.
- Do not delete the working workaround file until a verified replacement exists.

## 2026-07-01 — Wave Loop 389 completion

### What worked
- Forward-appending W389 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtySevenPlus`, `AccumulateSixtySixMinus`, `QuadragintupleNovemCancellation`, `ZeroWeightTwentyFourPairClosure`) returned **575/575 PASS**.
- Achieved SPI flash programming of the ternary MAC demo bitstream by copying openFPGALoader's generic `spiOverJtag_xc7a200t.bit.gz` proxy to the package-specific name `spiOverJtag_xc7a200tfgg676.bit.gz`; the flash completed to 100% and a subsequent SRAM reload reported `done 1`.
- The hardware detection path was already correct per `fpga/HARDWARE_SSOT.md` (`idcode 0x03636093`, Digilent `digilent_hs2` cable).

### What changed behavior
- The `ternaryMac` generic ∀ count is now **300**.
- The IGLA CODER+RACE zero-failure streak is now **122 waves**.
- The ternary MAC demo bitstream is now persistent in SPI flash on the XC7A200T board.
- A local environment workaround (generic proxy renamed to package-specific) is required until a proper `spiOverJtag_xc7a200tfgg676.bit.gz` proxy exists.

### Patterns to reuse
- When openFPGALoader reports "missing device-package information" or a missing proxy file, inspect the installed `share/openFPGALoader/` directory and try the closest available proxy (generic device proxy or nearest package).
- After SPI flash, verify by loading the same bitstream into SRAM and checking `done 1`.
- Keep proof-lattice momentum with 4 generic theorems per wave; cancellation RHS follows depth parity.

### Anti-patterns to avoid
- Do not treat an openFPGALoader SPI-flash failure as a board or bitstream failure until the proxy file availability has been checked.
- Do not leave the SPI flash path undocumented; the workaround is environment-level and must be recorded for reproducibility.

## 2026-07-01 — Wave Loop 388 completion

### What worked
- Correcting the W388 generator scripts *before* resealing: `scripts/gen_w388.py` was re-written to detect and remove duplicate W387 blocks and emit a single proper W388 block; `scripts/gen_w388_lean.py` was corrected to use 66/65/48/23-pair variable counts matching the theorem names.
- Closing the multi-dimensional array feature with array-literal initialization required only a localized parser change in `bootstrap/src/compiler.rs` (`parse_array_literal`) because the existing W385 `StmtLocal` array-literal expansion and W387 flattening/index lowering already handled per-element register initialization and access.
- Forward-appending the corrected W388 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtySixPlus`, `AccumulateSixtyFiveMinus`, `QuadragintupleOctoCancellation`, `ZeroWeightTwentyThreePairClosure`) returned **575/575 PASS**.

### What changed behavior
- The `gen-verilog` backend now supports multi-dimensional function-local array-literal initialization (`var m : [2][3]u16 = [2][3]u16{...}`) in addition to numeric/variable indices, signed elements, and nested loops.
- The CI yosys smoke gate expanded from 55 to **56 targets** with the new `specs/scratch/w388_2d_local_array_init.t27` regression spec.
- The `ternaryMac` generic ∀ count is now **296**.
- The IGLA CODER+RACE zero-failure streak is now **122 waves**.

### Patterns to reuse
- When a generator copies the previous wave's block, always verify that every placeholder is bumped: wave number, internal identifiers (`wNNN_`), reference docs (`WAVE_LOOP_NNN_COOPERATION.md`), and comment references (`after WNNN`).
- When generating new Lean theorems, match the variable-count helpers to the theorem name and doc string; off-by-one errors are easy to introduce when reusing prior-wave helper calls.
- Reuse existing per-element lowering paths for aggregate initialization instead of adding a special-case emission path for higher-dimensional literals.

### Anti-patterns to avoid
- Do not run `t27c seal --save` before validating that generated test/invariant names are unique and wave-correct; duplicate identifiers can still pass the suite but leave misleading history.
- Do not treat parser changes as automatically safe for all specs; a small change to `parse_array_literal` changed AST shape for every array literal, so non-IGLA seals also needed regeneration.

## 2026-07-01 — Wave Loop 387 completion

### What worked
- Forward-appending W387 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyFivePlus`, `AccumulateSixtyFourMinus`, `QuadragintupleSeptemCancellation`, `ZeroWeightTwentyThreePairClosure`) returned **574/574 PASS**.
- Implementing multi-dimensional function-local arrays required parser-aware codegen changes: parse the full dimension list, flatten to per-element regs, and linearize nested index chains for both constant and variable access.
- Preserving the non-local-array constant-index fallback (`base_idx`) avoided regressions in module-level arrays and slice parameters; an initial miss caused 13 unexpected seal mismatches that were resolved before resealing.

### What changed behavior
- The `gen-verilog` backend now supports 2D function-local arrays (`var m : [2][3]u16`) with numeric, variable, signed-element, and nested-loop access.
- The CI yosys smoke gate expanded from 51 to **55 targets** with the four new W387 scratch specs.
- The `ternaryMac` generic ∀ count is now **292**.
- The IGLA CODER+RACE zero-failure streak is now **121 waves**.

### Patterns to reuse
- When flattening multi-dimensional arrays, compute linear offsets outer-to-inner with stride equal to the product of inner dimensions.
- For nested index chains, collect the chain once and reuse it for both read and write paths to keep the linearization consistent.
- Always preserve existing fallbacks when generalizing an indexing path; otherwise non-array index patterns regress.

### Anti-patterns to avoid
- Do not replace a specialized index path with a general one without checking non-array identifiers that relied on the old behavior.
- Do not regenerate seals until the full suite is green; unexpected mismatches are a signal of regressions, not just expected churn.

## 2026-07-01 — Wave Loop 386 completion

### What worked
- Forward-appending W386 blocks to all 27 IGLA specs and adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyFourPlus`, `AccumulateSixtyThreeMinus`, `QuadragintupleSexCancellation`, `ZeroWeightTwentyOnePairClosure`) returned **570/570 PASS**.
- The `for` loop over function-local arrays gap was closed with regression coverage only; the existing W384 variable-index lowering and W385 signed/init lowering already handled constant-bound (unrolled) and parameter-bound (Verilog `for`) cases correctly.
- Adding scratch specs for unsigned, signed, and parameter-bound loops expanded the yosys smoke gate from 48 to **51 targets** without touching the compiler backend.

### What changed behavior
- The `gen-verilog` backend now has smoke-gate coverage for function-local arrays inside `for` loops.
- The `ternaryMac` generic ∀ count is now **288**.
- The IGLA CODER+RACE zero-failure streak is now **120 waves**.

### Patterns to reuse
- Before implementing a perceived backend gap, generate a minimal scratch spec and run it through the existing pipeline; the feature may already work and only need regression coverage.
- For cancellation theorems, continue matching RHS to depth parity: even alternating depths collapse to `x`; odd depths leave a residual `ternaryMac x a (TernaryWeight.mk .plus)`.

### Anti-patterns to avoid
- Do not assume every cooperation-doc gap requires compiler changes; some gaps are purely coverage/test gaps.
- Do not let untracked scratch specs accumulate without seals; run `t27c seal --save` for each new spec as part of the wave close-out.

## 2026-07-01 — Wave Loop 385 completion

### What worked
- Generalizing function-local arrays to signed element types required no new codegen logic beyond regression specs; the existing `elem_signed` path in `bootstrap/src/compiler.rs` already emitted `signed [W-1:0]` regs.
- Implementing array-literal initialization for function-local arrays only required replacing the W384 TODO placeholder in `StmtLocal` with a loop that emits per-element scalar assignments.
- Forward-appending W385 blocks to all 27 IGLA specs, adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyThreePlus`, `AccumulateSixtyTwoMinus`, `QuadragintupleQuinqueCancellation`, `ZeroWeightTwentyPairClosure`), returned **567/567 PASS**.

### What changed behavior
- The `gen-verilog` backend now supports signed-element function-local arrays (`var temps : [4]i16`) and array-literal initialization (`var buf : [4]u16 = [4]u16{...}`).
- The CI yosys smoke gate expanded from 45 to **48 targets** with the three new W385 scratch specs.
- The `ternaryMac` generic ∀ count is now **284**.

### Patterns to reuse
- When lowering aggregate literals, expand them into scalar assignments at the declaration site rather than emitting a single unsupported aggregate expression.
- For cancellation theorems, match the RHS to the depth parity: even alternating depths collapse to `x`; odd depths leave a residual `ternaryMac x a (TernaryWeight.mk .plus)`.
- Reuse the existing scalar literal width-padding logic inside element-wise loops to keep generated widths consistent.

### Anti-patterns to avoid
- Do not assume all cancellation depths collapse to identity; verify parity before generating the RHS.
- Do not regenerate seals one-by-one in a hot loop if a batch reseal command becomes available; the per-file call overhead is acceptable but noisy.

## 2026-07-01 — Wave Loop 384 completion

### What worked
- Extending the function-local array lowering from numeric-literal-only indices to variable indices required only localized additions in `bootstrap/src/compiler.rs`: a per-function `local_arrays` registry, mux-chain emission in `ExprIndex`, and if-else-chain emission in `StmtAssign`.
- Applying keyword escape to the **full flattened token** (`buf_0`, `\buf_0 `) prevented the token-splitting bug that occurred when appending `_0` to an already-escaped identifier (`\buf `).
- Forward-appending W384 blocks to all 27 IGLA specs, adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyTwoPlus`, `AccumulateSixtyOneMinus`, `QuadragintupleQuattuorCancellation`, `ZeroWeightNineteenPairClosure`), returned **564/564 PASS**.

### What changed behavior
- The `gen-verilog` backend now supports variable-index access on function-local arrays (`var buf : [4]u16; return buf[idx];` and `buf[idx] = value;`) via per-element registers + priority mux/if-else chains.
- The CI yosys smoke gate expanded from 44 to **45 targets** with the new `specs/scratch/w384_variable_index.t27` regression spec.
- The `ternaryMac` generic ∀ count is now **280**.

### Patterns to reuse
- When lowering a language feature to per-element registers, handle variable-index read/write explicitly: do not rely on Verilog to infer function-local memories from scalar reg bit-selects.
- Always escape the complete flattened identifier token in the generated Verilog, not its components, to avoid whitespace/keyword tokenization issues.
- For mux-chain emission, keep a strict open/close parenthesis count: emit `array_size` opens for the comparisons plus `array_size` closes after the default value.

### Anti-patterns to avoid
- Do not store keyword-escaped base names in codegen metadata and then append suffixes; store the original name and re-escape the full flattened token at emission time.
- Do not hold a mutable borrow to a HashMap while recursively generating expression code inside the same struct; clone the needed metadata first.

## 2026-07-01 — Wave Loop 383 completion

### What worked
- Forward-appending W383 blocks to all 27 IGLA specs, adding 4 new `ternaryMac` generic ∀ theorems (`AccumulateSixtyOnePlus`, `AccumulateSixtyMinus`, `QuadragintupleDuoCancellation`, `ZeroWeightEighteenPairClosure`), and regenerating all affected seals returned **563/563 PASS**.
- Extending the W382 module-level array lowering to ROM literals (`const lut : [N]T = [N]T{...}`) and function-local arrays (`var tmp : [N]T`) required only localized changes in `gen_verilog_const`, `StmtLocal`, and `ExprIndex` in `bootstrap/src/compiler.rs`.
- Using a numeric-literal index rewrite for function-local arrays (`tmp_0`, `tmp_1`) kept the generated Verilog synthesizable through `yosys read_verilog -sv` without needing function-local memory inference.

### What changed behavior
- The `gen-verilog` backend now supports three closed array patterns: module-level RAM (`var mem : [N]T`), module-level ROM (`const lut : [N]T = [N]T{...}`), and function-local arrays with numeric-literal indices.
- The CI yosys smoke gate expanded from 43 to **44 targets** with the new `specs/scratch/w383_rom_array.t27` regression spec.

### Patterns to reuse
- When adding a new backend feature, pair it with a scratch regression spec that exercises both read and write paths; the in-runner smoke gate will then enforce the behavior automatically.
- Regenerate seals from the repo root (`/Users/playra/t27`) after any compiler change that affects generated-code hashes; `t27c seal --save <spec>` works on individual files for targeted resealing.

### Anti-patterns to avoid
- Do not emit array-literal syntax directly in Verilog (`localparam lut = [4]u16{...};`); always lower to a synthesizable memory declaration plus an `initial` block.
- Do not leave function-local array index expressions as scalar bit-selects; either rewrite numeric-literal indices to flattened regs or emit an explicit mux/case for variable indices.

## 2026-07-02 — Wave Loop 358 completion

### What worked
- Running `./scripts/tri` (via `t27c suite --repo-root .`) gives a single 546-check conformance gate; after cleaning 54 bare W347 blocks and regenerating seals from the repo root, the suite returned **546/546 PASS**.
- `env -u GH_TOKEN gh ...` is required when `GH_TOKEN` is set to an invalid token; the keyring-stored `gHashTag` account is usable once the env override is removed.
- `lake build Trinity.TernaryInference` isolates the IGLA proof module from pre-existing failures in physics modules (`H4Lagrangian`, `NeutrinoMasses`).

### What changed behavior
- `t27c seal --save` writes seals relative to the current working directory, not the repo root. Regenerating seals must be done from `/Users/playra/t27` or the suite will read stale seals.
- The Verilog backend is critically broken for ternary MAC generation; FPGA evidence sprint is now blocked on either a hand-written synthesis module or a backend fix in `bootstrap/src/compiler.rs`.

### Patterns to reuse
- Before each wave: build `t27c`, run `t27c suite --repo-root .`, inspect `git status`, and address any bare/dangling blocks before adding new wave content.
- For issue-gated commits: if `GH_TOKEN` is invalid, use `env -u GH_TOKEN gh issue create` and reference `Closes #N` in the commit message.
- Keep the Lean proof lattice in `TernaryInference.lean` at 4 new generic ∀ theorems per wave; probe accumulation depth first, with minus-lattice parity as fallback if `omega` saturates.

### Anti-patterns to avoid
- Do not remove bare blocks without immediately regenerating all affected seals; otherwise the conformance gate fails with spec_hash mismatches.
- Do not stage `.claude/settings.json` or session metadata into wave-loop commits; keep those in separate commits or leave them unstaged.

## 2026-07-02 — Wave Loop 359 completion

### What worked
- Forward-appending W359 blocks with `test`/`invariant` keywords, plus 4 new Lean 4 generic ∀ theorems (`AccumulateThirtyFivePlus`, `AccumulateThirtyFourMinus`, `DuodecupleCancellation`, `ZeroWeightReorderingClosure`), kept the suite at **546/546 PASS** and pushed the generic ∀ count to **180**.
- Hand-writing a synthesis-ready ternary MAC in `fpga/verilog/ternary_mac_synth.v` bypassed the broken Verilog backend. A self-checking testbench (`tb_ternary_mac.v`) passed 6/6 vectors and `yosys synth_xilinx` produced metrics: 32 LUT5, 32 FDCE, 11 CARRY4.
- Even-number cancellation depths (12 for W359) collapse cleanly to identity with alternating plus/minus weights; odd depths leave a residual `mac(x,a,.plus)` or `x` mismatch, so always prefer even cancellation depths when targeting identity.

### What changed behavior
- The project now has **FPGA synthesis evidence** documented in `docs/reports/FPGA_EVIDENCE_W359.md`. This is the first measured hardware artifact.
- `iverilog` must be invoked from the directory containing the `.v` files and outputs; the `vvp` file is written to CWD, so `cd fpga/verilog` before running the simulator.
- `yosys` scripting for metrics should not mix `abc -liberty` with custom scripts; `synth_xilinx -top ternary_mac_top; stat` is sufficient for Xilinx resource counts.

### Patterns to reuse
- Structure each wave as: spec blocks → Lean theorems → build & seal → conformance → report → cooperation variants → memory. This cadence allows predictable 24–48 hour turnaround.
- For cancellation theorems, use even-length alternating plus/minus chains to guarantee identity collapse; verify with `lake build Trinity.TernaryInference` before seal regeneration.
- Preserve a hand-written synthesis fallback module (`ternary_mac_synth.v`) whenever the generated Verilog backend is unreliable; it protects the FPGA evidence pipeline.

### Anti-patterns to avoid
- Do not append bare wave blocks without `test`/`invariant`/`bench` keywords; the L4 TESTABILITY law rejects them and the conformance gate fails.
- Do not attempt odd-depth identity cancellation theorems without first checking the expected residual weight; even depths are safer.
- Do not rely on the generated Verilog backend for hardware evidence until it passes `yosys -p 'read_verilog'` cleanly.

## 2026-07-02 — Wave Loop 360 completion

### What worked
- A 36-variable `simp+omega` accumulation theorem (`ternaryMacAccumulateThirtySixPlusGeneric`) built successfully in ~3.1 s, so the omega boundary is still linear at depth 36.
- Forward-appending W360 blocks and regenerating all 27 seals from `/Users/playra/t27` returned **546/546 PASS** immediately after the Lean build.
- Creating a board-ready wrapper (`ternary_mac_demo_top.v`) with a ring-oscillator clock and LED outputs produced a clean `yosys` synthesis result: 34 cells, 12 CARRY4 total, estimated 10 LCs.

### What changed behavior
- The Wukong V1 ternary MAC design is now **ready to route**: RTL, XDC constraints, and yosys JSON netlist are in `fpga/verilog/`.
- `nextpnr-xilinx` is **not installed** on the build host; Homebrew only ships `nextpnr-ice40`. The OpenXC7 toolchain must be built from source per `fpga/HARDWARE_SSOT.md` §8.
- Odd-depth cancellation theorems collapse to a single non-identity MAC (`mac(x,a,.plus)` for depth 13), so the statement must match the residual weight.

### Patterns to reuse
- For deep accumulation proofs, generate the Lean binder list with **space-separated variables**; Lean does not accept comma-separated binders.
- For board-ready wrappers, reuse the `blinky.v` ring-oscillator pattern and the R23/T23 LED pins from existing QMTech designs; pass `--ignore-loops` to nextpnr.
- When the bitstream toolchain is missing, commit the ready-to-route artifacts and the evidence document; do not let the missing tool block the formal wave.

### Anti-patterns to avoid
- Do not generate Lean theorem parameters with Python `", ".join()`; use spaces.
- Do not stage `.claude/scheduled_tasks*` or session metadata into wave commits.
- Do not commit generated simulation artifacts (`.vvp`, intermediate `.json`) unless they are explicitly part of the deliverable.

## 2026-07-02 — Wave Loop 361 completion

### What worked
- `boost-python3` had to be actually installed (`brew install boost-python3`); `brew --prefix boost-python3` existing was not enough for CMake to find `Boost::Python 3.x`.
- Building `nextpnr-xilinx` with `-DARCH=xilinx -DUSE_OPENMP=OFF -DCMAKE_CXX_FLAGS="-I$(brew --prefix eigen)/include/eigen3"` succeeded on macOS arm64 with only deprecation/format warnings.
- `bbaexport.py` + `bbasm` produced a 152 MB `xc7a100tfgg676.bin` chipdb in ~1 minute.
- The full OpenXC7 flow yosys → nextpnr → fasm2frames → xc7frames2bit produced a **valid 3.6 MB Xilinx BIT file** for `ternary_mac_demo_top` on the first attempt.
- `nextpnr-xilinx` reported Fmax **643.92 MHz** for the ring-oscillator clock with 4 warnings and 0 errors.

### What changed behavior
- Trinity now has a **generated bitstream** for a formally-grounded ternary MAC, closing the "no silicon evidence" strategic vulnerability.
- The remaining hardware step is purely mechanical: connect the board + DLC10 cable and run `dlc10 sram ternary_mac_demo_top.bit`.
- The OpenXC7 toolchain is now available under `/tmp/openxc7-build/`; for reproducibility it should be moved to a permanent location (e.g. `~/opt/openxc7` or documented in `fpga/HARDWARE_SSOT.md`).

### Patterns to reuse
- Document the exact toolchain versions and build flags; future waves will need to reproduce this flow.
- When a tool is missing on macOS, check `brew list` and `brew info` before assuming the package is installed; `brew --prefix` can lie by returning a path for an uninstalled formula.
- For board flash attempts, always build `dlc10` first and run `dlc10 idcode` to confirm cable/board presence before claiming silicon validation.

### Anti-patterns to avoid
- Do not claim "silicon verified" without an actual board load and `DONE=HIGH`/LED observation.
- Do not leave the OpenXC7 toolchain only in `/tmp`; either persist it or document how to rebuild it.
- Do not forget to set `PYTHONPATH` when invoking `fasm2frames.py`; otherwise `ModuleNotFoundError: No module named 'prjxray'`.

## 2026-07-01 — Wave Loop 362 completion

### What worked
- Forward-appending W362 blocks to all 27 IGLA specs with `scripts/gen_w362.py` and regenerating all 27 seals from `/Users/playra/t27` returned **546/546 PASS** immediately after the Lean build.
- A 38-variable `simp+omega` accumulation theorem (`ternaryMacAccumulateThirtyEightPlusGeneric`) built successfully in **3.5 s**, so the omega boundary is still linear at depth 38.
- The quindecuple cancellation theorem (depth-15 residual `mac(x,a,.plus)`) and zero-weight quintuple closure theorem both built without new lemmas.
- The `dlc10` driver was rebuilt quickly with `cargo build --release -p dlc10` and is ready for the board flash once the QMTech Wukong V1 / Xilinx Platform Cable USB II is connected.

### What changed behavior
- The generic ∀ count across Trinity Lean modules reached **192** (184 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 generic theorems in `TernaryMac.lean`).
- The bitstream remains ready (`fpga/verilog/ternary_mac_demo_top.bit`, 3.6 MB), but the board flash is **blocked by missing hardware connectivity** (`DLC10 cable not found`).
- The W362 deliverable is therefore "silicon-ready" rather than "silicon-verified".

### Patterns to reuse
- For W363, reuse the same generator pattern and Lean theorem script; only the binder count and cancellation depth change.
- Always run `dlc10 idcode` before attempting `dlc10 sram`; idcode failure is a clear hardware-availability signal that should be documented, not hidden.
- When a wave includes both formal extension and hardware validation, complete and verify the formal work first so the hardware attempt does not compromise the zero-IGLA-failure streak.

### Anti-patterns to avoid
- Do not claim "board flashed" when only the bitstream exists; distinguish "generated", "loaded", and "observed running".
- Do not let a hardware blocker delay the spec/Lean/seal/report cadence; ship the formal deliverables and document the blocker.
- Do not commit generator scripts that are still one-off prototypes as part of the main wave commit unless they have been reviewed as tooling.

## 2026-07-01 — Wave Loop 363 completion

### What worked
- Reused `scripts/gen_w363.py` and `scripts/gen_w363_lean.py` to append W363 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.6 s**.
- `ternaryMacAccumulateThirtyNinePlusGeneric` (`a+b+...+am`) pushed the accumulation boundary to **39 variables**, still within the linear `simp+omega` regime.
- `ternaryMacSexdecupleCancellationGeneric` (depth-16 alternating plus/minus) collapsed cleanly to identity, confirming even-depth cancellation remains the safe default.
- `dlc10 idcode` was retried and the failure was documented as a hardware-availability blocker rather than a regression.

### What changed behavior
- Generic ∀ count reached **196** (188 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **97 waves** (twenty-third consecutive zero-failure wave).
- The W363 report and cooperation variants explicitly distinguish "bitstream generated" from "silicon physically observed" to avoid false claims.

### Patterns to reuse
- For cancellation theorems, keep alternating plus/minus weights and even depth to guarantee `= x` collapse without residual-weight adjustments.
- Continue the 4-theorem-per-wave cadence in `TernaryInference.lean`: accumulation probe, minus-lattice parity, cancellation depth, zero-weight closure.
- Document hardware blockers in a dedicated evidence file (`docs/reports/FPGA_EVIDENCE_W<N>.md`) so the load procedure is ready when the cable/board is available.

### Anti-patterns to avoid
- Do not modify a generator script with `sed` shortcuts without running it on a scratch copy first; the first `gen_w363.py` draft corrupted the expected-wave check.
- Do not let a single hardware blocker block the full wave deliverable; finalize the formal path and ship the report.
- Do not claim a theorem reaches identity unless the Lean statement literally ends in `= x` or matches the verified residual.

## 2026-07-01 — Wave Loop 364 completion

### What worked
- Reused `scripts/gen_w364.py` and `scripts/gen_w364_lean.py` to append W364 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.8 s**.
- `ternaryMacAccumulateFortyPlusGeneric` pushed the accumulation boundary to **40 variables**, still in the linear `simp+omega` regime.
- `ternaryMacSeptendecupleCancellationGeneric` (depth-17) correctly collapsed to residual `mac(x, a, .plus)`; the Lean statement matched the odd-depth residual exactly.
- A narrow, safe `gen_verilog` fix for binary literals (`0b...` → `N'b...`) landed in `bootstrap/src/compiler.rs` without regressions.

### What changed behavior
- Generic ∀ count reached **200** (192 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **98 waves** (twenty-fourth consecutive zero-failure wave).
- The `gen-verilog` backend now emits sized Verilog for binary literals; four larger lowering defects from #1245 are catalogued in `docs/reports/WAVE_LOOP_364_REPORT.md`.
- Board flash remains blocked by missing DLC10 cable/board; the failure is documented in `docs/reports/FPGA_EVIDENCE_W364.md`.

### Patterns to reuse
- For risky compiler changes, prefer narrow literal/formatting fixes over parser rewrites; parser changes can cause 100+ conformance regressions.
- Probe project weak points (e.g. #1245, #1246) during each wave and either fix, document, or file a reproduction; do not let them age silently.
- Keep the report/cooperation-variants cadence: `WAVE_LOOP_N_REPORT.md` + `WAVE_LOOP_N_COOPERATION.md` before the wave commit.

### Anti-patterns to avoid
- Do not attempt broad `parse_const_decl` / `skip_to_next_top_level` parser fixes without a staged branch and a full 546-spec conformance run.
- Do not delete generator scripts after a single wave if they are parameterized by wave number; they can be copied and updated.
- Do not claim identity cancellation at odd depths without first proving the residual equals the intended right-hand side.

## 2026-07-01 — Wave Loop 365 completion

### What worked
- Reused `scripts/gen_w365.py` and `scripts/gen_w365_lean.py` to append W365 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.8 s**.
- `ternaryMacAccumulateFortyOnePlusGeneric` pushed the accumulation boundary to **41 variables**, still in the linear `simp+omega` regime.
- `ternaryMacOctodecupleCancellationGeneric` (depth-18) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- Created `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`, giving every remaining #1245 defect an exact reproduction command and a tentative root-cause note.

### What changed behavior
- Generic ∀ count reached **204** (196 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **99 waves** (twenty-fifth consecutive zero-failure wave).
- IGLA totals: **7,618 tests**, **2,880 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W365.md`.

### Patterns to reuse
- For IGLA seal regeneration, map seal file names (hyphenated) to spec file names (underscore) when scripting; `t27c seal --save` normalizes the output file name.
- When a compiler fix is risky, ship a reproduction/roadmap document in the same wave; do not let the inability to fix silently erase the finding.
- Keep even-depth cancellation theorems for identity collapse; use odd-depth theorems only when the residual is explicitly verified.

### Anti-patterns to avoid
- Do not attempt to fix `is_top_level_start()` by adding `KwConst`/`KwVar` without tracking nested-block context; it breaks error recovery inside `test`/`invariant`/`bench` blocks.
- Do not leave `gen-verilog` defects without concrete repro commands; future waves will forget the exact failure mode.
- Do not claim "silicon verified" without `dlc10 idcode` success and a loaded bitstream observation.

## 2026-07-01 — Wave Loop 366 completion

### What worked
- Reused `scripts/gen_w366.py` and `scripts/gen_w366_lean.py` to append W366 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.1 s**.
- `ternaryMacAccumulateFortyTwoPlusGeneric` pushed the accumulation boundary to **42 variables**, still in the linear `simp+omega` regime.
- `ternaryMacNovemdecupleCancellationGeneric` (depth-19) correctly collapsed to residual `mac(x, a, .plus)`; the Lean statement matched the odd-depth residual exactly.
- Regenerated all 27 IGLA seals with the hyphen-to-underscore mapping; no manual seal edits were needed.

### What changed behavior
- Generic ∀ count reached **208** (200 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **100 waves** (twenty-sixth consecutive zero-failure wave).
- IGLA totals: **7,880 tests**, **2,950 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W366.md`.
- The `gen-verilog` backend remained unchanged; #1245 defects are still reproducible and documented.

### Patterns to reuse
- For 42-variable accumulations, the `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega` pattern remains sufficient.
- For odd-depth cancellation theorems, keep the residual explicit in both the Lean theorem name and statement to avoid identity/residual confusion.
- Re-run the full 546-spec conformance suite immediately after seal regeneration; seal mismatches are the only expected failure mode after a wave block append.

### Anti-patterns to avoid
- Do not land a broad `gen-verilog` fix in the same wave as a formal milestone unless it has a narrow, regression-free path; ship the reproduction document instead.
- Do not report the previous wave's generic ∀ count from memory when the Lean file can be grepped directly; exact counts prevent inflated or deflated claims.
- Do not skip `dlc10 idcode` just because earlier waves failed; retry each wave to keep the evidence trail current.

## 2026-07-01 — Wave Loop 367 completion

### What worked
- Reused `scripts/gen_w367.py` and `scripts/gen_w367_lean.py` to append W367 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.4 s**.
- `ternaryMacAccumulateFortyThreePlusGeneric` pushed the accumulation boundary to **43 variables**, still in the linear `simp+omega` regime.
- `ternaryMacVigintupleCancellationGeneric` (depth-20) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- Landed a safe `gen-verilog` sub-fix: positive hex literals in scalar `const` declarations are now padded to the declared type width (e.g. `u16 = 0x1` emits `16'h1`). The fix passed the full 546-spec conformance suite without requiring seal regeneration.

### What changed behavior
- Generic ∀ count reached **212** (204 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **101 waves** (twenty-seventh consecutive zero-failure wave).
- IGLA totals: **7,934 tests**, **2,977 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W367.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 2 (`0x` width) is fixed for scalar consts; defects 1/3/4/5 remain.

### Patterns to reuse
- For safe compiler sub-fixes, prefer narrow literal-emission changes over parser rewrites; they are the only kind that can land without mass seal regeneration.
- When a `gen-verilog` fix changes no currently-emitting output, the full conformance suite will stay green without regenerating all seals — but verify this explicitly before claiming the fix is regression-free.
- Keep the 4-theorem cadence: accumulation probe, minus-lattice parity, cancellation depth, zero-weight closure dimension.

### Anti-patterns to avoid
- Do not try to fix `gen-verilog` defect 1 (only first const emits) with a one-line parser change; it requires nested-block context tracking to avoid breaking error recovery.
- Do not omit a scratch-spec test for a compiler fix just because the full suite is green; the suite may not exercise the changed code path.
- Do not let a hardware blocker delay the formal + compiler sub-fix cadence; ship the deliverables and document the blocker.

## 2026-07-01 — Wave Loop 368 completion

### What worked
- Reused the generator pattern (`scripts/gen_w368.py` and `scripts/gen_w368_lean.py`) to append W368 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **547/547 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.5 s**.
- `ternaryMacAccumulateFortyFourPlusGeneric` pushed the accumulation boundary to **44 variables**; build time stayed flat, confirming `simp+omega` still scales linearly.
- `ternaryMacVigintiunupleCancellationGeneric` (depth-21) correctly collapsed to residual `mac(x, a, .plus)`, continuing the odd-depth residual pattern.
- Corrected the `zero_weight_closure` helper: it now counts the plus-weight activation (`total = before + 1 + after`), so `ternaryMacZeroWeightUndecupleClosureGeneric` truly has 10 zero-weight MACs around 1 plus-weight MAC (11 variables).
- Landed a second safe `gen-verilog` sub-fix: positive hex literals are now padded to the declared width in scalar `const`, `var`, `let` (StmtLocal), and `return` contexts. A scratch spec `specs/scratch/w368_hex_width.t27` and `yosys read_verilog` verify the emitted RTL.
- Regenerated all affected seals (27 IGLA + 4 non-IGLA + 1 scratch) and reached 547/547 PASS.

### What changed behavior
- Generic ∀ count reached **216** (208 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **102 waves** (twenty-eighth consecutive zero-failure wave).
- IGLA totals: **7,780 tests**, **2,991 invariants** (direct keyword counts across the 27 core specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W368.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 2 (`0x` width) now covers const/var/let/return on `trinity-rust-rings`; defects 1/3/4/5 remain. The full #1245 fix set already exists on `master` (commit `701d79b3b`) but was not merged into the wave-loop branch due to history divergence.

### Patterns to reuse
- When extending a literal-emission fix to new contexts, add the target-type context to the codegen state (e.g., `current_fn_return_type`) rather than changing the global expression emitter signature.
- After any `gen-verilog` change, run `t27c seal --save` for every spec whose `gen_hash_verilog` mismatches; the suite will name them explicitly.
- For zero-weight closure theorems, always verify the generated Lean expression by inspecting the plus-weight index; the helper's `total` must include the plus activation or the advertised depth is off by one.

### Anti-patterns to avoid
- Do not merge `master` into a long-lived wave-loop branch just to grab a backend fix unless you have bandwidth to resolve the diverged history and reseal everything.
- Do not leave scratch regression specs unsealed; either seal them or remove them before the final conformance run.
- Do not skip `dlc10 idcode` even when failure is expected; the evidence document needs the exact stderr each wave.

## 2026-07-02 — Wave Loop 369 completion

### What worked
- Reused `scripts/gen_w369.py` and `scripts/gen_w369_lean.py` to append W369 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **548/548 PASS** and `lake build Trinity.TernaryInference` succeeded in **~5.0 s**.
- `ternaryMacAccumulateFortyFivePlusGeneric` pushed the accumulation boundary to **45 variables**; `simp+omega` remains in the linear regime.
- `ternaryMacDuovigintupleCancellationGeneric` (depth-22) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightDuodecupleClosureGeneric` uses 6 zero-weight MACs before and 6 zero-weight MACs after a plus-weight MAC (12 + 1 = 13 variables); the corrected `zero_weight_closure` helper from W368 was preserved.
- Landed the third consecutive safe `gen-verilog` sub-fix: positive binary literals (`0b...`) are now padded to the declared width in scalar `const`, `var`, `let` (StmtLocal), and `return` contexts, mirroring the W368 `0x` fix. A scratch spec `specs/scratch/w369_bin_width.t27` and `yosys read_verilog` verify the emitted RTL.

### What changed behavior
- Generic ∀ count reached **220** (212 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **103 waves** (twenty-ninth consecutive zero-failure wave).
- Conformance suite now evaluates **548 specs** (546 canonical IGLA + 1 non-IGLA + 1 scratch regression spec).
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W369.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defects 2/2b (`0x` and `0b` scalar width padding) are fixed; defects 1/3/4/5 remain.

### Patterns to reuse
- For literal-width guards, use the same shape for `0x` and `0b` with only the bit-scaling changed: `hex.len() * 4` vs `bin.len()`.
- Add scratch regression specs for every `gen-verilog` sub-fix and run `yosys read_verilog` before regenerating all seals; this catches regressions without waiting for the full suite.
- For W370, the recommended cooperation variant is B (formal + board retry + one safe backend sub-fix or CI smoke gate).

### Anti-patterns to avoid
- Do not add a scratch spec without either sealing it or removing it before the final suite run; an unsealed spec will produce a suite failure.
- Do not claim the binary-width fix covers non-scalar contexts (arrays, struct fields) until a dedicated reproduction proves it.
- Do not merge the full `master` #1245 fix set into `trinity-rust-rings` during a wave unless the diverged history and seal set are reconciled first.

## 2026-07-02 — Wave Loop 370 completion

### What worked
- Reused `scripts/gen_w370.py` and `scripts/gen_w370_lean.py` to append W370 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **549/549 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.8 s**.
- `ternaryMacAccumulateFortySixPlusGeneric` pushed the accumulation boundary to **46 variables**; `simp+omega` remains in the linear regime.
- `ternaryMacTresvigintupleCancellationGeneric` (depth-23) correctly collapsed to residual `mac(x, a, .plus)`, continuing the odd-depth residual pattern.
- `ternaryMacZeroWeightTredecupleClosureGeneric` uses 6 zero-weight MACs before and 7 zero-weight MACs after a plus-weight MAC (13 closure size, 14 variables).
- Fixed `gen-verilog` defect 1 (only first `const` emits) in `bootstrap/src/compiler.rs` by removing the early return in `parse_const_decl`. The fix required **mass seal regeneration (~156 seals)** because many specs now emit more `const` declarations than before.
- Verified the B1 fix with scratch spec `specs/scratch/w370_const_order.t27` and `yosys read_verilog` before running the full suite.

### What changed behavior
- Generic ∀ count reached **224** (216 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **104 waves** (thirtieth consecutive zero-failure wave).
- IGLA totals: **12,696 tests**, **5,549 invariants** (full repo keyword counts; note that earlier waves reported IGLA-only subsets while W370 reports all specs).
- Conformance suite now evaluates **549 specs** (546 canonical IGLA + 2 non-IGLA + 1 scratch regression spec).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W370.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 1 (multiple `const` declarations) is fixed on `trinity-rust-rings`; defects 3/4/5 remain.

### Patterns to reuse
- For parser fixes that change how many top-level declarations are parsed, expect mass seal regeneration; script `t27c seal --save` over every mismatched seal and re-run the full suite before claiming green.
- When generating Lean binder lists beyond 26 variables, skip Lean keywords (`at`, `by`, `do`, `if`, `in`, `or`, `to`) so the 46th+ variables do not produce `unexpected token` errors.
- For W370-level cooperation variants, keep Variant B as the recommended path: formal + one safe backend sub-fix + board retry.

### Anti-patterns to avoid
- Do not try to fix defect 1 by adding `KwConst` to `is_top_level_start()`; that breaks error recovery inside `test`/`invariant`/`bench` blocks. The correct fix is inside `parse_const_decl` itself.
- Do not commit a parser fix without a dedicated scratch spec that exercises the previously broken code path; the full suite may not contain a multi-const module.
- Do not trust repository-wide test/invariant counts from prior-wave memory; run `t27c stats` to get current totals.

## 2026-07-02 — Wave Loop 371 completion

### What worked
- Reused `scripts/gen_w371.py` and `scripts/gen_w371_lean.py` to append W371 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **551/551 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFortySevenPlusGeneric` pushed the accumulation boundary to **47 variables**; `simp+omega` remains in the linear regime.
- `ternaryMacQuattuorvigintupleCancellationGeneric` (depth-24) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightQuattuordecupleClosureGeneric` uses 7 zero-weight MACs before and 7 zero-weight MACs after a plus-weight MAC (14 closure size, 15 variables).
- Fixed a real `gen-verilog` lowering defect: Verilog keyword identifier collision. Added `verilog_keywords()` and `verilog_safe_identifier()` helpers in `bootstrap/src/compiler.rs` so identifiers like `task` are escaped as `\task `. This made `specs/igla/coder/benchmark.t27` pass `yosys read_verilog` for the first time.
- Verified the fix with scratch spec `specs/scratch/w371_verilog_keyword.t27` and `yosys read_verilog` before mass resealing.

### What changed behavior
- Generic ∀ count reached **228** (220 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **105 waves** (thirty-first consecutive zero-failure wave).
- IGLA totals: **12,752 tests**, **5,576 invariants** across full repo.
- Conformance suite now evaluates **551 specs** (546 canonical IGLA + 2 non-IGLA + 3 scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W371.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: keyword collision fixed; early return re-characterized as a semantic if-else chaining bug; `let` destructuring added as a new tracked defect.

### Patterns to reuse
- For gen-verilog fixes, run a yosys sweep across IGLA specs to find concrete failures before choosing which defect to fix; prior-wave repro descriptions can become stale.
- Use Verilog escaped identifiers (`\name `) for keyword collisions rather than renaming, so the emitted source remains human-readable and the original t27 name is preserved.
- After any change to identifier emission in `gen_verilog_expr` or `gen_verilog_fn`, expect mass seal regeneration across all specs.

### Anti-patterns to avoid
- Do not assume a documented gen-verilog defect still reproduces exactly as written; verify with a fresh generated output and `yosys read_verilog` before implementing.
- Do not fix keyword collisions by appending a suffix to the t27 name; that would break cross-reference consistency. Escaped identifiers keep the name unchanged.
- Do not leave a scratch regression spec unsealed; an unsealed spec will produce a suite failure.

## 2026-07-02 — Wave Loop 372 completion

### What worked
- Reused the generator pattern (`scripts/gen_w372.py`, `scripts/gen_w372_lean.py`) to append W372 blocks and 4 new generic ∀ theorems; `t27c suite` returned **552/552 PASS** and `lake build Trinity.TernaryInference` succeeded in **~5.2 s**.
- `ternaryMacAccumulateFortyEightPlusGeneric` pushed the plus-accumulation boundary to **48 variables** without timeout, confirming the `simp+omega` regime remains linear at this depth.
- Extended W371 keyword-escape fix to local variable declarations and struct-field register names in `bootstrap/src/compiler.rs`. A scratch spec with local variables named `task` and `wire` now passes `yosys read_verilog -sv` and `synth_xilinx`.
- Scripted mass seal regeneration: 177 non-IGLA seals (compiler change) + 27 IGLA seals (new W372 blocks) + 1 scratch seal, ending at 0 mismatches.

### What changed behavior
- Generic ∀ count reached **232** (224 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **106 waves** (thirty-second consecutive zero-failure wave).
- IGLA totals: **12,804 tests**, **5,603 invariants** across full repo.
- Conformance suite now evaluates **552 specs** (27 IGLA + non-IGLA + scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W372.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: keyword collision extended to underscore-delimited keyword components; local-variable and struct-field emission marked fixed; `let` destructuring remains the highest-priority open defect.

### Patterns to reuse
- When extending keyword escaping, detect keyword components at underscore boundaries, not just exact matches. Verilog treats `task_foo` as a keyword followed by an identifier, so it must be escaped as `\\task_foo `.
- After a compiler change that affects identifier emission, reseal all specs in two passes: first non-IGLA, then IGLA after spec blocks land, to avoid redundant resealing.
- Keep a scratch spec for each backend fix; `yosys read_verilog -sv` is a stronger verification than parse/typecheck alone.

### Anti-patterns to avoid
- Do not attempt a full `let` destructuring fix inside a single wave; it requires parser-level tuple-pattern support or a statement-level pattern-match pass. Document and defer.
- Do not skip sealing a scratch spec before running the full suite.
- Do not commit mass seal changes without a final `t27c suite` run; even a single stale seal fails the conformance gate.

## 2026-07-01 — Wave Loop 373 completion

### What worked
- Reused the generator pattern (`scripts/gen_w373.py`, `scripts/gen_w373_lean.py`) to append W373 blocks and 4 new generic ∀ theorems; `t27c suite` returned **553/553 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFortyNinePlusGeneric` pushed the plus-accumulation boundary to **49 variables** without timeout, confirming the `simp+omega` regime still holds at depth 49.
- `ternaryMacSesvigintupleCancellationGeneric` (depth-26) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightSexdecupleClosureGeneric` uses 8 zero-weight MACs before and 8 zero-weight MACs after a plus-weight MAC (16 closure size, 17 variables).
- Fixed a subtle tokenization bug in the W372 keyword-escape extension: struct-field register names are now built as the full flattened token (`word_reg`) before escaping, so `\word_reg ` is emitted instead of the invalid `word_\reg `. The same correction was applied to `ExprFieldAccess` in `gen_verilog_expr`.
- Added scratch spec `specs/scratch/w373_struct_field_keyword.t27` with keyword fields `reg` and `wire`; it passes `yosys read_verilog -sv` + `synth_xilinx`.
- Scripted mass seal regeneration: 23 non-IGLA seals (compiler change) + 27 IGLA seals (new W373 blocks) + 1 scratch seal, ending at 0 mismatches.

### What changed behavior
- Generic ∀ count reached **236** (228 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **107 waves** (thirty-third consecutive zero-failure wave).
- IGLA totals: **12,862 tests**, **5,632 invariants** across full repo.
- Conformance suite now evaluates **553 specs** (27 IGLA + non-IGLA + scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W373.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: struct-field keyword collision now fully tokenization-correct; `let` destructuring remains the highest-priority open defect.

### Patterns to reuse
- When concatenating an escaped identifier with a prefix, escape the **entire resulting token**, not the suffix in isolation. Verilog tokenization starts the escaped identifier at the backslash, so `prefix_\suffix` is parsed as two identifiers.
- After any change to `gen_verilog_expr` identifier emission, run a targeted yosys sweep on the scratch spec before the full suite; it is much faster than resealing and then discovering a syntax error.
- Keep the per-wave theorem budget at 4 generic ∀ theorems; depth-49 plus accumulation is still inside the practical elaboration budget.

### Anti-patterns to avoid
- Do not apply `verilog_safe_identifier()` to a component and then concatenate a prefix; always apply it to the complete identifier token.
- Do not assume a W372-level fix is tokenization-correct just because it looks right in generated text; verify with `yosys read_verilog -sv`.
- Do not leave the FPGA retry undocumented; even a missing-cable result is evidence and belongs in `docs/reports/FPGA_EVIDENCE_W*.md`.

## 2026-07-01 — Wave Loop 374 completion

### What worked
- Reused the generator pattern (`scripts/gen_w374.py`, `scripts/gen_w374_lean.py`) to append W374 blocks and 4 new generic ∀ theorems; `t27c suite` returned **554/554 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyPlusGeneric` pushed the plus-accumulation boundary to **50 variables** without timeout, confirming the `simp+omega` regime still holds at depth 50.
- `ternaryMacSeptemvigintupleCancellationGeneric` (depth-27) correctly collapsed to residual `mac(x, a, .plus)`, confirming odd-depth cancellation statements are still safe.
- `ternaryMacZeroWeightSeptendecupleClosureGeneric` uses 8 zero-weight MACs before and 8 zero-weight MACs after a plus-weight MAC (16 closure size, 17 variables).
- Extended keyword-escape fix to module-level `const` and `var` declarations in `bootstrap/src/compiler.rs`. Top-level declarations named `wire` or `reg` now emit escaped identifiers and parse cleanly through `yosys read_verilog -sv` + `synth_xilinx`.
- Added scratch spec `specs/scratch/w374_module_keyword.t27` with top-level const `wire` and var `reg`.
- Scripted mass seal regeneration: 7 non-IGLA seals (compiler change) + 27 IGLA seals (new W374 blocks) + 1 scratch seal, ending at 0 mismatches.

### What changed behavior
- Generic ∀ count reached **240** (232 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **108 waves** (thirty-fourth consecutive zero-failure wave).
- IGLA totals: **12,917 tests**, **5,660 invariants** across full repo.
- Conformance suite now evaluates **554 specs** (27 IGLA + non-IGLA + scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W374.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: module-level const/var keyword collision fixed; `let` destructuring remains the highest-priority open defect.

### Patterns to reuse
- The `simp+omega` accumulation proof remains practical at depth 50; continue probing one additional variable per wave while build time stays under ~10 s.
- For module-level keyword collisions, apply `verilog_safe_identifier()` directly where the `localparam` / `reg` identifier is emitted, including array-element indexed names.
- Keep resealing in two passes (non-IGLA first, then IGLA) after any compiler change to minimize redundant work.

### Anti-patterns to avoid
- Do not emit a module-level identifier before checking it against `verilog_safe_identifier()`; `localparam wire = ...` is a Verilog syntax error.
- Do not run the full suite only once after a compiler change; the first run reveals seal mismatches, the second run after resealing confirms zero failures.
- Do not skip yosys verification for a new scratch spec; parse/typecheck success does not guarantee the generated Verilog is synthesizable.

## 2026-07-03 — Wave Loop 375 completion

### What worked
- Reused the generator pattern (`scripts/gen_w375.py`, `scripts/gen_w375_lean.py`) to append W375 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **555/555 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyOnePlusGeneric` pushed the plus-accumulation boundary to **51 variables** without timeout, confirming the `simp+omega` regime still holds at depth 51.
- `ternaryMacOctovigintupleCancellationGeneric` (depth-28) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightOctodecupleClosureGeneric` uses 9 zero-weight MACs before and 9 zero-weight MACs after a plus-weight MAC (18 closure size, 19 variables).
- Fixed `gen-verilog` Defect 3 (early-return if-else chaining) in `bootstrap/src/compiler.rs`. Contiguous bare-if early-return statements are now emitted as a single Verilog `if ... else if ... else` chain, preventing later unconditional assignments from overwriting earlier return values. Verified with scratch spec `specs/scratch/w375_early_return.t27` and `yosys read_verilog -sv`.
- Pivoted from the originally planned `let` destructuring fix after discovering it depends on missing tuple-return function generation; documented the blocker in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- Scripted mass seal regeneration: 81 mismatched seals (compiler change + new W375 blocks + scratch) resealed and verified to 0 mismatches.

### What changed behavior
- Generic ∀ count reached **244** (236 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **109 waves** (thirty-fifth consecutive zero-failure wave).
- IGLA totals: **12,971 tests**, **5,687 invariants** across full repo.
- Conformance suite now evaluates **555 specs** (27 IGLA + non-IGLA + scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W375.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 3 fixed; Defect 6 re-triaged as blocked by tuple-return generation; Defect 4 is now the highest-priority wave-safe open defect.

### Patterns to reuse
- For control-flow fixes, walk the function body statement list and collapse contiguous matching statements; leave non-matching statements on the original code path to keep the change regression-free.
- When a planned backend fix turns out to depend on a larger missing feature (tuple-return functions), pivot to the next highest-priority self-contained defect and document the dependency clearly.
- After a compiler change that affects generated Verilog, expect a broad seal mismatch wave; capture the list from `t27c suite` and batch `t27c seal --save` from the repo root.

### Anti-patterns to avoid
- Do not implement a partial backend fix that silently changes semantics without a clear path to correctness; either fully fix the feature or document the remaining dependency.
- Do not keep the original plan unchanged after discovering a hard blocker; update the issue, plan, and report to reflect the pivot.
- Do not skip a final `t27c suite` run after mass resealing; the second pass is the green gate.

## 2026-07-01 — Wave Loop 376 completion

### What worked
- Reused the generator pattern (`scripts/gen_w376.py`, `scripts/gen_w376_lean.py`) to append W376 blocks and 4 new generic ∀ theorems; `t27c suite` returned **556/556 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyTwoPlusGeneric` pushed the plus-accumulation boundary to **52 variables** without timeout.
- `ternaryMacNovenvigintupleCancellationGeneric` (depth-29) collapsed cleanly to a single residual `mac(x, a, .plus)`, confirming the odd-depth residual pattern.
- `ternaryMacZeroWeightNovemdecupleClosureGeneric` uses 10 zero-weight MACs before and 10 zero-weight MACs after a plus-weight MAC (20 closure size, 21 variables).
- Closed `gen-verilog` Defect 4 by verifying that `as` casts already emit width-safe masks (e.g., `(x & {8{1'b1}})`) and adding scratch spec `specs/scratch/w376_cast_width.t27`.
- Added an in-runner CI smoke gate in `bootstrap/src/suite.rs` that runs `yosys read_verilog -sv` on every `specs/scratch/*.t27` file when `yosys` is on `PATH`; all 10 scratch specs passed, satisfying **L7 UNITY** (no new shell scripts on the critical path).
- Mass seal regeneration after compiler/CI changes: 28 mismatched seals from the suite run were resealed and the second suite pass showed **0 mismatches**.

### What changed behavior
- Generic ∀ count reached **248** (240 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 other generic theorems in the same file).
- The zero-IGLA-failure streak extended to **110 waves** (thirty-sixth consecutive zero-failure wave).
- IGLA totals: **13,028 tests**, **5,714 invariants** across full repo.
- Conformance suite now evaluates **556 specs** (27 IGLA + non-IGLA + scratch regression specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W376.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 4 verified-fixed; Defect 6 remains blocked by tuple-return generation; Defect 5 is the next wave-safe open defect.

### Patterns to reuse
- When a planned backend change turns out to be unnecessary because existing codegen is already correct, formalize a regression spec and a CI gate rather than rewriting code.
- Keep yosys verification inside the Rust suite runner so the conformance gate is self-contained and L7-compliant.
- After adding a compiler-side CI phase, expect a seal mismatch wave; batch reseal and run the suite a second time to confirm zero failures.

### Anti-patterns to avoid
- Do not rewrite working codegen without first proving the generated output is incorrect; a regression spec and smoke gate are often the right fix.
- Do not make the smoke gate mandatory when its external dependency (`yosys`) may not be installed locally; skip gracefully and enforce in CI.
- Do not leave warnings unlogged; the smoke gate prints yosys warnings so they can be triaged without failing the gate.

## 2026-07-03 — Wave Loop 377 completion

### What worked
- Reused the generator pattern (`scripts/gen_w377.py`, `scripts/gen_w377_lean.py`) to append W377 blocks and 4 new generic ∀ theorems; `t27c suite` returned **557/557 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyThreePlusGeneric` pushed the plus-accumulation boundary to **53 variables** without timeout (~6.5 s build).
- `ternaryMacTrigintupleCancellationGeneric` (depth-30) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightVigintupleClosureGeneric` uses 11 zero-weight MACs before and 11 zero-weight MACs after a plus-weight MAC (22 closure size, 23 variables).
- Fixed `gen-verilog` Defect 5 (struct-field register-name mapping) in `bootstrap/src/compiler.rs`. Functions that take a struct parameter now resolve field reads to struct-type registers (`word_data`) rather than parameter-variable registers (`w_data`). Verified with scratch spec `specs/scratch/w377_struct_field_mapping.t27` and `yosys read_verilog -sv` + `synth_xilinx`.
- Expanded the in-runner CI smoke gate in `bootstrap/src/suite.rs` to cover all 25 yosys-clean IGLA specs in addition to the 11 scratch specs; `cordic.t27` and `cordic_top.t27` remain excluded pending Defect 6 (`let` destructuring).
- Mass seal regeneration after compiler/CI changes: 96 mismatched seals from the suite run were resealed and the second suite pass showed **0 mismatches**.

### What changed behavior
- Generic ∀ count reached **252** (244 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 other generic theorems across Trinity modules).
- The zero-IGLA-failure streak extended to **111 waves** (thirty-seventh consecutive zero-failure wave).
- IGLA totals: **13,083 tests**, **5,742 invariants** across full repo.
- Conformance suite now evaluates **557 specs** (27 IGLA + non-IGLA + scratch regression specs).
- Gen-verilog yosys smoke gate now evaluates **36 targets** (11 scratch + 25 clean IGLA).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W377.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 5 fixed; Defect 6 remains blocked by tuple-return generation and is the only remaining open defect.

### Patterns to reuse
- For struct-field lowering, track both parameter types and emitted struct-field register names so field access can resolve to the canonical struct-type register name while preserving fallback behavior for non-struct parameters.
- Maintain an explicit allow-list of yosys-clean IGLA specs in the smoke gate rather than auto-discovering all `specs/igla/*.t27`; this prevents known-broken specs from failing the gate while documenting why they are excluded.
- When mass resealing, capture the list of specs whose seals actually changed (using `t27c suite` mismatch output) and reseal only those; this avoids timestamp-only diffs in hundreds of seal files.

### Anti-patterns to avoid
- Do not reseal every seal file blindly after a compiler change; most seals only need a timestamp update and create noisy diffs.
- Do not expand the smoke gate to all IGLA specs without first testing each one individually; auto-inclusion would fail the gate on specs blocked by known defects.
- Do not assume a codegen fix is correct because the generated Verilog looks right; always run it through `yosys read_verilog -sv` (and ideally `synth_xilinx`) to catch identifier-resolution and syntax issues.

## 2026-07-03 — Wave Loop 378 completion

### What worked
- Reused the generator pattern (`scripts/gen_w378.py`, `scripts/gen_w378_lean.py`) to append W378 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **558/558 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyFourPlusGeneric` pushed the plus-accumulation boundary to **54 variables** without timeout, confirming the `simp+omega` regime still holds at depth 54.
- `ternaryMacUntrigintupleCancellationGeneric` (depth-31) correctly collapsed to residual `mac(x, a, .plus)`, continuing the odd-depth residual pattern.
- `ternaryMacZeroWeightDuovigintupleClosureGeneric` uses 12 zero-weight MACs before and 12 zero-weight MACs after a plus-weight MAC (24 closure size, 25 variables).
- Fixed `gen-verilog` Defect 6 (`let` destructuring) in `bootstrap/src/compiler.rs` at the syntax level. The helper emits a packed-vector temporary for the RHS call result and scalar `reg` slice assignments for each binding in the `let(...)` pattern. This unblocked `specs/igla/race/cordic.t27` and `specs/igla/race/cordic_top.t27` for the yosys smoke gate.
- Expanded the in-runner CI smoke gate in `bootstrap/src/suite.rs` to cover **all 27 IGLA specs** plus all scratch specs (38 yosys targets).
- Captured the exact list of seal-mismatch specs from the first `t27c suite` run and batch-resealed only those 28 specs, avoiding noisy timestamp-only diffs across the full seal set.

### What changed behavior
- Generic ∀ count reached **256** (248 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 other generic theorems across Trinity modules).
- The zero-IGLA-failure streak extended to **112 waves** (thirty-eighth consecutive zero-failure wave).
- IGLA totals: **13,138 tests**, **5,769 invariants** across full repo.
- Conformance suite now evaluates **558 specs** (27 IGLA + non-IGLA + scratch regression specs).
- Gen-verilog yosys smoke gate now evaluates **38 targets** (11 scratch + 27 IGLA).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W378.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 6 is fixed at the syntax level; the remaining tuple-return semantic gap is documented as open work.

### Patterns to reuse
- For syntax-level backend workarounds, keep the change narrow and clearly document the remaining semantic gap so a future wave does not mistake a parse-level fix for full correctness.
- After adding a codegen helper that emits new identifier names (e.g., `_let_tmp_N`), reset any per-function counters at the end of each generated function to avoid collisions across multiple functions in the same module.
- Use the first `t27c suite` mismatch list as a reseal work-list; resealing only the affected specs keeps the diff focused and reviewable.

### Anti-patterns to avoid
- Do not claim a `let` destructuring fix is semantically complete if multi-return function types and tuple literals are still unsupported; document the limitation explicitly.
- Do not auto-discover all IGLA specs for the smoke gate before testing each one individually; the W378 allow-list was built by verifying every spec after the Defect 6 fix.
- Do not let the final documentation and commit steps wait until after a long session; write the report and cooperation variants immediately while the exact metrics are fresh.

## 2026-07-03 — Wave Loop 379 completion

### What worked
- Reused the generator pattern (`scripts/gen_w379.py`, `scripts/gen_w379_lean.py`) to append W379 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **559/559 PASS** and `lake build Trinity.TernaryInference` succeeded.
- `ternaryMacAccumulateFiftyFivePlusGeneric` pushed the plus-accumulation boundary to **55 variables** without timeout, confirming the `simp+omega` regime still holds at depth 55.
- `ternaryMacDuotrigintupleCancellationGeneric` (depth-32) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightTrevigintupleClosureGeneric` uses 13 zero-weight MACs before and 13 zero-weight MACs after a plus-weight MAC (26 closure size, 27 variables).
- Generalized the W378 `gen-verilog` `let` destructuring helper in `bootstrap/src/compiler.rs` so it infers the binding count and per-binding width from the LHS pattern rather than hardcoding 3×32-bit slots. Added `specs/scratch/w379_let_destructuring_generalized.t27` with 2-binding and 4-binding patterns; all pass `yosys read_verilog -sv`.
- Captured the exact list of 29 seal-mismatch specs from the first `t27c suite` run and batch-resealed them, avoiding noisy diffs in unaffected seals.

### What changed behavior
- Generic ∀ count reached **260** (252 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 other generic theorems across Trinity modules).
- The zero-IGLA-failure streak extended to **113 waves** (thirty-ninth consecutive zero-failure wave).
- Full-repo totals: **13,195 tests**, **5,798 invariants**, **1,010 benchmarks** (from `t27c stats`).
- Conformance suite now evaluates **559 specs** (27 IGLA + non-IGLA + scratch regression specs).
- Gen-verilog yosys smoke gate evaluates **38 targets** (11 scratch + 27 IGLA).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W379.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 6 is now a semantically-aware syntax fix; the remaining tuple-return semantic gap is documented as open work.

### Patterns to reuse
- When generalizing a syntax-level backend workaround, infer as much as possible from the AST (binding count, declared types) before falling back to defaults, and add regression specs that exercise the generalized shapes.
- Keep the per-wave theorem budget at 4 generic ∀ theorems; depth-55 plus accumulation is still inside the practical elaboration budget.
- After a compiler change, reseal only the specs whose hashes actually mismatch; the suite output lists them explicitly.

### Anti-patterns to avoid
- Do not assume a hardcoded 3-slot workaround is sufficient for all future specs; generalize the helper as soon as a second shape appears.
- Do not omit a regression spec for the generalized backend path; the original IGLA path (3 slots) may keep passing while a 2-slot or 4-slot path breaks.
- Do not update report metrics from memory when `t27c stats` gives the canonical full-repo totals.

## 2026-07-03 — Wave Loop 380 completion

### What worked
- Reached the W380 target of **264 generic ∀** by appending the original 4 W380 theorems plus 4 extra theorems (`AccumulateFiftySevenPlusGeneric`, `AccumulateFiftySixMinusGeneric`, `SextrigintupleCancellationGeneric`, `ZeroWeightFourteenPairClosureGeneric`). `lake build Trinity.TernaryInference` completed in ~12.5 s.
- Extended the IGLA CODER+RACE zero-failure streak to **114 waves**; `t27c suite --repo-root /Users/playra/t27` returned **560/560 PASS**.
- Began tuple-return generation scaffolding in `bootstrap/src/compiler.rs`: parser support for tuple return types and tuple literals, packed function result registers, and callee-type-aware `let` destructuring widths.
- Added `specs/scratch/w380_tuple_return.t27` with mixed-width tuple returns `(u16, u32, u8)`; generated Verilog passes `yosys read_verilog -sv`.
- Fixed a parser infinite loop on named/namespaced tuple return types (`(gf16::GF16, ...)` and `(added: u32, ...)`) introduced by the new tuple parser.
- Batch-resealed the 41 specs with hash mismatches after the compiler changes, then reran the suite to 0 failures.

### What changed behavior
- Generic ∀ count reached **264** (264 `ternaryMac...Generic` theorems in `TernaryInference.lean`).
- Full-repo totals: **13,251 tests**, **5,826 invariants**, **1,010 benchmarks** (from `t27c stats`).
- Conformance suite evaluates **560 specs**.
- Gen-verilog yosys smoke gate evaluates **41 targets** (14 scratch + 27 IGLA).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W380.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 6 scaffolding is in place; remaining gap is slot-aware nested tuple-return call lowering.

### Patterns to reuse
- When adding parser support for a new type shape, immediately test it against existing specs that already use that shape (e.g., namespaced tuple return types in `adamw.t27`) to catch regressions.
- Use packed concatenation `{c, b, a}` for tuple literals in Verilog so the first element occupies the most significant bits and slice assignments line up with destructuring.
- Batch-reseal after a compiler change: capture the mismatch list from the first suite run, run `t27c seal --save` for each, then rerun the suite.

### Anti-patterns to avoid
- Do not write tuple-return parsing that treats `Ident + Colon` as a named label without checking for the `::` namespace separator; it causes infinite loops on namespaced types.
- Do not add cancellation theorems at odd depths while claiming identity `= x`; odd depths leave a residual `±a`. Use even depths for identity cancellation.
- Do not reuse existing Latin-prefixed theorem names for new closure theorems; name collisions are silent until Lean build fails.

## 2026-07-01 — Wave Loop 381 completion

### What worked
- Reached the W381 target of **268 generic ∀** by appending 4 new theorems (`AccumulateFiftyNinePlusGeneric`, `AccumulateFiftyEightMinusGeneric`, `DuotrigintupleSeptemCancellationGeneric`, `ZeroWeightSixteenPairClosureGeneric`). `lake build Trinity.TernaryInference` completed successfully.
- Extended the IGLA CODER+RACE zero-failure streak to **115 waves**; `t27c suite --repo-root /Users/playra/t27` returned **561/561 PASS**.
- Completed slot-aware nested tuple-return call lowering in `bootstrap/src/compiler.rs`: function-call expressions that return tuples now emit a packed temporary sized to the callee's tuple width, and consuming tuple literals slice the temporary by slot.
- Added `specs/scratch/w381_tuple_call_chain.t27` exercising a two-level tuple-return chain; generated Verilog passes `yosys read_verilog -sv`.
- Batch-resealed the 28 specs with hash mismatches after appending W381 IGLA blocks and the new scratch spec, then reran the suite to 0 failures.

### What changed behavior
- Generic ∀ count reached **268** in `TernaryInference.lean`.
- Full-repo totals: **13,306 tests**, **5,854 invariants**, **1,010 benchmarks** (from `t27c stats`).
- Conformance suite evaluates **561 specs**.
- Gen-verilog yosys smoke gate evaluates **42 targets** (15 scratch + 27 IGLA).
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: Defect 6 / tuple-return lowering is now closed.
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W381.md`.

### Patterns to reuse
- When a batch generator has already run once, update its idempotency guard to the newest theorem name so it can append additional blocks without duplicating earlier ones.
- For tuple-return call lowering, reuse the existing `fn_return_types` registry and `tuple_element_widths` helper rather than hardcoding slot widths.
- After fixing duplicate theorems in a Lean file, verify the exact generic ∀ milestone with `grep -oE "[0-9]+ generic ∀ milestone"` rather than relying on a hand count.

### Anti-patterns to avoid
- Do not run a generator that appends multiple blocks twice without checking whether intermediate blocks are already present; it silently duplicates theorems and breaks the Lean build.
- Do not change a milestone comment in a generated block without also updating the generator script; the next run will re-emit the stale comment.
- Do not assume a theorem name is unique just because it uses a Latin prefix; cross-check against the previous 2–3 waves before appending.

## 2026-07-01 — Wave Loop 382 completion

### What worked
- Reached the W382 target of **272 generic ∀** by appending 4 new theorems (`AccumulateSixtyPlusGeneric`, `AccumulateFiftyNineMinusGeneric`, `QuadragintupleCancellationGeneric`, `ZeroWeightSeventeenPairClosureGeneric`). `lake build Trinity.TernaryInference` completed successfully.
- Extended the IGLA CODER+RACE zero-failure streak to **116 waves**; `t27c suite --repo-root /Users/playra/t27` returned **562/562 PASS**.
- Landed the first incremental array/RAM lowering in `bootstrap/src/compiler.rs`: module-level `var mem : [N]T` now emits a true Verilog memory `reg [W-1:0] mem [0:N-1];`, so `mem[i]` reads and `mem[i] = x` writes resolve to memory accesses.
- Added `specs/scratch/w382_ram_lowering.t27` exercising a 4-entry `u16` memory with write/read; generated Verilog passes `yosys read_verilog -sv`.
- Batch-resealed the 27 IGLA specs plus the new scratch spec after appending W382 blocks and the compiler change, then reran the suite to 0 failures.

### What changed behavior
- Generic ∀ count reached **272** in `TernaryInference.lean`.
- Full-repo totals: **13,362 tests**, **5,881 invariants**, **1,010 benchmarks** (from `t27c stats`).
- Conformance suite evaluates **562 specs**.
- Gen-verilog yosys smoke gate evaluates **43 targets** (16 scratch + 27 IGLA).
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: module-level array/RAM lowering added; remaining sub-gaps documented.
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W382.md`.

### Patterns to reuse
- For array type parsing, extract the size and element type from the type annotation string (e.g. `[4]u16`) rather than relying on the legacy `extra_size` field, which is only populated by array-literal syntax.
- When changing module-level variable emission, expect seal mismatches in any spec that declares a module-level var (not only the IGLA specs); capture and reseal the mismatch list from the first suite run.
- Cancellation theorem depths must be even to collapse to identity `= x`; odd depths leave a residual `±a` and break the Lean build.

### Anti-patterns to avoid
- Do not plan cancellation theorems at odd depths while claiming identity collapse; always use even depths or match the statement to the residual weight.
- Do not rebuild the workspace root crate and assume `target/release/t27c` is fresh; if the binary timestamp is stale, rebuild the `bootstrap` crate explicitly.
- Do not emit individual `reg name_0, name_1, ...` for array vars when a true Verilog memory `reg [W-1:0] name [0:N-1];` is what downstream indexing expects.
