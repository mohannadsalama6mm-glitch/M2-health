# Phase 8B Offline Sync Engine

## Runtime shape

Phase 8B adds a dormant offline-first synchronization engine around the Phase 8A tenant and device foundation. Local business transactions remain authoritative while offline. Eligible canonical writes append an immutable event to `sync_outbox` in the same SQLite transaction. A synchronization cycle leases a bounded batch, pushes it through authenticated Supabase RPCs, durably acknowledges the server receipts, pulls events after the local checkpoint, stores them in `sync_inbox`, applies verified events through canonical SQLite constraints, and advances the checkpoint only after the inbox transaction commits.

The transport has two independent enablement gates. Local configuration must set `transportEnabled` to true and `sync_runtime_gate.phase8c_verified` must be one. Both are false by default. Status reporting also requires an enrolled device, configured endpoint, authenticated in-memory cloud session, active local-to-cloud identity link, and both gates before it can report synchronization as available.

## Authentication and authorization

The desktop uses the Supabase publishable key only. Email and password cross typed IPC once for cloud sign-in. Rust exchanges them for an access and refresh token and retains the cloud session in process memory. Tokens are never returned to React, placed in browser storage, written to SQLite, or included in safe attempt messages.

The server RPCs derive the actor from `auth.uid()`. They validate active organization membership, branch membership, enrolled-device ownership, tenant and branch identity, bounded batch sizes, event identity, payload hash, and schema version. Direct writes remain unavailable to `anon` and `authenticated`; only the four security-definer RPCs are executable by `authenticated`.

The local Tauri authorization gate protects all cloud commands. Cloud sign-in, sign-out, and sync require `cloud.manage`. Status uses the authenticated local session. Cloud sessions are bound to the local user that created them.

## Idempotency and ordering

- Local events use UUID identity, monotonically increasing local sequence, canonical JSON, SHA-256 payload identity, causation identity, and an immutable envelope.
- The server assigns organization-scoped sequence numbers. Replaying an identical event returns its existing receipt. Reusing an event identity with a different payload is rejected.
- Local receipt acceptance requires the exact pushed event set. A server acknowledgment happens before the local outbox is marked acknowledged, so an interrupted local acknowledgment safely retries the same event.
- Pulls are ordered by server sequence. The checkpoint advances only with durable inbox insertion.
- Repeated inbox identities with the same payload are idempotent. A changed payload under the same identity is rejected.

## Canonical application

Phase 8B has verified apply handlers for canonical inventory movements with their batch snapshot and balanced Finance journals with their immutable lines. Inventory writes pass through the existing batch, foreign-key, movement-sign, branch, and no-negative-stock constraints. Finance writes pass through the existing posting, balance, source, and immutability constraints. The apply context suppresses outbox echo.

Each remote event is applied under a savepoint. A failed event rolls back every domain write, becomes a persistent conflict, and leaves later inspection evidence. Retryable dependency, revision, and apply failures are reconsidered on later cycles and resolve their conflict record after a successful apply. Invalid, immutable, and unsupported events remain quarantined.

Atomic transfers are intentionally fail-closed in this contract. Individual transfer legs are never applied remotely because a paired transfer event contract and authorization check are required. Phase 8C must verify the enabled rollout scope and keep transfer synchronization disabled until that paired contract is present and tested.

## Retry behavior

Outbox leases prevent simultaneous local claims. Failed requests release the lease, increment the attempt counter, retain only a safe error message, and schedule bounded exponential backoff. Durable attempt rows record direction, outcome, count, and safe metadata. Credentials and response bodies are excluded.

## Current state

The engine is implemented and tested, but remains dormant. The committed example and ignored local configuration keep transport disabled, and SQLite migration 019 creates the Phase 8C gate at zero. Phase 8C must complete live security and integration verification before either gate is enabled.
