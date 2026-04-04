import { eq, desc, not, inArray, count as dbCount } from "drizzle-orm";
import { randomUUID } from "crypto";

import { config, railwayAccountCount } from "../config.js";
import { db } from "../db/client.js";
import { sessions } from "../db/schema.js";
import { railwayRequest } from "../railway/client.js";
import {
  serviceCreateMutation,
  serviceDeleteMutation,
  variableCollectionUpsertMutation,
} from "../railway/mutations.js";
import { HttpError } from "../utils/errors.js";

// ─────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────

type ServiceCreateResponse = {
  serviceCreate: {
    id: string;
    name: string;
  };
};

type ServiceDeleteResponse = {
  serviceDelete: boolean;
};

type VariableCollectionUpsertResponse = {
  variableCollectionUpsert: boolean;
};

export type CreateSessionInput = {
  name?: string;
  taskDescription?: string;
  repoUrl?: string;
  branch?: string;
};

// ─────────────────────────────────────────────────────────────
// Round-robin account selection
// ─────────────────────────────────────────────────────────────

let _accountCounter = 0;

const nextAccountIndex = (): number => {
  const count = railwayAccountCount();
  const idx = _accountCounter % count;
  _accountCounter = (_accountCounter + 1) % count;
  return idx;
};

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

const generateSessionName = () =>
  `t27-sandbox-${Date.now()}`;

// ─────────────────────────────────────────────────────────────
// Service functions
// ─────────────────────────────────────────────────────────────

export const createSession = async ({
  name,
  taskDescription,
  repoUrl,
  branch,
}: CreateSessionInput) => {
  if (config.localMode) {
    throw new HttpError(403, "Session creation disabled in local mode");
  }

  // ── Enforce MAX_SESSIONS (spec invariant: session_count_bounded) ────
  const [{ value: activeCount }] = await db
    .select({ value: dbCount() })
    .from(sessions)
    .where(not(inArray(sessions.status, ["deleted", "failed"])));

  if (activeCount >= config.maxSessions) {
    throw new HttpError(
      429,
      `Session limit reached (${config.maxSessions}). Delete unused sessions first.`,
    );
  }

  const resolvedName = name?.trim() ? name.trim() : generateSessionName();
  const accountIndex = nextAccountIndex();

  // ── 1. Create the Railway service ──────────────────────────
  const data = await railwayRequest<ServiceCreateResponse>(
    serviceCreateMutation,
    {
      input: {
        projectId: config.railwayProjectId,
        environmentId: config.railwayEnvironmentId,
        name: resolvedName,
        source: {
          image: config.railwayServiceImage,
        },
      },
    },
    accountIndex,
  );

  if (!data.serviceCreate?.id) {
    throw new HttpError(502, "Railway API error: missing service id");
  }

  const railwayServiceId = data.serviceCreate.id;

  // ── 2. Set environment variables on the new service ────────
  const sandboxVars: Record<string, string> = {};

  const effectiveRepoUrl = repoUrl ?? config.sandboxRepoUrl;
  if (effectiveRepoUrl) sandboxVars["SANDBOX_REPO_URL"] = effectiveRepoUrl;
  if (branch) sandboxVars["SANDBOX_BRANCH"] = branch;
  if (taskDescription) sandboxVars["TASK_DESCRIPTION"] = taskDescription;
  if (config.githubToken) sandboxVars["GH_TOKEN"] = config.githubToken;
  if (config.anthropicApiKey)
    sandboxVars["ANTHROPIC_API_KEY"] = config.anthropicApiKey;
  if (config.openaiApiKey) sandboxVars["OPENAI_API_KEY"] = config.openaiApiKey;

  if (Object.keys(sandboxVars).length > 0) {
    await railwayRequest<VariableCollectionUpsertResponse>(
      variableCollectionUpsertMutation,
      {
        input: {
          projectId: config.railwayProjectId,
          environmentId: config.railwayEnvironmentId,
          serviceId: railwayServiceId,
          variables: sandboxVars,
        },
      },
      accountIndex,
    );
  }

  // ── 3. Persist to database ─────────────────────────────────
  const id = randomUUID();
  const now = new Date();

  const [session] = await db
    .insert(sessions)
    .values({
      id,
      name: resolvedName,
      status: "starting",
      railwayServiceId,
      railwayAccountIndex: accountIndex,
      taskDescription: taskDescription ?? null,
      repoUrl: repoUrl ?? config.sandboxRepoUrl ?? null,
      branch: branch ?? null,
      createdAt: now,
      updatedAt: now,
    })
    .returning();

  return session;
};

export const listSessions = async () =>
  db.select().from(sessions).orderBy(desc(sessions.createdAt));

export const getSession = async (id: string) => {
  const [session] = await db
    .select()
    .from(sessions)
    .where(eq(sessions.id, id));

  if (!session) {
    throw new HttpError(404, "Session not found");
  }

  return session;
};

export const deleteSession = async (id: string) => {
  const session = await getSession(id);

  if (session.status === "deleted") {
    return session;
  }

  // Mark as terminating first so health polling stops
  await db
    .update(sessions)
    .set({ status: "terminating", updatedAt: new Date() })
    .where(eq(sessions.id, id));

  if (session.railwayServiceId) {
    try {
      await railwayRequest<ServiceDeleteResponse>(
        serviceDeleteMutation,
        { id: session.railwayServiceId },
        session.railwayAccountIndex ?? undefined,
      );
    } catch (error) {
      // Restore previous status so callers can retry
      await db
        .update(sessions)
        .set({ status: session.status, updatedAt: new Date() })
        .where(eq(sessions.id, id));
      throw error;
    }
  }

  const [updated] = await db
    .update(sessions)
    .set({ status: "deleted", updatedAt: new Date() })
    .where(eq(sessions.id, id))
    .returning();

  return updated;
};
