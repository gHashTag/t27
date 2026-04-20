#!/bin/bash
# Test real BPB after Issue #55 fix

echo "Testing Real BPB (Issue #55 Fix)"
echo "Expected: 1.5-5.0 range for 1000 steps"
echo ""

cargo build --release -p trios-train-cpu 2>&1 | tail -3
echo ""

# Create test config
cat > test_config.txt << 'CONFIG'
max_steps = 100
batch_size = 4
seq_len = 128
learning_rate = 0.0003
warmup_steps = 21
grad_clip = 0.618
log_every = 20
d_model = 144
n_heads = 8
d_ffn = 233
CONFIG

echo "Running 100-step training with real forward pass..."
echo ""

# Run training
./target/release/tri train-cpu test_config.txt 2>&1 | tail -30

echo ""
echo "Real BPB Test Complete"
echo "If BPB is still 0.0000, fix failed."
echo "If BPB is 1.5-5.0, fix succeeded."
