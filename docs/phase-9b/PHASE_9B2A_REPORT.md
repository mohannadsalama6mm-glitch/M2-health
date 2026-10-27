# Phase 9B.2A — First-Run Onboarding + Setup Checklists

Date: 2026-10-21

## Result

Phase 9B.2A is complete. The desktop now has a localized, resumable first-run setup checklist, and the authenticated portal overview has a state-aware Early Access checklist. Both reuse the Phase 9B.1 localization system and preserve runtime Arabic RTL and English LTR behavior.

No pharmacy business rule, licensing rule, navigation structure, Supabase behavior, payment behavior, or SQLite data was changed.

## Desktop onboarding

The desktop setup checklist appears after the first successful local sign-in. This placement reuses the existing secure setup gates instead of duplicating administrator or license workflows.

It covers:

- language selection, with immediate runtime `lang` and `dir` updates
- local administrator identity and its device-local purpose
- portal account identity and its separate online purpose
- the current device-license state
- the first detected pharmacy branch
- essential settings, catalog, and backup actions
- a clear return to the dashboard

System-owned completion is derived from the existing contexts:

- local administrator: authenticated local user exists
- device/license: the existing license state can operate and does not require activation
- branch: an active loaded branch is selected
- language: the currently active supported locale

The checklist does not ask users to repeat those completed steps. Portal-account and essential-setup acknowledgements are explicitly user-controlled because the desktop has no approved read path for inferring those external/manual facts.

## Persistence and recovery

- Storage key: `m2-health.onboarding.desktop.v1`.
- Only dismissal and the two user acknowledgements are stored.
- The state is versioned, origin-local, and contains no credential or pharmacy data.
- “Skip for now” preserves progress and prevents the checklist from being forced on every sign-in.
- The checklist can be reopened from the authenticated account menu.
- Closing without choosing “Skip for now” leaves first-run presentation active for the next session.
- Incomplete activation displays a localized recovery explanation. The existing license gate remains the authoritative activation workflow.

## Identity explanations

English and Arabic copy now explicitly separates:

- local administrator: signs in to the local desktop and controls pharmacy operations
- portal account: online customer identity for trials, licenses, downloads, and registered devices
- device license: controls whether one installation can operate
- pharmacy branch: scopes daily operations and stock

## Portal checklist

The portal overview now derives four ordered steps from the existing authenticated state:

1. portal account — complete when the protected portal has a signed-in user
2. trial — complete when license state is no longer `UNACTIVATED`
3. device — complete when at least one registered device exists
4. activation — complete when at least one registered device is active

When incomplete, the checklist links to the existing trial action or Devices page and explains the minimum recovery sequence. It performs no additional Supabase reads or writes; it consumes the snapshot already loaded by the overview.

## Mobile portal sign-out

The portal sign-out action is no longer hidden at mobile widths. It remains a full-width, keyboard-accessible button below the horizontally scrolling portal navigation.

## Localization

Every string introduced by this chunk uses a semantic key in the existing shared `localization/index.ts` catalog and has authored English and Arabic values. New layouts use logical properties or direction-neutral grids, so the existing root `dir` switch continues to control RTL/LTR behavior.

## Focused verification

Passed:

- `npm test -- tests/phase9b2a-onboarding.spec.ts` — 1 of 1 focused Playwright test passed. It verifies first-run presentation, automatic state completion, skip, persistence across reload, manual resume, checklist completion, Arabic copy, RTL, and dashboard continuation.
- `npm test -- src/test/portal-onboarding.test.tsx` in `website/` — 1 focused test file passed with 3 of 3 tests. It verifies incomplete portal recovery actions, live-state completion, Arabic RTL portal rendering, and mobile sign-out visibility.
- `npx tsc --noEmit --types vite/client` — desktop production-source typecheck passed.
- `npm run lint` — desktop lint passed.
- `npm run build` in `website/` — website typecheck and production bundle passed; 1,634 modules transformed.
- `npm run lint` in `website/` — website lint passed.
- `npx vite build` — desktop production bundle passed; 2,042 modules transformed.
- `git diff --check` — passed; only existing line-ending notices were emitted.

The final scope review covered only the Phase 9B.2A launch set: the shared message catalog, desktop shell hook, desktop onboarding component/styles/test, portal overview hook, portal onboarding component/styles/test, mobile sign-out rule, and this report. No unrelated working-tree file was edited as part of final verification.

The existing root `npm run build` wrapper still has the previously documented missing test-type dependency configuration. The production-source typecheck and production Vite build pass independently; this scoped chunk did not modify unrelated root dependency configuration.

## Scope and data safety

- The real development SQLite database was not accessed or modified.
- No navigation restructuring, empty-state rewrite, branch-scope UX change, or guided tutorial was started.
- No Supabase behavior, payment work, Phase 10 work, or unrelated refactor was performed.
- Existing unrelated working-tree changes were preserved and were not staged or committed.

## Remaining work for Phase 9B.2B

- clarify and simplify navigation hierarchy without changing feature scope
- add localized, action-oriented empty states to core workflows
- improve branch-scope visibility and recovery cues
- add contextual “next step” actions after successful setup and operational tasks
