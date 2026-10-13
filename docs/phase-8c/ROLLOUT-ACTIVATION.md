# Phase 8C Rollout and Activation

## Current state

Synchronization is disabled by independent gates:

1. ignored local configuration must set transport enabled;
2. SQLite `sync_schema_metadata.transport_enabled` must be changed from `0` to `1`;
3. SQLite `sync_runtime_gate.phase8c_verified` must be changed from `0` to `1` with a verification timestamp;
4. the device must be enrolled, the local user linked to the same cloud subject, and an unexpired cloud session present.

Migration 020 merely makes the metadata gate activatable and copies every existing installation to `0`. It never enables transport.

## Approval procedure

Activation requires a separate task and explicit approval after all blockers are closed. Before approval:

1. disable or rotate the exposed legacy service-role key and verify only modern keys remain in use;
2. repeat the isolated security and live integration suite;
3. decide whether the restricted entity set is acceptable or implement and verify document, Catalog, and transfer protocols;
4. select an isolated rollout-candidate SQLite database, never the existing real development database;
5. back up that candidate, verify migration and integrity checks, enroll its exact organization/branch/device/user identities, and record the approval;
6. enable configuration and both database gates only for that candidate;
7. run monitored canary synchronization with synthetic data and a documented rollback procedure.

No step in Phase 8C performs activation automatically.
