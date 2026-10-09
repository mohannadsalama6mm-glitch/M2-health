import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import * as api from "../lib/tauri/auth";
import { isDesktopRuntime, setNativeSession } from "../lib/tauri/client";
import { recoveryStatus } from "../lib/tauri/recovery";
import { userHasPermission, type Permission } from "../lib/permissions";

type Mode = "loading" | "bypass" | "setup" | "login" | "authenticated" | "error";
interface AuthContextValue {
  mode: Mode;
  enabled: boolean;
  user: api.User | null;
  token: string;
  error: string;
  hasPermission(permission: Permission): boolean;
  signIn(input: { username: string; password: string }): Promise<void>;
  setup(input: { username: string; displayName: string; password: string }): Promise<void>;
  signOut(): Promise<void>;
}
const Context = createContext<AuthContextValue | null>(null);
const message = (error: unknown) => error instanceof Error ? error.message : "Authentication could not be completed.";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [mode, setMode] = useState<Mode>(isDesktopRuntime() ? "loading" : "bypass");
  const [session, setSession] = useState<api.LoginResult | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!isDesktopRuntime()) return;
    let cancelled = false;
    recoveryStatus().then((flags) => {
      if (cancelled) return;
      if (flags.authEnabled !== true) { setMode("bypass"); return; }
      return api.authStatus().then((status) => { if (!cancelled) setMode(status.initialized ? "login" : "setup"); });
    }).catch((e) => { if (!cancelled) { setError(message(e)); setMode("error"); } });
    return () => { cancelled = true; };
  }, []);
  useEffect(() => {
    const invalidated = () => {
      setNativeSession("", "");
      setSession(null);
      setError("Your session ended. Sign in again.");
      setMode("login");
    };
    window.addEventListener("m2-auth-invalidated", invalidated);
    return () => window.removeEventListener("m2-auth-invalidated", invalidated);
  }, []);
  const accept = useCallback((result: api.LoginResult) => { setNativeSession(result.token, result.user.id); setSession(result); setError(""); setMode("authenticated"); }, []);
  const signIn = useCallback(async (input: { username: string; password: string }) => {
    try { accept(await api.login(input)); } catch (e) { setError(message(e)); throw e; }
  }, [accept]);
  const setup = useCallback(async (input: { username: string; displayName: string; password: string }) => {
    try { accept(await api.bootstrapAdmin(input)); } catch (e) { setError(message(e)); throw e; }
  }, [accept]);
  const signOut = useCallback(async () => {
    const token = session?.token;
    if (token) await api.logout(token).catch(() => undefined);
    setNativeSession("", "");
    setSession(null); setError(""); setMode("login");
  }, [session]);
  const value = useMemo(() => ({
    mode,
    enabled: mode !== "bypass",
    user: session?.user ?? null,
    token: session?.token ?? "",
    error,
    hasPermission: (permission: Permission) => mode === "bypass" || userHasPermission(session?.user ?? null, permission),
    signIn,
    setup,
    signOut,
  }), [mode, session, error, signIn, setup, signOut]);
  return <Context.Provider value={value}>{children}</Context.Provider>;
}
export function useAuth() {
  const value = useContext(Context);
  if (!value) throw new Error("AuthProvider is required");
  return value;
}
