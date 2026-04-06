import { Router } from "express";

import { config } from "../config.js";
import { asyncHandler } from "../utils/asyncHandler.js";
import {
  authTokenExpiresInSeconds,
  createAdminToken,
} from "../utils/auth.js";
import { HttpError } from "../utils/errors.js";
import { createRateLimiter } from "../middleware/rateLimit.js";

const router = Router();

// 10 attempts per 15-minute window per IP — protects against brute-force.
const loginLimiter = createRateLimiter({ windowMs: 15 * 60 * 1000, maxAttempts: 10 });

/**
 * POST /auth/login
 *
 * Body: { password: string }
 * Returns: { token: string, expiresIn: number }
 */
router.post(
  "/login",
  loginLimiter,
  asyncHandler(async (req, res) => {
    const { password } = req.body ?? {};

    if (typeof password !== "string" || password !== config.adminPassword) {
      throw new HttpError(401, "Invalid credentials");
    }

    const token = createAdminToken();
    res.json({ token, expiresIn: authTokenExpiresInSeconds });
  }),
);

export default router;
