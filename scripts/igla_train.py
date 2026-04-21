#!/usr/bin/env python3
"""
IGLA-GF16 Training Script for Parameter Golf
"""

import argparse
import math
import random
import time

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

class Config:
    vocab_size = 65
    d_model = 128
    bigram_vocab_size = 729
    bigram_dim = 64
    use_bigram = True
    use_smear_gate = True
    use_muon = False
    muon_weight_decay = 0.04
    batch_size = 4
    seq_len = 32
    iterations = 1000
    lr = 0.02
    weight_decay = 0.01
    val_every = 50
    data_path = "data/tinyshakespeare.txt"

class BigramHashEmbedding(nn.Module):
    def __init__(self, vocab_size: int, bigram_dim: int, model_dim: int):
        super().__init__()
        self.vocab_size = vocab_size
        self.embed = nn.Embedding(vocab_size, bigram_dim)
        self.proj = nn.Linear(bigram_dim, model_dim, bias=False) if bigram_dim != model_dim else None
        self.scale = nn.Parameter(torch.tensor(0.05, dtype=torch.float32))
        nn.init.zeros_(self.embed.weight)
        if self.proj is not None:
            nn.init.zeros_(self.proj.weight)

    def bigram_hash(self, tokens: torch.Tensor) -> torch.Tensor:
        t = tokens.to(torch.int32)
        mod_val = self.vocab_size - 1
        batch, seq = t.shape
        out = torch.zeros_like(t)
        out[:, 0] = mod_val
        if seq > 1:
            t_curr = t[:, 1:]
            t_prev = t[:, :-1]
            hash_val = torch.bitwise_xor(36313 * t_curr, 27191 * t_prev) % mod_val
            out[:, 1:] = hash_val
        return out.long()

    def forward(self, token_ids: torch.Tensor) -> torch.Tensor:
        h = self.embed(self.bigram_hash(token_ids))
        if self.proj is not None:
            h = self.proj(h)
        return h * self.scale

class SmearGate(nn.Module):
    def __init__(self, dim: int):
        super().__init__()
        self.gate = nn.Parameter(torch.zeros(dim, dtype=torch.float32))

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        g = torch.sigmoid(self.gate).view(1, 1, -1)
        x_prev = torch.cat([torch.zeros_like(x[:, :1]), x[:, :-1]], dim=1)
        return (1 - g) * x + g * x_prev

class RMSNorm(nn.Module):
    def __init__(self, dim: int, eps: float = 1e-6):
        super().__init__()
        self.scale = nn.Parameter(torch.ones(dim))
        self.eps = eps

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        rms = (x.square().mean(dim=-1, keepdim=True) + self.eps).sqrt()
        return x / rms * self.scale

class IGLAGF16Model(nn.Module):
    def __init__(self, config: Config):
        super().__init__()
        self.tok_emb = nn.Embedding(config.vocab_size, config.d_model)
        self.bigram = BigramHashEmbedding(config.bigram_vocab_size, config.bigram_dim, config.d_model) if config.use_bigram else None
        self.smear = SmearGate(config.d_model) if config.use_smear_gate else nn.Identity()
        self.norm = RMSNorm(config.d_model)

    def forward(self, tokens: torch.Tensor) -> torch.Tensor:
        x = self.tok_emb(tokens)
        if self.bigram is not None:
            x = x + self.bigram(tokens)
        x = self.smear(x)
        x = self.norm(x)
        return x @ self.tok_emb.weight.T

    def forward_with_loss(self, tokens: torch.Tensor):
        logits = self.forward(tokens[:, :-1])
        loss = F.cross_entropy(logits.reshape(-1, logits.size(-1)), tokens[:, 1:].reshape(-1))
        return logits, loss

class MuonOptimizer:
    def __init__(self, params, lr: float, momentum: float, weight_decay: float):
        self.params = list(params)
        self.lr = lr
        self.momentum = momentum
        self.weight_decay = weight_decay
        self.buffers = [None for _ in self.params]

    def step(self):
        for i, p in enumerate(self.params):
            if p.grad is None:
                continue
            if self.weight_decay > 0:
                p.data.mul_(1 - self.lr * self.weight_decay)
            grad = p.grad
            if self.buffers[i] is None:
                self.buffers[i] = grad.clone()
            else:
                self.buffers[i].mul_(self.momentum).add_(grad, alpha=1 - self.momentum)
            p.data.add_(self.buffers[i], alpha=-self.lr)

    def zero_grad(self):
        for p in self.params:
            if p.grad is not None:
                p.grad.zero_()

