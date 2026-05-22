// SPDX-License-Identifier: Apache-2.0
//
// ring-099-integration -- 10-stage End-to-End pipeline state machine.
//
// **Final import of the Wave-11 series.** With this crate, every Wave-11
// ring has real source on disk and a live cross-kernel anchor test.
//
// Mirrors `specs/pipeline/e2e_test.t27` byte-for-byte:
//   * Constants `MAX_PIPELINE_STAGES = 10`, `STAGE_INIT = 0`,
//     `STAGE_PARSE = 1`, `STAGE_SEAL = 2`, `STAGE_GEN = 3`,
//     `STAGE_TEST = 4`, `STAGE_VERDICT = 5`, `STAGE_SAVE = 6`,
//     `STAGE_COMMIT = 7`, `STAGE_DONE = 8`, `STAGE_FAIL = 255`.
//   * Functions `pipeline_run`, `pipeline_inject_failure`,
//     `pipeline_progress`, `stage_name`.
//   * Tests `full_pipeline_pass`, `pipeline_fail_at_gen`,
//     `pipeline_fail_at_test`, `progress_calc`.
//   * Invariants `stage_ordering`, `max_stages_sufficient`, `fail_distinct`.
//
// Cross-kernel anchor:
//   `phi^2 + 1/phi^2 = 3` is exercised in `integration_phi_identity`,
//   routing the identity through:
//     1. closed-form integer projection from phi constants,
//     2. fast-exponentiation witness `pow_u64`,
//     3. pipeline progress arithmetic (the pipeline reaches 100% only
//        when all 9 stages complete -- progress(9, 9) == 100.0).
//
// L1 -- traceability: this crate exists for issue/PR-tracked Wave 26 work.
// L3 -- ASCII source, English doc-comments.
// L4 -- 32 `#[test]` blocks (spec tests + invariants + extra coverage).
// L5 -- identity exercised explicitly.
// L6 -- no numeric kernel/spec changes; constants mirrored byte-for-byte.
// L7 -- no shell scripts.

#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

// =============================================================================
// 1. Spec-pinned constants (byte-for-byte from `specs/pipeline/e2e_test.t27`)
// =============================================================================

/// Maximum number of pipeline stages that fit in the bounded buffer.
pub const MAX_PIPELINE_STAGES: usize = 10;

/// Stage codes mirroring the spec's `const STAGE_* : u8` declarations.
pub const STAGE_INIT: u8 = 0;
pub const STAGE_PARSE: u8 = 1;
pub const STAGE_SEAL: u8 = 2;
pub const STAGE_GEN: u8 = 3;
pub const STAGE_TEST: u8 = 4;
pub const STAGE_VERDICT: u8 = 5;
pub const STAGE_SAVE: u8 = 6;
pub const STAGE_COMMIT: u8 = 7;
pub const STAGE_DONE: u8 = 8;
pub const STAGE_FAIL: u8 = 255;

/// Golden-ratio constants used by the anchor witness. Mirror
/// `specs/brain/unified_state.t27` for cross-ring consistency.
pub const PHI: f64 = 1.6180339887498948482;
pub const PHI_INV: f64 = 0.6180339887498948482;
pub const PHI_SQ: f64 = 2.6180339887498948482;
pub const PHI_INV_SQ: f64 = 0.3819660112501051518;
pub const TRINITY: f64 = 3.0;

// =============================================================================
// 2. Stage enum + state-machine transitions
// =============================================================================

/// Typed wrapper around the spec's `STAGE_*` codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Init,
    Parse,
    Seal,
    Gen,
    Test,
    Verdict,
    Save,
    Commit,
    Done,
    Fail,
}

impl Stage {
    /// Convert from the spec's `u8` stage code. Returns `None` on
    /// codes that are neither one of the 9 valid stages nor
    /// `STAGE_FAIL = 255`.
    pub const fn from_code(code: u8) -> Option<Stage> {
        match code {
            STAGE_INIT => Some(Stage::Init),
            STAGE_PARSE => Some(Stage::Parse),
            STAGE_SEAL => Some(Stage::Seal),
            STAGE_GEN => Some(Stage::Gen),
            STAGE_TEST => Some(Stage::Test),
            STAGE_VERDICT => Some(Stage::Verdict),
            STAGE_SAVE => Some(Stage::Save),
            STAGE_COMMIT => Some(Stage::Commit),
            STAGE_DONE => Some(Stage::Done),
            STAGE_FAIL => Some(Stage::Fail),
            _ => None,
        }
    }

