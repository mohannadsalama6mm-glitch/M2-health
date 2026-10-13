# Phase 8C Multi-device Verification

## Verified

- Three synthetic devices across two tenants were enrolled through authenticated RPCs.
- An event pushed by one device was pulled by a second authorized device and acknowledged.
- Replaying the same event returned the original server sequence and did not duplicate it.
- The server replaced client-supplied actor identity with `auth.uid()`.
- Cross-tenant push, revoked-device push, and revoked-member push were denied.
- Local isolated tests verify durable outbox leases, exact receipt matching, checkpoint ordering, duplicate inbox identity, retry scheduling, dependency retry, immutable Finance journals, and canonical Inventory constraints.
- Remote inventory that would create negative stock is quarantined and its partial batch creation rolls back.

## Entity coverage and consistency limits

The active apply contract supports immutable `inventory_movement` and `finance_journal` events. Sales, returns, receipts, and counts participate only through their canonical ledger and Finance effects; their complete business-document graphs are not replicated. Catalog mutation events are not supported and are quarantined. Consequently, Phase 8C does not prove concurrent Catalog merging or complete multi-device document replication.

Offline stock is locally protected from going negative. Two disconnected devices can still each sell against stale availability. On convergence, any movement that would violate canonical no-negative-stock constraints is quarantined for operator resolution; the engine does not silently overwrite stock. This preserves ledger integrity but does not provide reservation-based global oversell prevention.

## Transfers

Single transfer legs remain fail-closed with a missing-dependency conflict. No paired atomic cloud-transfer contract has been proven. Cloud-synchronized branch transfers are unsupported, and the engine never applies only one side.
