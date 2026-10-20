import { generateKeyPairSync, sign, verify } from "node:crypto";
import { base64Url, validateClaims } from "../../../supabase/functions/license-state/contract";

it("produces a desktop-compatible device-bound Ed25519 envelope", () => {
  const claims = validateClaims({ version: 1, licenseId: "account-1", deviceId: "93000000-0000-4000-8000-000000000001", state: "ACTIVE", issuedAt: 1_800_000_000, expiresAt: 1_800_604_800, revalidateAfter: 1_800_086_400, offlineGraceDays: 3 });
  const payload = Buffer.from(JSON.stringify(claims));
  const keys = generateKeyPairSync("ed25519");
  const signature = sign(null, payload, keys.privateKey);
  const activationCode = `${base64Url(payload)}.${base64Url(signature)}`;
  const [encodedPayload, encodedSignature] = activationCode.split(".");
  expect(verify(null, Buffer.from(encodedPayload, "base64url"), keys.publicKey, Buffer.from(encodedSignature, "base64url"))).toBe(true);
  expect(JSON.parse(Buffer.from(encodedPayload, "base64url").toString()).deviceId).toBe(claims.deviceId);
});

it("rejects client-forged or unsupported claim states before signing", () => {
  expect(() => validateClaims({ version: 1, state: "TRIAL" })).toThrow(/invalid license claims/i);
});
