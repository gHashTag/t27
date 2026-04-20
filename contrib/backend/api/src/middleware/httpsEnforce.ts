import type { NextFunction, Request, Response } from "express";

import { config } from "../config.js";

export const httpsEnforceMiddleware = (
  req: Request,
  res: Response,
  next: NextFunction,
): void => {
  if (config.nodeEnv === "development") {
    next();
    return;
  }

  const proto =
    req.headers["x-forwarded-proto"] ??
    ((req.socket as unknown as { encrypted?: boolean }).encrypted ? "https" : "http");

  if (typeof proto === "string" && proto.includes("http") && !proto.includes("https")) {
    res.redirect(301, `https://${req.headers.host}${req.originalUrl}`);
    return;
  }

  next();
};
