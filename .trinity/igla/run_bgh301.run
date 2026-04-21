#!/usr/bin/env bash
# IGLA-BGH-301: BigramHash(729) launch script
# Agent: FOXTROT
set -euo pipefail

cd "$(dirname "$0")/../.parameter-golf/parameter-golf"

export RUN_ID=igla_bgh301
export BIGRAM_VOCAB_SIZE=729
export BIGRAM_DIM=96
export TIED_EMBED_LR=0.1
export NUM_LAYERS=10
export WEIGHT_DECAY=0.04
export SWA_ENABLED=1
export EVAL_STRIDE=64

echo "[IGLA-BGH-301] Starting BigramHash(729) experiment..."
torchrun --standalone --nproc_per_node=8 train_gpt.py
