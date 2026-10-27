# M² Health Early Access Release Candidate Report

Date: 2026-10-22 (Africa/Cairo)
Status: **READY**

## Release identity

- Version: `0.4.0-beta.2`
- Build ID: `20261022.1`
- Channel: Early Access / Beta
- Platform: Windows x64
- Installer: `M2-Health_0.4.0-beta.2_x64-setup.exe`
- Build artifact: `H:\M2-health\tmp\cargo-phase9b-rc\release\bundle\nsis\M² Health_0.4.0-beta.2_x64-setup.exe`
- Published artifact: `H:\M2-health\website\public\releases\0.4.0-beta.2\M2-Health_0.4.0-beta.2_x64-setup.exe`
- Size: 6,411,438 bytes
- SHA-256: `3EF9F2190B64B76011F1A0312D411158224FF1511D86AFAA726C7B24FDB30D7A`
- Authenticode: `NotSigned`
- Updater artifacts: disabled

The beta.2 version follows the existing Early Access prerelease sequence and contains the completed Phase 9B localization, onboarding, setup checklist, navigation, empty-state, branch-scope, contextual guidance, tutorial, and visual-polish work.

## Version and release metadata

The active release references were updated consistently in:

- root npm package metadata and lockfile
- Rust package metadata and lockfile
- Tauri application metadata
- website package metadata and lockfile
- website release/download metadata
- portal license-envelope version/build metadata
- public release index and release notes
- focused website release and portal API tests

The prior `0.4.0-beta.1` release remains available only as an immutable historical artifact. The active website download flow and `current` release pointer select beta.2.

## Build result

`npm run tauri -- build --bundles nsis` completed with an isolated `CARGO_TARGET_DIR` at `H:\M2-health\tmp\cargo-phase9b-rc`, `CARGO_BUILD_JOBS=2`, `M2_BUILD_ID=20261022.1`, and `VITE_M2_BUILD_ID=20261022.1`.

- Desktop TypeScript/Vite production build: PASS, 2,044 modules transformed.
- Rust release compilation: PASS.
- Windows x64 NSIS packaging: PASS.
- First sandboxed packaging attempt: application compilation passed, but the NSIS tool download failed with DNS error `No such host is known`.
- Conservative retry outside the restricted network sandbox: PASS; Tauri downloaded its official NSIS 3.11 package and produced the installer.
- Existing Vite chunk-size advisory: non-failing and unchanged; no unrelated code-splitting refactor was introduced.

## Isolated installation and data safety

The installer smoke ran under a disposable local Windows account named `M2HealthRCTest`, created specifically so Tauri resolved application data under that account's profile rather than the project owner's profile.

- Upgrade install root: `H:\M2-health\tmp\phase9b-rc-smoke\upgrade-install`
- Clean install root: `H:\M2-health\tmp\phase9b-rc-smoke\clean-install`
- Isolated runtime data root: `C:\Users\M2HealthRCTest\AppData\Local\com.m2health.pharmacy`
- Upgrade uninstall: PASS; the application executable and uninstaller were removed.
- Clean uninstall: PASS; the application executable and uninstaller were removed.
- Temporary account/profile cleanup: PASS; `C:\Users\M2HealthRCTest` no longer exists.
- Customer/user data safety: only the disposable profile's test data was removed between the upgrade and clean-install cycles. No existing customer or project-owner profile data was targeted.

The real development SQLite database was not opened, queried, copied, migrated, seeded, reset, or modified. A metadata-only post-smoke check remained exactly at the recorded pre-smoke baseline: 67,641,344 bytes and `2026-10-09T13:15:34.0714897Z` UTC.

## Upgrade smoke

Result: **PASS**

1. The existing beta.1 installer installed into the isolated upgrade path.
2. Beta.1 launched successfully and initialized only the disposable profile.
3. Beta.2 installed over the same isolated application path without downgrade or startup failure.
4. The upgraded beta.2 application launched successfully against the isolated data created by beta.1.
5. The beta.2 localized local-administrator screen rendered after upgrade.
6. Silent uninstall completed and removed the application files.

## Clean-install product smoke

Result: **PASS**

- Clean beta.2 install and launch: PASS.
- Release identity on the license gate: PASS (`0.4.0-beta.2`, build `20261022.1`).
- Trial/license gate rendering: PASS; the explicit seven-day trial action was shown and started successfully with synthetic isolated data.
- Local administrator creation: PASS.
- First-run onboarding: PASS; the state-aware checklist opened with local administrator, device license, language, and first branch detected as complete.
- Onboarding skip: PASS.
- Onboarding reopen from the account menu: PASS.
- Arabic switch and RTL layout: PASS.
- Language persistence: PASS; a full WebView reload retained Arabic and presented the localized Arabic sign-in screen.
- English switch and LTR layout: PASS.
- Guided tutorial first-run presentation: PASS.
- Guided tutorial skip: PASS.
- Guided tutorial restart from the account menu: PASS.
- Branch selector and grouped navigation: PASS; the active operational branch was visibly `Main Pharmacy` and all route groups remained available.
- Dashboard: PASS in Arabic and English; localized empty-state guidance, branch/report scope, semantic trial state, numbers, and currency rendered.
- Startup/runtime stability: PASS; no obvious startup error, crash, or blocking runtime error appeared during the smoke.

## Focused verification

- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`: PASS.
- Root `npm run lint`: PASS.
- Desktop production build inside the Tauri build: PASS.
- Rust release compilation inside the Tauri build: PASS.
- Website `npm run lint`: PASS.
- Website `npm run build`: PASS, 1,634 modules transformed.
- Website `npm test`: PASS outside the restricted Windows sandbox, 8 files and 21/21 tests. The first sandboxed run executed no tests because Vitest hit an `EPERM` temp-cache rename restriction; this was an environment failure, not a test failure.
- Release artifact integrity test: PASS; published file length and SHA-256 exactly match release metadata.
- `git diff --check`: PASS; output contained only existing line-ending conversion warnings.

## Production website and download

- Project: `m2-f89c/m2-health`
- Deployment: `dpl_DGCiKnYZq8Bu6MvVwdAjhn4VB3hx`
- Deployment state: `READY`
- Production URL: `https://m2-health.vercel.app`
- Active release index: `0.4.0-beta.2` / build `20261022.1`
- Production Download page: HTTP 200 and visibly shows beta.2, the current build ID, the exact checksum, and the beta.2 installer link.
- Direct installer response: PASS; 6,411,438 bytes downloaded.
- Downloaded production SHA-256: `3EF9F2190B64B76011F1A0312D411158224FF1511D86AFAA726C7B24FDB30D7A`.
- Direct HTTP 200 routes: `/`, `/login`, `/register`, `/portal`, `/download`, `/features`, `/pricing`, `/faq`, and `/support`.

## Scope protections

- No pharmacy business logic changed.
- No licensing rule changed.
- No Supabase backend behavior changed.
- No payment work or Phase 10 work started.
- Cloud synchronization transport and rollout gates remain disabled.
- Existing unrelated working-tree changes were preserved.

## Final recommendation

The beta.2 installer, isolated install/upgrade behavior, bilingual desktop experience, published release metadata, production download, and checksum evidence all pass. No material blocker remains before first real Early Access user distribution, subject to the already-disclosed unsigned Early Access status.
