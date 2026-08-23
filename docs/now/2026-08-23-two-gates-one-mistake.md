# NOW — two gates, one mistake, made twice (2026-08-23)

`verify_multitarget.py` scored **0 killed of 7**, and its survivor lines are identical in shape to `verify_emit_bitexact.py`'s: the gate's own `sys.exit(0 if ok else 1)` under both return operators, and all three comparison FAIL branches under inversion.

- **Both controls cover the skip pair. Both leave through `skip()` and never reach `main()`.** Two gates, one mistake, and I made it twice — the second time three iterations after writing down the first.
- That is what a *class* looks like when you have not internalised it: the rule was recorded, the next control was written the same way, and only a measurement caught it.
- **Both plants move one arm.** `py_ref` reads the Python model while C and Rust come from `t27c`; perturbing it makes the model disagree with backends that are unchanged. Perturbing the spec or the emitter would move every arm together and plant nothing.

```
verify_emit_bitexact.py   0/17  ->  13/17
verify_multitarget.py      0/7  ->   5/7
```

Only boundary survivors remain in both.

## The honesty mechanism needed its own correction

Re-measuring produced a row with **two columns measured and three reused**, labelled `[cached]` wholesale — under-claiming rather than over-claiming, so the safe direction, and still wrong: the point of the marker is that a reader can tell.

Three states now: no marker when fresh, `[cached]` when every column is reused, `[3 cached, 2 fresh]` when mixed. The mixed case had to be reached by marking cache entries stale by hand — **a state that cannot occur naturally during testing is a state nobody has seen your code produce.**
