# Phase 7B Security Boundaries

## Trusted boundary

Rust owns authentication, session validation, permissions, branch scope, canonical-record ownership, and actor verification. The React client improves the user experience by hiding navigation and blocking direct routes, but it is not an authorization authority. Every protected command registered in `src-tauri/src/lib.rs` is wrapped by `authorization::guard` before its generated Tauri handler is entered.

The registry contains one explicit policy for each of the 162 registered commands. Unknown commands fail closed. The only public commands are `get_auth_status`, `bootstrap_admin`, `login`, and the compatibility-only `recovery_status`. All other commands require a live in-memory Rust session. Identity-management commands also retain their Phase 7A administrator checks inside the command handler, giving them a second backend check.

## Sessions and principals

The gate accepts the opaque `sessionToken` only as a credential. Rust resolves it to a current principal and rechecks session expiry, user activity, credential revision, durable roles, and durable branch memberships on every invocation. Missing, malformed, expired, revoked, and inactive-user sessions are rejected as `unauthorized`. Permission or scope failures are `forbidden`.

The raw session token and current user are held in React memory. They are not written to local storage, session storage, URLs, logs, or the database. Passwords are sent only to the Rust bootstrap, login, create-user, or reset-password commands and are not logged. Existing local-storage values are workflow idempotency keys, not authentication secrets.

## Permission and branch rules

The executable role grants and all command policies are listed in `PERMISSION-MATRIX.md`. `ADMIN` has every registered permission. The four staff roles receive explicit allowlists. No role supplied by the frontend is read as proof of authorization.

Organization-wide data includes Catalog identities, suppliers, customers, purchase-order headers, and finance account definitions. Branch-owned data includes inventory batches, stock, inventory movements and documents, counts, alerts, sales, receipts, purchase returns, expenses, branch-derived journals, and operational aggregates.

Selected-branch commands read only their documented branch path. Irrelevant `branchId` fields cannot turn an all-branch query into a scoped query. A non-admin request with a null or absent branch filter is rejected for authorized-set commands. Transfers require both branch memberships. The selected frontend branch may be sent as context, but it is never authorization proof.

Record-specific commands resolve branch ownership from persisted records before the domain handler runs. Sale, return, receipt, purchase-return, expense, journal, alert, count-session, and inventory-document identifiers therefore cannot be substituted to reach another branch. Missing protected records use the same forbidden result and do not expose whether the identifier exists.

Purchase orders and partner identities remain organization-wide because their schemas have no branch owner. Purchase receipts and returns are branch-owned. Finance and finance-trace permissions are currently administrator-only; non-admin roles cannot read their rows, aggregates, source detail, journals, or CSV exports. Operational reports and dashboards permit staff roles only with an authorized branch filter. CSV export selects the branch path for the requested export kind and requires `reports.export`.

## Actor provenance

The authenticated Rust principal is the authority for actor identity. Existing domain models still contain historical actor string fields. The frontend rewrites those fields to the authenticated user ID for compatibility, and the gate recursively rejects any recognized actor field that differs from the principal. A forged `createdBy`, `postedBy`, `voidedBy`, `performedBy`, `assignedBy`, or legacy `user` value never reaches domain code.

Some older tables store nullable actor text rather than a foreign-keyed identity column. Phase 7B prevents impersonation at IPC but does not rewrite historical schemas or claim that every legacy row has a durable user foreign key.

## Dispatch, synchronization, and atomicity

`AuthorizationState` contains one process-local mutex. For synchronous protected commands, the gate acquires this mutex, validates the current principal and scope, executes the generated command handler, and then releases it. Role and membership changes use the same path, so they cannot pass an in-flight synchronous command between its authorization check and handler completion. The gate performs no SQLite work while another application lock is already held; principal/database locks are released before domain handlers acquire their own transaction locks. Automated concurrency coverage verifies this ordering and no lock inversion was found.

Authorization denial occurs before the handler and therefore causes zero domain reads or writes through that invocation. Domain transactions continue to provide the actual write atomicity, ledger invariants, idempotency, and rollback behavior. The authorization mutex is not a database transaction and does not protect against code outside the guarded Tauri registration.

Three registered Catalog import inspection commands are asynchronous: `profile_catalog_source`, `dry_run_catalog_import`, and `get_catalog_import_report`. The gate validates them before Tauri schedules their futures, so async commands do not bypass authentication or permission checks. The mutex cannot remain held across the future after the generated handler returns. These commands are read/planning/report operations. `apply_catalog_import` is synchronously authorized before it schedules its existing organization-wide background import; a permission revocation after accepted dispatch does not cancel already accepted work. This is dispatch-time authorization, not continuous authorization of an already running job.

All branch-owned and identity mutations registered today are synchronous and stay inside the authorization mutex until their handler returns. Any future async mutation must add an authorization-aware job capability or revalidation inside the future before it may be registered.

## Invocation paths

There is one production `invoke_handler` registration. It wraps the single `tauri::generate_handler!` list in `authorization::guard`. The frontend has one typed native boundary, `nativeRequest`, and direct calls from a modified frontend still encounter the Rust gate. Automated source coverage extracts the exact production registration and fails if a registered command lacks a policy or if command names are duplicated. An unregistered alias cannot reach a domain command, and an unknown policy is denied.

The protected-dispatch test invokes the same `InvokeBody` authorization core used by the gate and uses an operation closure to prove denied calls cannot enter an operation. Separate coverage verifies the production handler is wrapped. A Tauri mock-runtime integration attempt was removed because enabling Tauri's test feature produced a Windows process-loader entry-point failure before any assertion; the retained tests avoid changing production dependency features and test the real dispatch inputs and production registration instead.

## Frontend behavior

Authenticated native requests receive the in-memory session token automatically. Navigation is filtered by permission, protected routes render an access-denied state, and branch selection contains only backend-returned memberships. A stale selected branch is cleared. An `unauthorized` response clears the in-memory session and returns to login. `forbidden` leaves the valid session active and shows the command or route error.

Browser preview remains separate and cannot invoke the native database. Automated tests exercise authentication, navigation, direct URL denial, branch propagation, backend denial, session invalidation, empty persistent storage, and layouts at 1366×768, 1440×900, and 1920×1080.

## Out-of-scope security

Phase 7B does not add database encryption, operating-system keychain storage, remote identity, refresh tokens, centralized audit ingestion, malicious-binary resistance, or cancellation of already accepted background work. It does not turn legacy actor columns into user foreign keys. It does not claim protection from code that is added outside the guarded registration in the future; the inventory test is required to prevent that regression.

No native application was launched and no real development database was accessed. Migrations 001–016 are byte-for-byte unchanged from the Phase 7A recorded hashes. The real development database remains at its previously verified v009 state.
