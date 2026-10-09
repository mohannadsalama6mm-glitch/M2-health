# Phase 7C.1 Security and IPC Verification Report

## Scope and completion decision

Phase 7C.1 independently audited the complete Tauri IPC registration surface, the centralized Rust authorization boundary, session and identity invariants, role and branch enforcement, direct-invocation resistance, reporting/export protection, and frontend handling of authentication secrets. Confirmed defects were fixed and covered by focused regression tests.

This phase used code review and automated tests only. The native application was not launched, no native build was run, and the real development SQLite database was not opened, read, copied, or migrated. Database tests used in-memory or temporary isolated databases. No migration file from 001 through 016 was changed during Phase 7C.1. Work stopped before Phase 7C.2, Phase 7D, and Phase 8.

**Phase 7C.1: COMPLETE**

## IPC inventory and centralized boundary

The application registers exactly **162 unique Tauri commands** in the single `tauri::generate_handler!` invocation in `src-tauri/src/lib.rs`. There is exactly one production `invoke_handler` call, and it wraps that generated handler with `authorization::guard`. A Rust coverage test extracts the registered command names, requires exactly 162 unique entries, and proves that every entry has exactly one explicit authorization policy.

The executable gate evaluates authorization before command deserialization reaches domain code. A denied request does not execute the protected operation. The dispatch boundary mutex also serializes protected command execution against identity role and membership changes, avoiding a check/use race inside the process.

The public allowlist is exactly:

- `get_auth_status`
- `bootstrap_admin`
- `login`
- `recovery_status`

All other commands require a live Rust-owned session. An unregistered or unclassified command has no policy and is denied. Raw IPC bodies, which cannot provide the required structured authentication context, are also rejected. There is no authenticated fallback for unknown commands.

## Session and administrator verification

The Rust backend remains the authority for credentials and sessions. Session tokens are random opaque values; only a SHA-256 digest is retained in Rust memory, and restart clears the store. Every protected invocation rechecks the durable user row, active state, credential revision, current roles, and current branch memberships. It does not trust a stale role or membership snapshot stored at login.

Automated coverage verifies missing, malformed, invalid, expired, logged-out/revoked, password-reset-revoked, and disabled-user sessions. These cases return `unauthorized` before domain access. Password reset and deactivation revoke the target user's sessions. The last active administrator cannot be deactivated or lose the `ADMIN` role; both service validation and database constraints protect that invariant. System roles remain immutable.

First-run bootstrap remains one-time and atomic, including concurrent attempts. Passwords are validated and processed only by Rust, stored only as Argon2id PHC hashes with unique salts, and never included in authentication-event detail.

## Role and branch authorization

The stable permission registry covers identity, branches, catalog, inventory, sales, purchasing, partners, operational reports, finance reports, exports, Finance, and Dashboard operations. `ADMIN` has organization scope. Other roles receive only their explicit grants; `VIEWER` remains read-only, and Finance plus export permissions remain administrator-only because Phase 7A introduced no accountant role.

Branch enforcement uses one of these explicit scopes:

- selected branch, requiring an active persisted membership;
- authorized set, allowing organization scope only to `ADMIN` and otherwise requiring one member branch;
- canonical record, resolving ownership from the persisted sale, receipt, return, expense, journal, alert, count session, or inventory document;
- both branches, requiring membership in both ends of a transfer;
- global, only for entities that are genuinely organization-wide.

Caller-provided `authorizationBranchId` is context only and is never accepted as proof of access. Canonical-record operations resolve the stored branch before domain work, and decoy top-level branch fields cannot hide an unauthorized nested branch. Durable role removal and membership replacement take effect on the next invocation even when the token remains otherwise valid.

## Confirmed defects and fixes

The audit found and fixed five security or policy defects:

1. The typed frontend IPC client could forward a caller-supplied `sessionToken`. It now always removes any supplied token and injects only the current in-memory authenticated token. Actor fields continue to be rewritten from the in-memory principal. A browser regression test proves both forged values are replaced.
2. `create_inventory_batch` was classified as global even though the input owns a branch. It now requires `inventory.receive` and membership in `/input/branchId`.
3. `record_opening_stock` inspected the wrong branch path. It now authorizes the actual nested `/input/batch/branchId` and rejects an unauthorized nested branch even when a permitted top-level decoy is supplied.
4. `record_adjustment` inspected the wrong branch path. It now authorizes the actual nested `/input/operation/branchId` with the same decoy-field resistance.
5. `create_finance_expense` incorrectly forced every expense to contain a branch although the canonical model permits administrator-owned organization expenses. It now uses authorized-set semantics: an administrator may use the documented null organization scope, while every supplied branch is validated and non-administrators still lack the Finance mutation permission.

The Phase 7B permission matrix and security-boundary wording were corrected to match the executable policy. Comments around the Tauri guard and its test seam were also clarified so documentation does not overstate what directly executes in production.

