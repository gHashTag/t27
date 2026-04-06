import type { NextFunction, Request, Response } from "express";

import { getAuthTokenPayload } from "../utils/auth.js";

/**
 * Express middleware that validates the JWT from the Authorization header,
 * query string (?token=…), or sandbox_token cookie, then attaches the
 * decoded payload to req.auth.
 *
 * If no valid token is present the request is rejected with 401.
 */
export const authTokenMiddleware = (
  req: Request,
  _res: Response,
  next: NextFunction,
): void => {
  try {
    const payload = getAuthTokenPayload(req);
    req.auth = payload;
    next();
  } catch (error) {
    next(error);
  }
};
