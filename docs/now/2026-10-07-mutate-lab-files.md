# NOW -- a `--lab` run uploads any number of files (2026-10-07)

## cli/tri/src/mutate.rs (Refs #7471)

- Slice 1 of 2 for `tri mutate census --dir D --lab`; the plan is on #7471. `LabRun.spec` (one spec's bytes) is now `files`: each file's path in the repo with its bytes. `lab_run_name` hashes every file, and `lab_launch` uploads `f<i>.b64` per file, then decodes each into the run's copy of the repo at its own path. `mutate spec --lab` still sends one file, so its behaviour does not change. Slice 2 sends a directory's specs.
- `git diff --numstat`: 30 lines added, 13 removed, within the #7371 budget of 40 per file.
- New test `every_file_of_a_lab_run_goes_up`: two files, in `specs/x/` and `specs/y/`, go up through the `T27C_LAB_LOCAL=1` fixture, and the fixture `tri` prints both in order. The fixture's `cat` now names `specs/y/b.t27` as well; the older tests send no such file, and its error goes to `/dev/null`.
- Results, on the Railway lab:
  - `cargo build --release -p tri` exits 0, with no warning from `mutate.rs`; `cargo test --release -p tri mutate::`: 57 passed, 0 failed.
  - `mutate spec --file specs/tri/mutate/survivors.t27 --lab --jobs 4 --timeout 60 --zig-threads 4`, through `T27C_LAB_LOCAL=1`, with the `tri` of #7472 and with this one. Both exit 0 and print `20 of 20 killed`. The outputs are the same except for the run's name, the launch pid and the tool's path and build time. The runs' directory is empty afterwards.
- Negative control: every file decoded from `f0.b64` (`S/f{i}.b64` -> `S/f0.b64`) fails `every_file_of_a_lab_run_goes_up` (56 passed, 1 failed). Every older test still passes under it, because each sends one file. That is why the new test exists.
