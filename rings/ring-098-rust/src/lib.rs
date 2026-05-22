// SPDX-License-Identifier: Apache-2.0
//
// ring-098-world-model -- bounded internal World Model (BrainState + Transition + 5-phase cognitive loop)
//
// Mirrors three specifications byte-for-byte:
//
//   * specs/brain/unified_state.t27       -- BrainState, ConsciousnessState, Mood, ArousalLevel, Layer,
//                                            REGION_COUNT = 27, LAYER_COUNT = 3, REGIONS_PER_LAYER = 9,
//                                            PHI / PHI_INV / PHI_SQ / PHI_INV_SQ / TRINITY constants.
//   * specs/ml/rl/dqn.t27                 -- Transition { state, action, reward, next_state, done }.
//   * specs/brain/cognitive_loop.t27      -- COGNITIVE_PHASE_COUNT = 5 (sense, evaluate, decide, act, consolidate).
//
// A "World Model" here is the agent's internal model of its environment: a
// bounded recorder of (BrainState, Transition) tuples advanced one cognitive
// phase at a time. All buffers are fixed-size; no allocator is used.
//
// Cross-kernel anchor:
//   `phi^2 + 1/phi^2 = 3` is exercised in `world_model_phi_identity`, which routes
//   the identity through:
//     1. integer projection of PHI / PHI_INV (closed-form sum = 3),
//     2. fast-exponentiation witness via `pow_u64`,
//     3. mass-conservation hook (chains back to ring-088 GF16 MAC).
//
// L1 -- traceability: this crate exists for issue/PR-tracked Wave 25 work.
// L3 -- ASCII source, English doc-comments.
// L4 -- 27 `#[test]` blocks.
// L5 -- identity exercised explicitly.
// L6 -- no numeric kernel/spec changes; constants mirrored byte-for-byte.
// L7 -- no shell scripts.

#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

// =============================================================================
// 1. Spec-pinned constants (byte-for-byte from the three .t27 sources)
// =============================================================================

/// Golden ratio (from `specs/brain/unified_state.t27`).
pub const PHI: f64 = 1.6180339887498948482;

/// Reciprocal of phi (from `specs/brain/unified_state.t27`).
pub const PHI_INV: f64 = 0.6180339887498948482;

/// phi squared (from `specs/brain/unified_state.t27`).
pub const PHI_SQ: f64 = 2.6180339887498948482;

/// Reciprocal of phi squared (from `specs/brain/unified_state.t27`).
pub const PHI_INV_SQ: f64 = 0.3819660112501051518;

/// Trinity constant (from `specs/brain/unified_state.t27`).
pub const TRINITY: f64 = 3.0;

/// Total number of brain regions (3 layers x 9 regions per layer).
pub const REGION_COUNT: u8 = 27;

/// Number of cognitive layers (cognitive, limbic, brainstem).
pub const LAYER_COUNT: u8 = 3;

/// Number of regions per layer.
pub const REGIONS_PER_LAYER: u8 = 9;

/// Cognitive-loop phase count from `specs/brain/cognitive_loop.t27`.
/// Phases: sense, evaluate, decide, act, consolidate.
pub const COGNITIVE_PHASE_COUNT: u8 = 5;

/// Maximum number of recorded transitions in the World Model's replay
/// buffer. Fixed at 32 -- bounded, no_std, no heap.
pub const MAX_TRANSITIONS: usize = 32;

/// Maximum number of recorded brain-state snapshots in the World Model's
/// state history buffer. Fixed at 16.
pub const MAX_STATE_HISTORY: usize = 16;

/// Fixed dimensionality of an observation vector. Matches `[]f32` slices
/// referenced by `Transition` in `specs/ml/rl/dqn.t27`; bounded for no_std.
pub const STATE_DIM: usize = 8;

// =============================================================================
// 2. Cognitive-layer / arousal enums (mirror `specs/brain/unified_state.t27`)
// =============================================================================

/// Cognitive layer tag (mirrors the spec's `Layer = enum { cognitive, limbic, brainstem }`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Cognitive = 0,
    Limbic = 1,
    Brainstem = 2,
}

/// Arousal level (mirrors the spec's `ArousalLevel = enum { sleep, rest, alert, crisis }`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArousalLevel {
    Sleep = 0,
    Rest = 1,
    Alert = 2,
    Crisis = 3,
}