def load_tiny_shakespeare(path: str):
    with open(path, 'r') as f:
        text = f.read()
    chars = sorted(list(set(text)))
    vocab_size = len(chars)
    char_to_id = {c: i for i, c in enumerate(chars)}
    tokens = [char_to_id[c] for c in text]
    split = int(len(tokens) * 0.9)
    return tokens[:split], tokens[split:], vocab_size

def get_batch(tokens, batch_size, seq_len, device):
    tokens_per_batch = batch_size * (seq_len + 1)
    max_start = max(0, len(tokens) - tokens_per_batch)
    start = random.randint(0, max_start)
    batch_data = tokens[start:start + tokens_per_batch]
    if len(batch_data) < tokens_per_batch:
        batch_data = batch_data + [0] * (tokens_per_batch - len(batch_data))
    batch = torch.tensor(batch_data, dtype=torch.long, device=device).view(batch_size, seq_len + 1)
    return batch

def train(config: Config, seed: int = 42):
    random.seed(seed)
    torch.manual_seed(seed)
    np.random.seed(seed)
    device = torch.device('cpu')

    train_tokens, val_tokens, vocab_size = load_tiny_shakespeare(config.data_path)
    config.vocab_size = vocab_size

    print(f"Vocab: {vocab_size} Train: {len(train_tokens)} Val: {len(val_tokens)}")
    model = IGLAGF16Model(config).to(device)
    print(f"Params: {sum(p.numel() for p in model.parameters()):,}")

    if config.use_muon:
        optimizer = MuonOptimizer(model.parameters(), config.lr, 0.99, config.muon_weight_decay)
        print(f"Opt: Muon (wd={config.muon_weight_decay})")
    else:
        optimizer = torch.optim.AdamW(model.parameters(), lr=config.lr, weight_decay=config.weight_decay)
        print(f"Opt: AdamW (wd={config.weight_decay})")

    best_val_bpb = float('inf')
    for iter in range(config.iterations):
        batch = get_batch(train_tokens, config.batch_size, config.seq_len, device)
        optimizer.zero_grad()
        _, loss = model.forward_with_loss(batch)
        loss.backward()
        optimizer.step()

        if iter % 50 == 0 or iter == 0:
            print(f"iter {iter:4d}/{config.iterations:<4d} loss: {loss.item():.4f}")

        if iter % config.val_every == 0 and iter > 0:
            val_loss = evaluate(model, val_tokens, config.batch_size, config.seq_len, device)
            val_bpb = val_loss / math.log(2)
            print(f"  val BPB: {val_bpb:.4f} (best: {best_val_bpb:.4f})")
            if val_bpb < best_val_bpb:
                best_val_bpb = val_bpb
                print(f"  *** NEW BEST ***")
    return best_val_bpb

def evaluate(model, tokens, batch_size, seq_len, device):
    model.eval()
    total_loss = 0.0
    n = 0
    tokens_per_batch = batch_size * (seq_len + 1)
    with torch.no_grad():
        for i in range(0, len(tokens) - tokens_per_batch, tokens_per_batch):
            batch = torch.tensor(tokens[i:i+tokens_per_batch], dtype=torch.long, device=device).view(batch_size, seq_len + 1)
            _, loss = model.forward_with_loss(batch)
            total_loss += loss.item()
            n += 1
    model.train()
    return total_loss / max(n, 1)

def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    bgh301 = subparsers.add_parser("bgh301")
    bgh301.add_argument("--seed", type=int, default=42)
    bgh301.add_argument("--iterations", type=int, default=1000)
    muon105 = subparsers.add_parser("muon105")
    muon105.add_argument("--seed", type=int, default=42)
    muon105.add_argument("--iterations", type=int, default=1000)
    args = parser.parse_args()

    config = Config()
    if args.command == "bgh301":
        print("🧵 IGLA-BGH-301 [FOXTROT]")
    elif args.command == "muon105":
        print("🧵 IGLA-MUON-105 [ALFA]")
        config.use_bigram = False
        config.use_smear_gate = False
        config.use_muon = True

    config.iterations = args.iterations
    print(f"Seed: {args.seed}")
    best_bpb = train(config, args.seed)
    print(f"\n=== FINAL: Best BPB = {best_bpb:.4f} ===")

if __name__ == "__main__":
    main()
