# Phase 7B Permission Matrix

## Enforcement contract

This matrix inventories all 162 commands registered by the single `tauri::generate_handler!` invocation in `src-tauri/src/lib.rs`. A coverage test compares that registration list with the Rust authorization registry. Commands not present in the registry are denied; there is no implicit permission or authenticated fallback.

`get_auth_status`, `bootstrap_admin`, `login`, and `recovery_status` are intentionally public. Bootstrap remains a one-time atomic operation guarded by the Phase 7A database invariants. Every other command requires a live Rust session. Authentication failures return `unauthorized`; permission and branch failures return `forbidden`; protected record lookups use a non-enumerating forbidden response when the record belongs to another branch. Authorization runs before domain reads or writes.

## Stable permission vocabulary

- `session.self`: current-session inspection and logout.
- `users.read`, `users.manage`, `roles.read`, `roles.manage`: identity administration.
- `branches.read`, `branches.manage`: branch directory administration.
- `catalog.read`, `catalog.manage`, `catalog.import`: global product/catalog operations.
- `inventory.read`, `inventory.adjust`, `inventory.receive`, `inventory.count`, `inventory.alert.manage`: branch stock and workflow operations.
- `sales.read`, `sales.create`, `sales.return`, `sales.void`: branch POS and posted-sale operations.
- `purchasing.read`, `purchasing.manage`: organization-wide purchase orders.
- `purchasing.receive`, `purchasing.return`: branch-owned receipts and returns.
- `suppliers.read`, `suppliers.manage`, `customers.read`, `customers.manage`: organization-wide partner records.
- `reports.operational.read`, `reports.finance.read`, `reports.export`: branch-filtered reporting and export.
- `finance.read`, `finance.expense.manage`: branch financial reads and expense posting.
- `dashboard.read`: branch-filtered operational dashboard.

## Scope and actor notation

- **Public**: no session; response is deliberately limited.
- **Global**: no invented branch ownership. Access is controlled only by permission.
- **Selected branch**: the request must name an authorized persisted branch ID. Null does not mean unrestricted.
- **Canonical ID**: Rust resolves the record's persisted branch before the operation; a caller filter cannot widen it.
- **Authorized set**: ADMIN may request organization-wide results. Other roles must select one authorized branch; null/all-branch requests are rejected rather than leaked.
- **Both branches**: transfers require membership in source and destination.
- **Backend principal**: mutation attribution is the validated Rust principal. Caller actor fields must match and are normalized to the principal by the typed frontend; forged values are denied.

## Role grants

| Role | Grants |
|---|---|
| ADMIN | Every permission, organization-wide scope |
| PHARMACIST | `branches.read`, `catalog.read`, `inventory.read`, `inventory.adjust`, `inventory.alert.manage`, `sales.read`, `sales.create`, `sales.return`, `customers.read`, `customers.manage`, `reports.operational.read`, `dashboard.read` |
| CASHIER | `branches.read`, `catalog.read`, `inventory.read`, `sales.read`, `sales.create`, `sales.return`, `customers.read`, `customers.manage`, `dashboard.read` |
| INVENTORY_MANAGER | `branches.read`, `catalog.read`, `catalog.manage`, `catalog.import`, `inventory.read`, `inventory.adjust`, `inventory.receive`, `inventory.count`, `inventory.alert.manage`, `purchasing.read`, `purchasing.manage`, `purchasing.receive`, `purchasing.return`, `suppliers.read`, `suppliers.manage`, `reports.operational.read`, `dashboard.read` |
| VIEWER | `branches.read`, `catalog.read`, `inventory.read`, `sales.read`, `purchasing.read`, `suppliers.read`, `customers.read`, `reports.operational.read`, `dashboard.read` |

VIEWER has no finance, export, identity, mutation, adjustment, receipt, return, or import grants. Because no accountant role exists in Phase 7A, finance permissions remain ADMIN-only in Phase 7B.

## Complete registered command inventory

The **Decision / error** column is additional to the common rule above: missing/invalid/expired/revoked/inactive session → `unauthorized`; missing permission, forged actor, missing branch, unauthorized branch, or cross-branch canonical ID → `forbidden`; authorization failure produces no domain call.

