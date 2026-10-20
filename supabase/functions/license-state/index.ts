import { createClient } from "npm:@supabase/supabase-js@2";
import { signEnvelope, validateClaims } from "./contract.ts";

const allowedOrigin = Deno.env.get("PORTAL_ALLOWED_ORIGIN") ?? "";
const cors = (origin: string | null) => ({
  "Access-Control-Allow-Origin": origin && origin === allowedOrigin ? origin : allowedOrigin,
  "Access-Control-Allow-Headers": "authorization, apikey, content-type, x-client-info",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
  "Vary": "Origin",
});
const json = (body: unknown, status: number, origin: string | null) => new Response(JSON.stringify(body), {
  status,
  headers: { "Content-Type": "application/json", ...cors(origin) },
});

Deno.serve(async (request) => {
  const origin = request.headers.get("Origin");
  if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: cors(origin) });
  if (request.method !== "POST") return json({ error: "Method not allowed." }, 405, origin);
  if (!allowedOrigin || (origin && origin !== allowedOrigin)) return json({ error: "Origin is not allowed." }, 403, origin);
  const authorization = request.headers.get("Authorization");
  if (!authorization?.startsWith("Bearer ")) return json({ error: "Authentication is required." }, 401, origin);
  try {
    const supabaseUrl = Deno.env.get("SUPABASE_URL") ?? "";
    const publishableKey = Deno.env.get("SUPABASE_PUBLISHABLE_KEY") ?? "";
    const privateKey = Deno.env.get("LICENSE_SIGNING_PRIVATE_KEY") ?? "";
    if (!supabaseUrl || !publishableKey.startsWith("sb_publishable_") || !privateKey) throw new Error("License service is not configured.");
    const token = authorization.slice(7);
    const supabase = createClient(supabaseUrl, publishableKey, {
      global: { headers: { Authorization: authorization } },
      auth: { persistSession: false, autoRefreshToken: false },
    });
    const { data: userData, error: userError } = await supabase.auth.getUser(token);
    if (userError || !userData.user) return json({ error: "Authentication is invalid or expired." }, 401, origin);
    const input = await request.json() as { deviceId?: string; displayName?: string; appVersion?: string; buildId?: string };
    if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(input.deviceId ?? "")) return json({ error: "A valid desktop device UUID is required." }, 400, origin);
    if ((input.displayName?.trim().length ?? 0) < 2 || (input.displayName?.trim().length ?? 0) > 120) return json({ error: "A device name between 2 and 120 characters is required." }, 400, origin);
    const { data, error } = await supabase.rpc("prepare_license_claims", {
      p_device_id: input.deviceId,
      p_display_name: input.displayName?.trim(),
      p_app_version: input.appVersion ?? null,
      p_build_id: input.buildId ?? null,
    });
    if (error) return json({ error: error.message }, 403, origin);
    const prepared = data as { status: string; claims: unknown | null };
    if (!prepared.claims) return json({ status: prepared.status, activationCode: null, expiresAt: null, revalidateAfter: null }, 200, origin);
    const claims = validateClaims(prepared.claims);
    const activationCode = await signEnvelope(claims, privateKey);
    return json({ status: prepared.status, activationCode, expiresAt: claims.expiresAt, revalidateAfter: claims.revalidateAfter }, 200, origin);
  } catch (error) {
    const message = error instanceof Error && error.message === "License service is not configured." ? error.message : "License issuance failed safely.";
    return json({ error: message }, 500, origin);
  }
});
