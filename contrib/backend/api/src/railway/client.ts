import { Agent, setGlobalDispatcher } from "undici";

import { config, getRailwayToken } from "../config.js";
import { HttpError } from "../utils/errors.js";
import { logger } from "../utils/logger.js";

type RailwayResponse<T> = {
  data?: T;
  errors?: Array<{ message?: string }>;
};

const railwayPool = new Agent({
  keepAliveTimeout: 60_000,
  keepAliveMaxTimeout: 120_000,
  connections: 10,
});

setGlobalDispatcher(railwayPool);

export const getPoolStats = () => ({
  maxConnections: 10,
  keepAlive: true,
});

/**
 * Execute a GraphQL request against the Railway API.
 *
 * Uses a persistent HTTP Agent with keep-alive for connection reuse,
 * reducing TCP handshake overhead on repeated requests.
 *
 * @param query        GraphQL query / mutation string
 * @param variables    Variables object
 * @param accountIndex Account index into the token pool.  Callers (e.g.
 *                     sessions service) are responsible for round-robin;
 *                     this function simply looks up the token.
 */
export const railwayRequest = async <T>(
  query: string,
  variables: Record<string, unknown>,
  accountIndex?: number,
): Promise<T> => {
  const idx = accountIndex ?? 0;
  const token = getRailwayToken(idx);

  const response = await fetch(config.railwayGraphqlUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ query, variables }),
  });

  if (!response.ok) {
    logger.error("Railway API HTTP error", {
      status: response.status,
      accountIndex: idx,
    });
    throw new HttpError(502, `Railway API error: ${response.status}`);
  }

  const payload = (await response.json()) as RailwayResponse<T>;

  if (payload.errors?.length) {
    const messages = payload.errors
      .map((e) => e.message ?? "Unknown error")
      .join("; ");
    logger.error("Railway API GraphQL error", {
      errors: messages,
      accountIndex: idx,
    });
    throw new HttpError(502, `Railway API error: ${messages}`);
  }

  if (!payload.data) {
    logger.error("Railway API missing data", { accountIndex: idx });
    throw new HttpError(502, "Railway API error: missing response data");
  }

  return payload.data;
};
