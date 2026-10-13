# Phase 8C Authenticated Integration

## Isolated environment

The integration harness targets only the linked Supabase development project `wjvsakmkkuycwrjaqmsz`. It creates synthetic users, organizations, branches, and devices, and removes every row and Auth user in a `finally` cleanup. No real pharmacy data or development SQLite database is opened.

The administrative setup path is separated in `scripts/admin/phase8c-live-integration.ps1`. It obtains the modern secret key transiently from the authenticated Supabase CLI, keeps it in process memory, uses it only for isolated Auth-user lifecycle operations, and never passes it to desktop code. All application protocol calls use signed-in user JWTs and the publishable key.

## Executed live result

The live automated run passed with three authenticated users, two tenants, two branches, and three devices. It verified organization bootstrap, owner-controlled membership, enrollment, authenticated push, duplicate replay, second-device pull, acknowledgment, server-derived actor identity, refresh-token exchange, cross-tenant denial, malformed-hash denial, device revocation, and member revocation.

Cleanup was verified afterward: zero Phase 8C organizations, zero Phase 8C events, and zero Phase 8C Auth users remained.

## Security incident and blocker

During a CLI key-metadata audit, `supabase projects api-keys --output json` unexpectedly emitted the complete legacy service-role JWT into command output. It was not copied into source, configuration, logs owned by the app, or frontend storage. Because command output is still an exposure boundary, that legacy key must be disabled or rotated in the Supabase Dashboard before rollout. The modern secret key was not exposed.

Synchronization remains disabled. Phase 8C cannot be declared complete while the exposed legacy credential remains active.
