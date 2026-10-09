import { invoke, isTauri } from "@tauri-apps/api/core";
export type AppErrorCode =
  | "unauthorized"
  | "forbidden"
  | "database"
  | "migration"
  | "validation"
  | "notFound"
  | "conflict"
  | "io"
  | "internal"
  | "unavailable";
export class NativeError extends Error {
  constructor(
    public readonly code: AppErrorCode,
    message: string,
  ) {
    super(message);
    this.name = "NativeError";
  }
}
let nativeSession = { token: "", userId: "", branchId: "" };
export function setNativeSession(token: string, userId: string) {
  nativeSession = { ...nativeSession, token, userId };
}
export function setNativeBranch(branchId: string) {
  nativeSession = { ...nativeSession, branchId };
}
const actorKeys = new Set([
  "createdBy", "created_by", "postedBy", "posted_by", "voidedBy",
  "voided_by", "performedBy", "performed_by", "assignedBy", "assigned_by", "user",
]);
function secureArgs(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(secureArgs);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
    key,
    actorKeys.has(key) && nativeSession.userId ? nativeSession.userId : secureArgs(entry),
  ]));
}
export function isDesktopRuntime(): boolean {
  return isTauri();
}
function normalizeError(error: unknown): NativeError {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    const codes: AppErrorCode[] = [
      "unauthorized",
      "forbidden",
      "database",
      "migration",
      "validation",
      "notFound",
      "conflict",
      "io",
      "internal",
    ];
    if (codes.includes(error.code as AppErrorCode))
      return new NativeError(error.code as AppErrorCode, error.message);
  }
  return new NativeError(
    "internal",
    "The desktop service could not complete this request. Please retry.",
  );
}
/** The only invoke boundary in the frontend. No browser API fallback or fake persistence. */
export async function nativeRequest<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isDesktopRuntime())
    throw new NativeError(
      "unavailable",
      "Local database access requires the M² Health desktop app.",
    );
  try {
    const secured = secureArgs(args ?? {}) as Record<string, unknown>;
    delete secured.sessionToken;
    if (nativeSession.token) secured.sessionToken = nativeSession.token;
    if (nativeSession.branchId && !("authorizationBranchId" in secured))
      secured.authorizationBranchId = nativeSession.branchId;
    return await invoke<T>(command, secured);
  } catch (error: unknown) {
    const normalized = normalizeError(error);
    if (normalized.code === "unauthorized")
      window.dispatchEvent(new CustomEvent("m2-auth-invalidated"));
    throw normalized;
  }
}
