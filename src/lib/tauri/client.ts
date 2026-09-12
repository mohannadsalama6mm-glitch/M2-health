import { invoke, isTauri } from "@tauri-apps/api/core";
export type AppErrorCode =
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
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    throw normalizeError(error);
  }
}
