// SPDX-License-Identifier: Apache-2.0
// ring-094-rust :: AGI Runtime (T27 Wave 21)
//
// Mirrors the runtime triad of specs/runtime/{execute,instance,process}.t27.
//
// Scope:
//   * Process / Task / Instance core enums and constants (byte-for-byte from spec).
//   * Trinity scheduler: ternary task priority via `Trit`, fixed-capacity
//     ready-queue, phi-weighted credit policy (no_std, zero-alloc, zero-deps).
//   * Sixth cross-kernel anchor test exercising phi^2 + 1/phi^2 = 3 through
//     the scheduler's credit accumulator.
//
// Out of scope:
//   * Real syscalls (spawn / kill / PTY I/O) -- runtime stays purely logical.
//   * Heap-backed Vec/HashMap -- only fixed-size arrays.
//   * Future wakers / executors -- promises are pure state machines.
//
// Constitutional:
//   L1 TRACEABILITY -- Closes #729.
//   L3 PURITY       -- ASCII source, English doc-comments.
//   L4 TESTABILITY  -- exhaustive #[test] blocks below.
//   L5 IDENTITY     -- phi^2 + 1/phi^2 = 3.
//   L6 CEILING      -- spec constants mirrored byte-for-byte; no kernel drift.

#![no_std]
#![forbid(unsafe_code)]
#![deny(warnings)]

// ============================================================================
// Sacred constants (T27 Trinity)
// ============================================================================

/// Golden ratio, phi = (1 + sqrt(5)) / 2.
pub const PHI: f64 = 1.618_033_988_749_894_8;

/// 1 / phi = phi - 1.
pub const PHI_INV: f64 = 0.618_033_988_749_894_8;

/// Trinity anchor: phi^2 + 1/phi^2 = 3.
pub const TRINITY_ANCHOR: f64 = 3.0;

/// Numerical tolerance for phi-identity tests.
pub const PHI_EPSILON: f64 = 1.0e-9;

// ============================================================================
// Spec constants -- runtime/execute.t27
// ============================================================================

/// Default execution timeout in milliseconds.
pub const DEFAULT_TIMEOUT_MS: u32 = 30_000;

/// Maximum concurrent executions.
pub const MAX_CONCURRENT_EXECUTIONS: u8 = 16;

/// Execution poll interval in milliseconds.
pub const POLL_INTERVAL_MS: u32 = 100;

/// Task identifier length in bytes.
pub const TASK_ID_LENGTH: usize = 32;

// ============================================================================
// Spec constants -- runtime/instance.t27
// ============================================================================

/// Maximum number of instances in the registry.
pub const MAX_INSTANCES: u16 = 256;

/// Instance name length in bytes (upper bound for storage).
pub const INSTANCE_NAME_LENGTH: usize = 128;

/// Instance lookup timeout in milliseconds.
pub const LOOKUP_TIMEOUT_MS: u32 = 100;

// ============================================================================
// Spec constants -- runtime/process.t27
// ============================================================================

/// Default spawn timeout in milliseconds.
pub const SPAWN_TIMEOUT_MS: u32 = 5_000;

/// Default PTY width in columns.
pub const PTY_COLS_DEFAULT: u16 = 80;

/// Default PTY height in rows.
pub const PTY_ROWS_DEFAULT: u16 = 24;

/// Maximum pipe buffer size in bytes (64 KiB).
pub const MAX_PIPE_BUFFER: u32 = 65_536;

// ============================================================================
// Ternary primitive
// ============================================================================

/// A balanced ternary digit: {-1, 0, +1}.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    /// Project the trit onto the integer line.
    pub const fn to_i8(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    /// Reconstruct a trit from a small integer.
    pub const fn from_i8(v: i8) -> Self {
        if v < 0 {
            Trit::Neg
        } else if v == 0 {
            Trit::Zero
        } else {
            Trit::Pos
        }
    }
}

// ============================================================================
// Enums -- execute.t27
// ============================================================================

/// Execution result discriminator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExecResultType {
    Success = 0,
    Timeout = 1,
    Cancelled = 2,
    Error = 3,
}

/// Task lifecycle state.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TaskState {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Cancelled = 4,
}

impl TaskState {
    /// A task is terminal once it cannot legitimately re-enter the queue.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Cancellation reason.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CancelReason {
    UserRequested = 0,
    Timeout = 1,
    Error = 2,
    Shutdown = 3,
}

