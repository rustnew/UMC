import { createMiddleware } from "@tanstack/react-start";

const TOKEN_KEY = "umc_access_token";

// Attaches the UMC JWT Bearer token to all server function RPCs.
export const attachUmcAuth = createMiddleware({ type: "function" }).client(
  async ({ next }) => {
    const token =
      typeof localStorage !== "undefined"
        ? localStorage.getItem(TOKEN_KEY)
        : null;
    return next({
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
  }
);
