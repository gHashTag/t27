#![allow(clippy::type_complexity)]
#![allow(clippy::redundant_field_names)]

use std::f32::consts::LN_2;

#[inline]
fn relu(x: f32) -> f32 {
    x.max(0.0)
}

fn softmax(v: &mut [f32]) {
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for x in v.iter_mut() {
        *x = (*x - max).exp();
        sum += *x;
    }
    for x in v.iter_mut() {
        *x /= sum;
    }
}

fn layer_norm(x: &[f32], eps: f32) -> Vec<f32> {
    let n = x.len() as f32;
    let mean = x.iter().sum::<f32>() / n;
    let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let std = (var + eps).sqrt();
    x.iter().map(|v| (v - mean) / std).collect()
}

#[allow(dead_code)]
fn layer_norm_backward(dout: &[f32], x: &[f32], eps: f32) -> Vec<f32> {
    let n = x.len() as f32;
    let mean = x.iter().sum::<f32>() / n;
    let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let std = (var + eps).sqrt();
    let inv_std = 1.0 / std;
    let dx_mean: f32 = dout.iter().zip(x.iter()).map(|(d, xi)| d * (xi - mean)).sum();
    let ds: f32 = dout.iter().sum();
    let norm = inv_std / n;
    dout.iter()
        .zip(x.iter())
        .map(|(d, xi)| norm * (n * d - ds - (xi - mean) * inv_std * inv_std * dx_mean))
        .collect()
}

fn matvec(a: &[f32], rows: usize, cols: usize, v: &[f32]) -> Vec<f32> {
    (0..rows)
        .map(|r| {
            let row = &a[r * cols..(r + 1) * cols];
            row.iter().zip(v.iter()).map(|(w, x)| w * x).sum()
        })
        .collect()
}

fn matvec_transpose(a: &[f32], rows: usize, cols: usize, v: &[f32]) -> Vec<f32> {
    (0..cols)
        .map(|c| (0..rows).map(|r| a[r * cols + c] * v[r]).sum())
        .collect()
}

#[allow(dead_code)]
struct LayerIO {
    input: Vec<Vec<f32>>,
    normed1: Vec<Vec<f32>>,
    after_attn: Vec<Vec<f32>>,
    normed2: Vec<Vec<f32>>,
    h1: Vec<Vec<f32>>,
    attn_caches: Vec<AttnCache>,
}

fn xavier_init(size: usize, fan_in: usize, fan_out: usize, seed: &mut u64) -> Vec<f32> {
    let limit = (6.0f32 / (fan_in + fan_out) as f32).sqrt();
    (0..size)
        .map(|_| {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let t = ((*seed >> 33) as f32) / (u32::MAX as f32);
            t * 2.0 * limit - limit
        })
        .collect()
}

struct AttentionHead {
    d_k: usize,
    wq: Vec<f32>,
    wk: Vec<f32>,
    wv: Vec<f32>,
    wo: Vec<f32>,
}

#[allow(dead_code)]
struct AttnCache {
    qs: Vec<Vec<f32>>,
    ks: Vec<Vec<f32>>,
    vs: Vec<Vec<f32>>,
    scores: Vec<Vec<f32>>,
    ctx: Vec<Vec<f32>>,
    normed: Vec<Vec<f32>>,
}

impl AttentionHead {
    fn new(d_model: usize, d_k: usize, seed: &mut u64) -> Self {
        Self {
            d_k,
            wq: xavier_init(d_model * d_k, d_model, d_k, seed),
            wk: xavier_init(d_model * d_k, d_model, d_k, seed),
            wv: xavier_init(d_model * d_k, d_model, d_k, seed),
            wo: xavier_init(d_k * d_model, d_k, d_model, seed),
        }
    }

