export const siteConfig = {
  supabaseUrl: import.meta.env.VITE_SUPABASE_URL?.trim() ?? "",
  publishableKey: import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY?.trim() ?? "",
  siteUrl: import.meta.env.VITE_SITE_URL?.trim() || window.location.origin,
  supportEmail: import.meta.env.VITE_SUPPORT_EMAIL?.trim() || "viralnest902@gmail.com",
} as const;

export const supabaseConfigured =
  /^https:\/\/[a-z0-9-]+\.supabase\.co$/i.test(siteConfig.supabaseUrl) &&
  siteConfig.publishableKey.startsWith("sb_publishable_") &&
  !siteConfig.publishableKey.includes("replace_me");
