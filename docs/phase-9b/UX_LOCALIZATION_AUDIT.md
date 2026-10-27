# Phase 9B — Product Experience & Localization Audit

**Audit date:** 2026-10-21
**Repository baseline:** `5e6b1e11fc9d38b5439954667a97512b5527f0dc` (`main`)
**Production website:** <https://m2-health.vercel.app>
**Outcome:** Audit complete; implementation has not started.

## Scope and evidence

This audit covers the released desktop route tree, first-launch licensing and local authentication gates, the shared desktop shell, the principal pharmacy workflows, and the production public/auth/portal website. It is based on:

- source review of the route map, shell, gates, shared components, canonical desktop pages, website pages, and CSS;
- the existing desktop route audit and current desktop screenshots;
- read-only live review of the production homepage, registration, login, download, and unauthenticated portal redirect on 2026-10-21;
- source review of authenticated portal states where no customer credential was used.

No business logic, licensing logic, Supabase behavior, payment architecture, core feature scope, or SQLite data was changed or exercised. No real development SQLite database was accessed. This report is the only file created by this audit.

### Severity

- **Critical:** likely to prevent or materially mislead a first-time user completing setup or starting safe operation.
- **High:** creates frequent uncertainty, error risk, or an avoidable support burden in a core workflow.
- **Medium:** reduces consistency, speed, accessibility, or polish but has a workable path today.

### End-to-end journey findings

| Journey stage | Current experience | Why a first-time user may feel lost | Required UX response |
|---|---|---|---|
| Website entry | Public pages explain local-first operation and Early Access well. “Start 7-day free trial” links to registration, although registration explicitly does not start the trial. | The CTA promises an action that the destination says has not happened. | Rename the CTA to “Create Early Access account” or add an explicit multi-step progress view. |
| Download/install | Version, build ID, checksum, unsigned warning, requirements, and backup advice are visible. | The complete order—account, download, device ID, trial, activation, local admin—is split across pages. | Add one canonical numbered getting-started path and reuse it on Download and Portal Overview. |
| Desktop activation | The license gate offers local trial start or a manual activation-code field and shows the device ID. | It does not link to the portal, explain the online account, or distinguish portal credentials from the later local administrator. | Explain the two identities and provide a copy-device-ID action plus portal/support link. |
| Local administrator | A secure local administrator is created after licensing. | The user may try website credentials here; after creation the app drops into a large workspace without setup guidance. | State “This is a local app account, separate from your portal account,” then continue into first-run setup. |
| Branch setup | Branches are a management page and the shell chooses an active/primary branch. | There is no guided confirmation that the selected branch is correct before stock-affecting work. The branch selector and page-specific branch filters use different meanings. | Require branch confirmation in first-run; label operational scope versus report filter scope. |
| Catalog | Add and import actions exist. The product form supports English/Arabic names and packages. | An empty catalog renders an empty table rather than a task-oriented choice. The product form is long and assumes pharmacy catalog knowledge. | Add an actionable empty state and progressive sections with examples and field help. |
| Purchasing | Draft, order, receipt, and supplier-return boundaries are explicit and confirmations are strong. | Creating a purchase presumes a supplier and catalog package already exist; the dependency is not surfaced until the user reaches the form. | Show prerequisites and direct links; after each state transition show the next valid action. |
| Inventory | Inventory has overview, movements, documents, counts, expiry, alerts, low stock, FEFO, and opening stock. | Nine peer buttons plus a duplicate top-level Expiry destination overwhelm new users; raw ledger terms and status tokens dominate. | Consolidate into task-oriented tabs and add plain-language help for ledger concepts. |
| Sales/returns | POS explains that drafts/held sales do not reserve stock; post and return confirmations exist. | “POS / Sales,” “New Sale,” “Sales History / Held,” and “Payment / Post” mix terminology. Empty inventory does not lead to receiving/opening stock. | Standardize terms and add prerequisite recovery actions from zero-stock states. |
| Stock counts | Session/count workflows exist and explain that stock changes only on completion. | Counts are buried under Inventory; users must understand the difference between legacy `/inventory/counts` and count sessions. | Present one canonical “Stock counts” entry and explain draft → enter → review → complete. |
| Dashboard/reports | Operational and finance summaries are present with filters and trace links. | A new pharmacy sees zeros without a readiness checklist. Reports mix tabs, selects, filters, and raw codes/timestamps. | Replace zero-only landing with setup progress; standardize report navigation and empty states. |
| Settings/backup | Canonical settings persist pharmacy identity and monitoring; backup/restore safety language is strong. | Language icon opens Settings with `?section=Language`, but canonical Settings has no language section and ignores the query. Backup is not part of setup completion. | Remove the misleading affordance until functional; include verified backup in readiness. |
| Portal/device | Portal separates overview, devices, downloads, subscription, and account. | Activation is a manual UUID/code loop with no persistent checklist. Portal error states have no retry. Mobile CSS hides the only sign-out control. | Add a resumable checklist, inline cross-links, retry/support actions, and mobile sign-out. |

