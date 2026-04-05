"""
HSLM (Hierarchical Sacred Learning Model) - Simple Example
Demonstrates a minimal HSLM-style model with ternary weights
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from t27 import K3Embedding, K3LayerNorm, K3Linear, K3Model

print("=" * 50)
print("HSLM Simple Model Demo")
print("=" * 50)
print()

# Configuration
VOCAB_SIZE = 100
EMBED_DIM = 27  # Small for demo
NUM_LAYERS = 2
NUM_HEADS = 3
SEQ_LEN = 16
BATCH_SIZE = 4

print(f"Configuration:")
print(f"  Vocab Size:    {VOCAB_SIZE}")
print(f"  Embed Dim:     {EMBED_DIM}")
print(f"  Num Layers:    {NUM_LAYERS}")
print(f"  Num Heads:     {NUM_HEADS}")
print(f"  Sequence Len:  {SEQ_LEN}")
print(f"  Batch Size:    {BATCH_SIZE}")
print()

# Create the model
print("Creating HSLM model...")
model = K3Model(
    vocab_size=VOCAB_SIZE,
    embed_dim=EMBED_DIM,
    num_layers=NUM_LAYERS,
    num_heads=NUM_HEADS,
)
print("Model created!")
print()

# Count parameters
total_params = sum(p.numel() for p in model.parameters())
print(f"Total parameters: {total_params:,}")
print()

# Count ternary weights (in K3 layers)
ternary_params = 0
for name, module in model.named_modules():
    if hasattr(module, 'weight_ternary'):
        trits = module.weight_ternary.numel()
        sparsity = (module.weight_ternary == 0).sum().item() / trits * 100
        ternary_params += trits
        print(f"  {name}: {trits:,} trits ({sparsity:.1f}% sparse)")
print(f"Total ternary weights: {ternary_params:,} trits")
print()

# Create sample input
input_ids = torch.randint(0, VOCAB_SIZE, (BATCH_SIZE, SEQ_LEN))
print(f"Input shape: {input_ids.shape}")
print(f"Sample input: {input_ids[0, :8].tolist()}...")
print()

# Forward pass
print("Running forward pass...")
model.eval()
with torch.no_grad():
    logits = model(input_ids)

print(f"Output shape: {logits.shape}")
print(f"Output logits range: [{logits.min():.2f}, {logits.max():.2f}]")
print()

# Get predictions
probs = F.softmax(logits, dim=-1)
predictions = probs.argmax(dim=-1)
print(f"Predictions shape: {predictions.shape}")
print(f"Sample predictions: {predictions[0, :8].tolist()}...")
print()

# Demonstrate ternary weight inspection
print("=" * 50)
print("Ternary Weight Inspection")
print("=" * 50)
print()

# Check token embedding weights
token_emb = model.token_embedding
print("Token Embedding Weights:")
print(f"  Continuous shape: {token_emb.weight_continuous.shape}")
print(f"  Ternary shape:    {token_emb.weight_ternary.shape}")
print(f"  Value distribution:")
for val in [-1, 0, 1]:
    count = (token_emb.weight_ternary == val).sum().item()
    pct = count / token_emb.weight_ternary.numel() * 100
    print(f"    {val:2d}: {count:6,} ({pct:5.2f}%)")
print()

# Check a specific K3Linear layer in first block
first_block = model.blocks[0]
first_ffn = first_block.ffn
first_linear = first_ffn[0]  # First linear in FFN

print("First FFN Layer (K3Linear):")
print(f"  Continuous shape: {first_linear.weight_continuous.shape}")
print(f"  Ternary shape:    {first_linear.weight_ternary.shape}")
print(f"  Value distribution:")
for val in [-1, 0, 1]:
    count = (first_linear.weight_ternary == val).sum().item()
    pct = count / first_linear.weight_ternary.numel() * 100
    print(f"    {val:2d}: {count:6,} ({pct:5.2f}%)")
print()

# Count non-zero connections
active_weights = (first_linear.weight_ternary != 0).sum().item()
total_weights = first_linear.weight_ternary.numel()
print(f"  Active connections: {active_weights}/{total_weights} ({active_weights/total_weights*100:.1f}%)")
print()

# Demonstrate weight update (simulating training step)
print("=" * 50)
print("Simulating Training Step")
print("=" * 50)
print()

# Before update
before_pos = (first_linear.weight_ternary == 1).sum().item()
before_neg = (first_linear.weight_ternary == -1).sum().item()
before_zero = (first_linear.weight_ternary == 0).sum().item()

print(f"Before quantization:")
print(f"  POS  (+1): {before_pos:,}")
print(f"  ZERO ( 0): {before_zero:,}")
print(f"  NEG  (-1): {before_neg:,}")
print()

# Simulate gradient update (add some noise to continuous weights)
with torch.no_grad():
    noise = torch.randn_like(first_linear.weight_continuous) * 0.01
    first_linear.weight_continuous.add_(noise)

# Re-quantize
first_linear.update_ternary_weights()

# After update
after_pos = (first_linear.weight_ternary == 1).sum().item()
after_neg = (first_linear.weight_ternary == -1).sum().item()
after_zero = (first_linear.weight_ternary == 0).sum().item()

print(f"After quantization:")
print(f"  POS  (+1): {after_pos:,}")
print(f"  ZERO ( 0): {after_zero:,}")
print(f"  NEG  (-1): {after_neg:,}")
print()

changed = abs(after_pos - before_pos) + abs(after_neg - before_neg) + abs(after_zero - before_zero)
print(f"Weights changed: {changed:,}")
print()

# Restraint demonstration
print("=" * 50)
print("Restraint Demonstration")
print("=" * 50)
print()

# Count ZERO weights as "restraint"
zero_fraction = (first_linear.weight_ternary == 0).sum().item() / first_linear.weight_ternary.numel()
print(f"Restraint (ZERO weights): {zero_fraction*100:.1f}%")
print()
print("Interpretation:")
print("  - ZERO represents 'uncertainty' or 'restraint'")
print("  - These weights are inactive, representing bounded rationality")
print("  - During inference, they contribute nothing to computation")
print()

print("=" * 50)
print("Demo Complete!")
print("=" * 50)
