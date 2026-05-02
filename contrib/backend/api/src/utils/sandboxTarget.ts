import { config } from "../config.js";

/**
 * Resolve the base URL for a sandbox service.
 *
 * In production this is the Railway internal network address.
 * In local development it can be overridden via SANDBOX_LOCAL_BASE_URL or
 * SANDBOX_LOCAL_MAP (per-session overrides).
 */
export const resolveSandboxTarget = (sessionName: string): string => {
  const localTarget =
    config.sandboxLocalMap[sessionName] ?? config.sandboxLocalBaseUrl;

  if (config.localMode && localTarget) {
    return localTarget;
  }

  return `http://${sessionName}.${config.sandboxInternalDomain}:${config.sandboxPort}`;
};

/** Full URL for the sandbox /healthz endpoint. */
export const resolveSandboxHealthUrl = (sessionName: string): string => {
  const target = resolveSandboxTarget(sessionName);
  return new URL("/healthz", target).toString();
};
