# Phase 7A Security Semantics

## Audit scope

The desktop process owns SQLite startup and registers every IPC command in one Tauri invoke handler. Before Phase 7A there is no canonical user, credential, login, session, role, or permission backend. `/employees` and `/roles` are browser-preview fixtures backed by `DemoContext`; the top-bar identity is also presentational. Operational commands accept caller-supplied actor text and have no authenticated identity. Branches are durable UUID records, but no user-to-branch membership exists. The product is local/offline-first and has no Supabase, remote identity provider, or cloud synchronization authority.

## Threat model

Phase 7A protects local application authentication data against accidental exposure, offline credential guessing within the running application, predictable sessions, privilege bootstrap races, stale administration writes, and removal of the final active administrator. It assumes the Tauri Rust process and the operating-system account that can read the application database are trusted. Database-file encryption, operating-system keychain integration, device compromise, malicious replacement of the application binary, remote synchronization conflicts, and authorization of every operational command are outside Phase 7A.

React is not an authorization boundary. User and role administration commands validate the backend session and ADMIN role. Phase 7B will add authorization checks to operational commands and branch-scoped enforcement.

Direct invocation of Tauri IPC does not bypass Phase 7A identity administration. User listing/detail, creation, profile/status changes, password reset, role changes, and branch-membership changes all validate the opaque session in Rust; every mutating administration command additionally requires the durable `ADMIN` role. Role listing requires a valid session. Bootstrap and login are intentionally unauthenticated entry points: bootstrap succeeds only while both the bootstrap marker and user table are empty and becomes permanently unavailable after its atomic transaction commits. Operational IPC outside the identity domain is not yet session- or role-protected; that is a known Phase 7B boundary, so Phase 7A must not be described as complete application authorization or production security.

## Trusted backend boundary

Rust owns normalization, validation, password hashing and verification, throttling, bootstrap eligibility, session issuance and revocation, user mutations, role invariants, and branch-membership persistence. The frontend may request an operation but cannot assert identity, roles, activity, or authorization.

Passwords, password hashes, and session tokens must never be logged or returned except that the newly issued opaque session token is returned once to the authenticated frontend. Errors exposed for login are generic.

## Password policy and hashing

Passwords must be 12–128 Unicode scalar values, contain no NUL, and must not equal or contain the normalized username. Phase 7A uses RustCrypto Argon2id with an independent operating-system-random salt for every credential and stores only the PHC hash string. Verification uses the library's password-verification implementation. Passwords are never encrypted, persisted in plaintext, included in request-payload audit records, or implemented with custom cryptography.

The initial parameters are Argon2id v1.3 with 19,456 KiB memory, 2 iterations, one lane, and a 32-byte output. Parameter changes require a documented compatibility and rehash plan.

## Bootstrap lifecycle

A fresh or migrated installation begins uninitialized after migration 016. No user or default password is created by migration or startup. Bootstrap is allowed only while no bootstrap marker and no user exist. One `IMMEDIATE` transaction creates the initial active user, credential, ADMIN assignment, and permanent bootstrap marker. Concurrent or repeated attempts fail without partial data. Existing v009 installations are not touched until a separately authorized future native migration run.

If credentials are lost, recovery requires an explicit future supported recovery procedure or an administrator with an active session. Deleting or editing the database manually is not an application recovery workflow.

## Session policy

A successful login creates an opaque high-entropy token. Rust stores only its SHA-256 digest in an in-memory session map. Sessions expire absolutely after eight hours, do not survive application-process restart, and are revoked on logout, password reset/change, or user deactivation. Every session lookup rechecks the durable user activity state and credential revision.

The raw token lives only in React memory for the current process. It is not written to localStorage, sessionStorage, SQLite, logs, URLs, or bundles. Application restart always requires login again.

The frontend does not persist a login across a document reload. Expiration is enforced by Rust on every protected identity request; an expired token is removed and rejected even if a stale React screen remains visible until its next backend operation.

## Login throttling

Authentication uses a generic invalid-credentials response for unknown usernames, wrong passwords, inactive users, and throttled attempts. The backend tracks failures by normalized username in memory. Five failures trigger exponential temporary backoff beginning at 30 seconds, capped at eight minutes. A successful login clears the counter. Restart clears throttling state but also clears sessions.

## User semantics

Users have stable UUIDs, a unique normalized username, display name, active state, optimistic revision, credential revision, and timestamps. User rows are retained and cannot be hard-deleted. Profile and status mutations require the expected revision. Password reset increments credential revision and revokes every existing session for that user.

## Role semantics

Phase 7A seeds protected stable roles: ADMIN, PHARMACIST, CASHIER, INVENTORY_MANAGER, and VIEWER. Role codes and system-role identities are immutable. Assignments are unique and retained through normal user lifecycle. Only ADMIN authorizes Phase 7A user and role administration. Removing or deactivating the final active ADMIN is rejected atomically.

The roles are identity and policy foundations only. Phase 7A does not infer permissions for Sales, Inventory, Purchasing, Finance, Reporting, Catalog, or other operational commands.

## Branch membership semantics

User-to-branch membership is durable and many-to-many, with at most one primary branch per user. Membership does not grant or deny operational access in Phase 7A. Phase 7B will define command authorization, branch switching, all-branch roles, and the interaction between role and membership.

## Rollout and future synchronization

Migration 016 creates identity structures and system roles without users, credentials, sessions, or branch assignments. Local UUIDs and immutable role codes are suitable synchronization identities, but Phase 7A defines no cloud authority, merge policy, credential synchronization, or remote session. Password hashes and local sessions must not be synchronized without a separately reviewed design.
