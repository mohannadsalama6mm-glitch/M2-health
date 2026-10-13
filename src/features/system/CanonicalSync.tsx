import { useEffect, useState, type FormEvent } from "react";
import { Alert, Button, Card, DetailList, FeaturePage, Input, Spinner, SummaryStrip } from "../../design-system";
import { cloudSignIn, cloudSignOut, getCloudFoundationStatus, getCloudSessionStatus, listSyncConflicts, resolveSyncConflict, runCloudSyncCycle, type CloudFoundationStatus, type CloudSessionStatus, type SyncConflict } from "../../lib/tauri/cloud";

export function CanonicalSync() {
  const [status, setStatus] = useState<CloudFoundationStatus | null>(null);
  const [session, setSession] = useState<CloudSessionStatus | null>(null);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [conflicts, setConflicts] = useState<SyncConflict[]>([]);
  const [resolutionNote, setResolutionNote] = useState("");
  const refresh = async () => {
    const [foundation, cloudSession, conflictRows] = await Promise.all([getCloudFoundationStatus(), getCloudSessionStatus(), listSyncConflicts()]);
    setStatus(foundation); setSession(cloudSession); setConflicts(conflictRows);
  };
  useEffect(() => {
    let active = true;
    Promise.all([getCloudFoundationStatus(), getCloudSessionStatus(), listSyncConflicts()])
      .then(([foundation, cloudSession, conflictRows]) => { if (active) { setStatus(foundation); setSession(cloudSession); setConflicts(conflictRows); } })
      .catch((reason: unknown) => { if (active) setError(reason instanceof Error ? reason.message : "Cloud status is unavailable."); });
    return () => { active = false; };
  }, []);
  const run = async (operation: () => Promise<void>) => {
    setBusy(true); setError(""); setNotice("");
    try { await operation(); await refresh(); }
    catch (reason: unknown) { setError(reason instanceof Error ? reason.message : "The cloud request failed."); }
    finally { setBusy(false); }
  };
  const signIn = (event: FormEvent) => {
    event.preventDefault();
    void run(async () => { await cloudSignIn({ email, password }); setPassword(""); setNotice("Cloud authentication succeeded. Enrollment and rollout gates are still enforced."); });
  };
  return <FeaturePage title="Sync / cloud status" description="Authenticated cloud enrollment and offline synchronization readiness for this device.">
    {error && <Alert title="Cloud action failed" tone="danger">{error}</Alert>}
    {notice && <Alert title="Cloud status updated" tone="success">{notice}</Alert>}
    {!status && !error && <Card><Spinner /> Loading local cloud foundation…</Card>}
    {status && <>
      <Alert title={status.syncAvailable ? "Connected mode" : "Local-only mode"} tone={status.syncAvailable ? "success" : "info"}>{status.endpointConfigured ? "The cloud endpoint is configured locally." : "Cloud endpoint setup is required."} Synchronization remains unavailable until authentication, enrollment, identity linkage, and both rollout gates are complete.</Alert>
      <SummaryStrip items={[{ label: "Operating mode", value: status.syncAvailable ? "Connected" : "Local only" }, { label: "Cloud setup", value: status.setupRequired ? "Required" : "Configured" }, { label: "Pending events", value: status.outboxPending }, { label: "Quarantined", value: status.openConflicts }]} />
      <div className="equal-columns">
        <Card><h2>Local device</h2><DetailList items={[{ label: "Device ID", value: status.deviceId }, { label: "Enrollment", value: status.deviceStatus === "LOCAL_ONLY" ? "Not enrolled" : status.deviceStatus }, { label: "Transport", value: status.transportEnabled ? "Enabled" : "Disabled" }, { label: "Retry queue", value: status.retryScheduled }, { label: "Next retry", value: status.nextRetryAt ?? "None scheduled" }]} /></Card>
        <Card><h2>Cloud organization</h2><DetailList items={[{ label: "Organization", value: status.organizationName ?? "Not configured" }, { label: "Cloud identity", value: status.cloudIdentityLinked ? (status.cloudSubjectId ?? "Linked") : "Not linked" }, { label: "Applied cloud events", value: status.inboxApplied }, { label: "Rejected events", value: status.inboxRejected }, { label: "Last successful sync", value: status.lastSuccessfulSyncAt ?? "Never" }, { label: "Last attempt", value: status.lastSyncAt ?? "Never" }, { label: "Last outcome", value: status.lastSyncOutcome ?? "Not run" }]} /></Card>
      </div>
      <Card><h2>Cloud authentication</h2>{session?.authenticated ? <div className="identity-password"><p>Authenticated cloud subject: <code>{session.subjectId}</code></p><Button disabled={busy} onClick={() => void run(async () => { await cloudSignOut(); setNotice("Cloud session cleared from memory."); })}>Sign out</Button></div> : <form className="identity-form" onSubmit={signIn}><Input label="Cloud email" type="email" autoComplete="username" required value={email} onChange={(event) => setEmail(event.target.value)} /><Input label="Cloud password" type="password" autoComplete="current-password" required value={password} onChange={(event) => setPassword(event.target.value)} /><Button variant="primary" type="submit" loading={busy}>Sign in to cloud</Button></form>}<p className="muted">Credentials are passed once to Rust and are never stored by the frontend.</p></Card>
      <Card><h2>Synchronization control</h2><p>{status.syncAvailable ? "All runtime gates report ready." : "Synchronization is fail-closed. Complete enrollment and the separately approved activation procedure first."}</p><Button variant="primary" disabled={!status.syncAvailable || busy} loading={busy} onClick={() => void run(async () => { const result = await runCloudSyncCycle(); setNotice(`Synchronization completed: ${result.pushed} pushed, ${result.applied} applied, ${result.conflicts} quarantined.`); })}>Synchronize now</Button></Card>
      <Card><h2>Conflict resolution</h2>{conflicts.length === 0 ? <p className="muted">No synchronization conflicts are recorded.</p> : <><Input label="Resolution note" required value={resolutionNote} onChange={(event) => setResolutionNote(event.target.value)} />{conflicts.map((conflict) => <div className="identity-password" key={conflict.id}><p><strong>{conflict.reasonCode}</strong> · {conflict.entityType} · {conflict.entityId}<br /><span className="muted">{conflict.detail}</span></p>{conflict.status === "OPEN" && <div><Button disabled={busy || !resolutionNote.trim()} onClick={() => void run(async () => { await resolveSyncConflict(conflict.id, "RETRY", resolutionNote); setResolutionNote(""); setNotice("Conflict queued for an authorized retry."); })}>Retry safely</Button> <Button variant="danger" disabled={busy || !resolutionNote.trim()} onClick={() => void run(async () => { await resolveSyncConflict(conflict.id, "DISMISS", resolutionNote); setResolutionNote(""); setNotice("Conflict dismissed with an audit note; no domain write was applied."); })}>Dismiss</Button></div>}</div>)}</>}</Card>
      <Alert title="Phase 8C rollout gate" tone="warning">Passing tests does not enable synchronization. Configuration and database gates require a separate approved activation procedure.</Alert>
    </>}
  </FeaturePage>;
}
