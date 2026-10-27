# Phase 9A.1 — Early Access Foundation Closure Report

Status date: 2026-10-21 (Africa/Cairo)

## Release artifact

- Installer: `H:\M2-health\tmp\cargo-phase9a\release\bundle\nsis\M² Health_0.4.0-beta.1_x64-setup.exe`
- Version: `0.4.0-beta.1`
- Build ID: `20261020.1` (verified in the production frontend asset used by the bundle)
- Architecture: Windows x64, per-user NSIS installation
- Size: 6,386,168 bytes
- SHA-256: `0C9F6A7F0BA431EFEBECF358A3DB85CD92CCD3E88AE87BA4BE8377DAA5BF4F4A`
- Authenticode: `NotSigned`
- Updater artifacts: disabled

The unsigned status is acceptable for this internal Early Access closure gate, but code signing remains required before public distribution.

## Silent installer smoke

The final installer was exercised with `/S` and an explicit `/D=H:\M2-health\tmp\ea-install-smoke` target. Because the sandbox blocked execution of the unsigned installer, the smoke process was run outside the sandbox while all install files remained inside that isolated workspace path.

- Silent install: PASS. The target directory, `m2-health.exe`, `uninstall.exe`, and the packaged catalog resource were created.
- Silent uninstall: PASS. The target directory, application executable, uninstaller, and packaged resource were removed.
- Data preservation: PASS. A sibling sentinel directory representing data outside `$INSTDIR` remained present throughout uninstall and was removed only by explicit test cleanup afterward.
- Uninstall hardening: the stock NSIS delete could lose a race with a transient executable lock. `src-tauri/windows/nsis-hooks.nsh` now performs a delayed post-uninstall retry and uses Windows reboot cleanup only if a lock remains. The hook targets only the application executable and `$INSTDIR`; it does not target Tauri app-local data.
- Application launch: intentionally skipped. The installer/uninstaller smoke did not start M² Health and therefore could not initialize or migrate a database.
- Real development SQLite: not opened, queried, copied, migrated, seeded, reset, or modified. A metadata-only check after the smoke showed `C:\Users\10\AppData\Local\com.m2health.pharmacy\m2-health.db` unchanged at 67,641,344 bytes and `2026-10-09T13:15:34.0714897Z` UTC, matching the recorded pre-smoke baseline.

## Licensing and trial architecture

- Trial duration is seven days and begins only through the explicit start-trial action.
- Licensing reuses the Phase 8C device UUID; licenses are bound to that device and verified locally with an embedded Ed25519 public key.
- The private signing key remains local-only, is ignored by Git, and is not embedded in the application or installer.
- License states cover active, suspended, revoked, and expired outcomes.
- Expired access becomes read-only: operational writes are blocked at the IPC authorization boundary while reads, backup, diagnostics, and activation remain available.
- Offline operation has a signed three-day grace policy, monotonic-clock rollback defense, and signed revalidation envelope support.
- Cloud synchronization/transport and rollout gates remain disabled.

## Manual activation workflow

1. The desktop displays/copies its existing device ID.
2. An authorized operator uses `scripts/admin/issue-manual-license.mjs` on the controlled issuer machine with the ignored private signing key.
3. The operator transfers only the signed activation payload to the customer.
4. The customer pastes the payload into the activation screen.
5. The desktop validates the signature, device binding, state, and validity window locally before storing the license.

The companion key-generation script is `scripts/admin/generate-license-signing-key.mjs`. A focused issuer smoke verified the generated signature with the public key; no key material was printed. Before public release, the private key must move to an approved protected issuer/vault workflow with operator audit controls.

## Backup and recovery

- Existing backup integrity and schema validation remain in place.
- Restore continues to require explicit confirmation, creates an automatic safety backup, and supports rollback on failure.
- Restore preserves the current device-bound license state and rejects incompatible device/license restoration.
- Backup and recovery remain available in license read-only mode; license expiry does not delete pharmacy or customer data.

## Diagnostics

- Local diagnostics include release version, build ID, schema version, license mode, database integrity status, and sanitized application events.
- The JSON diagnostic export excludes domain rows, credentials, signing material, and raw secrets.
- Diagnostics remain available in read-only/expired mode.

## Verification evidence

Focused verification completed before this closure step:

- `cargo check` with isolated `CARGO_TARGET_DIR`: PASS.
- Rust library tests: 25/25 PASS, including licensing, diagnostics, and desktop backup coverage.
- Authorization integration tests: 22/22 PASS, including expired-license write denial and read/backup availability.
- Focused Playwright: 6/6 PASS across Early Access licensing and desktop-closure workflows.
- Manual license issuer signature verification: PASS (`signatureValid=true`); activation payload removed after the smoke.
- Production fixture/credential scan: PASS; no demo fixture module or credential pattern was present in the production bundle.
- Final build-ID-corrected NSIS bundle: PASS.
- Final silent install/uninstall smoke: PASS.

Final post-edit closure checks are recorded after execution below:

- `cargo fmt --all -- --check`: PASS.
- `cargo check`: not repeated. The build-ID-corrected NSIS release build compiled the Rust application successfully after the last functional Rust change; the only later Rust edit applied rustfmt's requested whitespace to `build.rs`.
- `npm run lint`: PASS after declaring the two issuer scripts' Node globals in their ESLint file scope.
- `VITE_M2_BUILD_ID=20261020.1 npm run build`: PASS. Vite emitted only the existing non-failing chunk-size advisory.
- `git diff --check`: PASS. Git emitted only line-ending conversion notices for existing working-tree files.

The full Rust and Playwright suites were not rerun because the closure edit only changed NSIS uninstall behavior and reporting; the focused suites had already passed and the user explicitly requested no full-suite rerun unless required by a new code change.

## Deferred items and remaining release blockers

No material blocker remains for Phase 9A.1 once the final post-edit checks above pass.

Deferred beyond Phase 9A.1:

- Authenticode code signing and publisher reputation for public distribution.
- Hardened/vaulted production license issuing operations and operator audit process.
- Installer rollout, updater deployment, website, payments, subscriptions, and commercial licensing operations.
- Phase 9A.2 work; it is not started by this report.