// ============================================================================
// Enums -- process.t27
// ============================================================================

/// POSIX-style signal subset relevant to the runtime.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ProcessSignal {
    Terminate = 0,
    Kill = 1,
    Interrupt = 2,
    Hangup = 3,
    Stop = 4,
    Continue = 5,
}

/// Process lifecycle state.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ProcessState {
    NotStarted = 0,
    Running = 1,
    Stopped = 2,
    Terminated = 3,
    Zombie = 4,
}

/// PTY mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PTYMode {
    Raw = 0,
    Cooked = 1,
    Echo = 2,
}

// ============================================================================
// Enums -- instance.t27
// ============================================================================

/// Instance lifecycle state.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InstanceState {
    Registering = 0,
    Active = 1,
    Suspended = 2,
    Terminating = 3,
    Terminated = 4,
}

/// Instance type label.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InstanceType {
    Agent = 0,
    Server = 1,
    Worker = 2,
    Background = 3,
}

/// Reason an instance was terminated.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TerminationReason {
    Normal = 0,
    Error = 1,
    Timeout = 2,
    Cancelled = 3,
    ForceKilled = 4,
}

// ============================================================================
// Identifiers and small types
// ============================================================================

/// Process identifier (mirrors `pub const ProcessID = u32;`).
pub type ProcessId = u32;

/// Instance identifier.
pub type InstanceId = u32;

/// Task identifier (fixed-size byte array).
pub type TaskId = [u8; TASK_ID_LENGTH];

/// Allocate a deterministic task id from a counter and a small seed.
///
/// We avoid randomness so test outputs are reproducible. Bytes are derived
/// from the little-endian encoding of `counter` xor'd with the seed pattern.
pub fn task_id_from_counter(counter: u64, seed: u8) -> TaskId {
    let mut id = [0u8; TASK_ID_LENGTH];
    let raw = counter.to_le_bytes();
    let mut i = 0;
    while i < TASK_ID_LENGTH {
        id[i] = raw[i % raw.len()] ^ seed.wrapping_add(i as u8);
        i += 1;
    }
    id
}

// ============================================================================
// Task descriptor
// ============================================================================

/// Compact, no-alloc task descriptor.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    /// Ternary priority anchor: Neg = low, Zero = normal, Pos = high.
    pub priority: Trit,
    /// Timeout budget in milliseconds.
    pub timeout_ms: u32,
    /// Logical duration accumulated by the scheduler.
    pub duration_ms: u64,
}

impl Task {
    /// Construct a pending task with the spec default timeout.
    pub fn new(id: TaskId, priority: Trit) -> Self {
        Self {
            id,
            state: TaskState::Pending,
            priority,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            duration_ms: 0,
        }
    }

    /// Construct a pending task with an explicit timeout.
    pub fn with_timeout(id: TaskId, priority: Trit, timeout_ms: u32) -> Self {
        Self {
            id,
            state: TaskState::Pending,
            priority,
            timeout_ms,
            duration_ms: 0,
        }
    }

    /// True if the task has consumed its timeout budget.
    pub fn is_expired(self) -> bool {
        self.duration_ms >= self.timeout_ms as u64
    }
}

// ============================================================================
// Promise (state machine only)
// ============================================================================

/// Promise resolution status.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PromiseStatus {
    Pending,
    Resolved,
    Rejected,
    Cancelled,
}

/// Pure-state-machine promise. No executor / wakers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Promise {
    pub task_id: TaskId,
    pub status: PromiseStatus,
    pub result: ExecResultType,
}

impl Promise {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            status: PromiseStatus::Pending,
            result: ExecResultType::Success,
        }
    }

    pub fn resolve(&mut self, result: ExecResultType) {
        if self.status == PromiseStatus::Pending {
            self.status = PromiseStatus::Resolved;
            self.result = result;
        }
    }

    pub fn reject(&mut self) {
        if self.status == PromiseStatus::Pending {
            self.status = PromiseStatus::Rejected;
            self.result = ExecResultType::Error;
        }
    }

    pub fn cancel(&mut self, _reason: CancelReason) {
        if self.status == PromiseStatus::Pending {
            self.status = PromiseStatus::Cancelled;
            self.result = ExecResultType::Cancelled;
        }
    }

    pub fn is_pending(self) -> bool {
        self.status == PromiseStatus::Pending
    }

    pub fn is_resolved(self) -> bool {
        self.status == PromiseStatus::Resolved
    }

    pub fn is_rejected(self) -> bool {
        self.status == PromiseStatus::Rejected
    }

    pub fn is_cancelled(self) -> bool {
        self.status == PromiseStatus::Cancelled
    }
}

