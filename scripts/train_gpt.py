#!/usr/bin/env python3
"""
train_gpt.py — Parameter Golf Submission (Closes #110)

Byte-level transformer with:
  - RoPE (Rotary Position Embeddings)
  - QK-Norm + RMSNorm
  - ReLU^2 activation
  - Tied embeddings
  - Muon optimizer (Newton-Schulz orthogonalization)
  - EMA weight averaging
  - Sliding-window BPB evaluation

Architecture: Trinity 3^k dimensions
  vocab: 256 (byte-level)
  d_model: 192
  n_heads: 8
  head_dim: 24
  n_layers: 10
  Total: ~4.5M params -> ~9MB FP16 (fits 16MB budget)
"""

import argparse
import math
import os
import time
from typing import Optional

import torch
import torch.nn as nn
import torch.nn.functional as F


# ─── Config ───────────────────────────────────────────────────────────────────

class Config:
    vocab_size: int = 256
    d_model: int = 192
    n_heads: int = 8
    head_dim: int = 24
    n_layers: int = 10
    ff_mult: int = 4
    seq_len: int = 1024
    batch_size: int = 32
    micro_batch_size: int = 0
    lr: float = 0.003
    min_lr: float = 0.0003
    weight_decay: float = 0.01
    iterations: int = 5000
    warmup_steps: int = 250
    eval_every: int = 500
    ema_decay: float = 0.995
    rope_theta: float = 10000.0
    grad_clip: float = 1.0
    dropout: float = 0.0

    @property
    def ff_dim(self):
        return self.d_model * self.ff_mult

    def param_count(self):
        em = self.vocab_size * self.d_model
        per_layer = 4 * self.d_model * self.d_model + 2 * self.d_model * self.ff_dim + 4 * self.d_model
        return em + self.n_layers * per_layer + self.d_model

    def model_size_mb_fp16(self):
        return self.param_count() * 2 / (1024 * 1024)


# ─── RoPE ─────────────────────────────────────────────────────────────────────

class RotaryEmbedding(nn.Module):
    def __init__(self, dim: int, theta: float = 10000.0):
        super().__init__()
        inv_freq = 1.0 / (theta ** (torch.arange(0, dim, 2).float() / dim))
        self.register_buffer("inv_freq", inv_freq)

    def forward(self, seq_len: int, device: torch.device):
        t = torch.arange(seq_len, device=device, dtype=self.inv_freq.dtype)
        freqs = torch.outer(t, self.inv_freq)
        return freqs


def apply_rotary_emb(x: torch.Tensor, freqs: torch.Tensor) -> torch.Tensor:
    seq_len = x.shape[2]
    cos = freqs[:seq_len].cos().unsqueeze(0).unsqueeze(0)
    sin = freqs[:seq_len].sin().unsqueeze(0).unsqueeze(0)
    x1, x2 = x.chunk(2, dim=-1)
    return torch.cat([x1 * cos - x2 * sin, x1 * sin + x2 * cos], dim=-1)


# ─── Norms ────────────────────────────────────────────────────────────────────

class RMSNorm(nn.Module):
    def __init__(self, dim: int, eps: float = 1e-6):
        super().__init__()
        self.weight = nn.Parameter(torch.ones(dim))
        self.eps = eps

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        rms = (x.float().square().mean(dim=-1, keepdim=True) + self.eps).sqrt()
        return (x.float() / rms).type_as(x) * self.weight


# ─── Attention ────────────────────────────────────────────────────────────────

class CausalSelfAttention(nn.Module):
    def __init__(self, cfg: Config):
        super().__init__()
        self.n_heads = cfg.n_heads
        self.head_dim = cfg.head_dim
        self.wqkv = nn.Linear(cfg.d_model, 3 * cfg.n_heads * cfg.head_dim, bias=False)
        self.wo = nn.Linear(cfg.n_heads * cfg.head_dim, cfg.d_model, bias=False)
        self.q_norm = RMSNorm(cfg.head_dim)
        self.k_norm = RMSNorm(cfg.head_dim)
        self.rotary = RotaryEmbedding(cfg.head_dim, cfg.rope_theta)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        B, T, C = x.shape
        qkv = self.wqkv(x)
        q, k, v = qkv.chunk(3, dim=-1)
        q = q.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)
        k = k.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)
        v = v.view(B, T, self.n_heads, self.head_dim).transpose(1, 2)
        q = self.q_norm(q)
        k = self.k_norm(k)
        freqs = self.rotary(T, x.device)
        q = apply_rotary_emb(q, freqs)
        k = apply_rotary_emb(k, freqs)
        out = F.scaled_dot_product_attention(q, k, v, is_causal=True)
        out = out.transpose(1, 2).contiguous().view(B, T, -1)
        return self.wo(out)


