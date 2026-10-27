import { useState, type FormEvent, type ReactNode } from "react";
import { Alert, Button, Input, Logo } from "../../design-system";
import { useAuth } from "../../app/AuthContext";
import { useLocalization } from "../../localization/LocalizationContext";

export function AuthGate({ children }: { children: ReactNode }) {
  const auth = useAuth();
  const { t } = useLocalization();
  if (auth.mode === "bypass" || auth.mode === "authenticated") return children;
  if (auth.mode === "loading") return <AuthFrame><p role="status">{t("auth.preparing")}</p></AuthFrame>;
  if (auth.mode === "error") return <AuthFrame><Alert title={t("auth.unavailable")} tone="danger">{t("auth.unavailableCopy", { error: auth.error })}</Alert></AuthFrame>;
  return <Credentials setup={auth.mode === "setup"} />;
}

function AuthFrame({ children }: { children: ReactNode }) {
  return <main className="auth-screen"><section className="auth-card"><Logo variant="compact" />{children}</section></main>;
}

function Credentials({ setup }: { setup: boolean }) {
  const auth = useAuth();
  const { t } = useLocalization();
  const [username, setUsername] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [localError, setLocalError] = useState("");
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setLocalError("");
    if (setup && password !== confirm) {
      setLocalError(t("auth.passwordMismatch"));
      return;
    }
    setBusy(true);
    try {
      if (setup) await auth.setup({ username, displayName, password });
      else await auth.signIn({ username, password });
    } catch {
      // AuthContext owns the safe error text.
    } finally {
      setBusy(false);
    }
  };
  return <AuthFrame>
    <div>
      <h1>{setup ? t("auth.createAdmin") : t("auth.welcomeBack")}</h1>
      <p>{setup ? t("auth.createAdminCopy") : t("auth.signInCopy")}</p>
    </div>
    {(localError || auth.error) && <Alert title={t("auth.couldNotContinue")} tone="danger">{localError || auth.error}</Alert>}
    <form className="auth-form" onSubmit={submit}>
      {setup && <Input label={t("auth.displayName")} value={displayName} autoComplete="name" required onChange={(event) => setDisplayName(event.target.value)} />}
      <Input label={t("auth.username")} value={username} autoComplete="username" required onChange={(event) => setUsername(event.target.value)} />
      <Input label={t("auth.password")} type="password" value={password} autoComplete={setup ? "new-password" : "current-password"} required minLength={12} onChange={(event) => setPassword(event.target.value)} />
      {setup && <Input label={t("auth.confirmPassword")} type="password" value={confirm} autoComplete="new-password" required minLength={12} onChange={(event) => setConfirm(event.target.value)} />}
      {setup && <p className="field-help">{t("auth.passwordHelp")}</p>}
      <Button variant="primary" type="submit" disabled={busy}>{busy ? t("auth.wait") : setup ? t("auth.create") : t("auth.signIn")}</Button>
    </form>
    <p className="auth-local-note">{t("auth.localNote")}</p>
  </AuthFrame>;
}
