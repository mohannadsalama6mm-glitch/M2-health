# Cloud Identity

Local authentication remains Rust-managed and offline-capable. Passwords and password hashes remain local and are never part of an event payload. Local sessions authorize local IPC only; they do not authenticate to Supabase.

The future cloud identity is a Supabase Auth subject (`auth.users.id`). PostgreSQL organization memberships, branch memberships, and device ownership determine cloud access from `auth.uid()`. Frontend role claims, organization IDs, branch IDs, and device IDs are untrusted inputs.

The local `cloud_identity_links` table can record a trusted mapping between one local user and one cloud subject within an organization. It starts empty and has no public mutation command. Linking must later require both a valid local session and server-confirmed cloud enrollment authority. A local administrator does not automatically become an organization owner or cloud administrator.

The stable local device UUID is non-secret. Enrollment is a separate record with `PENDING`, `ENROLLED`, or `REVOKED` state and a server-issued cloud device ID. Revocation blocks new local outbox events through database enforcement. A future server must also reject uploads and token refresh for revoked devices; local state alone cannot enforce server revocation while offline.

Offline local login remains available under existing local policy. Cloud membership or device revocation discovered after disconnection takes effect for cloud operations at the next trusted contact. Phase 8B must define bounded offline authorization and reconciliation behavior before any cloud mutation is enabled.