## 1. Critical UX issues

### C1. There is no unified first-run journey

The current sequence is license gate → local administrator → dashboard. It does not collect or confirm pharmacy identity, operational branch, catalog bootstrap method, first stock source, or recovery readiness. A new user can reach an empty dashboard and sixteen-item sidebar without knowing the minimum safe setup order.

**Impact:** activation can succeed while the product still feels unusable; users may enter opening stock or attempt a sale before understanding branches, products, and receiving.

**Required before broad Early Access:** a small, resumable readiness flow and a persistent “Getting started” checklist. This is workflow guidance, not a change to domain behavior.

### C2. Online account, device license, and local user identities are not connected in the UX

M² Health correctly has separate security domains, but the interface does not teach them. The website creates an email/password portal account; the desktop later requests a local username/password; device activation moves a UUID and signed code between them. The desktop activation screen has no portal link or explanation of the online account, and the local-admin screen does not warn that portal credentials will not work there.

**Impact:** credential confusion, duplicate password-reset requests, abandoned activation, and support tickets.

**Required:** consistent copy and a shared step indicator on website and desktop. Never merge the credentials technically; explain their purpose clearly.

### C3. The language affordance is misleading and localization infrastructure is absent

The desktop top bar exposes a language icon that navigates to `/settings?section=Language`. The canonical Settings route does not implement a language section or consume that query. The shipped HTML roots are `lang="en"`; no runtime locale provider, resource catalog, translation function, or direction switch was found. Almost all user-facing copy is embedded in TSX.

**Impact:** users reasonably infer Arabic can be selected, but it cannot. This is worse than an explicit English-only beta label.

**Required:** either implement the Phase 9B locale foundation and a real language selector, or hide the language action and state “English interface in this Early Access build” until Arabic is ready.

### C4. Arabic and RTL are not release-ready

There are useful foundations: IBM Plex Sans Arabic is bundled, many desktop CSS rules use logical properties, product data has an Arabic name field, search copy acknowledges Arabic, and the design-system showcase has an RTL preview. These do not constitute runtime RTL support. The application root never changes `lang` or `dir`; navigation order, portal layout, physical arrows, pagination keys, website ribbons/floating elements, and several left/right CSS rules remain direction-specific.

**Impact:** an Arabic translation would render with mixed direction, unmirrored navigation cues, English-formatted numbers/dates, and likely table/form defects.

**Required:** do not advertise an Arabic interface until root direction switching and an RTL acceptance matrix pass on every core flow.

## 2. High-priority UX issues

### H1. Core empty states do not consistently advance the workflow

- Catalog shows an empty table and result count, not “Add product” versus “Import catalog.”
- Dashboard and report zero states are data summaries, not readiness guidance.
- Finance shows “No posted records match these filters” without a contextual next step.
- POS says “Add a package to begin,” but a zero-stock/no-package state does not link to Catalog, Opening Stock, or Purchasing.
- Branches, backups, purchase history, and some return states have descriptive empty components, but many omit an action.

Every empty state should answer: what is empty, why it matters, what prerequisite is missing, and the safest next action the current role may take.

### H2. Navigation presents the data model instead of the user’s jobs

The desktop exposes sixteen sidebar destinations. Inventory then presents nine peer workflow buttons. Expiry appears both in the sidebar and inventory navigation. Technical destinations such as Audit log and Sync status have the same visual weight as daily selling and receiving.