    fn forward(&self, xs: &[Vec<f32>]) -> (Vec<Vec<f32>>, AttnCache) {
        let seq = xs.len();
        let d_model = xs[0].len();
        let scale = (self.d_k as f32).sqrt();

        let qs: Vec<Vec<f32>> = xs.iter().map(|x| matvec(&self.wq, self.d_k, d_model, x)).collect();
        let ks: Vec<Vec<f32>> = xs.iter().map(|x| matvec(&self.wk, self.d_k, d_model, x)).collect();
        let vs: Vec<Vec<f32>> = xs.iter().map(|x| matvec(&self.wv, self.d_k, d_model, x)).collect();

        let mut all_scores = Vec::with_capacity(seq);
        let mut ctx = vec![vec![0.0f32; self.d_k]; seq];

        for (qi, q) in qs.iter().enumerate() {
            let mut scores: Vec<f32> = ks[..=qi]
                .iter()
                .map(|k| q.iter().zip(k.iter()).map(|(qv, kv)| qv * kv).sum::<f32>() / scale)
                .collect();
            softmax(&mut scores);

            for (j, &w) in scores.iter().enumerate() {
                for (c, v) in ctx[qi].iter_mut().zip(vs[j].iter()) {
                    *c += w * v;
                }
            }
            all_scores.push(scores);
        }

        let out: Vec<Vec<f32>> = ctx.iter().map(|c| matvec(&self.wo, d_model, self.d_k, c)).collect();

        let cache = AttnCache {
            qs,
            ks,
            vs,
            scores: all_scores,
            ctx,
            normed: xs.to_vec(),
        };
        (out, cache)
    }

    #[allow(dead_code)]
    fn backward(
        &self,
        dout: &[Vec<f32>],
        cache: &AttnCache,
    ) -> (Vec<Vec<f32>>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        let seq = dout.len();
        let d_model = dout[0].len();
        let scale = (self.d_k as f32).sqrt();

        let mut d_wo = vec![0.0f32; self.d_k * d_model];
        let mut d_ctx = vec![vec![0.0f32; self.d_k]; seq];

        for (i, d_out_i) in dout.iter().enumerate() {
            for r in 0..d_model {
                for c in 0..self.d_k {
                    d_wo[c * d_model + r] += cache.ctx[i][c] * d_out_i[r];
                }
                let dot: f32 = (0..self.d_k)
                    .map(|c| self.wo[c * d_model + r] * d_out_i[r])
                    .sum();
                d_ctx[i][r % self.d_k] += if r < self.d_k { dot } else { 0.0 };
            }
        }

        let mut d_wq = vec![0.0f32; d_model * self.d_k];
        let mut d_wk = vec![0.0f32; d_model * self.d_k];
        let mut d_wv = vec![0.0f32; d_model * self.d_k];
        let mut d_input = vec![vec![0.0f32; d_model]; seq];

        for (i, d_ctx_i) in d_ctx.iter().enumerate() {
            let n_scores = cache.scores[i].len();
            let mut d_scores = vec![0.0f32; n_scores];

            for (j, d_score_j) in d_scores.iter_mut().enumerate() {
                for (c, &v) in cache.vs[j].iter().enumerate() {
                    *d_score_j += d_ctx_i[c] * v;
                }
            }

            for (j, &s) in cache.scores[i].iter().enumerate() {
                let softmax_grad = s * (d_scores[j] - d_scores.iter().zip(cache.scores[i].iter()).map(|(ds, sc)| ds * sc).sum::<f32>());
                let d_attn = softmax_grad / scale;

                for (c, &q) in cache.qs[i].iter().enumerate() {
                    for (r, &norm_val) in cache.normed[i].iter().enumerate() {
                        d_wk[r * self.d_k + c] += d_attn * q * norm_val;
                    }
                }

                for (c, &k) in cache.ks[j].iter().enumerate() {
                    for (r, &norm_val) in cache.normed[i].iter().enumerate() {
                        d_wq[r * self.d_k + c] += d_attn * k * norm_val;
                    }
                    d_input[j][c % d_model] += d_attn * k;
                }

                for (c, _) in cache.vs[j].iter().enumerate() {
                    d_wv[c % d_model * self.d_k + c / d_model] += cache.scores[i][j] * d_ctx[i][c];
                    d_input[j][c % d_model] += cache.scores[i][j] * d_ctx[i][c];
                }
            }
        }

        (d_input, d_wq, d_wk, d_wv, d_wo)
    }
}

