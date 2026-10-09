# Phase 7C Cross-Domain Regression Evidence

The Phase 7C regression matrix combines one complete isolated pharmacy flow with the established domain suites.

The complete flow creates product/package, supplier, and customer identities; posts a purchase receipt to an exact batch; posts a supplier return; completes a customer sale; posts a partial customer return/refund; records an expense; verifies supplier and customer history; reconciles operational reports, Dashboard, and Finance; repeats idempotent requests; and ends with SQLite integrity and foreign-key checks.

Exact stock arithmetic is `5 - 1 - 2 + 1 = 3`. The supplier-return movement retains its source reference. The Finance trial balance is balanced and Dashboard coverage is complete.

Complementary suites verify package/barcode constraints, import consistency, FEFO, expiry exclusion, negative-stock prevention, counts, transfers, holds, split payments, discounts, voids, missing acquisition cost, immutable journals, historical-unposted semantics, cross-branch denials, pagination, Arabic/UTF-8 CSV data, and formula-injection protection.

All data used by these tests is synthetic and held in in-memory or temporary isolated databases. No real database or production dataset was opened.
