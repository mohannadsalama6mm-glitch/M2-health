# Phase 8C Authenticated Integration

## Isolated environment

The harness targets only the linked Supabase development project. It derives the linked project identity from ignored CLI state, retrieves one modern publishable key and one modern secret key transiently through the authenticated official CLI, holds them only in process memory, and never prints or writes them.

`scripts/admin/phase8c-live-integration.ps1` creates synthetic users, organizations, branches, and devices and removes every database fixture and Auth user in `finally`. No real pharmacy data or development SQLite database is opened.

## Current executed result — 2026-10-20

The live run passed with three authenticated users, two tenants, two branches, and three devices. It verified bootstrap, owner-controlled membership, enrollment, authenticated push, duplicate replay, second-device pull, acknowledgment, server-derived actor identity, refresh-token exchange, cross-tenant denial, malformed-hash denial, device revocation, and member revocation.

Independent cleanup verification returned zero synthetic users, organizations, and events.

## Credential blocker

The prior CLI output exposure of the legacy `service_role` JWT remains an open credential incident. Modern keys are proven functional, but the legacy JWT-based keys are still enabled. The dashboard deactivation control was unavailable during the owner-authorized closure attempt. The official Management API fallback was then denied because the current CLI credential lacks API-key write privilege. No deactivation or rejection probe is claimed.

Synchronization remains disabled until this incident is resolved.
