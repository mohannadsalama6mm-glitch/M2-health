CREATE TABLE manufacturers (
 id TEXT PRIMARY KEY NOT NULL,
 name TEXT NOT NULL CHECK(length(trim(name)) > 0),
 normalized_name TEXT NOT NULL UNIQUE CHECK(length(normalized_name) > 0),
 country TEXT, phone TEXT, email TEXT, website TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE categories (
 id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL CHECK(length(trim(name)) > 0),
 normalized_name TEXT NOT NULL UNIQUE CHECK(length(normalized_name) > 0), description TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX manufacturers_search ON manufacturers(normalized_name COLLATE NOCASE);
-- Source route labels mix route and form. Preserve the label without inferring a dosage form.
CREATE TABLE routes (
 id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL CHECK(length(trim(name)) > 0),
 normalized_name TEXT NOT NULL UNIQUE CHECK(length(normalized_name) > 0),
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE active_ingredients (
 id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL CHECK(length(trim(name)) > 0),
 normalized_name TEXT NOT NULL UNIQUE CHECK(length(normalized_name) > 0), description TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE products (
 id TEXT PRIMARY KEY NOT NULL,
 commercial_name_en TEXT, commercial_name_ar TEXT,
 normalized_name_en TEXT, normalized_name_ar TEXT,
 scientific_name TEXT, normalized_scientific_name TEXT,
 manufacturer_id TEXT REFERENCES manufacturers(id) ON DELETE RESTRICT,
 category_id TEXT REFERENCES categories(id) ON DELETE RESTRICT,
 route_id TEXT REFERENCES routes(id) ON DELETE RESTRICT,
 description TEXT, notes TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK(length(trim(coalesce(commercial_name_en,''))) > 0 OR length(trim(coalesce(commercial_name_ar,''))) > 0)
) STRICT;
CREATE INDEX products_name_en ON products(normalized_name_en COLLATE NOCASE);
CREATE INDEX products_name_ar ON products(normalized_name_ar COLLATE NOCASE);
CREATE INDEX products_scientific ON products(normalized_scientific_name COLLATE NOCASE);
CREATE INDEX products_manufacturer ON products(manufacturer_id);
CREATE INDEX products_category ON products(category_id);
CREATE INDEX products_route ON products(route_id);
CREATE TABLE product_active_ingredients (
 id TEXT PRIMARY KEY NOT NULL,
 product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
 active_ingredient_id TEXT NOT NULL REFERENCES active_ingredients(id) ON DELETE RESTRICT,
 strength_text TEXT, position INTEGER NOT NULL DEFAULT 0 CHECK(position >= 0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(product_id,active_ingredient_id)
) STRICT;
CREATE INDEX product_ingredients_reverse ON product_active_ingredients(active_ingredient_id);
CREATE TABLE product_packages (
 id TEXT PRIMARY KEY NOT NULL,
 product_id TEXT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
 package_label TEXT NOT NULL CHECK(length(trim(package_label)) > 0),
 pack_size TEXT, unit_name TEXT,
 units_per_package INTEGER CHECK(units_per_package > 0 AND units_per_package <= 9007199254740991),
 strength_text TEXT,
 is_default INTEGER NOT NULL DEFAULT 0 CHECK(is_default IN (0,1)),
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK(is_default = 0 OR is_active = 1)
) STRICT;
CREATE INDEX packages_product ON product_packages(product_id);
CREATE UNIQUE INDEX packages_one_default ON product_packages(product_id) WHERE is_default=1;
CREATE TABLE barcodes (
 id TEXT PRIMARY KEY NOT NULL,
 product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
 barcode TEXT NOT NULL UNIQUE CHECK(length(barcode) BETWEEN 1 AND 128 AND barcode = trim(barcode)),
 is_primary INTEGER NOT NULL DEFAULT 0 CHECK(is_primary IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX barcodes_package ON barcodes(product_package_id);
CREATE UNIQUE INDEX barcodes_one_primary ON barcodes(product_package_id) WHERE is_primary=1;
CREATE TABLE product_price_history (
 id TEXT PRIMARY KEY NOT NULL,
 product_package_id TEXT NOT NULL REFERENCES product_packages(id) ON DELETE RESTRICT,
 selling_price_minor INTEGER NOT NULL CHECK(selling_price_minor BETWEEN 0 AND 9007199254740991),
 cost_price_minor INTEGER CHECK(cost_price_minor BETWEEN 0 AND 9007199254740991),
 effective_from TEXT NOT NULL CHECK(length(effective_from)=24 AND strftime('%Y-%m-%dT%H:%M:%fZ',effective_from) IS effective_from),
 effective_to TEXT CHECK(effective_to IS NULL OR (length(effective_to)=24 AND strftime('%Y-%m-%dT%H:%M:%fZ',effective_to) IS effective_to AND effective_to > effective_from)),
 reason TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE UNIQUE INDEX price_one_current ON product_price_history(product_package_id) WHERE effective_to IS NULL;
CREATE INDEX price_package_time ON product_price_history(product_package_id,effective_from DESC);
-- Price values/history cannot be rewritten or deleted. Only a current interval may be closed.
CREATE TRIGGER price_immutable BEFORE UPDATE ON product_price_history
WHEN OLD.effective_to IS NOT NULL OR NEW.effective_to IS NULL
 OR NEW.id IS NOT OLD.id OR NEW.product_package_id IS NOT OLD.product_package_id
 OR NEW.selling_price_minor IS NOT OLD.selling_price_minor OR NEW.cost_price_minor IS NOT OLD.cost_price_minor
 OR NEW.effective_from IS NOT OLD.effective_from OR NEW.reason IS NOT OLD.reason OR NEW.created_at IS NOT OLD.created_at
BEGIN SELECT RAISE(ABORT,'price_history_immutable'); END;
CREATE TRIGGER price_no_delete BEFORE DELETE ON product_price_history
BEGIN SELECT RAISE(ABORT,'price_history_immutable'); END;
CREATE TRIGGER price_no_overlap BEFORE INSERT ON product_price_history
WHEN EXISTS (SELECT 1 FROM product_price_history p WHERE p.product_package_id=NEW.product_package_id
 AND (NEW.effective_to IS NULL OR p.effective_from < NEW.effective_to)
 AND (p.effective_to IS NULL OR NEW.effective_from < p.effective_to))
BEGIN SELECT RAISE(ABORT,'price_history_overlap'); END;
