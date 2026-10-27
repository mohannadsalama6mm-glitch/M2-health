# Phase 9B.1 — Localization Foundation + Truthful Messaging

Date: 2026-10-21

## Result

Phase 9B.1 is complete for the agreed first localization tier. The desktop application and website now share one English/Arabic message catalog, use semantic keys for visible state labels, persist the selected language, and switch the document language and writing direction at runtime. The implementation did not change pharmacy business rules, licensing rules, Supabase behavior, payments, or SQLite data.

## Localization infrastructure

- Shared source: `localization/index.ts`.
- Supported locales: `en` and `ar`.
- Runtime direction: English uses `ltr`; Arabic uses `rtl`.
- Persistence key: `m2-health.locale.v1` in origin-local browser storage.
- Both application roots update `<html lang>` and `<html dir>` whenever the locale changes.
- Desktop and website providers expose translation, number, currency-minor, date, date/time, and semantic-state formatters.
- The desktop top bar, desktop Settings language card, public website header, and portal shell contain working language selectors.
- Locale resources use stable semantic keys. Backend enum values are mapped once rather than translated ad hoc in individual views.

## Terminology and semantic states

The shared resources define consistent English and Arabic terms for portal account, local administrator, pharmacy branch, device, license, trial, activation, and subscription.

The shared semantic-state formatter covers trial, active, expired, suspended, revoked, pending, fulfilled, unactivated, offline grace, read-only, cancelled, approved, and denied states. Unknown values use an explicit localized fallback instead of displaying raw enum formatting as a translation.

## Translated first-tier surfaces

### Desktop

- local administrator setup and sign-in
- license/trial gate
- application shell, grouped navigation, top bar, profile menu, footer, offline status, and browser-mode warning
- dashboard shell, filters, operational/finance summaries, numeric and currency output, and coverage status
- Settings language area
- shared buttons and infrastructure states in the design system, including confirmation actions, empty tables, pagination, tabs, modal close actions, toast dismissal, loading states, and breadcrumbs

### Website and portal

- public shell, primary navigation, footer, language selector, and landing-page content
- registration, sign-in, forgot-password, and reset-password pages
- portal shell, account menu, sign-out action, loading/error shell, overview metrics, trial action, release summary, and license timeline
- semantic status badges

The Arabic translations in this tier are authored Arabic copy, not transliterated English or raw backend values. RTL layout adjustments cover the desktop shell and the public, auth, and portal shells without changing the established visual design.

## Truthful trial messaging

Website links that navigate to registration now say “Create Early Access account” / “إنشاء حساب وصول مبكر”; they no longer imply that registration starts a trial. Registration also states that creating an account does not start the trial. “Start 7-day trial” is reserved for the authenticated portal action that actually invokes trial start, and it retains explicit confirmation.

## Locale-aware formatting

- Dates and date/time values use `Intl.DateTimeFormat` with `en-GB` or `ar-EG`.
- Numbers use `Intl.NumberFormat` with the active locale.
- Existing minor-unit currency values use locale-aware formatting and EGP presentation.
- Direction-dependent pagination, breadcrumb, and tab keyboard behavior follows the active locale.

## Focused verification

Passed:

- `npm test -- tests/phase9b1-localization.spec.ts` — 1 Playwright test passed; proves desktop selector switching, persistence across reload, root `lang`/`dir`, Arabic shell/dashboard navigation, and return to English LTR.
- `npm test` in `website/` — 7 files and 17 tests passed. The new focused coverage proves website switching, persistence, root `lang`/`dir`, translated high-visibility content, semantic state labels, and truthful trial CTAs in both languages.
- `npm run lint` — desktop lint passed.
- `npm run lint` in `website/` — website lint passed.
- `npx tsc --noEmit --types vite/client` — desktop production-source typecheck passed.
- `npx vite build` — desktop production bundle passed (2,041 modules).
- `npm run build` in `website/` — website typecheck and production bundle passed (1,633 modules).
- `git diff --check` — passed; only existing line-ending notices were emitted.

Known verification distinction:

- The root `npm run build` wrapper stops before bundling because the already-modified root TypeScript configuration names `@testing-library/jest-dom`, `node`, and `vitest/globals`, while those type packages are not installed in the root package. This is not caused by the localization source: the production-source typecheck and direct Vite production build both pass. Dependency/TypeScript test-environment cleanup is intentionally left out of this scoped chunk.

## Remaining hardcoded high-priority strings

The first-tier surfaces above are localized. The remaining high-priority inventory is concentrated in route bodies that were outside this chunk’s “highest visibility first” boundary:

- desktop catalog, purchasing, inventory, sales/returns, partners, reports/finance, and system-management workflow bodies
- the remainder of the desktop Settings forms beyond the language card
- website Features/Pricing/Download/FAQ/Support secondary page bodies
- portal Devices, Downloads, Subscription, and Account page bodies
- server-originated error messages that do not yet carry stable error codes suitable for client localization

These strings should move to the same shared catalog in focused route batches; no second localization system should be introduced.

## Safety and scope confirmation

- No SQLite command, migration, seed, copy, reset, or development database access was performed.
- No business logic, licensing rule, Supabase behavior, payment architecture, or rollout gate was changed.
- No onboarding wizard, guided tutorial, Phase 10 work, or unrelated refactor was started.
- Existing unrelated working-tree changes were preserved and were not staged or committed.

## Next recommended chunk

Phase 9B.2 — Onboarding + Empty States + Navigation. Continue using the shared localization catalog while implementing that chunk, and translate each touched route body rather than embedding new user-facing literals.
