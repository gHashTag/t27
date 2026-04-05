/**
 * E2E test for Railway sandbox lifecycle.
 *
 * This test makes REAL API calls to Railway — it creates a service,
 * sets env vars, waits for deployment, then tears it down.
 *
 * Run manually:  RAILWAY_E2E=true npx vitest run src/__tests__/e2e-railway.test.ts
 *
 * Required env vars:
 *   RAILWAY_API_TOKEN
 *   RAILWAY_PROJECT_ID
 *   RAILWAY_ENVIRONMENT_ID
 */
import { describe, it, expect, beforeAll, afterAll } from "vitest";

const RAILWAY_E2E = process.env.RAILWAY_E2E === "true";
const TOKEN = process.env.RAILWAY_API_TOKEN ?? "";
const PROJECT_ID = process.env.RAILWAY_PROJECT_ID ?? "";
const ENV_ID = process.env.RAILWAY_ENVIRONMENT_ID ?? "";
const GRAPHQL_URL =
  process.env.RAILWAY_GRAPHQL_URL ??
  "https://backboard.railway.com/graphql/v2";

const gql = async <T>(query: string, variables: Record<string, unknown> = {}): Promise<T> => {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${TOKEN}`,
    },
    body: JSON.stringify({ query, variables }),
  });
  const payload = (await res.json()) as { data?: T; errors?: { message: string }[] };
  if (payload.errors?.length) {
    throw new Error(`Railway API: ${payload.errors.map((e) => e.message).join("; ")}`);
  }
  return payload.data!;
};

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

describe.skipIf(!RAILWAY_E2E)("E2E Railway Sandbox Lifecycle", () => {
  let serviceId: string | null = null;
  const SERVICE_NAME = `t27-e2e-test-${Date.now()}`;

  afterAll(async () => {
    // Cleanup: always delete the service
    if (serviceId) {
      try {
        await gql("mutation($id: String!) { serviceDelete(id: $id) }", { id: serviceId });
        console.log(`[cleanup] Deleted service ${serviceId}`);
      } catch (e) {
        console.error(`[cleanup] Failed to delete service ${serviceId}:`, e);
      }
    }
  });

  it("creates a sandbox service", async () => {
    const data = await gql<{ serviceCreate: { id: string; name: string } }>(
      `mutation($input: ServiceCreateInput!) { serviceCreate(input: $input) { id name } }`,
      {
        input: {
          projectId: PROJECT_ID,
          environmentId: ENV_ID,
          name: SERVICE_NAME,
          source: { image: "node:22-bookworm-slim" },
        },
      },
    );

    serviceId = data.serviceCreate.id;
    expect(serviceId).toBeTruthy();
    expect(data.serviceCreate.name).toBe(SERVICE_NAME);
    console.log(`[e2e] Created service: ${serviceId} (${SERVICE_NAME})`);
  });

  it("sets environment variables", async () => {
    expect(serviceId).toBeTruthy();

    const data = await gql<{ variableCollectionUpsert: boolean }>(
      `mutation($input: VariableCollectionUpsertInput!) { variableCollectionUpsert(input: $input) }`,
      {
        input: {
          projectId: PROJECT_ID,
          environmentId: ENV_ID,
          serviceId,
          variables: {
            SANDBOX_REPO_URL: "https://github.com/gHashTag/t27.git",
            T27_E2E_TEST: "true",
          },
        },
      },
    );

    expect(data.variableCollectionUpsert).toBe(true);
    console.log("[e2e] Environment variables set");
  });

  it("deployment reaches SUCCESS within 60s", async () => {
    expect(serviceId).toBeTruthy();

    const deadline = Date.now() + 60_000;
    let status = "UNKNOWN";

    while (Date.now() < deadline) {
      const data = await gql<{
        service: { deployments: { edges: { node: { status: string } }[] } };
      }>(
        `{ service(id: "${serviceId}") { deployments(first: 1) { edges { node { status } } } } }`,
      );

      const edge = data.service.deployments.edges[0];
      status = edge?.node?.status ?? "NO_DEPLOYMENT";
      console.log(`[e2e] Deployment status: ${status}`);

      if (status === "SUCCESS") break;
      if (status === "FAILED" || status === "CRASHED") {
        throw new Error(`Deployment failed with status: ${status}`);
      }

      await sleep(5_000);
    }

    expect(status).toBe("SUCCESS");
  }, 70_000);

  it("deletes the sandbox service", async () => {
    expect(serviceId).toBeTruthy();

    const data = await gql<{ serviceDelete: boolean }>(
      "mutation($id: String!) { serviceDelete(id: $id) }",
      { id: serviceId },
    );

    expect(data.serviceDelete).toBe(true);
    console.log(`[e2e] Deleted service ${serviceId}`);
    serviceId = null; // prevent afterAll from double-deleting
  }, 30_000);
});