The result is discoverable in the literal sense but not learnable. A cashier, inventory operator, and owner should see task emphasis appropriate to their work, while preserving the existing permissions and URLs.

### H3. Page hierarchy and action placement are inconsistent

Canonical pages use a mixture of `stack` and `feature-page`, header actions, inline button rows, link-style tabs, selects acting as report navigation, and direct hash assignments. Loading appears as plain text, a card, or an alert. Empty states appear inside or outside cards. Primary actions sometimes use `primary`, sometimes the default button, and sometimes a text link.

This makes users rescan each page to learn where actions and filters live. It also raises localization cost because layout behavior is not predictable when labels expand.

### H4. Branch scope is visible but semantically inconsistent

The top bar has an operational branch selector; Inventory repeats a branch selector; Dashboard can show “All branches”; reports and finance have their own filters; the status bar repeats the active branch. The UI does not consistently distinguish “this action will post to branch X” from “this report is filtered to branch X.”

Stock-affecting review dialogs are generally explicit, which is good. The missing piece is a standardized branch-scope banner/field and a warning when switching scope while a draft form contains unsaved input.

### H5. The global search control advertises a disabled feature

In desktop mode the top-bar input is disabled and says “Use the search in this workspace,” but still displays the `Ctrl K` key hint and installs the keyboard handler. Pressing the shortcut focuses a disabled control and produces no useful outcome.

Hide the control/shortcut until search works, or turn it into a command/search launcher that routes to the active workspace’s search.

### H6. Statuses, dates, money, and domain terms leak raw implementation values

Examples include `IN_STOCK`, `PURCHASE_RECEIPT`, `STOCK_COUNT_CORRECTION`, raw timestamps, string-built plurals, hardcoded `EGP`, and English matching inside status presentation. These are hard to read even in English and cannot be translated safely by replacing visible strings.

Use semantic status keys and formatters based on the selected locale and configured time zone. Preserve immutable codes/IDs in secondary detail where auditing requires them.

### H7. The website’s trial CTA and actual action disagree

The homepage and pricing page say “Start 7-day free trial,” but the registration page correctly says account creation does not start the trial. The real trial starts later in the portal (and the desktop also exposes a local trial action). The product should use “Create account” before authentication and reserve “Start trial” for the irreversible confirmation that starts the clock.

### H8. The portal lacks a persistent activation checklist and recovery actions

Portal Overview summarizes state but does not show the complete ordered activation path. Devices asks for a UUID and emits a code, but it does not anchor the user back to the desktop activation screen. Portal data errors show the error but no retry or support link. Registration confirmation lacks resend/change-email actions, and reset success lacks a direct sign-in link.

### H9. Mobile portal navigation removes sign-out

At the mobile breakpoint, `.sidebar-signout` is hidden and no replacement sign-out action is visible in the reviewed source. This strands authenticated mobile users unless they manipulate browser state or return to a larger viewport.

### H10. Long operational forms need progressive guidance

Product creation, purchase entry, receiving, returns, inventory documents, and counts contain dense domain forms. Labels are generally explicit, but prerequisites, examples, required-versus-optional grouping, and “what happens when I post?” explanations are uneven. Product creation in particular combines identity, clinical classification, ingredients, packages, pricing, barcodes, and notes on one long page.

Use progressive sections, completion summaries, and concise field help. Do not hide safety-critical review information.

## 3. Medium-priority UX issues

1. Page titles use inconsistent naming/capitalization: “Point of Sale,” “POS / Sales,” “Sales History,” “Reports & Analytics,” and “Finance.”
2. Several actions use technical verbs such as “post,” “allocation,” “canonical,” “revision,” and “envelope” without a plain-language explanation near first use.
3. Reports and Finance rely on dense tables and raw dates; useful cross-links exist, but hierarchy between overview, filter, table, and export is weak.
4. Success messages often confirm what happened but do not offer the next likely action.
5. The desktop has no persistent Help/Getting Started entry or searchable glossary.
6. Icon direction is not abstracted: back links and pagination use physical left/right chevrons, and keyboard arrow behavior is LTR-specific.
7. Table overflow is handled mechanically, but wide purchase, POS, report, and inventory tables will be difficult with longer Arabic labels.
8. Website and desktop use different visual systems and terminology for the same license states; they should share a copy glossary even if they do not share components.
9. Portal status labels derived from server enums are rendered mechanically (`replaceAll("_", " ")`) rather than through a localized state map.
10. The website has one global page title; route-specific document titles and descriptions are not evident, reducing orientation and accessibility.
11. Destructive buttons are usually confirmed, but the visual relationship between cancel, void, revoke, discontinue, and return should be standardized.
12. The current branch appears multiple times in the shell, consuming space without clarifying action scope.

