# NOW -- icarus-simulate learns imports: 95 -> 13 elaboration errors on mac (2026-08-22)

## the simulation path plumbs spec_path into the import-aware registries (Refs #2241)

- `compile_verilog_for_simulation` passed `None` as the spec path, so the
  import-aware registries (#2401's structs, the wave's enums) never loaded on
  the SIMULATION path -- every `Enum.variant` in a test block emitted an
  unbound name and icarus-simulate failed with 95 elaboration errors on mac
  while plain gen-verilog of the same spec elaborated clean.
- One plumbed parameter (`compile_verilog_for_simulation_at`) and two call
  sites later: 95 -> 13. The remainder is one named class -- test-local
  arrays of structs (`results_raw`) flattened in the test-block emission
  path -- plus the third caller (is_icarus_lowerable) still pathless, which
  can only misjudge lowerability, not emit wrong code.
- fifo assessed honestly: FifoConfig carries a &str field, so the struct is
  not lowerable and its config-model functions cannot elaborate -- the fifo
  "vectors" are descriptive (id+prose, no data fields) and their semantics
  already live in the spec's own test blocks. fifo joins the named-debt list,
  not the registry.
