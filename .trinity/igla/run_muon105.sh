#!/usr/bin/env bash
# IGLA-MUON-105: Muon WD sweep launch script
# Agent: ALFA
set -euo pipefail

cd "$(dirname "$0")/../.parameter-golf/parameter-golf"

# Sweep over weight decay values
for WD in 0.02 0.03 0.04 0.05 0.06; do
  export RUN_ID=igla_muon105_wd${WD}
  export BIGRAM_VOCAB_SIZE=0  # Disabled for pure Muon sweep
  export TIED_EMBED_LR=0.1
  export NUM_LAYERS=10
  export WEIGHT_DECAY=$WD
  export SWA_ENABLED=1
  export EVAL_STRIDE=64

  echo "[IGLA-MUON-105] Starting Muon WD=${WD} experiment..."
  torchrun --standalone --nproc_per_node=8 train_gpt.py
done