## 4. First-run onboarding proposal

Create a resumable first-run experience with two connected parts:

### Website onboarding

Show a persistent checklist after account creation:

1. Verify email.
2. Download and verify the current Windows build.
3. Start the trial when ready.
4. Open the desktop app and copy its device ID.
5. Register/name the device and copy the signed activation code.
6. Paste the code into the desktop and continue local setup.

Each step should show state, a single primary action, and a short explanation of what remains local versus online.

### Desktop onboarding

After licensing and local administrator creation, route into a local setup checklist. It must be resumable, dismissible after a clear warning, stored locally, and never seed demo pharmacy data. Existing permission checks and domain commands remain authoritative.

Completion should mean the pharmacy has a confirmed identity, an operational branch, a chosen catalog/stock bootstrap path, and a verified backup. It should not imply sales or inventory data exist.

## 5. Suggested onboarding wizard steps

1. **Language and direction** — choose English or Arabic only when both are actually available; set root `lang` and `dir` immediately.
2. **Understand the accounts** — explain portal account, device license, and local administrator in one diagram/copy block.
3. **Pharmacy profile** — name, phone, address, registration number, receipt header/footer, time zone, currency display.
4. **Operational branch** — confirm the existing primary branch or create the first branch; show its code and stock scope.
5. **Catalog start** — choose “Add first product” or “Import catalog”; show requirements and a safe import preview.
6. **Stock start** — choose “Create purchase order and receive” or “Record verified opening stock”; explain when each is appropriate.
7. **Team and permissions** — optional during first run; add users/roles or defer while keeping the first administrator.
8. **Backup and recovery** — create and validate the first backup; explain its managed location and restore safety backup.
9. **Readiness review** — display completed/missing prerequisites and role-aware links to resolve them.
10. **Start work** — choose the first destination: New sale, Purchasing, Inventory, or Dashboard.

The wizard should never silently create financial or stock transactions. Steps that call existing commands must retain current confirmation and authorization boundaries.

## 6. Guided tutorial proposal

Use a lightweight, opt-in tutorial plus a persistent checklist rather than a blocking product tour.

- **Owner/admin path:** branch scope → users/roles → pharmacy settings → backup → dashboard.
- **Inventory path:** catalog → supplier → purchase draft → order → receive → inventory ledger → stock count.
- **Cashier path:** branch confirmation → scan/search → cart → customer/walk-in → payment preview → post → receipt → return lookup.
- **Recovery path:** notifications → backup list → validation → restore safety explanation (no restore performed by the tutorial).

Tutorial behavior:

- highlight only one control at a time and allow “Skip,” “Back,” and “Show me later”;
- never place a tooltip over a confirmation dialog or change a transactional default;
- derive available steps from the current role/permissions;
- mark a step complete from observed navigation/success state, not by executing hidden actions;
- provide “Getting started” in Help/Profile so it can be replayed;
- localize tutorial copy through the same message catalog as the application.

## 7. Navigation restructuring recommendations

Preserve routes and authorization, but reorganize presentation around frequency and task:

1. **Home** — Dashboard plus setup/readiness.
2. **Sell** — New sale, Sales history, Returns.
3. **Buy** — Purchases, Receiving, Supplier returns, Suppliers.
4. **Stock** — Overview, Counts, Adjustments/documents, Expiry/alerts, Movements.
5. **Catalog & relationships** — Products, Customers, Suppliers where appropriate.
6. **Insights** — Reports, Finance.
7. **Administration** — Branches, Team/roles, Settings, Notifications, Audit, Backup, Sync status.

Specific changes:

- keep daily tasks expanded and administration collapsed or visually secondary;
- remove the duplicate top-level Expiry entry or make it a notification shortcut rather than a second information architecture;
- replace nine inventory buttons with 4–5 grouped tabs and a “More” menu for specialist views;
- provide breadcrumbs on detail/edit routes and preserve the user’s filters when returning;
- use one canonical route and label for Stock Counts;
- make the current posting branch a consistent contextual control, not another navigation destination.

