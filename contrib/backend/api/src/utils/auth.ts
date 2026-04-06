import type { IncomingMessage } from "http";
import type { Request } from "express";
import jwt from "jsonwebtoken";

import { config } from "../config.js";
import { HttpError } from "./errors.js";

export type AuthTokenPayload = {
  sub: string;
  role: "admin" | "sandbox";
  sessionName?: string;
  iat?: number;
  exp?: number;
};

export const TOKEN_TTL_SECONDS = 60 * 60 * 24; // 24 hours
export const authTokenExpiresInSeconds = TOKEN_TTL_SECONDS;

// ─────────────────────────────────────────────────────────────
// Token extraction helpers
// ─────────────────────────────────────────────────────────────

const getAuthorizationHeader = (
  req: Pick<Request, "header"> | IncomingMessage,
): string | undefined => {
  if ("header" in req) {
    return req.header("authorization");
  }
  const header = req.headers?.authorization;
  return Array.isArray(header) ? header[0] : header;
};

const getTokenFromQuery = (
  req: Pick<Request, "url"> | IncomingMessage,
): string | undefined => {
  if (!req.url) return undefined;

  try {
    const parsed = new URL(req.url, "http://localhost");
    const token = parsed.searchParams.get("token");
    if (token) return token;
  } catch {
    // fall through to regex path
  }

  const match = req.url.match(/[?&]token=([^&]+)/);
  return match ? decodeURIComponent(match[1]) : undefined;
};

const getTokenFromCookie = (
  req: Pick<Request, "headers"> | IncomingMessage,
): string | undefined => {
  const cookieHeader = req.headers?.cookie;
  if (!cookieHeader) return undefined;

  const tokenCookie = cookieHeader
    .split(";")
    .map((c) => c.trim())
    .find((c) => c.startsWith("sandbox_token="));

  if (!tokenCookie) return undefined;
  const [, value] = tokenCookie.split("=");
  return value ? decodeURIComponent(value) : undefined;
};

// ─────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────

export const getAuthTokenPayload = (
  req: Pick<Request, "header" | "url" | "headers"> | IncomingMessage,
): AuthTokenPayload => {
  const header = getAuthorizationHeader(req);
  const token = header?.startsWith("Bearer ")
    ? header.slice(7)
    : (getTokenFromQuery(req) ?? getTokenFromCookie(req));

  if (!token) {
    throw new HttpError(401, "Unauthorized");
  }

  try {
    return jwt.verify(token, config.authTokenSecret) as AuthTokenPayload;
  } catch {
    throw new HttpError(401, "Unauthorized");
  }
};

export const createAdminToken = (): string =>
  jwt.sign({ sub: "admin", role: "admin" }, config.authTokenSecret, {
    expiresIn: TOKEN_TTL_SECONDS,
  });

export const createSandboxToken = (sessionName: string): string =>
  jwt.sign(
    { sub: sessionName, role: "sandbox", sessionName },
    config.authTokenSecret,
    { expiresIn: TOKEN_TTL_SECONDS },
  );