| Command | Domain | Class | Permission | Branch scope | Actor provenance | Decision / error |
|---|---|---|---|---|---|---|
| `get_auth_status` | Identity | Read | public | Public | n/a | Allow limited bootstrap status |
| `bootstrap_admin` | Identity | Write | public | Public | Created principal | Allow only Phase 7A atomic first-run bootstrap |
| `login` | Identity | Write | public | Public | Authenticated credential owner | Generic credential failure |
| `current_session` | Identity | Read | `session.self` | Global | Backend principal | Live session only |
| `logout` | Identity | Write | `session.self` | Global | Backend principal | Live session; revoke itself |
| `list_users` | Identity | Read | `users.read` | Global | n/a | Permission required |
| `get_user` | Identity | Read | `users.read` | Global | n/a | Permission required |
| `create_user` | Identity | Write | `users.manage` | Global | Backend principal | Permission required |
| `update_user` | Identity | Write | `users.manage` | Global | Backend principal | Permission required |
| `set_user_active` | Identity | Write | `users.manage` | Global | Backend principal | Permission and final-admin invariant |
| `reset_user_password` | Identity | Write | `users.manage` | Global | Backend principal | Permission; revoke target sessions |
| `list_roles` | Identity | Read | `roles.read` | Global | n/a | Permission required |
| `assign_user_role` | Identity | Write | `roles.manage` | Global | Backend principal | Permission required |
| `remove_user_role` | Identity | Write | `roles.manage` | Global | Backend principal | Permission and final-admin invariant |
| `set_user_branches` | Identity | Write | `users.manage` | Global | Backend principal | Permission; revoke target sessions |
| `get_phase6_dashboard` | Dashboard | Read | `dashboard.read` | Authorized set | n/a | ADMIN all; others selected member branch |
| `get_phase6_reconciliation` | Reporting/Finance | Read | `reports.finance.read` | Authorized set | n/a | ADMIN all; others selected member branch |
| `get_sale_finance_trace` | Reporting/Finance | Read | `reports.finance.read` | Canonical sale ID | n/a | Resolve sale branch before trace |
| `export_phase6_csv` | Reporting | Read/export | `reports.export` | Authorized set | n/a | Branch-scoped export only |
| `list_finance_accounts` | Finance | Read | `finance.read` | Global chart | n/a | Permission required |
| `create_finance_expense` | Finance | Write | `finance.expense.manage` | Authorized set | Backend principal | ADMIN may use nullable organization scope; branch IDs are validated |
| `get_finance_expense` | Finance | Read | `finance.read` | Canonical ID | n/a | Resolve expense branch |
| `list_finance_expenses` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized totals or rows |
| `get_finance_journal` | Finance | Read | `finance.read` | Canonical ID | n/a | Resolve journal source branch |
| `list_finance_journals` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized rows |
| `get_trial_balance` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized aggregate |
| `get_account_activity` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized rows/totals |
| `get_sales_financial_summary` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized aggregate |
| `get_expense_report` | Finance | Read | `finance.read` | Authorized set | n/a | No unauthorized aggregate |
| `get_reporting_overview` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered aggregates |
| `get_sales_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered rows/totals |
| `get_sales_by_product_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered aggregate |
| `get_sales_by_customer_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered aggregate |
| `get_inventory_stock_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered stock |
| `get_inventory_movement_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered movements |
| `get_expiry_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered batches |
| `get_low_stock_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered levels |
| `get_purchasing_report` | Reporting | Read | `reports.operational.read` | Global PO + authorized receipts | n/a | Receipt metrics branch-filtered |
| `get_purchase_receipt_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered receipts |
| `get_purchase_return_report` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered returns |
| `get_supplier_analytics` | Reporting | Read | `reports.operational.read` | Authorized set | n/a | Branch-filtered receipt data |
| `create_purchase_order_draft` | Purchasing | Write | `purchasing.manage` | Global PO | Backend principal | Permission required |
| `update_purchase_order_draft` | Purchasing | Write | `purchasing.manage` | Global PO | Backend principal | Permission required |
| `order_purchase_order` | Purchasing | Write | `purchasing.manage` | Global PO | Backend principal | Permission required |
| `cancel_purchase_order` | Purchasing | Write | `purchasing.manage` | Global PO | Backend principal | Permission required |
| `get_purchase_order` | Purchasing | Read | `purchasing.read` | Global PO | n/a | Permission required |
| `list_purchase_orders` | Purchasing | Read | `purchasing.read` | Global PO | n/a | Permission required |
| `search_purchase_packages` | Purchasing | Read | `catalog.read` | Global catalog | n/a | Permission required |
| `receive_purchase_order` | Purchasing | Write | `purchasing.receive` | Selected branch | Backend principal | Membership before receipt transaction |
| `get_purchase_receipt` | Purchasing | Read | `purchasing.read` | Canonical ID | n/a | Resolve receipt branch |
| `list_purchase_receipts` | Purchasing | Read | `purchasing.read` | Authorized set | n/a | Branch-filtered rows |
| `create_purchase_return` | Purchasing | Write | `purchasing.return` | Canonical receipt ID | Backend principal | Resolve receipt branch before transaction |
| `get_purchase_return` | Purchasing | Read | `purchasing.read` | Canonical ID | n/a | Resolve return branch |
| `list_purchase_returns` | Purchasing | Read | `purchasing.read` | Authorized set | n/a | Branch-filtered rows |
| `create_canonical_supplier` | Suppliers | Write | `suppliers.manage` | Global | Backend principal | Permission required |
| `get_canonical_supplier` | Suppliers | Read | `suppliers.read` | Global | n/a | Permission required |
| `update_canonical_supplier` | Suppliers | Write | `suppliers.manage` | Global | Backend principal | Permission required |
| `set_canonical_supplier_active` | Suppliers | Write | `suppliers.manage` | Global | Backend principal | Permission required |
| `list_canonical_suppliers` | Suppliers | Read | `suppliers.read` | Global | n/a | Permission required |
| `create_canonical_customer` | Customers | Write | `customers.manage` | Global | Backend principal | Permission required |
| `get_canonical_customer` | Customers | Read | `customers.read` | Global | n/a | Permission required |
| `update_canonical_customer` | Customers | Write | `customers.manage` | Global | Backend principal | Permission required |
| `set_canonical_customer_active` | Customers | Write | `customers.manage` | Global | Backend principal | Permission required |
| `list_canonical_customers` | Customers | Read | `customers.read` | Global | n/a | Permission required |
| `search_pos_products` | Sales | Read | `sales.read` | Selected branch | n/a | Authorized availability only |
| `get_pos_package_availability` | Sales | Read | `sales.read` | Selected branch | n/a | Authorized availability only |
| `save_sale_draft` | Sales | Write | `sales.create` | Selected branch | Backend principal | Membership before transaction |
| `hold_sale` | Sales | Write | `sales.create` | Canonical sale ID | Backend principal | Resolve sale branch |
| `resume_sale` | Sales | Write | `sales.create` | Canonical sale ID | Backend principal | Resolve sale branch |
| `preview_sale_totals` | Sales | Read | `sales.create` | Selected branch | n/a | Membership required |
| `post_sale` | Sales | Write | `sales.create` | Canonical/input sale branch | Backend principal | Authorize before posting |
| `list_canonical_sales` | Sales | Read | `sales.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `list_customer_sales` | Sales | Read | `sales.read` | Authorized set | n/a | Branch-filtered customer history |
| `get_canonical_sale` | Sales | Read | `sales.read` | Canonical ID | n/a | Resolve sale branch |
| `preview_sale_return` | Sales | Read | `sales.return` | Canonical sale ID | n/a | Resolve sale branch |
| `create_sale_return` | Sales | Write | `sales.return` | Canonical sale ID | Backend principal | Resolve before transaction |
| `post_sale_return` | Sales | Write | `sales.return` | Canonical return/sale ID | Backend principal | Resolve before transaction |
| `void_canonical_sale` | Sales | Write | `sales.void` | Canonical sale ID | Backend principal | Resolve before transaction |
| `inspect_inventory_overview` | Inventory | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `inspect_inventory_batches` | Inventory | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `inspect_inventory_movements` | Inventory | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `recovery_status` | Recovery | Read | public | Public | n/a | Allow compatibility flags only |
| `get_inventory_alert_settings` | Inventory alerts | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `update_inventory_alert_settings` | Inventory alerts | Write | `inventory.alert.manage` | Selected branch | Backend principal | Membership required |
| `evaluate_inventory_alerts` | Inventory alerts | Write | `inventory.alert.manage` | Selected branch | Backend principal | Membership required |
| `list_inventory_alerts` | Inventory alerts | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `get_inventory_alert` | Inventory alerts | Read | `inventory.read` | Canonical ID | n/a | Resolve alert branch |
| `dismiss_inventory_alert` | Inventory alerts | Write | `inventory.alert.manage` | Canonical ID | Backend principal | Resolve alert branch |
| `list_expiry_inventory` | Inventory alerts | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows |
| `list_low_stock_inventory` | Inventory alerts | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows |
| `get_fefo_inventory_batches` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `create_count_session` | Stock count | Write | `inventory.count` | Selected branch | Backend principal | Membership before sequence/write |
| `get_count_session` | Stock count | Read | `inventory.read` | Canonical ID | n/a | Resolve count branch |
| `list_count_sessions` | Stock count | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `start_count_session` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `record_count_entry` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `discover_count_stock` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `review_count_session` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `resume_count_session` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `cancel_count_session` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch |
| `complete_count_session` | Stock count | Write | `inventory.count` | Canonical ID | Backend principal | Resolve count branch before ledger transaction |
| `save_inventory_document` | Inventory workflow | Write | `inventory.adjust` | Selected/both branches | Backend principal | Transfer documents require both memberships |
| `get_inventory_document` | Inventory workflow | Read | `inventory.read` | Canonical ID | n/a | Resolve source and destination branches |
| `list_inventory_documents` | Inventory workflow | Read | `inventory.read` | Authorized set | n/a | Branch-filtered rows/counts |
| `mark_inventory_document_ready` | Inventory workflow | Write | `inventory.adjust` | Canonical ID | Backend principal | Resolve document branches |
| `cancel_inventory_document` | Inventory workflow | Write | `inventory.adjust` | Canonical ID | Backend principal | Resolve document branches |
| `post_inventory_document` | Inventory workflow | Write | `inventory.adjust` | Canonical ID | Backend principal | Resolve before ledger transaction |
| `reverse_inventory_document` | Inventory workflow | Write | `inventory.adjust` | Canonical ID | Backend principal | Resolve before reversal transaction |
| `list_workflow_batches` | Inventory workflow | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `fefo_allocation` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `create_inventory_batch` | Inventory | Write | `inventory.receive` | Selected branch | Backend principal | Membership required; no stock write |
| `record_opening_stock` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership before ledger write |
| `record_batch_opening_stock` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership before ledger write |
| `record_adjustment` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership before ledger write |
| `record_damage` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership before ledger write |
| `record_expiry_write_off` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership before ledger write |
| `transfer_inventory_stock` | Inventory | Write | `inventory.adjust` | Both branches | Backend principal | Both memberships before atomic transfer |
| `get_package_stock` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `get_batch_stock` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `list_branch_inventory` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `list_stock_movements` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `set_inventory_level` | Inventory | Write | `inventory.adjust` | Selected branch | Backend principal | Membership required |
| `list_low_stock` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `list_expiring_batches` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `get_fefo_batches` | Inventory | Read | `inventory.read` | Selected branch | n/a | Membership required |
| `profile_catalog_source` | Catalog import | Read | `catalog.import` | Global | n/a | Permission required |
| `dry_run_catalog_import` | Catalog import | Read | `catalog.import` | Global | n/a | Permission required |
| `apply_catalog_import` | Catalog import | Write | `catalog.import` | Global | Backend principal | Permission before import transaction |
| `get_catalog_import_status` | Catalog import | Read | `catalog.import` | Global | n/a | Permission required |
| `get_catalog_import_report` | Catalog import | Read | `catalog.import` | Global | n/a | Permission required |
| `ensure_default_branch` | Branches | Write | `branches.manage` | Global | Backend principal | Permission required |
| `list_branches` | Branches | Read | `branches.read` | Membership-filtered | n/a | ADMIN all; others memberships only |
| `get_branch` | Branches | Read | `branches.read` | Canonical branch ID | n/a | Membership required unless ADMIN |
| `create_branch` | Branches | Write | `branches.manage` | Global | Backend principal | Permission required |
| `create_manufacturer` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_manufacturers` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_manufacturer_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_category` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_categories` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_category_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_route` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_routes` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_route_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_active_ingredient` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_active_ingredients` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_active_ingredient_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_product` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_product_full` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `update_product_full` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `get_product` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `get_product_detail` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `list_products` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_product_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `link_product_ingredient` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `create_product_package` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_product_packages` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_product_package_active` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `add_barcode` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `list_package_barcodes` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `set_package_price` | Catalog | Write | `catalog.manage` | Global | Backend principal | Permission required |
| `get_current_package_price` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |
| `get_package_price_history` | Catalog | Read | `catalog.read` | Global | n/a | Permission required |

## Organization-wide entities

Products, packages, categories, manufacturers, routes, active ingredients, barcodes, price history, suppliers, customers, finance account definitions, and purchase-order headers are organization-wide. Phase 7B does not add fictional branch columns to these entities. Stock, sales, receipts, purchase returns, expenses, journals derived from branch operations, inventory documents, count sessions, alerts, and operational aggregates are branch-owned.

## Coverage rule

The Rust registry is the executable counterpart of this document. Startup dispatch denies an unknown or unclassified registered command. Automated coverage reads the `generate_handler!` source list and asserts exactly one policy for every registered command, including public commands and legacy/recovery entry points.
