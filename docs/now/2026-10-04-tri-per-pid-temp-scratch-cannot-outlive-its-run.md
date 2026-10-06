# NOW -- tri: per-pid temp scratch cannot outlive its run (2026-10-04)

## tri: per-pid temp scratch cannot outlive its run (Closes #5982)

- The m2l standalone lake test built a ~7.6 GB package in temp_dir()/tri_m2l_standalone_pkg_<pid> and removed it only after assert!(status.success()); a failed build or a killed run left it behind, and on 2026-10-04 that filled the owner's disk twice.
- New cli/tri/src/piddir.rs: PidPath removes its path on drop (return and unwind), sweep_dead removes <prefix><pid>[.ext] entries whose pid is not running (ps -p; any unclear answer counts as running).
- Applied to the m2l lake test (build command is now a parameter), the consumable-lean test (its pkg dir was never removed), and census Scratch (a killed run's tree copy is swept by the next).
- Proof: m2l_scratch_is_gone_after_a_failed_build drives the build with false; the success-only-cleanup mutant turns it red. Shared lake cache deferred: no measurement possible under the disk limit.
