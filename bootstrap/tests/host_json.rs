use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn t27c");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn parse_json(s: &str) -> serde_json::Value {
    let trimmed = s.trim();
    serde_json::from_str(trimmed).unwrap_or_else(|e| panic!("invalid JSON: {e}\ninput: {trimmed}"))
}

// -- host-smoke --json --

#[test]
fn smoke_json_is_valid() {
    let (ok, stdout, _) = run(&["host-smoke", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["ok"], true);
}

#[test]
fn smoke_json_has_writes_reads() {
    let (ok, stdout, _) = run(&["host-smoke", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["writes"].is_number());
    assert!(v["reads"].is_number());
}

#[test]
fn smoke_json_has_config_fields() {
    let (ok, stdout, _) = run(&["host-smoke", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["layers"], 2);
    assert_eq!(v["neurons"], 16);
    assert_eq!(v["chunks"], 4);
    assert_eq!(v["threshold"], 1);
}

#[test]
fn smoke_json_weight_addr_is_string() {
    let (ok, stdout, _) = run(&["host-smoke", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["weight_addr"].is_string());
    assert!(v["weight_addr"].as_str().unwrap().starts_with("0x"));
}

#[test]
fn smoke_json_irq_stat_is_string() {
    let (ok, stdout, _) = run(&["host-smoke", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["irq_stat"].is_string());
}

// -- host-poll-vs-irq --json --

#[test]
fn poll_vs_irq_json_is_valid() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["ok"], true);
}

#[test]
fn poll_vs_irq_json_has_poll_and_irq_counts() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["poll_writes"].is_number());
    assert!(v["poll_reads"].is_number());
    assert!(v["irq_writes"].is_number());
    assert!(v["irq_reads"].is_number());
}

#[test]
fn poll_vs_irq_json_writes_match_is_bool() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["writes_match"].is_boolean());
}

// -- host-inference --json --

#[test]
fn inference_json_is_valid() {
    let (ok, stdout, _) = run(&["host-inference", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["ok"], true);
}

#[test]
fn inference_json_has_layers_completed() {
    let (ok, stdout, _) = run(&["host-inference", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["total_layers"], 2);
    assert_eq!(v["layers_completed"], 2);
}

#[test]
fn inference_json_has_writes_reads() {
    let (ok, stdout, _) = run(&["host-inference", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["total_writes"].is_number());
    assert!(v["total_reads"].is_number());
}

#[test]
fn inference_json_error_layer_is_null_on_success() {
    let (ok, stdout, _) = run(&["host-inference", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["error_layer"].is_null());
}

#[test]
fn inference_json_single_layer() {
    let (ok, stdout, _) = run(&["host-inference", "--json", "--num-layers", "1"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["total_layers"], 1);
    assert_eq!(v["layers_completed"], 1);
}

// -- host-perf --json --

#[test]
fn perf_json_is_valid() {
    let (ok, stdout, _) = run(&["host-perf", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["ok"], true);
}

#[test]
fn perf_json_has_config() {
    let (ok, stdout, _) = run(&["host-perf", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["layers"], 2);
    assert_eq!(v["neurons"], 16);
    assert_eq!(v["chunks"], 4);
}

#[test]
fn perf_json_has_cycles_and_dma() {
    let (ok, stdout, _) = run(&["host-perf", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["total_cycles"].is_number());
    assert!(v["total_dma_beats"].is_number());
    assert!(v["total_weight_words"].is_number());
}

#[test]
fn perf_json_has_bram_pct() {
    let (ok, stdout, _) = run(&["host-perf", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["bram_utilization_pct"].is_number());
}

#[test]
fn perf_json_has_throughput() {
    let (ok, stdout, _) = run(&["host-perf", "--json"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert!(v["throughput_inf_per_sec"].is_number());
    assert!(v["clock_mhz"].is_number());
}

#[test]
fn perf_json_custom_clock() {
    let (ok, stdout, _) = run(&["host-perf", "--json", "--clock-mhz", "100.0"]);
    assert!(ok);
    let v = parse_json(&stdout);
    assert_eq!(v["clock_mhz"], 100.0);
}

// -- cross-command: without --json, output is NOT JSON --

#[test]
fn smoke_without_json_is_not_json_object() {
    let (ok, stdout, _) = run(&["host-smoke"]);
    assert!(ok);
    let trimmed = stdout.trim();
    assert!(trimmed.starts_with("OK "), "expected human-readable: {trimmed}");
    assert!(!trimmed.starts_with("{"), "should not be JSON: {trimmed}");
}

#[test]
fn perf_without_json_is_not_json_object() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    let trimmed = stdout.trim();
    assert!(trimmed.starts_with("OK "), "expected human-readable: {trimmed}");
}

// -- determinism --

#[test]
fn smoke_json_deterministic() {
    let (ok1, s1, _) = run(&["host-smoke", "--json"]);
    let (ok2, s2, _) = run(&["host-smoke", "--json"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}

#[test]
fn perf_json_deterministic() {
    let (ok1, s1, _) = run(&["host-perf", "--json"]);
    let (ok2, s2, _) = run(&["host-perf", "--json"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}
