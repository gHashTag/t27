// SPDX-License-Identifier: Apache-2.0
// ring-097-rust: Chain-of-Thought reasoning primitives
//
// Mirrors specs/ar/proof_trace.t27 byte-for-byte:
//   - MAX_STEPS = 10 (DARPA CLARA bound)
//   - K3 ternary logic (Trit::{True, Unknown, False})
//   - ProofStep { step_id, operation, inputs, output, timestamp }
//   - ProofTrace { steps, start_timestamp, end_timestamp, verified }
//   - new_proof_trace / add_step / verify_trace / trace_length /
//     is_at_capacity / finalize_trace / format_trace / trit_to_string
//
// Identity anchor: phi^2 + 1/phi^2 = 3 | TRINITY
//
// no_std + no heap: fixed-capacity arrays. Operation names are interned as
// short ASCII byte buffers; inputs are fixed-arity (<= 3 trits per step,
// chosen to cover unary, K3-binary and K3-ternary operators).

#![no_std]
#![deny(warnings)]

// ============================================================================
// 1. Spec constants
// ============================================================================

/// DARPA CLARA bound: proof trace length must be <= 10 steps.
pub const MAX_STEPS: usize = 10;

/// Maximum bytes in an operation name (short ASCII identifier).
pub const MAX_OP_NAME: usize = 24;

/// Maximum number of input trits per step. Covers unary (1),
/// K3-binary (2), and K3-ternary (3) operators.
pub const MAX_INPUTS_PER_STEP: usize = 3;

// ============================================================================
// 2. Errors
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoTError {
    AtCapacity,
    OpNameTooLong,
    TooManyInputs,
}

// ============================================================================
// 3. K3 ternary logic
// ============================================================================

/// K3 (Kleene 3-valued) logic. Maps to ring-089 / ring-093 `Trit`
/// semantically (here we use the proof-trace symbol set: True / Unknown /
/// False, with `Null` reserved for "no output yet" per the spec's
/// verify_trace rejection of Null outputs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Trit {
    False = -1,
    Unknown = 0,
    True = 1,
    /// Reserved sentinel for "step output not yet produced".
    /// `verify_trace` rejects any step whose output is `Null`.
    Null = 2,
}

impl Trit {
    pub fn to_i8(self) -> i8 {
        self as i8
    }

    /// Single-character textual representation used in `format_trace`:
    /// True -> 'T', Unknown -> 'U', False -> 'F', Null -> '?'.
    pub fn to_char(self) -> u8 {
        match self {
            Trit::True => b'T',
            Trit::Unknown => b'U',
            Trit::False => b'F',
            Trit::Null => b'?',
        }
    }
}

// ---- K3 connectives (used inside the anchor / round-trip tests) ----

/// K3 AND: min in the lattice False < Unknown < True.
pub fn k3_and(a: Trit, b: Trit) -> Trit {
    let av = match a {
        Trit::False => -1,
        Trit::Unknown => 0,
        Trit::True => 1,
        Trit::Null => return Trit::Null,
    };
    let bv = match b {
        Trit::False => -1,
        Trit::Unknown => 0,
        Trit::True => 1,
        Trit::Null => return Trit::Null,
    };
    let m = if av < bv { av } else { bv };
    match m {
        -1 => Trit::False,
        0 => Trit::Unknown,
        1 => Trit::True,
        _ => Trit::Null,
    }
}

/// K3 OR: max in the lattice False < Unknown < True.
pub fn k3_or(a: Trit, b: Trit) -> Trit {
    let av = match a {
        Trit::False => -1,
        Trit::Unknown => 0,
        Trit::True => 1,
        Trit::Null => return Trit::Null,
    };
    let bv = match b {
        Trit::False => -1,
        Trit::Unknown => 0,
        Trit::True => 1,
        Trit::Null => return Trit::Null,
    };
    let m = if av > bv { av } else { bv };
    match m {
        -1 => Trit::False,
        0 => Trit::Unknown,
        1 => Trit::True,
        _ => Trit::Null,
    }
}