## 8. Desktop page layout/UI consistency issues

Adopt one page anatomy for released routes:

1. breadcrumb/back context;
2. page title, plain-language description, scope badge, and one primary action;
3. local workflow tabs;
4. filters/search with an explicit applied state;
5. summary/readiness strip;
6. content table/form/cards;
7. actionable loading, empty, error, and success state;
8. sticky save/review actions for long forms.

Current inconsistencies to resolve:

- wrappers and maximum widths differ (`stack`, `feature-page`, bespoke workspaces);
- reports use button/select hybrids while inventory uses link buttons and other pages use route tabs;
- loading indicators alternate among plain paragraphs, alerts, and cards;
- empty states sometimes have no action and Catalog has no true zero-data state;
- filter reset/apply behavior differs by page, and some filters apply immediately while Dashboard requires Apply;
- direct `window.location.hash` assignments in canonical purchasing are inconsistent with router links/navigation;
- success states rarely promote the next valid workflow step;
- raw identifiers/status codes compete with user labels instead of appearing as secondary audit detail;
- tables/forms are optimized for compact English strings, not Arabic expansion;
- branch scope is repeated but not standardized as operational versus analytical scope.

## 9. Website UX issues

### What is working

- Production routes reviewed load coherently, including direct registration, login, and download pages.
- The public site clearly states local-first operation, disabled payments, gated sync, and unsigned installer status.
- Download exposes version, build ID, platform, checksum, requirements, and backup guidance.
- Registration clearly states that account creation does not consume trial time.
- Protected portal navigation preserves the requested portal path through login in source.

### Issues

1. “Start trial” CTAs lead to account creation, not the trial-start confirmation.
2. There is no single numbered onboarding path across registration, download, trial, device registration, activation, and desktop setup.
3. Registration confirmation has no resend, change-email, or support action.
4. Reset-password success has no direct sign-in link.
5. Portal loading/error states have no retry or support path.
6. Devices and Downloads are separate destinations without contextual “next step” links.
7. The manual UUID/code transfer is not supported by a copy button for device ID on desktop or a scannable/structured handoff.
8. Mobile portal CSS hides sign-out without an alternative.
9. No language switch, Arabic copy, runtime `dir`, or route-localized metadata exists.
10. Public navigation remains visible on auth screens and competes with the focused account task.
11. Account/device/license terminology is accurate but dense for a nontechnical pharmacy owner.
12. Authenticated portal screens were source-reviewed, not exercised with a real customer account in this audit; this is a validation note, not an audit blocker.

## 10. Arabic/English localization readiness

### English

English is operationally complete enough to audit and use, but copy governance is weak. Terms, capitalization, status labels, and action verbs need a shared glossary before translation begins. English should be normalized first so Arabic does not preserve contradictions.

### Arabic

Arabic domain-data support is partial: product Arabic names and Arabic search are represented, and the desktop font supports Arabic. Interface localization is **not ready**. There are no Arabic resources, no runtime locale state, no translation fallback policy, no plural rules, and no acceptance tests for Arabic workflows.

### Required localization architecture

- introduce namespaced resources for `common`, `navigation`, `auth`, `licensing`, `onboarding`, `catalog`, `purchasing`, `inventory`, `sales`, `reports`, `finance`, `settings`, `backup`, and `portal`;
- use stable message keys rather than English status strings as logic inputs;
- add interpolation/plural support and prohibit sentence concatenation;
- format dates, times, numbers, percentages, and EGP through `Intl` with an explicit locale/time-zone policy;
- keep document numbers, UUIDs, barcodes, hashes, and technical IDs LTR inside bidirectional isolation;
- define Arabic terminology with a pharmacy-domain reviewer, including sale/return, batch/lot, receipt, stock count, adjustment, expiry, supplier, and journal;
- add a missing-key check and extraction/lint rule in CI;
- localize the website and desktop from the same glossary, even if resource bundles remain separate.

## 11. RTL/LTR issues

### Positive foundations

