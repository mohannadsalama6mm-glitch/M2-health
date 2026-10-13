# Phase 8A Architecture Audit

## Audited baseline

The desktop application is React/TypeScript over a Tauri v2 Rust boundary and one local SQLite database. Startup opens the application-data database, enables foreign keys and WAL, and applies a sequential migration registry in one `IMMEDIATE` transaction. Unknown or renamed migration metadata fails closed. This phase did not open the real development database; all execution used in-memory or temporary fixtures.

Rust owns credentials, in-memory session tokens, durable role and branch membership checks, and all 163 registered IPC policies. Four bootstrap/authentication commands remain public. The new cloud-status command is read-only, session-protected, and registered through the same guarded handler.

Canonical persistence is already domain-specific: Inventory stock is derived from immutable movements; Sales and Purchasing use transactional state changes and inventory movements; Finance uses balanced immutable journals; security events retain authenticated attribution. Reports and Dashboard are derived views, not independent sources of truth.

## Findings relevant to multi-device safety

- Branch IDs and domain record IDs are UUIDs, but the pre-8A schema had no organization owner.
- Local users, roles, and branch memberships represented one installation, not cloud identity or cloud membership.
- The prior Sync page used browser-only sample records and native mode reported unavailable.
- No outbox, inbox, durable receipt, checkpoint, device identity, cloud schema, or RLS policy existed.
- Business transactions did not atomically create synchronization envelopes. Describing such writes as reliably synchronized would be unsafe.
- Stock quantities and Finance balances cannot use last-write-wins. Inventory movements and posted journals require ordered append and server arbitration.
- A local administrator is not evidence of cloud organization authority.
- Tauri capabilities expose only the application window capability; cloud credentials and endpoints are absent from checked configuration and frontend source.

## Implemented boundary

Migration 018 adds tenant adoption records, organization and branch membership foundations, a stable non-secret local device ID, explicit enrollment state, and dormant outbox/inbox/receipt/checkpoint tables. It creates no organization, ownership, cloud identity, enrollment, or event rows. A separate PostgreSQL migration defines the future cloud control plane and RLS.

Phase 8A deliberately adds no transport and no transaction hooks. Phase 8B must finalize event contracts and add outbox creation inside each canonical domain transaction before enabling upload.
