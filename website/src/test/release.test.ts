import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { currentRelease } from "../content/release";

it("serves the exact closed Phase 9A.1 installer", () => {
  const path = resolve("public/releases/0.4.0-beta.1/M2-Health_0.4.0-beta.1_x64-setup.exe");
  const bytes = readFileSync(path);
  expect(bytes.length).toBe(currentRelease.sizeBytes);
  expect(createHash("sha256").update(bytes).digest("hex").toUpperCase()).toBe(currentRelease.checksum);
});
