import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Emitter
import Trinity.IcarusLowerable.Soundness
open Trinity.IcarusLowerable


def api_sdk_contract_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def api_sdk_contract_module : Module := {
  name := "api_sdk_contract",
  imports := [],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def api_tri_net_api_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def api_tri_net_api_module : Module := {
  name := "api_tri_net_api",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := [{ name := "", params := [], ret := none, body := [] }, { name := "", params := [], ret := none, body := [] }]
}

def ar_asp_solver_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_asp_solver_module : Module := {
  name := "ar_asp_solver",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_coa_planning_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_coa_planning_module : Module := {
  name := "ar_coa_planning",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_composition_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_composition_module : Module := {
  name := "ar_composition",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_datalog_engine_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_datalog_engine_module : Module := {
  name := "ar_datalog_engine",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_explainability_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_explainability_module : Module := {
  name := "ar_explainability",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_proof_trace_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_proof_trace_module : Module := {
  name := "ar_proof_trace",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_restraint_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_restraint_module : Module := {
  name := "ar_restraint",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ar_ternary_logic_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def ar_ternary_logic_module : Module := {
  name := "ar_ternary_logic",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def automation_wrapup_auto_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def automation_wrapup_auto_module : Module := {
  name := "automation_wrapup_auto",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def base_debounce_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("", ("math::sacred_physics", ""))],
  hostOnly := [],
  reachable := []
}

def base_debounce_module : Module := {
  name := "base_debounce",
  imports := [{ path := "math::sacred_physics::", items := [""] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def base_ring_32_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("", ("math::sacred_physics", ""))],
  hostOnly := [],
  reachable := []
}

def base_ring_32_module : Module := {
  name := "base_ring_32",
  imports := [{ path := "math::sacred_physics::", items := [""] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def base_ternary_encoding_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def base_ternary_encoding_module : Module := {
  name := "base_ternary_encoding",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "ENCODING_BALANCED" (.u8) (some (.intLit (0))), .constDecl "ENCODING_UNIPOLAR" (.u8) (some (.intLit (1))), .constDecl "ENCODING_BCT" (.u8) (some (.intLit (2))), .constDecl "BITS_PER_BYTE" (.struct "usize") (some (.intLit (8))), .constDecl "TRITS_PER_BYTE" (.struct "usize") (some (.intLit (6))), .constDecl "TRITS_PER_NYBBLE" (.struct "usize") (some (.intLit (3)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "bit_to_trit_pair_zero", params := [], ret := none, body := [] }, { name := "bit_to_trit_pair_one", params := [], ret := none, body := [] }, { name := "bits_to_trits_zero", params := [], ret := none, body := [] }, { name := "bits_to_trits_max_nibble", params := [], ret := none, body := [] }, { name := "trits_to_bits_roundtrip", params := [], ret := none, body := [] }, { name := "byte_to_trits_roundtrip", params := [], ret := none, body := [] }, { name := "balanced_unipolar_conversion", params := [], ret := none, body := [] }, { name := "is_valid_trit_check", params := [], ret := none, body := [] }, { name := "is_valid_unipolar_trit_check", params := [], ret := none, body := [] }, { name := "char_encoding_roundtrip", params := [], ret := none, body := [] }],
  benches := [{ name := "byte_to_trits_performance", params := [], ret := none, body := [] }, { name := "trits_to_byte_performance", params := [], ret := none, body := [] }, { name := "balanced_unipolar_performance", params := [], ret := none, body := [] }]
}

def base_ternary_memory_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def base_ternary_memory_module : Module := {
  name := "base_ternary_memory",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "TRIT_CAPACITY" (.struct "usize") (some (.intLit (27))), .constDecl "WORD_CAPACITY" (.struct "usize") (some (.intLit (1024))), .constDecl "PAGE_SIZE" (.struct "usize") (some (.intLit (256))), .constDecl "STATE_FREE" (.u8) (some (.intLit (0))), .constDecl "STATE_ALLOCATED" (.u8) (some (.intLit (1))), .constDecl "STATE_LOCKED" (.u8) (some (.intLit (2))), .constDecl "STATE_DIRTY" (.u8) (some (.intLit (3)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "trit_cell_initialization", params := [], ret := none, body := [] }, { name := "trit_cell_write_read", params := [], ret := none, body := [] }, { name := "trit_cell_locked", params := [], ret := none, body := [] }, { name := "ternary_word_initialization", params := [], ret := none, body := [] }, { name := "ternary_word_write_read_trit", params := [], ret := none, body := [] }, { name := "ternary_word_bounds_check", params := [], ret := none, body := [] }, { name := "ternary_memory_bank_initialization", params := [], ret := none, body := [] }, { name := "ternary_memory_alloc_free", params := [], ret := none, body := [] }, { name := "ternary_memory_write_read", params := [], ret := none, body := [] }],
  benches := [{ name := "trit_cell_write_performance", params := [], ret := none, body := [] }, { name := "ternary_word_read_performance", params := [], ret := none, body := [] }, { name := "ternary_memory_bank_access_performance", params := [], ret := none, body := [] }, { name := "checksum_computation_performance", params := [], ret := none, body := [] }]
}

def benchmarks_bench_main_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("different", ("different", "different"))],
  hostOnly := [],
  reachable := []
}

def benchmarks_bench_main_module : Module := {
  name := "benchmarks_bench_main",
  imports := [{ path := "different", items := ["different"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def benchmarks_bench_nn_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def benchmarks_bench_nn_module : Module := {
  name := "benchmarks_bench_nn",
  imports := [],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "NN-Bench: quantization roundtrip", params := [], ret := none, body := [] }, { name := "NN-Bench: GF16 accuracy", params := [], ret := none, body := [] }, { name := "NN-Bench: forward pass consistency", params := [], ret := none, body := [] }],
  benches := []
}

def benchmarks_gf16_bfloat16_nmse_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def benchmarks_gf16_bfloat16_nmse_module : Module := {
  name := "benchmarks_gf16_bfloat16_nmse",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "NMSE-Protocol: identity witness gates run", params := [], ret := none, body := [] }, { name := "NMSE-Protocol: non-negative", params := [], ret := none, body := [] }, { name := "NMSE-Protocol: deterministic for fixed seed", params := [], ret := none, body := [] }],
  benches := [{ name := "GF16 vs BF16 NMSE over D_NORM, D_LOG, D_RELU, D_PHI, D_DEEP", params := [], ret := none, body := [] }]
}

def brain_brain_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["expect"]
}

def brain_brain_module : Module := {
  name := "brain_brain",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "Brain-Atlas: region count", params := [], ret := none, body := [.bareCall (.call "expect" [(.binop "==" (.fieldAccess (.identifier "BRAIN_ATLAS") "len") (.intLit (23)))])] }, { name := "Brain-Atlas: dependency graph", params := [], ret := none, body := [.bareCall (.call "expect" [(.binop "==" (.fieldAccess (.identifier "REGION_DEPENDENCIES") "len") (.intLit (23)))])] }],
  benches := []
}

def brain_bus_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["@as", "std.testing.expectEqual", "brain_bus_version"]
}

def brain_bus_module : Module := {
  name := "brain_bus",
  imports := [],
  globals := [.constDecl "BRAIN_BUS_VERSION" (.u32) (some (.intLit (1)))],
  functions := [{ name := "brain_bus_version", params := [], ret := (some (.u32)), body := [.return_ (some (.identifier "BRAIN_BUS_VERSION"))] }],
  tests := [{ name := "brain_bus_version_stable", params := [], ret := none, body := [.bareCall (.unop "try " (.call "std.testing.expectEqual" [(.call "@as" [(.identifier "u32"), (.intLit (1))]), (.call "brain_bus_version" [])]))] }],
  benches := []
}

def brain_cognitive_loop_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["cognitive_loop_phase_count", "std.testing.expectEqual", "@as"]
}

def brain_cognitive_loop_module : Module := {
  name := "brain_cognitive_loop",
  imports := [],
  globals := [.constDecl "COGNITIVE_PHASE_COUNT" (.u8) (some (.intLit (5)))],
  functions := [{ name := "cognitive_loop_phase_count", params := [], ret := (some (.u8)), body := [.return_ (some (.identifier "COGNITIVE_PHASE_COUNT"))] }],
  tests := [{ name := "cognitive_loop_five_phases", params := [], ret := none, body := [.bareCall (.unop "try " (.call "std.testing.expectEqual" [(.call "@as" [(.identifier "u8"), (.intLit (5))]), (.call "cognitive_loop_phase_count" [])]))] }],
  benches := []
}

def brain_neural_gamma_env : Env := {
  structs := [],
  constructors := [],
  enums := ["ConsciousnessState"],
  imports := [],
  hostOnly := [],
  reachable := ["expect"]
}

def brain_neural_gamma_module : Module := {
  name := "brain_neural_gamma",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "Neural-67: phi cubed and gamma", params := [], ret := none, body := [] }, { name := "Neural-71: TRINITY identity", params := [], ret := none, body := [.bareCall (.call "expect" [(.identifier "TRINITY"), (.intLit (72)), (.intLit (0))])] }, { name := "Neural-73: consciousness threshold", params := [], ret := none, body := [] }, { name := "Neural-76: gamma frequency", params := [], ret := none, body := [] }],
  benches := []
}

def cloud_railway_deploy_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("", ("math::sacred_physics", "")), ("Trit", ("base::types", "Trit"))],
  hostOnly := [],
  reachable := []
}

def cloud_railway_deploy_module : Module := {
  name := "cloud_railway_deploy",
  imports := [{ path := "base::types::Trit", items := ["Trit"] }, { path := "math::sacred_physics::", items := [""] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def compiler_mod_structure_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("", ("math::sacred_physics", ""))],
  hostOnly := [],
  reachable := []
}

def compiler_mod_structure_module : Module := {
  name := "compiler_mod_structure",
  imports := [{ path := "math::sacred_physics::", items := [""] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def config_load_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("config", ("config", "config")), ("std", ("std", "std"))],
  hostOnly := [],
  reachable := []
}

def config_load_module : Module := {
  name := "config_load",
  imports := [{ path := "std", items := ["std"] }, { path := "config", items := ["config"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def demos_simple_test_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def demos_simple_test_module : Module := {
  name := "demos_simple_test",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TEST_VALUE" (.u8) (some (.intLit (42)))],
  functions := [],
  tests := [{ name := "simple_test", params := [], ret := none, body := [] }],
  benches := []
}

def fpga_bootrom_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("boot_config", "BootConfig"), ("boot_stage", "BootStage")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def fpga_bootrom_module : Module := {
  name := "fpga_bootrom",
  imports := [],
  globals := [],
  functions := [{ name := "boot_stage", params := [("name", (.u32)), ("str", (.u32)), ("idx", (.u32)), ("size", (.u32)), ("entry", (.u32))], ret := (some (.struct "BootStage")), body := [.return_ (some (.structLit "BootStage" [("name", (.identifier "name")), ("index", (.identifier "idx")), ("size_bytes", (.identifier "size")), ("entry_addr", (.identifier "entry"))]))] }, { name := "stage_end", params := [("s", (.struct "BootStage"))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.fieldAccess (.identifier "s") "entry_addr") (.fieldAccess (.identifier "s") "size_bytes")))] }, { name := "boot_config", params := [("name", (.u32)), ("str", (.u32)), ("rom_size", (.u32))], ret := (some (.struct "BootConfig")), body := [.return_ (some (.structLit "BootConfig" [("name", (.identifier "name")), ("rom_base", (.intLit (0))), ("rom_size", (.identifier "rom_size")), ("has_integrity_check", (.intLit (0))), ("has_chain_loader", (.intLit (0)))]))] }, { name := "validate_config", params := [("cfg", (.struct "BootConfig"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "config_end", params := [("cfg", (.struct "BootConfig"))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.fieldAccess (.identifier "cfg") "rom_base") (.fieldAccess (.identifier "cfg") "rom_size")))] }, { name := "fits", params := [("cfg", (.struct "BootConfig")), ("stages", (.struct "[BootStage]")), ("count", (.u32))], ret := (some (.bool)), body := [.varDecl "total" (.u32) (some (.intLit (0))), .varDecl "i" (.u32) (some (.intLit (0)))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "boot_stage_creation", params := [], ret := none, body := [] }, { name := "boot_config_creation", params := [], ret := none, body := [] }, { name := "validate_config_ok", params := [], ret := none, body := [] }, { name := "validate_config_empty", params := [], ret := none, body := [] }, { name := "fits_yes", params := [], ret := none, body := [] }, { name := "fits_no", params := [], ret := none, body := [] }, { name := "stage_end_nonzero", params := [], ret := none, body := [] }, { name := "fits_exact", params := [], ret := none, body := [] }, { name := "fits_empty_stages", params := [], ret := none, body := [] }, { name := "validate_config_zero_size", params := [], ret := none, body := [] }, { name := "validate_config_valid_name_size", params := [], ret := none, body := [] }, { name := "boot_stage_indexing", params := [], ret := none, body := [] }],
  benches := [{ name := "fits_check_latency", params := [], ret := none, body := [] }]
}

def fpga_cts_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("cts_ok", "CtsReport"), ("bufg", "ClockBuffer"), ("clock_tree", "ClockTree"), ("pll_config", "PllConfig"), ("bufh", "ClockBuffer")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def fpga_cts_module : Module := {
  name := "fpga_cts",
  imports := [],
  globals := [],
  functions := [{ name := "pll_config", params := [("name", (.u32)), ("str", (.u32)), ("input_mhz", (.u32)), ("output_mhz", (.u32))], ret := (some (.struct "PllConfig")), body := [.varDecl "m" (.u32) (some (.intLit (1))), .varDecl "d" (.u32) (some (.intLit (1)))] }, { name := "pll_period_ps", params := [("pll", (.struct "PllConfig"))], ret := (some (.u32)), body := [] }, { name := "bufg", params := [("name", (.u32)), ("str", (.u32))], ret := (some (.struct "ClockBuffer")), body := [.return_ (some (.structLit "ClockBuffer" [("name", (.identifier "name")), ("delay_ps", (.intLit (100))), ("fanout", (.intLit (32)))]))] }, { name := "bufh", params := [("name", (.u32)), ("str", (.u32))], ret := (some (.struct "ClockBuffer")), body := [.return_ (some (.structLit "ClockBuffer" [("name", (.identifier "name")), ("delay_ps", (.intLit (50))), ("fanout", (.intLit (16)))]))] }, { name := "bufg_has_higher_fanout", params := [("b", (.struct "ClockBuffer"))], ret := (some (.bool)), body := [.return_ (some (.binop ">=" (.fieldAccess (.identifier "b") "fanout") (.intLit (32))))] }, { name := "clock_tree", params := [("root", (.u32)), ("str", (.u32)), ("levels", (.u32)), ("bufs", (.u32))], ret := (some (.struct "ClockTree")), body := [.return_ (some (.structLit "ClockTree" [("root", (.identifier "root")), ("num_levels", (.identifier "levels")), ("total_buffers", (.identifier "bufs")), ("max_skew_ps", (.intLit (100)))]))] }, { name := "tree_delay_ps", params := [("tree", (.struct "ClockTree")), ("buf_delay", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "*" (.fieldAccess (.identifier "tree") "num_levels") (.identifier "buf_delay")))] }, { name := "skew_ok", params := [("tree", (.struct "ClockTree")), ("max_allowed_ps", (.u32))], ret := (some (.bool)), body := [.return_ (some (.binop "<=" (.fieldAccess (.identifier "tree") "max_skew_ps") (.identifier "max_allowed_ps")))] }, { name := "cts_ok", params := [("clocks", (.u32)), ("plls", (.u32)), ("bufs", (.u32)), ("skew", (.u32)), ("latency", (.u32))], ret := (some (.struct "CtsReport")), body := [.return_ (some (.structLit "CtsReport" [("num_clocks", (.identifier "clocks")), ("num_plls", (.identifier "plls")), ("total_buffers", (.identifier "bufs")), ("worst_skew_ps", (.identifier "skew")), ("worst_latency_ps", (.identifier "latency")), ("has_violations", (.intLit (0)))]))] }, { name := "passed", params := [("r", (.struct "CtsReport"))], ret := (some (.bool)), body := [.return_ (some (.binop "==" (.fieldAccess (.identifier "r") "has_violations") (.intLit (0))))] }, { name := "est_buffers_needed", params := [("num_sinks", (.u32))], ret := (some (.u32)), body := [] }, { name := "est_tree_levels", params := [("num_sinks", (.u32))], ret := (some (.u32)), body := [] }, { name := "validate_pll", params := [("pll", (.struct "PllConfig"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "pll_config_creation", params := [], ret := none, body := [] }, { name := "bufg_creation", params := [], ret := none, body := [] }, { name := "bufh_creation", params := [], ret := none, body := [] }, { name := "clock_tree_creation", params := [], ret := none, body := [] }, { name := "tree_delay", params := [], ret := none, body := [] }, { name := "skew_ok_yes", params := [], ret := none, body := [] }, { name := "skew_ok_no", params := [], ret := none, body := [] }, { name := "cts_report_ok", params := [], ret := none, body := [] }, { name := "est_buffers_one", params := [], ret := none, body := [] }, { name := "est_buffers_many", params := [], ret := none, body := [] }, { name := "est_tree_levels_one", params := [], ret := none, body := [] }, { name := "est_tree_levels_two", params := [], ret := none, body := [] }, { name := "est_tree_levels_three", params := [], ret := none, body := [] }, { name := "validate_pll_ok", params := [], ret := none, body := [] }, { name := "validate_pll_empty", params := [], ret := none, body := [] }],
  benches := [{ name := "buffer_estimation_latency", params := [], ret := none, body := [] }, { name := "tree_level_estimation_latency", params := [], ret := none, body := [] }]
}

def fpga_dft_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("scan_chain", "ScanChain"), ("memory_bist", "BistCtrl"), ("logic_bist", "BistCtrl"), ("test_coverage", "TestCoverage"), ("jtag_tap", "JtagTap")],
  enums := ["BistKind"],
  imports := [],
  hostOnly := [],
  reachable := []
}

def fpga_dft_module : Module := {
  name := "fpga_dft",
  imports := [],
  globals := [],
  functions := [{ name := "scan_chain", params := [("name", (.u32)), ("str", (.u32)), ("regs", (.u32))], ret := (some (.struct "ScanChain")), body := [.return_ (some (.structLit "ScanChain" [("name", (.identifier "name")), ("num_regs", (.identifier "regs")), ("chain_length_bits", (.binop "*" (.identifier "regs") (.intLit (32))))]))] }, { name := "scan_chain_cycles", params := [("chain", (.struct "ScanChain"))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.fieldAccess (.identifier "chain") "chain_length_bits") (.intLit (10))))] }, { name := "scan_chain_bytes", params := [("chain", (.struct "ScanChain"))], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.fieldAccess (.identifier "chain") "chain_length_bits") (.intLit (8))))] }, { name := "memory_bist", params := [("name", (.u32)), ("str", (.u32)), ("patterns", (.u32))], ret := (some (.struct "BistCtrl")), body := [.return_ (some (.structLit "BistCtrl" [("name", (.identifier "name")), ("kind", (.intLit (0))), ("patterns", (.identifier "patterns")), ("pass_threshold", (.identifier "patterns"))]))] }, { name := "logic_bist", params := [("name", (.u32)), ("str", (.u32)), ("patterns", (.u32))], ret := (some (.struct "BistCtrl")), body := [.return_ (some (.structLit "BistCtrl" [("name", (.identifier "name")), ("kind", (.intLit (1))), ("patterns", (.identifier "patterns")), ("pass_threshold", (.identifier "patterns"))]))] }, { name := "bist_cycles", params := [("ctrl", (.struct "BistCtrl"))], ret := (some (.u32)), body := [.return_ (some (.binop "*" (.fieldAccess (.identifier "ctrl") "patterns") (.intLit (2))))] }, { name := "bist_coverage", params := [("ctrl", (.struct "BistCtrl")), ("total_faults", (.u32))], ret := (some (.u32)), body := [] }, { name := "jtag_tap", params := [("name", (.u32)), ("str", (.u32)), ("ir_width", (.u32)), ("idcode", (.u32))], ret := (some (.struct "JtagTap")), body := [.return_ (some (.structLit "JtagTap" [("name", (.identifier "name")), ("ir_width", (.identifier "ir_width")), ("num_dr_regs", (.intLit (3))), ("bypass_code", (.intLit (255))), ("idcode", (.identifier "idcode"))]))] }, { name := "tap_total_bits", params := [("tap", (.struct "JtagTap"))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.fieldAccess (.identifier "tap") "ir_width") (.binop "*" (.intLit (32)) (.fieldAccess (.identifier "tap") "num_dr_regs"))))] }, { name := "tap_state_count", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (16)))] }, { name := "test_coverage", params := [("scan", (.u32)), ("bist", (.u32)), ("atpg", (.u32))], ret := (some (.struct "TestCoverage")), body := [.return_ (some (.structLit "TestCoverage" [("scan_coverage", (.identifier "scan")), ("bist_coverage", (.identifier "bist")), ("atpg_coverage", (.identifier "atpg")), ("total_coverage", (.binop "/" (.binop "+" (.binop "+" (.identifier "scan") (.identifier "bist")) (.identifier "atpg")) (.intLit (3))))]))] }, { name := "is_acceptable", params := [("cov", (.struct "TestCoverage"))], ret := (some (.bool)), body := [.return_ (some (.binop ">=" (.fieldAccess (.identifier "cov") "total_coverage") (.intLit (90))))] }, { name := "validate_chain", params := [("chain", (.struct "ScanChain"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "validate_bist", params := [("ctrl", (.struct "BistCtrl"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "validate_tap", params := [("tap", (.struct "JtagTap"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "scan_chain_creation", params := [], ret := none, body := [] }, { name := "memory_bist_creation", params := [], ret := none, body := [] }, { name := "logic_bist_creation", params := [], ret := none, body := [] }, { name := "bist_coverage_full", params := [], ret := none, body := [] }, { name := "bist_coverage_partial", params := [], ret := none, body := [] }, { name := "bist_coverage_zero_faults", params := [], ret := none, body := [] }, { name := "jtag_tap_creation", params := [], ret := none, body := [] }, { name := "tap_state_count", params := [], ret := none, body := [] }, { name := "test_coverage_creation", params := [], ret := none, body := [] }, { name := "test_coverage_acceptable", params := [], ret := none, body := [] }, { name := "test_coverage_not_acceptable", params := [], ret := none, body := [] }, { name := "validate_chain_ok", params := [], ret := none, body := [] }, { name := "validate_chain_empty", params := [], ret := none, body := [] }, { name := "validate_bist_ok", params := [], ret := none, body := [] }, { name := "validate_bist_empty", params := [], ret := none, body := [] }, { name := "validate_tap_ok", params := [], ret := none, body := [] }, { name := "validate_tap_empty", params := [], ret := none, body := [] }],
  benches := [{ name := "dft_latency", params := [], ret := none, body := [] }]
}

def fpga_hw_types_env : Env := {
  structs := [("HwType", [("value", .u32)])],
  constructors := [("hw_bits", "HwType"), ("hw_bool", "HwType"), ("hw_gf16", "HwType"), ("hw_reset", "HwType"), ("hw_clock", "HwType"), ("hw_sint", "HwType"), ("hw_uint", "HwType"), ("hw_vector", "HwType")],
  enums := ["ResetKind", "ResetPolarity", "HwTypeTag"],
  imports := [],
  hostOnly := [],
  reachable := []
}

def fpga_hw_types_module : Module := {
  name := "fpga_hw_types",
  imports := [],
  globals := [],
  functions := [{ name := "hw_bits", params := [("w", (.u32))], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (0))), ("width", (.identifier "w")), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_uint", params := [("w", (.u32))], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (1))), ("width", (.identifier "w")), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_sint", params := [("w", (.u32))], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (2))), ("width", (.identifier "w")), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_bool", params := [], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (3))), ("width", (.intLit (1))), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_clock", params := [], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (4))), ("width", (.intLit (1))), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_reset", params := [], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (5))), ("width", (.intLit (1))), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_vector", params := [("elem", (.struct "HwType")), ("len", (.u32))], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (6))), ("width", (.binop "*" (.fieldAccess (.identifier "elem") "width") (.identifier "len"))), ("is_signed_flag", (.fieldAccess (.identifier "elem") "is_signed_flag")), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.fieldAccess (.identifier "elem") "tag")), ("vec_len", (.identifier "len")), ("field_count", (.intLit (0)))]))] }, { name := "hw_gf16", params := [], ret := (some (.struct "HwType")), body := [.return_ (some (.structLit "HwType" [("tag", (.intLit (9))), ("width", (.intLit (16))), ("is_signed_flag", (.intLit (0))), ("is_clock_flag", (.intLit (0))), ("is_reset_flag", (.intLit (0))), ("elem_tag", (.intLit (0))), ("vec_len", (.intLit (0))), ("field_count", (.intLit (0)))]))] }, { name := "hw_width", params := [("ty", (.struct "HwType"))], ret := (some (.u32)), body := [.return_ (some (.fieldAccess (.identifier "ty") "width"))] }, { name := "is_signed", params := [("ty", (.struct "HwType"))], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "ty") "is_signed_flag"))] }, { name := "is_clock_like", params := [("ty", (.struct "HwType"))], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "ty") "is_clock_flag"))] }, { name := "is_reset_like", params := [("ty", (.struct "HwType"))], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "ty") "is_reset_flag"))] }, { name := "types_equal", params := [("a", (.struct "HwType")), ("b", (.struct "HwType"))], ret := (some (.bool)), body := [.ifThenElse (.binop "!=" (.fieldAccess (.identifier "a") "width") (.fieldAccess (.identifier "b") "width")) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "!=" (.fieldAccess (.identifier "a") "is_signed_flag") (.fieldAccess (.identifier "b") "is_signed_flag")) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "!=" (.fieldAccess (.identifier "a") "is_clock_flag") (.fieldAccess (.identifier "b") "is_clock_flag")) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "!=" (.fieldAccess (.identifier "a") "is_reset_flag") (.fieldAccess (.identifier "b") "is_reset_flag")) [.return_ (some (.intLit (0)))] [], .return_ (some (.intLit (0)))] }, { name := "verilog_range", params := [("ty", (.struct "HwType"))], ret := (some (.bool)), body := [.return_ (some (.binop ">" (.fieldAccess (.identifier "ty") "width") (.intLit (1))))] }, { name := "is_connectable", params := [("target", (.struct "HwType")), ("source", (.struct "HwType"))], ret := (some (.bool)), body := [.ifThenElse (.binop "!=" (.fieldAccess (.identifier "target") "width") (.fieldAccess (.identifier "source") "width")) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "!=" (.fieldAccess (.identifier "target") "is_clock_flag") (.fieldAccess (.identifier "source") "is_clock_flag")) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "!=" (.fieldAccess (.identifier "target") "is_reset_flag") (.fieldAccess (.identifier "source") "is_reset_flag")) [.return_ (some (.intLit (0)))] [], .return_ (some (.intLit (0)))] }],
  tests := [{ name := "bits8_width", params := [], ret := none, body := [] }, { name := "uint3_width", params := [], ret := none, body := [] }, { name := "bool_width", params := [], ret := none, body := [] }, { name := "clock_width", params := [], ret := none, body := [] }, { name := "reset_width", params := [], ret := none, body := [] }, { name := "vector_width", params := [], ret := none, body := [] }, { name := "gf16_width", params := [], ret := none, body := [] }, { name := "uint_not_signed", params := [], ret := none, body := [] }, { name := "sint_is_signed", params := [], ret := none, body := [] }, { name := "bool_not_signed", params := [], ret := none, body := [] }, { name := "clock_is_clock_like", params := [], ret := none, body := [] }, { name := "uint_not_clock_like", params := [], ret := none, body := [] }, { name := "reset_is_reset_like", params := [], ret := none, body := [] }, { name := "bool_not_reset_like", params := [], ret := none, body := [] }, { name := "vector_of_bools_width", params := [], ret := none, body := [] }, { name := "vector_uint16_2_width", params := [], ret := none, body := [] }, { name := "types_equal_same", params := [], ret := none, body := [] }, { name := "types_equal_diff_width", params := [], ret := none, body := [] }, { name := "types_equal_diff_signed", params := [], ret := none, body := [] }, { name := "is_connectable_same", params := [], ret := none, body := [] }, { name := "is_connectable_diff_width", params := [], ret := none, body := [] }, { name := "is_connectable_clock_mismatch", params := [], ret := none, body := [] }],
  benches := [{ name := "hw_width_latency", params := [], ret := none, body := [] }, { name := "is_clock_like_latency", params := [], ret := none, body := [] }]
}

def fpga_mac_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("registers", ("isa::registers", "registers")), ("types", ("base::types", "types")), ("ops", ("base::ops", "ops"))],
  hostOnly := [],
  reachable := []
}

