# NOW -- The release pipeline published the wrong product, unsequenced (2026-08-28)

## The release pipeline published the wrong product, unsequenced (Refs #2161)

- nine runs, nine failures, and two of those failures each permanently burned a registry -- PyPI golden-float 0.1.0 and crates.io golden-float-ffi 0.1.0
- a t27c tag fired golden-float's npm/crates/Zenodo jobs while t27c itself was published nowhere
- product gate, version truth, dry run first, concurrency -- every publishing job now needs: preflight
- two blockers found by rehearsing: a broken symlink from #408 that cargo package dies on, and includes reaching outside the package
