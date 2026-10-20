import { useState, type FormEvent } from "react";
import { Link, Navigate, useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";

const errorMessage = (error: unknown) => error instanceof Error ? error.message : "The request could not be completed.";

export function AuthPage({ mode }: { mode: "login" | "register" }) {
  const auth = useAuth();
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
    setBusy(true); setError(""); setNotice("");
    try {
      if (mode === "register") {
        const result = await auth.register(email, password, name);
        if (result.confirmationRequired) setNotice("Check your email to verify the account, then sign in.");
        else navigate("/portal");
      } else {
        await auth.login(email, password);
        const from = (location.state as { from?: string } | null)?.from || "/portal";
        navigate(from, { replace: true });
      }
    } catch (reason) { setError(errorMessage(reason)); }
    finally { setBusy(false); }
  };

  return <section className="auth-page"><div className="auth-panel"><span className="eyebrow">Customer portal</span><h1>{mode === "register" ? "Create your Early Access account" : "Welcome back"}</h1><p>{mode === "register" ? "Creating an account does not start the seven-day trial. You decide when the clock begins." : "Manage your license, devices, downloads, and activation requests."}</p>{!auth.configured && <div className="inline-alert error">Account services are not configured for this deployment. Public pages and downloads remain available.</div>}{error && <div className="inline-alert error" role="alert">{error}</div>}{notice && <div className="inline-alert success" role="status">{notice}</div>}<form onSubmit={submit}>{mode === "register" && <label>Full name<input value={name} onChange={(event) => setName(event.target.value)} autoComplete="name" required minLength={2} /></label>}<label>Email address<input type="email" value={email} onChange={(event) => setEmail(event.target.value)} autoComplete="email" required /></label><label>Password<input type="password" value={password} onChange={(event) => setPassword(event.target.value)} autoComplete={mode === "register" ? "new-password" : "current-password"} required minLength={8} /></label><button className="button button-primary button-full" disabled={busy || !auth.configured}>{busy ? "Working…" : mode === "register" ? "Create account" : "Sign in"}</button></form><div className="auth-links">{mode === "login" ? <><Link to="/forgot-password">Forgot password?</Link><span>New to M² Health? <Link to="/register">Create account</Link></span></> : <span>Already registered? <Link to="/login">Sign in</Link></span>}</div></div><aside className="auth-aside"><span className="eyebrow">Seven days, fully deliberate</span><h2>Trial time begins only when you start it.</h2><ul><li>Register without consuming the trial</li><li>Activate one eligible desktop device</li><li>Keep local data if access expires</li><li>Request manual activation without payment</li></ul></aside></section>;
}

export function ForgotPasswordPage() {
  const auth = useAuth(); const [email, setEmail] = useState(""); const [message, setMessage] = useState(""); const [error, setError] = useState("");
  const submit = async (event: FormEvent) => { event.preventDefault(); setError(""); try { await auth.requestPasswordReset(email); setMessage("If that address is registered, a reset link is on its way."); } catch (reason) { setError(errorMessage(reason)); } };
  return <section className="auth-page single"><div className="auth-panel"><span className="eyebrow">Account recovery</span><h1>Reset your password</h1><p>We will send a secure Supabase Auth recovery link.</p>{error && <div className="inline-alert error">{error}</div>}{message && <div className="inline-alert success">{message}</div>}<form onSubmit={submit}><label>Email address<input type="email" value={email} onChange={(event) => setEmail(event.target.value)} required /></label><button className="button button-primary button-full" disabled={!auth.configured}>Send reset link</button></form><Link to="/login">Back to sign in</Link></div></section>;
}

export function ResetPasswordPage() {
  const auth = useAuth(); const [password, setPassword] = useState(""); const [message, setMessage] = useState(""); const [error, setError] = useState("");
  const submit = async (event: FormEvent) => { event.preventDefault(); setError(""); try { await auth.updatePassword(password); setMessage("Password updated. You can return to your account."); } catch (reason) { setError(errorMessage(reason)); } };
  return <section className="auth-page single"><div className="auth-panel"><span className="eyebrow">Secure reset</span><h1>Choose a new password</h1>{error && <div className="inline-alert error">{error}</div>}{message && <div className="inline-alert success">{message}</div>}<form onSubmit={submit}><label>New password<input type="password" value={password} onChange={(event) => setPassword(event.target.value)} minLength={8} required /></label><button className="button button-primary button-full">Update password</button></form></div></section>;
}