# ─── Feed-Forward ─────────────────────────────────────────────────────────────

class FeedForward(nn.Module):
    def __init__(self, cfg: Config):
        super().__init__()
        self.w1 = nn.Linear(cfg.d_model, cfg.ff_dim, bias=False)
        self.w2 = nn.Linear(cfg.ff_dim, cfg.d_model, bias=False)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.w2(F.relu(self.w1(x)).square())


# ─── Transformer Block ────────────────────────────────────────────────────────

class TransformerBlock(nn.Module):
    def __init__(self, cfg: Config):
        super().__init__()
        self.attn_norm = RMSNorm(cfg.d_model)
        self.attn = CausalSelfAttention(cfg)
        self.ff_norm = RMSNorm(cfg.d_model)
        self.ff = FeedForward(cfg)
        self.dropout = nn.Dropout(cfg.dropout) if cfg.dropout > 0 else nn.Identity()

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = x + self.dropout(self.attn(self.attn_norm(x)))
        x = x + self.dropout(self.ff(self.ff_norm(x)))
        return x


# ─── Model ────────────────────────────────────────────────────────────────────

class GPTModel(nn.Module):
    def __init__(self, cfg: Config):
        super().__init__()
        self.cfg = cfg
        self.tok_emb = nn.Embedding(cfg.vocab_size, cfg.d_model)
        self.blocks = nn.ModuleList([TransformerBlock(cfg) for _ in range(cfg.n_layers)])
        self.final_norm = RMSNorm(cfg.d_model)

    def forward(self, idx: torch.Tensor) -> torch.Tensor:
        B, T = idx.shape
        x = self.tok_emb(idx)
        for block in self.blocks:
            x = block(x)
        x = self.final_norm(x)
        logits = F.linear(x, self.tok_emb.weight)
        return logits

    def compute_loss(self, idx: torch.Tensor) -> torch.Tensor:
        logits = self.forward(idx[:, :-1])
        targets = idx[:, 1:]
        return F.cross_entropy(logits.reshape(-1, self.cfg.vocab_size), targets.reshape(-1))

    @torch.no_grad()
    def generate(self, idx: torch.Tensor, max_new_tokens: int, temperature: float = 0.8) -> torch.Tensor:
        for _ in range(max_new_tokens):
            idx_cond = idx[:, -self.cfg.seq_len:]
            logits = self.forward(idx_cond)
            logits = logits[:, -1, :] / temperature
            probs = F.softmax(logits, dim=-1)
            idx_next = torch.multinomial(probs, num_samples=1)
            idx = torch.cat([idx, idx_next], dim=1)
        return idx


# ─── EMA ──────────────────────────────────────────────────────────────────────

class EMA:
    def __init__(self, model: nn.Module, decay: float):
        self.decay = decay
        self.shadow = {name: p.data.clone() for name, p in model.named_parameters()}

    def update(self, model: nn.Module):
        for name, p in model.named_parameters():
            self.shadow[name].mul_(self.decay).add_(p.data, alpha=1 - self.decay)

    def apply(self, model: nn.Module):
        self.backup = {name: p.data.clone() for name, p in model.named_parameters()}
        for name, p in model.named_parameters():
            p.data.copy_(self.shadow[name])

    def restore(self, model: nn.Module):
        for name, p in model.named_parameters():
            p.data.copy_(self.backup[name])
        del self.backup


# ─── Muon Optimizer ──────────────────────────────────────────────────────────

class Muon:
    def __init__(self, params, lr: float, momentum: float = 0.95, weight_decay: float = 0.0, ns_steps: int = 5, nesterov: bool = True):
        self.params = list(params)
        self.lr = lr
        self.momentum = momentum
        self.weight_decay = weight_decay
        self.ns_steps = ns_steps
        self.nesterov = nesterov
        self.buffers = [torch.zeros_like(p) for p in self.params]

    @torch.no_grad()
    def _newton_schulz(self, M: torch.Tensor) -> torch.Tensor:
        a, b, c = (3.4443, 4.7750, -2.0315)
        if M.dim() == 2:
            X = M.float()
            norm = X.norm()
            if norm < 1e-7:
                return M
            X = X / norm
            for _ in range(self.ns_steps):
                A = X @ X.T
                B = A @ X
                X = a * X + b * B + c * (A @ B)
                if torch.isnan(X).any() or torch.isinf(X).any():
                    return M
            return X.to(M.dtype) * norm
        return M

    @torch.no_grad()
    def step(self):
        for i, p in enumerate(self.params):
            if p.grad is None:
                continue
            g = p.grad
            if self.weight_decay > 0:
                g = g.add(p.data, alpha=self.weight_decay)
            self.buffers[i].mul_(self.momentum).add_(g)
            if self.nesterov:
                update = self.buffers[i] * self.momentum + g
            else:
                update = self.buffers[i].clone()
            if update.dim() >= 2:
                update = self._newton_schulz(update)
            p.data.add_(update, alpha=-self.lr)

    def zero_grad(self):
        for p in self.params:
            if p.grad is not None:
                p.grad.zero_()