// ============================================================================
// Process descriptor
// ============================================================================

/// Compact process descriptor.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub state: ProcessState,
    pub exit_code: Option<u8>,
}

impl ProcessInfo {
    pub fn new(pid: ProcessId) -> Self {
        Self {
            pid,
            state: ProcessState::NotStarted,
            exit_code: None,
        }
    }

    /// Logical state transition (no real syscalls). Disallows resurrection
    /// from terminal states.
    pub fn transition(&mut self, next: ProcessState) -> bool {
        let valid = match (self.state, next) {
            (ProcessState::NotStarted, ProcessState::Running) => true,
            (ProcessState::Running, ProcessState::Stopped) => true,
            (ProcessState::Stopped, ProcessState::Running) => true,
            (ProcessState::Running, ProcessState::Terminated) => true,
            (ProcessState::Stopped, ProcessState::Terminated) => true,
            (ProcessState::Terminated, ProcessState::Zombie) => true,
            _ => false,
        };
        if valid {
            self.state = next;
        }
        valid
    }

    pub fn set_exit_code(&mut self, code: u8) {
        self.exit_code = Some(code);
    }

    pub fn is_alive(self) -> bool {
        matches!(self.state, ProcessState::Running | ProcessState::Stopped)
    }
}

// ============================================================================
// Instance descriptor
// ============================================================================

/// Compact instance descriptor.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Instance {
    pub id: InstanceId,
    pub pid: ProcessId,
    pub kind: InstanceType,
    pub state: InstanceState,
}

impl Instance {
    pub fn agent(id: InstanceId, pid: ProcessId) -> Self {
        Self {
            id,
            pid,
            kind: InstanceType::Agent,
            state: InstanceState::Registering,
        }
    }

    pub fn server(id: InstanceId, pid: ProcessId) -> Self {
        Self {
            id,
            pid,
            kind: InstanceType::Server,
            state: InstanceState::Registering,
        }
    }

    pub fn worker(id: InstanceId, pid: ProcessId) -> Self {
        Self {
            id,
            pid,
            kind: InstanceType::Worker,
            state: InstanceState::Registering,
        }
    }

    pub fn background(id: InstanceId, pid: ProcessId) -> Self {
        Self {
            id,
            pid,
            kind: InstanceType::Background,
            state: InstanceState::Registering,
        }
    }

    pub fn activate(&mut self) {
        if self.state == InstanceState::Registering {
            self.state = InstanceState::Active;
        }
    }

    pub fn suspend(&mut self) {
        if self.state == InstanceState::Active {
            self.state = InstanceState::Suspended;
        }
    }

    pub fn resume(&mut self) {
        if self.state == InstanceState::Suspended {
            self.state = InstanceState::Active;
        }
    }

    pub fn terminate(&mut self, _reason: TerminationReason) {
        match self.state {
            InstanceState::Terminated | InstanceState::Terminating => {}
            _ => {
                self.state = InstanceState::Terminating;
            }
        }
    }

    pub fn finalize(&mut self) {
        if self.state == InstanceState::Terminating {
            self.state = InstanceState::Terminated;
        }
    }

    pub fn is_active(self) -> bool {
        self.state == InstanceState::Active
    }
}

// ============================================================================
// Instance registry (fixed-capacity, no_std)
// ============================================================================

/// Maximum registry capacity bounded by spec `MAX_INSTANCES`.
pub const REGISTRY_CAPACITY: usize = MAX_INSTANCES as usize;

/// Fixed-capacity instance registry. Slot index doubles as a fast handle.
#[derive(Copy, Clone)]
pub struct Registry {
    slots: [Option<Instance>; REGISTRY_CAPACITY],
    len: u16,
}

