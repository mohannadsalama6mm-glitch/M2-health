import { nativeRequest } from "./client";

export interface CloudFoundationStatus {
  mode: "LOCAL_ONLY";
  configured: boolean;
  endpointConfigured: boolean;
  deviceId: string;
  deviceStatus: "LOCAL_ONLY" | "PENDING" | "ENROLLED" | "REVOKED";
  organizationId: string | null;
  organizationName: string | null;
  cloudIdentityLinked: boolean;
  cloudSubjectId: string | null;
  transportEnabled: boolean;
  syncAvailable: boolean;
  setupRequired: boolean;
  outboxPending: number;
  inboxApplied: number;
  inboxRejected: number;
  openConflicts: number;
  lastSyncAt: string | null;
  lastSuccessfulSyncAt: string | null;
  lastSyncOutcome: string | null;
  lastAcknowledgedAt: string | null;
  retryScheduled: number;
  nextRetryAt: string | null;
}

export function getCloudFoundationStatus() {
  return nativeRequest<CloudFoundationStatus>("get_cloud_foundation_status");
}

export interface CloudSessionStatus {
  authenticated: boolean;
  subjectId: string | null;
  expiresAt: number | null;
}

export interface CloudLoginInput {
  email: string;
  password: string;
}

export interface SyncCycleResult {
  pushed: number;
  pulled: number;
  applied: number;
  conflicts: number;
  checkpoint: number;
}

export function getCloudSessionStatus() {
  return nativeRequest<CloudSessionStatus>("get_cloud_session_status");
}

// Credentials cross the IPC boundary once and are retained only by the Rust
// process for the in-memory Supabase session. Callers must never persist them.
export function cloudSignIn(input: CloudLoginInput) {
  return nativeRequest<CloudSessionStatus>("cloud_sign_in", { input });
}

export function cloudSignOut() {
  return nativeRequest<void>("cloud_sign_out");
}

export function runCloudSyncCycle() {
  return nativeRequest<SyncCycleResult>("run_cloud_sync_cycle");
}

export interface SyncConflict {
  id: string;
  eventId: string;
  entityType: string;
  entityId: string;
  reasonCode: string;
  detail: string;
  status: "OPEN" | "RESOLVED" | "DISMISSED";
  createdAt: string;
  resolvedAt: string | null;
  resolutionAction: "RETRIED" | "DISMISSED" | null;
  resolvedByUserId: string | null;
  resolutionNote: string | null;
}

export function listSyncConflicts() {
  return nativeRequest<SyncConflict[]>("list_sync_conflicts");
}

export function resolveSyncConflict(conflictId: string, action: "RETRY" | "DISMISS", note: string) {
  return nativeRequest<void>("resolve_sync_conflict", { input: { conflictId, action, note } });
}
