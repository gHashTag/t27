import type { AuthTokenPayload } from "../utils/auth.js";

declare global {
  namespace Express {
    interface Request {
      /** Set by authTokenMiddleware after successful JWT verification */
      auth?: AuthTokenPayload;
    }
  }
}
