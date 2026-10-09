# Phase 7C.2 — Actor Integrity, Security Events, Async and Concurrency

## Result

Phase 7C.2 is complete. The audit found that the authorization gate validated caller-supplied actor values but several legacy write commands could omit the actor and persist a null audit identity. The command boundary now obtains the actor from the validated Rust principal for every actor-bearing Sales, Inventory, purchasing, Finance, document, and stock-count mutation. Caller values are no longer authoritative.

## Actor provenance

`AuthorizationState` exposes the validated principal only for the lifetime of a protected synchronous dispatch. A scoped guard clears it after the handler returns. Command adapters use `authorization.actor()?` and pass that identity into domain transactions. Tests prove the principal is unavailable before and after dispatch, a forged actor is rejected, and all actor-bearing adapters source identity from the authorization state.

Migration 017 adds immutable `created_by` attribution to `inventory_batches` and `sale_returns`, the two persisted records that lacked it. Sales drafts, posted sale movements, return records, return movements, void movements, purchase receipts, purchase returns, Inventory batches and movements, transfer legs, inventory documents and reversals, stock-count corrections, and expenses now receive the authenticated actor at the Rust boundary. Finance journals retain canonical source-to-journal attribution rather than a second caller-controlled actor field.

Earlier partial-migration fixtures remain supported: batch and return services inspect schema capability before writing the v017 columns. Full databases always use the new columns.

## Security events

Migration `017_security_events.sql` creates an append-only, strict `security_events` table with foreign keys to actor and subject users. It records `USER_CREATED`, `USER_UPDATED`, `USER_ACTIVATED`, `USER_DEACTIVATED`, `PASSWORD_RESET`, `ROLE_ASSIGNED`, `ROLE_REMOVED`, and `BRANCH_MEMBERSHIP_CHANGED`.

The identity mutation and security-event insert share one `IMMEDIATE` transaction. An injected event-write failure rolls back the privileged identity change. Update and delete triggers make events immutable. Event detail is bounded and tests prove neither password material nor raw session tokens are stored.

## Async authorization

The registered async commands are exactly `profile_catalog_source`, `dry_run_catalog_import`, and `get_catalog_import_report`. All three are protected, read-only Catalog inspection/report operations. They are authorized at dispatch and do not hold the synchronous authorization mutex across `.await`. A permission change after an accepted read-only dispatch does not cancel that already accepted inspection. `apply_catalog_import` is a protected synchronous write command.

No registered async command mutates Inventory, Sales, purchasing, Finance, identity, or branch-owned data.

## Concurrency and revocation

Protected synchronous dispatch serializes its authorization check and handler completion with role and membership mutation. Deterministic tests prove logout waits for an in-flight protected request, then revokes the token; subsequent requests fail. Durable role and membership changes are re-read on every request. Password reset and user deactivation revoke sessions. Disabled, expired, malformed, missing, and revoked sessions fail before domain access.

The lock order is authorization boundary, short-lived session/database validation, then domain handler. No authorization lock is held across an async await. Existing concurrent stock overspend, receipt, return, expense numbering, and financial rollback tests pass.

## Evidence

- Focused authentication, authorization, Sales, and Inventory run: 121 passed.
- Authentication target: 12 passed.
- Authorization target: 20 passed.
- Complete Rust suite: 482 passed.
- Migration 017 applies only in isolated in-memory or temporary test databases.

**Phase 7C.2: COMPLETE**
