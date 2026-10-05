# NOW -- t27c serve points at Railway's public GraphQL API (2026-10-04)

## bootstrap/src/railway.rs -- the GraphQL host answers again

- Line 8 pointed `t27c serve` (session create: `create_railway_service`,
  `set_service_variables`, `check_service_health`) at a Railway GraphQL host
  that does not answer:

  ```
  const RAILWAY_GRAPHQL: &str = "https://backpack.railway.com/graphql";
  ```

  Measured 2026-10-04 with `curl -d '{"query":"{__typename}"}'`:

  | URL | HTTP | body |
  |---|---|---|
  | `https://backpack.railway.com/graphql` | 000 (no answer within 15 s) | -- |
  | `https://backboard.railway.com/graphql/v2` | 200 | `{"data":{"__typename":"Query"}}` |

- `RAILWAY_GRAPHQL` is now `https://backboard.railway.com/graphql/v2`,
  Railway's public API. Every session `t27c serve` tried to create before
  this failed before it reached Railway; now the GraphQL call reaches
  Railway and returns an answer (success or a Railway error), not a
  connection timeout.
- A unit test (`railway_graphql_targets_public_api`) asserts the constant's
  full URL and its host and path separately, so the old host cannot come
  back unnoticed.
- No Railway variable or token was set or read for this fix; it needs no
  credential. CI runs the cargo tests.