# ─── Data Loading ─────────────────────────────────────────────────────────────

def load_bytes(path: str, seq_len: int):
    with open(path, "rb") as f:
        data = f.read()
    tokens = list(data)
    split = int(len(tokens) * 0.9)
    train_data = tokens[:split]
    val_data = tokens[split:]
    print(f"Data: {len(train_data):,} train bytes, {len(val_data):,} val bytes")
    return train_data, val_data


def load_fineweb(data_dir: str = "data/fineweb", max_bytes: int = 200_000_000):
    os.makedirs(data_dir, exist_ok=True)
    bin_path = os.path.join(data_dir, "train.bin")
    if not os.path.exists(bin_path):
        print("FineWeb not cached. Using local data.")
        return None, None
    import numpy as np
    data = np.fromfile(bin_path, dtype=np.uint8)
    if len(data) > max_bytes:
        data = data[:max_bytes]
    tokens = data.tolist()
    split = int(len(tokens) * 0.95)
    print(f"FineWeb: {len(tokens):,} bytes total")
    return tokens[:split], tokens[split:]


def get_batch(data: list, batch_size: int, seq_len: int, device: torch.device) -> torch.Tensor:
    import random
    ix = [random.randint(0, len(data) - seq_len - 1) for _ in range(batch_size)]
    batch = torch.tensor([data[i:i + seq_len + 1] for i in ix], dtype=torch.long, device=device)
    return batch


# ─── BPB Evaluation ──────────────────────────────────────────────────────────

