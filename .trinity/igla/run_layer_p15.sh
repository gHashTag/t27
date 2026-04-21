#!/usr/bin/env bash
# IGLA-LAYER-P15: Layer weight sharing launch script
# Agent: INDIA
# Status: QUEUED - requires layer sharing implementation in train_gpt.py
set -euo pipefail

cd "$(dirname "$0")/../.parameter-golf/parameter-golf"

export RUN_ID=igla_layer_p15
export TIED_EMBED_LR=0.1
export NUM_UNIQUE_LAYERS=5
export LAYER_REPEATS=4
export WEIGHT_DECAY=0.04
export SWA_ENABLED=1
export EVAL_STRIDE=64

# Note: Requires layer sharing implementation
# export LAYER_SHARING_ENABLED=1

echo "[IGLA-LAYER-P15] WARNING: Requires layer sharing implementation - QUEUED"
# torchrun --standalone --nproc_per_node=8 train_gpt.py