impl Registry {
    pub const fn new() -> Self {
        Self {
            slots: [None; REGISTRY_CAPACITY],
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Register a new instance. Returns its handle on success.
    pub fn register(&mut self, inst: Instance) -> Result<usize, RuntimeError> {
        let mut i = 0;
        while i < REGISTRY_CAPACITY {
            if self.slots[i].is_none() {
                self.slots[i] = Some(inst);
                self.len += 1;
                return Ok(i);
            }
            i += 1;
        }
        Err(RuntimeError::RegistryFull)
    }

    /// Unregister by handle.
    pub fn unregister(&mut self, handle: usize) -> Result<Instance, RuntimeError> {
        if handle >= REGISTRY_CAPACITY {
            return Err(RuntimeError::HandleOutOfRange);
        }
        match self.slots[handle].take() {
            Some(inst) => {
                self.len -= 1;
                Ok(inst)
            }
            None => Err(RuntimeError::HandleEmpty),
        }
    }

    /// Lookup by instance id.
    pub fn lookup(&self, id: InstanceId) -> Option<Instance> {
        let mut i = 0;
        while i < REGISTRY_CAPACITY {
            if let Some(inst) = self.slots[i] {
                if inst.id == id {
                    return Some(inst);
                }
            }
            i += 1;
        }
        None
    }

    /// Count active instances.
    pub fn active_count(&self) -> usize {
        let mut count = 0usize;
        let mut i = 0;
        while i < REGISTRY_CAPACITY {
            if let Some(inst) = self.slots[i] {
                if inst.is_active() {
                    count += 1;
                }
            }
            i += 1;
        }
        count
    }

    /// Count instances of a given type.
    pub fn count_by_type(&self, kind: InstanceType) -> usize {
        let mut count = 0usize;
        let mut i = 0;
        while i < REGISTRY_CAPACITY {
            if let Some(inst) = self.slots[i] {
                if inst.kind == kind {
                    count += 1;
                }
            }
            i += 1;
        }
        count
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Runtime errors
// ============================================================================

/// Errors surfaced by the runtime registry / scheduler.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    RegistryFull,
    HandleOutOfRange,
    HandleEmpty,
    SchedulerFull,
    SchedulerEmpty,
    TaskNotRunnable,
}

// ============================================================================
// Trinity Scheduler
// ============================================================================

/// Scheduler ready-queue capacity. Pinned to `MAX_CONCURRENT_EXECUTIONS` from spec.
pub const SCHEDULER_CAPACITY: usize = MAX_CONCURRENT_EXECUTIONS as usize;

/// Phi-weighted credit policy applied to a ternary priority.
///
/// We map ternary priorities {-1, 0, +1} onto multiplicative credit weights:
///   * `Trit::Neg`  -> phi^-2 (least credit)
///   * `Trit::Zero` -> 1.0
///   * `Trit::Pos`  -> phi^2 (most credit)
///
/// The Trinity identity then states: weight(Pos) + weight(Neg) == 3 == 1 + 1 + 1
/// (i.e. equals the cardinality of the trit alphabet), giving the credit policy
/// a closed-form, exact-up-to-rounding mass-conservation law.
pub fn priority_to_credit(p: Trit) -> f64 {
    match p {
        Trit::Pos => PHI * PHI,
        Trit::Zero => 1.0,
        Trit::Neg => PHI_INV * PHI_INV,
    }
}

/// no_std-safe absolute value (avoids `f64::abs`, which needs libm in no_std).
#[cfg(test)]
#[inline]
fn fabs_no_std(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// Fixed-capacity scheduler. No allocator, no syscalls, no atomics.
#[derive(Copy, Clone)]
pub struct Scheduler {
    queue: [Option<Task>; SCHEDULER_CAPACITY],
    len: u8,
    /// Accumulated credit consumed by tick() calls. Used by the anchor test.
    credits_accumulated: f64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            queue: [None; SCHEDULER_CAPACITY],
            len: 0,
            credits_accumulated: 0.0,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        SCHEDULER_CAPACITY
    }

    pub fn credits_accumulated(&self) -> f64 {
        self.credits_accumulated
    }

    /// Submit a pending task to the ready-queue.
    pub fn submit(&mut self, task: Task) -> Result<(), RuntimeError> {
        if task.state.is_terminal() {
            return Err(RuntimeError::TaskNotRunnable);
        }
        let mut i = 0;
        while i < SCHEDULER_CAPACITY {
            if self.queue[i].is_none() {
                self.queue[i] = Some(task);
                self.len += 1;
                return Ok(());
            }
            i += 1;
        }
        Err(RuntimeError::SchedulerFull)
    }

    /// Pick the next task: highest-priority first; ties broken by earliest slot.
    ///
    /// Mapping: Pos > Zero > Neg.
    fn pick_index(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_score: i8 = i8::MIN;
        let mut i = 0;
        while i < SCHEDULER_CAPACITY {
            if let Some(t) = self.queue[i] {
                if t.state == TaskState::Pending || t.state == TaskState::Running {
                    let score = t.priority.to_i8();
                    if score > best_score {
                        best_score = score;
                        best = Some(i);
                    }
                }
            }
            i += 1;
        }
        best
    }

    /// Advance the highest-priority runnable task by `slice_ms` ms.
    ///
    /// Returns the task's `TaskId` and the credit consumed on this tick.
    /// Completed / cancelled / expired tasks are evicted on the spot.
    pub fn tick(&mut self, slice_ms: u32) -> Result<(TaskId, f64), RuntimeError> {
        let idx = self.pick_index().ok_or(RuntimeError::SchedulerEmpty)?;
        let mut task = self.queue[idx].expect("pick_index returned Some");
        // Transition to Running on first slice.
        if task.state == TaskState::Pending {
            task.state = TaskState::Running;
        }
        task.duration_ms = task.duration_ms.saturating_add(slice_ms as u64);

        // Charge phi-weighted credit proportional to slice length.
        let credit = priority_to_credit(task.priority) * (slice_ms as f64);
        self.credits_accumulated += credit;

        let id = task.id;
        if task.is_expired() {
            // `task.state = TaskState::Failed` stood here and was dead: the next
            // line drops the task out of the queue, so the field was written on a
            // value about to be discarded. `#![deny(warnings)]` caught it the
            // moment this crate could compile at all -- it never could, because
            // rings/ was in neither workspace.members nor workspace.exclude.
            //
            // Removed rather than kept: the behaviour is identical, and if an
            // expired task is meant to be RECORDED as failed rather than
            // forgotten, that is a change to what this function does and wants
            // deciding on purpose.
            self.queue[idx] = None;
            self.len -= 1;
        } else {
            self.queue[idx] = Some(task);
        }
        Ok((id, credit))
    }

    /// Mark a task complete by id. Returns true if found and evicted.
    pub fn complete(&mut self, id: &TaskId) -> bool {
        let mut i = 0;
        while i < SCHEDULER_CAPACITY {
            if let Some(t) = self.queue[i] {
                if &t.id == id {
                    self.queue[i] = None;
                    self.len -= 1;
                    return true;
                }
            }
            i += 1;
        }
        false
    }

    /// Cancel a task by id with a reason. Returns true if found and evicted.
    pub fn cancel(&mut self, id: &TaskId, _reason: CancelReason) -> bool {
        self.complete(id)
    }

    /// Drain all pending tasks (used on shutdown).
    pub fn shutdown(&mut self) -> usize {
        let cleared = self.len();
        self.queue = [None; SCHEDULER_CAPACITY];
        self.len = 0;
        cleared
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Identity witness -- cross-kernel anchor
// ============================================================================

/// Numerical witness for `phi^2 + 1/phi^2 = 3`.
pub fn identity_witness() -> f64 {
    PHI * PHI + PHI_INV * PHI_INV
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Sacred constants ------------------------------------------------

    #[test]
    fn phi_inverse_relation() {
        // 1/phi == phi - 1.
        let lhs = 1.0_f64 / PHI;
        let rhs = PHI - 1.0;
        assert!(fabs_no_std(lhs - rhs) < PHI_EPSILON);
        assert!(fabs_no_std(PHI_INV - rhs) < PHI_EPSILON);
    }

    #[test]
    fn identity_witness_equals_three() {
        let w = identity_witness();
        assert!(fabs_no_std(w - TRINITY_ANCHOR) < PHI_EPSILON);
    }

    #[test]
    fn spec_constants_match_byte_for_byte() {
        // execute.t27
        assert_eq!(DEFAULT_TIMEOUT_MS, 30_000);
        assert_eq!(MAX_CONCURRENT_EXECUTIONS, 16);
        assert_eq!(POLL_INTERVAL_MS, 100);
        assert_eq!(TASK_ID_LENGTH, 32);
        // instance.t27
        assert_eq!(MAX_INSTANCES, 256);
        assert_eq!(INSTANCE_NAME_LENGTH, 128);
        assert_eq!(LOOKUP_TIMEOUT_MS, 100);
        // process.t27
        assert_eq!(SPAWN_TIMEOUT_MS, 5_000);
        assert_eq!(PTY_COLS_DEFAULT, 80);
        assert_eq!(PTY_ROWS_DEFAULT, 24);
        assert_eq!(MAX_PIPE_BUFFER, 65_536);
    }

    // ----- Trit ------------------------------------------------------------

    #[test]
    fn trit_roundtrips_through_i8() {
        for v in &[-3i8, -1, 0, 1, 5] {
            let t = Trit::from_i8(*v);
            let p = t.to_i8();
            // Projection is sign(v).
            match *v {
                x if x < 0 => assert_eq!(p, -1),
                0 => assert_eq!(p, 0),
                _ => assert_eq!(p, 1),
            }
        }
    }

    // ----- TaskState -------------------------------------------------------

    #[test]
    fn task_state_terminality() {
        assert!(!TaskState::Pending.is_terminal());
        assert!(!TaskState::Running.is_terminal());
        assert!(TaskState::Completed.is_terminal());
        assert!(TaskState::Failed.is_terminal());
        assert!(TaskState::Cancelled.is_terminal());
    }

    // ----- Task id deterministic --------------------------------------------

    #[test]
    fn task_ids_are_deterministic_and_distinct() {
        let a = task_id_from_counter(1, 0);
        let b = task_id_from_counter(1, 0);
        let c = task_id_from_counter(2, 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), TASK_ID_LENGTH);
    }

    // ----- Task construction ------------------------------------------------

    #[test]
    fn task_default_timeout_is_spec_default() {
        let id = task_id_from_counter(7, 1);
        let t = Task::new(id, Trit::Zero);
        assert_eq!(t.timeout_ms, DEFAULT_TIMEOUT_MS);
        assert_eq!(t.state, TaskState::Pending);
        assert!(!t.is_expired());
    }

    #[test]
    fn task_with_timeout_overrides() {
        let id = task_id_from_counter(8, 1);
        let t = Task::with_timeout(id, Trit::Pos, 500);
        assert_eq!(t.timeout_ms, 500);
    }

    #[test]
    fn task_expires_when_duration_reaches_budget() {
        let id = task_id_from_counter(9, 1);
        let mut t = Task::with_timeout(id, Trit::Neg, 100);
        t.duration_ms = 99;
        assert!(!t.is_expired());
        t.duration_ms = 100;
        assert!(t.is_expired());
    }

    // ----- Promise ---------------------------------------------------------

    #[test]
    fn promise_resolves_only_when_pending() {
        let id = task_id_from_counter(10, 2);
        let mut p = Promise::new(id);
        assert!(p.is_pending());
        p.resolve(ExecResultType::Success);
        assert!(p.is_resolved());
        // No further transitions after resolution.
        p.reject();
        assert!(p.is_resolved());
        p.cancel(CancelReason::Shutdown);
        assert!(p.is_resolved());
    }

    #[test]
    fn promise_can_be_cancelled() {
        let id = task_id_from_counter(11, 2);
        let mut p = Promise::new(id);
        p.cancel(CancelReason::UserRequested);
        assert!(p.is_cancelled());
        assert_eq!(p.result, ExecResultType::Cancelled);
    }

    #[test]
    fn promise_can_be_rejected() {
        let id = task_id_from_counter(12, 2);
        let mut p = Promise::new(id);
        p.reject();
        assert!(p.is_rejected());
        assert_eq!(p.result, ExecResultType::Error);
    }

    // ----- ProcessInfo -----------------------------------------------------

    #[test]
    fn process_transitions_follow_lifecycle() {
        let mut p = ProcessInfo::new(42);
        assert_eq!(p.state, ProcessState::NotStarted);
        assert!(p.transition(ProcessState::Running));
        assert!(p.transition(ProcessState::Stopped));
        assert!(p.transition(ProcessState::Running));
        assert!(p.transition(ProcessState::Terminated));
        assert!(p.transition(ProcessState::Zombie));
        // No resurrection from Zombie.
        assert!(!p.transition(ProcessState::Running));
    }

    #[test]
    fn process_alive_predicate() {
        let mut p = ProcessInfo::new(43);
        assert!(!p.is_alive());
        p.transition(ProcessState::Running);
        assert!(p.is_alive());
        p.transition(ProcessState::Stopped);
        assert!(p.is_alive());
        p.transition(ProcessState::Terminated);
        assert!(!p.is_alive());
    }

    #[test]
    fn process_exit_code() {
        let mut p = ProcessInfo::new(44);
        assert!(p.exit_code.is_none());
        p.set_exit_code(7);
        assert_eq!(p.exit_code, Some(7));
    }

    // ----- Instance --------------------------------------------------------

    #[test]
    fn instance_kinds() {
        let a = Instance::agent(1, 100);
        let s = Instance::server(2, 101);
        let w = Instance::worker(3, 102);
        let b = Instance::background(4, 103);
        assert_eq!(a.kind, InstanceType::Agent);
        assert_eq!(s.kind, InstanceType::Server);
        assert_eq!(w.kind, InstanceType::Worker);
        assert_eq!(b.kind, InstanceType::Background);
    }

    #[test]
    fn instance_lifecycle() {
        let mut inst = Instance::agent(7, 7);
        assert_eq!(inst.state, InstanceState::Registering);
        inst.activate();
        assert!(inst.is_active());
        inst.suspend();
        assert_eq!(inst.state, InstanceState::Suspended);
        inst.resume();
        assert!(inst.is_active());
        inst.terminate(TerminationReason::Normal);
        assert_eq!(inst.state, InstanceState::Terminating);
        inst.finalize();
        assert_eq!(inst.state, InstanceState::Terminated);
    }

    // ----- Registry --------------------------------------------------------

    #[test]
    fn registry_register_and_lookup() {
        let mut reg = Registry::new();
        let a = Instance::agent(11, 200);
        let h = reg.register(a).unwrap();
        assert_eq!(reg.len(), 1);
        let got = reg.lookup(11).unwrap();
        assert_eq!(got.id, 11);
        let removed = reg.unregister(h).unwrap();
        assert_eq!(removed.id, 11);
        assert!(reg.is_empty());
    }

    #[test]
    fn registry_counts() {
        let mut reg = Registry::new();
        let mut a = Instance::agent(1, 10);
        a.activate();
        let mut s = Instance::server(2, 11);
        s.activate();
        let w = Instance::worker(3, 12); // stays Registering
        reg.register(a).unwrap();
        reg.register(s).unwrap();
        reg.register(w).unwrap();
        assert_eq!(reg.active_count(), 2);
        assert_eq!(reg.count_by_type(InstanceType::Agent), 1);
        assert_eq!(reg.count_by_type(InstanceType::Server), 1);
        assert_eq!(reg.count_by_type(InstanceType::Worker), 1);
        assert_eq!(reg.count_by_type(InstanceType::Background), 0);
    }

    #[test]
    fn registry_unregister_out_of_range_errors() {
        let mut reg = Registry::new();
        assert_eq!(
            reg.unregister(REGISTRY_CAPACITY + 1),
            Err(RuntimeError::HandleOutOfRange)
        );
        assert_eq!(reg.unregister(0), Err(RuntimeError::HandleEmpty));
    }

    // ----- Scheduler -------------------------------------------------------

    #[test]
    fn scheduler_capacity_pinned_to_spec() {
        let s = Scheduler::new();
        assert_eq!(s.capacity(), MAX_CONCURRENT_EXECUTIONS as usize);
    }

    #[test]
    fn scheduler_picks_highest_priority_first() {
        let mut s = Scheduler::new();
        let low = Task::new(task_id_from_counter(1, 9), Trit::Neg);
        let mid = Task::new(task_id_from_counter(2, 9), Trit::Zero);
        let hi = Task::new(task_id_from_counter(3, 9), Trit::Pos);
        s.submit(low).unwrap();
        s.submit(mid).unwrap();
        s.submit(hi).unwrap();
        let (id_first, _credit_first) = s.tick(10).unwrap();
        assert_eq!(id_first, hi.id);
    }

    #[test]
    fn scheduler_rejects_terminal_tasks() {
        let mut s = Scheduler::new();
        let mut t = Task::new(task_id_from_counter(99, 9), Trit::Zero);
        t.state = TaskState::Completed;
        assert_eq!(s.submit(t), Err(RuntimeError::TaskNotRunnable));
    }

    #[test]
    fn scheduler_fills_to_capacity() {
        let mut s = Scheduler::new();
        let mut i = 0u64;
        while (i as usize) < SCHEDULER_CAPACITY {
            let t = Task::new(task_id_from_counter(i, 5), Trit::Zero);
            assert!(s.submit(t).is_ok());
            i += 1;
        }
        let overflow = Task::new(task_id_from_counter(i, 5), Trit::Zero);
        assert_eq!(s.submit(overflow), Err(RuntimeError::SchedulerFull));
        assert_eq!(s.len(), SCHEDULER_CAPACITY);
    }

    #[test]
    fn scheduler_tick_on_empty_is_error() {
        let mut s = Scheduler::new();
        assert_eq!(s.tick(10), Err(RuntimeError::SchedulerEmpty));
    }

    #[test]
    fn scheduler_complete_removes_task() {
        let mut s = Scheduler::new();
        let t = Task::new(task_id_from_counter(7, 3), Trit::Zero);
        s.submit(t).unwrap();
        assert!(s.complete(&t.id));
        assert!(s.is_empty());
        assert!(!s.complete(&t.id));
    }

    #[test]
    fn scheduler_cancel_removes_task() {
        let mut s = Scheduler::new();
        let t = Task::new(task_id_from_counter(8, 3), Trit::Pos);
        s.submit(t).unwrap();
        assert!(s.cancel(&t.id, CancelReason::UserRequested));
        assert!(s.is_empty());
    }

    #[test]
    fn scheduler_shutdown_clears_queue() {
        let mut s = Scheduler::new();
        let t1 = Task::new(task_id_from_counter(1, 4), Trit::Zero);
        let t2 = Task::new(task_id_from_counter(2, 4), Trit::Pos);
        s.submit(t1).unwrap();
        s.submit(t2).unwrap();
        assert_eq!(s.shutdown(), 2);
        assert!(s.is_empty());
    }

    #[test]
    fn scheduler_expires_runaway_task() {
        let mut s = Scheduler::new();
        let t = Task::with_timeout(task_id_from_counter(50, 5), Trit::Zero, 25);
        s.submit(t).unwrap();
        // tick longer than the timeout budget -> task should be evicted.
        let (id, _credit) = s.tick(100).unwrap();
        assert_eq!(id, t.id);
        assert!(s.is_empty());
    }

    // ----- Priority -> credit ----------------------------------------------

    #[test]
    fn credit_ordering_respects_priority() {
        let neg = priority_to_credit(Trit::Neg);
        let zero = priority_to_credit(Trit::Zero);
        let pos = priority_to_credit(Trit::Pos);
        assert!(neg < zero);
        assert!(zero < pos);
    }

    #[test]
    fn credit_extremes_sum_to_three_per_unit_time() {
        // phi^2 + 1/phi^2 = 3 (Trinity anchor) directly on the credit policy.
        let pos = priority_to_credit(Trit::Pos);
        let neg = priority_to_credit(Trit::Neg);
        assert!(fabs_no_std((pos + neg) - TRINITY_ANCHOR) < PHI_EPSILON);
    }

    // ----- Cross-kernel anchor (sixth) -------------------------------------

    /// Sixth cross-kernel anchor (#6 in the chain that started at ring-088):
    ///
    /// Submit one Pos-priority and one Neg-priority task with identical
    /// timeout budgets, tick each by 1 ms. Accumulated credit should equal
    /// `phi^2 + 1/phi^2 = 3` exactly up to floating-point rounding.
    #[test]
    fn runtime_phi_identity_via_scheduler_credits() {
        let mut s = Scheduler::new();
        let pos = Task::with_timeout(task_id_from_counter(1, 0), Trit::Pos, 1_000);
        let neg = Task::with_timeout(task_id_from_counter(2, 0), Trit::Neg, 1_000);
        s.submit(pos).unwrap();
        s.submit(neg).unwrap();

        // tick(1 ms) on Pos -> charges phi^2; then complete Pos so the
        // priority queue surfaces Neg next; tick(1 ms) on Neg -> charges phi^-2.
        let (id1, c1) = s.tick(1).unwrap();
        assert_eq!(id1, pos.id);
        assert!(s.complete(&pos.id));
        let (id2, c2) = s.tick(1).unwrap();
        assert_eq!(id2, neg.id);

        let total = c1 + c2;
        assert!(fabs_no_std(total - TRINITY_ANCHOR) < PHI_EPSILON);
        assert!(fabs_no_std(s.credits_accumulated() - TRINITY_ANCHOR) < PHI_EPSILON);
    }
}