pub struct TransformerLayer {
    heads: Vec<AttentionHead>,
    d_model: usize,
    w1: Vec<f32>,
    w2: Vec<f32>,
}

#[allow(dead_code)]
struct LayerCache {
    attn_caches: Vec<AttnCache>,
    normed1: Vec<Vec<f32>>,
    after_attn: Vec<Vec<f32>>,
    normed2: Vec<Vec<f32>>,
    h1: Vec<Vec<f32>>,
}

impl TransformerLayer {
    fn new(d_model: usize, n_heads: usize, seed: &mut u64) -> Self {
        let d_k = d_model / n_heads;
        let heads = (0..n_heads).map(|_| AttentionHead::new(d_model, d_k, seed)).collect();
        let d_ff = d_model * 4;
        Self {
            heads,
            d_model,
            w1: xavier_init(d_ff * d_model, d_model, d_ff, seed),
            w2: xavier_init(d_model * d_ff, d_ff, d_model, seed),
        }
    }

    fn forward(&self, xs: &[Vec<f32>]) -> (Vec<Vec<f32>>, LayerCache) {
        let seq = xs.len();
        let d_model = self.d_model;
        let d_ff = d_model * 4;

        let normed1: Vec<Vec<f32>> = xs.iter().map(|x| layer_norm(x, 1e-5)).collect();
        let mut attn_out = vec![vec![0.0f32; d_model]; seq];
        let mut attn_caches = Vec::new();

        for head in &self.heads {
            let (h, cache) = head.forward(&normed1);
            attn_caches.push(cache);
            for (i, row) in attn_out.iter_mut().enumerate() {
                for (r, v) in row.iter_mut().zip(h[i].iter()) {
                    *r += v;
                }
            }
        }

        let after_attn: Vec<Vec<f32>> = xs.iter().zip(attn_out.iter())
            .map(|(x, a)| x.iter().zip(a.iter()).map(|(xi, ai)| xi + ai).collect())
            .collect();

        let normed2: Vec<Vec<f32>> = after_attn.iter().map(|x| layer_norm(x, 1e-5)).collect();
        let mut h1_all = Vec::with_capacity(seq);
        let mut result = Vec::with_capacity(seq);

        for (i, x) in normed2.iter().enumerate() {
            let h1: Vec<f32> = matvec(&self.w1, d_ff, d_model, x).into_iter().map(relu).collect();
            h1_all.push(h1.clone());
            let h2 = matvec(&self.w2, d_model, d_ff, &h1);
            let res: Vec<f32> = after_attn[i].iter().zip(h2.iter()).map(|(a, b)| a + b).collect();
            result.push(res);
        }

        let cache = LayerCache { attn_caches, normed1, after_attn, normed2, h1: h1_all };
        (result, cache)
    }

    #[allow(dead_code)]
    fn backward(&self, dout: &[Vec<f32>], cache: &LayerCache) -> (Vec<Vec<f32>>, Vec<f32>, Vec<f32>) {
        let seq = dout.len();
        let d_model = self.d_model;
        let d_ff = d_model * 4;

        let mut d_w1 = vec![0.0f32; d_ff * d_model];
        let mut d_w2 = vec![0.0f32; d_model * d_ff];
        let mut d_after_attn = vec![vec![0.0f32; d_model]; seq];

        for (i, d_out_i) in dout.iter().enumerate() {
            let h2_grad = d_out_i;
            let mut d_h1 = vec![0.0f32; d_ff];
            for r in 0..d_model {
                for c in 0..d_ff {
                    d_w2[r * d_ff + c] += cache.h1[i][c] * h2_grad[r];
                    d_h1[c] += self.w2[r * d_ff + c] * h2_grad[r];
                }
            }

            let d_preact: Vec<f32> = d_h1.iter().zip(cache.h1[i].iter())
                .map(|(d, h)| if *h > 0.0 { *d } else { 0.0 })
                .collect();

            let d_normed2 = vec![0.0f32; d_model];

            for r in 0..d_ff {
                for c in 0..d_model {
                    d_w1[r * d_model + c] += d_preact[r] * cache.normed2[i][c];
                }
            }

            let d_after_attn_i: Vec<f32> = layer_norm_backward(&d_normed2, &cache.after_attn[i], 1e-5);
            for (j, (a, b)) in d_after_attn[i].iter_mut().zip(d_after_attn_i.iter()).enumerate() {
                *a += d_out_i[j] + b;
            }
        }

        let mut d_normed1 = vec![vec![0.0f32; d_model]; seq];
        for (hi, head) in self.heads.iter().enumerate() {
            let (d_input, _d_wq, _d_wk, _d_wv, _d_wo) = head.backward(&d_after_attn, &cache.attn_caches[hi]);
            for (n, inp) in d_normed1.iter_mut().zip(d_input.iter()) {
                for (nj, v) in n.iter_mut().zip(inp.iter()) {
                    *nj += v;
                }
            }
            let _ = hi;
        }

        let mut d_xs = vec![vec![0.0f32; d_model]; seq];
        for i in 0..seq {
            let d_norm = layer_norm_backward(&d_normed1[i], &cache.normed1[i], 1e-5);
            for (j, v) in d_xs[i].iter_mut().zip(d_norm.iter()) {
                *j += v;
            }
            for (j, v) in d_xs[i].iter_mut().zip(d_after_attn[i].iter()) {
                *j += v;
            }
        }

        (d_xs, d_w1, d_w2)
    }
}

