#!/usr/bin/env bash
# IGLA-BGH-302: BigramHash(10240) + SmearGate launch script
# Agent: FOXTROT
set -euo pipefail

cd "$(dirname "$0")/../.parameter-golf/parameter-golf"

export RUN_ID=igla_bgh302
export BIGRAM_VOCAB_SIZE=10240
export BIGRAM_DIM=128
export TIED_EMBED_LR=0.1
export NUM_LAYERS=10
export WEIGHT_DECAY=0.04
export SWA_ENABLED=1
export EVAL_STRIDE=64

echo "[IGLA-BGH-302] Starting BigramHash(10240)+SmearGate experiment..."
torchrun --standalone --nproc_per_node=8 train_gpt.py