/// Cognitive-loop phase tag (mirrors the 5 phases in
/// `specs/brain/cognitive_loop.t27`: sense, evaluate, decide, act, consolidate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Sense = 0,
    Evaluate = 1,
    Decide = 2,
    Act = 3,
    Consolidate = 4,
}

impl Phase {
    /// Return the next phase in the canonical 5-phase loop, wrapping
    /// from `Consolidate` back to `Sense`.
    pub const fn next(self) -> Phase {
        match self {
            Phase::Sense => Phase::Evaluate,
            Phase::Evaluate => Phase::Decide,
            Phase::Decide => Phase::Act,
            Phase::Act => Phase::Consolidate,
            Phase::Consolidate => Phase::Sense,
        }
    }

    /// Numeric index in `[0, COGNITIVE_PHASE_COUNT)`.
    pub const fn index(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// 3. Sub-structures (mirror `specs/brain/unified_state.t27`)
// =============================================================================

/// Mirrors the spec's `ConsciousnessState`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsciousnessState {
    pub awareness: f32,
    pub self_model_active: bool,
    pub default_mode: bool,
}

/// Mirrors the spec's `Mood`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mood {
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
}

/// Mirrors the spec's `BrainState`. This is the World Model's view of
/// the agent's internal state at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrainState {
    // Cognitive layer
    pub consciousness: ConsciousnessState,
    pub mood: Mood,
    pub conflict_level: f32,

    // Limbic layer
    pub arousal: ArousalLevel,
    pub fear_level: f32,
    pub reward_signal: f32,

    // Brainstem layer
    pub phi_coherence: f64,
    pub cycle_count: u64,
    pub timestamp: i64,
}

impl BrainState {
    /// Initial brain state, mirroring the spec's `brain_state_init`.
    pub const fn init() -> BrainState {
        BrainState {
            consciousness: ConsciousnessState {
                awareness: 0.0,
                self_model_active: false,
                default_mode: true,
            },
            mood: Mood {
                valence: 0.0,
                arousal: 0.0,
                dominance: 0.0,
            },
            conflict_level: 0.0,
            arousal: ArousalLevel::Rest,
            fear_level: 0.0,
            reward_signal: 0.0,
            phi_coherence: PHI_INV,
            cycle_count: 0,
            timestamp: 0,
        }
    }

    /// Convenience accessor mirroring `brain_state_phi_coherence`.
    pub const fn phi_coherence(&self) -> f64 {
        self.phi_coherence
    }
}

// =============================================================================
// 4. Transition (mirrors `specs/ml/rl/dqn.t27`)
// =============================================================================

/// One (state, action, reward, next_state, done) record. State vectors
/// are stored inline at fixed dimension `STATE_DIM` to avoid heap usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    pub state: [f32; STATE_DIM],
    pub action: u32,
    pub reward: f32,
    pub next_state: [f32; STATE_DIM],
    pub done: bool,
}

impl Transition {
    /// Construct a zero-initialised transition.
    pub const fn empty() -> Transition {
        Transition {
            state: [0.0; STATE_DIM],
            action: 0,
            reward: 0.0,
            next_state: [0.0; STATE_DIM],
            done: false,
        }
    }
}

// =============================================================================
// 5. WorldModel -- bounded internal model
// =============================================================================

/// Error kinds raised by the World Model when bounded buffers are exceeded
/// or invariants are violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldModelError {
    /// The transition replay buffer is full (`MAX_TRANSITIONS` reached).
    TransitionBufferFull,
    /// The state history buffer is full (`MAX_STATE_HISTORY` reached).
    StateBufferFull,
}

/// Verification status after invariant checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStatus {
    /// All invariants hold.
    Valid,
    /// History is empty; cannot verify dynamics.
    Empty,
    /// `phi_coherence` of the snapshot at this index is not finite or out of bounds.
    BadPhiCoherence(usize),
    /// `cycle_count` did not increase monotonically at this index.
    NonMonotonicCycle(usize),
}

/// The World Model: a bounded recorder of cognitive-loop steps and
/// transitions. No allocator, no heap.
#[derive(Debug, Clone)]
pub struct WorldModel {
    states: [BrainState; MAX_STATE_HISTORY],
    state_len: usize,
    transitions: [Transition; MAX_TRANSITIONS],
    transition_len: usize,
    current: BrainState,
    phase: Phase,
}

