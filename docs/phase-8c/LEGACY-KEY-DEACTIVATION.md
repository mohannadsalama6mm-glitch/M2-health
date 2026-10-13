# Phase 8C Legacy Key Deactivation Procedure

Status: **prepared, not executed. Requires explicit owner approval.**

The legacy JWT-based `service_role` key of the linked development project was exposed in CLI output during the Phase 8C audit and was confirmed active by a redacted HTTP 200 probe. It has not been deactivated. The credential incident remains open until this procedure is executed and verified.

## Dependency audit summary (read-only)

| Surface | Result |
| --- | --- |
| Tracked repository files | No legacy JWT, `sb_secret_` value, or real publishable key. Only `sb_publishable_replace…` placeholders. |
| Untracked, non-ignored files | No legacy JWT or secret key. |
| Desktop application | Uses the modern publishable key only; Rust config rejects non-publishable keys. |
| Administrative test harness | Uses the modern secret key, fetched transiently in memory. |
| Edge Functions | None deployed. |
| Database webhooks / `supabase_functions.hooks` | Not present. |
| `pg_net`, `pg_cron` | Not installed. |
| Foreign servers | 0. |
| Vault secrets | `list_tables` reported 0 rows; a direct count query was declined by auto-review and was not retried. |
| Last 24 h logs | Available sources are `postgres_logs`, `pgbouncer_logs`, `postgrest_logs`; 0 messages mention `service_role`. API-gateway (`edge_logs`) logs are not exposed, so requests cannot be attributed to a specific API key. |

## Remaining uncertainty

- Consumers outside this repository (other machines, CI secrets, scripts, third-party dashboards, or copies of the CLI output) cannot be enumerated from here.
- Gateway logs that would show which key authenticated recent requests are unavailable through the permitted tools.
- The vault secret count was not re-confirmed after the declined query.
- Disabling legacy keys also disables the legacy `anon` JWT. No in-repo consumer of it was found, but external consumers cannot be ruled out.

## Supported procedure (Supabase Dashboard)

1. Confirm that every client uses `sb_publishable_…` or `sb_secret_…` keys. The desktop and harness already do.
2. Open the Supabase Dashboard → project **M2-health** → **Project Settings → API Keys**.
3. Optionally review **API Keys → Legacy API Keys** "last used" indicators where the dashboard shows them.
4. On the **Legacy API Keys** tab, choose **Disable JWT-based API keys**. This disables the legacy `anon` and `service_role` keys together and is reversible from the same screen.
5. Do **not** rotate the legacy JWT secret as a first step; it also invalidates every issued user session.
6. Verify with a redacted probe: a request using the legacy `service_role` value must return HTTP 401, printing the status code only.
7. Verify the desktop sign-in path and the isolated harness still work with the modern keys, then re-run `scripts/admin/phase8c-live-integration.ps1` and the PostgreSQL security script.
8. Record the date, operator, and probe result in this file and in `FINAL-REPORT.md`.

## Rollback

If a hidden consumer breaks, re-enable the legacy keys from the same dashboard screen, migrate that consumer to a modern key, and repeat the procedure. Re-enabling restores the exposed key, so treat it as temporary.