def fpga_mac_module : Module := {
  name := "fpga_mac",
  imports := [{ path := "base::types", items := ["types"] }, { path := "base::ops", items := ["ops"] }, { path := "isa::registers", items := ["registers"] }],
  globals := [.constDecl "MAC_WIDTH" (.struct "usize") (some (.intLit (27))), .constDecl "MAC_ACC_BITS" (.struct "usize") (some (.intLit (32))), .constDecl "NUM_MAC_UNITS" (.struct "usize") (some (.intLit (8))), .constDecl "PIPELINE_STAGES" (.struct "usize") (some (.intLit (4))), .constDecl "OP_MAC_MUL" (.u8) (some (.intLit (0))), .constDecl "OP_MAC_MAC" (.u8) (some (.intLit (1))), .constDecl "OP_MAC_MACC" (.u8) (some (.intLit (2))), .constDecl "OP_MAC_DOT" (.u8) (some (.intLit (3))), .constDecl "STATUS_READY" (.u8) (some (.intLit (0))), .constDecl "STATUS_BUSY" (.u8) (some (.intLit (1))), .constDecl "STATUS_DONE" (.u8) (some (.intLit (2))), .constDecl "MAC_LUT" (.array 9 (.i8)) (some (.identifier "[1,0,-1,0,0,0,-1,0,1,]")), .constDecl "mac_units" (.struct "[NUM_MAC_UNITS]MACUnit") (some (.arrayLit (.u32) []))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def fpga_power_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("power_domain", "PowerDomain"), ("power_estimate", "PowerEstimate"), ("est_total_power", "PowerEstimate"), ("zero_power", "PowerEstimate")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def fpga_power_module : Module := {
  name := "fpga_power",
  imports := [],
  globals := [],
  functions := [{ name := "power_domain", params := [("name", (.u32)), ("str", (.u32)), ("clock_mhz", (.u32))], ret := (some (.struct "PowerDomain")), body := [.return_ (some (.structLit "PowerDomain" [("name", (.identifier "name")), ("voltage_mv", (.intLit (1000))), ("clock_mhz", (.identifier "clock_mhz")), ("toggle_rate", (.intLit (12)))]))] }, { name := "lut_power_uw_per_mhz", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (10)))] }, { name := "ff_power_uw_per_mhz", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (5)))] }, { name := "bram_power_uw_per_mhz", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (50)))] }, { name := "dsp_power_uw_per_mhz", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (100)))] }, { name := "io_power_uw_per_mhz", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (20)))] }, { name := "static_base_mw", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (50)))] }, { name := "static_per_resource_uw", params := [], ret := (some (.u32)), body := [.return_ (some (.intLit (100)))] }, { name := "power_estimate", params := [("dynamic", (.u32)), ("static_p", (.u32))], ret := (some (.struct "PowerEstimate")), body := [.return_ (some (.structLit "PowerEstimate" [("dynamic_mw", (.identifier "dynamic")), ("static_mw", (.identifier "static_p")), ("total_mw", (.binop "+" (.identifier "dynamic") (.identifier "static_p"))), ("lut_power_uw", (.intLit (0))), ("ff_power_uw", (.intLit (0))), ("bram_power_uw", (.intLit (0))), ("dsp_power_uw", (.intLit (0)))]))] }, { name := "zero_power", params := [], ret := (some (.struct "PowerEstimate")), body := [.return_ (some (.call "power_estimate" [(.intLit (0)), (.intLit (0))]))] }, { name := "est_lut_dynamic", params := [("luts", (.u32)), ("clock_mhz", (.u32)), ("toggle_rate", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.binop "/" (.binop "*" (.binop "*" (.binop "*" (.identifier "luts") (.call "lut_power_uw_per_mhz" [])) (.identifier "clock_mhz")) (.identifier "toggle_rate")) (.intLit (1000))) (.intLit (100))))] }, { name := "est_ff_dynamic", params := [("ffs", (.u32)), ("clock_mhz", (.u32)), ("toggle_rate", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.binop "/" (.binop "*" (.binop "*" (.binop "*" (.identifier "ffs") (.call "ff_power_uw_per_mhz" [])) (.identifier "clock_mhz")) (.identifier "toggle_rate")) (.intLit (1000))) (.intLit (100))))] }, { name := "est_bram_dynamic", params := [("brams", (.u32)), ("clock_mhz", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.binop "*" (.binop "*" (.identifier "brams") (.call "bram_power_uw_per_mhz" [])) (.identifier "clock_mhz")) (.intLit (1000))))] }, { name := "est_dsp_dynamic", params := [("dsps", (.u32)), ("clock_mhz", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.binop "*" (.binop "*" (.identifier "dsps") (.call "dsp_power_uw_per_mhz" [])) (.identifier "clock_mhz")) (.intLit (1000))))] }, { name := "est_static", params := [("total_resources", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.call "static_base_mw" []) (.binop "/" (.binop "*" (.identifier "total_resources") (.call "static_per_resource_uw" [])) (.intLit (1000)))))] }, { name := "total_resources", params := [("luts", (.u32)), ("ffs", (.u32)), ("brams", (.u32)), ("dsps", (.u32))], ret := (some (.u32)), body := [.return_ (some (.binop "+" (.binop "+" (.binop "+" (.identifier "luts") (.identifier "ffs")) (.identifier "brams")) (.identifier "dsps")))] }, { name := "est_total_power", params := [("luts", (.u32)), ("ffs", (.u32)), ("brams", (.u32)), ("dsps", (.u32)), ("clock_mhz", (.u32)), ("toggle_rate", (.u32))], ret := (some (.struct "PowerEstimate")), body := [.varDecl "lut_p" (.u32) (some (.call "est_lut_dynamic" [(.identifier "luts"), (.identifier "clock_mhz"), (.identifier "toggle_rate")])), .varDecl "ff_p" (.u32) (some (.call "est_ff_dynamic" [(.identifier "ffs"), (.identifier "clock_mhz"), (.identifier "toggle_rate")])), .varDecl "bram_p" (.u32) (some (.call "est_bram_dynamic" [(.identifier "brams"), (.identifier "clock_mhz")])), .varDecl "dsp_p" (.u32) (some (.call "est_dsp_dynamic" [(.identifier "dsps"), (.identifier "clock_mhz")])), .varDecl "dyn" (.u32) (some (.binop "+" (.binop "+" (.binop "+" (.identifier "lut_p") (.identifier "ff_p")) (.identifier "bram_p")) (.identifier "dsp_p"))), .varDecl "stat" (.u32) (some (.call "est_static" [(.call "total_resources" [(.identifier "luts"), (.identifier "ffs"), (.identifier "brams"), (.identifier "dsps")])])), .return_ (some (.structLit "PowerEstimate" [("dynamic_mw", (.identifier "dyn")), ("static_mw", (.identifier "stat")), ("total_mw", (.binop "+" (.identifier "dyn") (.identifier "stat"))), ("lut_power_uw", (.identifier "lut_p")), ("ff_power_uw", (.identifier "ff_p")), ("bram_power_uw", (.identifier "bram_p")), ("dsp_power_uw", (.identifier "dsp_p"))]))] }, { name := "validate_domain", params := [("d", (.struct "PowerDomain"))], ret := (some (.u32)), body := [.varDecl "errors" (.u32) (some (.intLit (0)))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "power_domain_creation", params := [], ret := none, body := [] }, { name := "zero_power", params := [], ret := none, body := [] }, { name := "power_estimate_creation", params := [], ret := none, body := [] }, { name := "est_lut_dynamic", params := [], ret := none, body := [] }, { name := "est_ff_dynamic", params := [], ret := none, body := [] }, { name := "est_bram_dynamic", params := [], ret := none, body := [] }, { name := "est_dsp_dynamic", params := [], ret := none, body := [] }, { name := "est_static", params := [], ret := none, body := [] }, { name := "est_static_base", params := [], ret := none, body := [] }, { name := "total_resources_calc", params := [], ret := none, body := [] }, { name := "est_total_power_arty", params := [], ret := none, body := [] }, { name := "power_constants", params := [], ret := none, body := [] }, { name := "validate_domain_ok", params := [], ret := none, body := [] }, { name := "validate_domain_empty", params := [], ret := none, body := [] }, { name := "validate_domain_zero_clock", params := [], ret := none, body := [] }],
  benches := [{ name := "power_est_latency", params := [], ret := none, body := [] }]
}

def fpga_spi_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def fpga_spi_module : Module := {
  name := "fpga_spi",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "CLK_FREQ" (.u32) (some (.intLit (0))), .constDecl "SPI_CPOL" (.u8) (some (.intLit (0))), .constDecl "SPI_CPHA" (.u8) (some (.intLit (0))), .constDecl "MAX_DATA_WIDTH" (.u8) (some (.intLit (32))), .constDecl "CS_ASSERT_DELAY" (.u32) (some (.intLit (100))), .constDecl "CS_DEASSERT_DELAY" (.u32) (some (.intLit (100))), .constDecl "PRESCALER_2" (.u8) (some (.intLit (0))), .constDecl "PRESCALER_4" (.u8) (some (.intLit (1))), .constDecl "PRESCALER_8" (.u8) (some (.intLit (2))), .constDecl "PRESCALER_16" (.u8) (some (.intLit (3))), .constDecl "PRESCALER_32" (.u8) (some (.intLit (4))), .constDecl "PRESCALER_64" (.u8) (some (.intLit (5))), .constDecl "PRESCALER_128" (.u8) (some (.intLit (6))), .constDecl "PRESCALER_256" (.u8) (some (.intLit (7))), .constDecl "SPI_IDLE" (.u8) (some (.intLit (0))), .constDecl "SPI_CS_ASSERT" (.u8) (some (.intLit (1))), .constDecl "SPI_TRANSFER" (.u8) (some (.intLit (2))), .constDecl "SPI_CS_DEASSERT" (.u8) (some (.intLit (3))), .constDecl "TX_BIT" (.u8) (some (.intLit (0))), .constDecl "RX_BIT" (.u8) (some (.intLit (1))), .constDecl "WAIT_EDGE" (.u8) (some (.intLit (2))), .constDecl "spi" (.struct "SPI_Master_Unit") (some (.structLit "SPI_Master_Unit" [("state", (.identifier "SPI_IDLE")), ("tx_state", (.identifier "TX_BIT")), ("cs_asserted", (.intLit (0))), ("busy", (.intLit (0))), ("prescaler", (.identifier "PRESCALER_16")), ("data_width", (.intLit (8))), ("cs_mode", (.intLit (0))), ("tx_data", (.intLit (0))), ("rx_data", (.intLit (0))), ("bit_count", (.intLit (0))), ("bit_counter", (.intLit (0))), ("cs_assert_cnt", (.intLit (0))), ("cs_deassert_cnt", (.intLit (0)))]))],
  functions := [{ name := "spi_set_prescaler", params := [("psc", (.u8))], ret := (some (.bool)), body := [.ifThenElse (.binop ">" (.identifier "psc") (.identifier "PRESCALER_256")) [.return_ (some (.intLit (0)))] [], .assign (.fieldAccess (.identifier "spi") "prescaler") (.identifier "psc"), .return_ (some (.intLit (0)))] }, { name := "spi_get_prescaler_div", params := [], ret := (some (.u32)), body := [] }, { name := "spi_get_sck_freq", params := [], ret := (some (.u32)), body := [.return_ (some (.binop "/" (.identifier "CLK_FREQ") (.call "spi_get_prescaler_div" [])))] }, { name := "spi_set_data_width", params := [("width", (.u8))], ret := (some (.bool)), body := [.ifThenElse (.binop "or" (.binop "==" (.identifier "width") (.intLit (0))) (.binop ">" (.identifier "width") (.identifier "MAX_DATA_WIDTH"))) [.return_ (some (.intLit (0)))] [], .assign (.fieldAccess (.identifier "spi") "data_width") (.identifier "width"), .return_ (some (.intLit (0)))] }, { name := "spi_is_busy", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "spi") "busy"))] }, { name := "spi_transfer", params := [("data", (.u32))], ret := (some (.bool)), body := [.ifThenElse (.fieldAccess (.identifier "spi") "busy") [.return_ (some (.intLit (0)))] [], .assign (.fieldAccess (.identifier "spi") "tx_data") (.identifier "data"), .assign (.fieldAccess (.identifier "spi") "rx_data") (.intLit (0)), .assign (.fieldAccess (.identifier "spi") "bit_count") (.intLit (0)), .assign (.fieldAccess (.identifier "spi") "bit_counter") (.intLit (0)), .assign (.fieldAccess (.identifier "spi") "state") (.identifier "SPI_CS_ASSERT"), .assign (.fieldAccess (.identifier "spi") "busy") (.intLit (0)), .return_ (some (.intLit (0)))] }, { name := "spi_read_rx", params := [], ret := (some (.u32)), body := [.return_ (some (.binop "&" (.fieldAccess (.identifier "spi") "rx_data") (.binop "-" (.binop "<<" (.intLit (1)) (.fieldAccess (.identifier "spi") "data_width")) (.intLit (1)))))] }, { name := "spi_get_cs", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "spi") "cs_asserted"))] }, { name := "spi_get_sck", params := [], ret := (some (.bool)), body := [] }, { name := "spi_get_mosi", params := [], ret := (some (.bool)), body := [.ifThenElse (.binop "or" (.unop "!" (.fieldAccess (.identifier "spi") "busy")) (.binop "!=" (.fieldAccess (.identifier "spi") "state") (.identifier "SPI_TRANSFER"))) [.return_ (some (.intLit (0)))] [], .return_ (some (.binop "==" (.binop "&" (.binop ">>" (.fieldAccess (.identifier "spi") "tx_data") (.binop "-" (.binop "-" (.fieldAccess (.identifier "spi") "data_width") (.fieldAccess (.identifier "spi") "bit_count")) (.intLit (1)))) (.intLit (1))) (.intLit (1))))] }, { name := "spi_tick", params := [], ret := (some (.struct "void")), body := [] }, { name := "spi_transfer_bit", params := [], ret := (some (.struct "void")), body := [.varDecl "prescaler_div" (.u32) (some (.call "spi_get_prescaler_div" [])), .assign (.fieldAccess (.identifier "spi") "bit_counter") (.binop "+" (.fieldAccess (.identifier "spi") "bit_counter") (.intLit (1)))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "spi_mode_0_configuration", params := [], ret := none, body := [] }, { name := "spi_prescaler_16_default", params := [], ret := none, body := [] }, { name := "spi_set_prescaler_valid", params := [], ret := none, body := [] }, { name := "spi_set_prescaler_invalid", params := [], ret := none, body := [] }, { name := "spi_prescaler_div_16", params := [], ret := none, body := [] }, { name := "spi_sck_freq_at_50MHz", params := [], ret := none, body := [] }, { name := "spi_set_data_width_8", params := [], ret := none, body := [] }, { name := "spi_set_data_width_32", params := [], ret := none, body := [] }, { name := "spi_set_data_width_invalid", params := [], ret := none, body := [] }, { name := "spi_initially_not_busy", params := [], ret := none, body := [] }, { name := "spi_transfer_when_ready", params := [], ret := none, body := [] }, { name := "spi_transfer_when_busy", params := [], ret := none, body := [] }, { name := "spi_cs_idle_high", params := [], ret := none, body := [] }, { name := "spi_sck_idle_low", params := [], ret := none, body := [] }, { name := "spi_max_data_width_32", params := [], ret := none, body := [] }, { name := "spi_prescaler_range", params := [], ret := none, body := [] }, { name := "spi_cs_delays_defined", params := [], ret := none, body := [] }],
  benches := [{ name := "spi_transfer_latency", params := [], ret := none, body := [] }, { name := "spi_sck_max_frequency", params := [], ret := none, body := [] }, { name := "spi_cs_assertion_time", params := [], ret := none, body := [] }, { name := "spi_prescaler_change_latency", params := [], ret := none, body := [] }]
}

def fpga_top_level_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("ops", ("base::ops", "ops")), ("registers", ("isa::registers", "registers"))],
  hostOnly := [],
  reachable := []
}

def fpga_top_level_module : Module := {
  name := "fpga_top_level",
  imports := [{ path := "base::types", items := ["types"] }, { path := "base::ops", items := ["ops"] }, { path := "isa::registers", items := ["registers"] }],
  globals := [.constDecl "CLK_FREQ_HZ" (.u32) (some (.intLit (0))), .constDecl "SYSTICK_HZ" (.u32) (some (.intLit (1000))), .constDecl "NUM_MAC_UNITS" (.struct "usize") (some (.intLit (8))), .constDecl "DATA_WIDTH" (.struct "usize") (some (.intLit (32))), .constDecl "CMD_NOP" (.u8) (some (.intLit (0))), .constDecl "CMD_MAC_MULT" (.u8) (some (.intLit (1))), .constDecl "CMD_MAC_DOT" (.u8) (some (.intLit (2))), .constDecl "CMD_UART_SEND" (.u8) (some (.intLit (3))), .constDecl "CMD_RESET" (.u8) (some (.intLit (0))), .constDecl "system_state" (.struct "SystemState") (some (.structLit "SystemState" [("mac_ready", (.intLit (0))), ("uart_ready", (.intLit (0))), ("processing", (.intLit (0))), ("error", (.intLit (0)))])), .constDecl "mac_result" (.i32) (some (.intLit (0))), .constDecl "uart_tx_data" (.u8) (some (.intLit (0)))],
  functions := [{ name := "system_init", params := [], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "system_state") "mac_ready") (.intLit (0)), .assign (.fieldAccess (.identifier "system_state") "uart_ready") (.intLit (0)), .assign (.fieldAccess (.identifier "system_state") "processing") (.intLit (0)), .assign (.fieldAccess (.identifier "system_state") "error") (.intLit (0))] }, { name := "system_ready", params := [], ret := (some (.bool)), body := [.return_ (some (.binop "and" (.fieldAccess (.identifier "system_state") "mac_ready") (.fieldAccess (.identifier "system_state") "uart_ready")))] }, { name := "system_busy", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "system_state") "processing"))] }, { name := "system_error", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "system_state") "error"))] }, { name := "system_reset", params := [], ret := (some (.struct "void")), body := [.bareCall (.call "system_init" []), .assign (.identifier "mac_result") (.intLit (0)), .assign (.identifier "uart_tx_data") (.intLit (0))] }, { name := "set_mac_result", params := [("value", (.i32))], ret := (some (.struct "void")), body := [.assign (.identifier "mac_result") (.identifier "value"), .assign (.fieldAccess (.identifier "system_state") "processing") (.intLit (0))] }, { name := "get_mac_result", params := [], ret := (some (.i32)), body := [.return_ (some (.identifier "mac_result"))] }, { name := "set_uart_data", params := [("data", (.u8))], ret := (some (.struct "void")), body := [.assign (.identifier "uart_tx_data") (.identifier "data")] }, { name := "get_uart_data", params := [], ret := (some (.u8)), body := [.return_ (some (.identifier "uart_tx_data"))] }, { name := "start_processing", params := [], ret := (some (.struct "void")), body := [.ifThenElse (.call "system_ready" []) [.assign (.fieldAccess (.identifier "system_state") "processing") (.intLit (0))] []] }, { name := "stop_processing", params := [], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "system_state") "processing") (.intLit (0))] }, { name := "set_error", params := [], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "system_state") "error") (.intLit (0)), .assign (.fieldAccess (.identifier "system_state") "processing") (.intLit (0))] }, { name := "clear_error", params := [], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "system_state") "error") (.intLit (0))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "system_initially_ready", params := [], ret := none, body := [] }, { name := "system_initially_not_busy", params := [], ret := none, body := [] }, { name := "system_initially_no_error", params := [], ret := none, body := [] }, { name := "system_reset_clears_state", params := [], ret := none, body := [] }, { name := "start_processing_sets_busy", params := [], ret := none, body := [] }, { name := "stop_processing_clears_busy", params := [], ret := none, body := [] }, { name := "set_error_clears_busy", params := [], ret := none, body := [] }, { name := "get_mac_result_after_set", params := [], ret := none, body := [] }, { name := "get_uart_data_after_set", params := [], ret := none, body := [] }, { name := "mac_result_clears_processing", params := [], ret := none, body := [] }, { name := "constants_clk_freq", params := [], ret := none, body := [] }, { name := "command_constants", params := [], ret := none, body := [] }, { name := "system_reset_clears_mac_result", params := [], ret := none, body := [] }, { name := "system_reset_clears_uart_data", params := [], ret := none, body := [] }, { name := "clear_error_does_not_affect_ready", params := [], ret := none, body := [] }, { name := "start_processing_requires_ready", params := [], ret := none, body := [] }, { name := "set_mac_result_negative", params := [], ret := none, body := [] }, { name := "set_uart_data_boundary", params := [], ret := none, body := [] }],
  benches := [{ name := "system_ready_latency", params := [], ret := none, body := [] }, { name := "system_reset_latency", params := [], ret := none, body := [] }]
}

def fpga_uart_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("registers", ("isa::registers", "registers")), ("ops", ("base::ops", "ops"))],
  hostOnly := [],
  reachable := []
}

def fpga_uart_module : Module := {
  name := "fpga_uart",
  imports := [{ path := "base::types", items := ["types"] }, { path := "base::ops", items := ["ops"] }, { path := "isa::registers", items := ["registers"] }],
  globals := [.constDecl "UART_CLOCK_HZ" (.u32) (some (.intLit (0))), .constDecl "UART_BAUD_RATE" (.u32) (some (.intLit (115200))), .constDecl "UART_BIT_PERIOD" (.u32) (some (.identifier "UART_CLOCK_HZ")), .constDecl "UART_WIDTH" (.struct "usize") (some (.intLit (8))), .constDecl "UART_FIFO_DEPTH" (.struct "usize") (some (.intLit (16))), .constDecl "STATUS_IDLE" (.u8) (some (.intLit (0))), .constDecl "STATUS_TX_BUSY" (.u8) (some (.intLit (1))), .constDecl "STATUS_RX_BUSY" (.u8) (some (.intLit (2))), .constDecl "STATUS_ERROR" (.u8) (some (.intLit (3))), .constDecl "uart_state" (.struct "UARTState") (some (.structLit "UARTState" [("tx_data", (.intLit (0))), ("tx_valid", (.intLit (0))), ("tx_ready", (.intLit (0))), ("rx_data", (.intLit (0))), ("rx_valid", (.intLit (0))), ("rx_error", (.intLit (0))), ("bit_counter", (.intLit (0))), ("status", (.identifier "STATUS_IDLE"))])), .constDecl "uart_config" (.struct "UARTConfig") (some (.structLit "UARTConfig" [("baud_divisor", (.binop "/" (.identifier "UART_CLOCK_HZ") (.binop "*" (.identifier "UART_BAUD_RATE") (.intLit (16))))), ("parity_enable", (.intLit (0))), ("stop_bits", (.intLit (1))), ("fifo_enable", (.intLit (0)))]))],
  functions := [{ name := "uart_tx_ready", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "uart_state") "tx_ready"))] }, { name := "uart_tx_send", params := [("data", (.u8))], ret := (some (.bool)), body := [.ifThenElse (.unop "!" (.fieldAccess (.identifier "uart_state") "tx_ready")) [.return_ (some (.intLit (0)))] [], .assign (.fieldAccess (.identifier "uart_state") "tx_data") (.identifier "data"), .assign (.fieldAccess (.identifier "uart_state") "tx_valid") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "tx_ready") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "status") (.identifier "STATUS_TX_BUSY"), .return_ (some (.intLit (0)))] }, { name := "uart_rx_ready", params := [], ret := (some (.bool)), body := [.return_ (some (.fieldAccess (.identifier "uart_state") "rx_valid"))] }, { name := "uart_rx_read", params := [], ret := (some (.u8)), body := [.assign (.fieldAccess (.identifier "uart_state") "rx_valid") (.intLit (0)), .return_ (some (.fieldAccess (.identifier "uart_state") "rx_data"))] }, { name := "uart_status", params := [], ret := (some (.u8)), body := [.return_ (some (.fieldAccess (.identifier "uart_state") "status"))] }, { name := "uart_reset", params := [], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "uart_state") "tx_data") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "tx_valid") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "tx_ready") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "rx_data") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "rx_valid") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "rx_error") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "bit_counter") (.intLit (0)), .assign (.fieldAccess (.identifier "uart_state") "status") (.identifier "STATUS_IDLE")] }, { name := "uart_configure", params := [("baud_divisor", (.u32)), ("parity_enable", (.bool)), ("stop_bits", (.u8)), ("fifo_enable", (.bool))], ret := (some (.struct "void")), body := [.assign (.fieldAccess (.identifier "uart_config") "baud_divisor") (.identifier "baud_divisor"), .assign (.fieldAccess (.identifier "uart_config") "parity_enable") (.identifier "parity_enable"), .assign (.fieldAccess (.identifier "uart_config") "stop_bits") (.identifier "stop_bits"), .assign (.fieldAccess (.identifier "uart_config") "fifo_enable") (.identifier "fifo_enable")] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "uart_initially_idle", params := [], ret := none, body := [] }, { name := "uart_tx_ready_initially", params := [], ret := none, body := [] }, { name := "uart_rx_not_valid_initially", params := [], ret := none, body := [] }, { name := "uart_tx_send_returns_true_when_ready", params := [], ret := none, body := [] }, { name := "uart_tx_send_returns_false_when_busy", params := [], ret := none, body := [] }, { name := "uart_reset_clears_status", params := [], ret := none, body := [] }, { name := "uart_reset_restores_tx_ready", params := [], ret := none, body := [] }, { name := "uart_configure_changes_baud_divisor", params := [], ret := none, body := [] }, { name := "uart_configure_parity_enable", params := [], ret := none, body := [] }, { name := "uart_bit_period_calc", params := [], ret := none, body := [] }, { name := "uart_constants", params := [], ret := none, body := [] }, { name := "uart_tx_send_updates_state", params := [], ret := none, body := [] }, { name := "uart_rx_read_clears_valid", params := [], ret := none, body := [] }, { name := "uart_reset_clears_rx_error", params := [], ret := none, body := [] }, { name := "uart_reset_clears_bit_counter", params := [], ret := none, body := [] }],
  benches := [{ name := "uart_tx_ready_latency", params := [], ret := none, body := [] }, { name := "uart_rx_ready_latency", params := [], ret := none, body := [] }, { name := "uart_reset_latency", params := [], ret := none, body := [] }]
}

def fpga_verification_build_verify_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["coverage_percent"]
}

def fpga_verification_build_verify_module : Module := {
  name := "fpga_verification_build_verify",
  imports := [],
  globals := [.constDecl "TOTAL_FPGA_MODULES" (.u32) (some (.intLit (33))), .constDecl "TOTAL_TESTBENCHES" (.u32) (some (.intLit (30))), .constDecl "TOTAL_BOARD_CONFIGS" (.u32) (some (.intLit (3))), .constDecl "TOTAL_SPECS" (.u32) (some (.intLit (66))), .constDecl "NUM_BACKENDS" (.u32) (some (.intLit (4))), .constDecl "VERILOG_FILES" (.u32) (some (.intLit (66)))],
  functions := [{ name := "check_build_clean", params := [("result", (.struct "BuildResult"))], ret := (some (.bool)), body := [.return_ (some (.binop "==" (.fieldAccess (.identifier "result") "failures") (.intLit (0))))] }, { name := "coverage_percent", params := [("ok", (.u32)), ("total", (.u32))], ret := (some (.u32)), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "test_module_count", params := [], ret := none, body := [] }, { name := "test_testbench_count", params := [], ret := none, body := [] }, { name := "test_board_count", params := [], ret := none, body := [] }, { name := "test_total_specs", params := [], ret := none, body := [] }, { name := "test_backend_count", params := [], ret := none, body := [] }, { name := "test_verilog_file_count", params := [], ret := none, body := [] }, { name := "test_coverage_100", params := [], ret := none, body := [.varDecl "cov" (.u32) (some (.call "coverage_percent" [(.intLit (66)), (.intLit (66))]))] }, { name := "test_coverage_0", params := [], ret := none, body := [.varDecl "cov" (.u32) (some (.call "coverage_percent" [(.intLit (0)), (.intLit (66))]))] }, { name := "test_coverage_50", params := [], ret := none, body := [.varDecl "cov" (.u32) (some (.call "coverage_percent" [(.intLit (23)), (.intLit (46))]))] }],
  benches := []
}

