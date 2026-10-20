import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { siteConfig, supabaseConfigured } from "./config";

let client: SupabaseClient | null = null;

export function getSupabase(): SupabaseClient {
  if (!supabaseConfigured) {
    throw new Error("Customer accounts are not configured for this deployment yet.");
  }
  client ??= createClient(siteConfig.supabaseUrl, siteConfig.publishableKey, {
    auth: {
      persistSession: true,
      autoRefreshToken: true,
      detectSessionInUrl: true,
      storageKey: "m2-health-portal-session",
    },
  });
  return client;
}
