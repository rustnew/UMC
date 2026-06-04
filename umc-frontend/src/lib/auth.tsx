import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import {
  auth as authApi,
  clearTokens,
  getAccessToken,
  setTokens,
  type UserPublic,
} from "@/integrations/api/client";

interface AuthCtx {
  user: UserPublic | null;
  loading: boolean;
  signOut: () => Promise<void>;
  /** Convenience: truthy when logged in */
  session: { user: UserPublic } | null;
}

const Ctx = createContext<AuthCtx | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserPublic | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // On mount, try to restore session from localStorage token
    const token = getAccessToken();
    if (!token) { setLoading(false); return; }

    authApi.me()
      .then(setUser)
      .catch(() => { clearTokens(); setUser(null); })
      .finally(() => setLoading(false));
  }, []);

  const signOut = async () => {
    try { await authApi.logout(); } catch {}
    clearTokens();
    setUser(null);
  };

  /** Called by login/register handlers after successful auth */
  const handleAuthSuccess = (u: UserPublic, access: string, refresh: string) => {
    setTokens(access, refresh);
    setUser(u);
  };

  return (
    <Ctx.Provider value={{
      user,
      loading,
      signOut,
      session: user ? { user } : null,
    }}>
      <AuthSuccessContext.Provider value={handleAuthSuccess}>
        {children}
      </AuthSuccessContext.Provider>
    </Ctx.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useAuth must be used inside AuthProvider");
  return ctx;
}

/** Used by login/signup pages to update auth state after success */
type AuthSuccessFn = (user: UserPublic, access: string, refresh: string) => void;
const AuthSuccessContext = createContext<AuthSuccessFn>(() => {});
export function useAuthSuccess() { return useContext(AuthSuccessContext); }