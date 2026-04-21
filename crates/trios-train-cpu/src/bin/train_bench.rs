use trios_train_cpu::real_igla_model::RealIglaModel;
use trios_train_cpu::phi_ortho_init;

fn load_data() -> Vec<usize> {
    for p in ["data/input.txt", "data/tiny_shakespeare.txt"] {
        if let Ok(c) = std::fs::read_to_string(p) {
            return c.bytes().map(|b| b as usize).collect();
        }
    }
    (0..50000).map(|i| i % 256).collect()
}

fn main() {
    let data = load_data();
    println!("Data: {} bytes", data.len());

    let mut model = RealIglaModel::new(256, 64, 1);
    phi_ortho_init(&mut model.embed, 64, 256);
    println!("Params: {}", model.param_count());

    let seq_len = 32;
    let lr = 5e-4;
    let max_steps = 1000;

    let eval_tokens: Vec<usize> = data[..128.min(data.len())].to_vec();
    let (_, init_bpb) = model.loss_bpb(&eval_tokens);
    println!("Initial BPB: {:.4}\n", init_bpb);

    let start = std::time::Instant::now();
    let mut best = f64::MAX;

    for step in 0..max_steps {
        let idx = step * seq_len % (data.len().saturating_sub(seq_len + 1));
        let tokens: Vec<usize> = data[idx..idx + seq_len + 1].to_vec();
        let cur_lr = if step < 100 { lr * step as f32 / 100.0 } else { lr * (1.0 - (step - 100) as f32 / 900.0).max(0.01) };

        let loss = model.train_step(&tokens, cur_lr);
        let bpb = loss as f64 / std::f64::consts::LN_2;
        if bpb < best { best = bpb; }

        if step % 200 == 0 || step == max_steps - 1 {
            let (_, eb) = model.loss_bpb(&eval_tokens);
            let t = start.elapsed().as_secs_f64();
            println!("step {:>4}: bpb={:.3} eval={:.3} best={:.3} lr={:.5} {:.0}s", step, bpb, eb, best, cur_lr, t);
        }
    }

    let (_, final_bpb) = model.loss_bpb(&eval_tokens);
    println!("\nDone {:.0}s | best={:.3} final_eval={:.3}", start.elapsed().as_secs_f64(), best, final_bpb);
}