/// K3 NOT.
pub fn k3_not(a: Trit) -> Trit {
    match a {
        Trit::True => Trit::False,
        Trit::Unknown => Trit::Unknown,
        Trit::False => Trit::True,
        Trit::Null => Trit::Null,
    }
}

// ============================================================================
// 4. ProofStep
// ============================================================================

/// Single step in a chain-of-thought.
///
/// `operation` is held as a fixed-size ASCII buffer + length to keep the
/// crate no_std + heap-free. `inputs` is a fixed-arity array of trits
/// padded with `Trit::Null` for unused slots.
#[derive(Clone, Copy, Debug)]
pub struct ProofStep {
    pub step_id: usize,
    op_name: [u8; MAX_OP_NAME],
    op_len: u8,
    inputs: [Trit; MAX_INPUTS_PER_STEP],
    input_count: u8,
    pub output: Trit,
    /// Microseconds since trace start.
    pub timestamp_us: u64,
}

impl ProofStep {
    fn new(
        step_id: usize,
        op: &str,
        inputs_slice: &[Trit],
        output: Trit,
        timestamp_us: u64,
    ) -> Result<Self, CoTError> {
        if op.len() > MAX_OP_NAME {
            return Err(CoTError::OpNameTooLong);
        }
        if inputs_slice.len() > MAX_INPUTS_PER_STEP {
            return Err(CoTError::TooManyInputs);
        }
        let mut op_name = [0u8; MAX_OP_NAME];
        for (i, b) in op.as_bytes().iter().enumerate() {
            op_name[i] = *b;
        }
        let mut inputs = [Trit::Null; MAX_INPUTS_PER_STEP];
        for (i, t) in inputs_slice.iter().enumerate() {
            inputs[i] = *t;
        }
        Ok(ProofStep {
            step_id,
            op_name,
            op_len: op.len() as u8,
            inputs,
            input_count: inputs_slice.len() as u8,
            output,
            timestamp_us,
        })
    }

    pub fn operation(&self) -> &str {
        let bytes = &self.op_name[..self.op_len as usize];
        // op_name is ASCII by construction (we only accept &str).
        core::str::from_utf8(bytes).unwrap_or("?")
    }

    pub fn input_count(&self) -> usize {
        self.input_count as usize
    }

    pub fn input(&self, i: usize) -> Trit {
        if i < self.input_count() {
            self.inputs[i]
        } else {
            Trit::Null
        }
    }
}

// ============================================================================
// 5. ProofTrace
// ============================================================================

/// Bounded proof trace -- the heart of the CoT primitive.
#[derive(Clone, Copy, Debug)]
pub struct ProofTrace {
    steps: [ProofStep; MAX_STEPS],
    step_count: u8,
    /// Caller-supplied or zero-initialised tick.
    pub start_timestamp_us: u64,
    pub end_timestamp_us: u64,
    pub verified: bool,
}

/// Verification result returned by `verify_trace`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyStatus {
    /// Valid trace within bounds with all step outputs set.
    Valid,
    Empty,
    TooManySteps,
    /// Step at the given index had output == Null.
    NullOutput(usize),
}

/// Create an empty proof trace starting at `start_timestamp_us` (microseconds).
pub fn new_proof_trace(start_timestamp_us: u64) -> ProofTrace {
    let empty_step = ProofStep {
        step_id: 0,
        op_name: [0u8; MAX_OP_NAME],
        op_len: 0,
        inputs: [Trit::Null; MAX_INPUTS_PER_STEP],
        input_count: 0,
        output: Trit::Null,
        timestamp_us: 0,
    };
    ProofTrace {
        steps: [empty_step; MAX_STEPS],
        step_count: 0,
        start_timestamp_us,
        end_timestamp_us: 0,
        verified: false,
    }
}

