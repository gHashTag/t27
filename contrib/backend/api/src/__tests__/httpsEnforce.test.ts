import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../config.js", () => ({
  config: { nodeEnv: "production" },
}));

import { httpsEnforceMiddleware } from "../middleware/httpsEnforce.js";
import type { NextFunction, Request, Response } from "express";

const mockRedirect = vi.fn();
const mockNext = vi.fn();

const baseReq = {
  headers: { host: "api.example.com" },
  originalUrl: "/sessions",
  socket: {},
} as unknown as Request;

const baseRes = {
  redirect: mockRedirect,
} as unknown as Response;

beforeEach(() => {
  mockRedirect.mockClear();
  mockNext.mockClear();
});

describe("httpsEnforceMiddleware", () => {
  it("redirects HTTP to HTTPS with 301", () => {
    const req = {
      ...baseReq,
      headers: { ...baseReq.headers, "x-forwarded-proto": "http" },
    } as unknown as Request;

    httpsEnforceMiddleware(req, baseRes, mockNext as NextFunction);

    expect(mockRedirect).toHaveBeenCalledWith(
      301,
      "https://api.example.com/sessions",
    );
    expect(mockNext).not.toHaveBeenCalled();
  });

  it("passes through HTTPS requests", () => {
    const req = {
      ...baseReq,
      headers: { ...baseReq.headers, "x-forwarded-proto": "https" },
    } as unknown as Request;

    httpsEnforceMiddleware(req, baseRes, mockNext as NextFunction);

    expect(mockRedirect).not.toHaveBeenCalled();
    expect(mockNext).toHaveBeenCalled();
  });

  it("passes through when socket is encrypted", () => {
    const req = {
      ...baseReq,
      headers: { host: "api.example.com" },
      socket: { encrypted: true },
    } as unknown as Request;

    httpsEnforceMiddleware(req, baseRes, mockNext as NextFunction);

    expect(mockRedirect).not.toHaveBeenCalled();
    expect(mockNext).toHaveBeenCalled();
  });
});
