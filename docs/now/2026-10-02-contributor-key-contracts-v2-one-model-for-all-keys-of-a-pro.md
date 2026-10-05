# NOW -- Contributor key contracts v2: one model for all keys of a provider (2026-10-02)

## Contributor key contracts v2: one model for all keys of a provider (Closes #5633)

- Queen contract v2 (specs/automation/queen-contributor-keys.t27): a model switch needs an owned key of the provider and a tool call, at most MODEL_CHECK_ATTEMPTS keys are tried, model names are bounded by MODEL_LIMIT; PROBE_TIMEOUT_MS 15 s -> 90 s because z-ai/glm-5.3 on NVIDIA answered in 20-46 s and glm-4.5-flash on Z.ai in up to 87 s on 2026-10-02.
- Render contract v2 (specs/automation/hive-contributor-keys.t27): only provider and model cross the model boundary, model names are bounded by MAX_MODEL_CHARS, PROXY_TIMEOUT_MS 45 s -> 200 s so the proxy outlasts the Queen's 2 x 90 s check.
- t27c test-report with Zig 0.16.0: 13/13 and 11/11 pass; negative controls 13/13 and 13/13 caught (conformance/automation/*.controls.json); both seals saved with this tree's t27c and verify MATCH; gen-ts output is byte-identical to the hosts gHashTag/BrowserOS#524 and gHashTag/999-multibots-telegraf#3429.
