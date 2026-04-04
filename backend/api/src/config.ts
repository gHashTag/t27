const requiredEnv = (name: string): string => {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
};

const parseSandboxLocalMap = (value?: string): Record<string, string> => {
  if (!value) {
    return {};
  }

  return value.split(",").reduce<Record<string, string>>((acc, entry) => {
    const [key, target] = entry.split("=");
    if (!key || !target) {
      return acc;
    }
    acc[key.trim()] = target.trim();
    return acc;
  }, {});
};

/**
 * Collect Railway API tokens for multiple accounts.
 *
 * Tokens can be provided as:
 *   RAILWAY_API_TOKEN          – single primary token
 *   RAILWAY_API_TOKEN_0        – first account in the pool
 *   RAILWAY_API_TOKEN_1        – second account …
 *   RAILWAY_API_TOKEN_2        – third account …
 *
 * When numbered variants are present they take precedence over the plain
 * RAILWAY_API_TOKEN.  The plain token is appended as a fallback so a
 * deployment that only sets RAILWAY_API_TOKEN still works.
 */
const buildRailwayTokenPool = (): string[] => {
  const numbered: string[] = [];
  let i = 0;
  while (true) {
    const token = process.env[`RAILWAY_API_TOKEN_${i}`];
    if (!token) break;
    numbered.push(token);
    i++;
  }

  if (numbered.length > 0) {
    return numbered;
  }

  // Fall back to the single required token
  return [requiredEnv("RAILWAY_API_TOKEN")];
};

export const config = {
  nodeEnv: process.env.NODE_ENV ?? "development",
  port: Number(process.env.PORT ?? 3000),

  // Database
  databaseUrl: process.env.DATABASE_URL,

  // Railway – single-account helpers (first token in pool)
  railwayApiToken: process.env.RAILWAY_API_TOKEN ?? "",
  railwayApiTokenPool: buildRailwayTokenPool(),

  railwayProjectId: requiredEnv("RAILWAY_PROJECT_ID"),
  railwayEnvironmentId: requiredEnv("RAILWAY_ENVIRONMENT_ID"),
  railwayServiceImage:
    process.env.RAILWAY_SERVICE_IMAGE ??
    "ghcr.io/ghashtagg/t27-sandbox:latest",
  railwayGraphqlUrl:
    process.env.RAILWAY_GRAPHQL_URL ??
    "https://backboard.railway.app/graphql/v2",

  // Auth
  adminPassword: process.env.ADMIN_PASSWORD ?? "",
  authTokenSecret: process.env.AUTH_TOKEN_SECRET ?? "dev-secret",

  // CORS / host routing
  webOrigin: process.env.WEB_ORIGIN,
  apiDirectHost: process.env.API_DIRECT_HOST ?? "localhost",
  apiProxyHost: process.env.API_PROXY_HOST ?? "proxy.localhost",

  // Sandbox networking
  sandboxInternalDomain:
    process.env.SANDBOX_INTERNAL_DOMAIN ?? "railway.internal",
  sandboxPort: Number(process.env.SANDBOX_PORT ?? 8080),

  // Local development overrides — localMode MUST be explicitly enabled.
  // Setting SANDBOX_LOCAL_BASE_URL alone is NOT enough; this prevents
  // accidental activation in staging environments.
  sandboxLocalBaseUrl: process.env.SANDBOX_LOCAL_BASE_URL,
  sandboxLocalMap: parseSandboxLocalMap(process.env.SANDBOX_LOCAL_MAP),
  localMode: process.env.LOCAL_MODE === "true",

  // Session limits
  maxSessions: Number(process.env.MAX_SESSIONS ?? 100),

  // Sandbox pass-through env vars
  githubToken: process.env.GH_TOKEN,
  sandboxRepoUrl:
    process.env.SANDBOX_REPO_URL ?? "https://github.com/gHashTag/t27.git",
  anthropicApiKey: process.env.ANTHROPIC_API_KEY,
  openaiApiKey: process.env.OPENAI_API_KEY,
};

/** Return the Railway API token for the given account index (round-robin). */
export const getRailwayToken = (accountIndex?: number): string => {
  const pool = config.railwayApiTokenPool;
  if (pool.length === 0) {
    throw new Error("No Railway API tokens configured");
  }
  const idx = (accountIndex ?? 0) % pool.length;
  return pool[idx];
};

/** Number of Railway accounts available in the token pool. */
export const railwayAccountCount = (): number =>
  config.railwayApiTokenPool.length;
