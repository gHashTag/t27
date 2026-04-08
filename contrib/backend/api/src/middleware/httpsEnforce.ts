// SPDX-License-Identifier: CC0-1.0
// SANDBOX-012: HTTPS Enforcement
// Generated from specs/sandbox/https_enforce.t27

// ─────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────

export const HTTPS_REDIRECT_STATUS = 301;
export const LOCAL_HOSTNAMES = ["localhost", "127.0.0.1", "[::1]"];

// ─────────────────────────────────────────────
// Types (mirrors sandbox.https_enforce spec)
// ─────────────────────────────────────────────

/**
 * Request context containing headers and local flag.
 * Mirrors Rust: RequestContext
 */
export interface RequestContext {
  forwardedProto: string | null;
  host: string;
  isLocal: boolean;
}

// ─────────────────────────────────────────────
// Core Logic (from .t27 spec)
// ─────────────────────────────────────────────

/**
 * Determine if a request should be redirected to HTTPS.
 *
 * Redirect rules:
 * 1. Never redirect in local mode (localhost/127.0.0.1/::1)
 * 2. Never redirect if X-Forwarded-Proto is "https"
 * 3. Always redirect if X-Forwarded-Proto is "http" or missing
 *
 * Mirrors Rust: fn should_redirect(ctx: &RequestContext) -> bool
 *
 * @param ctx - Request context containing headers and local flag
 * @return true if redirect to HTTPS is required
 */
export function shouldRedirect(ctx: RequestContext): boolean {
  // Local mode: never redirect
  if (ctx.isLocal) {
    return false;
  }

  // Check X-Forwarded-Proto header
  const proto = ctx.forwardedProto?.toLowerCase();
  return proto !== "https";
}

/**
 * Build HTTPS redirect URL from original HTTP URL.
 *
 * Mirrors Rust: fn redirect_url(original_url: &[u8]) -> Vec<u8>
 *
 * @param originalUrl - Original HTTP URL
 * @return HTTPS URL
 */
export function redirectUrl(originalUrl: string): string {
  if (originalUrl.startsWith("https://")) {
    return originalUrl; // Already HTTPS
  }
  return originalUrl.replace(/^http:\/\//i, "https://");
}

/**
 * Check if a hostname indicates a local development environment.
 *
 * Mirrors Rust: fn is_local_hostname(host: &[u8]) -> bool
 *
 * @param host - Hostname to check
 * @return true if hostname is local
 */
export function isLocalHostname(host: string): boolean {
  return LOCAL_HOSTNAMES.includes(host);
}

/**
 * Create request context from Express request.
 *
 * @param req - Express request object
 * @return Request context
 */
export function createRequestContext(req: any): RequestContext {
  const host = req.hostname || req.headers.host || "localhost";
  const forwardedProto = (req.headers["x-forwarded-proto"] as string) || null;

  return {
    forwardedProto,
    host,
    isLocal: isLocalHostname(host),
  };
}

// ─────────────────────────────────────────────
// Express Middleware
// ─────────────────────────────────────────────

/**
 * Express middleware to enforce HTTPS.
 *
 * Must be registered FIRST in the middleware chain.
 * Redirects HTTP to HTTPS in production; allows HTTP for local dev.
 *
 * Usage:
 *   app.use(httpsEnforceMiddleware());
 *
 * Mirrors Rust: trait HttpsEnforcer
 */
export function httpsEnforceMiddleware(req: any, res: any, next: () => void): void {
  const ctx = createRequestContext(req);

  if (shouldRedirect(ctx)) {
    // Build HTTPS URL
    const protocol = req.secure ? "https" : "http";
    const host = req.headers.host;
    const originalUrl = req.originalUrl || req.url;
    const httpsUrl = `${protocol === "http" ? "https" : protocol}://${host}${originalUrl}`;

    return res.redirect(HTTPS_REDIRECT_STATUS, httpsUrl);
  }

  next();
}

/**
 * HTTPS enforcer class with enforce method.
 * Mirrors Rust: trait HttpsEnforcer
 */
export class HttpsEnforcer {
  /**
   * Check if request should redirect and build redirect URL.
   *
   * Mirrors Rust: fn enforce(&self, ctx: &RequestContext) -> Option<Vec<u8>>
   *
   * @param ctx - Request context
   * @return null if no redirect, https_url string if redirect needed
   */
  enforce(ctx: RequestContext): string | null {
    if (shouldRedirect(ctx)) {
      const protocol = ctx.isLocal ? "http" : "https";
      return `${protocol}://${ctx.host}`;
    }
    return null;
  }
}