## Direct IPC and mutation evidence

Focused Rust tests invoke the authorization layer with crafted IPC payloads rather than relying on route visibility. They verify:

- a non-administrator cannot invoke identity mutations and the user table remains unchanged;
- forged actor identities are rejected, while the validated principal is accepted;
- missing permissions and unauthorized branches fail before the operation closure is entered;
- same-token role and membership changes are applied from durable state on the next call;
- branch-owned inventory batch creation, opening stock, and adjustment cannot cross branches;
- transfers require both branch memberships;
- canonical sale and finance record IDs resolve persisted ownership;
- irrelevant branch fields cannot widen report or export scope;
- a cashier cannot invoke Finance export through the native test double;
- `VIEWER` cannot mutate inventory, sales, purchasing, partners, identity, Finance, or exports;
- denied mutation tests observe zero domain writes.

Operational reporting, Finance reporting, Dashboard aggregates, and CSV export use authorized-set or canonical-record policy. Null/all-branch requests are restricted to `ADMIN`; other roles must supply an authorized branch. The policy is checked before report generation, so unauthorized callers do not receive rows, totals, or exported data.

## Frontend token and secret handling

The typed IPC client stores token, user ID, and selected branch only in module/React memory. It strips a request's `sessionToken` and replaces it with the current in-memory token; it also normalizes known actor fields to the authenticated user. An `unauthorized` backend response broadcasts session invalidation and clears the React session.

Static inspection found no authentication token, password, or password hash written to `localStorage`, `sessionStorage`, console output, Rust print output, tracing, or log macros. The application does persist non-secret workflow idempotency IDs for inventory documents and stock counts. Browser tests confirm authentication flows leave both storage areas empty. Test doubles contain fixture passwords and temporary session state only inside the automated browser test environment; they are not production persistence.

Frontend navigation and route checks improve user experience but are not treated as the security boundary. A caller who bypasses the UI still encounters the Rust dispatch guard.

## Migration verification

The SHA-256 hashes of migrations 001–016 exactly match the Phase 7A baseline, including migration 016:

`949287ad42864d425a39b8a1f3543f9ab9c15ea20eab7bdfc617aae01db01bcc`

The foundation suite creates isolated databases, applies all 16 migrations, verifies repeatability and foreign keys, rolls back failed migrations without partial metadata or schema, and refuses a newer unknown schema. The authentication suite also verifies migration 016 creates identity structures without creating users or credentials.

## Automated evidence

Focused security checks:

- Rust authorization suite: **16 passed, 0 failed**.
- Focused authentication/authorization browser checks: **10 passed, 0 failed**.
- Authentication viewport coverage: **1366×768, 1440×900, and 1920×1080**, all passed without document-level overflow.

Final standard checks:

- `cargo fmt --check`: passed.
- `cargo check`: passed.
- `cargo test`: **472 passed, 0 failed** across library and integration targets; main and documentation targets contain no tests.
- `npm run lint`: passed with zero errors and zero warnings.
- `npm run build`: passed; Vite emitted only its advisory chunk-size warning.
- `npm test`: **289 passed, 0 failed**.
- `git diff --check`: passed.

Total tests in the final Rust and frontend suites: **761 passed, 0 failed**.

## Files changed for Phase 7C.1

- `src-tauri/src/authorization.rs`
- `src-tauri/tests/authorization.rs`
- `src/lib/tauri/client.ts`
- `tests/authorization.spec.ts`
- `docs/phase-7b/PERMISSION-MATRIX.md`
- `docs/phase-7b/SECURITY-BOUNDARIES.md`
- `docs/phase-7c/7C1-SECURITY-REPORT.md`

The working tree also contains the completed work from earlier phases; Phase 7C.1 did not discard or rewrite that baseline.

## Remaining risks and boundaries

- Authentication and authorization protect the application IPC boundary within the local process. Database encryption, operating-system keychain integration, device compromise, malicious binary replacement, remote identity, cloud synchronization, and credential recovery remain outside this phase.
- Sessions have an eight-hour absolute lifetime and no refresh-token mechanism. Frontend reload and process restart require login by design.
- Asynchronous catalog import inspection is authorized at dispatch. Its background inspection continues under that dispatch-time authorization snapshot; it performs no catalog mutation. A future cancellation/revalidation design may be considered if long-running privileged background mutations are introduced.
- Some legacy domain actor columns remain optional. Direct IPC cannot forge another actor because mismatches are denied, but a direct caller can omit an optional actor and leave legacy attribution null. Authorization still uses the validated Rust principal. Making all legacy audit-attribution columns mandatory would require a separately scoped schema/domain migration and is not part of Phase 7C.1.
- The full automated evidence establishes policy coverage and regression behavior, not resistance to a hostile operating system or modified executable.

No known registered IPC command bypasses the centralized authorization gate, branch policy, protected reporting/export policy, or no-negative-stock/domain transaction safeguards after these fixes.
