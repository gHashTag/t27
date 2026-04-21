# IGLA NEEDLE HUNT Agent Dispatch

## Session
Date: 2026-04-21 08:30 UTC
Issue: #143

## Background
- Phase A: BPB=5.91 @ step 99 (full transformer, n_layers=1) ✅
- Phase B fine: BPB=6.56 @ step 300 (embedding-only) ❌
- Architecture mismatch identified: Phase B tested wrong model
- Decision: Return to Phase A config

## IGLA NEEDLE HUNT Launched
- Created TRI-CLI issue #168 (automation tool)
- Dispatched 3 NATO agents to P0 tasks:
  - FOXTROT → #183 (BigramHash + SmearGate)
  - GOLF → #184 (φ-OrthoInit + SWA)
  - ALFA → #185 (Muon WD=0.04)

## Target Stack
BigramHash(729) + SmearGate + φ-OrthoInit + SWA(1/φ) + Muon WD=0.04 + GF16 QAT + TTT-LoRA → BPB ≤ 1.10

## Decision Matrix
- Full transformer (n_layers=1) beats embedding-only (n_layers=0)
- Fix tri-cli compilation (21 errors) → then automate reporting
- Manual dispatch until tri-cli ready