pub struct RealIglaModel {
    pub vocab_size: usize,
    pub d_model: usize,
    pub d_ff: usize,
    pub n_heads: usize,
    pub n_layers: usize,
    pub max_seq_len: usize,
    pub embed: Vec<f32>,
    pub lm_head: Vec<f32>,
    pub layers: Vec<TransformerLayer>,
}

pub struct SelfAttentionCache;

#[allow(dead_code)]
struct ForwardCache {
    embed_out: Vec<Vec<f32>>,
    layer_caches: Vec<LayerCache>,
}

impl RealIglaModel {
    pub fn new(vocab_size: usize, d_model: usize, n_layers: usize) -> Self {
        let mut seed = 0x1337_c0de_u64;
        let d_ff = d_model * 4;
        let n_heads = (d_model / 64).max(1);

        let embed = xavier_init(vocab_size * d_model, vocab_size, d_model, &mut seed);
        let lm_head = xavier_init(vocab_size * d_model, d_model, vocab_size, &mut seed);
        let layers = (0..n_layers)
            .map(|_| TransformerLayer::new(d_model, n_heads, &mut seed))
            .collect();

        Self { vocab_size, d_model, d_ff, n_heads, n_layers, max_seq_len: 256, embed, lm_head, layers }
    }

    pub fn forward(&self, input_ids: &[usize], _cache: Option<&SelfAttentionCache>) -> Vec<Vec<f32>> {
        if input_ids.is_empty() {
            return vec![];
        }
        let d_model = self.d_model;
        let mut xs: Vec<Vec<f32>> = input_ids.iter().map(|&id| {
            let id = id.min(self.vocab_size - 1);
            self.embed[id * d_model..(id + 1) * d_model].to_vec()
        }).collect();

        for layer in &self.layers {
            let (new_xs, _) = layer.forward(&xs);
            xs = new_xs;
        }

        xs.iter().map(|x| matvec(&self.lm_head, self.vocab_size, d_model, x)).collect()
    }

    #[allow(dead_code)]
    fn forward_cached(&self, input_ids: &[usize]) -> (Vec<Vec<f32>>, ForwardCache) {
        let d_model = self.d_model;
        let embed_out: Vec<Vec<f32>> = input_ids.iter().map(|&id| {
            let id = id.min(self.vocab_size - 1);
            self.embed[id * d_model..(id + 1) * d_model].to_vec()
        }).collect();

        let mut xs = embed_out.clone();
        let mut layer_caches = Vec::new();

        for layer in &self.layers {
            let (new_xs, cache) = layer.forward(&xs);
            xs = new_xs;
            layer_caches.push(cache);
        }

        let logits: Vec<Vec<f32>> = xs.iter().map(|x| matvec(&self.lm_head, self.vocab_size, d_model, x)).collect();
        (logits, ForwardCache { embed_out, layer_caches })
    }

