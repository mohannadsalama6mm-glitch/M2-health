import { useState, type FormEvent } from "react";
import { Link, Navigate, useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { useLocalization } from "../localization/LocalizationContext";

const errorMessage = (error: unknown) => error instanceof Error ? error.message : "The request could not be completed.";

export function AuthPage({ mode }: { mode: "login" | "register" }) {
  const auth = useAuth();
  const { t } = useLocalization();
  const navigate = useNavigate();
  const location = useLocation();
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");
  if (auth.user) return <Navigate to="/portal" replace />;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      if (mode === "register") {
        const result = await auth.register(email, password, name);
        if (result.confirmationRequired) setNotice(t("authWeb.checkEmail"));
        else navigate("/portal");
      } else {
        await auth.login(email, password);
        const from = (location.state as { from?: string } | null)?.from || "/portal";
        navigate(from, { replace: true });
      }
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  };

  return <section className="auth-page"><div className="auth-panel"><span className="eyebrow">{t("authWeb.customerPortal")}</span><h1>{mode === "register" ? t("authWeb.createTitle") : t("authWeb.loginTitle")}</h1><p>{mode === "register" ? t("authWeb.createCopy") : t("authWeb.loginCopy")}</p>{!auth.configured && <div className="inline-alert error">{t("authWeb.notConfigured")}</div>}{error && <div className="inline-alert error" role="alert">{error}</div>}{notice && <div className="inline-alert success" role="status">{notice}</div>}<form onSubmit={submit}>{mode === "register" && <label>{t("authWeb.fullName")}<input value={name} onChange={(event) => setName(event.target.value)} autoComplete="name" required minLength={2} /></label>}<label>{t("authWeb.email")}<input type="email" value={email} onChange={(event) => setEmail(event.target.value)} autoComplete="email" required /></label><label>{t("authWeb.password")}<input type="password" value={password} onChange={(event) => setPassword(event.target.value)} autoComplete={mode === "register" ? "new-password" : "current-password"} required minLength={8} /></label><button className="button button-primary button-full" disabled={busy || !auth.configured}>{busy ? t("common.working") : mode === "register" ? t("authWeb.create") : t("authWeb.signIn")}</button></form><div className="auth-links">{mode === "login" ? <><Link to="/forgot-password">{t("authWeb.forgot")}</Link><span>{t("authWeb.new")} <Link to="/register">{t("authWeb.create")}</Link></span></> : <span>{t("authWeb.already")} <Link to="/login">{t("authWeb.signIn")}</Link></span>}</div></div><aside className="auth-aside"><span className="eyebrow">{t("authWeb.deliberate")}</span><h2>{t("authWeb.trialTitle")}</h2><ul><li>{t("authWeb.bulletRegister")}</li><li>{t("authWeb.bulletDevice")}</li><li>{t("authWeb.bulletData")}</li><li>{t("authWeb.bulletManual")}</li></ul></aside></section>;
}

export function ForgotPasswordPage() {
  const auth = useAuth();
  const { t } = useLocalization();
  const [email, setEmail] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError("");
    try {
      await auth.requestPasswordReset(email);
      setMessage(t("authWeb.resetPrivacy"));
    } catch (reason) {
      setError(errorMessage(reason));
    }
  };
  return <section className="auth-page single"><div className="auth-panel"><span className="eyebrow">{t("authWeb.recovery")}</span><h1>{t("authWeb.resetTitle")}</h1><p>{t("authWeb.resetCopy")}</p>{error && <div className="inline-alert error">{error}</div>}{message && <div className="inline-alert success">{message}</div>}<form onSubmit={submit}><label>{t("authWeb.email")}<input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required /></label><button className="button button-primary button-full" disabled={!auth.configured}>{t("authWeb.sendReset")}</button></form><Link to="/login">{t("authWeb.backSignIn")}</Link></div></section>;
}

export function ResetPasswordPage() {
  const auth = useAuth();
  const { t } = useLocalization();
  const [password, setPassword] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError("");
    try {
      await auth.updatePassword(password);
      setMessage(t("authWeb.updated"));
    } catch (reason) {
      setError(errorMessage(reason));
    }
  };
  return <section className="auth-page single"><div className="auth-panel"><span className="eyebrow">{t("authWeb.secureReset")}</span><h1>{t("authWeb.choosePassword")}</h1>{error && <div className="inline-alert error">{error}</div>}{message && <div className="inline-alert success">{message} <Link to="/login">{t("authWeb.backSignIn")}</Link></div>}<form onSubmit={submit}><label>{t("authWeb.newPassword")}<input type="password" value={password} onChange={(event) => setPassword(event.target.value)} minLength={8} required /></label><button className="button button-primary button-full">{t("authWeb.updatePassword")}</button></form></div></section>;
}
