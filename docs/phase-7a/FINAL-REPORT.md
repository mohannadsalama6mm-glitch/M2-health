# Phase 7A Final Report

## Scope and result

Phase 7A adds the local identity foundation for M² Health: canonical users, Argon2id credentials, first-run administrator bootstrap, backend-owned sessions, protected system roles, user-role assignments, branch memberships, and canonical desktop setup/login/user-administration screens. It does not implement Phase 7B authorization for operational commands and does not claim production-wide security.

The work used code and automated tests only. The native Tauri application was not launched. The real development database was not opened, read, copied, seeded, reset, or migrated; its last verified state remains v009.

## Security audit and boundary

The audit covered Tauri startup and command registration, fixture user/role screens, hard-coded presentation identity, branch persistence, migrations, dependencies, native/browser route selection, offline operation, and future synchronization. The resulting contract is in `SECURITY-SEMANTICS.md`.

Rust is the trusted identity boundary. A frontend caller cannot authorize itself by supplying a user or role. Protected identity commands validate a 64-character opaque session token against the Rust in-memory store, recheck the durable user activity and credential revision, and require `ADMIN` for user and assignment administration. Direct Tauri invocation therefore does not bypass identity administration. The unauthenticated entry points are limited to status, login, and one-time bootstrap; bootstrap is serialized with `BEGIN IMMEDIATE` and refuses any database with a bootstrap marker or existing user.

Operational commands from earlier phases remain callable without the Phase 7A session. Their actor fields and branch inputs are not yet replaced by the authenticated principal. This is the explicit Phase 7B boundary and the reason Phase 7A is not described as production-secure.

## Migration 016

`016_identity_foundation.sql` appends a strict, foreign-keyed identity schema:

- retained UUID users with normalized unique usernames, active state, profile revision, credential revision, and timestamps;
- one credential row per user containing only a PHC password hash;
- five protected roles with stable codes: `ADMIN`, `PHARMACIST`, `CASHIER`, `INVENTORY_MANAGER`, and `VIEWER`;
- unique user-role assignments and many-to-many branch memberships with at most one primary branch;
- a singleton bootstrap marker and non-secret authentication event history;
- triggers preventing hard deletion, system-role mutation, and removal or deactivation of the final active administrator.

The migration creates no users, passwords, sessions, or branch assignments. It was applied only to in-memory and temporary test databases. Clean migration, repeat application, upgrade, rollback, future-version refusal, foreign keys, bootstrap concurrency, and prior-domain regression tests pass.

Phase 7A did not edit migrations 001–015. Their final SHA-256 values are recorded below for handoff verification:

- 001 `192b2f0e16bf13e6d8fc355a173efbc19b15be753d21a44c7027658f4d6cb6fb`
- 002 `c71291d00c8d7580d9650bbb465a0b6e1e8aa056d50896d4e963c9982d440738`
- 003 `860281f335d2877fcd77c04bc8d9a84b297ecc3ac673b24eae126f508d8c6be2`
- 004 `0ac19da1fb94f190aa387367037a3dc3ef521dca290abf94faa3a021e87b4df0`
- 005 `e4d00170e2010ed9354cca3a615e779e84e8f1915ebaff1d78e8b07f6d5feb81`
- 006 `f0a579d39fed0c44cdfc4c33657242304e8cb12680cc635bbd33ad22b2f7d308`
- 007 `df5d72f61f6a2c0f98fe3a6ad318ece16df060a491f7efda3f985cd3f7546074`
- 008 `7125642b0a5bd1196fb21bc0bf94d5c606ef769881a087a38ff6732270390874`
- 009 `15a66d34611b9781b879d9a77dcf88f8570f0355b8c0fb8d35b91c11fcc66760`
- 010 `de49eddd3c577d11231d9c2375ffcf3181aaee53d19657d27a033e4be14e9f60`
- 011 `bbc49c3fb3f3425ab6e80e3c2c72c9076e19299cb9d02cb489a2649b7ff98e64`
- 012 `6eaea591ef77595c78022012ab7b395e1018e52a6e51b7dd92655a3d276376dc`
- 013 `cac57b462b392642ab9dcf167936b5a16c49b02cb81bc54ba7685eff67777975`
- 014 `685c448e787b5e838ee807eb91f71a333b86cbb6d9762df1f5e976ebe6837828`
- 015 `d977d4e909f744ffac5a8a0915030cf0c162b896aca4fa234e326ab4beab9188`

