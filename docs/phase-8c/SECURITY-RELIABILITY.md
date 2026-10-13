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

## Remaining critical issue

The legacy Supabase service-role JWT was exposed by CLI output during this audit. It must be deactivated or rotated, and downstream command-output retention must be treated as exposed, before any rollout approval.

## Remaining product limits

Full document-graph replication, concurrent Catalog reconciliation, globally serialized offline stock reservations, and atomic cloud branch transfers are not implemented. These limits are shown as unsupported behavior; they do not fall back to last-write-wins.
