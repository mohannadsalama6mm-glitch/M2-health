import { randomUUID, sign } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const values = Object.fromEntries(
  process.argv.slice(2).map((argument) => {
    const [name, ...rest] = argument.replace(/^--/, "").split("=");
    return [name, rest.join("=")];
  }),
);
const deviceId = values.device;
const state = (values.state || "ACTIVE").toUpperCase();
const output = values.output;
if (!deviceId || !/^[0-9a-f-]{36}$/i.test(deviceId)) throw new Error("Pass --device=<device UUID>.");
if (!output) throw new Error("Pass --output=<private .license.local path>.");
if (!["ACTIVE", "SUSPENDED", "REVOKED"].includes(state)) throw new Error("State must be ACTIVE, SUSPENDED, or REVOKED.");
const issuedAt = Math.floor(Date.now() / 1000);
const expiresAt = values.expires ? Math.floor(new Date(values.expires).getTime() / 1000) : null;
const revalidateDays = Number(values.revalidateDays || 30);
const offlineGraceDays = Number(values.offlineGraceDays || 3);
if (!Number.isInteger(revalidateDays) || revalidateDays < 1) throw new Error("Revalidation days must be a positive integer.");
if (!Number.isInteger(offlineGraceDays) || offlineGraceDays < 0 || offlineGraceDays > 30) throw new Error("Offline grace days must be between 0 and 30.");
const claims = {
  version: 1,
  licenseId: values.licenseId || randomUUID(),
  deviceId,
  state,
  issuedAt,
  expiresAt,
  revalidateAfter: issuedAt + revalidateDays * 86_400,
  offlineGraceDays,
};
const payload = Buffer.from(JSON.stringify(claims));
const privateKey = readFileSync(resolve("config/license-signing.private.local.pem"), "utf8");
const signature = sign(null, payload, privateKey);
const activationCode = `${payload.toString("base64url")}.${signature.toString("base64url")}`;
writeFileSync(resolve(output), `${activationCode}\n`, { encoding: "utf8", mode: 0o600, flag: "wx" });
console.log("Created the device-scoped activation file. No activation code or signing key was printed.");
