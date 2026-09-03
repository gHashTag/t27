# NOW -- Two states and a stated limitation (2026-09-04)

## `gates empty` says whether a passing gate could reach a tree at all

- `tri gates empty` reported 5 invocations that passed over an empty tree. Going
  through them by hand found **0 defects**: three never touch a tree, and the
  two that do print their scope or their population.
- The discriminator is "can this reach a tree", and the output did not say. That
  column is now in the command, decided from the script's source, with the
  marker list printed so it can be argued with.
- Two states, plus *source not read* -- never `false`, because a file nobody
  opened cannot be reported as one that touches nothing.
- A third state was tried and REMOVED, and that is the finding. "Reads one and
  builds one" was meant for `pack_index_consistency_gate.py --selftest`, whose
  `os.listdir` is aimed at its own `mkdtemp`. It also swallowed
  `check_conflict_markers.py`, which reads 7741 tracked files and merely uses a
  `TemporaryDirectory` in its self-check. The bucket held two members, neither
  belonged, and the count of the category worth reading went to zero while the
  output looked richer.
- The limitation is stated instead: the column is about the SCRIPT, and an
  invocation's flags can narrow it. Its removal has a test, so the absence is a
  decision.
- The mutation round ran under `cargo test ... reach` first, which matched 16
  tests in `leanreach` and `modreach` and none of mine. The filter is a
  substring; expecting 3 and reading 16 is what caught it.