    pub fn loss_bpb(&self, tokens: &[usize]) -> (f32, f32) {
        if tokens.len() < 2 {
            return (0.0, 0.0);
        }
        let input = &tokens[..tokens.len() - 1];
        let targets = &tokens[1..];

        let logits = self.forward(input, None);
        let mut total_loss = 0.0f32;

        for (logit_row, &target) in logits.iter().zip(targets.iter()) {
            let mut probs = logit_row.clone();
            softmax(&mut probs);
            let p = probs[target.min(self.vocab_size - 1)].max(1e-10);
            total_loss -= p.ln();
        }

        let loss = total_loss / targets.len() as f32;
        (loss, loss / LN_2)
    }

    #[allow(clippy::needless_range_loop)]
    pub fn train_step(&mut self, tokens: &[usize], lr: f32) -> f32 {
        if tokens.len() < 3 {
            return 0.0;
        }
        let seq_len = tokens.len() - 1;
        let input = &tokens[..seq_len];
        let targets = &tokens[1..];
        let d_model = self.d_model;
        let d_ff = d_model * 4;

        // === FORWARD ===
        let embed_out: Vec<Vec<f32>> = input.iter().map(|&id| {
            let id = id.min(self.vocab_size - 1);
            self.embed[id * d_model..(id + 1) * d_model].to_vec()
        }).collect();

        let mut all_layer_io: Vec<LayerIO> = Vec::new();
        let mut xs = embed_out.clone();

        for layer in &self.layers {
            let normed1: Vec<Vec<f32>> = xs.iter().map(|x| layer_norm(x, 1e-5)).collect();
            let mut attn_out = vec![vec![0.0f32; d_model]; seq_len];
            let mut attn_caches = Vec::new();
            for head in &layer.heads {
                let (h, cache) = head.forward(&normed1);
                attn_caches.push(cache);
                for (i, row) in attn_out.iter_mut().enumerate() {
                    for (r, v) in row.iter_mut().zip(h[i].iter()) {
                        *r += v;
                    }
                }
            }
            let after_attn: Vec<Vec<f32>> = xs.iter().zip(attn_out.iter())
                .map(|(x, a)| x.iter().zip(a.iter()).map(|(xi, ai)| xi + ai).collect())
                .collect();
            let normed2: Vec<Vec<f32>> = after_attn.iter().map(|x| layer_norm(x, 1e-5)).collect();
            let mut h1_all = Vec::with_capacity(seq_len);
            let mut result = Vec::with_capacity(seq_len);
            for (i, x) in normed2.iter().enumerate() {
                let h1: Vec<f32> = matvec(&layer.w1, d_ff, d_model, x).into_iter().map(relu).collect();
                h1_all.push(h1.clone());
                let h2 = matvec(&layer.w2, d_model, d_ff, &h1);
                let res: Vec<f32> = after_attn[i].iter().zip(h2.iter()).map(|(a, b)| a + b).collect();
                result.push(res);
            }
            all_layer_io.push(LayerIO { input: xs, normed1, after_attn, normed2, h1: h1_all, attn_caches });
            xs = result;
        }

        let logits: Vec<Vec<f32>> = xs.iter().map(|x| matvec(&self.lm_head, self.vocab_size, d_model, x)).collect();

        // === LOSS + d_logits ===
        let mut d_logits = vec![vec![0.0f32; self.vocab_size]; seq_len];
        let mut total_loss = 0.0f32;
        for i in 0..seq_len {
            let mut probs = logits[i].clone();
            softmax(&mut probs);
            let target = targets[i].min(self.vocab_size - 1);
            total_loss -= probs[target].max(1e-10).ln();
            d_logits[i][..self.vocab_size].copy_from_slice(&probs[..self.vocab_size]);
            d_logits[i][target] -= 1.0;
        }
        let loss = total_loss / seq_len as f32;
        let scale = 1.0 / seq_len as f32;

        // === BACKPROP: lm_head ===
        let mut d_lm_head = vec![0.0f32; self.vocab_size * d_model];
        let mut d_out = vec![vec![0.0f32; d_model]; seq_len];
        for i in 0..seq_len {
            for j in 0..self.vocab_size {
                let dl = d_logits[i][j];
                for k in 0..d_model {
                    d_lm_head[j * d_model + k] += dl * xs[i][k];
                    d_out[i][k] += self.lm_head[j * d_model + k] * dl;
                }
            }
        }
        for (i, v) in self.lm_head.iter_mut().enumerate() {
            *v -= lr * scale * d_lm_head[i];
        }

        // === BACKPROP: transformer layers (reverse) ===
        for li in (0..self.layers.len()).rev() {
            let layer = &self.layers[li];
            let io = &all_layer_io[li];
            let n_heads = layer.heads.len();
            let d_k = d_model / n_heads;

            let mut d_w1 = vec![0.0f32; d_ff * d_model];
            let mut d_w2 = vec![0.0f32; d_model * d_ff];
            let mut d_after_attn = vec![vec![0.0f32; d_model]; seq_len];

            // FFN backward
            for i in 0..seq_len {
                let mut d_h1 = vec![0.0f32; d_ff];
                for r in 0..d_model {
                    for c in 0..d_ff {
                        d_w2[r * d_ff + c] += d_out[i][r] * io.h1[i][c];
                        d_h1[c] += layer.w2[r * d_ff + c] * d_out[i][r];
                    }
                }
                let d_preact: Vec<f32> = d_h1.iter().zip(io.h1[i].iter())
                    .map(|(d, h)| if *h > 0.0 { *d } else { 0.0 }).collect();
                for r in 0..d_ff {
                    for c in 0..d_model {
                        d_w1[r * d_model + c] += d_preact[r] * io.normed2[i][c];
                    }
                }
                let d_n2 = matvec_transpose(&layer.w1, d_ff, d_model, &d_preact);
                let d_aa = layer_norm_backward(&d_n2, &io.after_attn[i], 1e-5);
                for k in 0..d_model {
                    d_after_attn[i][k] = d_out[i][k] + d_aa[k];
                }
            }

            // Attention backward (simplified: just pass gradient through)
            let mut d_normed1 = vec![vec![0.0f32; d_model]; seq_len];
            for (hi, head) in layer.heads.iter().enumerate() {
                let ac = &io.attn_caches[hi];
                for i in 0..seq_len {
                    let q = &ac.qs[i];
                    let ks = &ac.ks;
                    let vs = &ac.vs;
                    let attn_scale = (d_k as f32).sqrt();

                    let mut scores: Vec<f32> = (0..=i)
                        .map(|j| q.iter().zip(ks[j].iter()).map(|(qi, ki)| qi * ki).sum::<f32>() / attn_scale)
                        .collect();
                    softmax(&mut scores);

                    let d_ctx = matvec_transpose(&head.wo, d_model, d_k, &d_after_attn[i]);

                    let mut d_scores = vec![0.0f32; scores.len()];
                    for j in 0..scores.len() {
                        for c in 0..d_k {
                            d_scores[j] += d_ctx[c] * vs[j][c];
                        }
                    }

                    let s_dot: f32 = d_scores.iter().zip(scores.iter()).map(|(d, s)| d * s).sum();
                    let mut d_q = vec![0.0f32; d_k];
                    for j in 0..scores.len() {
                        let ds = scores[j] * (d_scores[j] - s_dot) / attn_scale;
                        for c in 0..d_k {
                            d_q[c] += ds * ks[j][c];
                        }
                        let d_k_j: Vec<f32> = (0..d_k).map(|c| ds * q[c]).collect();
                        let d_v_j: Vec<f32> = (0..d_k).map(|c| scores[j] * d_ctx[c]).collect();
                        for r in 0..d_model {
                            for c in 0..d_k {
                                d_normed1[j][r] += head.wk[r * d_k + c] * d_k_j[c];
                            }
                        }
                        for r in 0..d_model {
                            for c in 0..d_k {
                                d_normed1[j][r] += head.wv[r * d_k + c] * d_v_j[c];
                            }
                        }
                    }
                    for r in 0..d_model {
                        for c in 0..d_k {
                            d_normed1[i][r] += head.wq[r * d_k + c] * d_q[c];
                        }
                    }
                }
            }

            // Layer norm 1 backward + residual
            let mut d_xs = vec![vec![0.0f32; d_model]; seq_len];
            for i in 0..seq_len {
                let d_ln1 = layer_norm_backward(&d_normed1[i], &io.input[i], 1e-5);
                for k in 0..d_model {
                    d_xs[i][k] = d_ln1[k] + d_after_attn[i][k];
                }
            }

            // Update weights
            let layer_mut = &mut self.layers[li];
            for (i, v) in layer_mut.w1.iter_mut().enumerate() {
                *v -= lr * scale * d_w1[i];
            }
            for (i, v) in layer_mut.w2.iter_mut().enumerate() {
                *v -= lr * scale * d_w2[i];
            }

            d_out = d_xs;
        }

        // === BACKPROP: embeddings ===
        for (i, &token_id) in input.iter().enumerate() {
            let id = token_id.min(self.vocab_size - 1);
            for k in 0..d_model {
                self.embed[id * d_model + k] -= lr * scale * d_out[i][k];
            }
        }

        loss
    }

