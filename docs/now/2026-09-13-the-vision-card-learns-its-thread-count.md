# NOW -- The vision card learns its thread count (2026-09-13)

## The vision card learns its thread count (Refs #3607)

- `specs/system/unified-agent-operations.t27` goes to `VERSION = 2`: `VISION_MODEL = "unsloth/Qwen3-VL-2B-Instruct-GGUF:Q4_K_M"` (the `ggml-org` repository publishes only Q8_0, the server refused `:Q4_K_M` there), new `VISION_THREADS = 8`, `VISION_TIMEOUT_S = 600`, fallbacks reduced to `["frames=2", "quant=Q3_K_M"]`.
- Measured on the Railway `vision` service, same clip `IMG_6859.MOV`, four frames at 448 px longest side: with default threads (27 vCPU of load against a 24 vCPU limit) the server fell to 6 tok/s prompt and about 0.5 tok/s generation and the third run missed 300 s; with `LLAMA_ARG_THREADS=8` and Q4_K_M the run took 4.0 s -- 525 prompt tokens at 216 tok/s, 105 answer tokens at 66 tok/s -- and the clip is described. The thread cap was the larger win; a CPU inference card must declare its thread count.
- Not checked: the card was not run through `t27c` on the authoring host. Not claimed: no host reads this card yet. Russian record of the measurement: gHashTag/999-multibots-telegraf PR #2391.
