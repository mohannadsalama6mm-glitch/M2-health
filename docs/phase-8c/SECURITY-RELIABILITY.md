# Phase 8C Security and Reliability

## Server boundary

PostgreSQL RLS is enabled and forced on tenant tables. Direct table access is denied to `anon` and `authenticated`; clients use narrowly granted security-definer RPCs. RPCs derive the actor from `auth.uid()`, validate active organization and branch membership, validate enrolled device ownership, and reject cross-tenant identifiers.

Migration `202610100002_phase_8c_security_integration.sql` adds authenticated bootstrap, owner-controlled membership, revocation, and a canonical SHA-256 payload-integrity trigger. Public and anonymous execution is revoked.

## Reliability behavior

- The domain write and outbox insert share one SQLite transaction.
- Push receipts must exactly match leased events before acknowledgment.
- Failed network pushes keep pending events and schedule bounded backoff.
- Inbox insertion, apply result, conflict record, and checkpoint advancement are durable and ordered.
- Duplicate delivery is idempotent by tenant and event identity.
- Invalid envelopes do not advance the checkpoint.
- Unsupported, malformed, missing-dependency, and constraint-violating events are quarantined without partial domain writes.
- Sessions and credentials are memory-only. The frontend does not use localStorage, sessionStorage, or IndexedDB for cloud secrets.

- Remote events created while applying (`applying=1`) do not re-enter the outbox.
- Conflict resolution (`RETRY`/`DISMISS`) is permission-checked, requires an audit note, and never writes domain data directly.

## Remaining critical issue (open)

The legacy Supabase `service_role` JWT was exposed by CLI output during this audit and a redacted HTTP 200 probe confirmed it is still active. It has **not** been deactivated. See `LEGACY-KEY-DEACTIVATION.md`; the incident is not resolved.

## Remaining server findings

- Security advisor (2026-10-13): `public.rls_auto_enable()` is a `SECURITY DEFINER` function executable by `anon` and `authenticated`. The fix (`supabase/migrations/202610130001_phase_8c_revoke_rls_auto_enable.sql`) exists locally but is **not applied** remotely.
- The eight `authenticated` `SECURITY DEFINER` sync RPCs flagged by the advisor are the intended, actor-deriving API surface.
- The transfer-authorization migration was applied remotely as ledger version `20261013202018`, while the local file is `202610100003_…`. The ledger must be reconciled before the next `supabase db push`.
- The rollback-only PostgreSQL security script was not re-executed in the final pass.

## Remaining product limits

Purchase receipts, purchase returns, and inventory documents converge at the stock-ledger level only. Stock counts are quarantined. Post-creation Catalog/partner edits are not merged. Offline oversell is quarantined rather than prevented. None of these fall back to last-write-wins.
