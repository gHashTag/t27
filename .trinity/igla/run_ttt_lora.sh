#!/usr/bin/env bash
# IGLA-TTT-LoRA: Test-Time Training with LoRA launch script
# Agent: HOTEL
# Status: QUEUED - requires LoRA implementation in train_gpt.py
set -euo pipefail

cd "$(dirname "$0")/../.parameter-golf/parameter-golf"

export RUN_ID=igla_ttt_lora
export TIED_EMBED_LR=0.1
export NUM_LAYERS=10
export WEIGHT_DECAY=0.04
export SWA_ENABLED=1
export EVAL_STRIDE=64

# Note: Requires LoRA implementation
# export LORA_ENABLED=1
# export LORA_RANK=8
# export LORA_ALPHA=16
# export TTT_ENABLED=1
# export TTT_STEPS=50

echo "[IGLA-TTT-LoRA] WARNING: Requires LoRA implementation - QUEUED"
# torchrun --standalone --nproc_per_node=8 train_gpt.py
