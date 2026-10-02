# Seal triage, 2026-10-02

The seal-coverage gate (`tools/check_seal_coverage.py`, restored in #5454) reported, on
master `a26af0ad5`, **647 seals that newly do not hold**: 542
`stale` and 105 `gen-drift`. (#5453 counted 649 = 540 + 109;
master moved since.) This report puts each one in a class, with the evidence.

Tracking: #5453. Class (b): #5576. Class (c): #5577.

## Counts

| class | seals | specs | resealed here |
|---|---:|---:|---:|
| (a) STALE-ONLY | 18 | 9 | 2 |
| (b) GEN-DRIFT | 599 | 341 | 0 |
| (c) REAL SPEC DEFECT | 30 | 15 | 0 |
| **total** | **647** | | **2** |

(b) by sub-class: `edited-gen-changed` 480, `edited-gen-unchanged` 7, `gen-drift` 105, `hidden-gen-drift` 7.

Test status per class (`t27c test-report`, one run per spec): (a) BLOCKED 16, PASS 2; (b) BLOCKED 483, PASS 114, TIMEOUT 2; (c) FAIL 30.

## Rules

Every entry the gate reported is in exactly one class. Rules are applied in this order.

- **(c) REAL SPEC DEFECT** -- `t27c test-report <spec>` reports at least one `FAIL`.
  The spec's own test blocks do not hold against the code generated from it.
- **(a) STALE-ONLY** -- gate status `stale`, and all of:
  1. the spec text changed since sealing, but **every changed line was read by hand** and is
     whitespace, a plain comment, a quoted test name, or redundant parentheses around an `if`
     condition;
  2. `t27c seal <spec>` (t27c built from master `a26af0ad5`) gives the **same four gen hashes**
     as the seal on record -- the outputs are byte-identical;
  3. the parsed AST (`t27c parse --json`, `line` keys stripped) of the sealed blob and of the
     current spec are equal.
- **(b) GEN-DRIFT** -- everything else, in four sub-classes:
  - `gen-drift` -- gate status `gen-drift`: the spec is byte-identical to what was sealed, but
    the compiler now emits different output.
  - `hidden-gen-drift` -- spec text changed, the AST did not, and the output did anyway. The
    text change is not the cause; the compiler moved.
  - `edited-gen-changed` -- the spec's meaning was edited (bodies, tests, types) and the output
    changed. Resealing these is a decision to accept the new output, not a refresh.
  - `edited-gen-unchanged` -- the spec was edited and the output did not move, but the edit is
    not trivial (or not reviewed). Two were reviewed and **excluded on purpose**, see below.

`BLOCKED` (the generated Zig does not compile, so tests cannot run) is not a failure -- the
repo's rule is that a blocked spec is not a failing one. It is reported as a column, not a class.

## How it was measured

1. `t27c` built from master `a26af0ad5` (`cargo build --release -p t27c`); the gate run with it.
2. For each `stale` seal, the git history of the spec was walked until a blob whose sha256
   equals the seal's `spec_hash` -- found for all 542.
3. That sealed blob and the current spec were each parsed (`t27c parse --json`) and the
   current spec sealed without saving (`t27c seal <spec>`), so every entry has: text diff,
   AST equal or not, which of the four gen hashes moved.
4. `t27c test-report` on each of the 365 distinct specs, each with its own `TMPDIR` (see
   "Surprises").
5. Every AST-equal spec (14) was read by hand line by line; that is the spot check behind (a).

## (a) STALE-ONLY -- 18 seals, 9 specs

Resealed in this PR only where the spec's tests were seen to **pass**. The other 16
seals meet every other rule, but `test-report` is `BLOCKED` for them, so "its tests still pass"
cannot be shown -- they are held back and listed here; their outputs are byte-identical to the
sealed ones, so nothing a test could see has changed. Reseal them with
`t27c seal <spec> --save` once the block is fixed or if a reviewer accepts the argument.

| spec | seals | change since seal | tests | resealed |
|---|---|---|---|---|
| `specs/api/c_api_contract.t27` | `api_c_api_contract`, `c_api_contract` | 13 Markdown `#` heading comments rewritten as `//` comments (#4888) | BLOCKED: does not compile: spec.zig:19:17: error: use of undeclared identifier 'trinity_vsa_version | held |
| `specs/depin/prove.t27` | `depin_depin.prove`, `prove` | 3-line 'superseded, do not deploy' comment added (#5415) | BLOCKED: does not compile: TriSha256.zig:1:1: error: unable to load 'TriSha256.zig': FileNotFound | held |
| `specs/fpga/boards/arty_a7_integration.t27` | `ArtyA7_Integration`, `boards_ArtyA7_Integration` | redundant parens added around 4 `if` conditions (#4656) | PASS 6 | yes |
| `specs/fpga/testbench/spi_tb.t27` | `SPI_Testbench`, `testbench_SPI_Testbench` | two blank lines added (#4714) | BLOCKED: does not compile: spec.zig:80:9: error: local variable shadows declaration of 'spi_start' | held |
| `specs/isa/ternary_memory.t27` | `ISAMemoryOps`, `isa_ISAMemoryOps` | one comment line added (#4835) | BLOCKED: does not compile: spec.zig:44:55: error: expected type expression, found 'align' | held |
| `specs/ml/activation/relu_activation.t27` | `Relu`, `activation_Relu` | 6 test names quoted | BLOCKED: does not compile: spec.zig:51:20: error: expected 3 argument(s), found 1 | held |
| `specs/tri/collections/array.t27` | `TriArray`, `collections_TriArray` | 9 test names quoted | BLOCKED: does not compile: spec.zig:21:13: error: use of undeclared identifier 'T' | held |
| `specs/tri/math/constants.t27` | `TriConstants`, `math_TriConstants` | 29 test names quoted | BLOCKED: does not compile: spec.zig:71:19: error: use of undeclared identifier 'default_input' | held |
| `specs/tri/utils/random.t27` | `TriRandom`, `utils_TriRandom` | 10 test names quoted | BLOCKED: does not compile: spec.zig:61:18: error: operator != not allowed for type 'spec.Rng' | held |

Reviewed by hand and **excluded** from (a) although the AST and all four outputs are unchanged:

- `specs/base/ternary_add.t27` -- `for all i8 s` was appended to three asserts (#4837). The
  parser drops the quantifier, so nothing downstream moves, but the spec now claims more than
  the sealed one. Resealing would certify a claim the compiler never saw.
- `specs/numeric/formats_catalog.t27` -- tnf16 `phi_distance` changed 0.086 -> 0.0417 inside a
  `// CATALOG:` line, which is read by tooling. It is data, not a comment.

## Surprises

- **Only 13 of 542 `stale` seals are text-trivial** (2 whitespace-only, 11 comment-only by
  text diff). The rest come from real edits -- bodies filled in, tests added, parse fixes.
  "Stale" mostly does not mean "hash out of date"; it means "nobody re-certified the edit".
- **The parser silently drops meaning.** `for all i8 s` on asserts (`base/ternary_add`),
  `forall a : i32` -> `string` (`igla/race/backend`) and invariant bodies (`opcodes`) leave the
  AST unchanged. An AST-equal check alone would have resealed real spec changes; (a) needs a
  human read.
- **`t27c test-report` work-dir collision.** The temp dir is `t27c-test-report-<spec stem>`,
  so two specs with the same stem run in parallel overwrite each other's `spec.zig`. Every run
  here used its own `TMPDIR`.
- **Most specs cannot run their tests at all**: 499 of 647 seals are on specs whose generated
  Zig does not compile (`BLOCKED`), including 16 of the 18 (a) seals.
- The gate count moved from 649 (540 + 109) to 647 (542 + 105) between #5454 and this run.

## (c) REAL SPEC DEFECT -- 30 seals, 15 specs

| spec | seals | gate | failing tests |
|---|---|---|---|
| `specs/fpga/testbench/timing_tb.t27` | `Timing_Testbench`, `testbench_Timing_Testbench` | stale | 1 of 10 tests fail: test_fmax_computation |
| `specs/fpga/verification/build_verify.t27` | `BuildVerify`, `verification_BuildVerify` | stale | 1 of 11 tests fail: test_module_count |
| `specs/ml/layers/residual_connection.t27` | `Residual`, `layers_Residual` | stale | 2 of 2 tests fail: forward_basic_case; forward_residual_connection |
| `specs/ml/loss/huber_loss.t27` | `HuberLoss`, `loss_HuberLoss` | stale | 1 of 1 tests fail: forward_basic_case |
| `specs/ml/loss/kl_divergence.t27` | `KlDivergence`, `loss_KlDivergence` | stale | 1 of 1 tests fail: forward_basic_case |
| `specs/ml/loss/mse_loss.t27` | `MseLoss`, `loss_MseLoss` | stale | 3 of 3 tests fail: forward_basic_case; forward_empty_input; forward_negative_values |
| `specs/ml/optimizer/rmsprop.t27` | `Rmsprop`, `optimizer_Rmsprop` | stale | 1 of 1 tests fail: step_basic_case |
| `specs/ml/transformer/multi_head_attention.t27` | `MultiHeadAttn`, `transformer_MultiHeadAttn` | stale | 1 of 2 tests fail: forward_basic_case |
| `specs/tri/encoding/html.t27` | `TriHtml`, `encoding_TriHtml` | stale | 2 of 4 tests fail: parse_basic_case; query_selector_basic_case |
| `specs/tri/encoding/xml.t27` | `TriXml`, `encoding_TriXml` | stale | 1 of 2 tests fail: parse_basic_case |
| `specs/tri/graph/bellman_ford.t27` | `TriBellmanFord`, `graph_TriBellmanFord` | stale | 1 of 1 tests fail: shortest_path_basic_case |
| `specs/tri/search/pattern.t27` | `TriPattern`, `search_TriPattern` | stale | 1 of 11 tests fail: wildcard_match_no_match |
| `specs/tri/sort/merge_sort.t27` | `TriMergeSort`, `sort_TriMergeSort` | stale | 2 of 2 tests fail: sort_basic_case; sort_in_place_basic_case |
| `specs/tri/utils/template.t27` | `TriTemplate`, `utils_TriTemplate` | stale | 2 of 2 tests fail: compile_basic_case; render_basic_case |
| `specs/tri/utils/terminal.t27` | `TriTerminal`, `utils_TriTerminal` | stale | 1 of 3 tests fail: reset_returns_ansi_sequence |

## (b) GEN-DRIFT -- 599 seals, 341 specs

### `gen-drift` -- 105 seals

| spec | seals | reason | tests |
|---|---|---|---|
| `specs/api/c_abi.t27` | `api_TrinityCAbi` | spec unchanged; zig differ | BLOCKED |
| `specs/api/tri_api_context.t27` | `api_TriApiContext` | spec unchanged; zig differ | BLOCKED |
| `specs/api/tri_api_loop.t27` | `api_TriApiLoop` | spec unchanged; zig differ | BLOCKED |
| `specs/api/tri_api_permissions.t27` | `api_TriApiPermissions` | spec unchanged; zig differ | BLOCKED |
| `specs/api/tri_api_session.t27` | `api_TriApiSession` | spec unchanged; zig differ | PASS 9 |
| `specs/ar/coa_planning.t27` | `ar_coa_planning`, `coa_planning` | spec unchanged; zig differ | BLOCKED |
| `specs/ar/explainability.t27` | `Explainability`, `ar_Explainability` | spec unchanged; zig differ | BLOCKED |
| `specs/ar/proof_trace.t27` | `ProofTrace`, `ar_ProofTrace`, `ar_proof_trace`, `proof_trace` | spec unchanged; zig differ | BLOCKED |
| `specs/ar/restraint.t27` | `Restraint`, `ar_Restraint` | spec unchanged; zig differ | BLOCKED |
| `specs/automation/wrapup-auto.t27` | `automation::wrapup`, `automation_automation::wrapup` | spec unchanged; zig, c differ | PASS 1 |
| `specs/base/ring_32.t27` | `base-ring-32`, `base_base-ring-32` | spec unchanged; zig differ | PASS 1 |
| `specs/catalog/discovery.t27` | `catalog_catalog_discovery` | spec unchanged; zig, verilog, c, rust differ | PASS 5 |
| `specs/compiler/stdlib.t27` | `compiler_Stdlib` | spec unchanged; zig differ | BLOCKED |
| `specs/demos/jones_topology_decision_gate.t27` | `JonesTopologyDecisionGate`, `demos_JonesTopologyDecisionGate` | spec unchanged; zig differ | BLOCKED |
| `specs/demos/jones_topology_filter.t27` | `JonesTopologyFilter`, `demos_JonesTopologyFilter` | spec unchanged; zig differ | BLOCKED |
| `specs/fpga/adapter.t27` | `fpga_adapter` | spec unchanged; zig differ | PASS 3 |
| `specs/igla/coder/arch.t27` | `coder_igla-coder-arch` | spec unchanged; zig differ | BLOCKED |
| `specs/igla/coder/dataset.t27` | `coder_igla-coder-dataset` | spec unchanged; zig, c differ | BLOCKED |
| `specs/igla/coder/eval.t27` | `coder_igla-coder-eval` | spec unchanged; zig, c, rust differ | BLOCKED |
| `specs/igla/coder/prm.t27` | `coder_igla-coder-prm` | spec unchanged; zig differ | BLOCKED |
| `specs/igla/coder/weights.t27` | `coder_igla-coder-weights` | spec unchanged; zig differ | BLOCKED |
| `specs/igla/race/bram_weights.t27` | `race_igla-race-bram-weights` | spec unchanged; zig differ | BLOCKED |
| `specs/igla/race/eda.t27` | `race_igla-race-eda` | spec unchanged; zig differ | BLOCKED |
| `specs/igla/race/formal.t27` | `race_igla-race-formal` | spec unchanged; zig, c differ | BLOCKED |
| `specs/igla/race/rtl.t27` | `RTL`, `race_igla-race-rtl` | spec unchanged; zig, c differ | BLOCKED |
| `specs/igla/race/systolic_ternary.t27` | `race_igla-race-systolic-ternary` | spec unchanged; zig differ | BLOCKED |
| `specs/isa/ternary_encoding.t27` | `isa_Tri27Encoding` | spec unchanged; zig differ | BLOCKED |
| `specs/isa/tri27_bytecode.t27` | `isa_Tri27Bytecode` | spec unchanged; zig differ | BLOCKED |
| `specs/isa/tri27_machine.t27` | `isa_Tri27Machine` | spec unchanged; zig differ | BLOCKED |
| `specs/memory/notebooklm.t27` | `NotebookLM`, `memory_NotebookLM` | spec unchanged; zig differ | BLOCKED |
| `specs/queen/dispatch.t27` | `queen_QueenDispatch` | spec unchanged; zig differ | PASS 6 |
| `specs/queen/views.t27` | `queen_QueenViews` | spec unchanged; zig differ | PASS 7 |
| `specs/server/project.t27` | `Project`, `server_Project` | spec unchanged; zig differ | BLOCKED |
| `specs/tools/catalog.t27` | `tools_ToolsCatalog` | spec unchanged; zig differ | BLOCKED |
| `specs/tools/mcp_protocol.t27` | `tools_McpProtocol` | spec unchanged; zig differ | BLOCKED |
| `specs/tri/math/bezier.t27` | `TriBezier`, `math_TriBezier` | spec unchanged; zig differ | BLOCKED |
| `specs/trinity/build_graph.t27` | `trinity_trinity_build_graph` | spec unchanged; zig, verilog, c, rust differ | PASS 4 |
| `specs/trinity/capabilities/abi.c-api.t27` | `capabilities_trinity_capability_abi_c_api` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/agent.daemons.t27` | `capabilities_trinity_capability_agent_daemons` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/agent.phi-loop.t27` | `capabilities_trinity_capability_agent_phi_loop` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/agent.tri-api.t27` | `capabilities_trinity_capability_agent_tri_api` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/api.http-server.t27` | `capabilities_trinity_capability_api_http_server` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/archive.legacy.t27` | `capabilities_trinity_capability_archive_legacy` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/bench.suite.t27` | `capabilities_trinity_capability_bench_suite` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/bot.tri-bot.t27` | `capabilities_trinity_capability_bot_tri_bot` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/catalog.spec-mirror.t27` | `capabilities_trinity_capability_catalog_spec_mirror` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/catalog.world-scan.t27` | `capabilities_trinity_capability_catalog_world_scan` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/cli.tri.t27` | `capabilities_trinity_capability_cli_tri` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/convert.b2t.t27` | `capabilities_trinity_capability_convert_b2t` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/examples.zig.t27` | `capabilities_trinity_capability_examples_zig` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/ext.vscode-swe.t27` | `capabilities_trinity_capability_ext_vscode_swe` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/fpga.adapter.t27` | `capabilities_trinity_capability_fpga_adapter` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/fpga.in-repo.t27` | `capabilities_trinity_capability_fpga_in_repo` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/lib.trinity.t27` | `capabilities_trinity_capability_lib_trinity` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/llm.firebird.t27` | `capabilities_trinity_capability_llm_firebird` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/llm.igla-chat.t27` | `capabilities_trinity_capability_llm_igla_chat` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/native.canvas.t27` | `capabilities_trinity_capability_native_canvas` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/native.node-gui.t27` | `capabilities_trinity_capability_native_node_gui` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/native.queen-app.t27` | `capabilities_trinity_capability_native_queen_app` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/node.trinity-node.t27` | `capabilities_trinity_capability_node_trinity_node` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/numeric.golden-float.t27` | `capabilities_trinity_capability_numeric_golden_float` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/ops.deploy-infra.t27` | `capabilities_trinity_capability_ops_deploy_infra` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/ops.railway-cli.t27` | `capabilities_trinity_capability_ops_railway_cli` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/pkg.vendored-cache.t27` | `capabilities_trinity_capability_pkg_vendored_cache` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/queen.lib.t27` | `capabilities_trinity_capability_queen_lib` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/registry.commands.t27` | `capabilities_trinity_capability_registry_commands` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/release.packaging.t27` | `capabilities_trinity_capability_release_packaging` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/research.nexus-docs.t27` | `capabilities_trinity_capability_research_nexus_docs` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/research.programs.t27` | `capabilities_trinity_capability_research_programs` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/search.vsa-index.t27` | `capabilities_trinity_capability_search_vsa_index` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/specs.t27-vendored-compiler.t27` | `capabilities_trinity_capability_specs_t27_vendored_compiler` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/specs.tri-corpus.t27` | `capabilities_trinity_capability_specs_tri_corpus` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/specs.vibee-corpus.t27` | `capabilities_trinity_capability_specs_vibee_corpus` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/state.trinity-dir.t27` | `capabilities_trinity_capability_state_trinity_dir` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/test.graph.t27` | `capabilities_trinity_capability_test_graph` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/tools.scripts.t27` | `capabilities_trinity_capability_tools_scripts` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/train.hslm.t27` | `capabilities_trinity_capability_train_hslm` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/training.targets.t27` | `capabilities_trinity_capability_training_targets` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/tri27.programs.t27` | `capabilities_trinity_capability_tri27_programs` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/tri27.toolchain.t27` | `capabilities_trinity_capability_tri27_toolchain` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/vibee.compiler.t27` | `capabilities_trinity_capability_vibee_compiler` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/vsa.zig-hdc.t27` | `capabilities_trinity_capability_vsa_zig_hdc` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/capabilities/web.docsite.t27` | `capabilities_trinity_capability_web_docsite` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/web.queen-web.t27` | `capabilities_trinity_capability_web_queen_web` | spec unchanged; zig, verilog, c differ | PASS 1 |
| `specs/trinity/capabilities/web.site.t27` | `capabilities_trinity_capability_web_site` | spec unchanged; zig, verilog, c, rust differ | PASS 1 |
| `specs/trinity/compiler_matrix.t27` | `trinity_trinity_compiler_matrix` | spec unchanged; zig differ | PASS 6 |
| `specs/ui/queen_evidence.t27` | `ui_ui_queen_evidence` | spec unchanged; zig differ | PASS 2 |
| `specs/vm/trinity_vm.t27` | `vm_TrinityVsaVm` | spec unchanged; zig differ | BLOCKED |
| `specs/vsa/ops.t27` | `VSAOps`, `vsa_VSAOps` | spec unchanged; zig differ | BLOCKED |
| `specs/vsa/trinity_compat.t27` | `vsa_vsa_trinity_compat` | spec unchanged; zig differ | BLOCKED |

### `hidden-gen-drift` -- 7 seals

| spec | seals | reason | tests |
|---|---|---|---|
| `specs/ar/asp_solver.t27` | `AspSolver`, `ar_AspSolver`, `ar_asp_solver`, `asp_solver` | AST unchanged since sealing, but zig, c differ | BLOCKED |
| `specs/igla/race/backend.t27` | `Backend`, `race_igla-race-backend` | AST unchanged since sealing, but zig differ | BLOCKED |
| `specs/igla/race/opcodes.t27` | `race_igla-race-opcodes` | AST unchanged since sealing, but zig differ | BLOCKED |

### `edited-gen-unchanged` -- 7 seals

| spec | seals | reason | tests |
|---|---|---|---|
| `specs/ar/ternary_logic.t27` | `TernaryLogic`, `ar_TernaryLogic`, `ar_ternary_logic`, `ternary_logic` | spec edited since sealing, all four outputs unchanged | BLOCKED |
| `specs/base/ternary_add.t27` | `base_ternary_add`, `ternary_add` | `for all i8 s` appended to 3 asserts (#4837); the parser drops it, so the AST and all four outputs are unchanged while the spec text says more | BLOCKED |
| `specs/numeric/formats_catalog.t27` | `numeric_FormatsCatalog` | tnf16 phi_distance 0.086 -> 0.0417 in a machine-read `// CATALOG:` line (#4795); data, not a comment | PASS 0 |

### `edited-gen-changed` -- 480 seals

| spec | seals | reason | tests |
|---|---|---|---|
| `specs/account/repo.t27` | `AccountRepo`, `account_AccountRepo` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ar/composition.t27` | `Composition`, `ar_Composition` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/auth/config.t27` | `AuthConfig`, `auth_AuthConfig` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/base/ternary_encoding.t27` | `TernaryEncoding`, `base_TernaryEncoding` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/base/types.t27` | `BaseTypes`, `base_tritype-base`, `tritype-base` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/boards/xc7a100t_full.t27` | `BoardFullXC7A100T`, `boards_BoardFullXC7A100T` | spec edited since sealing; zig, verilog, c, rust differ | PASS 15 |
| `specs/brain/phi_timing.t27` | `brain-phi-timing`, `brain_brain-phi-timing` | spec edited since sealing; zig, verilog, c, rust differ | PASS 5 |
| `specs/brain/unified_state.t27` | `brain-unified-state`, `brain_brain-unified-state` | spec edited since sealing; zig, verilog, c, rust differ | PASS 3 |
| `specs/bus/schema.t27` | `bus-schema`, `bus_bus-schema` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/cloud/railway_deploy.t27` | `cloud-railway-deploy`, `cloud_cloud-railway-deploy` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/compiler/diagnostics.t27` | `Diagnostics`, `compiler_Diagnostics` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/compiler/linker.t27` | `Linking`, `compiler_Linking` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/compiler/mod_structure.t27` | `compiler-mod-structure`, `compiler_compiler-mod-structure` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/compiler/typechecker.t27` | `TypeChecking`, `compiler_TypeChecking` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/config/schema.t27` | `config-schema`, `config_config-schema` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/file/watcher.t27` | `FileWatcher`, `file_FileWatcher` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/assembler.t27` | `Assembler`, `fpga_Assembler` | spec edited since sealing; zig, rust differ | PASS 19 |
| `specs/fpga/boards/qmtech_a100t_integration.t27` | `QMTech_A100T_Integration`, `boards_QMTech_A100T_Integration` | spec edited since sealing; zig, verilog, c, rust differ | PASS 8 |
| `specs/fpga/formal.t27` | `Formal`, `fpga_Formal` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/hir.t27` | `Hir`, `fpga_Hir` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/hw_types.t27` | `HwTypes`, `fpga_HwTypes` | spec edited since sealing; zig, verilog, c, rust differ | PASS 24 |
| `specs/fpga/linker.t27` | `Linker`, `fpga_Linker` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/partition.t27` | `Partition`, `fpga_Partition` | spec edited since sealing; zig, verilog, c, rust differ | PASS 13 |
| `specs/fpga/placement.t27` | `Placement`, `fpga_Placement` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/power.t27` | `Power`, `fpga_Power` | spec edited since sealing; zig, rust differ | PASS 15 |
| `specs/fpga/power_analysis.t27` | `PowerAnalysis`, `fpga_PowerAnalysis` | spec edited since sealing; zig differ | BLOCKED |
| `specs/fpga/router.t27` | `Router`, `fpga_Router` | spec edited since sealing; zig, verilog, c, rust differ | PASS 22 |
| `specs/fpga/simulator.t27` | `Simulator`, `fpga_Simulator` | spec edited since sealing; zig, verilog, c, rust differ | PASS 18 |
| `specs/fpga/ternary_isa.t27` | `TernaryIsa`, `fpga_TernaryIsa` | spec edited since sealing; zig, verilog, c, rust differ | PASS 30 |
| `specs/fpga/testbench.t27` | `Testbench`, `fpga_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/apb_bridge_tb.t27` | `APB_Bridge_Testbench`, `testbench_APB_Bridge_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/assembler_tb.t27` | `Assembler_Testbench`, `testbench_Assembler_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 7 |
| `specs/fpga/testbench/axi4_tb.t27` | `AXI4_Testbench`, `testbench_AXI4_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/bridge_tb.t27` | `Bridge_Testbench`, `testbench_Bridge_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/cts_tb.t27` | `CTS_Testbench`, `testbench_CTS_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 10 |
| `specs/fpga/testbench/dft_tb.t27` | `DFT_Testbench`, `testbench_DFT_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/fifo_tb.t27` | `FIFO_Testbench`, `testbench_FIFO_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/formal_tb.t27` | `Formal_Testbench`, `testbench_Formal_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/gf16_accel_tb.t27` | `GF16_Accel_Testbench`, `testbench_GF16_Accel_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | TIMEOUT |
| `specs/fpga/testbench/hir_tb.t27` | `HIR_Testbench`, `testbench_HIR_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/integration_tb.t27` | `Integration_Testbench`, `testbench_Integration_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/linker_tb.t27` | `Linker_Testbench`, `testbench_Linker_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/memory_tb.t27` | `Memory_Testbench`, `testbench_Memory_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/partition_tb.t27` | `Partition_Testbench`, `testbench_Partition_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 8 |
| `specs/fpga/testbench/placement_tb.t27` | `Placement_Testbench`, `testbench_Placement_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 9 |
| `specs/fpga/testbench/power_analysis_tb.t27` | `PowerAnalysis_Testbench`, `testbench_PowerAnalysis_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/power_tb.t27` | `Power_Testbench`, `testbench_Power_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 8 |
| `specs/fpga/testbench/router_tb.t27` | `Router_Testbench`, `testbench_Router_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | PASS 9 |
| `specs/fpga/testbench/stdlib_tb.t27` | `Stdlib_Testbench`, `testbench_Stdlib_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/ternary_isa_tb.t27` | `Ternary_ISA_Testbench`, `testbench_Ternary_ISA_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/testbench/vcd_conformance_compare_tb.t27` | `VcdConformanceCompare_Testbench`, `testbench_VcdConformanceCompare_Testbench` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/fpga/testbench/vcd_trace_tb.t27` | `VCD_Trace_Testbench`, `testbench_VCD_Trace_Testbench` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/timing.t27` | `Timing`, `fpga_Timing` | spec edited since sealing; zig, verilog, c, rust differ | PASS 25 |
| `specs/fpga/top_level.t27` | `Trinity_FPGA_Top`, `ZeroDSP_TopLevel`, `fpga_ZeroDSP_TopLevel` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/uart.t27` | `UART_Bridge`, `ZeroDSP_UART`, `fpga_ZeroDSP_UART` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/fpga/vcd_trace.t27` | `VcdTrace`, `fpga_VcdTrace` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/git/diff.t27` | `GitDiff`, `git_GitDiff` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/git/operations.t27` | `GitOperations`, `git_GitOperations` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/git/status.t27` | `GitStatus`, `git_GitStatus` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/graph/knowledge_graph.t27` | `KnowledgeGraph`, `graph_KnowledgeGraph` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/coder/bench_proxy.t27` | `coder_igla-coder-bench-proxy` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/coder/tokenizer.t27` | `coder_igla-coder-tokenizer` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/coder/training.t27` | `coder_igla-coder-training` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/evaluation/multi_lang_harness.t27` | `evaluation_IGLAMultiLangHarness` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/integration/publication.t27` | `integration_IGLAPublication` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/race/cordic.t27` | `race_igla-race-cordic` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/race/gemm.t27` | `race_igla-race-gemm` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/race/ternary_inference.t27` | `race_igla-race-ternary-inference` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/igla/training/low_bit_ternary.t27` | `training_IGLALowBitTernary` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/training/pilot_pretraining.t27` | `training_IGLAPilotPretraining` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/training/roadmap.t27` | `training_IGLARoadmap` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/igla/training/scale_up.t27` | `training_IGLAScaleUp` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/isa/ternary_control_flow.t27` | `TernaryControlFlow`, `isa_TernaryControlFlow` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/jit/jit.t27` | `jit`, `jit_jit` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/lsp/client.t27` | `lsp-client`, `lsp_lsp-client` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/lsp/language.t27` | `lsp-language`, `lsp_lsp-language` | spec edited since sealing; zig, verilog, c, rust differ | PASS 0 |
| `specs/lsp/protocol.t27` | `lsp-protocol`, `lsp_lsp-protocol` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/lsp/server.t27` | `lsp-server`, `lsp_lsp-server` | spec edited since sealing; zig differ | BLOCKED |
| `specs/math/constants.t27` | `Constants`, `math_Constants` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/math/phi_split_optimality.t27` | `PhiSplitOptimality`, `math_PhiSplitOptimality` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/math/phi_universal_attractor.t27` | `PhiUniversalAttractor`, `math_PhiUniversalAttractor` | spec edited since sealing; zig, c, rust differ | BLOCKED |
| `specs/ml/activation/elu_activation.t27` | `Elu`, `activation_Elu` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/activation/gelu_approx_activation.t27` | `GeluApprox`, `activation_GeluApprox` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/activation/leaky_relu_activation.t27` | `LeakyRelu`, `activation_LeakyRelu` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/activation/sigmoid_activation.t27` | `Sigmoid`, `activation_Sigmoid` | spec edited since sealing; zig, verilog, c, rust differ | PASS 6 |
| `specs/ml/activation/silu_swish_activation.t27` | `SiluSwish`, `activation_SiluSwish` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/activation/softmax.t27` | `Softmax`, `activation_Softmax` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/activation/tanh_activation.t27` | `Tanh`, `activation_Tanh` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/avgpool2d_layer.t27` | `Avgpool2d`, `layers_Avgpool2d` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/batchnorm_layer.t27` | `Batchnorm`, `layers_Batchnorm` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/conv2d_layer.t27` | `Conv2d`, `layers_Conv2d` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/dense_layer.t27` | `Dense`, `layers_Dense` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/dropout_layer.t27` | `Dropout`, `layers_Dropout` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/embedding_layer.t27` | `Embedding`, `layers_Embedding` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/flatten_layer.t27` | `Flatten`, `layers_Flatten` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/layers/layernorm_layer.t27` | `Layernorm`, `layers_Layernorm` | spec edited since sealing; zig, verilog, c, rust differ | PASS 3 |
| `specs/ml/layers/maxpool2d_layer.t27` | `Maxpool2d`, `layers_Maxpool2d` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/loss/binary_crossentropy_loss.t27` | `BinaryCe`, `loss_BinaryCe` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/loss/contrastive_loss.t27` | `ContrastiveLoss`, `loss_ContrastiveLoss` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/loss/cross_entropy_loss.t27` | `CrossEntropy`, `loss_CrossEntropy` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/adagrad.t27` | `Adagrad`, `optimizer_Adagrad` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/adam.t27` | `Adam`, `optimizer_Adam` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/lamb.t27` | `Lamb`, `optimizer_Lamb` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/lr_scheduler.t27` | `LrScheduler`, `optimizer_LrScheduler` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/sgd.t27` | `Sgd`, `optimizer_Sgd` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/optimizer/sgd_momentum.t27` | `SgdMomentum`, `optimizer_SgdMomentum` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/pathway/mlp.t27` | `Mlp`, `pathway_Mlp` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/attention_mechanism.t27` | `Attention`, `recurrent_Attention` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/bilstm.t27` | `Bilstm`, `recurrent_Bilstm` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/gru_cell.t27` | `Gru`, `recurrent_Gru` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/lstm_cell.t27` | `Lstm`, `recurrent_Lstm` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/lstm_single.t27` | `LstmCell`, `recurrent_LstmCell` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/rnn_cell.t27` | `RnnCell`, `recurrent_RnnCell` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/recurrent/self_attention.t27` | `SelfAttention`, `recurrent_SelfAttention` | spec edited since sealing; zig, verilog, c, rust differ | PASS 1 |
| `specs/ml/recurrent/seq2seq.t27` | `Seq2seq`, `recurrent_Seq2seq` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/advantage_estimator.t27` | `Advantage`, `rl_Advantage` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/dqn.t27` | `Dqn`, `rl_Dqn` | spec edited since sealing; zig, verilog, c, rust differ | PASS 3 |
| `specs/ml/rl/dqn_target_network.t27` | `DqnTarget`, `rl_DqnTarget` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/ppo_actor.t27` | `PpoActor`, `rl_PpoActor` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/ppo_clip_loss.t27` | `PpoClipLoss`, `rl_PpoClipLoss` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/ppo_critic.t27` | `PpoCritic`, `rl_PpoCritic` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/sac_actor.t27` | `SacActor`, `rl_SacActor` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/rl/sac_critic.t27` | `SacCritic`, `rl_SacCritic` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/encoder_block.t27` | `EncoderBlock`, `transformer_EncoderBlock` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/feed_forward.t27` | `transformer_FeedForwardLayer` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/feed_forward_network.t27` | `FeedForward`, `transformer_FeedForward`, `transformer_FeedForwardNetwork` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/mha_block.t27` | `MHABlock`, `transformer_MHABlock` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/multi_head_attn.t27` | `MultiHeadAttention`, `transformer_MultiHeadAttention` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/norm.t27` | `transformer_LayerNorm` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ml/transformer/positional_encoding.t27` | `PositionalEnc`, `transformer_PositionalEnc` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/nn/hslm.t27` | `HSLM`, `nn_HSLM` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/numeric/bigint.t27` | `bigint`, `numeric_BigInt` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/numeric/gf16.t27` | `GF16`, `numeric_triformat-gf16`, `triformat-gf16` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/physics/chimera_best_gamma.t27` | `chimera`, `physics_chimera` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/runtime/instance.t27` | `runtime-instance`, `runtime_runtime-instance` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/server/sse.t27` | `server-sse`, `server_server-sse` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/storage/kv.t27` | `StorageKv`, `storage_StorageKv` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/storage/lock.t27` | `StorageLock`, `storage_StorageLock` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/storage/migrate.t27` | `StorageMigrate`, `storage_StorageMigrate` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/ternary/bigint.t27` | `TernaryBigInt`, `ternary_TernaryBigInt` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/ternary/clocked_counter.t27` | `ternary_ClockedCounter` | spec edited since sealing; zig, verilog, c, rust differ | PASS 1 |
| `specs/ternary/packed_trit.t27` | `PackedTrit`, `ternary_PackedTrit` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tools/tri_to_t27_converter.t27` | `TriToT27Converter`, `tools_TriToT27Converter` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/bitmap.t27` | `TriBitmap`, `collections_TriBitmap` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/bitset.t27` | `TriBitset`, `collections_TriBitset` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/bitvector.t27` | `TriBitvector`, `collections_TriBitvector` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/btree.t27` | `collections_TriBtree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/circular_buffer.t27` | `TriCircularBuffer`, `collections_TriCircularBuffer` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/deque.t27` | `TriDeque`, `collections_TriDeque` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/either.t27` | `TriEither`, `collections_TriEither` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/interval.t27` | `TriInterval`, `collections_TriInterval` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/linked_list.t27` | `TriLinkedList`, `collections_TriLinkedList` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/list.t27` | `TriList`, `collections_TriList` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/lockfree_stack.t27` | `TriLockfreeStack`, `collections_TriLockfreeStack` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/lru.t27` | `TriLru`, `collections_TriLru` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/lru_cache.t27` | `TriLruCache`, `collections_TriLruCache` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/map.t27` | `TriMap`, `collections_TriMap` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/option.t27` | `TriOption`, `collections_TriOption` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/priority_queue.t27` | `TriPriorityQueue`, `collections_TriPriorityQueue` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/queue.t27` | `TriQueue`, `collections_TriQueue` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/result.t27` | `TriResult`, `collections_TriResult` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/ring_buffer.t27` | `TriRing`, `collections_TriRing` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/set.t27` | `TriSet`, `collections_TriSet` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/skip_list.t27` | `TriSkipList`, `collections_TriSkipList` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/stack.t27` | `TriStack`, `collections_TriStack` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/state.t27` | `TriState`, `collections_TriState` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/tuple.t27` | `TriTuple`, `collections_TriTuple` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/collections/variant.t27` | `TriVariant`, `collections_TriVariant` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/base32.t27` | `TriBase32`, `crypto_TriBase32` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/base64.t27` | `TriBase64`, `crypto_TriBase64` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/crypto.t27` | `TriCrypto`, `crypto_TriCrypto` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/ecc.t27` | `TriEcc`, `crypto_TriEcc` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/hex.t27` | `TriHex`, `crypto_TriHex` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/hmac.t27` | `TriHmac`, `crypto_TriHmac` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/reed_solomon.t27` | `TriReedSolomon`, `crypto_TriReedSolomon` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/crypto/rsa.t27` | `TriRsa`, `crypto_TriRsa` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/bson.t27` | `TriBson`, `encoding_TriBson` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/csv.t27` | `TriCsv`, `encoding_TriCsv` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/json.t27` | `TriJson`, `encoding_TriJson` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/markup.t27` | `TriMarkup`, `encoding_TriMarkup` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/mime.t27` | `TriMime`, `encoding_TriMime` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/encoding/msgpack.t27` | `TriMsgpack`, `encoding_TriMsgpack` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/graph/disjoint_set.t27` | `TriDisjointSet`, `graph_TriDisjointSet` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/graph/graph.t27` | `TriGraph`, `graph_TriGraph` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/graph/topological_sort.t27` | `TriTopological`, `graph_TriTopological` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/compress.t27` | `TriCompress`, `io_TriCompress` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/filesystem.t27` | `TriFilesystem`, `io_TriFilesystem` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/fs.t27` | `TriFs`, `io_TriFs` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/io.t27` | `TriIo`, `io_TriIo` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/reader.t27` | `TriReader`, `io_TriReader` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/writer.t27` | `TriWriter`, `io_TriWriter` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/io/zip.t27` | `TriZipper`, `io_TriZipper` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/math/matrix.t27` | `TriMatrix`, `math_TriMatrix` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/math/polynomial.t27` | `TriPolynomial`, `math_TriPolynomial` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/math/probability.t27` | `TriProbability`, `math_TriProbability` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/math/statistics.t27` | `TriStatistics`, `math_TriStatistics` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/async.t27` | `TriAsync`, `net_TriAsync` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/async_stream.t27` | `TriAsyncStream`, `net_TriAsyncStream` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/channel.t27` | `TriChannel`, `net_TriChannel` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/http.t27` | `TriHttp`, `net_TriHttp` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/net.t27` | `TriNet`, `net_TriNet` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/net/url.t27` | `TriUrl`, `net_TriUrl` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/pipeline/builder.t27` | `TriBuilder`, `pipeline_TriBuilder` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/pipeline/pipeline_parallel.t27` | `TriPipelineParallel`, `pipeline_TriPipelineParallel` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/bloom_filter.t27` | `TriBloomFilter`, `search_TriBloomFilter` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/boyer_moore.t27` | `TriBoyerMoore`, `search_TriBoyerMoore` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/knuth_morris_pratt.t27` | `TriKmp`, `search_TriKmp` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/match.t27` | `search_SearchMatch`, `search_[]const u8` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/rabin_karp.t27` | `TriRabinKarp`, `search_TriRabinKarp` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/regex.t27` | `TriRegex`, `search_TriRegex` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/regex_advanced.t27` | `TriRegexAdvanced`, `search_TriRegexAdvanced` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/search/search.t27` | `TriSearch`, `search_TriSearch` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/sort/quick_sort.t27` | `TriQuickSort`, `sort_TriQuickSort` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/sort/radix_sort.t27` | `TriRadixSort`, `sort_TriRadixSort` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/sort/selection_sort.t27` | `TriSelectionSort`, `sort_TriSelectionSort` | spec edited since sealing; zig, verilog, c, rust differ | PASS 5 |
| `specs/tri/sort/shell_sort.t27` | `TriShellSort`, `sort_TriShellSort` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/sort/sort.t27` | `TriSort`, `sort_TriSort` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/avl_tree.t27` | `TriAvlTree`, `trees_TriAvlTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/b_tree.t27` | `TriBTree`, `trees_TriBTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/fenwick_tree.t27` | `TriFenwick`, `trees_TriFenwick` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/kd_tree.t27` | `TriKdTree`, `trees_TriKdTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/octree.t27` | `TriOctree`, `trees_TriOctree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/quadtree.t27` | `TriQuadtree`, `trees_TriQuadtree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/red_black_tree.t27` | `TriRbTree`, `trees_TriRbTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/rtree.t27` | `TriRtree`, `trees_TriRtree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/segment_tree.t27` | `TriSegmentTree`, `trees_TriSegmentTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/splay_tree.t27` | `TriSplayTree`, `trees_TriSplayTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/suffix_array.t27` | `TriSuffixArray`, `trees_TriSuffixArray` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/tree.t27` | `TriTree`, `trees_TriTree` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/trees/trie.t27` | `TriTrie`, `trees_TriTrie` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/args.t27` | `[]const u8`, `utils_TriArgs`, `utils_[]const u8` | spec edited since sealing; zig, verilog, c, rust differ | PASS 1 |
| `specs/tri/utils/bytes.t27` | `TriBytes`, `utils_TriBytes` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/color.t27` | `TriColor`, `utils_TriColor` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/config.t27` | `TriConfig`, `utils_TriConfig` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/logger.t27` | `"[]const u8"`, `utils_"[]const u8"`, `utils_TriLogger` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/logging.t27` | `TriLogging`, `utils_TriLogging` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/text.t27` | `TriText`, `utils_TriText` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/time.t27` | `TriTime`, `utils_TriTime` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/utf8.t27` | `TriUtf8`, `utils_TriUtf8` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/tri/utils/version.t27` | `TriVersion`, `utils_TriVersion` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/vm/jit_semantics.t27` | `JitSemantics`, `vm_JitSemantics` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/vsa/jones_polynomial.t27` | `JonesPolynomial`, `vsa_JonesPolynomial` | spec edited since sealing; zig, c differ | BLOCKED |
| `specs/vsa/packed_vsa.t27` | `PackedVsa`, `vsa_PackedVsa` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `specs/vsa/sequence_hdc.t27` | `SequenceHdc`, `vsa_SequenceHdc` | spec edited since sealing; zig, verilog, c, rust differ | BLOCKED |
| `test_highlight.t27` | `test_highlight` | spec edited since sealing; zig, verilog, c, rust differ | PASS 0 |
