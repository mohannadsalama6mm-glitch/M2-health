-- Partners (customers & suppliers) and purchases. The customers table is the
-- parent of sales.customer_id (migration 004); it must exist before any sale row
-- can reference it, so it lives here at the start of the Phase 5 data model.
-- Every completed purchase is one atomic transaction in the service: validate ->
-- restock into matching or new lots -> post stock_movements of type 'purchase' ->
-- persist purchase + items + batch allocation + payment. Purchase costs are
-- immutable snapshots: purchase_items.unit_cost_minor and, per lot,
-- purchase_item_batches.unit_cost_minor. Amounts are integer EGP minor units.

CREATE TABLE customers (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE CHECK(length(code) BETWEEN 1 AND 40),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 180),
    phone TEXT NOT NULL DEFAULT '' CHECK(length(phone) <= 40),
    email TEXT NOT NULL DEFAULT '' CHECK(length(email) <= 120),
    address TEXT NOT NULL DEFAULT '' CHECK(length(address) <= 300),
    notes TEXT NOT NULL DEFAULT '' CHECK(length(notes) <= 500),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX customers_active ON customers(is_active);

CREATE TABLE suppliers (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE CHECK(length(code) BETWEEN 1 AND 40),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 180),
    phone TEXT NOT NULL DEFAULT '' CHECK(length(phone) <= 40),
    email TEXT NOT NULL DEFAULT '' CHECK(length(email) <= 120),
    address TEXT NOT NULL DEFAULT '' CHECK(length(address) <= 300),
    notes TEXT NOT NULL DEFAULT '' CHECK(length(notes) <= 500),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX suppliers_active ON suppliers(is_active);

CREATE TABLE purchases (
    id TEXT PRIMARY KEY NOT NULL,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    supplier_id TEXT REFERENCES suppliers(id) ON DELETE RESTRICT,
    sequence INTEGER NOT NULL CHECK(sequence >= 1),
    purchase_number TEXT NOT NULL CHECK(length(purchase_number) BETWEEN 1 AND 40),
    invoice_number TEXT NOT NULL DEFAULT '' CHECK(length(invoice_number) <= 60),
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
CREATE INDEX purchases_branch_time ON purchases(branch_id, completed_at DESC);
CREATE INDEX purchases_supplier ON purchases(supplier_id);
CREATE INDEX purchases_status ON purchases(status);

-- Moment-of-purchase snapshots: product name, package label and unit cost are
-- copied at purchase time so later catalog edits never mutate historic invoices.
CREATE TABLE purchase_items (
    id TEXT PRIMARY KEY NOT NULL,
    purchase_id TEXT NOT NULL REFERENCES purchases(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    product_name TEXT NOT NULL CHECK(length(product_name) BETWEEN 1 AND 240),
    package_label TEXT NOT NULL CHECK(length(package_label) BETWEEN 1 AND 160),
    quantity INTEGER NOT NULL CHECK(quantity > 0 AND quantity <= 9007199254740991),
    unit_cost_minor INTEGER NOT NULL CHECK(unit_cost_minor BETWEEN 0 AND 9007199254740991),
    line_total_minor INTEGER NOT NULL CHECK(line_total_minor BETWEEN 0 AND 9007199254740991),
    position INTEGER NOT NULL DEFAULT 0 CHECK(position >= 0),
    CHECK(line_total_minor = quantity * unit_cost_minor)
) STRICT;
CREATE INDEX purchase_items_purchase ON purchase_items(purchase_id);
CREATE INDEX purchase_items_package ON purchase_items(product_package_id);

-- Which lot(s) received the purchased quantity and at what unit cost. This makes a
-- purchase a precise booking into the branch's FEFO pool and preserves per-lot cost.
CREATE TABLE purchase_item_batches (
    id TEXT PRIMARY KEY NOT NULL,
    purchase_id TEXT NOT NULL REFERENCES purchases(id) ON DELETE RESTRICT,
    purchase_item_id TEXT NOT NULL REFERENCES purchase_items(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES inventory_batches(id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK(quantity > 0 AND quantity <= 9007199254740991),
    unit_cost_minor INTEGER CHECK(unit_cost_minor BETWEEN 0 AND 9007199254740991),
    UNIQUE(purchase_item_id, batch_id)
) STRICT;
CREATE INDEX purchase_item_batches_purchase ON purchase_item_batches(purchase_id);
CREATE INDEX purchase_item_batches_batch ON purchase_item_batches(batch_id);

CREATE TABLE purchase_payments (
    id TEXT PRIMARY KEY NOT NULL,
    purchase_id TEXT NOT NULL REFERENCES purchases(id) ON DELETE RESTRICT,
    method TEXT NOT NULL CHECK(method IN ('cash', 'card', 'other')),
    amount_minor INTEGER NOT NULL CHECK(amount_minor > 0 AND amount_minor <= 9007199254740991),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX purchase_payments_purchase ON purchase_payments(purchase_id);