import { config, getRailwayToken, railwayAccountCount } from "../config.js";
import { HttpError } from "../utils/errors.js";

type RailwayResponse<T> = {
  data?: T;
  errors?: Array<{ message?: string }>;
};

// Simple round-robin counter shared across requests within the process.
let _rrCounter = 0;

const nextAccountIndex = (): number => {
  const count = railwayAccountCount();
  const idx = _rrCounter % count;
  _rrCounter = (_rrCounter + 1) % count;
  return idx;
};

/**
 * Execute a GraphQL request against the Railway API.
 *
 * @param query        GraphQL query / mutation string
 * @param variables    Variables object
 * @param accountIndex Optional explicit account index.  When omitted the
 *                     next account in the round-robin pool is used.
 */
export const railwayRequest = async <T>(
  query: string,
  variables: Record<string, unknown>,
  accountIndex?: number,
): Promise<T> => {
  const idx = accountIndex ?? nextAccountIndex();
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
    throw new HttpError(502, `Railway API error: ${response.status}`);
  }

  const payload = (await response.json()) as RailwayResponse<T>;

  if (payload.errors?.length) {
    const messages = payload.errors
      .map((e) => e.message ?? "Unknown error")
      .join("; ");
    throw new HttpError(502, `Railway API error: ${messages}`);
  }

  if (!payload.data) {
    throw new HttpError(502, "Railway API error: missing response data");
  }

  return payload.data;
};