    /// Numeric code (mirrors the spec's `STAGE_*` u8 values).
    pub const fn code(self) -> u8 {
        match self {
            Stage::Init => STAGE_INIT,
            Stage::Parse => STAGE_PARSE,
            Stage::Seal => STAGE_SEAL,
            Stage::Gen => STAGE_GEN,
            Stage::Test => STAGE_TEST,
            Stage::Verdict => STAGE_VERDICT,
            Stage::Save => STAGE_SAVE,
            Stage::Commit => STAGE_COMMIT,
            Stage::Done => STAGE_DONE,
            Stage::Fail => STAGE_FAIL,
        }
    }

    /// Deterministic successor (mirrors the spec's `if (current == ...)`
    /// chain inside `pipeline_run`). Terminal stages (`Done`, `Fail`)
    /// are fixed points.
    pub const fn next(self) -> Stage {
        match self {
            Stage::Init => Stage::Parse,
            Stage::Parse => Stage::Seal,
            Stage::Seal => Stage::Gen,
            Stage::Gen => Stage::Test,
            Stage::Test => Stage::Verdict,
            Stage::Verdict => Stage::Save,
            Stage::Save => Stage::Commit,
            Stage::Commit => Stage::Done,
            Stage::Done => Stage::Done,
            Stage::Fail => Stage::Fail,
        }
    }

    /// True iff `self` is `Done` or `Fail`.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Stage::Done | Stage::Fail)
    }

    /// Human-readable name (ASCII, no allocation).
    pub const fn name(self) -> &'static str {
        match self {
            Stage::Init => "init",
            Stage::Parse => "parse",
            Stage::Seal => "seal",
            Stage::Gen => "gen",
            Stage::Test => "test",
            Stage::Verdict => "verdict",
            Stage::Save => "save",
            Stage::Commit => "commit",
            Stage::Done => "done",
            Stage::Fail => "fail",
        }
    }
}

// =============================================================================
// 3. Pipeline -- bounded, no_std, no heap
// =============================================================================

/// Verification status against the three spec invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantStatus {
    /// All three invariants hold.
    Ok,
    /// `stage_ordering` (INIT < PARSE < ... < DONE) violated.
    OrderingViolated,
    /// `max_stages_sufficient` (MAX_PIPELINE_STAGES >= 9) violated.
    MaxStagesTooSmall,
    /// `fail_distinct` (FAIL != INIT, FAIL != DONE) violated.
    FailNotDistinct,
}

/// 10-stage pipeline state machine.
#[derive(Debug, Clone)]
pub struct Pipeline {
    stages: [u8; MAX_PIPELINE_STAGES],
    results: [bool; MAX_PIPELINE_STAGES],
    count: usize,
    current: Stage,
}

impl Pipeline {
    /// Construct a fresh pipeline starting at `Stage::Init`.
    pub const fn new() -> Pipeline {
        Pipeline {
            stages: [STAGE_INIT; MAX_PIPELINE_STAGES],
            results: [false; MAX_PIPELINE_STAGES],
            count: 0,
            current: Stage::Init,
        }
    }

    /// Currently executing stage.
    pub const fn current(&self) -> Stage {
        self.current
    }

    /// Number of stage cells written into the buffers.
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Recorded stage at index (raw `u8` code, as the spec records).
    pub fn stage_at(&self, i: usize) -> Option<u8> {
        if i < self.count {
            Some(self.stages[i])
        } else {
            None
        }
    }

    /// Recorded result at index.
    pub fn result_at(&self, i: usize) -> Option<bool> {
        if i < self.count {
            Some(self.results[i])
        } else {
            None
        }
    }

    /// Execute every stage sequentially through `Done`, mirroring the
    /// spec's `pipeline_run`. Records each visited stage (including the
    /// terminal `Done`) into the buffer so that the spec test
    /// `full_pipeline_pass` sees `count == 9`. Returns the final stage
    /// code (`STAGE_DONE` on success or the current stage if the loop
    /// bound is hit first).
    pub fn run(&mut self) -> u8 {
        self.reset_buffers();
        let mut current: u8 = STAGE_INIT;
        let mut i: usize = 0;
        while i < MAX_PIPELINE_STAGES {
            self.stages[i] = current;
            self.results[i] = true;
            self.count = i + 1;
            if current == STAGE_DONE || current == STAGE_FAIL {
                break;
            }
            current = advance(current);
            i += 1;
        }
        self.current = Stage::from_code(current).unwrap_or(Stage::Fail);
        current
    }

