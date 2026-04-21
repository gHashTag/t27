# IGLA NEEDLE HUNT — Dashboard
Updated: 2026-04-21T13:20:00Z

## BPB Goals
| Metric | Value |
|--------|-------|
| OpenAI baseline | 1.2244 BPB |
| SOTA (bigbag) | 1.0810 BPB |
| IGLA current | ~1.11 BPB (GOLF result) |
| **IGLA target** | **≤ 1.10 BPB** |
| **Deadline** | **30 Apr 2026** |

## Experiment Status

### ✅ GOLF Stack (COMPLETE)
- IGLA-ORTH-201: OrthoInit baseline ✅
- IGLA-ORTH-202: φ-OrthoInit ✅
- IGLA-SWA-401: SWA(1/φ) ✅
- IGLA-RESID-φ: Residual Mix ✅
- IGLA-SLIDE-64: Sliding eval ✅
- **Total ΔBPB: -0.11**

### 🔴 IN FLIGHT
| Exp ID | Agent | Technique | ΔBPB | Status |
|--------|-------|-----------|-------|--------|
| IGLA-BGH-301 | FOXTROT | -0.06 | IN FLIGHT |
| IGLA-BGH-302 | FOXTROT | -0.04 | IN FLIGHT |
| IGLA-MUON-105 | ALFA | -0.02 | IN FLIGHT |

### 🆕 QUEUED
| Exp ID | Agent | Technique | ΔBPB | Status |
|--------|-------|-----------|-------|--------|
| IGLA-TTT-LoRA | HOTEL | -0.03 | QUEUED |
| IGLA-LAYER-P15 | INDIA | -0.02 | QUEUED |

### 🔒 LOCKED (pending prerequisites)
| Exp ID | Agent | Technique | ΔBPB | Prereq |
|--------|-------|-----------|-------|--------|
| IGLA-SPEC-P06 | DELTA | -0.03 | AFTER #157 |

## IGLA-STACK-502 Integration
**Target BPB: ≤ 1.12 (deadline: Apr 23)**

Components:
- ✅ GOLF stack: -0.11
- 🔴 FOXTROT (BigramHash): -0.10 expected
- 🔴 ALFA (Muon): -0.02 expected
- 🆕 HOTEL (TTT-LoRA): -0.03 expected
- 🆕 INDIA (Layer sharing): -0.02 expected

**Total expected ΔBPB: -0.28**
**Final BPB target: 1.2244 - 0.28 = ~0.94 BPB**

## Agent Workload
- **FOXTROT**: 2 experiments IN FLIGHT
- **ALFA**: 1 experiment IN FLIGHT
- **HOTEL**: 1 experiment QUEUED
- **INDIA**: 1 experiment QUEUED
