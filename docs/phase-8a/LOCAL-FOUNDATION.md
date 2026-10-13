# Local Cloud Foundation

Migration 018 creates one stable, random, non-secret `local_device_identity` row. It creates no organization or enrollment. `cloud_device_enrollments` separates device identity from cloud authorization and enforces organization membership for the enrolling user.

The transactional outbox stores immutable versioned envelopes and mutable attempt, scheduling, error, and acknowledgment metadata. Inserts require an enrolled, non-revoked device, an active actor membership, and branch membership for branch events. The inbox uses `(organization_id,event_id)` as its idempotency key and stores the payload hash and apply state. Receipts and checkpoints are tenant/device-scoped. Protocol metadata fixes version 1 and hard-disables transport.

No domain table trigger or service writes outbox rows in Phase 8A. This is intentional: an outbox written after a business commit would create silent loss, while a generic trigger would lack stable versioned domain contracts. Phase 8B must add event creation inside the same canonical transaction as each accepted mutation.

The protected status query reads the device and queue state without mutation. The native Sync page reports local-only mode, no organization, no linked identity, no enrollment, disabled transport, and unavailable synchronization. It exposes no enrollment or retry action and uses no fabricated queue records.