- desktop tokens use IBM Plex Sans Arabic;
- many desktop layout rules use `padding-inline`, `margin-inline`, `inset-inline-start/end`, and `text-align: start`;
- the design-system showcase can preview `dir="rtl"`;
- the Switch component has an RTL-specific rule.

### Gaps to fix

1. Both HTML roots are permanently `lang="en"`; runtime code does not set `document.documentElement.lang` or `dir`.
2. The desktop shell uses a fixed first-column sidebar. Confirm whether Arabic places navigation on the right; implement deliberately rather than relying on DOM reversal.
3. `ArrowLeft`, `ArrowRight`, `ChevronLeft`, `ChevronRight`, and panel-left icons do not mirror semantically.
4. Pagination keyboard behavior explicitly maps physical ArrowRight/ArrowLeft.
5. Website CSS uses physical `left`, `right`, `margin-left`, and `margin-right` for cards, ribbons, mobile navigation, and check marks.
6. Website portal grids/sidebar order are fixed for LTR.
7. Mixed Arabic names with barcodes, UUIDs, prices, document IDs, and English scientific names need `bdi`/direction isolation.
8. Tables need RTL header/cell alignment tests and horizontal-scroll origin checks.
9. Numeric inputs, dates, currency, and hashes must remain legible and copyable under RTL.
10. Charts, progress steps, breadcrumbs, back controls, toast placement, modal actions, and print/receipt output need mirrored acceptance tests.

Minimum RTL matrix: 1366×768 and 1440×900 desktop views for every critical workflow, plus mobile and desktop website widths, with Arabic expansion and mixed-direction sample data.

## 12. Hardcoded text inventory

No localization resource catalog or translation runtime was found. A source-level heuristic scan found **1,264 candidate literal matches across 50 of 54 desktop TSX files** (excluding mock and design-system showcase folders) and **183 matches across 7 production website TSX files** (tests excluded). These are sizing signals, not exact translation-unit counts: some matches are internal text and dynamic/backend messages are not fully captured.

| Surface | Principal files/areas | Representative hardcoded content | Localization action |
|---|---|---|---|
| Shell/navigation | `src/app/navigation.ts`, `src/app/AppShell.tsx`, `src/app/AppRoutes.tsx` | groups, route labels, tooltips, branch/session/status bar, access denied, not found | Move to `navigation`/`common`; remove English-based status logic. |
| License/local auth | `src/features/licensing/LicenseGate.tsx`, `src/features/auth/AuthGate.tsx` | activation reasons, trial CTA, local-admin explanation, validation/errors | `licensing` and `auth`; explicitly distinguish identities. |
| Dashboard | `src/features/dashboard/CanonicalDashboard.tsx` | metrics, coverage messages, filters, EGP strings | `dashboard`; locale-aware metrics and plurals. |
| Catalog | `Catalog.tsx`, `ProductForm.tsx`, `ProductDetails.tsx`, `ImportProducts.tsx` | fields, package/barcode labels, import stages, empty/errors | `catalog`; preserve product data language separately from UI locale. |
| Purchasing | `CanonicalPurchases.tsx` | draft/order/receive/return states, confirmation copy, table headers | `purchasing`; map domain enums to messages. |
| Inventory | `InventoryWorkspace.tsx`, `Inventory.tsx`, `Documents.tsx`, `CountSessions.tsx`, `StockCounts.tsx`, alerts/expiry pages | nine workflow labels, movements, documents, count guidance, stock states | `inventory`; normalize duplicate/legacy route vocabulary first. |
| Sales/returns | `CanonicalSales.tsx` | POS search, cart, payment/post, held/draft, returns/voids | `sales`; standardize sale/post/payment terminology. |
| Partners/team/branches | canonical partners/people/branches files | customer/supplier fields, roles, permissions, branch labels | `partners`, `team`, `branches`; role-aware empty actions. |
| Reports/finance | `CanonicalReports.tsx`, `CanonicalFinance.tsx` | report names, metrics, filters, raw timestamps/statuses, EGP | `reports`, `finance`; semantic keys plus `Intl`. |
| System/recovery | `CanonicalDesktopSystem.tsx` | notifications, audit, backup/restore, settings, sync/runtime state | `system`, `backup`; preserve exact safety meaning in Arabic review. |
| Website public/auth | `PublicPages.tsx`, `AuthPages.tsx`, layouts | all marketing, auth, support, download, warning copy | website resource bundles and localized route metadata. |
| Website portal | `PortalPages.tsx`, `PortalLayout.tsx`, `StatusBadge.tsx` | plan/license/device states, timeline, activation, account | `portal`; never translate by mutating enum text. |
| Errors from native/server boundaries | API error rendering throughout | many caught `Error.message` values are shown directly | Map stable error codes to localized safe messages; retain diagnostic detail separately. |

