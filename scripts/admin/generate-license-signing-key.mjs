import { generateKeyPairSync } from "node:crypto";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const privatePath = resolve("config/license-signing.private.local.pem");
const publicPath = resolve("config/license-public-key.txt");
if (existsSync(privatePath) || existsSync(publicPath)) {
  throw new Error("License signing material already exists; refusing to overwrite it.");
}
const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const privatePem = privateKey.export({ type: "pkcs8", format: "pem" });
const publicJwk = publicKey.export({ format: "jwk" });
if (!publicJwk.x) throw new Error("The Ed25519 public key could not be exported.");
mkdirSync(dirname(privatePath), { recursive: true });
writeFileSync(privatePath, privatePem, { encoding: "utf8", mode: 0o600, flag: "wx" });
writeFileSync(publicPath, `${publicJwk.x}\n`, { encoding: "utf8", flag: "wx" });
console.log("Created local-only private signing material and the public build verification key. No key value was printed.");
