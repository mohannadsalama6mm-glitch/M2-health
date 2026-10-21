import { argv, env, exit } from "node:process";

const values = Object.fromEntries(argv.slice(2).map((argument) => {
  const [name, ...rest] = argument.replace(/^--/, "").split("=");
  return [name, rest.join("=")];
}));
const action = values.action || "list";
const baseUrl = env.M2_SUPABASE_URL?.replace(/\/$/, "") ?? "";
const secret = env.M2_SUPABASE_SECRET_KEY ?? "";
if (!/^https:\/\/[a-z0-9-]+\.supabase\.co$/i.test(baseUrl)) throw new Error("Set M2_SUPABASE_URL to the approved HTTPS project URL.");
if (!secret.startsWith("sb_secret_") || /\s/.test(secret)) throw new Error("Set M2_SUPABASE_SECRET_KEY to an approved modern server key.");

async function rpc(name, body = {}) {
  const response = await globalThis.fetch(`${baseUrl}/rest/v1/rpc/${name}`, {
    method: "POST",
    headers: { apikey: secret, "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const payload = await response.json().catch(() => null);
  if (!response.ok) throw new Error(`Admin operation rejected with HTTP ${response.status}.`);
  return payload;
}

if (action === "list") {
  const requests = await rpc("admin_list_activation_requests");
  console.log(JSON.stringify({ pendingRequests: requests }, null, 2));
} else if (action === "activate") {
  if (!/^[0-9a-f-]{36}$/i.test(values.account ?? "")) throw new Error("Pass --account=<account UUID>.");
  const state = (values.state || "ACTIVE").toUpperCase();
  const plan = (values.plan || "SINGLE_PHARMACY").toUpperCase();
  const devices = Number(values.devices || 1);
  const note = values.note?.trim() ?? "";
  if (!note) throw new Error("Pass --note=<auditable reason>.");
  const starts = values.starts || new Date().toISOString();
  const ends = values.ends || null;
  const result = await rpc("admin_set_license_state", {
    p_account_id: values.account,
    p_state: state,
    p_plan_code: plan,
    p_starts_at: starts,
    p_ends_at: ends,
    p_max_devices: devices,
    p_note: note,
  });
  console.log(JSON.stringify({ updated: true, accountId: result.accountId, state: result.status, plan: result.planCode, maxDevices: result.maxDevices }));
} else {
  console.error("Use --action=list or --action=activate.");
  exit(2);
}
