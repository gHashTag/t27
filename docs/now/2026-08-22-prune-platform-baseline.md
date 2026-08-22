# NOW — a CI-passing test sat in the failing baseline

Last updated: 2026-08-22

## Prune four macOS-only failures from the ratchet baseline (Closes #2387)

- Branch: `fix/2387-prune-platform-baseline`
- Issue: #2387

### Что легло

`scripts/ci/test-baseline.txt`, 382 → 378 entries. The four `spec_first_*` tests in
`bitnet_layer`, `bitnet_mlp`, `bitnet_mlp3` and `bitnet_neuron_nchunk` fail on macOS/arm64,
where the baseline was generated, and **pass on the Linux runner** — the first
`test-ratchet` run on master reported all four under "now PASS".

A CI-passing test left in the baseline is invisible if it breaks: the ratchet fails only on
names new relative to the baseline, so these four could go red on Linux and the gate would
stay green. That is the blind spot the ratchet exists to remove, reintroduced in its own
input. **The baseline must describe the platform that gates.**

### Границы честности (BINDING)

- **The cause of the platform divergence was not investigated.** Endianness, float
  formatting, path separators and filesystem ordering are all plausible; none is confirmed.
  That they are all `spec_first_*` inference-vs-reference comparisons hints at a numeric or
  ordering difference — a hint, not a finding.
- Pruning means a **macOS** developer now sees four local failures the ratchet does not know
  about. Correct trade — CI is what gates — but a trade, noted in the file itself.
- **This fixes nothing.** The four still fail on macOS.
- The baseline is still generated on macOS. Regenerating it from a runner log would be
  better and is not done here.