/// Append a step to the trace.
///
/// Returns `CoTError::AtCapacity` when `step_count == MAX_STEPS`. The step's
/// `step_id` is its insertion index; `timestamp_us` is microseconds *from
/// trace start* (caller supplies the absolute `now_us`).
pub fn add_step(
    trace: &mut ProofTrace,
    operation: &str,
    inputs: &[Trit],
    output: Trit,
    now_us: u64,
) -> Result<(), CoTError> {
    if trace.step_count as usize >= MAX_STEPS {
        return Err(CoTError::AtCapacity);
    }
    let step_id = trace.step_count as usize;
    let ts = now_us.saturating_sub(trace.start_timestamp_us);
    let step = ProofStep::new(step_id, operation, inputs, output, ts)?;
    trace.steps[step_id] = step;
    trace.step_count += 1;
    Ok(())
}

/// Verify that the trace is non-empty, within MAX_STEPS, and every step has
/// a non-Null output (per spec invariant `valid_trace_passes`).
pub fn verify_trace(trace: &ProofTrace) -> VerifyStatus {
    let n = trace.step_count as usize;
    if n == 0 {
        return VerifyStatus::Empty;
    }
    if n > MAX_STEPS {
        return VerifyStatus::TooManySteps;
    }
    for i in 0..n {
        if trace.steps[i].output == Trit::Null {
            return VerifyStatus::NullOutput(i);
        }
    }
    VerifyStatus::Valid
}

/// Number of steps in the trace.
pub fn trace_length(trace: &ProofTrace) -> usize {
    trace.step_count as usize
}

/// True when the trace has reached the spec-mandated maximum.
pub fn is_at_capacity(trace: &ProofTrace) -> bool {
    trace.step_count as usize >= MAX_STEPS
}

/// Stamp the trace as verified at `now_us`.
pub fn finalize_trace(trace: &mut ProofTrace, now_us: u64) {
    trace.end_timestamp_us = now_us;
    trace.verified = true;
}

/// Borrow a step by index. Returns `None` for indices beyond `trace_length`.
pub fn step_at(trace: &ProofTrace, i: usize) -> Option<&ProofStep> {
    if i < trace.step_count as usize {
        Some(&trace.steps[i])
    } else {
        None
    }
}

/// Convert a Trit to a single-character ASCII byte (spec function).
pub fn trit_to_string(t: Trit) -> u8 {
    t.to_char()
}

// ============================================================================
// 6. format_trace -- bounded ASCII rendering
// ============================================================================

/// Maximum size of the formatted-trace buffer.
/// Header + (per step "NN. <op>(T, T, T) = T (timestamp)\n") + footer.
/// We size for the worst case: header 18B, footer 64B, 10 lines x 96B = 960B.
pub const FORMAT_TRACE_BUFFER: usize = 18 + 960 + 64;

/// Write a human-readable rendering of the trace into `out`, returning the
/// number of bytes written. Bounded, no allocations.
///
/// Layout per step (mirroring the spec): `"N. op(T, T) = T (Tus)\n"` where
/// `T` are single-character trit symbols and `N` is the 1-based step index.
pub fn format_trace(trace: &ProofTrace, out: &mut [u8]) -> usize {
    let mut pos = 0usize;
    pos += write_str(out, pos, b"=== Proof Trace ===\n");

    let n = trace.step_count as usize;
    for i in 0..n {
        let step = &trace.steps[i];
        pos += write_usize(out, pos, step.step_id + 1);
        pos += write_str(out, pos, b". ");
        // Operation name
        let op_bytes = &step.op_name[..step.op_len as usize];
        pos += write_bytes(out, pos, op_bytes);
        pos += write_str(out, pos, b"(");
        for k in 0..step.input_count as usize {
            if k > 0 {
                pos += write_str(out, pos, b", ");
            }
            pos += write_byte(out, pos, step.input(k).to_char());
        }
        pos += write_str(out, pos, b") = ");
        pos += write_byte(out, pos, step.output.to_char());
        pos += write_str(out, pos, b" (");
        pos += write_u64(out, pos, step.timestamp_us);
        pos += write_str(out, pos, b"us)\n");
    }

    pos += write_str(out, pos, b"Total: ");
    pos += write_usize(out, pos, n);
    pos += write_str(out, pos, b" steps, verified: ");
    pos += write_str(out, pos, if trace.verified { b"true" } else { b"false" });
    pos += write_str(out, pos, b"\n");

    pos
}

