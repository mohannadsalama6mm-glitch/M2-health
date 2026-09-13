-- Inventory / batches / expiry / ledger. Quantity is NEVER stored on products or
-- packages; every quantity change is a stock_movements row and balances are derived
-- from the ledger (stock_balances view).

CREATE TABLE inventory_batches (
    id TEXT PRIMARY KEY NOT NULL,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    batch_number TEXT NOT NULL COLLATE NOCASE CHECK(length(trim(batch_number)) BETWEEN 1 AND 64),
    expiry_date TEXT NOT NULL DEFAULT '' CHECK(expiry_date = '' OR (length(expiry_date) = 10 AND expiry_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]')),
    cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE UNIQUE INDEX inventory_batches_lot
    ON inventory_batches(branch_id, product_package_id, batch_number, COALESCE(expiry_date, ''));
CREATE INDEX inventory_batches_branch_package
    ON inventory_batches(branch_id, product_package_id);
CREATE INDEX inventory_batches_expiry
    ON inventory_batches(branch_id, expiry_date);

CREATE TABLE stock_movements (
    id TEXT PRIMARY KEY NOT NULL,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES inventory_batches(id) ON DELETE RESTRICT,
    movement_type TEXT NOT NULL CHECK(movement_type IN (
        'opening', 'purchase', 'sale', 'customer_return', 'supplier_return',
        'adjustment', 'damage', 'expired', 'transfer_out', 'transfer_in', 'count_correction'
    )),
    quantity_delta INTEGER NOT NULL CHECK(quantity_delta <> 0),
    cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
    reference_type TEXT CHECK(reference_type IS NULL OR length(reference_type) <= 40),
    reference_id TEXT CHECK(reference_id IS NULL OR length(reference_id) <= 128),
    reason TEXT NOT NULL DEFAULT '' CHECK(length(reason) <= 500),
    user TEXT NOT NULL DEFAULT '' CHECK(length(user) <= 160),
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX stock_movements_branch ON stock_movements(branch_id);
CREATE INDEX stock_movements_package ON stock_movements(product_package_id);
CREATE INDEX stock_movements_batch ON stock_movements(batch_id);
CREATE INDEX stock_movements_time ON stock_movements(occurred_at);

CREATE VIEW stock_balances AS
SELECT batch_id, product_package_id, branch_id, SUM(quantity_delta) AS quantity
FROM stock_movements
GROUP BY batch_id;

CREATE TABLE stock_counts (
    id TEXT PRIMARY KEY NOT NULL,
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    scope TEXT NOT NULL DEFAULT '' CHECK(length(scope) <= 160),
    category_id TEXT REFERENCES categories(id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'completed')),
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE INDEX stock_counts_branch_status ON stock_counts(branch_id, status);

CREATE TABLE stock_count_items (
    id TEXT PRIMARY KEY NOT NULL,
    stock_count_id TEXT NOT NULL REFERENCES stock_counts(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES inventory_batches(id) ON DELETE RESTRICT,
    system_quantity INTEGER NOT NULL CHECK(system_quantity >= 0),
    counted_quantity INTEGER NOT NULL CHECK(counted_quantity >= 0),
    variance INTEGER NOT NULL DEFAULT 0 CHECK(variance = counted_quantity - system_quantity),
    entered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE UNIQUE INDEX stock_count_items_batch ON stock_count_items(stock_count_id, batch_id);
CREATE INDEX stock_count_items_package ON stock_count_items(product_package_id);

-- Branch-aware reorder configuration. Absent rows default to 10 in queries;
-- reorder_level 0 disables the threshold for that package/branch pair.
CREATE TABLE inventory_settings (
    branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
    product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
    reorder_level INTEGER NOT NULL DEFAULT 10 CHECK(reorder_level BETWEEN 0 AND 9007199254740991),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (branch_id, product_package_id)
) STRICT;