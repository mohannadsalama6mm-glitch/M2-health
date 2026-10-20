export type LicenseStatus = "UNACTIVATED" | "TRIAL" | "ACTIVE" | "EXPIRED" | "SUSPENDED";
export type DeviceStatus = "ACTIVE" | "REVOKED";

export interface PortalDevice {
  id: string;
  displayName: string;
  status: DeviceStatus;
  activatedAt: string;
  lastValidatedAt: string | null;
  appVersion: string | null;
  buildId: string | null;
}

export interface ActivationRequest {
  id: string;
  status: "PENDING" | "APPROVED" | "DENIED" | "FULFILLED";
  message: string;
  createdAt: string;
}

export interface PortalSnapshot {
  accountId: string;
  fullName: string;
  email: string;
  planCode: string;
  status: LicenseStatus;
  trialStartedAt: string | null;
  trialEndsAt: string | null;
  licenseStartsAt: string | null;
  licenseEndsAt: string | null;
  daysRemaining: number | null;
  maxDevices: number;
  devices: PortalDevice[];
  activationRequests: ActivationRequest[];
}

export interface LicenseEnvelopeResponse {
  status: LicenseStatus | "REVOKED";
  activationCode: string | null;
  expiresAt: number | null;
  revalidateAfter: number | null;
}