// ---- Private rendering helpers ----

fn write_byte(out: &mut [u8], pos: usize, b: u8) -> usize {
    if pos < out.len() {
        out[pos] = b;
        1
    } else {
        0
    }
}

fn write_str(out: &mut [u8], pos: usize, s: &[u8]) -> usize {
    let mut n = 0;
    for &b in s {
        if pos + n < out.len() {
            out[pos + n] = b;
            n += 1;
        } else {
            break;
        }
    }
    n
}

fn write_bytes(out: &mut [u8], pos: usize, s: &[u8]) -> usize {
    write_str(out, pos, s)
}

fn write_usize(out: &mut [u8], pos: usize, mut v: usize) -> usize {
    if v == 0 {
        return write_byte(out, pos, b'0');
    }
    // Digits buffer (enough for usize).
    let mut buf = [0u8; 20];
    let mut k = 0;
    while v > 0 {
        buf[k] = b'0' + (v % 10) as u8;
        v /= 10;
        k += 1;
    }
    let mut n = 0;
    while k > 0 {
        k -= 1;
        n += write_byte(out, pos + n, buf[k]);
    }
    n
}

fn write_u64(out: &mut [u8], pos: usize, mut v: u64) -> usize {
    if v == 0 {
        return write_byte(out, pos, b'0');
    }
    let mut buf = [0u8; 20];
    let mut k = 0;
    while v > 0 {
        buf[k] = b'0' + (v % 10) as u8;
        v /= 10;
        k += 1;
    }
    let mut n = 0;
    while k > 0 {
        k -= 1;
        n += write_byte(out, pos + n, buf[k]);
    }
    n
}

// ============================================================================
// 7. no_std math helpers
// ============================================================================

/// Fast integer exponentiation by squaring (no libm). Negative exponents
/// invert via `1/base`.
fn pow_u64(base: f64, exp: i32) -> f64 {
    if exp == 0 {
        return 1.0;
    }
    let (b, mut e) = if exp < 0 {
        (1.0 / base, -exp as u32)
    } else {
        (base, exp as u32)
    };
    let mut result = 1.0;
    let mut acc = b;
    while e > 0 {
        if e & 1 == 1 {
            result *= acc;
        }
        acc *= acc;
        e >>= 1;
    }
    result
}

// ============================================================================
// 8. Identity witness
// ============================================================================

/// Closed-form Trinity identity: phi^2 + 1/phi^2 = 3.
pub fn identity_witness() -> f64 {
    let phi: f64 = 1.618_033_988_749_894_8;
    pow_u64(phi, 2) + pow_u64(phi, -2)
}

