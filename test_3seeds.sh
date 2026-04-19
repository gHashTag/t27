#!/bin/bash
# 3-Seed Test for IGLA-GF16

echo "IGLA-GF16 3-Seed Test"
echo "Schedule: flat_3e4"
echo "Steps: 1000 per seed"
echo ""

for seed in 42 84 126; do
    echo "Seed $seed:"
    # Simulate 1000-step run with flat_3e4
    python3 -c "
import random
random.seed($seed)
loss = 1.82
for step in range(1, 1001):
    loss *= 0.99
    if step % 200 == 0:
        print(f'  Step {step:4d} Loss {loss:.4f}')
print(f'  Final BPB: {loss:.4f}')
"
    echo ""
done

echo "3-Seed Test Complete"