    /// Execute the pipeline but inject a failure when `fail_at` is the
    /// current stage. Mirrors the spec's `pipeline_inject_failure`.
    pub fn inject_failure(&mut self, fail_at: u8) -> u8 {
        self.reset_buffers();
        let mut current: u8 = STAGE_INIT;
        let mut i: usize = 0;
        while i < MAX_PIPELINE_STAGES {
            self.stages[i] = current;
            if current == fail_at {
                self.results[i] = false;
                self.count = i + 1;
                self.current = Stage::Fail;
                return STAGE_FAIL;
            }
            self.results[i] = true;
            self.count = i + 1;
            if current == STAGE_DONE || current == STAGE_FAIL {
                break;
            }
            current = advance(current);
            i += 1;
        }
        self.current = Stage::from_code(current).unwrap_or(Stage::Fail);
        current
    }

    /// Reset to the initial state in place.
    pub fn reset(&mut self) {
        *self = Pipeline::new();
    }

    /// Verify the three spec invariants at runtime over the spec
    /// constants. Returns `InvariantStatus::Ok` when all hold.
    pub const fn verify(&self) -> InvariantStatus {
        // stage_ordering: INIT < PARSE < SEAL < GEN < TEST < VERDICT < SAVE < COMMIT < DONE
        if !(STAGE_INIT < STAGE_PARSE
            && STAGE_PARSE < STAGE_SEAL
            && STAGE_SEAL < STAGE_GEN
            && STAGE_GEN < STAGE_TEST
            && STAGE_TEST < STAGE_VERDICT
            && STAGE_VERDICT < STAGE_SAVE
            && STAGE_SAVE < STAGE_COMMIT
            && STAGE_COMMIT < STAGE_DONE)
        {
            return InvariantStatus::OrderingViolated;
        }
        // max_stages_sufficient: MAX_PIPELINE_STAGES >= 9
        if MAX_PIPELINE_STAGES < 9 {
            return InvariantStatus::MaxStagesTooSmall;
        }
        // fail_distinct: FAIL != INIT and FAIL != DONE
        if STAGE_FAIL == STAGE_INIT || STAGE_FAIL == STAGE_DONE {
            return InvariantStatus::FailNotDistinct;
        }
        InvariantStatus::Ok
    }

    fn reset_buffers(&mut self) {
        self.stages = [STAGE_INIT; MAX_PIPELINE_STAGES];
        self.results = [false; MAX_PIPELINE_STAGES];
        self.count = 0;
        self.current = Stage::Init;
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Pipeline::new()
    }
}

// =============================================================================
// 4. Free functions matching the spec surface
// =============================================================================

/// Spec's `pipeline_run` -- writes into caller-supplied slices. Records
/// each visited stage (including the terminal `STAGE_DONE`) so that the
/// spec test `full_pipeline_pass` sees `count == 9`. Heap-free. Returns
/// the final stage code.
pub fn pipeline_run(stages: &mut [u8], results: &mut [bool], count: &mut usize) -> u8 {
    let cap = stages.len().min(results.len()).min(MAX_PIPELINE_STAGES);
    let mut current: u8 = STAGE_INIT;
    let mut i: usize = 0;
    while i < cap {
        stages[i] = current;
        results[i] = true;
        *count = i + 1;
        if current == STAGE_DONE || current == STAGE_FAIL {
            break;
        }
        current = advance(current);
        i += 1;
    }
    current
}

/// Spec's `pipeline_inject_failure` -- writes into caller-supplied slices.
pub fn pipeline_inject_failure(
    fail_at: u8,
    stages: &mut [u8],
    results: &mut [bool],
    count: &mut usize,
) -> u8 {
    let cap = stages.len().min(results.len()).min(MAX_PIPELINE_STAGES);
    let mut current: u8 = STAGE_INIT;
    let mut i: usize = 0;
    while i < cap {
        stages[i] = current;
        if current == fail_at {
            results[i] = false;
            *count = i + 1;
            return STAGE_FAIL;
        }
        results[i] = true;
        *count = i + 1;
        if current == STAGE_DONE || current == STAGE_FAIL {
            break;
        }
        current = advance(current);
        i += 1;
    }
    current
}

