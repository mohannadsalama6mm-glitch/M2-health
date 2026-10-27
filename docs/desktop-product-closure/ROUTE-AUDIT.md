# Desktop Product Closure Gate route audit

Date: 2026-10-20

This audit covers the released route surface in `src/app/AppRoutes.tsx`. Historical Phases 1–8B are not renamed by this gate. The development-only `/design-system` route and the wildcard not-found view are not release workflows.

## Route to data source matrix

| Released route | Desktop implementation and source | Browser behavior | UI permission |
| --- | --- | --- | --- |
| `/` | Redirect only to `/dashboard` | Same redirect | Authenticated shell |
| `/dashboard` | `CanonicalDashboard`; reporting, finance, inventory-alert and branch IPC over SQLite | Clearly labeled fixture-backed UI preview; no storage API | `dashboard.read` |
| `/sales` | `CanonicalSales`; canonical sales, catalog search, customer and branch IPC over SQLite | Explicit desktop-required state; no simulated sale | `sales.read` |
| `/sales/history` | `CanonicalSales` history; canonical sales IPC over SQLite | Not a browser release subroute | `sales.read` |
| `/sales/:id` | `CanonicalSales` detail/return/void/finance trace; SQLite ledger and finance journals | Explicit desktop-required state | `sales.read`; mutations have command-specific IPC policy |
| `/catalog` | `Catalog`; catalog IPC and SQLite | Explicit desktop-required state | `catalog.read` |
| `/catalog/import` | `ImportProducts`; catalog source/import IPC and SQLite | Explicit desktop-required state | `catalog.import` |
| `/catalog/new` | `ProductForm`; catalog create IPC and SQLite | Explicit desktop-required state | `catalog.manage` |
| `/catalog/:id` | `ProductDetails`; catalog detail IPC and SQLite | Explicit desktop-required state | `catalog.read` |
| `/catalog/:id/edit` | `ProductForm`; catalog update IPC and SQLite | Explicit desktop-required state | `catalog.manage` |
| `/inventory` | `Inventory`; stock overview, balances, movements and adjustment IPC over SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/batches` | `InventoryWorkspace`; batch IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/opening` | `InventoryWorkspace`; opening-stock IPC and SQLite ledger | Explicit desktop-required state | `inventory.adjust` |
| `/inventory/levels` | `InventoryWorkspace`; inventory-level IPC and SQLite | Explicit desktop-required state | `inventory.adjust` |
| `/inventory/documents` | `InventoryDocuments`; document IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/documents/:id` | `InventoryDocuments`; document detail/post IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/count-sessions` | `CountSessions`; count-session IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/count-sessions/:id` | `CountSessions`; count entry/review/completion IPC and SQLite ledger | Explicit desktop-required state | `inventory.read`; mutations have command-specific IPC policy |
| `/inventory/movements` | `StockMovements`; movement IPC and immutable SQLite ledger | Explicit desktop-required state | `inventory.read` |
| `/inventory/counts` | `StockCounts`; count IPC and SQLite | Explicit desktop-required state | `inventory.count` |
| `/inventory/expiry` | `InventoryAlerts`; alert IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/alerts` | `InventoryAlerts`; alert IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/low-stock` | `InventoryAlerts`; alert IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/inventory/fefo` | `InventoryAlerts`; FEFO IPC and SQLite | Explicit desktop-required state | `inventory.read` |
| `/purchases` | `CanonicalPurchases`; purchase-order IPC and SQLite | Explicit desktop-required state | `purchasing.read` |
| `/purchases/new` | `CanonicalPurchases` editor; purchase create IPC and SQLite | Explicit desktop-required state | `purchasing.read`; create IPC requires `purchasing.manage` |
| `/purchases/:id` | `CanonicalPurchases` detail; purchase IPC and SQLite | Explicit desktop-required state | `purchasing.read` |
| `/purchases/:id/edit` | `CanonicalPurchases` editor; purchase update IPC and SQLite | Not a browser release subroute | `purchasing.read`; update IPC requires `purchasing.manage` |
| `/purchases/:id/receive` | `CanonicalPurchases` receiving; receipt and inventory-movement IPC over SQLite | Explicit desktop-required state; no false “already received” result | `purchasing.read`; receive IPC requires `purchasing.receive` |
| `/purchases/receipts/:id` | `CanonicalPurchases` receipt detail; receipt IPC and SQLite | Not a browser release subroute | `purchasing.read` |
| `/purchases/receipts/:id/return` | `CanonicalPurchases` supplier return; return and inventory-movement IPC over SQLite | Not a browser release subroute | `purchasing.read`; return IPC requires `purchasing.return` |
| `/purchases/returns` | `CanonicalPurchases` return history; return IPC and SQLite | Not a browser release subroute | `purchasing.read` |
| `/purchases/returns/:id` | `CanonicalPurchases` return detail; return IPC and SQLite | Not a browser release subroute | `purchasing.read` |
| `/suppliers` | `CanonicalPartners`; supplier IPC and SQLite | Explicit desktop-required state | `suppliers.read` |
| `/suppliers/:id` | `CanonicalPartners`; supplier detail/analytics IPC and SQLite | Explicit desktop-required state | `suppliers.read` |
| `/customers` | `CanonicalPartners`; customer IPC and SQLite | Explicit desktop-required state | `customers.read` |
| `/customers/:id` | `CanonicalPartners`; customer detail/history IPC and SQLite | Explicit desktop-required state | `customers.read` |
| `/reports` | `CanonicalReports`; reporting IPC and SQLite | Clearly labeled fixture report preview | `reports.operational.read` |
| `/finance` | `CanonicalFinance`; accounts, expenses, journals and trial-balance IPC over SQLite | Clearly labeled fixture finance preview | `finance.read` |
| `/finance/expenses/:id` | `CanonicalFinance`; immutable expense/journal IPC and SQLite | Preview family only | `finance.read` |
| `/finance/journals/:id` | `CanonicalFinance`; immutable journal IPC and SQLite | Preview family only | `finance.read` |
| `/employees` | `CanonicalEmployees`; user/role/branch membership IPC and SQLite | Clearly labeled fixture preview | `users.read` |
| `/employees/:id` | `CanonicalEmployeeDetails`; identity administration IPC and SQLite | Fixture detail preview | `users.read`; mutations require identity-management IPC permissions |
| `/roles` | `CanonicalRoles`; role IPC and SQLite | Fixture preview | `roles.read` |
| `/branches` | `CanonicalBranches`; branch IPC and SQLite | Fixture preview | `branches.read` |
| `/branches/:id` | `CanonicalBranchDetails`; branch IPC and SQLite | Fixture detail preview | `branches.read`; membership checked at IPC boundary |
| `/notifications` | `CanonicalNotifications`; active inventory-alert IPC for selected branch | Explicit desktop-required state; no simulated persistence | `inventory.read` |
| `/audit-log` | `CanonicalAuditLog`; immutable `security_events` IPC and SQLite | Explicit desktop-required state | `audit.read` |
| `/backup` | `CanonicalBackup`; managed SQLite backup/validation/restore IPC | Explicit desktop-required state | `backup.manage` |
| `/sync` | `CanonicalSync`; local sync status IPC; transport remains gated off | Explicit desktop-required state | Authenticated shell; command policies still apply |
| `/settings` | `CanonicalSettings`; persisted local profile and alert-settings IPC over SQLite | Explicit desktop-required state | `settings.read`; mutation requires `settings.manage` |

