import type { IncomingMessage } from "http";
import type { ServerResponse } from "http";
import type { Duplex } from "stream";
import httpProxy from "http-proxy";

import { config } from "../config.js";
import { getAuthTokenPayload } from "../utils/auth.js";
import { HttpError } from "../utils/errors.js";
import { resolveSandboxTarget } from "../utils/sandboxTarget.js";

// ─────────────────────────────────────────────────────────────
// Proxy instance
// ─────────────────────────────────────────────────────────────

const proxy = httpProxy.createProxyServer({
  ws: true,
  changeOrigin: true,
  xfwd: true,
});

/**
 * Strip origin headers from WebSocket upgrade requests so that
 * proxied sandbox servers don't reject cross-origin upgrades.
 */
proxy.on("proxyReqWs", (proxyReq) => {
  proxyReq.removeHeader("origin");
  proxyReq.removeHeader("sec-websocket-origin");
});

/**
 * After the first authenticated request, set a cookie so that subsequent
 * requests from the browser (e.g. WebSocket, XHR) are automatically
 * authenticated without needing to re-send a token in the query string.
 */
proxy.on("proxyRes", (proxyRes, req, res) => {
  const token = (req as IncomingMessage & { sandboxToken?: string })
    .sandboxToken;
  if (!token) return;

  const cookieValue = `sandbox_token=${encodeURIComponent(token)}; Path=/; HttpOnly; SameSite=Lax`;
  const existing = (res as ServerResponse).getHeader("Set-Cookie");

  if (!existing) {
    (res as ServerResponse).setHeader("Set-Cookie", cookieValue);
  } else if (Array.isArray(existing)) {
    (res as ServerResponse).setHeader("Set-Cookie", [...existing, cookieValue]);
  } else {
    (res as ServerResponse).setHeader("Set-Cookie", [
      existing.toString(),
      cookieValue,
    ]);
  }
});

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

const normalizeHost = (host?: string) => host?.toLowerCase().split(":")[0];

export const isProxyHost = (host?: string): boolean =>
  normalizeHost(host) === normalizeHost(config.apiProxyHost);

export const isDirectHost = (host?: string): boolean =>
  normalizeHost(host) === normalizeHost(config.apiDirectHost);

const getTokenFromUrl = (url?: string): string | undefined => {
  if (!url) return undefined;

  try {
    const parsed = new URL(url, "http://localhost");
    const token = parsed.searchParams.get("token");
    if (token) return token;
  } catch {
    // fall through
  }

  const match = url.match(/[?&]token=([^&]+)/);
  return match ? decodeURIComponent(match[1]) : undefined;
};

const getProxyTarget = (req: IncomingMessage): string => {
  const payload = getAuthTokenPayload(req);
  const sessionName = payload.sessionName ?? payload.sub;

  if (!sessionName) {
    throw new HttpError(401, "Unauthorized");
  }

  return resolveSandboxTarget(sessionName);
};

const sendJsonError = (res: ServerResponse, error: HttpError): void => {
  if (res.headersSent) return;
  res.writeHead(error.status, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: error.message }));
};

const rejectUpgrade = (socket: Duplex, error: HttpError): void => {
  socket.write(`HTTP/1.1 ${error.status} ${error.message}\r\n\r\n`);
  socket.destroy();
};

// ─────────────────────────────────────────────────────────────
// Exported handlers
// ─────────────────────────────────────────────────────────────

/**
 * Handle an incoming HTTP request that should be proxied to a sandbox.
 */
export const handleProxyRequest = (
  req: IncomingMessage,
  res: ServerResponse,
): void => {
  try {
    const tokenFromUrl = getTokenFromUrl(req.url);
    if (tokenFromUrl) {
      (req as IncomingMessage & { sandboxToken?: string }).sandboxToken =
        tokenFromUrl;
    }

    const target = getProxyTarget(req);
    proxy.web(req, res, { target }, (err) => {
      if (err) sendJsonError(res, new HttpError(502, "Sandbox proxy error"));
    });
  } catch (error) {
    if (error instanceof HttpError) {
      sendJsonError(res, error);
      return;
    }
    sendJsonError(res, new HttpError(500, "Internal server error"));
  }
};

/**
 * Handle a WebSocket upgrade that should be proxied to a sandbox.
 */
export const handleProxyUpgrade = (
  req: IncomingMessage,
  socket: Duplex,
  head: Buffer,
): void => {
  try {
    const target = getProxyTarget(req);
    proxy.ws(req, socket, head, { target }, (err) => {
      if (err) rejectUpgrade(socket, new HttpError(502, "Sandbox proxy error"));
    });
  } catch (error) {
    if (error instanceof HttpError) {
      rejectUpgrade(socket, error);
      return;
    }
    rejectUpgrade(socket, new HttpError(500, "Internal server error"));
  }
};
