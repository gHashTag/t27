// Agent file for ball-grants: implements grant system for mailbox access.
// This file is rendered by the host: gHashTag/999-multibots-telegraf

// We'll implement a simple in-memory grant store for demonstration.
// In a real implementation, this would use a persistent database.

interface Grant {
  viewerTelegramId: string;
  client: string; // or "*" for all clients
  expiry: number; // Unix timestamp in seconds
  grantedAt: number; // timestamp when the grant was created
  lastRead: number | null; // timestamp of the last read, null if never read
}

// In-memory store: key is a grant ID (we'll use a simple counter)
let grants: Map<string, Grant> = new Map();
let grantIdCounter = 0;

/**
 * Create a new grant.
 * @param viewerTelegramId - The Telegram ID of the viewer to grant access to.
 * @param client - The client string or "*" for all clients.
 * @param expiryDays - Optional expiry in days. Defaults to 30 days. Must be between 1 and 90.
 * @returns A grant ID string.
 */
export function createGrant(viewerTelegramId: string, client: string, expiryDays?: number): string {
  // Validate input
  if (!viewerTelegramId || !client) {
    throw new Error("Viewer telegram ID and client are required");
  }

  // Set default expiry days
  let days = expiryDays ?? 30;
  if (days < 1) days = 1;
  if (days > 90) days = 90;

  const expiry = Math.floor(Date.now() / 1000) + days * 24 * 60 * 60;
  const grantedAt = Math.floor(Date.now() / 1000);

  const grant: Grant = {
    viewerTelegramId,
    client,
    expiry,
    grantedAt,
    lastRead: null
  };

  const grantId = `grant_${++grantIdCounter}`;
  grants.set(grantId, grant);
  return grantId;
}

/**
 * Check if a grant is valid for a given viewer and client.
 * @param grantId - The ID of the grant to check.
 * @param viewerTelegramId - The Telegram ID of the viewer (for double-checking).
 * @param client - The client string to check against.
 * @returns True if the grant exists, matches the viewer and client, and is not expired.
 */
export function checkGrant(grantId: string, viewerTelegramId: string, client: string): boolean {
  const grant = grants.get(grantId);
  if (!grant) return false;

  // Check that the grant matches the viewer and client
  if (grant.viewerTelegramId !== viewerTelegramId) return false;
  if (grant.client !== client && grant.client !== "*") return false;

  // Check if the grant has expired
  const now = Math.floor(Date.now() / 1000);
  if (grant.expiry <= now) {
    // Optionally, we could remove expired grants here
    return false;
  }

  return true;
}

/**
 * Record a read for a grant.
 * @param grantId - The ID of the grant for which to record a read.
 */
export function recordRead(grantId: string): void {
  const grant = grants.get(grantId);
  if (!grant) return;

  grant.lastRead = Math.floor(Date.now() / 1000);
}

/**
 * Get all grants (for debugging or administrative purposes).
 * @returns An array of grants.
 */
export function getGrants(): Grant[] {
  return Array.from(grants.values());
}

/**
 * Remove expired grants.
 * @returns The number of grants removed.
 */
export function removeExpiredGrants(): number {
  const now = Math.floor(Date.now() / 1000);
  let removed = 0;
  for (const [id, grant] of grants.entries()) {
    if (grant.expiry <= now) {
      grants.delete(id);
      removed++;
    }
  }
  return removed;
}

// We can also export a function to clean up old grants periodically if needed.

// For now, we'll just export the main functions.