def github_auth_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def github_auth_module : Module := {
  name := "github_auth",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def github_comments_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def github_comments_module : Module := {
  name := "github_comments",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def github_issues_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def github_issues_module : Module := {
  name := "github_issues",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def github_prs_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def github_prs_module : Module := {
  name := "github_prs",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def github_tests_e2e_full_flow_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def github_tests_e2e_full_flow_module : Module := {
  name := "github_tests_e2e_full_flow",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := [{ name := "e2e_full_flow_bench", params := [], ret := none, body := [] }]
}

def graph_knowledge_graph_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("hybrid_arithmetic", ("ternary::hybrid_arithmetic", "hybrid_arithmetic")), ("packed_trit", ("ternary::packed_trit", "packed_trit")), ("types", ("base::types", "types")), ("gf16", ("numeric::gf16", "gf16"))],
  hostOnly := [],
  reachable := []
}

def graph_knowledge_graph_module : Module := {
  name := "graph_knowledge_graph",
  imports := [{ path := "base::types", items := ["types"] }, { path := "ternary::packed_trit", items := ["packed_trit"] }, { path := "numeric::gf16", items := ["gf16"] }, { path := "ternary::hybrid_arithmetic", items := ["hybrid_arithmetic"] }],
  globals := [.constDecl "FILE_MAGIC" (.array 4 (.u8)) (some (.identifier "[T,R,K,G]")), .constDecl "FILE_VERSION" (.u32) (some (.intLit (1))), .constDecl "VECTOR_DIM" (.u16) (some (.intLit (500))), .constDecl "MAX_ENTITIES" (.u16) (some (.intLit (100))), .constDecl "MAX_TRIPLES" (.u16) (some (.intLit (200))), .constDecl "SIMILARITY_THRESHOLD" (.struct "f64") (some (.intLit (0)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "entity_init_creates_valid_entity", params := [], ret := none, body := [] }, { name := "hash_string_consistent_for_same_input", params := [], ret := none, body := [] }, { name := "hash_string_different_for_different_input", params := [], ret := none, body := [] }, { name := "graph_init_creates_empty_graph", params := [], ret := none, body := [] }, { name := "add_triple_creates_entities_and_relations", params := [], ret := none, body := [] }, { name := "query_object_finds_correct_entity", params := [], ret := none, body := [] }, { name := "query_subject_finds_correct_entity", params := [], ret := none, body := [] }, { name := "find_similar_returns_top_results", params := [], ret := none, body := [] }, { name := "find_entity_returns_entity_when_exists", params := [], ret := none, body := [] }, { name := "find_entity_returns_null_when_not_exists", params := [], ret := none, body := [] }, { name := "stats_returns_correct_counts", params := [], ret := none, body := [] }],
  benches := [{ name := "entity_init_latency", params := [], ret := none, body := [] }, { name := "graph_add_triple_latency", params := [], ret := none, body := [] }, { name := "query_object_latency", params := [], ret := none, body := [] }, { name := "query_subject_latency", params := [], ret := none, body := [] }, { name := "find_similar_latency", params := [], ret := none, body := [] }, { name := "save_latency_100_entities", params := [], ret := none, body := [] }, { name := "load_latency_100_entities", params := [], ret := none, body := [] }, { name := "stats_latency", params := [], ret := none, body := [] }]
}

def hslm_forward_pass_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("packed_trit", ("ternary::packed_trit", "packed_trit")), ("gf16", ("numeric::gf16", "gf16")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def hslm_forward_pass_module : Module := {
  name := "hslm_forward_pass",
  imports := [{ path := "base::types", items := ["types"] }, { path := "numeric::gf16", items := ["gf16"] }, { path := "ternary::packed_trit", items := ["packed_trit"] }],
  globals := [.constDecl "HEAD_COUNT" (.u8) (some (.intLit (3))), .constDecl "FORWARD_VERSION" (.u32) (some (.intLit (2))), .constDecl "ROLE_DIM" (.u16) (some (.intLit (500))), .constDecl "CONTEXT_WINDOW" (.u8) (some (.intLit (8))), .constDecl "HEBBIAN_CHARS" (.struct "usize") (some (.intLit (95))), .constDecl "HEBBIAN_OFFSET" (.struct "usize") (some (.intLit (32))), .constDecl "MAX_REFINE_PASSES" (.u8) (some (.intLit (3)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "single_head_attention_returns_role", params := [], ret := none, body := [] }, { name := "summarize_context_creates_valid_hv", params := [], ret := none, body := [] }, { name := "direct_decode_returns_chars", params := [], ret := none, body := [] }, { name := "hebbian_counts_correct_for_corpus", params := [], ret := none, body := [] }, { name := "hebbian_lookup_returns_valid_hv", params := [], ret := none, body := [] }],
  benches := [{ name := "single_head_attention_latency", params := [], ret := none, body := [] }, { name := "multi_head_attention_latency", params := [], ret := none, body := [] }, { name := "forward_pass_latency", params := [], ret := none, body := [] }, { name := "forward_pass_multi_head_latency", params := [], ret := none, body := [] }, { name := "summarize_context_latency", params := [], ret := none, body := [] }, { name := "direct_decode_latency", params := [], ret := none, body := [] }, { name := "hebbian_counts_build_latency", params := [], ret := none, body := [] }, { name := "hebbian_lookup_latency", params := [], ret := none, body := [] }]
}

def igla_race_adder_tree_env : Env := {
  structs := [("Vec8", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := ["assert"]
}

def igla_race_adder_tree_module : Module := {
  name := "igla_race_adder_tree",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [],
  functions := [{ name := "adder_tree_8", params := [("v", (.struct "Vec8"))], ret := (some (.i32)), body := [.varDecl "s1_0" (.u32) (some (.binop "+" (.fieldAccess (.identifier "v") "v0") (.fieldAccess (.identifier "v") "v1"))), .varDecl "s1_1" (.u32) (some (.binop "+" (.fieldAccess (.identifier "v") "v2") (.fieldAccess (.identifier "v") "v3"))), .varDecl "s1_2" (.u32) (some (.binop "+" (.fieldAccess (.identifier "v") "v4") (.fieldAccess (.identifier "v") "v5"))), .varDecl "s1_3" (.u32) (some (.binop "+" (.fieldAccess (.identifier "v") "v6") (.fieldAccess (.identifier "v") "v7"))), .varDecl "s2_0" (.u32) (some (.binop "+" (.identifier "s1_0") (.identifier "s1_1"))), .varDecl "s2_1" (.u32) (some (.binop "+" (.identifier "s1_2") (.identifier "s1_3"))), .return_ (some (.binop "+" (.identifier "s2_0") (.identifier "s2_1")))] }, { name := "adder_tree_4", params := [("v0", (.i32)), ("v1", (.i32)), ("v2", (.i32)), ("v3", (.i32))], ret := (some (.i32)), body := [.varDecl "s1_0" (.u32) (some (.binop "+" (.identifier "v0") (.identifier "v1"))), .varDecl "s1_1" (.u32) (some (.binop "+" (.identifier "v2") (.identifier "v3"))), .return_ (some (.binop "+" (.identifier "s1_0") (.identifier "s1_1")))] }],
  tests := [{ name := "adder_tree_8_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_ones", params := [], ret := none, body := [] }, { name := "adder_tree_8_mixed", params := [], ret := none, body := [] }, { name := "adder_tree_4_positive", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_nonzero", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_large", params := [], ret := none, body := [] }, { name := "adder_tree_8_mixed_negative", params := [], ret := none, body := [] }, { name := "adder_tree_4_large_values", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_same", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_large", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_negative", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large", params := [], ret := none, body := [] }, { name := "adder_tree_2_basic", params := [], ret := none, body := [] }, { name := "adder_tree_8_zero", params := [], ret := none, body := [] }, { name := "adder_tree_4_identity_nonzero", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_equal", params := [], ret := none, body := [] }, { name := "adder_tree_8_large_mixed", params := [], ret := none, body := [] }, { name := "adder_tree_4_small_values", params := [], ret := none, body := [] }, { name := "adder_tree_8_extreme_values", params := [], ret := none, body := [] }, { name := "adder_tree_4_antisymmetric", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_2_negative_result", params := [], ret := none, body := [] }, { name := "adder_tree_8_zero_vector", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_values", params := [], ret := none, body := [] }, { name := "adder_tree_2_max_i32", params := [], ret := none, body := [] }, { name := "adder_tree_8_alternating_signs", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_values", params := [], ret := none, body := [] }, { name := "adder_tree_2_equal_values", params := [], ret := none, body := [] }, { name := "adder_tree_2_equal_negative", params := [], ret := none, body := [] }, { name := "adder_tree_8_extreme_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_4_i32_max_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_i32_min", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_i32_min_underflow", params := [], ret := none, body := [] }, { name := "adder_tree_4_i32_max_double_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_4_i32_boundary_oscillation", params := [], ret := none, body := [] }, { name := "adder_tree_8_commutativity_vs_naive", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_i32_max_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_2_both_i32_min_underflow", params := [], ret := none, body := [] }, { name := "adder_tree_2_max_min", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_zeros", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_i32_max", params := [], ret := none, body := [] }, { name := "adder_tree_4_i32_min_double_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_8_alternating_max_min", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_i32_min", params := [], ret := none, body := [] }, { name := "adder_tree_2_positive_negative_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_equal_max", params := [], ret := none, body := [] }, { name := "adder_tree_8_two_nonzero_rest_zero", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_sum", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_max_i8", params := [], ret := none, body := [] }, { name := "adder_tree_8_min_i32_values", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_i32_values", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_negative_i8", params := [], ret := none, body := [] }, { name := "adder_tree_4_identity_nonzero_second", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_zero", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative", params := [], ret := none, body := [] }, { name := "adder_tree_2_max_i32", params := [], ret := none, body := [] }, { name := "adder_tree_8_mixed_signs", params := [], ret := none, body := [] }, { name := "adder_tree_2_min_i32", params := [], ret := none, body := [] }, { name := "adder_tree_4_symmetric_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_2_commutative", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_nonzero_v4_middle", params := [], ret := none, body := [] }, { name := "adder_tree_2_both_i32_max_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_2_neg_max_plus_one", params := [], ret := none, body := [] }, { name := "adder_tree_4_power_of_two_sum", params := [], ret := none, body := [] }, { name := "adder_tree_2_large_neg_plus_pos", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_zeros", params := [], ret := none, body := [] }, { name := "adder_tree_8_ascending_values", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_nonzero_third", params := [], ret := none, body := [] }, { name := "adder_tree_2_near_max_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_8_power_of_two_pattern", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_pair_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_2_symmetric_bounds", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_signs", params := [], ret := none, body := [] }, { name := "adder_tree_8_alternating_cancel", params := [], ret := none, body := [] }, { name := "adder_tree_4_three_zeros_one_value", params := [], ret := none, body := [] }, { name := "adder_tree_2_i32_min_max", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_zeros", params := [], ret := none, body := [] }, { name := "adder_tree_2_zero_operands", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_signs", params := [], ret := none, body := [] }, { name := "adder_tree_2_large_positive_negative", params := [], ret := none, body := [] }, { name := "adder_tree_8_descending_values", params := [], ret := none, body := [] }, { name := "adder_tree_4_boundary_int16_max", params := [], ret := none, body := [] }, { name := "adder_tree_8_two_element_swap", params := [], ret := none, body := [] }, { name := "adder_tree_4_reorder_inputs", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_positive_sum_positive", params := [], ret := none, body := [] }, { name := "adder_tree_2_neg_zero_identity", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_inputs", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_8_commutative_permutation", params := [], ret := none, body := [] }, { name := "adder_tree_4_identity_element_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_nonzero_identity", params := [], ret := none, body := [] }, { name := "adder_tree_4_reorder_three", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_sum_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones_eight", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_negatives", params := [], ret := none, body := [] }, { name := "adder_tree_4_two_positive_two_negative", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_nonzero_identity", params := [], ret := none, body := [] }, { name := "adder_tree_8_zero_vector_zero", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_ones_sum", params := [], ret := none, body := [] }, { name := "adder_tree_4_commutative_swap_pairs", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_inputs", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_all", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_negative", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_zero", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_positive", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_signs", params := [], ret := none, body := [] }, { name := "adder_tree_8_zero_identity", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_negative_identity", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_same_values", params := [], ret := none, body := [] }, { name := "adder_tree_8_single_nonzero_first", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_sum", params := [], ret := none, body := [] }, { name := "adder_tree_8_mixed_signs_sum", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_plus_any_identity", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_positive_sum", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_twos", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_plus_one_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_plus_any_identity", params := [], ret := none, body := [] }, { name := "adder_tree_4_negative_plus_positive", params := [], ret := none, body := [] }, { name := "adder_tree_2_negation_identity", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_negative_same", params := [], ret := none, body := [] }, { name := "adder_tree_2_large_positive_overflow", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_input_zero_output", params := [], ret := none, body := [] }, { name := "adder_tree_2_identity_zero", params := [], ret := none, body := [] }, { name := "adder_tree_4_commutative_pair", params := [], ret := none, body := [] }, { name := "adder_tree_4_zero_input_zero_output_w294", params := [], ret := none, body := [] }, { name := "adder_tree_8_all_ones_sum_w294", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negatives_sum_w295", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_sum_zero_w295", params := [], ret := none, body := [] }, { name := "adder_tree_4_single_input_passthrough_w296", params := [], ret := none, body := [] }, { name := "adder_tree_4_large_numbers_sum_w296", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_same_positive_w297", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_same_negative_w297", params := [], ret := none, body := [] }, { name := "adder_tree_4_large_mixed_sum_w298", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_int32_sum_w298", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_positive_sum_w299", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w299", params := [], ret := none, body := [] }, { name := "adder_tree_4_zeros_zero_w300", params := [], ret := none, body := [] }, { name := "adder_tree_4_ones_four_w300", params := [], ret := none, body := [] }, { name := "adder_tree_4_large_negative_sum_w301", params := [], ret := none, body := [] }, { name := "adder_tree_4_alternating_zero_w301", params := [], ret := none, body := [] }, { name := "adder_tree_4_symmetric_zero_w302", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_positive_sum_w302", params := [], ret := none, body := [] }, { name := "adder_tree_4_min_int_sum_w303", params := [], ret := none, body := [] }, { name := "adder_tree_4_max_int_sum_w303", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_signs_sum_w304", params := [], ret := none, body := [] }, { name := "adder_tree_4_equal_pairs_sum_w304", params := [], ret := none, body := [] }, { name := "adder_tree_5_mixed_sum_w305", params := [], ret := none, body := [] }, { name := "adder_tree_5_uniform_sum_w305", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w306", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w306", params := [], ret := none, body := [] }, { name := "adder_tree_7_mixed_sum_w307", params := [], ret := none, body := [] }, { name := "adder_tree_7_zero_sum_w307", params := [], ret := none, body := [] }, { name := "adder_tree_8_mixed_sum_w308", params := [], ret := none, body := [] }, { name := "adder_tree_8_uniform_sum_w308", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w309", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w309", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w309", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w309", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w310", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w310", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w311", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w311", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w311", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w311", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w312", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w312", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w313", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w313", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w314", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w314", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w315", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w315", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w316", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w316", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w317", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w317", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w318", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w318", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w319", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w319", params := [], ret := none, body := [] }, { name := "adder_tree_4_all_negative_sum_w320", params := [], ret := none, body := [] }, { name := "adder_tree_4_mixed_large_sum_w320", params := [], ret := none, body := [] }, { name := "adder_tree_w321_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w321_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w322_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w322_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w323_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w323_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w324_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w324_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w325_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w325_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w326_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w326_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w327_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w327_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w328_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w328_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w329_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w329_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w330_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w330_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w331_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w331_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w332_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w332_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w333_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w333_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w334_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w334_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w335_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w335_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w336_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w336_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w337_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w337_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w338_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w338_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w339_batch_depth_invariant_1", params := [], ret := none, body := [.bareCall (.call "assert" [(.intLit (0))])] }, { name := "adder_tree_w339_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w343_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w343_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "adder_tree_w345_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "adder_tree_w345_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w346_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w346_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w347_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w347_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w348_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w348_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w349_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w349_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w350_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w350_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w351_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w351_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w352_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w352_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w353_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w353_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w354_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w354_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w355_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w355_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w356_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w356_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w357_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w357_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w358_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w358_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w359_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w359_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w360_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w360_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w361_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w361_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w362_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w362_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w363_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w363_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w364_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w364_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w365_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w365_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w366_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w366_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w367_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w367_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w368_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w368_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w369_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w369_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w370_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w370_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w371_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w371_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w372_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w372_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w373_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w373_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w374_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w374_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w375_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w375_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w376_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w376_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w377_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w377_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w378_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w378_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w379_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w379_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w380_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w380_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w381_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w381_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w382_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w382_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w383_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w383_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w384_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w384_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w385_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w385_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w386_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w386_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w387_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w387_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w388_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w388_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w389_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w389_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w390_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w390_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w391_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w391_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w392_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_adder_tree_w392_batch_depth_invariant_2", params := [], ret := none, body := [] }],
  benches := [{ name := "adder_tree_8_latency", params := [], ret := none, body := [] }, { name := "adder_tree_4_latency", params := [], ret := none, body := [] }, { name := "adder_tree_2_latency", params := [], ret := none, body := [] }]
}

def igla_race_systolic_ternary_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("systolic_ternary_pe_reg", "SystolicTernaryPE")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("ternary_mac", ("igla::race::ternary_mac", "ternary_mac"))],
  hostOnly := [],
  reachable := ["assert"]
}

def igla_race_systolic_ternary_module : Module := {
  name := "igla_race_systolic_ternary",
  imports := [{ path := "base::types", items := ["types"] }, { path := "igla::race::ternary_mac", items := ["ternary_mac"] }],
  globals := [],
  functions := [{ name := "systolic_ternary_pe", params := [("a_in", (.i8)), ("w", (.struct "TernaryWeight")), ("psum_in", (.i16))], ret := (some (.struct "(i8,i16)")), body := [.varDecl "prod" (.u32) (some (.call "ternary_mul" [(.identifier "a_in"), (.identifier "w")])), .varDecl "psum_out" (.u32) (some (.binop "+" (.identifier "psum_in") (.identifier "prod"))), .return_ (some (.arrayLit (.u32) [(.identifier "a_in"), (.identifier "psum_out")]))] }, { name := "systolic_ternary_pe_reg", params := [("clk", (.bool)), ("rst_n", (.bool)), ("pe", (.struct "SystolicTernaryPE")), ("a_in", (.i8)), ("w", (.struct "TernaryWeight")), ("psum_in", (.i16))], ret := (some (.struct "SystolicTernaryPE")), body := [.ifThenElse (.unop "!" (.identifier "rst_n")) [.return_ (some (.structLit "SystolicTernaryPE" [("a_reg", (.intLit (0))), ("psum_reg", (.intLit (0)))]))] [], .assign (.arrayLit (.u32) [(.identifier "_"), (.identifier "psum_out")]) (.call "systolic_ternary_pe" [(.identifier "a_in"), (.identifier "w"), (.identifier "psum_in")]), .return_ (some (.structLit "SystolicTernaryPE" [("a_reg", (.identifier "a_in")), ("psum_reg", (.identifier "psum_out"))]))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "systolic_pe_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_reset", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_update", params := [], ret := none, body := [] }, { name := "systolic_pe_negative_activation", params := [], ret := none, body := [] }, { name := "systolic_pe_large_psum", params := [], ret := none, body := [] }, { name := "systolic_pe_weight_zero_code", params := [], ret := none, body := [] }, { name := "systolic_pe_negative_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_zero_activation", params := [], ret := none, body := [] }, { name := "systolic_pe_ternary_minus_one", params := [], ret := none, body := [] }, { name := "systolic_pe_max_accumulate", params := [], ret := none, body := [] }, { name := "systolic_pe_zero_weight_update", params := [], ret := none, body := [] }, { name := "systolic_pe_min_accumulate", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_pe_negative_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_zero_size", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_hold", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_negative_weights", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_reset_then_update", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty_weights", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_activation", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_max_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements", params := [], ret := none, body := [] }, { name := "ternary_decode_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_mul_zero_result", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_max_saturation", params := [], ret := none, body := [] }, { name := "ternary_decode_neg_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_rst_n", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_mixed_weights", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation", params := [], ret := none, body := [] }, { name := "ternary_decode_pos_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_i8_min_sign_flip", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_three_mixed", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_rst_n_dominates_clk", params := [], ret := none, body := [] }, { name := "systolic_pe_i8_min_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_psum_i16_overflow_boundary", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_clk_low_ignores_input", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_reset_dominates_clk_low", params := [], ret := none, body := [] }, { name := "systolic_pe_i8_min_neg_weight_wrap", params := [], ret := none, body := [] }, { name := "systolic_pe_illegal_weight_code_3_absorption", params := [], ret := none, body := [] }, { name := "systolic_pe_illegal_weight_code_absorption", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation", params := [], ret := none, body := [] }, { name := "ternary_weight_decode_all_codes", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_psum_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_update_from_nonzero_state", params := [], ret := none, body := [] }, { name := "systolic_pe_i8_min_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_pe_reg_min_activation_neg_weight", params := [], ret := none, body := [] }, { name := "decode_weight_code_0_returns_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_hold_state_when_clk_low", params := [], ret := none, body := [] }, { name := "systolic_pe_illegal_weight_code_3_zero", params := [], ret := none, body := [] }, { name := "decode_weight_code_1_returns_pos_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_activation", params := [], ret := none, body := [] }, { name := "decode_weight_code_neg_one_returns_neg_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_min_activation", params := [], ret := none, body := [] }, { name := "decode_weight_code_3_returns_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation", params := [], ret := none, body := [] }, { name := "decode_weight_code_255_returns_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_psum_boundary", params := [], ret := none, body := [] }, { name := "decode_weight_code_0_returns_zero_explicit", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_psum_identity", params := [], ret := none, body := [] }, { name := "decode_weight_code_2_returns_neg_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_neg_activation_pos_weight", params := [], ret := none, body := [] }, { name := "decode_weight_code_1_returns_pos_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_neg_weight_subtracts", params := [], ret := none, body := [] }, { name := "decode_weight_all_codes_mapped", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_psum_i16_boundary_overflow", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_cascade_activation_passthrough", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_cascade_positive_twice", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_neg_activation_neg_weight", params := [], ret := none, body := [] }, { name := "decode_weight_code_0_returns_zero_implicit", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_i16_min_psum_overflow", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_max_psum_overflow", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_reset_clears", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_zero_size", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_identity", params := [], ret := none, body := [] }, { name := "decode_weight_code_3_aliased_to_zero", params := [], ret := none, body := [] }, { name := "ternary_decode_negative_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_activation_pos_weight", params := [], ret := none, body := [] }, { name := "ternary_decode_weight_code_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_clk_updates", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_preserves_psum", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_negative_psum", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_hold_no_clock", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_neg_activation_pos_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_reset_psum", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements_mixed", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_positive_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_negative_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_active_clock", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements_positive", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty_input", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_weight_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_weight_negates", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_weight_one", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty_returns_empty", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_preserves_psum", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_negative_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_reset_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements_positive", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_weight_zero_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_weight_minus_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_update_preserve_a_reg", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_zero_weight_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_min_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_positive_and_negative", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_reset_preserve", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_min_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_update_preserves_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_negative_activation", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_mixed_positive_negative", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_activation_passthrough", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_one_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_one_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_one_weight_negative", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_2x2_identity", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_zero_activation_update", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_three_elements_all_positive", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_minus_weight_update", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_all_zero_activations", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_plus_weight_accumulate", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_minus_weight_subtract", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_zero_weights_all_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_reg_zero_weight_nop", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_mid_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_mid_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_min_activation_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_three_elements_mixed", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_any_weight_nop", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_zero_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_max_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_min_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_plus_weight_nop", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_zero_weight_nop", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight_increment", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_minus_weight_decrement", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_single_element_plus", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_minus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_empty_zero", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_plus_weight", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_plus_weight_w294", params := [], ret := none, body := [] }, { name := "systolic_ternary_array_two_elements_w294", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_nop_w295", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_minus_weight_positive_w295", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_weight_nop_acc_w296", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight_w296", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_plus_weight_w297", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_minus_weight_w297", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_small_positive_activation_minus_weight_w298", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_small_negative_activation_plus_weight_w298", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight_w299", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_negative_activation_minus_weight_w299", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_acc_plus_weight_adds_w300", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_acc_minus_weight_negates_w300", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_zero_weight_nop_w301", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_acc_zero_activation_zero_w301", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_small_activation_plus_weight_adds_w302", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_small_activation_minus_weight_negates_w302", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_plus_weight_zero_w303", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_minus_weight_zero_w303", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_plus_weight_preserve_w304", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_zero_weight_preserve_w304", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_zero_activation_any_weight_preserve_w305", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_positive_activation_plus_weight_adds_w305", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w306", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w306", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_plus_weight_add_w307", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_minus_weight_negate_w307", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_zero_weight_nop_w308", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_plus_weight_add_w308", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_zero_weight_nop_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_large_activation_plus_weight_add_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w310", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w310", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w312", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w312", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w313", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w313", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w314", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w314", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w315", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w315", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w316", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w316", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w317", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w317", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w318", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w318", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w319", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w319", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_plus_activation_plus_weight_adds_w320", params := [], ret := none, body := [] }, { name := "systolic_ternary_pe_minus_activation_minus_weight_adds_w320", params := [], ret := none, body := [] }, { name := "systolic_ternary_w321_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w321_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w322_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w322_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w323_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w323_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w324_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w324_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w325_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w325_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w326_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w326_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w327_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w327_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w328_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w328_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w329_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w329_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w330_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w330_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w331_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w331_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w332_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w332_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w333_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w333_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w334_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w334_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w335_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w335_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w336_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w336_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w337_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w337_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w338_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w338_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w339_batch_depth_invariant_1", params := [], ret := none, body := [.bareCall (.call "assert" [(.intLit (0))])] }, { name := "systolic_ternary_w339_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w343_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w343_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "systolic_ternary_w345_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "systolic_ternary_w345_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w346_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w346_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w347_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w347_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w348_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w348_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w349_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w349_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w350_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w350_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w351_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w351_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w352_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w352_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w353_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w353_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w354_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w354_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w355_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w355_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w356_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w356_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w357_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w357_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w358_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w358_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w359_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w359_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w360_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w360_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w361_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w361_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w362_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w362_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w363_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w363_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w364_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w364_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w365_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w365_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w366_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w366_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w367_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w367_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w368_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w368_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w369_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w369_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w370_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w370_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w371_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w371_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w372_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w372_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w373_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w373_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w374_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w374_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w375_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w375_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w376_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w376_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w377_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w377_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w378_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w378_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w379_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w379_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w380_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w380_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w381_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w381_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w382_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w382_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w383_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w383_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w384_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w384_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w385_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w385_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w386_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w386_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w387_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w387_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w388_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w388_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w389_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w389_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w390_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w390_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w391_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w391_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w392_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_systolic_ternary_w392_batch_depth_invariant_2", params := [], ret := none, body := [] }],
  benches := [{ name := "systolic_pe_latency", params := [], ret := none, body := [] }, { name := "systolic_array_throughput", params := [], ret := none, body := [] }, { name := "systolic_pe_stress", params := [], ret := none, body := [] }]
}

def igla_race_ternary_inference_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("ternary_inference_2x2", "InferenceResult"), ("ternary_inference_identity", "InferenceResult"), ("ternary_inference_zero_weights", "InferenceResult"), ("load_ternary_weights", "TernaryModel")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("ternary_mac", ("igla::race::ternary_mac", "ternary_mac")), ("bram_weights", ("igla::race::bram_weights", "bram_weights")), ("systolic_ternary", ("igla::race::systolic_ternary", "systolic_ternary")), ("ternary_gemm", ("igla::race::ternary_gemm", "ternary_gemm"))],
  hostOnly := ["model_weight_count"],
  reachable := ["assert"]
}

def igla_race_ternary_inference_module : Module := {
  name := "igla_race_ternary_inference",
  imports := [{ path := "base::types", items := ["types"] }, { path := "igla::race::bram_weights", items := ["bram_weights"] }, { path := "igla::race::ternary_mac", items := ["ternary_mac"] }, { path := "igla::race::ternary_gemm", items := ["ternary_gemm"] }, { path := "igla::race::systolic_ternary", items := ["systolic_ternary"] }],
  globals := [],
  functions := [{ name := "load_ternary_weights", params := [("codes", (.struct "[]TernaryWeight"))], ret := (some (.struct "TernaryModel")), body := [.return_ (some (.structLit "TernaryModel" [("weights", (.identifier "codes"))]))] }, { name := "model_weight_count", params := [("model", (.struct "TernaryModel"))], ret := (some (.u32)), body := [.return_ (some (.call "model.weights.len" []))] }, { name := "ternary_inference_2x2", params := [("input", (.struct "InferenceInput")), ("model", (.struct "TernaryModel"))], ret := (some (.struct "InferenceResult")), body := [.varDecl "result" (.u32) (some (.call "ternary_gemm_2x2" [(.fieldAccess (.identifier "input") "activations"), (.fieldAccess (.identifier "model") "weights")])), .return_ (some (.structLit "InferenceResult" [("outputs", (.identifier "result"))]))] }, { name := "ternary_inference_identity", params := [("input", (.struct "InferenceInput"))], ret := (some (.struct "InferenceResult")), body := [.varDecl "identity_weights" (.u32) (some (.arrayLit (.u32) [])), .varDecl "model" (.u32) (some (.call "load_ternary_weights" [(.identifier "identity_weights")])), .return_ (some (.call "ternary_inference_2x2" [(.identifier "input"), (.identifier "model")]))] }, { name := "ternary_inference_zero_weights", params := [("input", (.struct "InferenceInput"))], ret := (some (.struct "InferenceResult")), body := [.varDecl "zero_weights" (.u32) (some (.arrayLit (.u32) [])), .varDecl "model" (.u32) (some (.call "load_ternary_weights" [(.identifier "zero_weights")])), .return_ (some (.call "ternary_inference_2x2" [(.identifier "input"), (.identifier "model")]))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "ternary_inference_identity_basic", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_basic", params := [], ret := none, body := [] }, { name := "ternary_inference_model_weight_count", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_plus_minus_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_mixed_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_negative_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_invalid_weight_all_zero", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_negative_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_model_weight_count_four", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_zero_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_model_weight_count_empty", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_element", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_negative_weight", params := [], ret := none, body := [] }, { name := "ternary_inference_model_weight_count_two", params := [], ret := none, body := [] }, { name := "ternary_inference_model_weight_count_len", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_plus_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_minus_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_mixed_weights_detailed", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_zero_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_zero_weights_all_zero", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_minus_weights_negate", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_mixed_weights_first_plus_second_minus", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_single_nonzero", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_all_positive", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_permute_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_negative_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_empty_model_weight_count", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_plus_weight_position_3", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_mixed_weights_first_row_plus_second_row_minus", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_plus_weights_sum", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_empty_activations", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_single_activation", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_minus_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_zero_weights_all_zeros", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_activations_sum", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_activations_sum", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_zero_activations_zero_output", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_activation_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_activation_minus_weight", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_large_activation", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_single_row_minus_weights", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_activations_identity_output_zero", params := [], ret := none, body := [] }, { name := "ternary_inference_mixed_weights_small_positive", params := [], ret := none, body := [] }, { name := "ternary_inference_all_minus_weights_negate", params := [], ret := none, body := [] }, { name := "ternary_inference_single_activation_identity", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_length_preserved_w294", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_first_row_identity_w294", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_first_element_w295", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_second_row_identity_w295", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_second_element_w296", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_plus_weights_sum_w296", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_third_element_w297", params := [], ret := none, body := [] }, { name := "ternary_inference_2x2_all_minus_weights_neg_w297", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_fourth_element_w298", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_all_elements_w298", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_first_element_w299", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_last_element_w299", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_second_element_w300", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_third_element_w300", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_zero_input_zero_output_w301", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_negative_input_passthrough_w301", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_input_passthrough_w302", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_max_positive_passthrough_w302", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_min_negative_passthrough_w303", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_second_positive_passthrough_w303", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_second_positive_passthrough_w304", params := [], ret := none, body := [] }, { name := "ternary_inference_minus_weights_negate_w304", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_activations_all_zero_w305", params := [], ret := none, body := [] }, { name := "ternary_inference_uniform_plus_weights_double_w305", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w306", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w306", params := [], ret := none, body := [] }, { name := "ternary_inference_all_plus_weights_double_w307", params := [], ret := none, body := [] }, { name := "ternary_inference_all_minus_weights_negate_w307", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_passthrough_w308", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_all_zero_w308", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_passthrough_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_all_zero_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w309", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w310", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w310", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w311", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w311", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w311", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w311", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w312", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w312", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w313", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w313", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w314", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w314", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w315", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w315", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w316", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w316", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w317", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w317", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w318", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w318", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w319", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w319", params := [], ret := none, body := [] }, { name := "ternary_inference_identity_mixed_activations_passthrough_w320", params := [], ret := none, body := [] }, { name := "ternary_inference_zero_weights_mixed_activations_zero_w320", params := [], ret := none, body := [] }, { name := "ternary_inference_w321_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w321_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w322_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w322_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w323_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w323_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w324_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w324_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w325_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w325_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w326_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w326_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w327_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w327_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w328_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w328_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w329_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w329_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w330_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w330_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w331_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w331_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w332_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w332_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w333_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w333_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w334_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w334_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w335_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w335_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w336_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w336_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w337_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w337_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w338_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w338_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w339_batch_depth_invariant_1", params := [], ret := none, body := [.bareCall (.call "assert" [(.intLit (0))])] }, { name := "ternary_inference_w339_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w343_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w343_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_inference_w345_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_inference_w345_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w346_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w346_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w347_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w347_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w348_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w348_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w349_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w349_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w350_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w350_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w351_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w351_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w352_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w352_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w353_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w353_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w354_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w354_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w355_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w355_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w356_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w356_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w357_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w357_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w358_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w358_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w359_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w359_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w360_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w360_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w361_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w361_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w362_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w362_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w363_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w363_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w364_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w364_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w365_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w365_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w366_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w366_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w367_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w367_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w368_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w368_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w369_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w369_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w370_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w370_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w371_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w371_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w372_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w372_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w373_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w373_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w374_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w374_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w375_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w375_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w376_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w376_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w377_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w377_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w378_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w378_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w379_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w379_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w380_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w380_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w381_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w381_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w382_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w382_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w383_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w383_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w384_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w384_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w385_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w385_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w386_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w386_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w387_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w387_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w388_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w388_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w389_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w389_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w390_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w390_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w391_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w391_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w392_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_inference_w392_batch_depth_invariant_2", params := [], ret := none, body := [] }],
  benches := []
}

def igla_race_ternary_mac_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := ["ternary_dot"],
  reachable := ["assert"]
}

def igla_race_ternary_mac_module : Module := {
  name := "igla_race_ternary_mac",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [],
  functions := [{ name := "ternary_decode", params := [("w", (.struct "TernaryWeight"))], ret := (some (.i8)), body := [.ifThenElse (.binop "==" (.fieldAccess (.identifier "w") "code") (.intLit (1))) [.return_ (some (.intLit (1)))] [], .ifThenElse (.binop "==" (.fieldAccess (.identifier "w") "code") (.intLit (2))) [.return_ (some (.unop "-" (.intLit (1))))] [], .return_ (some (.intLit (0)))] }, { name := "ternary_mul", params := [("a", (.i8)), ("w", (.struct "TernaryWeight"))], ret := (some (.i8)), body := [.varDecl "decoded" (.u32) (some (.call "ternary_decode" [(.identifier "w")])), .ifThenElse (.binop "==" (.identifier "decoded") (.intLit (0))) [.return_ (some (.intLit (0)))] [], .ifThenElse (.binop "==" (.identifier "decoded") (.intLit (1))) [.return_ (some (.identifier "a"))] [], .return_ (some (.unop "-" (.identifier "a")))] }, { name := "ternary_mac", params := [("acc", (.i32)), ("a", (.i8)), ("w", (.struct "TernaryWeight"))], ret := (some (.i32)), body := [.varDecl "prod" (.u32) (some (.call "ternary_mul" [(.identifier "a"), (.identifier "w")])), .return_ (some (.binop "+" (.identifier "acc") (.identifier "prod")))] }, { name := "ternary_dot", params := [("a", (.struct "[]i8")), ("w", (.struct "[]TernaryWeight")), ("idx", (.u32)), ("acc", (.i32))], ret := (some (.i32)), body := [.ifThenElse (.binop ">=" (.identifier "idx") (.call "a.len" [])) [.return_ (some (.identifier "acc"))] [], .ifThenElse (.binop ">=" (.identifier "idx") (.call "w.len" [])) [.return_ (some (.identifier "acc"))] [], .varDecl "prod" (.u32) (some (.call "ternary_mul" [(.index (.identifier "a") (.identifier "idx")), (.index (.identifier "w") (.identifier "idx"))])), .return_ (some (.call "ternary_dot" [(.identifier "a"), (.identifier "w"), (.binop "+" (.identifier "idx") (.intLit (1))), (.binop "+" (.identifier "acc") (.identifier "prod"))]))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "ternary_decode_zero", params := [], ret := none, body := [] }, { name := "ternary_decode_plus", params := [], ret := none, body := [] }, { name := "ternary_decode_minus", params := [], ret := none, body := [] }, { name := "ternary_mul_zero", params := [], ret := none, body := [] }, { name := "ternary_mul_plus", params := [], ret := none, body := [] }, { name := "ternary_mul_minus", params := [], ret := none, body := [] }, { name := "ternary_mul_negative_input", params := [], ret := none, body := [] }, { name := "ternary_mac_accumulates", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_basic", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_accumulates", params := [], ret := none, body := [] }, { name := "ternary_dot_all_zero", params := [], ret := none, body := [] }, { name := "ternary_dot_all_minus_one", params := [], ret := none, body := [] }, { name := "ternary_mac_large_negative", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element", params := [], ret := none, body := [] }, { name := "ternary_mac_max_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_zero_elements", params := [], ret := none, body := [] }, { name := "ternary_mac_min_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_mixed_length", params := [], ret := none, body := [] }, { name := "ternary_mac_max_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_weights", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_slices", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_decode_all_weights", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_positive_sum", params := [], ret := none, body := [] }, { name := "ternary_mac_max_positive", params := [], ret := none, body := [] }, { name := "ternary_dot_mixed_weights", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_vectors", params := [], ret := none, body := [] }, { name := "ternary_decode_invalid_far_code", params := [], ret := none, body := [] }, { name := "ternary_dot_offset_seed", params := [], ret := none, body := [] }, { name := "ternary_mul_i8_min_negate_overflow", params := [], ret := none, body := [] }, { name := "ternary_dot_unequal_lengths_short_w", params := [], ret := none, body := [] }, { name := "ternary_mac_i32_max_overflow", params := [], ret := none, body := [] }, { name := "ternary_dot_a_shorter_than_w", params := [], ret := none, body := [] }, { name := "ternary_mac_i32_min_underflow_wrap", params := [], ret := none, body := [] }, { name := "ternary_dot_both_empty_seed_idx", params := [], ret := none, body := [] }, { name := "ternary_mul_commutative_sign_flip", params := [], ret := none, body := [] }, { name := "ternary_dot_idx_beyond_both_lengths", params := [], ret := none, body := [] }, { name := "ternary_dot_longer_weight_vec", params := [], ret := none, body := [] }, { name := "ternary_dot_all_zeros", params := [], ret := none, body := [] }, { name := "ternary_decode_code_3_returns_zero", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_near_max_activation_max_wrap", params := [], ret := none, body := [] }, { name := "ternary_dot_idx_beyond_nonempty_arrays", params := [], ret := none, body := [] }, { name := "ternary_mul_i8_max_negate", params := [], ret := none, body := [] }, { name := "ternary_decode_code_2_returns_neg_one", params := [], ret := none, body := [] }, { name := "ternary_mac_i8_min_activation_pos_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_zero_weight_identity", params := [], ret := none, body := [] }, { name := "ternary_dot_both_empty_returns_acc", params := [], ret := none, body := [] }, { name := "ternary_mac_max_i8_activation", params := [], ret := none, body := [] }, { name := "ternary_mac_neg_weight_neg_result", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_nonzero_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_arrays_zero_acc", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_negative_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_two_elements_mixed", params := [], ret := none, body := [] }, { name := "ternary_mac_max_acc_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_three_elements_all_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_min_i8_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_three_elements_positive", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_identity", params := [], ret := none, body := [] }, { name := "ternary_mac_i32_max_minus_weight_avoids_wrap", params := [], ret := none, body := [] }, { name := "ternary_dot_unequal_lengths_nonzero_idx", params := [], ret := none, body := [] }, { name := "ternary_mac_neg_acc_minus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_arrays_zero", params := [], ret := none, body := [] }, { name := "ternary_mac_neg_activation_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_two_elements_mixed_weights", params := [], ret := none, body := [] }, { name := "ternary_dot_four_elements_all_positive", params := [], ret := none, body := [] }, { name := "ternary_mac_i32_min_acc_plus_min_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_unequal_lengths_truncates", params := [], ret := none, body := [] }, { name := "ternary_mac_max_activation_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_three_elements_acc_carry", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element", params := [], ret := none, body := [] }, { name := "ternary_mac_min_activation_pos_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_weights_longer_activations", params := [], ret := none, body := [] }, { name := "ternary_mac_boundary_i8_max_neg_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_preserves_acc", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_arrays", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_two_elements_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_two_elements_negative", params := [], ret := none, body := [] }, { name := "ternary_mul_large_activation_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_large_psum_saturation", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element_positive", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_only_identity", params := [], ret := none, body := [] }, { name := "ternary_mul_negative_activation", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_zero_activation", params := [], ret := none, body := [] }, { name := "ternary_decode_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_arrays", params := [], ret := none, body := [] }, { name := "ternary_mul_positive_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_positive_activation_negative_weight", params := [], ret := none, body := [] }, { name := "ternary_decode_zero_returns_zero", params := [], ret := none, body := [] }, { name := "ternary_mul_zero_weight_returns_zero", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_zero_weight", params := [], ret := none, body := [] }, { name := "ternary_decode_one_returns_one", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_weight_positive_activation", params := [], ret := none, body := [] }, { name := "ternary_decode_negative_returns_neg_one", params := [], ret := none, body := [] }, { name := "ternary_mul_negative_activation_positive_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_activation_negative_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_activation_positive_weight_zero_acc", params := [], ret := none, body := [] }, { name := "ternary_dot_negative_activation", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_zero_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_two_elements_positive", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_acc_positive_result", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_vectors_zero", params := [], ret := none, body := [] }, { name := "ternary_mac_large_acc_pos_weight_exact", params := [], ret := none, body := [] }, { name := "ternary_dot_three_elements_mixed_weights", params := [], ret := none, body := [] }, { name := "ternary_mul_zero_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_two_negative_activations", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_42_neg_activation", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element_zero_acc", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_zero_neg_activation_minus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_single_element_minus_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_100_zero_weight_nop", params := [], ret := none, body := [] }, { name := "ternary_mul_positive_activation_plus_weight", params := [], ret := none, body := [] }, { name := "ternary_mac_max_i32_acc_zero_weight_identity", params := [], ret := none, body := [] }, { name := "ternary_dot_three_elements_all_zero_weights", params := [], ret := none, body := [] }, { name := "ternary_mac_acc_minus_weight", params := [], ret := none, body := [] }, { name := "ternary_dot_empty_list", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_zero_weight_nop", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_doubles_acc", params := [], ret := none, body := [] }, { name := "ternary_dot_nonzero_idx_skips_first", params := [], ret := none, body := [] }, { name := "ternary_mac_minus_weight_negates_acc", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_preserves_acc", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_nop", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w294", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_preserves_w294", params := [], ret := none, body := [] }, { name := "ternary_mac_minus_weight_negates_w295", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_zero_w295", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w296", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_plus_weight_identity_w296", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_minus_weight_identity_w297", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_plus_weight_w297", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_minus_weight_neg_w298", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_acc_zero_activation_zero_w298", params := [], ret := none, body := [] }, { name := "ternary_mac_small_activation_plus_weight_w299", params := [], ret := none, body := [] }, { name := "ternary_mac_negative_activation_plus_weight_w299", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_any_weight_zero_w300", params := [], ret := none, body := [] }, { name := "ternary_mac_large_acc_small_change_w300", params := [], ret := none, body := [] }, { name := "ternary_mac_large_negative_activation_plus_weight_w301", params := [], ret := none, body := [] }, { name := "ternary_mac_large_negative_activation_minus_weight_w301", params := [], ret := none, body := [] }, { name := "ternary_mac_small_activation_zero_weight_preserve_w302", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_zero_weight_preserve_w302", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_plus_weight_preserve_w303", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_minus_weight_preserve_w303", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_minus_weight_preserve_w304", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w304", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_plus_weight_preserve_w305", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_activation_zero_weight_preserve_w305", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w306", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w306", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_plus_weight_add_w307", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_minus_weight_sub_w307", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_zero_weight_preserve_w308", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_plus_weight_add_w308", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_zero_weight_preserve_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_large_activation_plus_weight_add_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w309", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w310", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w310", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w311", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w311", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w311", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w312", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w312", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w313", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w313", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w314", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w314", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w315", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w315", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w316", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w316", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w317", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w317", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w318", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w318", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w319", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w319", params := [], ret := none, body := [] }, { name := "ternary_mac_zero_weight_nop_w320", params := [], ret := none, body := [] }, { name := "ternary_mac_plus_weight_adds_w320", params := [], ret := none, body := [] }, { name := "ternary_mac_w321_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w321_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w322_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w322_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w323_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w323_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w324_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w324_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w325_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w325_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w326_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w326_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w327_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w327_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w328_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w328_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w329_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w329_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w330_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w330_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w331_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w331_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w332_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w332_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w333_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w333_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w334_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w334_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w335_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w335_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w336_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w336_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w337_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w337_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w338_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w338_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w339_batch_depth_invariant_1", params := [], ret := none, body := [.bareCall (.call "assert" [(.intLit (0))])] }, { name := "ternary_mac_w339_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w343_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w343_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w344_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w344_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "ternary_mac_w345_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "ternary_mac_w345_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w346_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w346_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w347_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w347_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w348_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w348_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w349_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w349_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w350_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w350_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w351_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w351_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w352_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w352_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w353_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w353_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w354_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w354_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w355_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w355_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w356_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w356_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w357_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w357_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w358_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w358_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w359_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w359_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w360_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w360_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w361_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w361_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w362_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w362_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w363_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w363_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w364_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w364_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w365_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w365_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w366_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w366_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w367_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w367_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w368_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w368_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w369_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w369_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w370_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w370_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w371_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w371_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w372_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w372_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w373_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w373_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w374_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w374_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w375_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w375_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w376_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w376_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w377_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w377_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w378_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w378_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w379_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w379_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w380_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w380_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w381_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w381_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w382_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w382_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w383_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w383_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w384_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w384_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w385_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w385_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w386_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w386_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w387_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w387_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w388_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w388_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w389_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w389_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w390_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w390_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w391_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w391_batch_depth_invariant_2", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w392_batch_depth_invariant_1", params := [], ret := none, body := [] }, { name := "igla_race_ternary_mac_w392_batch_depth_invariant_2", params := [], ret := none, body := [] }],
  benches := [{ name := "mac_accumulator_nonneg", params := [], ret := none, body := [] }, { name := "ternary_mac_latency", params := [], ret := none, body := [] }, { name := "ternary_dot_latency", params := [], ret := none, body := [] }]
}

def igla_w521_2d_aos_param_soundness_env : Env := {
  structs := [("Pt", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["assert_eq", "sum_param"]
}

def igla_w521_2d_aos_param_soundness_module : Module := {
  name := "igla_w521_2d_aos_param_soundness",
  imports := [],
  globals := [.constDecl "g_pts" (.array 2 (.array 3 (.struct "Pt"))) (some (.arrayLit (.array 3 (.struct "Pt")) [(.structLit "Pt" [("x", (.intLit (1))), ("y", (.intLit (2)))]), (.structLit "Pt" [("x", (.intLit (3))), ("y", (.intLit (4)))]), (.structLit "Pt" [("x", (.intLit (5))), ("y", (.intLit (6)))]), (.structLit "Pt" [("x", (.intLit (7))), ("y", (.intLit (8)))]), (.structLit "Pt" [("x", (.intLit (9))), ("y", (.intLit (10)))]), (.structLit "Pt" [("x", (.intLit (11))), ("y", (.intLit (12)))])]))],
  functions := [{ name := "sum_param", params := [("m", (.array 2 (.array 3 (.struct "Pt"))))], ret := (some (.u32)), body := [.varDecl "total" (.u32) (some (.intLit (0))), .forLoop "i" (.intLit (2)) [.forLoop "j" (.intLit (3)) [.assign (.identifier "total") (.binop "+" (.identifier "total") (.fieldAccess (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j")) "x"))]], .return_ (some (.identifier "total"))] }],
  tests := [{ name := "basic", params := [], ret := none, body := [.bareCall (.call "assert_eq" [(.call "sum_param" [(.identifier "g_pts")]), (.intLit (36))])] }],
  benches := [{ name := "param_throughput", params := [], ret := none, body := [] }]
}

def igla_w524_2d_packed_aos_param_module_env : Env := {
  structs := [("Buf", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_bufs", "assert_eq"]
}

def igla_w524_2d_packed_aos_param_module_module : Module := {
  name := "igla_w524_2d_packed_aos_param_module",
  imports := [],
  globals := [.constDecl "g_bufs" (.array 2 (.array 2 (.struct "Buf"))) (some (.arrayLit (.array 2 (.struct "Buf")) [(.structLit "Buf" [("data", (.arrayLit (.u32) [(.intLit (1)), (.intLit (2)), (.intLit (3)), (.intLit (4))])), ("tag", (.intLit (10)))]), (.structLit "Buf" [("data", (.arrayLit (.u32) [(.intLit (5)), (.intLit (6)), (.intLit (7)), (.intLit (8))])), ("tag", (.intLit (11)))]), (.structLit "Buf" [("data", (.arrayLit (.u32) [(.intLit (9)), (.intLit (10)), (.intLit (11)), (.intLit (12))])), ("tag", (.intLit (12)))]), (.structLit "Buf" [("data", (.arrayLit (.u32) [(.intLit (13)), (.intLit (14)), (.intLit (15)), (.intLit (16))])), ("tag", (.intLit (13)))])]))],
  functions := [{ name := "sum_bufs", params := [("m", (.array 2 (.array 2 (.struct "Buf"))))], ret := (some (.u32)), body := [.varDecl "total" (.u32) (some (.intLit (0))), .forLoop "i" (.intLit (2)) [.forLoop "j" (.intLit (2)) [.forLoop "k" (.intLit (4)) [.assign (.identifier "total") (.binop "+" (.identifier "total") (.index (.fieldAccess (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j")) "data") (.identifier "k")))]]], .return_ (some (.identifier "total"))] }],
  tests := [{ name := "basic", params := [], ret := none, body := [.bareCall (.call "assert_eq" [(.call "sum_bufs" [(.identifier "g_bufs")]), (.intLit (136))])] }],
  benches := [{ name := "throughput", params := [], ret := none, body := [] }]
}

def interop_gf_cross_language_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["gf16_from_f64", "gf16_is_zero", "gf16_add", "gf16_extract_mantissa", "gf32_extract_mantissa", "gf32_from_f64", "gf16_to_f64"]
}

def interop_gf_cross_language_module : Module := {
  name := "interop_gf_cross_language",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "phi_gf32_bits", params := [], ret := none, body := [.varDecl "phi" (.u32) (some (.intLit (0))), .varDecl "gf32_bits" (.u32) (some (.call "gf32_from_f64" [(.identifier "phi")])), .varDecl "expected" (.u32) (some (.intLit (0)))] }, { name := "phi_ffi_encoding", params := [], ret := none, body := [.varDecl "phi" (.u32) (some (.identifier "constants::PHI")), .varDecl "gf16_phi" (.u32) (some (.call "gf16_from_f64" [(.identifier "phi")])), .varDecl "gf32_phi" (.u32) (some (.call "gf32_from_f64" [(.identifier "phi")]))] }, { name := "roundtrip_gf16_precision", params := [], ret := none, body := [.varDecl "test_vals" (.u32) (some (.arrayLit (.u32) []))] }, { name := "classification_functions", params := [], ret := none, body := [.varDecl "zero" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "inf" (.u32) (some (.call "gf16_from_f64" [(.intLit (1)), (.identifier "e38_f64")])), .varDecl "nan" (.u32) (some (.intLit (0)))] }, { name := "gf16_addition", params := [], ret := none, body := [.varDecl "a" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "b" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "sum" (.u32) (some (.call "gf16_add" [(.identifier "a"), (.identifier "b")])), .varDecl "result" (.u32) (some (.call "gf16_to_f64" [(.identifier "sum")]))] }, { name := "zero_roundtrip", params := [], ret := none, body := [.varDecl "zero_enc" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "zero_dec" (.u32) (some (.call "gf16_to_f64" [(.identifier "zero_enc")]))] }, { name := "mantissa_extraction", params := [], ret := none, body := [.varDecl "phi_gf16" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "phi_gf32" (.u32) (some (.call "gf32_from_f64" [(.intLit (0))])), .varDecl "mant16" (.u32) (some (.call "gf16_extract_mantissa" [(.identifier "phi_gf16")])), .varDecl "mant32" (.u32) (some (.call "gf32_extract_mantissa" [(.identifier "phi_gf32")]))] }, { name := "special_values_roundtrip", params := [], ret := none, body := [.varDecl "zero_enc" (.u32) (some (.call "gf16_from_f64" [(.intLit (0))])), .varDecl "zero_dec" (.u32) (some (.call "gf16_to_f64" [(.identifier "zero_enc")])), .varDecl "zero_ok" (.u32) (some (.binop "and" (.binop "==" (.identifier "zero_dec") (.intLit (0))) (.call "gf16_is_zero" [(.identifier "zero_enc")]))), .varDecl "neg_zero_enc" (.u32) (some (.call "gf16_from_f64" [(.unop "-" (.intLit (0)))])), .varDecl "neg_zero_dec" (.u32) (some (.call "gf16_to_f64" [(.identifier "neg_zero_enc")])), .varDecl "neg_zero_ok" (.u32) (some (.binop "and" (.binop "==" (.identifier "neg_zero_dec") (.unop "-" (.intLit (0)))) (.call "gf16_is_zero" [(.identifier "neg_zero_enc")])))] }],
  benches := []
}

def isa_ternary_arithmetic_env : Env := {
  structs := [("usize", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def isa_ternary_arithmetic_module : Module := {
  name := "isa_ternary_arithmetic",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "MIN_TRIT" (.i32) (some (.intLit (-1))), .constDecl "MAX_TRIT" (.i32) (some (.intLit (1))), .constDecl "TRITS_PER_WORD" (.struct "usize") (some (.intLit (27)))],
  functions := [],
  tests := [{ name := "trit_add_basic", params := [], ret := none, body := [] }, { name := "trit_sub_basic", params := [], ret := none, body := [] }, { name := "trit_mul_basic", params := [], ret := none, body := [] }, { name := "ternary_to_decimal_basic", params := [], ret := none, body := [] }, { name := "decimal_to_ternary_basic", params := [], ret := none, body := [] }, { name := "is_valid_trit_check", params := [], ret := none, body := [] }, { name := "ternary_compare_basic", params := [], ret := none, body := [] }],
  benches := [{ name := "trit_add_performance", params := [], ret := none, body := [] }, { name := "trit_mul_performance", params := [], ret := none, body := [] }, { name := "conversion_performance", params := [], ret := none, body := [] }, { name := "comparison_performance", params := [], ret := none, body := [] }]
}

def isa_ternary_bitwise_env : Env := {
  structs := [("usize", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def isa_ternary_bitwise_module : Module := {
  name := "isa_ternary_bitwise",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "WORD_SIZE" (.struct "usize") (some (.intLit (27)))],
  functions := [],
  tests := [{ name := "tritwise_and_basic", params := [], ret := none, body := [] }, { name := "tritwise_or_basic", params := [], ret := none, body := [] }, { name := "tritwise_xor_basic", params := [], ret := none, body := [] }, { name := "tritwise_not_basic", params := [], ret := none, body := [] }, { name := "tritwise_nand_basic", params := [], ret := none, body := [] }, { name := "tritwise_nor_basic", params := [], ret := none, body := [] }, { name := "tritwise_xnor_basic", params := [], ret := none, body := [] }, { name := "tritwise_mask_basic", params := [], ret := none, body := [] }, { name := "tritwise_merge_basic", params := [], ret := none, body := [] }],
  benches := [{ name := "tritwise_and_performance", params := [], ret := none, body := [] }, { name := "tritwise_or_performance", params := [], ret := none, body := [] }, { name := "tritwise_xor_performance", params := [], ret := none, body := [] }, { name := "tritwise_not_performance", params := [], ret := none, body := [] }, { name := "tritwise_mask_performance", params := [], ret := none, body := [] }]
}

def isa_ternary_deque_env : Env := {
  structs := [("usize", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def isa_ternary_deque_module : Module := {
  name := "isa_ternary_deque",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "DEQUE_MAX_SIZE" (.struct "usize") (some (.intLit (27)))],
  functions := [],
  tests := [{ name := "deque_init_empty", params := [], ret := none, body := [] }, { name := "deque_push_back_pop_front", params := [], ret := none, body := [] }, { name := "deque_push_front_pop_back", params := [], ret := none, body := [] }, { name := "deque_peek_front", params := [], ret := none, body := [] }, { name := "deque_peek_back", params := [], ret := none, body := [] }, { name := "deque_is_empty_full", params := [], ret := none, body := [] }, { name := "deque_push_back_full", params := [], ret := none, body := [] }, { name := "deque_clear", params := [], ret := none, body := [] }],
  benches := [{ name := "deque_push_back_performance", params := [], ret := none, body := [] }, { name := "deque_push_front_performance", params := [], ret := none, body := [] }, { name := "deque_pop_front_performance", params := [], ret := none, body := [] }, { name := "deque_peek_performance", params := [], ret := none, body := [] }, { name := "deque_mixed_performance", params := [], ret := none, body := [] }]
}

def isa_ternary_encoding_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def isa_ternary_encoding_module : Module := {
  name := "isa_ternary_encoding",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def isa_ternary_gates_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def isa_ternary_gates_module : Module := {
  name := "isa_ternary_gates",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "GATE_DELAY_UNIT" (.u64) (some (.intLit (1)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "ternary_not_inversion", params := [], ret := none, body := [] }, { name := "ternary_min_behavior", params := [], ret := none, body := [] }, { name := "ternary_max_behavior", params := [], ret := none, body := [] }, { name := "ternary_and_logic", params := [], ret := none, body := [] }, { name := "ternary_or_logic", params := [], ret := none, body := [] }, { name := "ternary_consensus_all_equal", params := [], ret := none, body := [] }, { name := "ternary_consensus_not_all_equal", params := [], ret := none, body := [] }, { name := "ternary_majority_clear_cases", params := [], ret := none, body := [] }, { name := "ternary_any_with_positive", params := [], ret := none, body := [] }, { name := "ternary_any_without_positive", params := [], ret := none, body := [] }, { name := "ternary_all_with_negative", params := [], ret := none, body := [] }, { name := "ternary_all_with_positive", params := [], ret := none, body := [] }],
  benches := [{ name := "ternary_not_performance", params := [], ret := none, body := [] }, { name := "ternary_min_performance", params := [], ret := none, body := [] }, { name := "ternary_max_performance", params := [], ret := none, body := [] }, { name := "ternary_consensus_performance", params := [], ret := none, body := [] }]
}

def isa_ternary_shift_env : Env := {
  structs := [("usize", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def isa_ternary_shift_module : Module := {
  name := "isa_ternary_shift",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "TRIT_NEG" (.i32) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i32) (some (.intLit (0))), .constDecl "TRIT_POS" (.i32) (some (.intLit (1))), .constDecl "WORD_SIZE" (.struct "usize") (some (.intLit (27))), .constDecl "SHIFT_MASK" (.struct "usize") (some (.intLit (26))), .constDecl "SHIFT_LEFT" (.u8) (some (.intLit (0))), .constDecl "SHIFT_RIGHT" (.u8) (some (.intLit (1)))],
  functions := [],
  tests := [{ name := "ternary_shift_left_basic", params := [], ret := none, body := [] }, { name := "ternary_shift_right_basic", params := [], ret := none, body := [] }, { name := "ternary_rotate_left_basic", params := [], ret := none, body := [] }, { name := "ternary_rotate_right_basic", params := [], ret := none, body := [] }, { name := "ternary_rotate_full_circle", params := [], ret := none, body := [] }, { name := "ternary_arithmetic_shift_right_positive", params := [], ret := none, body := [] }, { name := "ternary_arithmetic_shift_right_negative", params := [], ret := none, body := [] }, { name := "extract_trits_basic", params := [], ret := none, body := [] }, { name := "insert_trits_basic", params := [], ret := none, body := [] }],
  benches := [{ name := "shift_left_performance", params := [], ret := none, body := [] }, { name := "rotate_left_performance", params := [], ret := none, body := [] }, { name := "rotate_right_performance", params := [], ret := none, body := [] }, { name := "arithmetic_shift_right_performance", params := [], ret := none, body := [] }]
}

def math_property_test_template_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def math_property_test_template_module : Module := {
  name := "math_property_test_template",
  imports := [{ path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "STRATEGY_RANDOM" (.i32) (some (.intLit (0)))],
  functions := [],
  tests := [],
  benches := [{ name := "property_test_associative", params := [], ret := none, body := [] }, { name := "property_test_commutative", params := [], ret := none, body := [] }, { name := "property_test_distributive", params := [], ret := none, body := [] }, { name := "property_test_identity", params := [], ret := none, body := [] }]
}

def ml_activation_silu_swish_vbt_activation_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_activation_silu_swish_vbt_activation_module : Module := {
  name := "ml_activation_silu_swish_vbt_activation",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def ml_igla_champion_capsule_env : Env := {
  structs := [("f64", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_igla_champion_capsule_module : Module := {
  name := "ml_igla_champion_capsule",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "PHI" (.struct "f64") (some (.intLit (0))), .constDecl "CHAMPION_SEED" (.u32) (some (.intLit (43))), .constDecl "CHAMPION_BPB" (.struct "f64") (some (.intLit (0))), .constDecl "CHAMPION_STEPS" (.u32) (some (.intLit (81000))), .constDecl "CHAMPION_HIDDEN" (.u32) (some (.intLit (828))), .constDecl "BETA1_CANONICAL" (.struct "f64") (some (.intLit (0))), .constDecl "WEIGHT_DECAY_CANONICAL" (.struct "f64") (some (.intLit (0))), .constDecl "LR_CANONICAL" (.struct "f64") (some (.intLit (0))), .constDecl "BETA2_DEFAULT" (.struct "f64") (some (.intLit (0))), .constDecl "EPSILON_DEFAULT" (.struct "f64") (some (.intLit (1))), .constDecl "RANDOM_CHAR_BPB_CEILING" (.struct "f64") (some (.intLit (0))), .constDecl "CORPUS_UNIQUE_TOKENS" (.u64) (some (.intLit (1115394))), .constDecl "CHINCHILLA_MIN_UNIQUE" (.u64) (some (.intLit (300000000))), .constDecl "PHI_INV3_HALF_WRONG" (.struct "f64") (some (.intLit (0)))],
  functions := [],
  tests := [{ name := "capsule_beta1_is_phi_inverse", params := [], ret := none, body := [] }, { name := "capsule_weight_decay_is_phi_inv_cubed", params := [], ret := none, body := [] }, { name := "capsule_weight_decay_equals_one_over_2phi_plus_1", params := [], ret := none, body := [] }, { name := "capsule_lr_equals_weight_decay", params := [], ret := none, body := [] }, { name := "capsule_wrong_value_is_half_of_weight_decay", params := [], ret := none, body := [] }, { name := "capsule_l5_trinity_identity_holds", params := [], ret := none, body := [] }, { name := "capsule_champion_corpus_below_chinchilla_unique", params := [], ret := none, body := [] }],
  benches := [{ name := "capsule_constant_load", params := [], ret := none, body := [] }]
}

def ml_layers_avgpool2d_layer_env : Env := {
  structs := [("[[]U32 = [2,2];
    const DEFAULT_STRIDE : [[]U32=[2, 2];pubconstPoolConfig=struct{kernel_size:[]u32,stride:[]u32,padding:[]u32,};pubconstPoolShape=struct{batch:u32,channels:u32,height_in:u32,width_in:u32,height_out:u32,width_out:u32,};fncalc_output_size(height_in:u32)->void{}fnforward(input:[]f32)->void{}testcalc_output_size_basic_casegiveninput=default_input()whenresult=calc_output_size(input)thenresult!=undefinedtestforward_basic_casegiveninput=default_input()whenresult=forward(input)thenresult!=undefinedinvariantavgpool2d_constraint_0giveninput=valid_input()thentrueinvariantavgpool2d_constraint_1giveninput=valid_input()thentrueinvariantavgpool2d_constraint_2giveninput=valid_input()thentrueinvariantavgpool2d_constraint_3giveninput=valid_input()thentrueinvariantavgpool2d_constraint_4giveninput=valid_input()thentrueinvariantavgpool2d_constraint_5giveninput=valid_input()thentrue", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_avgpool2d_layer_module : Module := {
  name := "ml_layers_avgpool2d_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_KERNEL" (.struct "[[]U32 = [2,2];
    const DEFAULT_STRIDE : [[]U32=[2, 2];pubconstPoolConfig=struct{kernel_size:[]u32,stride:[]u32,padding:[]u32,};pubconstPoolShape=struct{batch:u32,channels:u32,height_in:u32,width_in:u32,height_out:u32,width_out:u32,};fncalc_output_size(height_in:u32)->void{}fnforward(input:[]f32)->void{}testcalc_output_size_basic_casegiveninput=default_input()whenresult=calc_output_size(input)thenresult!=undefinedtestforward_basic_casegiveninput=default_input()whenresult=forward(input)thenresult!=undefinedinvariantavgpool2d_constraint_0giveninput=valid_input()thentrueinvariantavgpool2d_constraint_1giveninput=valid_input()thentrueinvariantavgpool2d_constraint_2giveninput=valid_input()thentrueinvariantavgpool2d_constraint_3giveninput=valid_input()thentrueinvariantavgpool2d_constraint_4giveninput=valid_input()thentrueinvariantavgpool2d_constraint_5giveninput=valid_input()thentrue") none],
  functions := [],
  tests := [],
  benches := []
}

def ml_layers_conv2d_layer_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_conv2d_layer_module : Module := {
  name := "ml_layers_conv2d_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "MIN_KERNEL_SIZE" (.u32) (some (.intLit (1))), .constDecl "MAX_KERNEL_SIZE" (.u32) (some (.intLit (7)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "backward", params := [("grad_output", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "backward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_layers_dense_layer_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_dense_layer_module : Module := {
  name := "ml_layers_dense_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "MIN_INPUT_SIZE" (.u32) (some (.intLit (1))), .constDecl "MAX_INPUT_SIZE" (.u32) (some (.intLit (4096)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "backward", params := [("grad_output", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "backward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_layers_embedding_layer_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_embedding_layer_module : Module := {
  name := "ml_layers_embedding_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_D_MODEL" (.u32) (some (.intLit (64)))],
  functions := [{ name := "forward", params := [("token_ids", (.struct "[]const u32"))], ret := (some (.struct "void")), body := [] }, { name := "init", params := [("weights", (.struct "EmbeddingWeights"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "init_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_layers_flatten_layer_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_flatten_layer_module : Module := {
  name := "ml_layers_flatten_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "MAX_DIMS" (.u32) (some (.intLit (8)))],
  functions := [{ name := "calc_output_size", params := [("input_dims", (.struct "[]u32"))], ret := (some (.struct "void")), body := [] }, { name := "forward", params := [("input", (.struct "[]f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "calc_output_size_basic_case", params := [], ret := none, body := [] }, { name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_layers_maxpool2d_layer_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_maxpool2d_layer_module : Module := {
  name := "ml_layers_maxpool2d_layer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "backward", params := [("grad_output", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "backward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_layers_residual_connection_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_layers_residual_connection_module : Module := {
  name := "ml_layers_residual_connection",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_loss_kl_divergence_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_loss_kl_divergence_module : Module := {
  name := "ml_loss_kl_divergence",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("p", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_loss_mse_loss_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_loss_mse_loss_module : Module := {
  name := "ml_loss_mse_loss",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("predictions", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_optimizer_lr_scheduler_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("step", "SchedulerState"), ("init", "SchedulerState")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants")), ("gf16", ("numeric::gf16", "gf16")), ("trigonometry", ("math", "trigonometry"))],
  hostOnly := [],
  reachable := []
}

def ml_optimizer_lr_scheduler_module : Module := {
  name := "ml_optimizer_lr_scheduler",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }, { path := "math::trigonometry", items := ["trigonometry"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "PHI" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "INV_PHI" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "DEFAULT_MAX_LR" (.struct "gf16::GF16") (some (.intLit (1))), .constDecl "DEFAULT_MIN_LR" (.struct "gf16::GF16") (some (.intLit (1))), .constDecl "DEFAULT_WARMUP_STEPS" (.u32) (some (.intLit (2000))), .constDecl "DEFAULT_MAX_STEPS" (.u32) (some (.intLit (100000)))],
  functions := [{ name := "init", params := [("config", (.struct "SchedulerConfig"))], ret := (some (.struct "SchedulerState")), body := [] }, { name := "step", params := [("state", (.struct "SchedulerState"))], ret := (some (.struct "SchedulerState")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_creates_valid_state", params := [], ret := none, body := [] }, { name := "get_lr_at_step_zero_returns_min", params := [], ret := none, body := [] }, { name := "get_lr_at_warmup_end_returns_max", params := [], ret := none, body := [] }, { name := "get_lr_at_max_steps_returns_min", params := [], ret := none, body := [] }, { name := "linear_warmup_increases_monotonically", params := [], ret := none, body := [] }, { name := "cosine_decay_decreases_monotonically", params := [], ret := none, body := [] }, { name := "step_increases_step_counter", params := [], ret := none, body := [] }],
  benches := [{ name := "init", params := [], ret := none, body := [] }, { name := "get_lr", params := [], ret := none, body := [] }, { name := "get_lr_at_step", params := [], ret := none, body := [] }]
}

def ml_optimizer_sgd_momentum_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("zero_grad", "SgdMomentumState"), ("step", "OptimizerStepResult"), ("init", "SgdMomentumState")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants")), ("gf16", ("numeric::gf16", "gf16"))],
  hostOnly := [],
  reachable := []
}

def ml_optimizer_sgd_momentum_module : Module := {
  name := "ml_optimizer_sgd_momentum",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "PHI" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "DEFAULT_LEARNING_RATE" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "DEFAULT_MOMENTUM" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "PHI_DAMPED_MOMENTUM" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "DEFAULT_WEIGHT_DECAY" (.struct "gf16::GF16") (some (.intLit (0))), .constDecl "DEFAULT_NESTEROV" (.bool) (some (.intLit (0)))],
  functions := [{ name := "init", params := [("config", (.struct "SgdMomentumConfig")), ("param_count", (.u32))], ret := (some (.struct "SgdMomentumState")), body := [] }, { name := "step", params := [("state", (.struct "SgdMomentumState")), ("params", (.struct "[]gf16::GF16")), ("grads", (.struct "[]gf16::GF16"))], ret := (some (.struct "OptimizerStepResult")), body := [] }, { name := "zero_grad", params := [("state", (.struct "SgdMomentumState"))], ret := (some (.struct "SgdMomentumState")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_creates_zero_velocities", params := [], ret := none, body := [] }, { name := "compute_velocity_basic_case", params := [], ret := none, body := [] }, { name := "compute_velocity_with_previous_velocity", params := [], ret := none, body := [] }, { name := "standard_update_decreases_param", params := [], ret := none, body := [] }, { name := "nesterov_update_considers_lookahead", params := [], ret := none, body := [] }, { name := "step_without_momentum_behaves_like_sgd", params := [], ret := none, body := [] }, { name := "step_with_momentum_accumulates_velocity", params := [], ret := none, body := [] }, { name := "step_with_weight_decay_modifies_gradients", params := [], ret := none, body := [] }, { name := "apply_weight_decay_increases_gradient_magnitude", params := [], ret := none, body := [] }, { name := "phi_damped_momentum_reduces_momentum", params := [], ret := none, body := [] }, { name := "get_effective_momentum_returns_phi_damped_when_enabled", params := [], ret := none, body := [] }, { name := "get_effective_momentum_returns_base_when_disabled", params := [], ret := none, body := [] }, { name := "step_norm_is_correct", params := [], ret := none, body := [] }],
  benches := [{ name := "init_small", params := [], ret := none, body := [] }, { name := "init_large", params := [], ret := none, body := [] }, { name := "step_small", params := [], ret := none, body := [] }, { name := "step_medium", params := [], ret := none, body := [] }, { name := "step_large", params := [], ret := none, body := [] }, { name := "compute_velocity", params := [], ret := none, body := [] }, { name := "nesterov_update", params := [], ret := none, body := [] }]
}

def ml_pathway_mlp_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_pathway_mlp_module : Module := {
  name := "ml_pathway_mlp",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "MNIST_INPUT_SIZE" (.u32) (some (.intLit (784))), .constDecl "MNIST_OUTPUT_SIZE" (.u32) (some (.intLit (10))), .constDecl "DEFAULT_HIDDEN_SIZE" (.u32) (some (.intLit (128)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "init", params := [("state", (.struct "MLPState"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "init_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_bilstm_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_bilstm_module : Module := {
  name := "ml_recurrent_bilstm",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_INPUT_SIZE" (.u32) (some (.intLit (256))), .constDecl "DEFAULT_HIDDEN_SIZE" (.u32) (some (.intLit (256)))],
  functions := [{ name := "forward_direction", params := [("inputs", (.struct "[][]f32"))], ret := (some (.struct "void")), body := [] }, { name := "forward", params := [("inputs", (.struct "[][]f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_direction_basic_case", params := [], ret := none, body := [] }, { name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_gru_cell_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_gru_cell_module : Module := {
  name := "ml_recurrent_gru_cell",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "init", params := [("weights", (.struct "GRUWeights"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "init_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_lstm_cell_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_lstm_cell_module : Module := {
  name := "ml_recurrent_lstm_cell",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "LSTM_DEFAULT_HIDDEN" (.u32) (some (.intLit (128)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "init", params := [("weights", (.struct "LSTMWeights"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }, { name := "init_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_rnn_cell_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_rnn_cell_module : Module := {
  name := "ml_recurrent_rnn_cell",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_INPUT_SIZE" (.u32) (some (.intLit (128))), .constDecl "DEFAULT_HIDDEN_SIZE" (.u32) (some (.intLit (128)))],
  functions := [{ name := "forward_step", params := [("input", (.struct "[]f32"))], ret := (some (.struct "void")), body := [] }, { name := "forward_sequence", params := [("inputs", (.struct "[][]f32"))], ret := (some (.struct "void")), body := [] }, { name := "init_state", params := [("hidden_size", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_step_basic_case", params := [], ret := none, body := [] }, { name := "forward_sequence_basic_case", params := [], ret := none, body := [] }, { name := "init_state_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_self_attention_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_self_attention_module : Module := {
  name := "ml_recurrent_self_attention",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "D_MODEL" (.u32) (some (.intLit (64)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_recurrent_seq2seq_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_recurrent_seq2seq_module : Module := {
  name := "ml_recurrent_seq2seq",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "encode", params := [("input_seq", (.struct "[]const u32"))], ret := (some (.struct "void")), body := [] }, { name := "decode", params := [("target_seq", (.struct "[]const u32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "encode_basic_case", params := [], ret := none, body := [] }, { name := "decode_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_rl_ppo_critic_env : Env := {
  structs := [("[[]U32 = [64,64];

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // 2. Types
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    pub const CriticConfig = struct {
        state_dim : u32,
        hidden_dims : []u32,
        activation : Activation,
    };

    pub const Activation = struct {
        enum : ,
    };

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // 3. Core Functions
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    // forward(state: []const f32) â void
    fn forward(state: []const f32) -> void {
        // TODO: Implement from .tri spec
    }

    // compute_advantage(rewards: []f32) â void
    fn compute_advantage(rewards: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // compute_returns(rewards: []f32) â void
    fn compute_returns(rewards: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // value_loss(predicted_values: []f32) â void
    fn value_loss(predicted_values: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // TDD: Tests (from .tri behaviors)
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    test forward_basic_case
        given input = default_input()
        when result = forward(input)
        then result != undefined

    test compute_advantage_basic_case
        given input = default_input()
        when result = compute_advantage(input)
        then result != undefined

    test compute_returns_basic_case
        given input = default_input()
        when result = compute_returns(input)
        then result != undefined

    test value_loss_basic_case
        given input = default_input()
        when result = value_loss(input)
        then result != undefined

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // TDD: Invariants (from .tri constraints)
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    invariant ppo_critic_constraint_0
        given input = valid_input()
        then true // 0 <= gamma <= 1

    invariant ppo_critic_constraint_1
        given input = valid_input()
        then true // 0 <= lambda_ <= 1

    invariant ppo_critic_constraint_2
        given input = valid_input()
        then true // name: gae_formula

    invariant ppo_critic_constraint_3
        given input = valid_input()
        then true // name: td_error

    invariant ppo_critic_constraint_4
        given input = valid_input()
        then true // Schulmanetal.(2016)-High-dimensionalcontinuouscontrolusingGAE

    invariant ppo_critic_constraint_5
        given input = valid_input()
        then true // Schulmanetal.(2017)-ProximalPolicyOptimizationAlgorithms", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_rl_ppo_critic_module : Module := {
  name := "ml_rl_ppo_critic",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_HIDDEN" (.struct "[[]U32 = [64,64];

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // 2. Types
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    pub const CriticConfig = struct {
        state_dim : u32,
        hidden_dims : []u32,
        activation : Activation,
    };

    pub const Activation = struct {
        enum : ,
    };

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // 3. Core Functions
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    // forward(state: []const f32) â void
    fn forward(state: []const f32) -> void {
        // TODO: Implement from .tri spec
    }

    // compute_advantage(rewards: []f32) â void
    fn compute_advantage(rewards: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // compute_returns(rewards: []f32) â void
    fn compute_returns(rewards: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // value_loss(predicted_values: []f32) â void
    fn value_loss(predicted_values: []f32) -> void {
        // TODO: Implement from .tri spec
    }

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // TDD: Tests (from .tri behaviors)
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    test forward_basic_case
        given input = default_input()
        when result = forward(input)
        then result != undefined

    test compute_advantage_basic_case
        given input = default_input()
        when result = compute_advantage(input)
        then result != undefined

    test compute_returns_basic_case
        given input = default_input()
        when result = compute_returns(input)
        then result != undefined

    test value_loss_basic_case
        given input = default_input()
        when result = value_loss(input)
        then result != undefined

    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
    // TDD: Invariants (from .tri constraints)
    // âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

    invariant ppo_critic_constraint_0
        given input = valid_input()
        then true // 0 <= gamma <= 1

    invariant ppo_critic_constraint_1
        given input = valid_input()
        then true // 0 <= lambda_ <= 1

    invariant ppo_critic_constraint_2
        given input = valid_input()
        then true // name: gae_formula

    invariant ppo_critic_constraint_3
        given input = valid_input()
        then true // name: td_error

    invariant ppo_critic_constraint_4
        given input = valid_input()
        then true // Schulmanetal.(2016)-High-dimensionalcontinuouscontrolusingGAE

    invariant ppo_critic_constraint_5
        given input = valid_input()
        then true // Schulmanetal.(2017)-ProximalPolicyOptimizationAlgorithms") none],
  functions := [],
  tests := [],
  benches := []
}

def ml_transformer_feed_forward_network_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def ml_transformer_feed_forward_network_module : Module := {
  name := "ml_transformer_feed_forward_network",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_D_FF" (.u32) (some (.intLit (2048)))],
  functions := [{ name := "forward", params := [("input", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_transformer_multi_head_attention_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_transformer_multi_head_attention_module : Module := {
  name := "ml_transformer_multi_head_attention",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "forward", params := [("query", (.struct "[]const f32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def ml_transformer_positional_encoding_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def ml_transformer_positional_encoding_module : Module := {
  name := "ml_transformer_positional_encoding",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "DEFAULT_MAX_LEN" (.u32) (some (.intLit (2048)))],
  functions := [{ name := "forward", params := [("positions", (.struct "[]const u32"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "forward_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def nn_phi_rope_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def nn_phi_rope_module : Module := {
  name := "nn_phi_rope",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def nn_sacred_attention_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def nn_sacred_attention_module : Module := {
  name := "nn_sacred_attention",
  imports := [],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def numeric_bigint_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("gf16", ("numeric::gf16", "gf16"))],
  hostOnly := [],
  reachable := []
}

def numeric_bigint_module : Module := {
  name := "numeric_bigint",
  imports := [{ path := "base::types", items := ["types"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "TRIT_NEG" (.i8) (some (.intLit (-1))), .constDecl "TRIT_ZERO" (.i8) (some (.intLit (0))), .constDecl "TRIT_POS" (.i8) (some (.intLit (1))), .constDecl "MAX_TRITS" (.u16) (some (.intLit (256))), .constDecl "SIMD_CHUNKS" (.u16) (some (.intLit (8))), .constDecl "KARATSUBA_THRESHOLD" (.u16) (some (.intLit (32)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "bigint_zero_is_zero", params := [], ret := none, body := [] }, { name := "bigint_from_i64_zero", params := [], ret := none, body := [] }, { name := "bigint_from_i64_positive", params := [], ret := none, body := [] }, { name := "bigint_from_i64_negative", params := [], ret := none, body := [] }, { name := "bigint_normalize_removes_leading_zeros", params := [], ret := none, body := [] }, { name := "bigint_is_zero_after_normalize", params := [], ret := none, body := [] }, { name := "bigint_negate_flips_signs", params := [], ret := none, body := [] }, { name := "bigint_negate_zero_is_zero", params := [], ret := none, body := [] }, { name := "bigint_add_commutes", params := [], ret := none, body := [] }, { name := "bigint_add_zero_identity", params := [], ret := none, body := [] }, { name := "bigint_sub_reverses_addition", params := [], ret := none, body := [] }, { name := "bigint_mul_by_zero_returns_zero", params := [], ret := none, body := [] }, { name := "bigint_mul_commutates", params := [], ret := none, body := [] }, { name := "bigint_compare_abs_returns_correct_comparison", params := [], ret := none, body := [] }, { name := "bigint_abs_returns_positive_for_negative", params := [], ret := none, body := [] }, { name := "bigint_to_i64_roundtrip_small", params := [], ret := none, body := [] }],
  benches := [{ name := "bigint_from_i64_latency", params := [], ret := none, body := [] }, { name := "bigint_to_i64_latency", params := [], ret := none, body := [] }, { name := "bigint_add_latency_32_trits", params := [], ret := none, body := [] }, { name := "bigint_sub_latency_32_trits", params := [], ret := none, body := [] }, { name := "bigint_mul_simple_latency_16_trits", params := [], ret := none, body := [] }, { name := "bigint_mul_karatsuba_latency_64_trits", params := [], ret := none, body := [] }, { name := "bigint_normalize_latency_64_trits", params := [], ret := none, body := [] }, { name := "bigint_negate_latency_64_trits", params := [], ret := none, body := [] }, { name := "bigint_compare_latency_64_trits", params := [], ret := none, body := [] }]
}

def numeric_formats_env : Env := {
  structs := [("u4", [("value", .u32)]), ("u5", [("value", .u32)])],
  constructors := [],
  enums := ["Format"],
  imports := [("gf16", ("numeric::gf16", "gf16")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def numeric_formats_module : Module := {
  name := "numeric_formats",
  imports := [{ path := "base::types", items := ["types"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "SignMask" (.u16) (some (.intLit (0))), .constDecl "ExpMask" (.u16) (some (.intLit (0))), .constDecl "MantMask" (.u16) (some (.intLit (0))), .constDecl "ExpShift" (.struct "u5") (some (.intLit (9))), .constDecl "SignShift" (.struct "u4") (some (.intLit (15))), .constDecl "Bias" (.i32) (some (.intLit (31))), .constDecl "ExpMax" (.u16) (some (.intLit (63))), .constDecl "ExpMin" (.u16) (some (.intLit (0)))],
  functions := [],
  tests := [{ name := "gf16_to_f32_zero_positive", params := [], ret := none, body := [] }, { name := "gf16_to_f32_zero_negative", params := [], ret := none, body := [] }, { name := "gf16_to_f32_denormal", params := [], ret := none, body := [] }, { name := "gf16_to_f32_normal_one", params := [], ret := none, body := [] }, { name := "gf16_to_f32_positive_inf", params := [], ret := none, body := [] }, { name := "gf16_to_f32_negative_inf", params := [], ret := none, body := [] }, { name := "gf16_to_f32_nan", params := [], ret := none, body := [] }, { name := "f32_to_gf16_zero_positive", params := [], ret := none, body := [] }, { name := "f32_to_gf16_zero_negative", params := [], ret := none, body := [] }, { name := "f32_to_gf16_one", params := [], ret := none, body := [] }, { name := "f32_to_gf16_inf_positive", params := [], ret := none, body := [] }, { name := "f32_to_gf16_inf_negative", params := [], ret := none, body := [] }, { name := "f32_to_gf16_nan", params := [], ret := none, body := [] }, { name := "f32_to_ternary_positive", params := [], ret := none, body := [] }, { name := "f32_to_ternary_zero", params := [], ret := none, body := [] }, { name := "f32_to_ternary_negative", params := [], ret := none, body := [] }, { name := "f32_to_ternary_threshold", params := [], ret := none, body := [] }, { name := "f32_to_ternary_negative_threshold", params := [], ret := none, body := [] }, { name := "ternary_to_f32_positive", params := [], ret := none, body := [] }, { name := "ternary_to_f32_zero", params := [], ret := none, body := [] }, { name := "ternary_to_f32_negative", params := [], ret := none, body := [] }, { name := "format_bytes_fp32", params := [], ret := none, body := [] }, { name := "format_bytes_fp16", params := [], ret := none, body := [] }, { name := "format_bytes_ternary", params := [], ret := none, body := [] }, { name := "quantize_value_fp32", params := [], ret := none, body := [] }, { name := "quantize_value_ternary", params := [], ret := none, body := [] }],
  benches := [{ name := "gf16_to_f32_latency", params := [], ret := none, body := [] }, { name := "f32_to_gf16_latency", params := [], ret := none, body := [] }, { name := "f32_to_ternary_latency", params := [], ret := none, body := [] }, { name := "ternary_to_f32_latency", params := [], ret := none, body := [] }]
}

def numeric_gf_competitive_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := ["phi_identity_check", "gf16_phi_relative_error", "trinity_identity_check", "roundtrip_error", "accumulation_check"]
}

def numeric_gf_competitive_module : Module := {
  name := "numeric_gf_competitive",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "PHI" (.struct "f64") (some (.intLit (0))), .constDecl "TRINITY" (.struct "f64") (some (.intLit (0))), .constDecl "TOLERANCE_1E4" (.struct "f64") (some (.intLit (1))), .constDecl "TOLERANCE_1E3" (.struct "f64") (some (.intLit (0)))],
  functions := [{ name := "gf16_phi_relative_error", params := [("encoded", (.struct "f64"))], ret := (some (.struct "f64")), body := [.ifThenElse (.binop "==" (.identifier "PHI") (.intLit (0))) [.return_ (some (.intLit (0)))] [], .varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "encoded") (.identifier "PHI"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.binop "/" (.identifier "diff") (.identifier "PHI")))] }, { name := "phi_identity_check", params := [("phi_sq", (.struct "f64")), ("phi_plus_1", (.struct "f64"))], ret := (some (.struct "f64")), body := [.varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "phi_sq") (.identifier "phi_plus_1"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.identifier "diff"))] }, { name := "trinity_identity_check", params := [("computed", (.struct "f64"))], ret := (some (.struct "f64")), body := [.varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "computed") (.identifier "TRINITY"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.identifier "diff"))] }, { name := "roundtrip_error", params := [("original", (.struct "f64")), ("roundtripped", (.struct "f64"))], ret := (some (.struct "f64")), body := [.ifThenElse (.binop "==" (.identifier "original") (.intLit (0))) [.return_ (some (.intLit (0)))] [], .varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "roundtripped") (.identifier "original"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.binop "/" (.identifier "diff") (.identifier "original")))] }, { name := "accumulation_check", params := [("n", (.struct "usize")), ("expected_sum", (.struct "f64")), ("actual_sum", (.struct "f64"))], ret := (some (.struct "f64")), body := [.ifThenElse (.binop "==" (.identifier "expected_sum") (.intLit (0))) [.return_ (some (.intLit (0)))] [], .varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "actual_sum") (.identifier "expected_sum"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.binop "/" (.identifier "diff") (.identifier "expected_sum")))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "gf32_phi_representation", params := [], ret := none, body := [.varDecl "encoded_phi" (.struct "f64") (some (.intLit (0))), .varDecl "err" (.u32) (some (.call "gf16_phi_relative_error" [(.identifier "encoded_phi")])), .bareCall (.binop "<" (.unop "try " (.identifier "err")) (.identifier "TOLERANCE_1E3"))] }, { name := "phi_identity_gf16", params := [], ret := none, body := [.varDecl "phi_sq" (.struct "f64") (some (.intLit (0))), .varDecl "phi_p1" (.struct "f64") (some (.intLit (0))), .varDecl "err" (.u32) (some (.call "phi_identity_check" [(.identifier "phi_sq"), (.identifier "phi_p1")])), .bareCall (.binop "<" (.unop "try " (.identifier "err")) (.identifier "TOLERANCE_1E3"))] }, { name := "trinity_identity", params := [], ret := none, body := [.varDecl "computed" (.struct "f64") (some (.intLit (0))), .varDecl "err" (.u32) (some (.call "trinity_identity_check" [(.identifier "computed")])), .bareCall (.binop "<" (.unop "try " (.identifier "err")) (.identifier "TOLERANCE_1E4"))] }, { name := "roundtrip_precision", params := [], ret := none, body := [.varDecl "original" (.struct "f64") (some (.intLit (0))), .varDecl "roundtripped" (.struct "f64") (some (.intLit (0))), .varDecl "err" (.u32) (some (.call "roundtrip_error" [(.identifier "original"), (.identifier "roundtripped")])), .bareCall (.binop "<" (.unop "try " (.identifier "err")) (.identifier "TOLERANCE_1E3"))] }, { name := "accumulation", params := [], ret := none, body := [.varDecl "expected" (.struct "f64") (some (.intLit (0))), .varDecl "actual" (.struct "f64") (some (.intLit (0))), .varDecl "err" (.u32) (some (.call "accumulation_check" [(.intLit (1000)), (.identifier "expected"), (.identifier "actual")])), .bareCall (.binop "<" (.unop "try " (.identifier "err")) (.identifier "TOLERANCE_1E4"))] }],
  benches := [{ name := "gf16_encode_decode", params := [], ret := none, body := [.varDecl "x" (.struct "f64") (some (.intLit (0))), .varDecl "y" (.u32) (some (.call "roundtrip_error" [(.identifier "x"), (.identifier "x")]))] }]
}

def numeric_pellis_verify_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := ["compare_with_codata", "pellis_closed_form"]
}

def numeric_pellis_verify_module : Module := {
  name := "numeric_pellis_verify",
  imports := [{ path := "base::types", items := ["types"] }],
  globals := [.constDecl "CODATA_ALPHA_INV" (.struct "f64") (some (.intLit (0))), .constDecl "PELLIS_PRELIMINARY" (.struct "f64") (some (.intLit (0))), .constDecl "VERIFICATION_TOLERANCE" (.struct "f64") (some (.intLit (0)))],
  functions := [{ name := "pellis_closed_form", params := [("phi_sq", (.struct "f64")), ("phi_4", (.struct "f64")), ("phi_5_3", (.struct "f64"))], ret := (some (.struct "f64")), body := [.return_ (some (.binop "+" (.binop "-" (.binop "/" (.intLit (0)) (.identifier "phi_sq")) (.binop "/" (.intLit (0)) (.identifier "phi_4"))) (.binop "/" (.intLit (0)) (.identifier "phi_5_3"))))] }, { name := "compare_with_codata", params := [("pellis", (.struct "f64"))], ret := (some (.struct "f64")), body := [.ifThenElse (.binop "==" (.identifier "CODATA_ALPHA_INV") (.intLit (0))) [.return_ (some (.intLit (0)))] [], .varDecl "diff" (.struct "f64") (some (.binop "-" (.identifier "pellis") (.identifier "CODATA_ALPHA_INV"))), .ifThenElse (.binop "<" (.identifier "diff") (.intLit (0))) [.assign (.identifier "diff") (.unop "-" (.identifier "diff"))] [], .return_ (some (.binop "/" (.identifier "diff") (.identifier "CODATA_ALPHA_INV")))] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "pellis_near_alpha_inv", params := [], ret := none, body := [.varDecl "phi_sq" (.struct "f64") (some (.intLit (0))), .varDecl "phi_4" (.struct "f64") (some (.intLit (0))), .varDecl "phi_5_3" (.struct "f64") (some (.intLit (0))), .varDecl "pellis" (.u32) (some (.call "pellis_closed_form" [(.identifier "phi_sq"), (.identifier "phi_4"), (.identifier "phi_5_3")])), .varDecl "rel_err" (.u32) (some (.call "compare_with_codata" [(.identifier "pellis")])), .bareCall (.binop "<" (.unop "try " (.identifier "rel_err")) (.identifier "VERIFICATION_TOLERANCE"))] }, { name := "pellis_gt_137", params := [], ret := none, body := [.varDecl "phi_sq" (.struct "f64") (some (.intLit (0))), .varDecl "phi_4" (.struct "f64") (some (.intLit (0))), .varDecl "phi_5_3" (.struct "f64") (some (.intLit (0))), .varDecl "pellis" (.u32) (some (.call "pellis_closed_form" [(.identifier "phi_sq"), (.identifier "phi_4"), (.identifier "phi_5_3")])), .bareCall (.binop ">" (.unop "try " (.identifier "pellis")) (.intLit (0)))] }],
  benches := [{ name := "pellis_compute", params := [], ret := none, body := [.varDecl "phi_sq" (.struct "f64") (some (.intLit (0))), .varDecl "phi_4" (.struct "f64") (some (.intLit (0))), .varDecl "phi_5_3" (.struct "f64") (some (.intLit (0))), .varDecl "p" (.u32) (some (.call "pellis_closed_form" [(.identifier "phi_sq"), (.identifier "phi_4"), (.identifier "phi_5_3")]))] }]
}

def numeric_trinity_numeric_surface_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["@as", "std.testing.expectEqual"]
}

def numeric_trinity_numeric_surface_module : Module := {
  name := "numeric_trinity_numeric_surface",
  imports := [],
  globals := [.constDecl "POLICY_VERSION" (.u8) (some (.intLit (1))), .constDecl "GF4_RAW_BITS" (.u8) (some (.intLit (4))), .constDecl "GF8_RAW_BITS" (.u8) (some (.intLit (8))), .constDecl "GF12_RAW_BITS" (.u8) (some (.intLit (12))), .constDecl "GF16_RAW_BITS" (.u8) (some (.intLit (16))), .constDecl "GF20_RAW_BITS" (.u8) (some (.intLit (20))), .constDecl "GF24_RAW_BITS" (.u8) (some (.intLit (24))), .constDecl "GF32_RAW_BITS" (.u8) (some (.intLit (32))), .constDecl "PRIMARY_INFERENCE_RAW_BITS" (.u8) (some (.intLit (16)))],
  functions := [],
  tests := [{ name := "surface_primary_is_gf16", params := [], ret := none, body := [.bareCall (.unop "try " (.call "std.testing.expectEqual" [(.call "@as" [(.identifier "u8"), (.intLit (16))]), (.identifier "PRIMARY_INFERENCE_RAW_BITS")]))] }],
  benches := []
}

def physics_e8_lqg_bridge_env : Env := {
  structs := [("f64", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["params.calculate", "expect", "E8Root.init", "root.quantumProjection", "generateAll"]
}

def physics_e8_lqg_bridge_module : Module := {
  name := "physics_e8_lqg_bridge",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "E8: root generation", params := [], ret := none, body := [.varDecl "roots" (.u32) (some (.call "generateAll" []))] }, { name := "E8: sacred formula", params := [], ret := none, body := [.varDecl "params" (.u32) (some (.structLit "SacredParams" [("n", (.intLit (1))), ("k", (.unop "-" (.intLit (1)))), ("m", (.intLit (0))), ("p", (.unop "-" (.intLit (1)))), ("q", (.intLit (0)))])), .varDecl "V" (.u32) (some (.call "params.calculate" [])), .bareCall (.call "expect" [(.binop ">" (.identifier "V") (.intLit (0)))])] }, { name := "E8-LQG: projection", params := [], ret := none, body := [.varDecl "root" (.u32) (some (.call "E8Root.init" [(.arrayLit (.struct "f64") [(.intLit (1)), (.intLit (1)), (.intLit (0)), (.intLit (0)), (.intLit (0)), (.intLit (0)), (.intLit (0)), (.intLit (0))])])), .varDecl "proj" (.u32) (some (.call "root.quantumProjection" []))] }],
  benches := []
}

def physics_gamma_conflict_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def physics_gamma_conflict_module : Module := {
  name := "physics_gamma_conflict",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "", params := [], ret := none, body := [] }, { name := "", params := [], ret := none, body := [] }],
  benches := [{ name := "", params := [], ret := none, body := [] }, { name := "", params := [], ret := none, body := [] }]
}

def physics_hslm_benchmark_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["expect"]
}

def physics_hslm_benchmark_module : Module := {
  name := "physics_hslm_benchmark",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "HSLM-Bench: parameter estimation", params := [], ret := none, body := [.bareCall (.call "expect" [(.identifier "ESTIMATED_PARAMS"), (.intLit (4)), (.intLit (0)), (.identifier "M")])] }, { name := "HSLM-Bench: configuration consistency", params := [], ret := none, body := [] }],
  benches := []
}

def physics_lqg_cs_bridge_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def physics_lqg_cs_bridge_module : Module := {
  name := "physics_lqg_cs_bridge",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "lqg_cs_bridge_module_exists", params := [], ret := none, body := [] }, { name := "lqg_cs_gamma_conclusion_documented", params := [], ret := none, body := [] }],
  benches := []
}

def physics_lqg_entropy_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := []
}

def physics_lqg_entropy_module : Module := {
  name := "physics_lqg_entropy",
  imports := [],
  globals := [],
  functions := [],
  tests := [{ name := "lqg_entropy_module_exists", params := [], ret := none, body := [] }, { name := "lqg_cs_gamma_origin_different_from_lqg", params := [], ret := none, body := [] }],
  benches := []
}

def physics_quantum_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("sacred", ("sacred", "sacred"))],
  hostOnly := [],
  reachable := ["expect"]
}

def physics_quantum_module : Module := {
  name := "physics_quantum",
  imports := [{ path := "sacred", items := ["sacred"] }],
  globals := [],
  functions := [],
  tests := [{ name := "on", params := [], ret := none, body := [] }, { name := "with", params := [], ret := none, body := [] }, { name := "Quantum-CHSH: correlation calculation", params := [], ret := none, body := [] }, { name := "Quantum-CGLMP: entangled vs separable", params := [], ret := none, body := [] }, { name := "Quantum: TRINITY identity", params := [], ret := none, body := [.bareCall (.call "expect" [(.binop "^" (.binop "^" (.identifier "phi") (.binop "+" (.intLit (2)) (.identifier "phi"))) (.unop "-" (.intLit (2)))), (.intLit (7)), (.intLit (0))])] }],
  benches := [{ name := "quantum", params := [], ret := none, body := [] }, { name := "", params := [], ret := none, body := [] }]
}

def physics_su2_chern_simons_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("fibonacci_fusion_matrix", "FusionMatrix")],
  enums := [],
  imports := [("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def physics_su2_chern_simons_module : Module := {
  name := "physics_su2_chern_simons",
  imports := [{ path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "CS_LEVEL" (.i64) (some (.intLit (3))), .constDecl "NUM_SECTORS" (.i64) (some (.identifier "CS_LEVEL")), .constDecl "CYCLOTOMIC_INDEX" (.i64) (some (.identifier "CS_LEVEL"))],
  functions := [{ name := "fibonacci_fusion_matrix", params := [], ret := (some (.struct "FusionMatrix")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def queen_task_analysis_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("", ("math::sacred_physics", "")), ("Trit", ("base::types", "Trit"))],
  hostOnly := [],
  reachable := []
}

def queen_task_analysis_module : Module := {
  name := "queen_task_analysis",
  imports := [{ path := "base::types::Trit", items := ["Trit"] }, { path := "math::sacred_physics::", items := [""] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def runtime_instance_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("runtime", ("runtime", "runtime")), ("std", ("std", "std"))],
  hostOnly := [],
  reachable := []
}

def runtime_instance_module : Module := {
  name := "runtime_instance",
  imports := [{ path := "std", items := ["std"] }, { path := "runtime", items := ["runtime"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def sacred_cosmology_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def sacred_cosmology_module : Module := {
  name := "sacred_cosmology",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_dark_matter_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_dark_matter_module : Module := {
  name := "sacred_dark_matter",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_gravity_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def sacred_gravity_module : Module := {
  name := "sacred_gravity",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_monopoles_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def sacred_monopoles_module : Module := {
  name := "sacred_monopoles",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_quantum_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_quantum_module : Module := {
  name := "sacred_quantum",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_quantum_gravity_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_quantum_gravity_module : Module := {
  name := "sacred_quantum_gravity",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sacred_sacred_constants_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_sacred_constants_module : Module := {
  name := "sacred_sacred_constants",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def sacred_sacred_governance_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_sacred_governance_module : Module := {
  name := "sacred_sacred_governance",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def sacred_sacred_identity_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def sacred_sacred_identity_module : Module := {
  name := "sacred_sacred_identity",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "SACRED_MATH" (.u32) (some (.intLit (0)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def sacred_superconductivity_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def sacred_superconductivity_module : Module := {
  name := "sacred_superconductivity",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def sync_index_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("sync", ("sync", "sync")), ("std", ("std", "std"))],
  hostOnly := [],
  reachable := []
}

def sync_index_module : Module := {
  name := "sync_index",
  imports := [{ path := "std", items := ["std"] }, { path := "sync", items := ["sync"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_agent_agent_run_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_agent_run_module : Module := {
  name := "tri_agent_agent_run",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_agents_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_agents_module : Module := {
  name := "tri_agent_agents",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_autonomous_lifecycle_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_autonomous_lifecycle_module : Module := {
  name := "tri_agent_autonomous_lifecycle",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_agent_autonomous_universe_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_autonomous_universe_module : Module := {
  name := "tri_agent_autonomous_universe",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_eternal_monitor_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_eternal_monitor_module : Module := {
  name := "tri_agent_eternal_monitor",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_experience_hooks_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_experience_hooks_module : Module := {
  name := "tri_agent_experience_hooks",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_faculty_board_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_faculty_board_module : Module := {
  name := "tri_agent_faculty_board",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_governance_agent_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_governance_agent_module : Module := {
  name := "tri_agent_governance_agent",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_agent_handoff_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_handoff_module : Module := {
  name := "tri_agent_handoff",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_agent_memory_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_memory_module : Module := {
  name := "tri_agent_memory",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_agent_swarm_agents_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_agent_swarm_agents_module : Module := {
  name := "tri_agent_swarm_agents",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_collections_bitmap_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_bitmap_module : Module := {
  name := "tri_collections_bitmap",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_bitset_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_bitset_module : Module := {
  name := "tri_collections_bitset",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_bitvector_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_bitvector_module : Module := {
  name := "tri_collections_bitvector",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_btree_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_btree_module : Module := {
  name := "tri_collections_btree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "BTree" (.u32) none, .constDecl "BTreeNode" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_circular_buffer_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_circular_buffer_module : Module := {
  name := "tri_collections_circular_buffer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_context_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_context_module : Module := {
  name := "tri_collections_context",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_deque_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_deque_module : Module := {
  name := "tri_collections_deque",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_either_env : Env := {
  structs := [("L", [("value", .u32)]), ("R", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_either_module : Module := {
  name := "tri_collections_either",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Either" (.u32) none],
  functions := [{ name := "left", params := [("value", (.struct "L"))], ret := (some (.struct "void")), body := [] }, { name := "right", params := [("value", (.struct "R"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_interval_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_interval_module : Module := {
  name := "tri_collections_interval",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_linked_list_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("init", "LinkedList")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_linked_list_module : Module := {
  name := "tri_collections_linked_list",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [], ret := (some (.struct "LinkedList")), body := [] }, { name := "append", params := [("list", (.struct "*LinkedList"))], ret := (some (.struct "void")), body := [] }, { name := "prepend", params := [("list", (.struct "*LinkedList"))], ret := (some (.struct "void")), body := [] }, { name := "remove", params := [("list", (.struct "*LinkedList"))], ret := (some (.struct "void")), body := [] }, { name := "deinit", params := [("list", (.struct "*LinkedList"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "append_basic_case", params := [], ret := none, body := [] }, { name := "prepend_basic_case", params := [], ret := none, body := [] }, { name := "remove_basic_case", params := [], ret := none, body := [] }, { name := "deinit_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_collections_list_env : Env := {
  structs := [("T", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_list_module : Module := {
  name := "tri_collections_list",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "List" (.u32) none],
  functions := [{ name := "cons", params := [("head", (.struct "T"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_lockfree_stack_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("init", "LockFreeStack")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_lockfree_stack_module : Module := {
  name := "tri_collections_lockfree_stack",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [], ret := (some (.struct "LockFreeStack")), body := [] }, { name := "push", params := [("s", (.struct "*LockFreeStack"))], ret := (some (.struct "void")), body := [] }, { name := "pop", params := [("s", (.struct "*LockFreeStack"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "push_basic_case", params := [], ret := none, body := [] }, { name := "pop_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_collections_lru_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_lru_module : Module := {
  name := "tri_collections_lru",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "LRU" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_lru_cache_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_lru_cache_module : Module := {
  name := "tri_collections_lru_cache",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [("capacity", (.struct "usize"))], ret := (some (.struct "void")), body := [] }, { name := "get", params := [("cache", (.struct "*LRUCache"))], ret := (some (.struct "void")), body := [] }, { name := "put", params := [("cache", (.struct "*LRUCache"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "get_basic_case", params := [], ret := none, body := [] }, { name := "put_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_collections_map_env : Env := {
  structs := [("K", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_map_module : Module := {
  name := "tri_collections_map",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Map" (.u32) none],
  functions := [{ name := "singleton", params := [("key", (.struct "K"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_namespace_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_namespace_module : Module := {
  name := "tri_collections_namespace",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_collections_option_env : Env := {
  structs := [("T", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_option_module : Module := {
  name := "tri_collections_option",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Option" (.u32) none],
  functions := [{ name := "some", params := [("value", (.struct "T"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_priority_queue_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_priority_queue_module : Module := {
  name := "tri_collections_priority_queue",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_queue_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_queue_module : Module := {
  name := "tri_collections_queue",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Queue" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_result_env : Env := {
  structs := [("E", [("value", .u32)]), ("T", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_result_module : Module := {
  name := "tri_collections_result",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Result" (.u32) none],
  functions := [{ name := "ok", params := [("value", (.struct "T"))], ret := (some (.struct "void")), body := [] }, { name := "err", params := [("error", (.struct "E"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_ring_buffer_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_ring_buffer_module : Module := {
  name := "tri_collections_ring_buffer",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Ring" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_skip_list_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_skip_list_module : Module := {
  name := "tri_collections_skip_list",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "SkipNode" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_stack_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_stack_module : Module := {
  name := "tri_collections_stack",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Stack" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_collections_tuple_env : Env := {
  structs := [("A", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_tuple_module : Module := {
  name := "tri_collections_tuple",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Tuple2" (.u32) none, .constDecl "Tuple3" (.u32) none],
  functions := [{ name := "pair", params := [("a", (.struct "A"))], ret := (some (.struct "void")), body := [] }, { name := "triple", params := [("a", (.struct "A"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_collections_variant_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_collections_variant_module : Module := {
  name := "tri_collections_variant",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Variant" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_crypto_base32_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_base32_module : Module := {
  name := "tri_crypto_base32",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_crypto_base64_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_base64_module : Module := {
  name := "tri_crypto_base64",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_crypto_crypto_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_crypto_module : Module := {
  name := "tri_crypto_crypto",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "generate_key_pair", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "sha256", params := [("data", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "hmac", params := [("key", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "generate_key_pair_basic_case", params := [], ret := none, body := [] }, { name := "sha256_basic_case", params := [], ret := none, body := [] }, { name := "hmac_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_crypto_ecc_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_ecc_module : Module := {
  name := "tri_crypto_ecc",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "add", params := [("curve", (.struct "*EllipticCurve"))], ret := (some (.struct "void")), body := [] }, { name := "multiply", params := [("curve", (.struct "*EllipticCurve"))], ret := (some (.struct "void")), body := [] }, { name := "is_on_curve", params := [("curve", (.struct "*EllipticCurve"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "add_basic_case", params := [], ret := none, body := [] }, { name := "multiply_basic_case", params := [], ret := none, body := [] }, { name := "is_on_curve_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_crypto_hmac_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_hmac_module : Module := {
  name := "tri_crypto_hmac",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_crypto_reed_solomon_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_reed_solomon_module : Module := {
  name := "tri_crypto_reed_solomon",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "encode", params := [("data", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "decode", params := [("shards", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "encode_basic_case", params := [], ret := none, body := [] }, { name := "decode_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_crypto_rsa_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_crypto_rsa_module : Module := {
  name := "tri_crypto_rsa",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "generate", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "encrypt", params := [("message", (.u64))], ret := (some (.struct "void")), body := [] }, { name := "decrypt", params := [("ciphertext", (.u64))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "generate_basic_case", params := [], ret := none, body := [] }, { name := "encrypt_basic_case", params := [], ret := none, body := [] }, { name := "decrypt_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_encoding_bson_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_bson_module : Module := {
  name := "tri_encoding_bson",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("data", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "serialize", params := [("doc", (.struct "BsonDocument"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "serialize_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_encoding_csv_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_csv_module : Module := {
  name := "tri_encoding_csv",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("text", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "get", params := [("doc", (.struct "CsvDocument"))], ret := (some (.struct "void")), body := [] }, { name := "set", params := [("doc", (.struct "*CsvDocument"))], ret := (some (.struct "void")), body := [] }, { name := "serialize", params := [("doc", (.struct "CsvDocument"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "get_basic_case", params := [], ret := none, body := [] }, { name := "set_basic_case", params := [], ret := none, body := [] }, { name := "serialize_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_encoding_html_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_html_module : Module := {
  name := "tri_encoding_html",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("html", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "query_selector", params := [("node", (.struct "HtmlNode"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "query_selector_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_encoding_json_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_json_module : Module := {
  name := "tri_encoding_json",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_encoding_markup_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_markup_module : Module := {
  name := "tri_encoding_markup",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_encoding_mime_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_mime_module : Module := {
  name := "tri_encoding_mime",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_encoding_msgpack_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_msgpack_module : Module := {
  name := "tri_encoding_msgpack",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_encoding_xml_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_encoding_xml_module : Module := {
  name := "tri_encoding_xml",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("text", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "format", params := [("node", (.struct "XmlNode"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "format_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_graph_dijkstra_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_dijkstra_module : Module := {
  name := "tri_graph_dijkstra",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "shortest_path", params := [("graph", (.struct "*Graph"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "shortest_path_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_graph_graph_env : Env := {
  structs := [("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_graph_module : Module := {
  name := "tri_graph_graph",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Graph" (.u32) none],
  functions := [{ name := "empty", params := [("directed", (.bool))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_graph_graph_bfs_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_graph_bfs_module : Module := {
  name := "tri_graph_graph_bfs",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "add_edge", params := [("graph", (.struct "*Graph"))], ret := (some (.struct "void")), body := [] }, { name := "traverse", params := [("graph", (.struct "*Graph"))], ret := (some (.struct "void")), body := [] }, { name := "deinit", params := [("graph", (.struct "*Graph"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "add_edge_basic_case", params := [], ret := none, body := [] }, { name := "traverse_basic_case", params := [], ret := none, body := [] }, { name := "deinit_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_graph_graph_dfs_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_graph_dfs_module : Module := {
  name := "tri_graph_graph_dfs",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "traverse", params := [("graph", (.struct "*Graph"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "traverse_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_graph_prims_mst_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_prims_mst_module : Module := {
  name := "tri_graph_prims_mst",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_graph_topological_sort_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_graph_topological_sort_module : Module := {
  name := "tri_graph_topological_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_io_compress_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_io_compress_module : Module := {
  name := "tri_io_compress",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_io_filesystem_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_io_filesystem_module : Module := {
  name := "tri_io_filesystem",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "join", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "basename", params := [("path", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "dirname", params := [("path", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "ext", params := [("path", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "has_ext", params := [("path", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "is_absolute", params := [("path", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "normalize", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "join_basic_case", params := [], ret := none, body := [] }, { name := "basename_basic_case", params := [], ret := none, body := [] }, { name := "dirname_basic_case", params := [], ret := none, body := [] }, { name := "ext_basic_case", params := [], ret := none, body := [] }, { name := "has_ext_basic_case", params := [], ret := none, body := [] }, { name := "is_absolute_basic_case", params := [], ret := none, body := [] }, { name := "normalize_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_io_fs_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_io_fs_module : Module := {
  name := "tri_io_fs",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "join", params := [("base", (.struct "Path"))], ret := (some (.struct "void")), body := [] }, { name := "basename", params := [("path", (.struct "Path"))], ret := (some (.struct "void")), body := [] }, { name := "dirname", params := [("path", (.struct "Path"))], ret := (some (.struct "void")), body := [] }, { name := "extension", params := [("path", (.struct "Path"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "join_basic_case", params := [], ret := none, body := [] }, { name := "basename_basic_case", params := [], ret := none, body := [] }, { name := "dirname_basic_case", params := [], ret := none, body := [] }, { name := "extension_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_io_zip_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_io_zip_module : Module := {
  name := "tri_io_zip",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Zipper" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_math_bezier_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_math_bezier_module : Module := {
  name := "tri_math_bezier",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_math_constants_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("get_sacred_constants", "SacredConstants"), ("get_system_limits", "SystemLimits")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_constants_module : Module := {
  name := "tri_math_constants",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "max_path_len", params := [], ret := (some (.struct "usize")), body := [] }, { name := "max_line_len", params := [], ret := (some (.struct "usize")), body := [] }, { name := "max_args", params := [], ret := (some (.struct "usize")), body := [] }, { name := "max_env_vars", params := [], ret := (some (.struct "usize")), body := [] }, { name := "get_p_h_i", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_p_i", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_e", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_s_q_r_t2", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_s_q_r_t3", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_golden_ratio", params := [], ret := (some (.struct "f64")), body := [] }, { name := "get_system_limits", params := [], ret := (some (.struct "SystemLimits")), body := [] }, { name := "get_sacred_constants", params := [], ret := (some (.struct "SacredConstants")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "max_path_len_basic_case", params := [], ret := none, body := [] }, { name := "max_line_len_basic_case", params := [], ret := none, body := [] }, { name := "max_args_basic_case", params := [], ret := none, body := [] }, { name := "max_env_vars_basic_case", params := [], ret := none, body := [] }, { name := "get_p_h_i_basic_case", params := [], ret := none, body := [] }, { name := "get_p_i_basic_case", params := [], ret := none, body := [] }, { name := "get_e_basic_case", params := [], ret := none, body := [] }, { name := "get_s_q_r_t2_basic_case", params := [], ret := none, body := [] }, { name := "get_s_q_r_t3_basic_case", params := [], ret := none, body := [] }, { name := "get_golden_ratio_basic_case", params := [], ret := none, body := [] }, { name := "get_system_limits_basic_case", params := [], ret := none, body := [] }, { name := "get_sacred_constants_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_math_math_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_math_module : Module := {
  name := "tri_math_math",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_math_matrix_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_math_matrix_module : Module := {
  name := "tri_math_matrix",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_math_measurement_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_measurement_module : Module := {
  name := "tri_math_measurement",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_math_polynomial_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_polynomial_module : Module := {
  name := "tri_math_polynomial",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_math_probability_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_probability_module : Module := {
  name := "tri_math_probability",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "bernoulli", params := [("p", (.struct "f64"))], ret := (some (.struct "void")), body := [] }, { name := "binomial", params := [("n", (.struct "usize"))], ret := (some (.struct "void")), body := [] }, { name := "poisson", params := [("lambda", (.struct "f64"))], ret := (some (.struct "void")), body := [] }, { name := "normal", params := [("mean", (.struct "f64"))], ret := (some (.struct "void")), body := [] }, { name := "exponential", params := [("lambda", (.struct "f64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "bernoulli_basic_case", params := [], ret := none, body := [] }, { name := "binomial_basic_case", params := [], ret := none, body := [] }, { name := "poisson_basic_case", params := [], ret := none, body := [] }, { name := "normal_basic_case", params := [], ret := none, body := [] }, { name := "exponential_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_math_statistics_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_math_statistics_module : Module := {
  name := "tri_math_statistics",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "mean", params := [("values", (.struct "[]f64"))], ret := (some (.struct "void")), body := [] }, { name := "variance", params := [("values", (.struct "[]f64"))], ret := (some (.struct "void")), body := [] }, { name := "std_dev", params := [("values", (.struct "[]f64"))], ret := (some (.struct "void")), body := [] }, { name := "median", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "percentile", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "correlation", params := [("x", (.struct "[]f64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "mean_basic_case", params := [], ret := none, body := [] }, { name := "variance_basic_case", params := [], ret := none, body := [] }, { name := "std_dev_basic_case", params := [], ret := none, body := [] }, { name := "median_basic_case", params := [], ret := none, body := [] }, { name := "percentile_basic_case", params := [], ret := none, body := [] }, { name := "correlation_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_net_async_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_net_async_module : Module := {
  name := "tri_net_async",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Future" (.u32) none, .constDecl "Promise" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_net_async_stream_env : Env := {
  structs := [("[]T", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_async_stream_module : Module := {
  name := "tri_net_async_stream",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Stream" (.u32) none],
  functions := [{ name := "from", params := [("items", (.struct "[]T"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_net_channel_env : Env := {
  structs := [("usize", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_channel_module : Module := {
  name := "tri_net_channel",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Channel" (.u32) none],
  functions := [{ name := "new_channel", params := [("capacity", (.struct "usize"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_net_cloud_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_cloud_module : Module := {
  name := "tri_net_cloud",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_net_http_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_http_module : Module := {
  name := "tri_net_http",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "method_to_string", params := [("method", (.struct "HttpMethod"))], ret := (some (.struct "void")), body := [] }, { name := "status_from_code", params := [("code", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "is_success", params := [("code", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "is_redirect", params := [("code", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "is_client_error", params := [("code", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "is_server_error", params := [("code", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "parse_url", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "method_to_string_basic_case", params := [], ret := none, body := [] }, { name := "status_from_code_basic_case", params := [], ret := none, body := [] }, { name := "is_success_basic_case", params := [], ret := none, body := [] }, { name := "is_redirect_basic_case", params := [], ret := none, body := [] }, { name := "is_client_error_basic_case", params := [], ret := none, body := [] }, { name := "is_server_error_basic_case", params := [], ret := none, body := [] }, { name := "parse_url_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_net_net_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_net_module : Module := {
  name := "tri_net_net",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse_ip", params := [("addr", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "is_localhost", params := [("addr", (.struct "IpAddress"))], ret := (some (.struct "void")), body := [] }, { name := "is_valid_port", params := [("port", (.u16))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_ip_basic_case", params := [], ret := none, body := [] }, { name := "is_localhost_basic_case", params := [], ret := none, body := [] }, { name := "is_valid_port_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_net_url_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_net_url_module : Module := {
  name := "tri_net_url",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_batch_runner_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_batch_runner_module : Module := {
  name := "tri_pipeline_batch_runner",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_pipeline_builder_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_builder_module : Module := {
  name := "tri_pipeline_builder",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Builder" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_cloud_orchestrator_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_cloud_orchestrator_module : Module := {
  name := "tri_pipeline_cloud_orchestrator",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_codegen_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_codegen_module : Module := {
  name := "tri_pipeline_codegen",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_pipeline_pipeline_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_pipeline_module : Module := {
  name := "tri_pipeline_pipeline",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_pipeline_parallel_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_pipeline_parallel_module : Module := {
  name := "tri_pipeline_pipeline_parallel",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_pipeline_spec_parser_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_spec_parser_module : Module := {
  name := "tri_pipeline_spec_parser",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_workflow_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_workflow_module : Module := {
  name := "tri_pipeline_workflow",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_pipeline_workflow_executor_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_workflow_executor_module : Module := {
  name := "tri_pipeline_workflow_executor",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_pipeline_workflow_parser_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_pipeline_workflow_parser_module : Module := {
  name := "tri_pipeline_workflow_parser",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_search_aho_corasick_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_search_aho_corasick_module : Module := {
  name := "tri_search_aho_corasick",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_search_bloom_filter_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_bloom_filter_module : Module := {
  name := "tri_search_bloom_filter",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_search_boyer_moore_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_boyer_moore_module : Module := {
  name := "tri_search_boyer_moore",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_search_knuth_morris_pratt_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_knuth_morris_pratt_module : Module := {
  name := "tri_search_knuth_morris_pratt",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "build_prefix", params := [("pattern", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "search", params := [("text", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "build_prefix_basic_case", params := [], ret := none, body := [] }, { name := "search_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_search_match_env : Env := {
  structs := [("[][]const u8", [("value", .u32)]), ("[]const u8", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_search_match_module : Module := {
  name := "tri_search_match",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "match_literal", params := [("input", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "match_type", params := [("type_name", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "exhaustive", params := [("cases", (.struct "[][]const u8"))], ret := (some (.struct "void")), body := [] }],
  tests := [{ name := "match_literal_basic_case", params := [], ret := none, body := [] }, { name := "match_type_basic_case", params := [], ret := none, body := [] }, { name := "exhaustive_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_search_pattern_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_pattern_module : Module := {
  name := "tri_search_pattern",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "glob_match", params := [("pattern", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "wildcard_match", params := [("pattern", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "glob_match_basic_case", params := [], ret := none, body := [] }, { name := "wildcard_match_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_search_rabin_karp_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_rabin_karp_module : Module := {
  name := "tri_search_rabin_karp",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [("pattern", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "search", params := [("state", (.struct "*RKState"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "search_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_search_regex_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_regex_module : Module := {
  name := "tri_search_regex",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_search_regex_advanced_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_regex_advanced_module : Module := {
  name := "tri_search_regex_advanced",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "compile", params := [("pattern", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "match", params := [("regex", (.struct "Regex"))], ret := (some (.struct "void")), body := [] }, { name := "replace", params := [("regex", (.struct "Regex"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "compile_basic_case", params := [], ret := none, body := [] }, { name := "match_basic_case", params := [], ret := none, body := [] }, { name := "replace_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_search_search_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_search_search_module : Module := {
  name := "tri_search_search",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "binary", params := [("sorted", (.struct "[]const T"))], ret := (some (.struct "void")), body := [] }, { name := "linear", params := [("items", (.struct "[]const T"))], ret := (some (.struct "void")), body := [] }, { name := "lower_bound", params := [("sorted", (.struct "[]const T"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "binary_basic_case", params := [], ret := none, body := [] }, { name := "linear_basic_case", params := [], ret := none, body := [] }, { name := "lower_bound_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_counting_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_counting_sort_module : Module := {
  name := "tri_sort_counting_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_heap_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_heap_sort_module : Module := {
  name := "tri_sort_heap_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_insertion_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_insertion_sort_module : Module := {
  name := "tri_sort_insertion_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_merge_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_merge_sort_module : Module := {
  name := "tri_sort_merge_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "sort_in_place", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }, { name := "sort_in_place_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_quick_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_quick_sort_module : Module := {
  name := "tri_sort_quick_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "sort_range", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }, { name := "sort_range_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_radix_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_radix_sort_module : Module := {
  name := "tri_sort_radix_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "sort_in_place", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }, { name := "sort_in_place_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_selection_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_selection_sort_module : Module := {
  name := "tri_sort_selection_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_shell_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_shell_sort_module : Module := {
  name := "tri_sort_shell_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("values", (.struct "[]i64"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_sort_module : Module := {
  name := "tri_sort_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("items", (.struct "[]const T"))], ret := (some (.struct "void")), body := [] }, { name := "sort_by", params := [("items", (.struct "[]const T"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }, { name := "sort_by_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_sort_tim_sort_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_sort_tim_sort_module : Module := {
  name := "tri_sort_tim_sort",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "sort", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "sort_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_trees_avl_tree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("init", "AVLTree")],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_avl_tree_module : Module := {
  name := "tri_trees_avl_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [], ret := (some (.struct "AVLTree")), body := [] }, { name := "insert", params := [("tree", (.struct "*AVLTree"))], ret := (some (.struct "void")), body := [] }, { name := "find", params := [("tree", (.struct "*const AVLTree"))], ret := (some (.struct "void")), body := [] }, { name := "delete", params := [("tree", (.struct "*AVLTree"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "insert_basic_case", params := [], ret := none, body := [] }, { name := "find_basic_case", params := [], ret := none, body := [] }, { name := "delete_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_trees_b_tree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_b_tree_module : Module := {
  name := "tri_trees_b_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "search", params := [("tree", (.struct "*BTree"))], ret := (some (.struct "void")), body := [] }, { name := "insert", params := [("tree", (.struct "*BTree"))], ret := (some (.struct "void")), body := [] }, { name := "deinit", params := [("tree", (.struct "*BTree"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "search_basic_case", params := [], ret := none, body := [] }, { name := "insert_basic_case", params := [], ret := none, body := [] }, { name := "deinit_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_trees_fenwick_tree_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_fenwick_tree_module : Module := {
  name := "tri_trees_fenwick_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_trees_kd_tree_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_kd_tree_module : Module := {
  name := "tri_trees_kd_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_trees_octree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_octree_module : Module := {
  name := "tri_trees_octree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_trees_quadtree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_quadtree_module : Module := {
  name := "tri_trees_quadtree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_trees_red_black_tree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("init", "RBTree")],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_red_black_tree_module : Module := {
  name := "tri_trees_red_black_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [], ret := (some (.struct "RBTree")), body := [] }, { name := "insert", params := [("tree", (.struct "*RBTree"))], ret := (some (.struct "void")), body := [] }, { name := "find", params := [("tree", (.struct "*const RBTree"))], ret := (some (.struct "void")), body := [] }, { name := "delete", params := [("tree", (.struct "*RBTree"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "insert_basic_case", params := [], ret := none, body := [] }, { name := "find_basic_case", params := [], ret := none, body := [] }, { name := "delete_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_trees_rtree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_rtree_module : Module := {
  name := "tri_trees_rtree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_trees_segment_tree_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_segment_tree_module : Module := {
  name := "tri_trees_segment_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_trees_splay_tree_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("init", "SplayTree")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_splay_tree_module : Module := {
  name := "tri_trees_splay_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [], ret := (some (.struct "SplayTree")), body := [] }, { name := "find", params := [("tree", (.struct "*SplayTree"))], ret := (some (.struct "void")), body := [] }, { name := "insert", params := [("tree", (.struct "*SplayTree"))], ret := (some (.struct "void")), body := [] }, { name := "delete", params := [("tree", (.struct "*SplayTree"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "find_basic_case", params := [], ret := none, body := [] }, { name := "insert_basic_case", params := [], ret := none, body := [] }, { name := "delete_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_trees_suffix_array_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_suffix_array_module : Module := {
  name := "tri_trees_suffix_array",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_trees_tree_env : Env := {
  structs := [("T", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_tree_module : Module := {
  name := "tri_trees_tree",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "Tree" (.u32) none],
  functions := [{ name := "leaf", params := [("value", (.struct "T"))], ret := (some (.struct "void")), body := [] }],
  tests := [],
  benches := []
}

def tri_trees_trie_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_trees_trie_module : Module := {
  name := "tri_trees_trie",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [.constDecl "TrieNode" (.u32) none, .constDecl "Trie" (.u32) none],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_args_env : Env := {
  structs := [("ParseResult", [("value", .u32)]), ("Std", [("value", .u32)]), ("void", [("value", .u32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_args_module : Module := {
  name := "tri_utils_args",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "has_flag", params := [("result", (.struct "ParseResult"))], ret := (some (.struct "void")), body := [] }, { name := "get_value", params := [("result", (.struct "ParseResult"))], ret := (some (.struct "void")), body := [] }, { name := "get_positional", params := [("result", (.struct "ParseResult"))], ret := (some (.struct "void")), body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "has_flag_basic_case", params := [], ret := none, body := [] }, { name := "get_value_basic_case", params := [], ret := none, body := [] }, { name := "get_positional_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_arrow_time_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_arrow_time_module : Module := {
  name := "tri_utils_arrow_time",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_bytes_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_bytes_module : Module := {
  name := "tri_utils_bytes",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_color_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_color_module : Module := {
  name := "tri_utils_color",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "rgb", params := [("r", (.u8))], ret := (some (.struct "void")), body := [] }, { name := "to_hex", params := [("color", (.struct "Color"))], ret := (some (.struct "void")), body := [] }, { name := "blend", params := [("a", (.struct "Color"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "rgb_basic_case", params := [], ret := none, body := [] }, { name := "to_hex_basic_case", params := [], ret := none, body := [] }, { name := "blend_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_colors_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_colors_module : Module := {
  name := "tri_utils_colors",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_config_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_config_module : Module := {
  name := "tri_utils_config",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "get_string", params := [("config", (.struct "Config"))], ret := (some (.struct "void")), body := [] }, { name := "get_number", params := [("config", (.struct "Config"))], ret := (some (.struct "void")), body := [] }, { name := "get_bool", params := [("config", (.struct "Config"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "get_string_basic_case", params := [], ret := none, body := [] }, { name := "get_number_basic_case", params := [], ret := none, body := [] }, { name := "get_bool_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_error_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_error_module : Module := {
  name := "tri_utils_error",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_exit_codes_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_exit_codes_module : Module := {
  name := "tri_utils_exit_codes",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_utils_help_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_help_module : Module := {
  name := "tri_utils_help",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_utils_logging_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_logging_module : Module := {
  name := "tri_utils_logging",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "level_to_string", params := [("level", (.struct "LogLevel"))], ret := (some (.struct "void")), body := [] }, { name := "level_from_string", params := [("s", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "level_color", params := [("level", (.struct "LogLevel"))], ret := (some (.struct "void")), body := [] }, { name := "format_entry", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "should_log", params := [("msg_level", (.struct "LogLevel"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "level_to_string_basic_case", params := [], ret := none, body := [] }, { name := "level_from_string_basic_case", params := [], ret := none, body := [] }, { name := "level_color_basic_case", params := [], ret := none, body := [] }, { name := "format_entry_basic_case", params := [], ret := none, body := [] }, { name := "should_log_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_random_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_random_module : Module := {
  name := "tri_utils_random",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "init", params := [("seed", (.u64))], ret := (some (.struct "void")), body := [] }, { name := "next", params := [("rng", (.struct "*Rng"))], ret := (some (.struct "void")), body := [] }, { name := "range", params := [("rng", (.struct "*Rng"))], ret := (some (.struct "void")), body := [] }, { name := "range_inclusive", params := [("rng", (.struct "*Rng"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "init_basic_case", params := [], ret := none, body := [] }, { name := "next_basic_case", params := [], ret := none, body := [] }, { name := "range_basic_case", params := [], ret := none, body := [] }, { name := "range_inclusive_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_string_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_string_module : Module := {
  name := "tri_utils_string",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_template_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_template_module : Module := {
  name := "tri_utils_template",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [],
  tests := [],
  benches := []
}

def tri_utils_terminal_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_terminal_module : Module := {
  name := "tri_utils_terminal",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "get_size", params := [], ret := (some (.struct "TerminalSize")), body := [] }, { name := "colorize", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "reset", params := [], ret := (some (.struct "[]const u8")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "get_size_basic_case", params := [], ret := none, body := [] }, { name := "colorize_basic_case", params := [], ret := none, body := [] }, { name := "reset_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_text_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_text_module : Module := {
  name := "tri_utils_text",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "word_wrap", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "count_words", params := [("text", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "count_lines", params := [("text", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "indent", params := [("allocator", (.struct "Std")), ("mem", (.u32)), ("Allocator", (.u32))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "word_wrap_basic_case", params := [], ret := none, body := [] }, { name := "count_words_basic_case", params := [], ret := none, body := [] }, { name := "count_lines_basic_case", params := [], ret := none, body := [] }, { name := "indent_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_time_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [("now", "Instant")],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_time_module : Module := {
  name := "tri_utils_time",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "now", params := [], ret := (some (.struct "Instant")), body := [] }, { name := "since_epoch", params := [("instant", (.struct "Instant"))], ret := (some (.struct "void")), body := [] }, { name := "add", params := [("instant", (.struct "Instant"))], ret := (some (.struct "void")), body := [] }, { name := "sub", params := [("a", (.struct "Instant"))], ret := (some (.struct "void")), body := [] }, { name := "format", params := [("instant", (.struct "Instant"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "now_basic_case", params := [], ret := none, body := [] }, { name := "since_epoch_basic_case", params := [], ret := none, body := [] }, { name := "add_basic_case", params := [], ret := none, body := [] }, { name := "sub_basic_case", params := [], ret := none, body := [] }, { name := "format_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def tri_utils_utf8_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("types", ("base::types", "types")), ("constants", ("math::constants", "constants"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_utf8_module : Module := {
  name := "tri_utils_utf8",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [],
  benches := []
}

def tri_utils_version_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("constants", ("math::constants", "constants")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def tri_utils_version_module : Module := {
  name := "tri_utils_version",
  imports := [{ path := "base::types", items := ["types"] }, { path := "math::constants", items := ["constants"] }],
  globals := [],
  functions := [{ name := "parse", params := [("version_str", (.struct "[]const u8"))], ret := (some (.struct "void")), body := [] }, { name := "compare", params := [("a", (.struct "Version"))], ret := (some (.struct "void")), body := [] }, { name := "next", params := [("version", (.struct "Version"))], ret := (some (.struct "void")), body := [] }, { name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "parse_basic_case", params := [], ret := none, body := [] }, { name := "compare_basic_case", params := [], ret := none, body := [] }, { name := "next_basic_case", params := [], ret := none, body := [] }],
  benches := []
}

def vm_jit_semantics_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := ["VsaOperation"],
  imports := [("types", ("base::types", "types")), ("hybrid_arithmetic", ("ternary::hybrid_arithmetic", "hybrid_arithmetic")), ("gf16", ("numeric::gf16", "gf16"))],
  hostOnly := [],
  reachable := []
}

def vm_jit_semantics_module : Module := {
  name := "vm_jit_semantics",
  imports := [{ path := "base::types", items := ["types"] }, { path := "ternary::hybrid_arithmetic", items := ["hybrid_arithmetic"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "JitVsaFn" (.u32) none, .constDecl "JitSimilarityFn" (.u32) none],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "jit_compiler_init_creates_valid_compiler", params := [], ret := none, body := [] }, { name := "jit_compiler_reset_clears_buffer", params := [], ret := none, body := [] }, { name := "jit_compile_operation_generates_code", params := [], ret := none, body := [] }, { name := "jit_finalize_returns_function_ptr", params := [], ret := none, body := [] }, { name := "jit_cache_init_creates_valid_cache", params := [], ret := none, body := [] }, { name := "jit_cache_deinit_clears_resources", params := [], ret := none, body := [] }, { name := "jit_cache_hit_returns_cached_function", params := [], ret := none, body := [] }, { name := "jit_cache_miss_compiles_new_function", params := [], ret := none, body := [] }, { name := "jit_dot_product_returns_f64", params := [], ret := none, body := [] }],
  benches := [{ name := "jit_compile_bind_latency", params := [], ret := none, body := [] }, { name := "jit_compile_bundle_latency", params := [], ret := none, body := [] }, { name := "jit_compile_dot_product_latency", params := [], ret := none, body := [] }, { name := "jit_cache_get_hit_latency", params := [], ret := none, body := [] }, { name := "jit_cache_get_miss_latency", params := [], ret := none, body := [] }, { name := "jit_finalize_latency", params := [], ret := none, body := [] }, { name := "jit_bind_execution_latency", params := [], ret := none, body := [] }, { name := "jit_dot_product_execution_latency", params := [], ret := none, body := [] }]
}

def vsa_packed_vsa_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("packed_trit", ("ternary::packed_trit", "packed_trit")), ("gf16", ("numeric::gf16", "gf16")), ("types", ("base::types", "types")), ("hybrid_arithmetic", ("ternary::hybrid_arithmetic", "hybrid_arithmetic"))],
  hostOnly := [],
  reachable := []
}

def vsa_packed_vsa_module : Module := {
  name := "vsa_packed_vsa",
  imports := [{ path := "base::types", items := ["types"] }, { path := "ternary::packed_trit", items := ["packed_trit"] }, { path := "ternary::hybrid_arithmetic", items := ["hybrid_arithmetic"] }, { path := "numeric::gf16", items := ["gf16"] }],
  globals := [.constDecl "TRITS_PER_BYTE" (.u8) (some (.intLit (5))), .constDecl "MAX_PACKED_BYTES" (.u16) (some (.intLit (2400))), .constDecl "LUT_DIM" (.u16) (some (.intLit (243))), .constDecl "LUT_SIZE" (.u32) (some (.intLit (59049))), .constDecl "BindLookupTable" (.array 59049 (.u8)) none, .constDecl "BundleLookupTable" (.array 59049 (.u8)) none, .constDecl "DotLookupTable" (.array 59049 (.u8)) none],
  functions := [],
  tests := [{ name := "packed_bind_matches_unpacked", params := [], ret := none, body := [] }, { name := "packed_bundle_matches_unpacked", params := [], ret := none, body := [] }, { name := "packed_dot_matches_unpacked", params := [], ret := none, body := [] }, { name := "packed_unbind_recovers_original", params := [], ret := none, body := [] }, { name := "packed_cosine_self_similarity_is_one", params := [], ret := none, body := [] }, { name := "packed_cosine_orthogonal_is_zero", params := [], ret := none, body := [] }, { name := "from_hybrid_preserves_data", params := [], ret := none, body := [] }, { name := "to_hybrid_preserves_data", params := [], ret := none, body := [] }],
  benches := [{ name := "packed_bind_latency_100_trits", params := [], ret := none, body := [] }, { name := "packed_bundle_latency_100_trits", params := [], ret := none, body := [] }, { name := "packed_dot_latency_100_trits", params := [], ret := none, body := [] }, { name := "packed_cosine_similarity_latency_100_trits", params := [], ret := none, body := [] }, { name := "from_hybrid_latency_100_trits", params := [], ret := none, body := [] }, { name := "to_hybrid_latency_100_trits", params := [], ret := none, body := [] }, { name := "random_packed_vector_latency_1000_trits", params := [], ret := none, body := [] }, { name := "packed_unbind_latency_100_trits", params := [], ret := none, body := [] }]
}

def vsa_sequence_hdc_env : Env := {
  structs := [("w537_non_lowerable_marker", [("dummy", .f32)])],
  constructors := [],
  enums := [],
  imports := [("gf16", ("numeric::gf16", "gf16")), ("packed_trit", ("ternary::packed_trit", "packed_trit")), ("types", ("base::types", "types"))],
  hostOnly := [],
  reachable := []
}

def vsa_sequence_hdc_module : Module := {
  name := "vsa_sequence_hdc",
  imports := [{ path := "base::types", items := ["types"] }, { path := "numeric::gf16", items := ["gf16"] }, { path := "ternary::packed_trit", items := ["packed_trit"] }],
  globals := [.constDecl "NGRAM_ORDER" (.u16) (some (.intLit (3))), .constDecl "DEFAULT_DIM" (.u16) (some (.intLit (1000))), .constDecl "HEBDIAN_CHARS" (.struct "usize") (some (.intLit (95))), .constDecl "HEBDIAN_OFFSET" (.struct "usize") (some (.intLit (32)))],
  functions := [{ name := "_w537_non_lowerable_marker", params := [("dummy", (.struct "w537_non_lowerable_marker"))], ret := none, body := [] }],
  tests := [{ name := "item_memory_get_vector_creates_on_miss", params := [], ret := none, body := [] }, { name := "item_memory_get_vector_returns_existing", params := [], ret := none, body := [] }, { name := "ngram_encoder_creates_valid_encoding", params := [], ret := none, body := [] }, { name := "ngram_encoder_same_input_same_output", params := [], ret := none, body := [] }, { name := "ngram_encoder_different_input_different_output", params := [], ret := none, body := [] }, { name := "sequence_store_creates_entry", params := [], ret := none, body := [] }, { name := "sequence_query_finds_best_match", params := [], ret := none, body := [] }, { name := "detector_trains_on_samples", params := [], ret := none, body := [] }, { name := "detector_detects_correct_language", params := [], ret := none, body := [] }],
  benches := [{ name := "ngram_encode_latency_10_chars", params := [], ret := none, body := [] }, { name := "ngram_decode_latency_10_chars", params := [], ret := none, body := [] }, { name := "sequence_query_latency_100_entries", params := [], ret := none, body := [] }, { name := "detector_train_latency", params := [], ret := none, body := [] }, { name := "item_memory_cache_hit_latency", params := [], ret := none, body := [] }, { name := "item_memory_cache_miss_latency", params := [], ret := none, body := [] }]
}

/-- W535 positive corpus witness: a bounded `while` loop remains lowerable after
    tightening the predicate to reject `while (true)`. -/
def igla_w535_bounded_while_module_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["count_to"]
}

def igla_w535_bounded_while_module_module : Module := {
  name := "w535_bounded_while_module",
  imports := [],
  globals := [],
  functions := [{
    name := "count_to",
    params := [("n", .u32)],
    ret := some .u32,
    body := [
      .varDecl "i" .u32 (some (.intLit 0)),
      .varDecl "acc" .u32 (some (.intLit 0)),
      .whileLoop (.binop "<" (.identifier "i") (.identifier "n")) [
        .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.intLit 1)),
        .assign (.identifier "i") (.binop "+" (.identifier "i") (.intLit 1))
      ],
      .return_ (some (.identifier "acc"))
    ]
  }],
  tests := [{ name := "bounded_while_counts", params := [], ret := none, body := [
    .bareCall (.call "assert_eq" [.call "count_to" [.intLit 0], .intLit 0]),
    .bareCall (.call "assert_eq" [.call "count_to" [.intLit 3], .intLit 3]),
    .bareCall (.call "assert_eq" [.call "count_to" [.intLit 7], .intLit 7])
  ] }],
  benches := []
}

theorem api_sdk_contract_lowerable : Module.isLowerable api_sdk_contract_env api_sdk_contract_module = false := by native_decide
theorem api_tri_net_api_lowerable : Module.isLowerable api_tri_net_api_env api_tri_net_api_module = true := by native_decide
theorem ar_asp_solver_lowerable : Module.isLowerable ar_asp_solver_env ar_asp_solver_module = true := by native_decide
theorem ar_coa_planning_lowerable : Module.isLowerable ar_coa_planning_env ar_coa_planning_module = true := by native_decide
theorem ar_composition_lowerable : Module.isLowerable ar_composition_env ar_composition_module = true := by native_decide
theorem ar_datalog_engine_lowerable : Module.isLowerable ar_datalog_engine_env ar_datalog_engine_module = true := by native_decide
theorem ar_explainability_lowerable : Module.isLowerable ar_explainability_env ar_explainability_module = true := by native_decide
theorem ar_proof_trace_lowerable : Module.isLowerable ar_proof_trace_env ar_proof_trace_module = true := by native_decide
theorem ar_restraint_lowerable : Module.isLowerable ar_restraint_env ar_restraint_module = true := by native_decide
theorem ar_ternary_logic_lowerable : Module.isLowerable ar_ternary_logic_env ar_ternary_logic_module = true := by native_decide
theorem automation_wrapup_auto_lowerable : Module.isLowerable automation_wrapup_auto_env automation_wrapup_auto_module = true := by native_decide
theorem base_debounce_lowerable : Module.isLowerable base_debounce_env base_debounce_module = true := by native_decide
theorem base_ring_32_lowerable : Module.isLowerable base_ring_32_env base_ring_32_module = true := by native_decide
theorem base_ternary_encoding_lowerable : Module.isLowerable base_ternary_encoding_env base_ternary_encoding_module = false := by native_decide
theorem base_ternary_memory_lowerable : Module.isLowerable base_ternary_memory_env base_ternary_memory_module = false := by native_decide
theorem benchmarks_bench_main_lowerable : Module.isLowerable benchmarks_bench_main_env benchmarks_bench_main_module = true := by native_decide
theorem benchmarks_bench_nn_lowerable : Module.isLowerable benchmarks_bench_nn_env benchmarks_bench_nn_module = false := by native_decide
theorem benchmarks_gf16_bfloat16_nmse_lowerable : Module.isLowerable benchmarks_gf16_bfloat16_nmse_env benchmarks_gf16_bfloat16_nmse_module = true := by native_decide
theorem brain_brain_lowerable : Module.isLowerable brain_brain_env brain_brain_module = true := by native_decide
theorem brain_bus_lowerable : Module.isLowerable brain_bus_env brain_bus_module = true := by native_decide
theorem brain_cognitive_loop_lowerable : Module.isLowerable brain_cognitive_loop_env brain_cognitive_loop_module = true := by native_decide
theorem brain_neural_gamma_lowerable : Module.isLowerable brain_neural_gamma_env brain_neural_gamma_module = true := by native_decide
theorem cloud_railway_deploy_lowerable : Module.isLowerable cloud_railway_deploy_env cloud_railway_deploy_module = true := by native_decide
theorem compiler_mod_structure_lowerable : Module.isLowerable compiler_mod_structure_env compiler_mod_structure_module = true := by native_decide
theorem config_load_lowerable : Module.isLowerable config_load_env config_load_module = false := by native_decide
theorem demos_simple_test_lowerable : Module.isLowerable demos_simple_test_env demos_simple_test_module = true := by native_decide
theorem fpga_bootrom_lowerable : Module.isLowerable fpga_bootrom_env fpga_bootrom_module = false := by native_decide
theorem fpga_cts_lowerable : Module.isLowerable fpga_cts_env fpga_cts_module = false := by native_decide
theorem fpga_dft_lowerable : Module.isLowerable fpga_dft_env fpga_dft_module = false := by native_decide
theorem fpga_hw_types_lowerable : Module.isLowerable fpga_hw_types_env fpga_hw_types_module = true := by native_decide
theorem fpga_mac_lowerable : Module.isLowerable fpga_mac_env fpga_mac_module = false := by native_decide
theorem fpga_power_lowerable : Module.isLowerable fpga_power_env fpga_power_module = false := by native_decide
theorem fpga_spi_lowerable : Module.isLowerable fpga_spi_env fpga_spi_module = false := by native_decide
theorem fpga_top_level_lowerable : Module.isLowerable fpga_top_level_env fpga_top_level_module = false := by native_decide
theorem fpga_uart_lowerable : Module.isLowerable fpga_uart_env fpga_uart_module = false := by native_decide
theorem fpga_verification_build_verify_lowerable : Module.isLowerable fpga_verification_build_verify_env fpga_verification_build_verify_module = false := by native_decide
theorem github_auth_lowerable : Module.isLowerable github_auth_env github_auth_module = true := by native_decide
theorem github_comments_lowerable : Module.isLowerable github_comments_env github_comments_module = true := by native_decide
theorem github_issues_lowerable : Module.isLowerable github_issues_env github_issues_module = true := by native_decide
theorem github_prs_lowerable : Module.isLowerable github_prs_env github_prs_module = true := by native_decide
theorem github_tests_e2e_full_flow_lowerable : Module.isLowerable github_tests_e2e_full_flow_env github_tests_e2e_full_flow_module = true := by native_decide
theorem graph_knowledge_graph_lowerable : Module.isLowerable graph_knowledge_graph_env graph_knowledge_graph_module = false := by native_decide
theorem hslm_forward_pass_lowerable : Module.isLowerable hslm_forward_pass_env hslm_forward_pass_module = false := by native_decide
theorem igla_race_adder_tree_lowerable : Module.isLowerable igla_race_adder_tree_env igla_race_adder_tree_module = true := by native_decide
theorem igla_race_systolic_ternary_lowerable : Module.isLowerable igla_race_systolic_ternary_env igla_race_systolic_ternary_module = false := by native_decide
theorem igla_race_ternary_inference_lowerable : Module.isLowerable igla_race_ternary_inference_env igla_race_ternary_inference_module = false := by native_decide
theorem igla_race_ternary_mac_lowerable : Module.isLowerable igla_race_ternary_mac_env igla_race_ternary_mac_module = false := by native_decide
theorem igla_w521_2d_aos_param_soundness_lowerable : Module.isLowerable igla_w521_2d_aos_param_soundness_env igla_w521_2d_aos_param_soundness_module = true := by native_decide
theorem igla_w524_2d_packed_aos_param_module_lowerable : Module.isLowerable igla_w524_2d_packed_aos_param_module_env igla_w524_2d_packed_aos_param_module_module = true := by native_decide
theorem interop_gf_cross_language_lowerable : Module.isLowerable interop_gf_cross_language_env interop_gf_cross_language_module = true := by native_decide
theorem isa_ternary_arithmetic_lowerable : Module.isLowerable isa_ternary_arithmetic_env isa_ternary_arithmetic_module = true := by native_decide
theorem isa_ternary_bitwise_lowerable : Module.isLowerable isa_ternary_bitwise_env isa_ternary_bitwise_module = true := by native_decide
theorem isa_ternary_deque_lowerable : Module.isLowerable isa_ternary_deque_env isa_ternary_deque_module = true := by native_decide
theorem isa_ternary_encoding_lowerable : Module.isLowerable isa_ternary_encoding_env isa_ternary_encoding_module = true := by native_decide
theorem isa_ternary_gates_lowerable : Module.isLowerable isa_ternary_gates_env isa_ternary_gates_module = false := by native_decide
theorem isa_ternary_shift_lowerable : Module.isLowerable isa_ternary_shift_env isa_ternary_shift_module = true := by native_decide
theorem math_property_test_template_lowerable : Module.isLowerable math_property_test_template_env math_property_test_template_module = true := by native_decide
theorem ml_activation_silu_swish_vbt_activation_lowerable : Module.isLowerable ml_activation_silu_swish_vbt_activation_env ml_activation_silu_swish_vbt_activation_module = true := by native_decide
theorem ml_igla_champion_capsule_lowerable : Module.isLowerable ml_igla_champion_capsule_env ml_igla_champion_capsule_module = true := by native_decide
theorem ml_layers_avgpool2d_layer_lowerable : Module.isLowerable ml_layers_avgpool2d_layer_env ml_layers_avgpool2d_layer_module = true := by native_decide
theorem ml_layers_conv2d_layer_lowerable : Module.isLowerable ml_layers_conv2d_layer_env ml_layers_conv2d_layer_module = false := by native_decide
theorem ml_layers_dense_layer_lowerable : Module.isLowerable ml_layers_dense_layer_env ml_layers_dense_layer_module = false := by native_decide
theorem ml_layers_embedding_layer_lowerable : Module.isLowerable ml_layers_embedding_layer_env ml_layers_embedding_layer_module = false := by native_decide
theorem ml_layers_flatten_layer_lowerable : Module.isLowerable ml_layers_flatten_layer_env ml_layers_flatten_layer_module = false := by native_decide
theorem ml_layers_maxpool2d_layer_lowerable : Module.isLowerable ml_layers_maxpool2d_layer_env ml_layers_maxpool2d_layer_module = false := by native_decide
theorem ml_layers_residual_connection_lowerable : Module.isLowerable ml_layers_residual_connection_env ml_layers_residual_connection_module = false := by native_decide
theorem ml_loss_kl_divergence_lowerable : Module.isLowerable ml_loss_kl_divergence_env ml_loss_kl_divergence_module = false := by native_decide
theorem ml_loss_mse_loss_lowerable : Module.isLowerable ml_loss_mse_loss_env ml_loss_mse_loss_module = false := by native_decide
theorem ml_optimizer_lr_scheduler_lowerable : Module.isLowerable ml_optimizer_lr_scheduler_env ml_optimizer_lr_scheduler_module = false := by native_decide
theorem ml_optimizer_sgd_momentum_lowerable : Module.isLowerable ml_optimizer_sgd_momentum_env ml_optimizer_sgd_momentum_module = false := by native_decide
theorem ml_pathway_mlp_lowerable : Module.isLowerable ml_pathway_mlp_env ml_pathway_mlp_module = false := by native_decide
theorem ml_recurrent_bilstm_lowerable : Module.isLowerable ml_recurrent_bilstm_env ml_recurrent_bilstm_module = false := by native_decide
theorem ml_recurrent_gru_cell_lowerable : Module.isLowerable ml_recurrent_gru_cell_env ml_recurrent_gru_cell_module = false := by native_decide
theorem ml_recurrent_lstm_cell_lowerable : Module.isLowerable ml_recurrent_lstm_cell_env ml_recurrent_lstm_cell_module = false := by native_decide
theorem ml_recurrent_rnn_cell_lowerable : Module.isLowerable ml_recurrent_rnn_cell_env ml_recurrent_rnn_cell_module = false := by native_decide
theorem ml_recurrent_self_attention_lowerable : Module.isLowerable ml_recurrent_self_attention_env ml_recurrent_self_attention_module = false := by native_decide
theorem ml_recurrent_seq2seq_lowerable : Module.isLowerable ml_recurrent_seq2seq_env ml_recurrent_seq2seq_module = false := by native_decide
theorem ml_rl_ppo_critic_lowerable : Module.isLowerable ml_rl_ppo_critic_env ml_rl_ppo_critic_module = true := by native_decide
theorem ml_transformer_feed_forward_network_lowerable : Module.isLowerable ml_transformer_feed_forward_network_env ml_transformer_feed_forward_network_module = false := by native_decide
theorem ml_transformer_multi_head_attention_lowerable : Module.isLowerable ml_transformer_multi_head_attention_env ml_transformer_multi_head_attention_module = false := by native_decide
theorem ml_transformer_positional_encoding_lowerable : Module.isLowerable ml_transformer_positional_encoding_env ml_transformer_positional_encoding_module = false := by native_decide
theorem nn_phi_rope_lowerable : Module.isLowerable nn_phi_rope_env nn_phi_rope_module = true := by native_decide
theorem nn_sacred_attention_lowerable : Module.isLowerable nn_sacred_attention_env nn_sacred_attention_module = true := by native_decide
theorem numeric_bigint_lowerable : Module.isLowerable numeric_bigint_env numeric_bigint_module = false := by native_decide
theorem numeric_formats_lowerable : Module.isLowerable numeric_formats_env numeric_formats_module = true := by native_decide
theorem numeric_gf_competitive_lowerable : Module.isLowerable numeric_gf_competitive_env numeric_gf_competitive_module = false := by native_decide
theorem numeric_pellis_verify_lowerable : Module.isLowerable numeric_pellis_verify_env numeric_pellis_verify_module = false := by native_decide
theorem numeric_trinity_numeric_surface_lowerable : Module.isLowerable numeric_trinity_numeric_surface_env numeric_trinity_numeric_surface_module = true := by native_decide
theorem physics_e8_lqg_bridge_lowerable : Module.isLowerable physics_e8_lqg_bridge_env physics_e8_lqg_bridge_module = true := by native_decide
theorem physics_gamma_conflict_lowerable : Module.isLowerable physics_gamma_conflict_env physics_gamma_conflict_module = true := by native_decide
theorem physics_hslm_benchmark_lowerable : Module.isLowerable physics_hslm_benchmark_env physics_hslm_benchmark_module = true := by native_decide
theorem physics_lqg_cs_bridge_lowerable : Module.isLowerable physics_lqg_cs_bridge_env physics_lqg_cs_bridge_module = true := by native_decide
theorem physics_lqg_entropy_lowerable : Module.isLowerable physics_lqg_entropy_env physics_lqg_entropy_module = true := by native_decide
theorem physics_quantum_lowerable : Module.isLowerable physics_quantum_env physics_quantum_module = true := by native_decide
theorem physics_su2_chern_simons_lowerable : Module.isLowerable physics_su2_chern_simons_env physics_su2_chern_simons_module = false := by native_decide
theorem queen_task_analysis_lowerable : Module.isLowerable queen_task_analysis_env queen_task_analysis_module = true := by native_decide
theorem runtime_instance_lowerable : Module.isLowerable runtime_instance_env runtime_instance_module = false := by native_decide
theorem sacred_cosmology_lowerable : Module.isLowerable sacred_cosmology_env sacred_cosmology_module = true := by native_decide
theorem sacred_dark_matter_lowerable : Module.isLowerable sacred_dark_matter_env sacred_dark_matter_module = true := by native_decide
theorem sacred_gravity_lowerable : Module.isLowerable sacred_gravity_env sacred_gravity_module = true := by native_decide
theorem sacred_monopoles_lowerable : Module.isLowerable sacred_monopoles_env sacred_monopoles_module = true := by native_decide
theorem sacred_quantum_lowerable : Module.isLowerable sacred_quantum_env sacred_quantum_module = true := by native_decide
theorem sacred_quantum_gravity_lowerable : Module.isLowerable sacred_quantum_gravity_env sacred_quantum_gravity_module = true := by native_decide
theorem sacred_sacred_constants_lowerable : Module.isLowerable sacred_sacred_constants_env sacred_sacred_constants_module = false := by native_decide
theorem sacred_sacred_governance_lowerable : Module.isLowerable sacred_sacred_governance_env sacred_sacred_governance_module = false := by native_decide
theorem sacred_sacred_identity_lowerable : Module.isLowerable sacred_sacred_identity_env sacred_sacred_identity_module = false := by native_decide
theorem sacred_superconductivity_lowerable : Module.isLowerable sacred_superconductivity_env sacred_superconductivity_module = true := by native_decide
theorem sync_index_lowerable : Module.isLowerable sync_index_env sync_index_module = false := by native_decide
theorem tri_agent_agent_run_lowerable : Module.isLowerable tri_agent_agent_run_env tri_agent_agent_run_module = true := by native_decide
theorem tri_agent_agents_lowerable : Module.isLowerable tri_agent_agents_env tri_agent_agents_module = true := by native_decide
theorem tri_agent_autonomous_lifecycle_lowerable : Module.isLowerable tri_agent_autonomous_lifecycle_env tri_agent_autonomous_lifecycle_module = false := by native_decide
theorem tri_agent_autonomous_universe_lowerable : Module.isLowerable tri_agent_autonomous_universe_env tri_agent_autonomous_universe_module = true := by native_decide
theorem tri_agent_eternal_monitor_lowerable : Module.isLowerable tri_agent_eternal_monitor_env tri_agent_eternal_monitor_module = true := by native_decide
theorem tri_agent_experience_hooks_lowerable : Module.isLowerable tri_agent_experience_hooks_env tri_agent_experience_hooks_module = true := by native_decide
theorem tri_agent_faculty_board_lowerable : Module.isLowerable tri_agent_faculty_board_env tri_agent_faculty_board_module = true := by native_decide
theorem tri_agent_governance_agent_lowerable : Module.isLowerable tri_agent_governance_agent_env tri_agent_governance_agent_module = false := by native_decide
theorem tri_agent_handoff_lowerable : Module.isLowerable tri_agent_handoff_env tri_agent_handoff_module = false := by native_decide
theorem tri_agent_memory_lowerable : Module.isLowerable tri_agent_memory_env tri_agent_memory_module = true := by native_decide
theorem tri_agent_swarm_agents_lowerable : Module.isLowerable tri_agent_swarm_agents_env tri_agent_swarm_agents_module = false := by native_decide
theorem tri_collections_bitmap_lowerable : Module.isLowerable tri_collections_bitmap_env tri_collections_bitmap_module = true := by native_decide
theorem tri_collections_bitset_lowerable : Module.isLowerable tri_collections_bitset_env tri_collections_bitset_module = true := by native_decide
theorem tri_collections_bitvector_lowerable : Module.isLowerable tri_collections_bitvector_env tri_collections_bitvector_module = true := by native_decide
theorem tri_collections_btree_lowerable : Module.isLowerable tri_collections_btree_env tri_collections_btree_module = true := by native_decide
theorem tri_collections_circular_buffer_lowerable : Module.isLowerable tri_collections_circular_buffer_env tri_collections_circular_buffer_module = true := by native_decide
theorem tri_collections_context_lowerable : Module.isLowerable tri_collections_context_env tri_collections_context_module = true := by native_decide
theorem tri_collections_deque_lowerable : Module.isLowerable tri_collections_deque_env tri_collections_deque_module = true := by native_decide
theorem tri_collections_either_lowerable : Module.isLowerable tri_collections_either_env tri_collections_either_module = true := by native_decide
theorem tri_collections_interval_lowerable : Module.isLowerable tri_collections_interval_env tri_collections_interval_module = true := by native_decide
theorem tri_collections_linked_list_lowerable : Module.isLowerable tri_collections_linked_list_env tri_collections_linked_list_module = false := by native_decide
theorem tri_collections_list_lowerable : Module.isLowerable tri_collections_list_env tri_collections_list_module = true := by native_decide
theorem tri_collections_lockfree_stack_lowerable : Module.isLowerable tri_collections_lockfree_stack_env tri_collections_lockfree_stack_module = false := by native_decide
theorem tri_collections_lru_lowerable : Module.isLowerable tri_collections_lru_env tri_collections_lru_module = true := by native_decide
theorem tri_collections_lru_cache_lowerable : Module.isLowerable tri_collections_lru_cache_env tri_collections_lru_cache_module = false := by native_decide
theorem tri_collections_map_lowerable : Module.isLowerable tri_collections_map_env tri_collections_map_module = true := by native_decide
theorem tri_collections_namespace_lowerable : Module.isLowerable tri_collections_namespace_env tri_collections_namespace_module = false := by native_decide
theorem tri_collections_option_lowerable : Module.isLowerable tri_collections_option_env tri_collections_option_module = true := by native_decide
theorem tri_collections_priority_queue_lowerable : Module.isLowerable tri_collections_priority_queue_env tri_collections_priority_queue_module = true := by native_decide
theorem tri_collections_queue_lowerable : Module.isLowerable tri_collections_queue_env tri_collections_queue_module = true := by native_decide
theorem tri_collections_result_lowerable : Module.isLowerable tri_collections_result_env tri_collections_result_module = true := by native_decide
theorem tri_collections_ring_buffer_lowerable : Module.isLowerable tri_collections_ring_buffer_env tri_collections_ring_buffer_module = true := by native_decide
theorem tri_collections_skip_list_lowerable : Module.isLowerable tri_collections_skip_list_env tri_collections_skip_list_module = true := by native_decide
theorem tri_collections_stack_lowerable : Module.isLowerable tri_collections_stack_env tri_collections_stack_module = true := by native_decide
theorem tri_collections_tuple_lowerable : Module.isLowerable tri_collections_tuple_env tri_collections_tuple_module = true := by native_decide
theorem tri_collections_variant_lowerable : Module.isLowerable tri_collections_variant_env tri_collections_variant_module = true := by native_decide
theorem tri_crypto_base32_lowerable : Module.isLowerable tri_crypto_base32_env tri_crypto_base32_module = true := by native_decide
theorem tri_crypto_base64_lowerable : Module.isLowerable tri_crypto_base64_env tri_crypto_base64_module = true := by native_decide
theorem tri_crypto_crypto_lowerable : Module.isLowerable tri_crypto_crypto_env tri_crypto_crypto_module = false := by native_decide
theorem tri_crypto_ecc_lowerable : Module.isLowerable tri_crypto_ecc_env tri_crypto_ecc_module = false := by native_decide
theorem tri_crypto_hmac_lowerable : Module.isLowerable tri_crypto_hmac_env tri_crypto_hmac_module = true := by native_decide
theorem tri_crypto_reed_solomon_lowerable : Module.isLowerable tri_crypto_reed_solomon_env tri_crypto_reed_solomon_module = false := by native_decide
theorem tri_crypto_rsa_lowerable : Module.isLowerable tri_crypto_rsa_env tri_crypto_rsa_module = false := by native_decide
theorem tri_encoding_bson_lowerable : Module.isLowerable tri_encoding_bson_env tri_encoding_bson_module = false := by native_decide
theorem tri_encoding_csv_lowerable : Module.isLowerable tri_encoding_csv_env tri_encoding_csv_module = false := by native_decide
theorem tri_encoding_html_lowerable : Module.isLowerable tri_encoding_html_env tri_encoding_html_module = false := by native_decide
theorem tri_encoding_json_lowerable : Module.isLowerable tri_encoding_json_env tri_encoding_json_module = false := by native_decide
theorem tri_encoding_markup_lowerable : Module.isLowerable tri_encoding_markup_env tri_encoding_markup_module = true := by native_decide
theorem tri_encoding_mime_lowerable : Module.isLowerable tri_encoding_mime_env tri_encoding_mime_module = true := by native_decide
theorem tri_encoding_msgpack_lowerable : Module.isLowerable tri_encoding_msgpack_env tri_encoding_msgpack_module = true := by native_decide
theorem tri_encoding_xml_lowerable : Module.isLowerable tri_encoding_xml_env tri_encoding_xml_module = false := by native_decide
theorem tri_graph_dijkstra_lowerable : Module.isLowerable tri_graph_dijkstra_env tri_graph_dijkstra_module = false := by native_decide
theorem tri_graph_graph_lowerable : Module.isLowerable tri_graph_graph_env tri_graph_graph_module = true := by native_decide
theorem tri_graph_graph_bfs_lowerable : Module.isLowerable tri_graph_graph_bfs_env tri_graph_graph_bfs_module = false := by native_decide
theorem tri_graph_graph_dfs_lowerable : Module.isLowerable tri_graph_graph_dfs_env tri_graph_graph_dfs_module = false := by native_decide
theorem tri_graph_prims_mst_lowerable : Module.isLowerable tri_graph_prims_mst_env tri_graph_prims_mst_module = true := by native_decide
theorem tri_graph_topological_sort_lowerable : Module.isLowerable tri_graph_topological_sort_env tri_graph_topological_sort_module = true := by native_decide
theorem tri_io_compress_lowerable : Module.isLowerable tri_io_compress_env tri_io_compress_module = true := by native_decide
theorem tri_io_filesystem_lowerable : Module.isLowerable tri_io_filesystem_env tri_io_filesystem_module = false := by native_decide
theorem tri_io_fs_lowerable : Module.isLowerable tri_io_fs_env tri_io_fs_module = false := by native_decide
theorem tri_io_zip_lowerable : Module.isLowerable tri_io_zip_env tri_io_zip_module = true := by native_decide
theorem tri_math_bezier_lowerable : Module.isLowerable tri_math_bezier_env tri_math_bezier_module = false := by native_decide
theorem tri_math_constants_lowerable : Module.isLowerable tri_math_constants_env tri_math_constants_module = false := by native_decide
theorem tri_math_math_lowerable : Module.isLowerable tri_math_math_env tri_math_math_module = true := by native_decide
theorem tri_math_matrix_lowerable : Module.isLowerable tri_math_matrix_env tri_math_matrix_module = true := by native_decide
theorem tri_math_measurement_lowerable : Module.isLowerable tri_math_measurement_env tri_math_measurement_module = true := by native_decide
theorem tri_math_polynomial_lowerable : Module.isLowerable tri_math_polynomial_env tri_math_polynomial_module = true := by native_decide
theorem tri_math_probability_lowerable : Module.isLowerable tri_math_probability_env tri_math_probability_module = false := by native_decide
theorem tri_math_statistics_lowerable : Module.isLowerable tri_math_statistics_env tri_math_statistics_module = false := by native_decide
theorem tri_net_async_lowerable : Module.isLowerable tri_net_async_env tri_net_async_module = true := by native_decide
theorem tri_net_async_stream_lowerable : Module.isLowerable tri_net_async_stream_env tri_net_async_stream_module = true := by native_decide
theorem tri_net_channel_lowerable : Module.isLowerable tri_net_channel_env tri_net_channel_module = true := by native_decide
theorem tri_net_cloud_lowerable : Module.isLowerable tri_net_cloud_env tri_net_cloud_module = true := by native_decide
theorem tri_net_http_lowerable : Module.isLowerable tri_net_http_env tri_net_http_module = false := by native_decide
theorem tri_net_net_lowerable : Module.isLowerable tri_net_net_env tri_net_net_module = false := by native_decide
theorem tri_net_url_lowerable : Module.isLowerable tri_net_url_env tri_net_url_module = true := by native_decide
theorem tri_pipeline_batch_runner_lowerable : Module.isLowerable tri_pipeline_batch_runner_env tri_pipeline_batch_runner_module = false := by native_decide
theorem tri_pipeline_builder_lowerable : Module.isLowerable tri_pipeline_builder_env tri_pipeline_builder_module = true := by native_decide
theorem tri_pipeline_cloud_orchestrator_lowerable : Module.isLowerable tri_pipeline_cloud_orchestrator_env tri_pipeline_cloud_orchestrator_module = true := by native_decide
theorem tri_pipeline_codegen_lowerable : Module.isLowerable tri_pipeline_codegen_env tri_pipeline_codegen_module = false := by native_decide
theorem tri_pipeline_pipeline_lowerable : Module.isLowerable tri_pipeline_pipeline_env tri_pipeline_pipeline_module = true := by native_decide
theorem tri_pipeline_pipeline_parallel_lowerable : Module.isLowerable tri_pipeline_pipeline_parallel_env tri_pipeline_pipeline_parallel_module = false := by native_decide
theorem tri_pipeline_spec_parser_lowerable : Module.isLowerable tri_pipeline_spec_parser_env tri_pipeline_spec_parser_module = true := by native_decide
theorem tri_pipeline_workflow_lowerable : Module.isLowerable tri_pipeline_workflow_env tri_pipeline_workflow_module = false := by native_decide
theorem tri_pipeline_workflow_executor_lowerable : Module.isLowerable tri_pipeline_workflow_executor_env tri_pipeline_workflow_executor_module = true := by native_decide
theorem tri_pipeline_workflow_parser_lowerable : Module.isLowerable tri_pipeline_workflow_parser_env tri_pipeline_workflow_parser_module = true := by native_decide
theorem tri_search_aho_corasick_lowerable : Module.isLowerable tri_search_aho_corasick_env tri_search_aho_corasick_module = true := by native_decide
theorem tri_search_bloom_filter_lowerable : Module.isLowerable tri_search_bloom_filter_env tri_search_bloom_filter_module = true := by native_decide
theorem tri_search_boyer_moore_lowerable : Module.isLowerable tri_search_boyer_moore_env tri_search_boyer_moore_module = true := by native_decide
theorem tri_search_knuth_morris_pratt_lowerable : Module.isLowerable tri_search_knuth_morris_pratt_env tri_search_knuth_morris_pratt_module = false := by native_decide
theorem tri_search_match_lowerable : Module.isLowerable tri_search_match_env tri_search_match_module = true := by native_decide
theorem tri_search_pattern_lowerable : Module.isLowerable tri_search_pattern_env tri_search_pattern_module = false := by native_decide
theorem tri_search_rabin_karp_lowerable : Module.isLowerable tri_search_rabin_karp_env tri_search_rabin_karp_module = false := by native_decide
theorem tri_search_regex_lowerable : Module.isLowerable tri_search_regex_env tri_search_regex_module = true := by native_decide
theorem tri_search_regex_advanced_lowerable : Module.isLowerable tri_search_regex_advanced_env tri_search_regex_advanced_module = false := by native_decide
theorem tri_search_search_lowerable : Module.isLowerable tri_search_search_env tri_search_search_module = false := by native_decide
theorem tri_sort_counting_sort_lowerable : Module.isLowerable tri_sort_counting_sort_env tri_sort_counting_sort_module = false := by native_decide
theorem tri_sort_heap_sort_lowerable : Module.isLowerable tri_sort_heap_sort_env tri_sort_heap_sort_module = false := by native_decide
theorem tri_sort_insertion_sort_lowerable : Module.isLowerable tri_sort_insertion_sort_env tri_sort_insertion_sort_module = false := by native_decide
theorem tri_sort_merge_sort_lowerable : Module.isLowerable tri_sort_merge_sort_env tri_sort_merge_sort_module = false := by native_decide
theorem tri_sort_quick_sort_lowerable : Module.isLowerable tri_sort_quick_sort_env tri_sort_quick_sort_module = false := by native_decide
theorem tri_sort_radix_sort_lowerable : Module.isLowerable tri_sort_radix_sort_env tri_sort_radix_sort_module = false := by native_decide
theorem tri_sort_selection_sort_lowerable : Module.isLowerable tri_sort_selection_sort_env tri_sort_selection_sort_module = false := by native_decide
theorem tri_sort_shell_sort_lowerable : Module.isLowerable tri_sort_shell_sort_env tri_sort_shell_sort_module = false := by native_decide
theorem tri_sort_sort_lowerable : Module.isLowerable tri_sort_sort_env tri_sort_sort_module = false := by native_decide
theorem tri_sort_tim_sort_lowerable : Module.isLowerable tri_sort_tim_sort_env tri_sort_tim_sort_module = false := by native_decide
theorem tri_trees_avl_tree_lowerable : Module.isLowerable tri_trees_avl_tree_env tri_trees_avl_tree_module = false := by native_decide
theorem tri_trees_b_tree_lowerable : Module.isLowerable tri_trees_b_tree_env tri_trees_b_tree_module = false := by native_decide
theorem tri_trees_fenwick_tree_lowerable : Module.isLowerable tri_trees_fenwick_tree_env tri_trees_fenwick_tree_module = true := by native_decide
theorem tri_trees_kd_tree_lowerable : Module.isLowerable tri_trees_kd_tree_env tri_trees_kd_tree_module = true := by native_decide
theorem tri_trees_octree_lowerable : Module.isLowerable tri_trees_octree_env tri_trees_octree_module = false := by native_decide
theorem tri_trees_quadtree_lowerable : Module.isLowerable tri_trees_quadtree_env tri_trees_quadtree_module = false := by native_decide
theorem tri_trees_red_black_tree_lowerable : Module.isLowerable tri_trees_red_black_tree_env tri_trees_red_black_tree_module = false := by native_decide
theorem tri_trees_rtree_lowerable : Module.isLowerable tri_trees_rtree_env tri_trees_rtree_module = false := by native_decide
theorem tri_trees_segment_tree_lowerable : Module.isLowerable tri_trees_segment_tree_env tri_trees_segment_tree_module = true := by native_decide
theorem tri_trees_splay_tree_lowerable : Module.isLowerable tri_trees_splay_tree_env tri_trees_splay_tree_module = false := by native_decide
theorem tri_trees_suffix_array_lowerable : Module.isLowerable tri_trees_suffix_array_env tri_trees_suffix_array_module = true := by native_decide
theorem tri_trees_tree_lowerable : Module.isLowerable tri_trees_tree_env tri_trees_tree_module = true := by native_decide
theorem tri_trees_trie_lowerable : Module.isLowerable tri_trees_trie_env tri_trees_trie_module = true := by native_decide
theorem tri_utils_args_lowerable : Module.isLowerable tri_utils_args_env tri_utils_args_module = true := by native_decide
theorem tri_utils_arrow_time_lowerable : Module.isLowerable tri_utils_arrow_time_env tri_utils_arrow_time_module = true := by native_decide
theorem tri_utils_bytes_lowerable : Module.isLowerable tri_utils_bytes_env tri_utils_bytes_module = true := by native_decide
theorem tri_utils_color_lowerable : Module.isLowerable tri_utils_color_env tri_utils_color_module = false := by native_decide
theorem tri_utils_colors_lowerable : Module.isLowerable tri_utils_colors_env tri_utils_colors_module = true := by native_decide
theorem tri_utils_config_lowerable : Module.isLowerable tri_utils_config_env tri_utils_config_module = false := by native_decide
theorem tri_utils_error_lowerable : Module.isLowerable tri_utils_error_env tri_utils_error_module = true := by native_decide
theorem tri_utils_exit_codes_lowerable : Module.isLowerable tri_utils_exit_codes_env tri_utils_exit_codes_module = false := by native_decide
theorem tri_utils_help_lowerable : Module.isLowerable tri_utils_help_env tri_utils_help_module = false := by native_decide
theorem tri_utils_logging_lowerable : Module.isLowerable tri_utils_logging_env tri_utils_logging_module = false := by native_decide
theorem tri_utils_random_lowerable : Module.isLowerable tri_utils_random_env tri_utils_random_module = false := by native_decide
theorem tri_utils_string_lowerable : Module.isLowerable tri_utils_string_env tri_utils_string_module = true := by native_decide
theorem tri_utils_template_lowerable : Module.isLowerable tri_utils_template_env tri_utils_template_module = true := by native_decide
theorem tri_utils_terminal_lowerable : Module.isLowerable tri_utils_terminal_env tri_utils_terminal_module = false := by native_decide
theorem tri_utils_text_lowerable : Module.isLowerable tri_utils_text_env tri_utils_text_module = false := by native_decide
theorem tri_utils_time_lowerable : Module.isLowerable tri_utils_time_env tri_utils_time_module = false := by native_decide
theorem tri_utils_utf8_lowerable : Module.isLowerable tri_utils_utf8_env tri_utils_utf8_module = false := by native_decide
theorem tri_utils_version_lowerable : Module.isLowerable tri_utils_version_env tri_utils_version_module = false := by native_decide
theorem vm_jit_semantics_lowerable : Module.isLowerable vm_jit_semantics_env vm_jit_semantics_module = false := by native_decide
theorem vsa_packed_vsa_lowerable : Module.isLowerable vsa_packed_vsa_env vsa_packed_vsa_module = true := by native_decide
theorem vsa_sequence_hdc_lowerable : Module.isLowerable vsa_sequence_hdc_env vsa_sequence_hdc_module = false := by native_decide
theorem igla_w535_bounded_while_module_lowerable : Module.isLowerable igla_w535_bounded_while_module_env igla_w535_bounded_while_module_module = true := by native_decide

def scratch_w545_call_init_returns_array_env : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["seq"]
}

def scratch_w545_call_init_returns_array_module : Module := {
  name := "w545_call_init_returns_array",
  imports := [],
  globals := [
    .constDecl "a" (.array 3 .u8) (some (.call "seq" []))
  ],
  functions := [{
    name := "seq",
    params := [],
    ret := some (.array 3 .u8),
    body := [
      .return_ (some (.arrayLit (.array 3 .u8)
        [.intLit 1, .intLit 2, .intLit 3]))
    ]
  }],
  tests := [
    { name := "call_init_returns_array", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 0), .intLit 1]),
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 1), .intLit 2]),
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 2), .intLit 3])
    ]}
  ],
  benches := []
}

theorem scratch_w545_call_init_returns_array_lowerable :
  Module.isLowerable scratch_w545_call_init_returns_array_env scratch_w545_call_init_returns_array_module = true := by native_decide
