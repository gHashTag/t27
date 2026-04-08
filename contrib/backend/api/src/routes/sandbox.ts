// SPDX-License-Identifier: CC0-1.0
// SANDBOX-010 + SANDBOX-011 + SANDBOX-012: Session Timeout, Orphan Detection, HTTPS Enforcement
// Generated from specs/sandbox/*.t27
// HTTPS enforcement is provided by middleware/httpsEnforce.ts (must be registered FIRST)

// ─────────────────────────────────────────────
// Types (mirrors sandbox.session_timeout and sandbox.orphan_detection specs)
// ─────────────────────────────────────────────

/**
 * Session type for sandbox management.
 * Mirrors Rust: Session { id: [u8; 32], name: [u8; 64], railway_id: Option<[u8; 64]>, ... }
 */
export interface Session {
  id: string;          // hex-encoded [u8; 32]
  name: string;        // UTF-8 from [u8; 64]
  status: SessionStatus;
  created_at: Timestamp;
  updated_at: Timestamp;
  railway_id?: string; // hex-encoded [u8; 64] | undefined for None
}

/**
 * Timestamp for session tracking.
 * Mirrors Rust: Timestamp { ms: u64 }
 */
export interface Timestamp {
  ms: number;
}

/**
 * Session status enum matching Rust/PostgreSQL conventions.
 * Mirrors Rust: SessionStatus { Starting, Active, Failed, Terminating, Deleted }
 */
export enum SessionStatus {
  Starting = "starting",
  Active = "active",
  Failed = "failed",
  Terminating = "terminating",
  Deleted = "deleted",
}

/**
 * Health check result type.
 * Mirrors Rust: HealthCheckResult
 */
export interface HealthCheckResult {
  healthy: boolean;
  timed_out_sessions: string[];
  orphaned_sessions: string[];
}

// ─────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────

export const DEFAULT_MAX_SESSION_DURATION_MS = 3_600_000; // 1 hour
export const ORPHANED_THRESHOLD_MINUTES = 15; // Sessions without railway_id for 15+ min are orphans

// ─────────────────────────────────────────────
// Business Logic: Session Timeout (from session_timeout.t27)
// ─────────────────────────────────────────────

/**
 * Check if a session has exceeded its maximum allowed duration.
 *
 * Mirrors Rust: fn should_terminate_session(session: &Session, max_duration_ms: u64) -> bool
 *
 * @param session - The session to check
 * @param max_duration_ms - Maximum allowed duration in milliseconds
 * @return true if session should be terminated
 */
export function shouldTerminateSession(
  session: Session,
  maxDurationMs: number = DEFAULT_MAX_SESSION_DURATION_MS,
): boolean {
  if (session.status !== SessionStatus.Active) {
    return false;
  }

  const elapsed = timestampNowMs() - session.created_at.ms;
  return elapsed > maxDurationMs;
}

// ─────────────────────────────────────────────
// Business Logic: Orphan Detection (from orphan_detection.t27)
// ─────────────────────────────────────────────

/**
 * Check if a session is orphaned.
 *
 * A session is considered orphaned if:
 * 1. It has no railway_id (undefined or empty)
 * 2. Its status is Starting or Active (not Failed/Terminating/Deleted)
 * 3. It was created more than ORPHANED_THRESHOLD_MINUTES ago
 *
 * Mirrors Rust: fn is_session_orphaned(session: &Session, current_time_ms: u64) -> bool
 *
 * @param session - The session to check
 * @param current_time_ms - Current timestamp in milliseconds
 * @return true if session is orphaned
 */
export function isSessionOrphaned(
  session: Session,
  currentTimeMs: number = timestampNowMs(),
): boolean {
  // Must have status that should have a railway resource
  if (session.status !== SessionStatus.Starting && session.status !== SessionStatus.Active) {
    return false;
  }

  // Must have no railway_id (undefined) or empty
  if (session.railway_id && session.railway_id.length > 0) {
    return false; // Has valid railway_id
  }

  // Must be old enough to be considered orphaned
  const ageMs = currentTimeMs - session.created_at.ms;
  const thresholdMs = ORPHANED_THRESHOLD_MINUTES * 60_000;
  return ageMs >= thresholdMs;
}

