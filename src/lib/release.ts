import packageMetadata from "../../package.json";

export const release = {
  version: packageMetadata.version,
  channel: "Early Access",
  buildId: import.meta.env.VITE_M2_BUILD_ID?.trim() || "local",
} as const;
