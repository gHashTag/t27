import type { Request, Response, NextFunction } from "express";

/**
 * Simple in-memory rate limiter.
 *
 * Tracks request counts per IP within a sliding window.  When the limit is
 * exceeded the client receives a 429 response with a Retry-After header.
 *
 * Not designed for multi-process deployments — use Redis-backed rate limiting
 * in production clusters.
 */
export const createRateLimiter = ({
  windowMs = 15 * 60 * 1000, // 15 minutes
  maxAttempts = 10,
}: {
  windowMs?: number;
  maxAttempts?: number;
} = {}) => {
  const attempts = new Map<string, { count: number; resetAt: number }>();

  // Periodic cleanup every windowMs to prevent unbounded memory growth.
  const cleanup = setInterval(() => {
    const now = Date.now();
    for (const [key, entry] of attempts) {
      if (entry.resetAt <= now) {
        attempts.delete(key);
      }
    }
  }, windowMs);

  // Allow the timer to not block process exit.
  if (cleanup.unref) cleanup.unref();

  return (req: Request, res: Response, next: NextFunction) => {
    const ip = req.ip ?? req.socket.remoteAddress ?? "unknown";
    const now = Date.now();

    let entry = attempts.get(ip);

    if (!entry || entry.resetAt <= now) {
      entry = { count: 0, resetAt: now + windowMs };
      attempts.set(ip, entry);
    }

    entry.count += 1;

    if (entry.count > maxAttempts) {
      const retryAfter = Math.ceil((entry.resetAt - now) / 1000);
      res.set("Retry-After", String(retryAfter));
      res.status(429).json({
        error: `Too many attempts. Try again in ${retryAfter}s.`,
      });
      return;
    }

    next();
  };
};