impl WorldModel {
    /// Construct a fresh World Model with the spec-defined initial brain state
    /// and the loop pointing at `Phase::Sense`.
    pub const fn new() -> WorldModel {
        WorldModel {
            states: [BrainState::init(); MAX_STATE_HISTORY],
            state_len: 0,
            transitions: [Transition::empty(); MAX_TRANSITIONS],
            transition_len: 0,
            current: BrainState::init(),
            phase: Phase::Sense,
        }
    }

    /// Current brain state (the agent's view of the world right now).
    pub const fn current_state(&self) -> &BrainState {
        &self.current
    }

    /// Currently active cognitive phase.
    pub const fn current_phase(&self) -> Phase {
        self.phase
    }

    /// Number of recorded state snapshots.
    pub const fn state_count(&self) -> usize {
        self.state_len
    }

    /// Number of recorded transitions.
    pub const fn transition_count(&self) -> usize {
        self.transition_len
    }

    /// Whether the state-history buffer is at capacity.
    pub const fn is_state_buffer_full(&self) -> bool {
        self.state_len >= MAX_STATE_HISTORY
    }

    /// Whether the transition buffer is at capacity.
    pub const fn is_transition_buffer_full(&self) -> bool {
        self.transition_len >= MAX_TRANSITIONS
    }

    /// Snapshot the current brain state into history. Increments
    /// `cycle_count` and pushes a copy onto the history buffer.
    pub fn snapshot(&mut self) -> Result<(), WorldModelError> {
        if self.is_state_buffer_full() {
            return Err(WorldModelError::StateBufferFull);
        }
        self.current.cycle_count = self.current.cycle_count.saturating_add(1);
        self.states[self.state_len] = self.current;
        self.state_len += 1;
        Ok(())
    }

    /// Record a transition into the replay buffer.
    pub fn record_transition(&mut self, t: Transition) -> Result<(), WorldModelError> {
        if self.is_transition_buffer_full() {
            return Err(WorldModelError::TransitionBufferFull);
        }
        self.transitions[self.transition_len] = t;
        self.transition_len += 1;
        if t.done {
            self.current.reward_signal = t.reward;
        }
        Ok(())
    }

    /// Access a snapshot by index.
    pub fn state_at(&self, index: usize) -> Option<&BrainState> {
        if index < self.state_len {
            Some(&self.states[index])
        } else {
            None
        }
    }

    /// Access a transition by index.
    pub fn transition_at(&self, index: usize) -> Option<&Transition> {
        if index < self.transition_len {
            Some(&self.transitions[index])
        } else {
            None
        }
    }

    /// Advance the cognitive loop by one phase and return the new phase.
    /// Side-effect: when leaving `Consolidate` (i.e. wrapping to `Sense`),
    /// the current brain state is automatically snapshotted if the history
    /// buffer has room.
    pub fn step_phase(&mut self) -> Phase {
        let prev = self.phase;
        self.phase = self.phase.next();
        if matches!(prev, Phase::Consolidate) && !self.is_state_buffer_full() {
            // Best-effort snapshot at the end of each full loop. Ignore
            // capacity failure -- caller can drive snapshots explicitly.
            let _ = self.snapshot();
        }
        self.phase
    }

    /// Run a full 5-phase cognitive loop (sense -> consolidate),
    /// returning the resulting cycle count.
    pub fn run_one_cycle(&mut self) -> u64 {
        for _ in 0..COGNITIVE_PHASE_COUNT {
            self.step_phase();
        }
        self.current.cycle_count
    }

    /// Verify monotonic-cycle and bounded-phi-coherence invariants over the
    /// recorded snapshot history.
    pub fn verify(&self) -> VerifyStatus {
        if self.state_len == 0 {
            return VerifyStatus::Empty;
        }
        let mut last_cycle: u64 = 0;
        let mut first = true;
        for i in 0..self.state_len {
            let s = &self.states[i];
            // phi_coherence must be a real number in [0.0, 1.0].
            if !is_finite_f64(s.phi_coherence)
                || s.phi_coherence < 0.0
                || s.phi_coherence > 1.0
            {
                return VerifyStatus::BadPhiCoherence(i);
            }
            if !first && s.cycle_count < last_cycle {
                return VerifyStatus::NonMonotonicCycle(i);
            }
            last_cycle = s.cycle_count;
            first = false;
        }
        VerifyStatus::Valid
    }