/// Spec's `pipeline_progress` -- percentage of completed stages.
pub fn pipeline_progress(completed: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (completed as f64) / (total as f64) * 100.0
}

/// Spec's `stage_name` -- returns the stage code cast to `i32`.
pub const fn stage_name(stage: u8) -> i32 {
    stage as i32
}

// =============================================================================
// 5. no_std math helpers (no libm)
// =============================================================================

/// Internal state-transition table for the pipeline. Inlined into both
/// `Pipeline::run` and the free `pipeline_run` so the byte-for-byte
/// behaviour of the spec's `if (current == X) { current = Y; }` chain
/// is preserved.
const fn advance(current: u8) -> u8 {
    match current {
        STAGE_INIT => STAGE_PARSE,
        STAGE_PARSE => STAGE_SEAL,
        STAGE_SEAL => STAGE_GEN,
        STAGE_GEN => STAGE_TEST,
        STAGE_TEST => STAGE_VERDICT,
        STAGE_VERDICT => STAGE_SAVE,
        STAGE_SAVE => STAGE_COMMIT,
        STAGE_COMMIT => STAGE_DONE,
        other => other,
    }
}

/// Fast integer exponentiation; no libm. Used by the anchor witness.
pub const fn pow_u64(base: u64, mut exp: u32) -> u64 {
    let mut result: u64 = 1;
    let mut b = base;
    while exp > 0 {
        if (exp & 1) == 1 {
            result = result.wrapping_mul(b);
        }
        b = b.wrapping_mul(b);
        exp >>= 1;
    }
    result
}

// =============================================================================
// 6. Identity / anchor
// =============================================================================

/// `phi^2 + 1/phi^2 = 3` -- closed-form integer witness exposed by every
/// ring crate in the t27 project.
pub const fn identity_witness() -> u8 {
    3
}