Before extraction, restrict the desktop message catalog to route-reachable production components. Legacy browser/demo pages such as older `Sales.tsx` and `Purchases.tsx` should not inflate translation scope unless they remain intentionally shipped in browser development mode.

## 13. Recommended implementation order

1. **Truthful baseline:** decide Early Access language policy; hide/disable misleading language controls and correct “Start trial” CTA semantics.
2. **Copy and state model:** create the shared English glossary, semantic status/error keys, and one canonical onboarding sequence.
3. **Localization foundation:** add locale resources, runtime locale state, `Intl` formatters, root `lang`/`dir`, missing-key checks, and persistence.
4. **First-run guidance:** implement website activation checklist and desktop setup/readiness checklist using existing commands and permissions.
5. **Core zero states:** add role-aware next actions to Branches, Catalog, Purchasing, Inventory/POS, Counts, Dashboard, Reports, Finance, and Backup.
6. **Navigation/page anatomy:** reduce peer choices, remove duplicates, standardize headers/tabs/filters/states/actions, and preserve URLs/authorization.
7. **Arabic translation:** translate reviewed English resources with pharmacy-domain review; do not translate raw identifiers.
8. **RTL hardening:** replace physical CSS/icons, isolate mixed-direction data, and test all critical screens at target desktop/mobile sizes.
9. **Guided tutorial:** add role-specific, replayable contextual education after stable layout and translation keys exist.
10. **Acceptance:** English and Arabic first-run-to-sale usability sessions, keyboard/accessibility checks, screenshot regression, and localization completeness gates.

## 14. What must be fixed before first real Early Access distribution

The minimum safe UX release gate is:

1. Make the supported interface language truthful: either ship working Arabic/RTL or explicitly ship English-only and remove the nonfunctional language action.
2. Add a visible, resumable getting-started checklist spanning portal activation and desktop setup.
3. Explain portal account versus device license versus local administrator before users enter credentials.
4. Correct website “Start trial” links so the label matches the action that actually occurs.
5. Add actionable zero states for Catalog, stock bootstrap, Purchasing prerequisites, POS no-stock, and Backup.
6. Require/confirm operational branch scope before the first stock-affecting workflow and clearly distinguish report filters.
7. Remove or implement the disabled global search and `Ctrl K` promise.
8. Restore a mobile portal sign-out path.
9. Add retry/support/next-step actions to account confirmation, portal errors, device activation, and password-reset completion.
10. Run an observed first-time-user test from production registration through one complete local journey: activation → local admin → branch → product → purchase/receipt → sale/return → count → report → backup.

Arabic translation itself may follow an English-only closed Early Access only if the limitation is explicit everywhere and no Arabic interface is advertised. Arabic/RTL must be complete before inviting Arabic-interface users.

## 15. What can wait until after Early Access begins

- a broad visual redesign or new brand system;
- configurable themes, dense/comfortable modes, and advanced personalization;
- command-palette/global-search expansion beyond an honest initial implementation;
- sophisticated in-app analytics or personalized recommendations;
- animated tours and advanced coach marks beyond the minimal guided checklist;
- extensive dashboard customization;
- multi-language support beyond English and Arabic;
- automated screenshot translation previews for every noncritical route, after the core RTL matrix is green;
- deeper mobile portal optimization beyond restoring essential account actions;
- payments, subscriptions, installer code signing, cloud-sync rollout, and Phase 10 work, which remain outside Phase 9B scope.

## Audit conclusion

The product has the core operational depth needed for Early Access, and several high-risk actions already communicate their posting/restore boundaries well. The main product-experience risk is not missing pharmacy capability; it is missing orientation. Phase 9B should first create a truthful language foundation and a connected setup path, then standardize navigation, empty states, and page hierarchy without changing business rules.

**Phase 9B UX & Localization Audit: COMPLETE**
