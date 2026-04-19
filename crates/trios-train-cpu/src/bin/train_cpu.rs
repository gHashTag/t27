//! CPU training binary for IGLA-GF16
//!
//! Demonstrates the CPU training loop with BPB measurement.

use trios_train_cpu::{
    bpb_from_loss, estimate_model_size, print_metrics, train_cpu_loop, TrainConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IGLA-GF16 CPU Training Loop");
    println!("============================");
    println!();

    // Create training configuration
    let config = TrainConfig {
        max_steps: 100,
        batch_size: 4,
        seq_len: 128,
        learning_rate: 0.001,
        warmup_steps: 10,
        grad_clip: 0.618,
        log_every: 10,
        checkpoint_path: "igla-gf16-cpu.bin".to_string(),
        ..Default::default()
    };

    // Print configuration
    println!("Configuration:");
    println!("  max_steps: {}", config.max_steps);
    println!("  batch_size: {}", config.batch_size);
    println!("  seq_len: {}", config.seq_len);
    println!("  learning_rate: {}", config.learning_rate);
    println!("  warmup_steps: {}", config.warmup_steps);
    println!("  grad_clip: {}", config.grad_clip);
    println!();

    // Estimate model size
    let vocab_size = 32000;
    let dims = config.dims;
    let model_size_bytes = estimate_model_size(vocab_size, dims.d_model, 7, dims.d_ffn);
    let model_size_mb = model_size_bytes as f64 / (1024.0 * 1024.0);

    println!("Model Size:");
    println!("  vocab_size: {}", vocab_size);
    println!("  d_model: {}", dims.d_model);
    println!("  n_heads: {}", dims.n_heads);
    println!("  d_ffn: {}", dims.d_ffn);
    println!("  estimated_size: {:.2} MB", model_size_mb);
    println!();

    // Demonstrate BPB calculation
    println!("BPB Calculation:");
    let loss: f64 = 2.5;
    let bpb = bpb_from_loss(loss);
    println!("  loss = {}", loss);
    println!("  bpb = loss / ln(2) = {:.4}", bpb);
    println!();

    // Run training loop (fast demo)
    println!("Training Loop:");
    println!("-------------");
    let metrics = train_cpu_loop(&config, vocab_size);
    println!();

    println!("Training completed!");
    print_metrics(&metrics);

    Ok(())
}
