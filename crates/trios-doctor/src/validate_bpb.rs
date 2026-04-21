use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: validate_bpb <predictions_file>");
        std::process::exit(1);
    }
    let path = &args[1];
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        eprintln!("Empty file: {}", path);
        std::process::exit(1);
    }
    let total: f64 = lines.iter().filter_map(|l| l.parse::<f64>().ok()).sum();
    let count = lines.iter().filter(|l| l.parse::<f64>().is_ok()).count();
    if count == 0 {
        eprintln!("No valid BPB values found");
        std::process::exit(1);
    }
    let avg_bpb = total / count as f64;
    let baseline: f64 = 1.2244;
    let delta = avg_bpb - baseline;
    let gate = 1.12;
    let pass = avg_bpb <= gate;
    println!("BPB Validation Report");
    println!("=====================");
    println!("Samples:    {}", count);
    println!("Avg BPB:    {:.6}", avg_bpb);
    println!("Baseline:   {:.4}", baseline);
    println!("Delta:      {:+.6}", delta);
    println!(
        "G-STACK:    {} (gate <= {:.2})",
        if pass { "PASS" } else { "FAIL" },
        gate
    );
    if !pass {
        std::process::exit(1);
    }
    Ok(())
}