// ============================================================================
// 9. Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Spec constants ----

    #[test]
    fn spec_max_steps_byte_for_byte() {
        assert_eq!(MAX_STEPS, 10);
    }

    #[test]
    fn spec_trit_values() {
        assert_eq!(Trit::True.to_i8(), 1);
        assert_eq!(Trit::Unknown.to_i8(), 0);
        assert_eq!(Trit::False.to_i8(), -1);
    }

    // ---- K3 connectives (sanity) ----

    #[test]
    fn k3_and_truth_table() {
        assert_eq!(k3_and(Trit::True, Trit::True), Trit::True);
        assert_eq!(k3_and(Trit::True, Trit::False), Trit::False);
        assert_eq!(k3_and(Trit::True, Trit::Unknown), Trit::Unknown);
        assert_eq!(k3_and(Trit::Unknown, Trit::False), Trit::False);
        assert_eq!(k3_and(Trit::Unknown, Trit::Unknown), Trit::Unknown);
    }

    #[test]
    fn k3_or_truth_table() {
        assert_eq!(k3_or(Trit::True, Trit::False), Trit::True);
        assert_eq!(k3_or(Trit::False, Trit::False), Trit::False);
        assert_eq!(k3_or(Trit::Unknown, Trit::False), Trit::Unknown);
        assert_eq!(k3_or(Trit::Unknown, Trit::True), Trit::True);
    }

    #[test]
    fn k3_not_involution() {
        for &t in &[Trit::True, Trit::Unknown, Trit::False] {
            assert_eq!(k3_not(k3_not(t)), t);
        }
    }

    // ---- new_proof_trace ----

    #[test]
    fn new_proof_trace_creates_empty() {
        let t = new_proof_trace(1000);
        assert_eq!(trace_length(&t), 0);
        assert!(!t.verified);
        assert_eq!(t.start_timestamp_us, 1000);
        assert_eq!(t.end_timestamp_us, 0);
    }

    // ---- add_step ----

    #[test]
    fn add_step_increments_count() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "k3_and", &[Trit::True, Trit::True], Trit::True, 5).unwrap();
        add_step(&mut t, "k3_or", &[Trit::True, Trit::False], Trit::True, 10).unwrap();
        assert_eq!(trace_length(&t), 2);
    }

    #[test]
    fn add_step_records_relative_timestamp() {
        let mut t = new_proof_trace(1_000_000);
        add_step(&mut t, "op", &[Trit::True], Trit::True, 1_000_050).unwrap();
        let step = step_at(&t, 0).unwrap();
        assert_eq!(step.timestamp_us, 50);
    }

    #[test]
    fn add_step_fails_when_at_capacity() {
        let mut t = new_proof_trace(0);
        for _ in 0..MAX_STEPS {
            add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        }
        assert!(is_at_capacity(&t));
        let err = add_step(&mut t, "extra", &[Trit::True], Trit::True, 0).unwrap_err();
        assert_eq!(err, CoTError::AtCapacity);
    }

    #[test]
    fn add_step_rejects_too_long_op_name() {
        let mut t = new_proof_trace(0);
        let long_op = "this_operation_name_is_too_long_to_fit";
        assert!(long_op.len() > MAX_OP_NAME);
        let err = add_step(&mut t, long_op, &[Trit::True], Trit::True, 0).unwrap_err();
        assert_eq!(err, CoTError::OpNameTooLong);
    }

    #[test]
    fn add_step_rejects_too_many_inputs() {
        let mut t = new_proof_trace(0);
        let many_inputs = [Trit::True; MAX_INPUTS_PER_STEP + 1];
        let err = add_step(&mut t, "op", &many_inputs, Trit::True, 0).unwrap_err();
        assert_eq!(err, CoTError::TooManyInputs);
    }

    #[test]
    fn add_step_preserves_step_id_as_index() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "a", &[Trit::True], Trit::True, 0).unwrap();
        add_step(&mut t, "b", &[Trit::True], Trit::True, 0).unwrap();
        add_step(&mut t, "c", &[Trit::True], Trit::True, 0).unwrap();
        assert_eq!(step_at(&t, 0).unwrap().step_id, 0);
        assert_eq!(step_at(&t, 1).unwrap().step_id, 1);
        assert_eq!(step_at(&t, 2).unwrap().step_id, 2);
    }

    // ---- verify_trace ----

    #[test]
    fn verify_empty_trace_fails() {
        let t = new_proof_trace(0);
        assert_eq!(verify_trace(&t), VerifyStatus::Empty);
    }

    #[test]
    fn verify_valid_small_trace() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "k3_and", &[Trit::True, Trit::True], Trit::True, 0).unwrap();
        add_step(&mut t, "k3_or", &[Trit::True, Trit::False], Trit::True, 0).unwrap();
        assert_eq!(verify_trace(&t), VerifyStatus::Valid);
    }

    #[test]
    fn verify_accepts_exactly_max_steps() {
        let mut t = new_proof_trace(0);
        for _ in 0..MAX_STEPS {
            add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        }
        assert_eq!(verify_trace(&t), VerifyStatus::Valid);
    }

    #[test]
    fn verify_rejects_null_output() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "ok", &[Trit::True], Trit::True, 0).unwrap();
        add_step(&mut t, "pending", &[Trit::True], Trit::Null, 0).unwrap();
        assert_eq!(verify_trace(&t), VerifyStatus::NullOutput(1));
    }

    // ---- trace_length / is_at_capacity ----

    #[test]
    fn trace_length_reports_correct() {
        let mut t = new_proof_trace(0);
        for _ in 0..4 {
            add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        }
        assert_eq!(trace_length(&t), 4);
    }

    #[test]
    fn is_at_capacity_when_full() {
        let mut t = new_proof_trace(0);
        for _ in 0..MAX_STEPS {
            add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        }
        assert!(is_at_capacity(&t));
    }

    #[test]
    fn is_at_capacity_false_when_partial() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        assert!(!is_at_capacity(&t));
    }

    // ---- finalize_trace ----

    #[test]
    fn finalize_sets_verified_and_end_timestamp() {
        let mut t = new_proof_trace(100);
        add_step(&mut t, "op", &[Trit::True], Trit::True, 110).unwrap();
        finalize_trace(&mut t, 500);
        assert!(t.verified);
        assert_eq!(t.end_timestamp_us, 500);
    }

    // ---- trit_to_string ----

    #[test]
    fn trit_to_string_maps_symbols() {
        assert_eq!(trit_to_string(Trit::True), b'T');
        assert_eq!(trit_to_string(Trit::Unknown), b'U');
        assert_eq!(trit_to_string(Trit::False), b'F');
        assert_eq!(trit_to_string(Trit::Null), b'?');
    }

    // ---- format_trace ----

    #[test]
    fn format_trace_produces_readable_output() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "k3_and", &[Trit::True, Trit::True], Trit::True, 1).unwrap();
        add_step(&mut t, "k3_or", &[Trit::True, Trit::False], Trit::True, 2).unwrap();
        let mut buf = [0u8; FORMAT_TRACE_BUFFER];
        let n = format_trace(&t, &mut buf);
        let rendered = core::str::from_utf8(&buf[..n]).unwrap();
        assert!(rendered.contains("=== Proof Trace ==="));
        assert!(rendered.contains("1. k3_and(T, T) = T"));
        assert!(rendered.contains("2. k3_or(T, F) = T"));
        assert!(rendered.contains("Total: 2 steps, verified: false"));
    }

    #[test]
    fn format_trace_marks_verified_after_finalize() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "op", &[Trit::True], Trit::True, 0).unwrap();
        finalize_trace(&mut t, 100);
        let mut buf = [0u8; FORMAT_TRACE_BUFFER];
        let n = format_trace(&t, &mut buf);
        let rendered = core::str::from_utf8(&buf[..n]).unwrap();
        assert!(rendered.contains("verified: true"));
    }

    // ---- ProofStep accessors ----

    #[test]
    fn step_accessors() {
        let mut t = new_proof_trace(0);
        add_step(&mut t, "k3_and", &[Trit::True, Trit::Unknown], Trit::Unknown, 0).unwrap();
        let s = step_at(&t, 0).unwrap();
        assert_eq!(s.operation(), "k3_and");
        assert_eq!(s.input_count(), 2);
        assert_eq!(s.input(0), Trit::True);
        assert_eq!(s.input(1), Trit::Unknown);
        assert_eq!(s.output, Trit::Unknown);
    }

    #[test]
    fn step_at_out_of_range_returns_none() {
        let t = new_proof_trace(0);
        assert!(step_at(&t, 0).is_none());
        assert!(step_at(&t, MAX_STEPS).is_none());
    }

    // ---- End-to-end: "actual reasoning" chain from the spec ----

    #[test]
    fn proof_trace_with_actual_reasoning() {
        // Spec's `proof_trace_with_actual_reasoning` test:
        // 4-step diagnostic reasoning, then verify.
        let mut t = new_proof_trace(0);
        add_step(&mut t, "input_symptom", &[Trit::True], Trit::True, 1).unwrap();
        add_step(&mut t, "k3_and", &[Trit::True, Trit::True], Trit::True, 2).unwrap();
        add_step(&mut t, "k3_or", &[Trit::True, Trit::Unknown], Trit::True, 3).unwrap();
        add_step(&mut t, "conclusion", &[Trit::True], Trit::True, 4).unwrap();
        assert_eq!(verify_trace(&t), VerifyStatus::Valid);
        assert_eq!(trace_length(&t), 4);
    }

    // ---- pow_u64 ----

    #[test]
    fn pow_u64_basics() {
        assert_eq!(pow_u64(2.0, 0), 1.0);
        assert!((pow_u64(2.0, 10) - 1024.0).abs() < 1e-9);
        assert!((pow_u64(2.0, -3) - 0.125).abs() < 1e-12);
    }

    // ---- Identity witness ----

    #[test]
    fn identity_witness_equals_three() {
        let v = identity_witness();
        assert!((v - 3.0).abs() < 1e-9, "phi^2 + 1/phi^2 = {}", v);
    }

    // ---- Anchor #9: cross-kernel Trinity identity through CoT chain ----
    //
    // Build a bounded proof trace that *reasons* about the identity:
    //   step 1: input  -- "phi positive, > 1"           (True)
    //   step 2: input  -- "1/phi positive, < 1"         (True)
    //   step 3: k3_and -- both positives are True       (True AND True = True)
    //   step 4: derive -- phi^2 + 1/phi^2 evaluated      (True via numeric witness)
    //   step 5: k3_or  -- alternative explanation       (True OR Unknown = True)
    //   step 6: conclude -- identity holds              (True)
    // Then verify the trace, finalize it, and assert the numeric witness ~ 3.0.
    // The chain ties together: K3 logic + bounded buffer + pow_u64 + the
    // sacred identity. This is the eighth cross-kernel anchor in the chain
    // (ring-088 -> 089 -> 091 -> 092 -> 093 -> 094 -> 095 -> 096 -> 097).

    #[test]
    fn cot_phi_identity() {
        let mut t = new_proof_trace(0);

        // Symbolic reasoning steps.
        add_step(&mut t, "phi_pos", &[Trit::True], Trit::True, 1).unwrap();
        add_step(&mut t, "inv_pos", &[Trit::True], Trit::True, 2).unwrap();

        let both_pos = k3_and(Trit::True, Trit::True);
        assert_eq!(both_pos, Trit::True);
        add_step(&mut t, "k3_and", &[Trit::True, Trit::True], both_pos, 3).unwrap();

        // Numeric witness step: the symbolic output is True iff the numeric
        // identity holds within 1e-9.
        let phi: f64 = 1.618_033_988_749_894_8;
        let lhs = pow_u64(phi, 2) + pow_u64(phi, -2);
        let identity_holds = (lhs - 3.0).abs() < 1e-9;
        let derived_output = if identity_holds {
            Trit::True
        } else {
            Trit::False
        };
        assert_eq!(derived_output, Trit::True);
        add_step(&mut t, "derive_id", &[Trit::True, Trit::True], derived_output, 4).unwrap();

        // K3 OR with Unknown still yields True (alternative-path admissible).
        let alt = k3_or(Trit::True, Trit::Unknown);
        assert_eq!(alt, Trit::True);
        add_step(&mut t, "k3_or", &[Trit::True, Trit::Unknown], alt, 5).unwrap();

        add_step(&mut t, "conclude", &[Trit::True], Trit::True, 6).unwrap();

        assert_eq!(verify_trace(&t), VerifyStatus::Valid);
        assert_eq!(trace_length(&t), 6);

        finalize_trace(&mut t, 100);
        assert!(t.verified);

        // The numeric anchor itself: phi^2 + 1/phi^2 = 3.
        assert!((lhs - 3.0).abs() < 1e-9);

        // Mass-conservation hook -- if a Pos-priority subgoal carries weight
        // phi^2 and a Neg-priority subgoal carries weight phi^-2, their
        // ternary OR (max-credit selection) still satisfies the identity
        // numerically:
        let pos_weight = pow_u64(phi, 2);
        let neg_weight = pow_u64(phi, -2);
        assert!((pos_weight + neg_weight - 3.0).abs() < 1e-9);
    }
}
