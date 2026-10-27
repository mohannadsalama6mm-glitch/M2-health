# Phase 8C Security and Reliability

## Server boundary

PostgreSQL RLS is enabled and forced on tenant tables. Direct table access is denied to `anon` and `authenticated`; clients use narrowly granted security-definer RPCs. RPCs derive actors from `auth.uid()`, validate active organization and branch membership, validate enrolled-device ownership, and reject cross-tenant identifiers.

The linked migration ledger exactly matches local versions through `202610130001`. The forward revoke migration is applied, and a current privilege query confirms `public`, `anon`, and `authenticated` cannot execute `public.rls_auto_enable()`.

## Reliability behavior

- Domain writes and outbox inserts share one SQLite transaction.
- Push receipts must exactly match leased events before acknowledgment.
- Failed pushes keep pending events and schedule bounded backoff.
- Inbox insertion, canonical apply, conflict recording, and checkpoint advancement are ordered transactionally.
- Duplicate and out-of-order delivery is idempotent or retained for dependency retry.
- Invalid envelopes do not advance the checkpoint.
- Missing dependency, immutable identity, revision divergence, and constraint failures are quarantined without partial domain writes.
- Remote stock counts apply only against an exact captured batch snapshot; otherwise they fail with `REVISION_CONFLICT`.
- Paired transfers apply both stock legs atomically or neither.
- Sessions and cloud credentials are memory-only; the frontend does not persist them.
- Conflict `RETRY`/`DISMISS` is permission-checked, requires an audit note, and never edits domain data directly.

## Current live evidence

- Rollback-only tenant/branch RLS and revocation script: passed on 2026-10-20.
- Modern-key authenticated multi-device harness: passed on 2026-10-20.
- Synthetic cleanup: zero Auth users, organizations, and events.
- Full local Rust and browser regressions: passed.

## Remaining critical issue

The exposed legacy Supabase `service_role` JWT remains enabled. The dependency audit found no known consumer, modern-key replacement paths pass, and the owner-authorized deactivation workflow reached the correct dashboard page. Supabase currently renders the reversible disable control unavailable, while the official Management API rejects the current CLI identity for insufficient API-key write privilege. The incident is not resolved until an authorized owner deactivates the keys and a status-only rejection probe succeeds.

Synchronization and rollout gates remain disabled.