    pub fn param_count(&self) -> usize {
        let embed = self.vocab_size * self.d_model;
        let lm_head = self.vocab_size * self.d_model;
        let mut layer_params = 0;
        for layer in &self.layers {
            let dm = self.d_model;
            let d_k = dm / self.n_heads;
            let d_ff = dm * 4;
            for _ in &layer.heads {
                layer_params += 4 * dm * d_k;
            }
            layer_params += d_ff * dm + dm * d_ff;
        }
        embed + lm_head + layer_params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_shape() {
        let model = RealIglaModel::new(256, 64, 1);
        let tokens = vec![1usize, 2, 3, 4];
        let logits = model.forward(&tokens, None);
        assert_eq!(logits.len(), 4);
        assert_eq!(logits[0].len(), 256);
    }

    #[test]
    fn test_loss_bpb_finite() {
        let model = RealIglaModel::new(256, 64, 1);
        let tokens: Vec<usize> = (0..16).map(|i| i % 256).collect();
        let (loss, bpb) = model.loss_bpb(&tokens);
        assert!(loss.is_finite());
        assert!(bpb.is_finite());
        assert!(bpb > 0.0);
    }

    #[test]
    fn test_train_step_reduces_loss() {
        let mut model = RealIglaModel::new(256, 64, 1);
        let tokens: Vec<usize> = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let (loss_before, _) = model.loss_bpb(&tokens);

        for _ in 0..5 {
            model.train_step(&tokens, 0.01);
        }

        let (loss_after, _) = model.loss_bpb(&tokens);
        assert!(loss_after.is_finite());
        assert!(loss_after < loss_before, "Loss should decrease: {} >= {}", loss_after, loss_before);
    }

    #[test]
    fn test_param_count() {
        let model = RealIglaModel::new(256, 64, 1);
        let count = model.param_count();
        assert!(count > 0);
        assert!(count < 1_000_000);
    }

    #[test]
    fn test_overfit_small_batch() {
        let mut model = RealIglaModel::new(64, 32, 2);
        let tokens: Vec<usize> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let (initial_loss, _) = model.loss_bpb(&tokens);
        for _ in 0..100 {
            model.train_step(&tokens, 0.01);
        }
        let (final_loss, _) = model.loss_bpb(&tokens);
        assert!(
            final_loss < initial_loss * 0.5,
            "Should overfit: {} vs {}",
            final_loss,
            initial_loss
        );
    }

    #[test]
    fn test_multi_layer_gradient_flow() {
        let mut model = RealIglaModel::new(128, 64, 4);
        let tokens: Vec<usize> = (0..32).map(|i| i % 128).collect();
        let (l1, _) = model.loss_bpb(&tokens);
        model.train_step(&tokens, 0.001);
        let (l2, _) = model.loss_bpb(&tokens);
        assert!(l2.is_finite(), "Loss NaN after train step");
        assert!(l2 != l1, "Loss unchanged — gradient not flowing");
    }
}
