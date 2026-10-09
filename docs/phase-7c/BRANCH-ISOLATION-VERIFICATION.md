# Phase 7C Branch Isolation Verification

Branch isolation is enforced in Rust before domain execution and again by persisted foreign keys and domain queries where applicable.

- Selected-branch operations require current membership in the persisted branch.
- Transfers require authorization for both source and destination branches.
- Canonical record operations resolve branch ownership from the stored sale, return, receipt, purchase return, expense, journal, alert, count session, or Inventory document.
- Non-admin all-branch Dashboard, reporting, and export requests are rejected.
- Organization-wide Catalog, supplier, customer, and purchase-order identities do not grant access to branch-owned stock or transactions.
- Decoy top-level branch fields and `authorizationBranchId` cannot override nested or persisted ownership.
- Durable membership removal is observed on the next invocation using the same otherwise-valid token.

Automated tests deny cross-branch batch creation, opening stock, adjustment, transfers without both memberships, sale-ID substitution, Finance source access, report/export scope widening, and irrelevant branch-field bypass. Inventory, reports, Finance, purchasing, Dashboard, partner histories, and CSV filters also have positive branch-scoped exact-value tests.

No known active branch-owned mutation or report bypasses centralized branch authorization.