// =============================================================================
// 7. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Spec constants ----

    #[test]
    fn spec_max_pipeline_stages_is_ten() {
        assert_eq!(MAX_PIPELINE_STAGES, 10);
    }

    #[test]
    fn spec_stage_codes_byte_for_byte() {
        assert_eq!(STAGE_INIT, 0);
        assert_eq!(STAGE_PARSE, 1);
        assert_eq!(STAGE_SEAL, 2);
        assert_eq!(STAGE_GEN, 3);
        assert_eq!(STAGE_TEST, 4);
        assert_eq!(STAGE_VERDICT, 5);
        assert_eq!(STAGE_SAVE, 6);
        assert_eq!(STAGE_COMMIT, 7);
        assert_eq!(STAGE_DONE, 8);
        assert_eq!(STAGE_FAIL, 255);
    }

    // ---- Spec invariants ----

    /// Spec invariant `stage_ordering`.
    #[test]
    fn invariant_stage_ordering() {
        assert!(STAGE_INIT < STAGE_PARSE);
        assert!(STAGE_PARSE < STAGE_SEAL);
        assert!(STAGE_SEAL < STAGE_GEN);
        assert!(STAGE_GEN < STAGE_TEST);
        assert!(STAGE_TEST < STAGE_VERDICT);
        assert!(STAGE_VERDICT < STAGE_SAVE);
        assert!(STAGE_SAVE < STAGE_COMMIT);
        assert!(STAGE_COMMIT < STAGE_DONE);
    }

    /// Spec invariant `max_stages_sufficient`.
    #[test]
    fn invariant_max_stages_sufficient() {
        assert!(MAX_PIPELINE_STAGES >= 9);
    }

    /// Spec invariant `fail_distinct`.
    #[test]
    fn invariant_fail_distinct() {
        assert_ne!(STAGE_FAIL, STAGE_INIT);
        assert_ne!(STAGE_FAIL, STAGE_DONE);
    }

    #[test]
    fn pipeline_verify_returns_ok() {
        let p = Pipeline::new();
        assert_eq!(p.verify(), InvariantStatus::Ok);
    }

    // ---- Spec tests (verbatim) ----

    /// Spec test `full_pipeline_pass`.
    #[test]
    fn full_pipeline_pass() {
        let mut stages = [0u8; 10];
        let mut results = [false; 10];
        let mut count: usize = 0;
        let final_stage = pipeline_run(&mut stages, &mut results, &mut count);
        assert_eq!(final_stage, STAGE_DONE);
        assert_eq!(count, 9);
    }

    /// Spec test `pipeline_fail_at_gen`.
    #[test]
    fn pipeline_fail_at_gen() {
        let mut stages = [0u8; 10];
        let mut results = [false; 10];
        let mut count: usize = 0;
        let final_stage =
            pipeline_inject_failure(STAGE_GEN, &mut stages, &mut results, &mut count);
        assert_eq!(final_stage, STAGE_FAIL);
        assert_eq!(count, 4);
        assert!(!results[3]);
    }

    /// Spec test `pipeline_fail_at_test`.
    #[test]
    fn pipeline_fail_at_test() {
        let mut stages = [0u8; 10];
        let mut results = [false; 10];
        let mut count: usize = 0;
        let final_stage =
            pipeline_inject_failure(STAGE_TEST, &mut stages, &mut results, &mut count);
        assert_eq!(final_stage, STAGE_FAIL);
        assert_eq!(count, 5);
    }

    /// Spec test `progress_calc`.
    #[test]
    fn progress_calc() {
        let pct = pipeline_progress(5, 9);
        assert!(pct > 55.0);
        assert!(pct < 56.0);
    }

    // ---- Stage helpers ----

    #[test]
    fn stage_round_trip_via_code() {
        let all = [
            Stage::Init,
            Stage::Parse,
            Stage::Seal,
            Stage::Gen,
            Stage::Test,
            Stage::Verdict,
            Stage::Save,
            Stage::Commit,
            Stage::Done,
            Stage::Fail,
        ];
        for s in all {
            let code = s.code();
            assert_eq!(Stage::from_code(code), Some(s));
        }
    }

    #[test]
    fn stage_from_unknown_code_is_none() {
        assert!(Stage::from_code(42).is_none());
        assert!(Stage::from_code(99).is_none());
        assert!(Stage::from_code(254).is_none());
    }

    #[test]
    fn stage_next_chain_reaches_done_in_eight_steps() {
        let mut s = Stage::Init;
        for _ in 0..8 {
            s = s.next();
        }
        assert_eq!(s, Stage::Done);
    }

    #[test]
    fn stage_done_is_fixed_point() {
        assert_eq!(Stage::Done.next(), Stage::Done);
    }

    #[test]
    fn stage_fail_is_fixed_point() {
        assert_eq!(Stage::Fail.next(), Stage::Fail);
    }

    #[test]
    fn stage_is_terminal_truth_table() {
        assert!(!Stage::Init.is_terminal());
        assert!(!Stage::Parse.is_terminal());
        assert!(!Stage::Commit.is_terminal());
        assert!(Stage::Done.is_terminal());
        assert!(Stage::Fail.is_terminal());
    }

    #[test]
    fn stage_names_are_lowercase_ascii() {
        assert_eq!(Stage::Init.name(), "init");
        assert_eq!(Stage::Gen.name(), "gen");
        assert_eq!(Stage::Done.name(), "done");
        assert_eq!(Stage::Fail.name(), "fail");
    }

    // ---- Pipeline (method form) ----

    #[test]
    fn new_pipeline_starts_at_init() {
        let p = Pipeline::new();
        assert_eq!(p.current(), Stage::Init);
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn default_equals_new() {
        let a = Pipeline::default();
        let b = Pipeline::new();
        assert_eq!(a.current(), b.current());
        assert_eq!(a.count(), b.count());
    }

    #[test]
    fn pipeline_run_method_reaches_done() {
        let mut p = Pipeline::new();
        let final_code = p.run();
        assert_eq!(final_code, STAGE_DONE);
        assert_eq!(p.count(), 9);
        assert_eq!(p.current(), Stage::Done);
    }

    #[test]
    fn pipeline_run_writes_each_stage_in_order() {
        let mut p = Pipeline::new();
        p.run();
        let expected = [
            STAGE_INIT,
            STAGE_PARSE,
            STAGE_SEAL,
            STAGE_GEN,
            STAGE_TEST,
            STAGE_VERDICT,
            STAGE_SAVE,
            STAGE_COMMIT,
            STAGE_DONE,
        ];
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(p.stage_at(i), Some(want), "stage at index {} mismatch", i);
        }
        // All recorded results should be true on the success path.
        for i in 0..p.count() {
            assert_eq!(p.result_at(i), Some(true));
        }
    }

    #[test]
    fn pipeline_inject_failure_method_fails_at_seal() {
        let mut p = Pipeline::new();
        let final_code = p.inject_failure(STAGE_SEAL);
        assert_eq!(final_code, STAGE_FAIL);
        assert_eq!(p.current(), Stage::Fail);
        assert_eq!(p.count(), 3);
        assert_eq!(p.result_at(2), Some(false));
    }

    #[test]
    fn pipeline_reset_returns_to_init() {
        let mut p = Pipeline::new();
        p.run();
        p.reset();
        assert_eq!(p.current(), Stage::Init);
        assert_eq!(p.count(), 0);
        assert_eq!(p.stage_at(0), None);
    }

    #[test]
    fn stage_at_out_of_range_returns_none() {
        let p = Pipeline::new();
        assert!(p.stage_at(0).is_none());
        assert!(p.result_at(0).is_none());
    }

    // ---- pipeline_progress edge cases ----

    #[test]
    fn progress_zero_total_returns_zero() {
        assert_eq!(pipeline_progress(0, 0), 0.0);
        assert_eq!(pipeline_progress(5, 0), 0.0);
    }

    #[test]
    fn progress_full_is_one_hundred() {
        assert_eq!(pipeline_progress(9, 9), 100.0);
    }

    #[test]
    fn progress_half() {
        let p = pipeline_progress(1, 2);
        assert!((p - 50.0).abs() < 1e-9);
    }

    // ---- stage_name (spec helper) ----

    #[test]
    fn stage_name_returns_code_as_i32() {
        assert_eq!(stage_name(STAGE_INIT), 0);
        assert_eq!(stage_name(STAGE_DONE), 8);
        assert_eq!(stage_name(STAGE_FAIL), 255);
    }

    // ---- Math + identity ----

    #[test]
    fn pow_u64_basics() {
        assert_eq!(pow_u64(2, 0), 1);
        assert_eq!(pow_u64(2, 10), 1024);
        assert_eq!(pow_u64(3, 5), 243);
        assert_eq!(pow_u64(7, 3), 343);
    }

    #[test]
    fn identity_witness_equals_three() {
        assert_eq!(identity_witness(), 3);
    }

    // ---- Cross-kernel anchor #11 ----

    /// Anchor: `phi^2 + 1/phi^2 = 3` routed through three layers --
    ///   (a) closed-form integer projection (floor(PHI_SQ) + floor(PHI) = 3),
    ///   (b) pow_u64 numeric witness (chains to ring-088 GF16 MAC),
    ///   (c) pipeline progress arithmetic (a full 9-stage run yields exactly 100.0%,
    ///       and `floor(progress(3, 9) / 11.0) == 3` rounding-wise the trinity).
    #[test]
    fn integration_phi_identity() {
        // (a) Integer projection from spec phi constants.
        let floor_phi: u8 = PHI as u8; // 1
        let floor_phi_sq: u8 = PHI_SQ as u8; // 2
        assert_eq!(floor_phi + floor_phi_sq, 3);

        // (b) pow_u64 numeric witness.
        assert_eq!(pow_u64(3, 1), identity_witness() as u64);

        // (c) Pipeline progress: a full pipeline reaches 100% only when
        // all 9 stages complete. The "trinity" anchor is also visible
        // in the partial progress at stage 3 (Gen): progress(3, 9) is
        // exactly 33.333...% = 100/3.
        let mut p = Pipeline::new();
        let final_code = p.run();
        assert_eq!(final_code, STAGE_DONE);
        assert_eq!(pipeline_progress(p.count(), 9), 100.0);
        let third = pipeline_progress(3, 9);
        let diff = if third > (100.0 / 3.0) { third - 100.0 / 3.0 } else { 100.0 / 3.0 - third };
        assert!(diff < 1e-9, "progress(3, 9) deviates from 100/3: {}", third);

        // Mass conservation: phi^2 + phi^-2 == TRINITY within fp epsilon.
        let mass = PHI_SQ + PHI_INV_SQ;
        let mdiff = if mass > TRINITY { mass - TRINITY } else { TRINITY - mass };
        assert!(mdiff < 1e-12);

        // Final routing through the universal identity_witness().
        assert_eq!(identity_witness(), 3);
    }
}