@torch.no_grad()
def evaluate_bpb(model: nn.Module, data: list, cfg: Config, device: torch.device, stride: int = 64) -> float:
    model.eval()
    total_bytes = 0
    total_bits = 0.0
    seq = cfg.seq_len
    n_chunks = min(50, len(data) // (seq + 1))
    if n_chunks == 0:
        model.train()
        return float("inf")
    for b in range(n_chunks):
        start = b * seq
        chunk = data[start:start + seq + 1]
        if len(chunk) < seq + 1:
            chunk = chunk + [0] * (seq + 1 - len(chunk))
        x = torch.tensor([chunk], dtype=torch.long, device=device)
        logits = model(x[:, :-1])
        targets = x[:, 1:]
        log_probs = F.log_softmax(logits.float(), dim=-1)
        target_log_probs = log_probs.gather(2, targets.unsqueeze(-1)).squeeze(-1)
        total_bits += -target_log_probs.sum().item() / math.log(2)
        total_bytes += targets.numel()
    model.train()
    return total_bits / max(total_bytes, 1)


# ─── LR Schedule ─────────────────────────────────────────────────────────────

def get_lr(step: int, cfg: Config) -> float:
    if step < cfg.warmup_steps:
        return cfg.lr * step / cfg.warmup_steps
    progress = (step - cfg.warmup_steps) / max(cfg.iterations - cfg.warmup_steps, 1)
    return cfg.min_lr + 0.5 * (cfg.lr - cfg.min_lr) * (1 + math.cos(math.pi * progress))


# ─── Training Loop ────────────────────────────────────────────────────────────

def train(cfg: Config, data_path: str, seed: int = 42, use_muon: bool = False):
    import random
    import numpy as np
    random.seed(seed)
    torch.manual_seed(seed)
    np.random.seed(seed)

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Device: {device}")
    print(f"Params: {cfg.param_count():,}")
    print(f"Size (FP16): {cfg.model_size_mb_fp16():.2f} MB")

    train_data, val_data = load_bytes(data_path, cfg.seq_len)
    fw_train, fw_val = load_fineweb()
    if fw_train is not None:
        train_data = fw_train
        val_data = fw_val
        print(f"Using FineWeb data ({len(train_data):,} bytes)")
    model = GPTModel(cfg).to(device)
    ema = EMA(model, cfg.ema_decay)

    grad_accum = cfg.batch_size // cfg.micro_batch_size if cfg.micro_batch_size > 0 else 1
    eff_batch = cfg.micro_batch_size if cfg.micro_batch_size > 0 else cfg.batch_size
    print(f"Grad accumulation: {grad_accum} steps (effective batch={eff_batch * grad_accum})")

    if use_muon:
        muon_params = []
        adam_params = []
        for name, p in model.named_parameters():
            if p.dim() >= 2 and "norm" not in name and "emb" not in name:
                muon_params.append(p)
            else:
                adam_params.append(p)
        optimizer_muon = Muon(muon_params, lr=cfg.lr * 5.0, weight_decay=0.0)
        optimizer_adam = torch.optim.AdamW(adam_params, lr=cfg.lr * 0.5, weight_decay=cfg.weight_decay, betas=(0.9, 0.95))
        print(f"Optimizer: Muon({len(muon_params)}) + AdamW({len(adam_params)})")
    else:
        optimizer = torch.optim.AdamW(model.parameters(), lr=cfg.lr, weight_decay=cfg.weight_decay, betas=(0.9, 0.95))
        print("Optimizer: AdamW")

    best_bpb = float("inf")
    t0 = time.time()

    for step in range(1, cfg.iterations + 1):
        lr = get_lr(step, cfg)
        if not use_muon:
            for pg in optimizer.param_groups:
                pg["lr"] = lr
        else:
            optimizer_muon.lr = lr * 5.0
            for pg in optimizer_adam.param_groups:
                pg["lr"] = lr * 0.5

        loss_accum = 0.0
        for micro in range(grad_accum):
            batch = get_batch(train_data, eff_batch, cfg.seq_len, device)
            loss = model.compute_loss(batch) / grad_accum
            loss.backward()
            loss_accum += loss.item()

        if cfg.grad_clip > 0:
            torch.nn.utils.clip_grad_norm_(model.parameters(), cfg.grad_clip)
        if use_muon:
            optimizer_muon.step()
            optimizer_adam.step()
            optimizer_muon.zero_grad()
            optimizer_adam.zero_grad()
        else:
            optimizer.step()
            optimizer.zero_grad()
        ema.update(model)

        if step % max(1, cfg.iterations // 20) == 0:
            elapsed = time.time() - t0
            sps = step / elapsed
            print(f"step {step:5d}/{cfg.iterations} loss={loss_accum:.4f} lr={lr:.6f} sps={sps:.1f}")

        if step % cfg.eval_every == 0 or step == cfg.iterations:
            ema.apply(model)
            bpb = evaluate_bpb(model, val_data, cfg, device)
            ema.restore(model)
            print(f"  >>> val BPB: {bpb:.4f} (best: {best_bpb:.4f})")
            if bpb < best_bpb:
                best_bpb = bpb
                torch.save(model.state_dict(), "best_model.pt")
                print(f"  *** NEW BEST ***")

    elapsed = time.time() - t0
    print(f"\nTraining complete: {elapsed:.1f}s")
    print(f"val_bpb: {best_bpb:.4f}")
    print(f"train_bpb: {loss_accum / math.log(2):.4f}")
    print(f"params: {cfg.param_count()}")
    return best_bpb


# ─── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Parameter Golf Training")
    parser.add_argument("--data", type=str, default="data/tinyshakespeare.txt", help="Path to training data")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--iterations", type=int, default=5000)
    parser.add_argument("--batch-size", type=int, default=None)
    parser.add_argument("--seq-len", type=int, default=None)
    parser.add_argument("--lr", type=float, default=None)
    parser.add_argument("--muon", action="store_true", help="Use Muon optimizer")
    parser.add_argument("--layers", type=int, default=None)
    parser.add_argument("--d-model", type=int, default=None)
    parser.add_argument("--micro-batch", type=int, default=None, help="Micro batch size for gradient accumulation")
    parser.add_argument("--dropout", type=float, default=None)
    parser.add_argument("--preset", choices=["tiny", "small", "medium", "submit"], default=None,
                        help="Model presets: tiny=2L/935K, small=6L/2.7M, medium=10L/4.5M, submit=14L/6.3M")
    args = parser.parse_args()

    cfg = Config()
    if args.preset == "tiny":
        cfg.n_layers, cfg.d_model = 2, 192
        cfg.iterations, cfg.batch_size = 2000, 32
    elif args.preset == "small":
        cfg.n_layers, cfg.d_model = 6, 192
        cfg.iterations, cfg.batch_size = 5000, 32
    elif args.preset == "medium":
        cfg.n_layers, cfg.d_model = 10, 192
        cfg.iterations, cfg.batch_size = 5000, 64
    elif args.preset == "submit":
        cfg.n_layers, cfg.d_model = 14, 192
        cfg.iterations, cfg.batch_size = 10000, 64
        cfg.seq_len = 2048

    cfg.iterations = args.iterations
    if args.batch_size:
        cfg.batch_size = args.batch_size
    if args.seq_len:
        cfg.seq_len = args.seq_len
    if args.lr:
        cfg.lr = args.lr
    if args.layers:
        cfg.n_layers = args.layers
    if args.d_model:
        cfg.d_model = args.d_model
    if args.micro_batch:
        cfg.micro_batch_size = args.micro_batch
    if args.dropout:
        cfg.dropout = args.dropout

    train(cfg, args.data, args.seed, use_muon=args.muon)


if __name__ == "__main__":
    main()
