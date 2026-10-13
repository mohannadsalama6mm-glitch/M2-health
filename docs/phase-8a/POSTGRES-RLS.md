# PostgreSQL RLS

The deployable migration is `supabase/migrations/202610090001_phase_8a_cloud_foundation.sql`. It is independent from SQLite and creates only the cloud control plane: organizations, memberships, branches, branch memberships, local identity links, devices, sync event envelopes, and receipts.

Every tenant-scoped table enables and forces RLS. The migration first revokes all table privileges from `anon` and `authenticated`, then grants authenticated users read access only. There are no client insert, update, or delete policies. Membership, role, branch, device enrollment, event ingestion, and receipt writes therefore require a future trusted backend path.

Security-definer predicates use `auth.uid()` to verify active organization membership, organization administration, branch membership, and active device ownership. Read policies enforce organization scope; branch-scoped events additionally require branch membership; receipt reads require device ownership or organization administration. A user cannot create its own membership, elevate a role, enroll a device, or write a forged event through the authenticated database role.

Static tests verify RLS and `FORCE ROW LEVEL SECURITY` on all eight tables, the blanket privilege revocation, helper predicates, and absence of a service-role literal. PostgreSQL, Docker, and Podman are unavailable in this environment. Policy execution, recursion behavior under the deployment owner, JWT integration, and negative cross-tenant runtime tests are pending against an isolated Supabase/PostgreSQL test project. This phase does not claim runtime RLS verification.

Deployment order is: provision an isolated Supabase project; apply this migration as the trusted migration owner; verify function ownership/search paths and RLS with two organizations, multiple users, branches, and devices; run negative tests as `anon` and `authenticated`; only then authorize Phase 8B service endpoints. No desktop service-role credential is permitted.
