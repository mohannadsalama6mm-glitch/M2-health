export interface LicenseClaims {
  version: 1;
  licenseId: string;
  deviceId: string;
  state: "ACTIVE" | "SUSPENDED" | "REVOKED";
  issuedAt: number;
  expiresAt: number | null;
  revalidateAfter: number;
  offlineGraceDays: number;
}

export function validateClaims(value: unknown): LicenseClaims {
  const claims = value as Partial<LicenseClaims>;
  if (
    claims.version !== 1 ||
    typeof claims.licenseId !== "string" ||
    typeof claims.deviceId !== "string" ||
    !["ACTIVE", "SUSPENDED", "REVOKED"].includes(claims.state ?? "") ||
    !Number.isInteger(claims.issuedAt) ||
    (claims.expiresAt !== null && !Number.isInteger(claims.expiresAt)) ||
    !Number.isInteger(claims.revalidateAfter) ||
    !Number.isInteger(claims.offlineGraceDays) ||
    claims.offlineGraceDays! < 0 ||
    claims.offlineGraceDays! > 30
  ) throw new Error("The server produced invalid license claims.");
  return claims as LicenseClaims;
}

export function base64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

export async function signEnvelope(claims: LicenseClaims, privateKeyPem: string): Promise<string> {
  const body = privateKeyPem.replace(/-----BEGIN PRIVATE KEY-----|-----END PRIVATE KEY-----|\s/g, "");
  if (!body) throw new Error("License signing is not configured.");
  const der = Uint8Array.from(atob(body), (character) => character.charCodeAt(0));
  const key = await crypto.subtle.importKey("pkcs8", der, { name: "Ed25519" }, false, ["sign"]);
  const payload = new TextEncoder().encode(JSON.stringify(claims));
  const signature = new Uint8Array(await crypto.subtle.sign("Ed25519", key, payload));
  return `${base64Url(payload)}.${base64Url(signature)}`;
}