Migration 016 SHA-256 is `949287ad42864d425a39b8a1f3543f9ab9c15ea20eab7bdfc617aae01db01bcc`.

## Passwords and sessions

Passwords are 12–128 Unicode scalar values, reject NUL and the normalized username, and are hashed with RustCrypto Argon2id v1.3 using a fresh random salt, 19,456 KiB memory, two iterations, one lane, and 32-byte output. The database stores only the PHC string. Unknown users, incorrect passwords, inactive users, and throttled attempts share one generic error. Five failures begin exponential in-memory backoff from 30 seconds to an eight-minute cap.

Sessions use two random UUIDv4 values as one opaque 64-hex-character token. Rust stores only its SHA-256 digest. Absolute lifetime is eight hours. Expired tokens are removed and rejected; logout removes the token; password reset and deactivation revoke every session for the affected user; credential revision is rechecked on each request. Restart clears every session and requires login.

The raw token is held only in React memory. Static inspection finds no authentication use of `localStorage`, `sessionStorage`, console logging, Rust printing, tracing, or log macros. Automated browser tests confirm both frontend storage areas remain empty through setup, login, and supported viewport flows. Passwords, hashes, and tokens are absent from audit-event detail.

## Users, roles, and branch memberships

The backend supports user creation, profile update with optimistic revision, activation/deactivation, detail retrieval, search, active/role filters, bounded pagination, administrator password reset, role assignment/removal, and complete branch-membership replacement. Duplicate or unknown assignments are rejected. User and credential hard deletion is blocked. The last active administrator cannot lose `ADMIN` or be deactivated, including through direct SQL trigger enforcement.

The canonical desktop UI replaces fixture Employees/Roles screens only when the backend advertises Phase 7A. It supports first-run setup, login/logout, user list/search/filter/page, user creation, profile edits, activation, password reset, role changes, branch membership, primary branch, and a read-only system-role directory. Browser preview continues to use its fixture experience.

## Automated verification

Final verification covers bootstrap once-only semantics and concurrent attempts, unique salted hashes and no plaintext, generic login failures, throttling, inactive rejection, logout, process-restart invalidation, absolute expiry rejection, password-reset/deactivation revocation, valid/non-admin/invalid sessions, user search/filter/pagination, stale writes, final-admin protection, protected roles, role and branch validation, isolated migration behavior, storage absence, and canonical UI flows.

Automated viewport tests cover 1366×768, 1440×900, and 1920×1080 for authenticated user administration and assert no document-level horizontal overflow. Existing application viewport suites also remain active.

Final command results:

- `cargo fmt --check`: passed
- `cargo check`: passed
- `cargo test`: **456 passed, 0 failed** across 24 test binaries including library tests; main and documentation targets contain no tests
- `npm run lint`: passed with zero errors and zero warnings
- `npm run build`: passed; Vite emitted only its advisory bundle-size warning
- `npm test`: **284 passed, 0 failed**
- `git diff --check`: passed

Total automated tests in the final standard suites: **740 passed, 0 failed**.

## Remaining limitations and Phase 7B boundary

- Existing operational IPC does not yet require a session or derive actor/branch scope from the authenticated principal.
- Role codes are durable, but operational permission matrices are intentionally undefined.
- Branch memberships are durable identity data, but do not yet grant or deny access or constrain branch switching.
- Local database encryption, operating-system keychain integration, remote identity, cloud synchronization, credential recovery, device compromise, and malicious binary replacement remain outside Phase 7A.
- An app-process restart and frontend document reload require a new login by design; no refresh-token mechanism exists.

## Status

**Phase 7A: COMPLETE**