## Desktop fixture and browser honesty conclusions

- Desktop routing selects canonical components before any browser preview component for sales, purchasing, partners, dashboard, reports, finance, identity, branches, notifications, audit, backup/restore, sync, and settings.
- Shared catalog, inventory, sales, purchasing, and partner components branch on `isDesktopRuntime()` and show explicit desktop-required states in browser mode; they do not simulate persistence.
- `AppShell` invokes `useDemo()` only inside `BrowserAppShell`; the desktop shell receives the live branch name and never reads fixture product state.
- Browser preview is labeled `Browser UI preview` and `no persistent data`. System routes that could falsely imply durable state are explicit desktop-required screens. Browser preview mutations are ephemeral React state and are never described as local persistence.

## IPC authorization boundary

The registered Tauri invoke surface is intercepted by the centralized authorization plugin. Each non-public command has exactly one explicit policy. The policy resolves a live session principal, permission, selected branch, record-owned branch, organization scope, and actor identity before the domain closure is entered. Focused authorization tests cover missing, invalid and revoked sessions; forged actor fields; branch membership; record ownership; all-branch/export scope; and the new settings, audit and backup commands.

## Migration and database safety

- Working-tree SHA-256 content for migrations `001` through `020` matches `HEAD`; none of those files is modified.
- Migration `021_sync_conflict_resolution.sql` remains the existing predecessor.
- Migration `022_desktop_product_closure.sql` is additive and registered last. It creates the persisted singleton `local_settings` record only.
- `.gitignore` excludes `*.db`, `*.db-wal`, `*.db-shm`, `backups/`, `*.local`, `config/cloud.local.json`, and `supabase/.temp/`.
- Validation used in-memory databases or temporary files under `H:\M2-health\tmp\rust-test-temp`. No real development SQLite database was opened, copied, seeded, migrated, reset, or inspected.

## Backup and restore controls

Restore requires exact `RESTORE` confirmation and a non-empty reason. Before replacement, the selected managed backup is checked for SQLite integrity, foreign-key integrity, sequential recognized migration history, and required schema objects. A safety backup is created first. After restore, migrations, integrity, relationships, schema history, and required objects are revalidated. On failure, the safety backup is reapplied and revalidated; the UI is told that data was retained only if that rollback verification succeeds. If rollback verification also fails, the error preserves and identifies the safety backup for recovery without claiming success.

## Scope boundary

Cloud rollout/transport remains disabled. Phase 8C, website, payments, subscriptions, licensing, installer rollout, updater deployment, and Phase 10 audit are not part of this gate.
