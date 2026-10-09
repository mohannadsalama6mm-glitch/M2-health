CREATE TABLE users (
 id TEXT PRIMARY KEY NOT NULL,
 username TEXT NOT NULL CHECK(length(username) BETWEEN 3 AND 64),
 username_normalized TEXT NOT NULL UNIQUE CHECK(length(username_normalized) BETWEEN 3 AND 64),
 display_name TEXT NOT NULL CHECK(length(trim(display_name)) BETWEEN 1 AND 160),
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
 credential_revision INTEGER NOT NULL DEFAULT 1 CHECK(credential_revision>0),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX users_list ON users(is_active,username_normalized,id);

CREATE TABLE user_credentials (
 user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 password_hash TEXT NOT NULL CHECK(length(password_hash) BETWEEN 40 AND 1000),
 password_changed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

CREATE TABLE roles (
 id TEXT PRIMARY KEY NOT NULL,
 code TEXT NOT NULL UNIQUE CHECK(length(code) BETWEEN 2 AND 64),
 name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
 description TEXT NOT NULL DEFAULT '' CHECK(length(description)<=1000),
 is_system INTEGER NOT NULL DEFAULT 1 CHECK(is_system IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
INSERT INTO roles(id,code,name,description) VALUES
 ('70000000-0000-4000-8000-000000000001','ADMIN','Administrator','User and role administration foundation.'),
 ('70000000-0000-4000-8000-000000000002','PHARMACIST','Pharmacist','Pharmacist role foundation; operational enforcement begins in Phase 7B.'),
 ('70000000-0000-4000-8000-000000000003','CASHIER','Cashier','Cashier role foundation; operational enforcement begins in Phase 7B.'),
 ('70000000-0000-4000-8000-000000000004','INVENTORY_MANAGER','Inventory Manager','Inventory role foundation; operational enforcement begins in Phase 7B.'),
 ('70000000-0000-4000-8000-000000000005','VIEWER','Viewer','Read-oriented role foundation; operational enforcement begins in Phase 7B.');

CREATE TABLE user_roles (
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
 assigned_by TEXT REFERENCES users(id) ON DELETE RESTRICT,
 assigned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(user_id,role_id)
) STRICT;
CREATE INDEX user_roles_role ON user_roles(role_id,user_id);

CREATE TABLE user_branch_memberships (
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 branch_id TEXT NOT NULL REFERENCES branches(id) ON DELETE RESTRICT,
 is_primary INTEGER NOT NULL DEFAULT 0 CHECK(is_primary IN (0,1)),
 assigned_by TEXT REFERENCES users(id) ON DELETE RESTRICT,
 assigned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 PRIMARY KEY(user_id,branch_id)
) STRICT;
CREATE UNIQUE INDEX user_one_primary_branch ON user_branch_memberships(user_id) WHERE is_primary=1;
CREATE INDEX user_branch_memberships_branch ON user_branch_memberships(branch_id,user_id);

CREATE TABLE auth_bootstrap (
 singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton=1),
 initialized_by TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
 initialized_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

CREATE TABLE auth_events (
 id TEXT PRIMARY KEY NOT NULL,
 user_id TEXT REFERENCES users(id) ON DELETE RESTRICT,
 event_type TEXT NOT NULL CHECK(event_type IN ('BOOTSTRAP','LOGIN_SUCCESS','LOGIN_FAILURE','LOGOUT','PASSWORD_RESET','USER_ACTIVATED','USER_DEACTIVATED','ROLE_ASSIGNED','ROLE_REMOVED','BRANCH_MEMBERSHIP_CHANGED')),
 username_normalized TEXT,
 detail TEXT NOT NULL DEFAULT '' CHECK(length(detail)<=1000),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX auth_events_history ON auth_events(created_at DESC,id DESC);
CREATE INDEX auth_events_user ON auth_events(user_id,created_at DESC,id DESC);

CREATE TRIGGER users_no_delete BEFORE DELETE ON users BEGIN SELECT RAISE(ABORT,'users_are_retained'); END;
CREATE TRIGGER credentials_no_delete BEFORE DELETE ON user_credentials BEGIN SELECT RAISE(ABORT,'credentials_are_retained'); END;
CREATE TRIGGER system_roles_no_update BEFORE UPDATE ON roles WHEN OLD.is_system=1 BEGIN SELECT RAISE(ABORT,'system_roles_are_immutable'); END;
CREATE TRIGGER roles_no_delete BEFORE DELETE ON roles BEGIN SELECT RAISE(ABORT,'roles_are_retained'); END;
CREATE TRIGGER final_active_admin_user BEFORE UPDATE OF is_active ON users
 WHEN OLD.is_active=1 AND NEW.is_active=0 AND EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=OLD.id AND r.code='ADMIN')
 AND NOT EXISTS(SELECT 1 FROM users u JOIN user_roles ur ON ur.user_id=u.id JOIN roles r ON r.id=ur.role_id WHERE u.is_active=1 AND u.id<>OLD.id AND r.code='ADMIN')
 BEGIN SELECT RAISE(ABORT,'final_active_admin_required'); END;
CREATE TRIGGER final_active_admin_role BEFORE DELETE ON user_roles
 WHEN EXISTS(SELECT 1 FROM roles r WHERE r.id=OLD.role_id AND r.code='ADMIN')
 AND EXISTS(SELECT 1 FROM users u WHERE u.id=OLD.user_id AND u.is_active=1)
 AND NOT EXISTS(SELECT 1 FROM users u JOIN user_roles ur ON ur.user_id=u.id JOIN roles r ON r.id=ur.role_id WHERE u.is_active=1 AND u.id<>OLD.user_id AND r.code='ADMIN')
 BEGIN SELECT RAISE(ABORT,'final_active_admin_required'); END;
