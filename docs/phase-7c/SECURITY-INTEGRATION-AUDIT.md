# Phase 7C Security Integration Audit

## Boundary

All 162 registered Tauri commands pass through one centralized Rust authorization guard. The four-command public allowlist is unchanged, every protected command has exactly one explicit policy, and unknown commands fail closed. Frontend route visibility is usability only; direct IPC encounters the Rust boundary.

The validated principal is reloaded from durable user, role, credential-revision, and branch-membership state for each protected invocation. The principal is exposed to synchronous command adapters only within the guarded dispatch scope. Actor-bearing domain commands stamp the resulting user ID. Caller-controlled actor fields cannot choose another identity and omission no longer creates null attribution on active mutation paths.

## Events and secrets

Migration 017 provides immutable structured identity-security events with actor, optional subject, bounded non-secret detail, type, and timestamp. Identity mutation and event insert are atomic. Passwords, hashes, raw tokens, and authentication payloads are excluded. Tokens remain opaque and in memory; the frontend does not persist them or log them.

## Async and concurrency

The only async commands are protected read-only Catalog source/report operations. Authorization happens before acceptance; accepted reads are not cancelled by a later permission change. No global synchronous lock crosses an `.await`. Synchronous protected handlers are serialized against identity/permission mutation to close the in-process authorization check/use race. Logout, reset, deactivation, role removal, and membership removal take effect deterministically.

## Remaining security boundaries

This protects the local application IPC and domain transaction boundary. It does not provide database-at-rest encryption, OS keychain integration, recovery credentials, device compromise protection, malicious binary replacement protection, remote identity, or synchronization security. Finance access remains administrator-only because no accountant role exists. These are future design areas and do not weaken the verified Phase 7C local boundary.

No unresolved critical or high-severity security issue was found in the registered IPC surface.
