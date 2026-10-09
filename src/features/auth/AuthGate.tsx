import { useState, type FormEvent, type ReactNode } from "react";
import { Alert, Button, Input, Logo } from "../../design-system";
import { useAuth } from "../../app/AuthContext";

export function AuthGate({ children }: { children: ReactNode }) {
  const auth = useAuth();
  if (auth.mode === "bypass" || auth.mode === "authenticated") return children;
  if (auth.mode === "loading") return <AuthFrame><p role="status">Preparing secure local workspace…</p></AuthFrame>;
  if (auth.mode === "error") return <AuthFrame><Alert title="Authentication unavailable" tone="danger">Workflow availability could not be verified. {auth.error}</Alert></AuthFrame>;
  return <Credentials setup={auth.mode === "setup"} />;
}
function AuthFrame({ children }: { children: ReactNode }) {
  return <main className="auth-screen"><section className="auth-card"><Logo variant="compact" />{children}</section></main>;
}
function Credentials({ setup }: { setup: boolean }) {
  const auth = useAuth();
  const [username, setUsername] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [localError, setLocalError] = useState("");
  const submit = async (event: FormEvent) => {
    event.preventDefault(); setLocalError("");
    if (setup && password !== confirm) { setLocalError("Passwords do not match."); return; }
    setBusy(true);
    try { if (setup) await auth.setup({ username, displayName, password }); else await auth.signIn({ username, password }); }
    catch { /* AuthContext owns the safe error text. */ }
    finally { setBusy(false); }
  };
  return <AuthFrame>
    <div><h1>{setup ? "Create local administrator" : "Welcome back"}</h1><p>{setup ? "Secure this pharmacy workspace with its first administrator account." : "Sign in to the local M² Health workspace."}</p></div>
    {(localError || auth.error) && <Alert title="Could not continue" tone="danger">{localError || auth.error}</Alert>}
    <form className="auth-form" onSubmit={submit}>
      {setup && <Input label="Display name" value={displayName} autoComplete="name" required onChange={(e) => setDisplayName(e.target.value)} />}
      <Input label="Username" value={username} autoComplete="username" required onChange={(e) => setUsername(e.target.value)} />
      <Input label="Password" type="password" value={password} autoComplete={setup ? "new-password" : "current-password"} required minLength={12} onChange={(e) => setPassword(e.target.value)} />
      {setup && <Input label="Confirm password" type="password" value={confirm} autoComplete="new-password" required minLength={12} onChange={(e) => setConfirm(e.target.value)} />}
      {setup && <p className="field-help">Use 12–128 characters. The password cannot contain the username.</p>}
      <Button variant="primary" type="submit" disabled={busy}>{busy ? "Please wait…" : setup ? "Create administrator" : "Sign in"}</Button>
    </form>
    <p className="auth-local-note">Credentials and sessions stay on this device. Sessions end when the app restarts.</p>
  </AuthFrame>;
}
