import { nativeRequest } from "./client";

export type LicenseState =
  | "UNACTIVATED"
  | "TRIAL"
  | "ACTIVE"
  | "OFFLINE_GRACE"
  | "EXPIRED"
  | "SUSPENDED"
  | "REVOKED"
  | "INVALID";

export interface LicenseStatus {
  state: LicenseState;
  deviceId: string;
  activatedAt: number | null;
  expiresAt: number | null;
  lastValidatedAt: number | null;
  nextRevalidationAt: number | null;
  offlineGraceEndsAt: number | null;
  daysRemaining: number | null;
  canOperate: boolean;
  readOnly: boolean;
  requiresActivation: boolean;
  reason: string;
}

export interface ReleaseInfo {
  version: string;
  channel: string;
  buildId: string;
  schemaVersion: number;
  updaterEnabled: boolean;
  signedBuild: boolean;
}

export const getLicenseStatus = () =>
  nativeRequest<LicenseStatus>("get_license_status");
export const startLicenseTrial = () =>
  nativeRequest<LicenseStatus>("start_license_trial");
export const activateManualLicense = (activationCode: string) =>
  nativeRequest<LicenseStatus>("activate_manual_license", { activationCode });
export const revalidateManualLicense = (activationCode: string) =>
  nativeRequest<LicenseStatus>("revalidate_manual_license", { activationCode });
export const getReleaseInfo = () =>
  nativeRequest<ReleaseInfo>("get_release_info");
export const exportDiagnostics = () =>
  nativeRequest<string>("export_diagnostics");
