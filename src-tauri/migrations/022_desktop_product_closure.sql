CREATE TABLE local_settings (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 pharmacy_name TEXT NOT NULL CHECK(length(trim(pharmacy_name)) BETWEEN 1 AND 160),
 pharmacy_phone TEXT NOT NULL DEFAULT '' CHECK(length(pharmacy_phone)<=64),
 pharmacy_address TEXT NOT NULL DEFAULT '' CHECK(length(pharmacy_address)<=500),
 registration_number TEXT NOT NULL DEFAULT '' CHECK(length(registration_number)<=100),
 timezone TEXT NOT NULL DEFAULT 'Africa/Cairo' CHECK(timezone IN ('Africa/Cairo','UTC')),
 receipt_header TEXT NOT NULL DEFAULT 'M² Health Pharmacy' CHECK(length(receipt_header)<=160),
 receipt_footer TEXT NOT NULL DEFAULT 'Thank you for choosing M² Health.' CHECK(length(receipt_footer)<=500),
 updated_by_user_id TEXT REFERENCES users(id) ON DELETE RESTRICT,
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

INSERT INTO local_settings(singleton,pharmacy_name)
VALUES(1,'M² Health Pharmacy');

CREATE TRIGGER local_settings_singleton_no_delete
BEFORE DELETE ON local_settings
BEGIN SELECT RAISE(ABORT,'local_settings_are_required'); END;
