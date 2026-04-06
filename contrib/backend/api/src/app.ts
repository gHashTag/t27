import cors from "cors";
import express, { type NextFunction, type Request, type Response } from "express";

import { config } from "./config.js";
import { authTokenMiddleware } from "./middleware/authToken.js";
import authRouter from "./routes/auth.js";
import healthRouter from "./routes/health.js";
import sessionsRouter from "./routes/sessions.js";
import { HttpError } from "./utils/errors.js";

const app = express();

// ─────────────────────────────────────────────────────────────
// Global middleware
// ─────────────────────────────────────────────────────────────

app.use(
  cors({
    origin: config.webOrigin ?? true,
  }),
);

app.use(express.json({ limit: "1mb" }));

// ─────────────────────────────────────────────────────────────
// Public routes (no auth required)
// ─────────────────────────────────────────────────────────────

app.use("/health", healthRouter);
app.use("/auth", authRouter);

// ─────────────────────────────────────────────────────────────
// Auth middleware (all routes below require a valid JWT)
// ─────────────────────────────────────────────────────────────

app.use(authTokenMiddleware);

// ─────────────────────────────────────────────────────────────
// Protected routes
// ─────────────────────────────────────────────────────────────

app.use("/sessions", sessionsRouter);

// ─────────────────────────────────────────────────────────────
// 404 catch-all
// ─────────────────────────────────────────────────────────────

app.use((_req, _res, next) => {
  next(new HttpError(404, "Not found"));
});

// ─────────────────────────────────────────────────────────────
// Global error handler
// ─────────────────────────────────────────────────────────────

// eslint-disable-next-line @typescript-eslint/no-unused-vars
app.use((error: Error, _req: Request, res: Response, _next: NextFunction) => {
  if (error instanceof HttpError) {
    res.status(error.status).json({ error: error.message });
    return;
  }

  console.error("Unhandled error", error);
  res.status(500).json({ error: "Internal server error" });
});

export default app;