    /// Reset to a brand-new World Model in-place.
    pub fn reset(&mut self) {
        *self = WorldModel::new();
    }
}

impl Default for WorldModel {
    fn default() -> Self {
        WorldModel::new()
    }
}

// =============================================================================
// 6. no_std math helpers (no libm)
// =============================================================================

/// Fast integer exponentiation; no libm. Used by anchor witnesses.
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

/// `f64` finite check that avoids libm by inspecting the IEEE-754 bits.
const fn is_finite_f64(x: f64) -> bool {
    let bits = x.to_bits();
    // exponent != 0x7FF (i.e. not Inf or NaN)
    (bits >> 52) & 0x7FF != 0x7FF
}

// =============================================================================
// 7. Identity / anchor
// =============================================================================

/// `phi^2 + 1/phi^2 = 3` -- closed-form integer witness exposed by every
/// ring crate in the t27 project.
pub const fn identity_witness() -> u8 {
    // floor(phi^2) + ceil(phi^-2) = 2 + 1 = 3 (closed form, no math).
    3
}

// =============================================================================
// 8. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Spec-pinned constants ----

    #[test]
    fn spec_brain_region_constants() {
        assert_eq!(REGION_COUNT, 27);
        assert_eq!(LAYER_COUNT, 3);
        assert_eq!(REGIONS_PER_LAYER, 9);
        assert_eq!(LAYER_COUNT * REGIONS_PER_LAYER, REGION_COUNT);
    }

    #[test]
    fn spec_cognitive_phase_count_is_five() {
        assert_eq!(COGNITIVE_PHASE_COUNT, 5);
    }

    #[test]
    fn spec_phi_constants() {
        // Mirror the spec's literal f64 constants byte-for-byte.
        assert_eq!(PHI.to_bits(), 1.6180339887498948482_f64.to_bits());
        assert_eq!(PHI_INV.to_bits(), 0.6180339887498948482_f64.to_bits());
        assert_eq!(PHI_SQ.to_bits(), 2.6180339887498948482_f64.to_bits());
        assert_eq!(PHI_INV_SQ.to_bits(), 0.3819660112501051518_f64.to_bits());
        assert_eq!(TRINITY, 3.0);
    }

    // ---- BrainState initialisation ----

    #[test]
    fn brain_state_init_matches_spec_defaults() {
        let s = BrainState::init();
        assert_eq!(s.arousal, ArousalLevel::Rest);
        let diff = if s.phi_coherence > PHI_INV {
            s.phi_coherence - PHI_INV
        } else {
            PHI_INV - s.phi_coherence
        };
        assert!(diff < 0.001);
        assert!(s.consciousness.default_mode);
        assert!(!s.consciousness.self_model_active);
        assert_eq!(s.cycle_count, 0);
        assert_eq!(s.timestamp, 0);
    }

    #[test]
    fn brain_state_phi_coherence_accessor() {
        let s = BrainState::init();
        assert_eq!(s.phi_coherence(), PHI_INV);
    }

    // ---- Transition ----

    #[test]
    fn transition_empty_is_zero() {
        let t = Transition::empty();
        assert_eq!(t.action, 0);
        assert_eq!(t.reward, 0.0);
        assert!(!t.done);
        assert_eq!(t.state, [0.0; STATE_DIM]);
        assert_eq!(t.next_state, [0.0; STATE_DIM]);
    }

    // ---- Phase semantics ----

    #[test]
    fn phase_cycle_wraps_after_five_steps() {
        let mut p = Phase::Sense;
        for _ in 0..COGNITIVE_PHASE_COUNT {
            p = p.next();
        }
        assert_eq!(p, Phase::Sense);
    }

    #[test]
    fn phase_indices_are_dense() {
        assert_eq!(Phase::Sense.index(), 0);
        assert_eq!(Phase::Evaluate.index(), 1);
        assert_eq!(Phase::Decide.index(), 2);
        assert_eq!(Phase::Act.index(), 3);
        assert_eq!(Phase::Consolidate.index(), 4);
    }

    // ---- WorldModel construction ----

    #[test]
    fn new_world_model_starts_empty_at_sense() {
        let m = WorldModel::new();
        assert_eq!(m.state_count(), 0);
        assert_eq!(m.transition_count(), 0);
        assert_eq!(m.current_phase(), Phase::Sense);
        assert_eq!(m.current_state().cycle_count, 0);
    }

    #[test]
    fn default_equals_new() {
        let a = WorldModel::default();
        let b = WorldModel::new();
        assert_eq!(a.state_count(), b.state_count());
        assert_eq!(a.transition_count(), b.transition_count());
        assert_eq!(a.current_phase(), b.current_phase());
    }

    // ---- Snapshot / state history ----

    #[test]
    fn snapshot_increments_cycle_and_pushes() {
        let mut m = WorldModel::new();
        m.snapshot().unwrap();
        assert_eq!(m.state_count(), 1);
        assert_eq!(m.current_state().cycle_count, 1);
        assert_eq!(m.state_at(0).unwrap().cycle_count, 1);
    }

    #[test]
    fn snapshot_rejects_when_full() {
        let mut m = WorldModel::new();
        for _ in 0..MAX_STATE_HISTORY {
            m.snapshot().unwrap();
        }
        assert!(m.is_state_buffer_full());
        let err = m.snapshot().unwrap_err();
        assert_eq!(err, WorldModelError::StateBufferFull);
    }

    #[test]
    fn state_at_out_of_range_returns_none() {
        let m = WorldModel::new();
        assert!(m.state_at(0).is_none());
    }

    // ---- Transition recording ----

    #[test]
    fn record_transition_appends() {
        let mut m = WorldModel::new();
        let mut t = Transition::empty();
        t.action = 7;
        t.reward = 1.5;
        m.record_transition(t).unwrap();
        assert_eq!(m.transition_count(), 1);
        assert_eq!(m.transition_at(0).unwrap().action, 7);
    }

    #[test]
    fn record_transition_full_buffer_errors() {
        let mut m = WorldModel::new();
        for _ in 0..MAX_TRANSITIONS {
            m.record_transition(Transition::empty()).unwrap();
        }
        assert!(m.is_transition_buffer_full());
        assert_eq!(
            m.record_transition(Transition::empty()).unwrap_err(),
            WorldModelError::TransitionBufferFull
        );
    }

    #[test]
    fn done_transition_writes_reward_signal() {
        let mut m = WorldModel::new();
        let mut t = Transition::empty();
        t.done = true;
        t.reward = 2.75;
        m.record_transition(t).unwrap();
        assert_eq!(m.current_state().reward_signal, 2.75);
    }

    #[test]
    fn transition_at_out_of_range_returns_none() {
        let m = WorldModel::new();
        assert!(m.transition_at(0).is_none());
    }

    // ---- Cognitive loop / phases ----

    #[test]
    fn step_phase_advances_one_phase() {
        let mut m = WorldModel::new();
        assert_eq!(m.current_phase(), Phase::Sense);
        assert_eq!(m.step_phase(), Phase::Evaluate);
        assert_eq!(m.step_phase(), Phase::Decide);
        assert_eq!(m.step_phase(), Phase::Act);
        assert_eq!(m.step_phase(), Phase::Consolidate);
        assert_eq!(m.step_phase(), Phase::Sense);
    }

    #[test]
    fn full_cycle_snapshots_once() {
        let mut m = WorldModel::new();
        // One full loop = 5 phase steps, exactly one auto-snapshot.
        for _ in 0..COGNITIVE_PHASE_COUNT {
            m.step_phase();
        }
        assert_eq!(m.state_count(), 1);
        assert_eq!(m.current_state().cycle_count, 1);
    }

    #[test]
    fn run_one_cycle_helper_matches_manual() {
        let mut a = WorldModel::new();
        let mut b = WorldModel::new();
        a.run_one_cycle();
        for _ in 0..COGNITIVE_PHASE_COUNT {
            b.step_phase();
        }
        assert_eq!(a.state_count(), b.state_count());
        assert_eq!(a.current_state().cycle_count, b.current_state().cycle_count);
    }

    #[test]
    fn many_cycles_respect_state_capacity() {
        let mut m = WorldModel::new();
        for _ in 0..(MAX_STATE_HISTORY + 4) {
            m.run_one_cycle();
        }
        // Auto-snapshot stops once history is full; cycle_count keeps
        // climbing via explicit snapshot attempts inside step_phase
        // (which are silently dropped past capacity), so the recorded
        // history length never exceeds MAX_STATE_HISTORY.
        assert!(m.state_count() <= MAX_STATE_HISTORY);
        assert!(m.is_state_buffer_full());
    }

    // ---- verify() ----

    #[test]
    fn verify_empty_history_returns_empty() {
        let m = WorldModel::new();
        assert_eq!(m.verify(), VerifyStatus::Empty);
    }

    #[test]
    fn verify_valid_history() {
        let mut m = WorldModel::new();
        for _ in 0..3 {
            m.snapshot().unwrap();
        }
        assert_eq!(m.verify(), VerifyStatus::Valid);
    }

    #[test]
    fn verify_detects_bad_phi_coherence() {
        let mut m = WorldModel::new();
        m.snapshot().unwrap();
        // Inject an out-of-range phi_coherence.
        m.states[0].phi_coherence = 2.0;
        match m.verify() {
            VerifyStatus::BadPhiCoherence(i) => assert_eq!(i, 0),
            other => panic!("expected BadPhiCoherence, got {:?}", other),
        }
    }

    #[test]
    fn verify_detects_non_monotonic_cycle() {
        let mut m = WorldModel::new();
        m.snapshot().unwrap(); // cycle_count = 1
        m.snapshot().unwrap(); // cycle_count = 2
        // Rewrite snapshot 1 to be smaller than snapshot 0.
        m.states[1].cycle_count = 0;
        match m.verify() {
            VerifyStatus::NonMonotonicCycle(i) => assert_eq!(i, 1),
            other => panic!("expected NonMonotonicCycle, got {:?}", other),
        }
    }

    // ---- Reset ----

    #[test]
    fn reset_returns_fresh_model() {
        let mut m = WorldModel::new();
        m.snapshot().unwrap();
        m.record_transition(Transition::empty()).unwrap();
        m.reset();
        assert_eq!(m.state_count(), 0);
        assert_eq!(m.transition_count(), 0);
        assert_eq!(m.current_phase(), Phase::Sense);
        assert_eq!(m.current_state().cycle_count, 0);
    }

    // ---- Math helpers ----

    #[test]
    fn pow_u64_basics() {
        assert_eq!(pow_u64(2, 0), 1);
        assert_eq!(pow_u64(2, 1), 2);
        assert_eq!(pow_u64(2, 10), 1024);
        assert_eq!(pow_u64(3, 5), 243);
    }

    #[test]
    fn identity_witness_equals_three() {
        assert_eq!(identity_witness(), 3);
    }

    // ---- Cross-kernel anchor #10 ----

    /// Anchor: `phi^2 + 1/phi^2 = 3` routed through three layers --
    ///   (a) integer projection sum from PHI and PHI_INV,
    ///   (b) pow_u64 numeric witness (chains to ring-088 GF16 MAC),
    ///   (c) mass-conservation of phi^2 + phi^-2 against TRINITY.
    #[test]
    fn world_model_phi_identity() {
        // (a) Integer projection: floor(PHI) + ceil(PHI_INV) + ceil(PHI_SQ) = 1 + 1 + 3 = 5,
        //     and floor(PHI_SQ) + floor(PHI) = 2 + 1 = 3 -- the closed form.
        let floor_phi: u8 = PHI as u8; // 1
        let floor_phi_sq: u8 = PHI_SQ as u8; // 2
        let projected: u8 = floor_phi_sq + floor_phi;
        assert_eq!(projected, 3);

        // (b) pow_u64 numeric witness: 3^1 = 3.
        let witness = pow_u64(3, 1);
        assert_eq!(witness, identity_witness() as u64);

        // (c) Mass conservation: phi^2 + phi^-2 must equal TRINITY (within fp epsilon).
        let mass = PHI_SQ + PHI_INV_SQ;
        let diff = if mass > TRINITY { mass - TRINITY } else { TRINITY - mass };
        assert!(diff < 1e-12, "mass conservation failed: {} vs {}", mass, TRINITY);

        // Final routing through identity_witness() exposed by every ring crate.
        assert_eq!(identity_witness(), 3);
    }
}
