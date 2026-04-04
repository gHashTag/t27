import { Router } from "express";

import {
  createSession,
  deleteSession,
  getSession,
  listSessions,
} from "../services/sessions.js";
import { asyncHandler } from "../utils/asyncHandler.js";
import {
  authTokenExpiresInSeconds,
  createSandboxToken,
} from "../utils/auth.js";
import { HttpError } from "../utils/errors.js";

const router = Router();

// GET /sessions – list all sessions
router.get(
  "/",
  asyncHandler(async (_req, res) => {
    const data = await listSessions();
    res.json({ data });
  }),
);

// GET /sessions/:id – get one session
router.get(
  "/:id",
  asyncHandler(async (req, res) => {
    const session = await getSession(req.params.id);
    res.json({ data: session });
  }),
);

// POST /sessions – create a new sandbox
router.post(
  "/",
  asyncHandler(async (req, res) => {
    const { name, taskDescription, repoUrl, branch } = req.body ?? {};

    const session = await createSession({
      name: typeof name === "string" ? name : undefined,
      taskDescription:
        typeof taskDescription === "string" ? taskDescription : undefined,
      repoUrl: typeof repoUrl === "string" ? repoUrl : undefined,
      branch: typeof branch === "string" ? branch : undefined,
    });

    res.status(201).json({ data: session });
  }),
);

// DELETE /sessions/:id – delete / cleanup a session
router.delete(
  "/:id",
  asyncHandler(async (req, res) => {
    const session = await deleteSession(req.params.id);
    res.json({ data: session });
  }),
);

// POST /sessions/:id/token – issue a short-lived sandbox access token
router.post(
  "/:id/token",
  asyncHandler(async (req, res) => {
    if (req.auth?.role !== "admin") {
      throw new HttpError(403, "Forbidden");
    }

    const session = await getSession(req.params.id);
    const token = createSandboxToken(session.name);

    res.json({
      data: {
        token,
        expiresIn: authTokenExpiresInSeconds,
        sessionName: session.name,
      },
    });
  }),
);

export default router;
