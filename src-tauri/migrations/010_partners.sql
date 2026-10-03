CREATE TABLE partner_sequences (
 kind TEXT PRIMARY KEY NOT NULL CHECK(kind IN ('SUPPLIER','CUSTOMER')),
 next_number INTEGER NOT NULL CHECK(next_number>0)
) STRICT;

CREATE TABLE suppliers (
 id TEXT PRIMARY KEY NOT NULL,
 code TEXT NOT NULL UNIQUE COLLATE NOCASE,
 display_name TEXT NOT NULL CHECK(length(trim(display_name)) BETWEEN 1 AND 180),
 contact_person TEXT,
 primary_phone TEXT,
 secondary_phone TEXT,
 email TEXT,
 address TEXT,
 tax_registration_id TEXT,
 notes TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE UNIQUE INDEX suppliers_tax_unique ON suppliers(lower(tax_registration_id)) WHERE tax_registration_id IS NOT NULL;
CREATE INDEX suppliers_list ON suppliers(is_active,display_name COLLATE NOCASE,id);

CREATE TABLE customers (
 id TEXT PRIMARY KEY NOT NULL,
 code TEXT NOT NULL UNIQUE COLLATE NOCASE,
 display_name TEXT NOT NULL CHECK(length(trim(display_name)) BETWEEN 1 AND 180),
 primary_phone TEXT,
 secondary_phone TEXT,
 email TEXT,
 address TEXT,
 notes TEXT,
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX customers_list ON customers(is_active,display_name COLLATE NOCASE,id);

CREATE TRIGGER suppliers_retain_history BEFORE DELETE ON suppliers BEGIN
 SELECT RAISE(ABORT,'Supplier history must be retained');
END;
CREATE TRIGGER customers_retain_history BEFORE DELETE ON customers BEGIN
 SELECT RAISE(ABORT,'Customer history must be retained');
END;