/**
 * Find all orphaned sessions in a list.
 *
 * Mirrors Rust: fn detect_orphaned_sessions(sessions: &[Session], current_time_ms: u64) -> Vec<[u8; 32]>
 *
 * @param sessions - List of sessions to check
 * @param current_time_ms - Current timestamp in milliseconds
 * @return List of orphaned session IDs
 */
export function detectOrphanedSessions(
  sessions: Session[],
  currentTimeMs: number = timestampNowMs(),
): string[] {
  return sessions
    .filter((s) => isSessionOrphaned(s, currentTimeMs))
    .map((s) => s.id);
}

// ─────────────────────────────────────────────
// Business Logic: Health Check (from health.t27)
// ─────────────────────────────────────────────

/**
 * Perform comprehensive health check on a session.
 *
 * Mirrors Rust: fn check_session_health(session: &Session) -> HealthCheckResult
 *
 * @param session - The session to check
 * @return Health check result
 */
export function checkSessionHealth(session: Session): HealthCheckResult {
  const result: HealthCheckResult = {
    healthy: true,
    timed_out_sessions: [],
    orphaned_sessions: [],
  };

  // Check for timeout
  if (shouldTerminateSession(session)) {
    result.healthy = false;
    result.timed_out_sessions.push(session.id);
  }

  // Check for orphaned status
  if (isSessionOrphaned(session)) {
    result.healthy = false;
    result.orphaned_sessions.push(session.id);
  }

  return result;
}

/**
 * Get current timestamp in milliseconds.
 * Mirrors Rust: fn timestamp_now_ms() -> u64
 */
export function timestampNowMs(): number {
  return Date.now();
}

// ─────────────────────────────────────────────
// Route Handlers (thin routing layer)
// ─────────────────────────────────────────────

export async function createSession(req: any, res: any): Promise<void> {
  const sessionId = generateSessionId();
  const now = timestampNowMs();

  const session: Session = {
    id: sessionId,
    name: req.body?.name || "unnamed-session",
    status: SessionStatus.Starting,
    created_at: { ms: now },
    updated_at: { ms: now },
    // railway_id will be set when Railway deployment is created
  };

  // In production, save to database here

  res.status(201).json({
    id: session.id,
    status: session.status,
    created_at: session.created_at,
  });
}

export async function checkSessionHealthHandler(req: any, res: any): Promise<void> {
  const sessionId = req.params.id;

  // In production, fetch from database
  const session: Session | null = await getSessionById(sessionId);

  if (!session) {
    return res.status(404).json({ error: "Session not found" });
  }

  const health = checkSessionHealth(session);

  if (!health.healthy) {
    const reasons: string[] = [];
    if (health.timed_out_sessions.includes(sessionId)) {
      reasons.push("timeout");
      await updateSessionStatus(sessionId, SessionStatus.Terminating);
    }
    if (health.orphaned_sessions.includes(sessionId)) {
      reasons.push("orphaned");
      await updateSessionStatus(sessionId, SessionStatus.Failed);
    }

    return res.status(200).json({
      id: session.id,
      status: session.status,
      healthy: false,
      reasons,
    });
  }

  res.status(200).json({
    id: session.id,
    status: session.status,
    healthy: true,
  });
}

export async function scanOrphans(req: any, res: any): Promise<void> {
  // In production, fetch all sessions from database
  const sessions: Session[] = await getAllSessions();

  const orphanIds = detectOrphanedSessions(sessions);

  // Mark orphaned sessions as failed for cleanup
  for (const id of orphanIds) {
    await updateSessionStatus(id, SessionStatus.Failed);
  }

  res.status(200).json({
    orphaned_sessions: orphanIds,
    count: orphanIds.length,
  });
}

// ─────────────────────────────────────────────
// Mock Database Functions (to be replaced)
// ─────────────────────────────────────────────

function generateSessionId(): string {
  return crypto.randomUUID();
}

async function getSessionById(id: string): Promise<Session | null> {
  // TODO: Replace with actual database query
  return null;
}

async function getAllSessions(): Promise<Session[]> {
  // TODO: Replace with actual database query
  return [];
}

async function updateSessionStatus(
  id: string,
  status: SessionStatus,
): Promise<void> {
  // TODO: Replace with actual database update
  console.log(`[MOCK] Updating session ${id} to ${status}`);
}
