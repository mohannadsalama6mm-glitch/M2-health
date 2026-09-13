-- Sales / POS. Every completed sale is one atomic transaction in the service:
-- validate -> price/stock snapshots -> persist sale + items + batch allocation +
-- payments -> post stock_movements of type 'sale'. Voiding reverses the exact
-- batches via 'adjustment' movements; returns restock sold lines as
-- 'customer_return' movements. Amounts are integer EGP minor units (piastres).
-- customer_name/phone remain immutable snapshots; customer_id links the optional
-- profile (customers table is introduced by migration 005).

CREATE TABLE sales (
    id TEXT PRIMARY KEY NOT NULL,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    sequence INTEGER NOT NULL CHECK(sequence >= 1),
    receipt_number TEXT NOT NULL CHECK(length(receipt_number) BETWEEN 1 AND 40),
    customer_id TEXT REFERENCES customers(id) ON DELETE RESTRICT,
    customer_name TEXT NOT NULL DEFAULT '' CHECK(length(customer_name) <= 120),
    customer_phone TEXT NOT NULL DEFAULT '' CHECK(length(customer_phone) <= 40),
    status TEXT NOT NULL DEFAULT 'completed' CHECK(status IN ('completed', 'void')),
    subtotal_minor INTEGER NOT NULL CHECK(subtotal_minor BETWEEN 0 AND 9007199254740991),
    discount_minor INTEGER NOT NULL CHECK(discount_minor BETWEEN 0 AND 9007199254740991),
    tax_minor INTEGER NOT NULL CHECK(tax_minor BETWEEN 0 AND 9007199254740991),
    total_minor INTEGER NOT NULL CHECK(total_minor BETWEEN 0 AND 9007199254740991),
    paid_minor INTEGER NOT NULL CHECK(paid_minor BETWEEN 0 AND 9007199254740991),
    change_minor INTEGER NOT NULL CHECK(change_minor BETWEEN 0 AND 9007199254740991),
    payment_method TEXT NOT NULL DEFAULT 'cash' CHECK(payment_method IN ('cash', 'card', 'other')),
    user TEXT NOT NULL DEFAULT '' CHECK(length(user) <= 160),
    note TEXT NOT NULL DEFAULT '' CHECK(length(note) <= 500),
    void_reason TEXT NOT NULL DEFAULT '' CHECK(length(void_reason) <= 500),
    voided_at TEXT,
    voided_by TEXT CHECK(voided_by IS NULL OR length(voided_by) <= 160),
    completed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK(total_minor = subtotal_minor - discount_minor + tax_minor),
    CHECK(paid_minor >= total_minor),
    CHECK(change_minor = paid_minor - total_minor),
    CHECK(status != 'void' OR (length(trim(void_reason)) > 0 AND voided_at IS NOT NULL)),
    UNIQUE(branch_id, sequence)
) STRICT;
CREATE INDEX sales_branch_time ON sales(branch_id, completed_at DESC);
CREATE INDEX sales_status ON sales(status);

-- Moment-of-sale snapshots: product name, package label, selling price and cost are
-- copied at sale time so later catalog edits never mutate historic receipts.
CREATE TABLE sale_items (
    id TEXT PRIMARY KEY NOT NULL,
    sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    product_name TEXT NOT NULL CHECK(length(product_name) BETWEEN 1 AND 240),
    package_label TEXT NOT NULL CHECK(length(package_label) BETWEEN 1 AND 160),
    quantity INTEGER NOT NULL CHECK(quantity > 0 AND quantity <= 9007199254740991),
    selling_price_minor INTEGER NOT NULL CHECK(selling_price_minor BETWEEN 0 AND 9007199254740991),
    cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
    line_total_minor INTEGER NOT NULL CHECK(line_total_minor BETWEEN 0 AND 9007199254740991),
    line_cost_minor INTEGER CHECK(line_cost_minor BETWEEN 0 AND 9007199254740991),
    position INTEGER NOT NULL DEFAULT 0 CHECK(position >= 0),
    CHECK(line_total_minor = quantity * selling_price_minor),
    CHECK((cost_price_minor IS NULL AND line_cost_minor IS NULL)
          OR (cost_price_minor IS NOT NULL AND line_cost_minor = quantity * cost_price_minor))
) STRICT;
CREATE INDEX sale_items_sale ON sale_items(sale_id);
CREATE INDEX sale_items_package ON sale_items(product_package_id);

-- FEFO allocation record: which batch(es) fed each line and at what cost. This makes
-- a void a precise reversal of the exact lots and feeds return eligibility.
CREATE TABLE sale_item_batches (
    id TEXT PRIMARY KEY NOT NULL,
    sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
    sale_item_id TEXT NOT NULL REFERENCES sale_items(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES inventory_batches(id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK(quantity > 0 AND quantity <= 9007199254740991),
    cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
    UNIQUE(sale_item_id, batch_id)
) STRICT;
CREATE INDEX sale_item_batches_sale ON sale_item_batches(sale_id);
CREATE INDEX sale_item_batches_batch ON sale_item_batches(batch_id);

CREATE TABLE sale_payments (
    id TEXT PRIMARY KEY NOT NULL,
    sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
    method TEXT NOT NULL CHECK(method IN ('cash', 'card', 'other')),
    amount_minor INTEGER NOT NULL CHECK(amount_minor > 0 AND amount_minor <= 9007199254740991),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX sale_payments_sale ON sale_payments(sale_id);

CREATE TABLE sale_returns (
    id TEXT PRIMARY KEY NOT NULL,
    sale_id TEXT NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    reason TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 500),
    user TEXT NOT NULL DEFAULT '' CHECK(length(user) <= 160),
    total_refund_minor INTEGER NOT NULL CHECK(total_refund_minor BETWEEN 0 AND 9007199254740991),
    returned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX sale_returns_sale ON sale_returns(sale_id);
-- Returned line quantity capped at what was sold minus what was already returned.
CREATE TABLE sale_return_items (
    id TEXT PRIMARY KEY NOT NULL,
    sale_return_id TEXT NOT NULL REFERENCES sale_returns(id) ON DELETE RESTRICT,
    sale_item_id TEXT NOT NULL REFERENCES sale_items(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK(quantity > 0 AND quantity <= 9007199254740991),
    refund_minor INTEGER NOT NULL CHECK(refund_minor BETWEEN 0 AND 9007199254740991),
    line_refund_minor INTEGER NOT NULL CHECK(line_refund_minor BETWEEN 0 AND 9007199254740991),
    CHECK(line_refund_minor = quantity * refund_minor),
    UNIQUE(sale_return_id, sale_item_id)
) STRICT;
CREATE INDEX sale_return_items_item ON sale_return_items(sale_item_id);
CREATE INDEX sale_return_items_package ON sale_return_items(product_package_id);