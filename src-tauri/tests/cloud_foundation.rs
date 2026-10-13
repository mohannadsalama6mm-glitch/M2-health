use m2_health_lib::{
    cloud,
    db::{
        connection::AppDb,
        migrations::{self, Migration, MIGRATIONS},
    },
};
use rusqlite::{params, Connection};
use uuid::Uuid;

fn id() -> String {
    Uuid::new_v4().to_string()
}

fn count(db: &AppDb, table: &str) -> i64 {
    db.lock()
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn insert_user(connection: &Connection, id: &str, name: &str) {
    connection
        .execute(
            "INSERT INTO users(id,username,username_normalized,display_name) VALUES(?1,?2,?2,?2)",
            params![id, name],
        )
        .unwrap();
}

#[test]
fn fresh_foundation_is_local_only_and_fabricates_no_tenant() {
    let db = AppDb::in_memory().unwrap();
    assert_eq!(count(&db, "_schema_version"), 21);
    assert_eq!(count(&db, "organizations"), 0);
    assert_eq!(count(&db, "branch_tenant_ownership"), 0);
    assert_eq!(count(&db, "organization_memberships"), 0);
    assert_eq!(count(&db, "cloud_device_enrollments"), 0);
    assert_eq!(count(&db, "sync_outbox"), 0);
    assert_eq!(count(&db, "sync_inbox"), 0);
    assert_eq!(count(&db, "local_device_identity"), 1);
    let status = cloud::status(&db).unwrap();
    assert_eq!(status.mode, "LOCAL_ONLY");
    assert_eq!(status.device_status, "LOCAL_ONLY");
    assert!(!status.configured);
    assert!(!status.transport_enabled);
    assert!(!status.sync_available);
    assert!(status.setup_required);
    assert!(status.organization_id.is_none());
    assert!(Uuid::parse_str(&status.device_id).is_ok());
}

#[test]
fn isolated_v17_upgrade_preserves_rows_and_does_not_adopt_them() {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    migrations::apply(&mut connection, &MIGRATIONS[..17]).unwrap();
    connection
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'KEEP','Preserved')",
            [id()],
        )
        .unwrap();
    migrations::apply(&mut connection, MIGRATIONS).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT max(version) FROM _schema_version", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        21
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM branches", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM organizations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM branch_tenant_ownership", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap(),
        0
    );
    assert!(connection
        .pragma_query_value(None, "foreign_key_check", |row| row.get::<_, String>(0))
        .is_err());
}

#[test]
fn tenant_and_branch_boundaries_are_enforced_by_composite_keys() {
    let db = AppDb::in_memory().unwrap();
    let connection = db.lock().unwrap();
    let org_a = id();
    let org_b = id();
    let owner = id();
    let member = id();
    let branch_a = id();
    let branch_b = id();
    let branch_c = id();
    insert_user(&connection, &owner, "owner-user");
    insert_user(&connection, &member, "member-user");
    connection
        .execute(
            "INSERT INTO organizations(id,name,slug) VALUES(?1,'A','org-a'),(?2,'B','org-b')",
            params![org_a, org_b],
        )
        .unwrap();
    connection.execute("INSERT INTO organization_memberships(organization_id,user_id,role_code) VALUES(?1,?3,'OWNER'),(?1,?4,'MEMBER'),(?2,?3,'OWNER')", params![org_a,org_b,owner,member]).unwrap();
    connection
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'A','A'),(?2,'B','B'),(?3,'C','C')",
            params![branch_a, branch_b, branch_c],
        )
        .unwrap();
    connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?3,?5),(?2,?4,?5)", params![org_a,org_b,branch_a,branch_b,owner]).unwrap();
    connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?4)", params![org_a,member,branch_a,owner]).unwrap();
    assert!(connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?4)", params![org_a,member,branch_b,owner]).is_err());
    assert!(connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?2,?3)", params![org_b,branch_a,owner]).is_err());
    assert!(connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?2,?3)", params![org_a,branch_c,member]).is_err());
    assert!(connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?2)", params![org_a,member,branch_a]).is_err());
}

#[test]
fn outbox_requires_active_membership_enrollment_and_branch_access() {
    let db = AppDb::in_memory().unwrap();
    let connection = db.lock().unwrap();
    let org = id();
    let actor = id();
    let outsider = id();
    let branch = id();
    let device: String = connection
        .query_row("SELECT device_id FROM local_device_identity", [], |row| {
            row.get(0)
        })
        .unwrap();
    insert_user(&connection, &actor, "actor-user");
    insert_user(&connection, &outsider, "outside-user");
    connection
        .execute(
            "INSERT INTO organizations(id,name,slug) VALUES(?1,'A','org-a')",
            [&org],
        )
        .unwrap();
    connection.execute("INSERT INTO organization_memberships(organization_id,user_id,role_code) VALUES(?1,?2,'OWNER')", params![org,actor]).unwrap();
    connection
        .execute(
            "INSERT INTO branches(id,code,name) VALUES(?1,'A','A')",
            [&branch],
        )
        .unwrap();
    connection.execute("INSERT INTO branch_tenant_ownership(organization_id,branch_id,adopted_by_user_id) VALUES(?1,?2,?3)", params![org,branch,actor]).unwrap();
    connection.execute("INSERT INTO organization_branch_memberships(organization_id,user_id,branch_id,granted_by_user_id) VALUES(?1,?2,?3,?2)", params![org,actor,branch]).unwrap();
    connection.execute("INSERT INTO cloud_device_enrollments(device_id,organization_id,cloud_device_id,status,enrolled_by_user_id,enrolled_at) VALUES(?1,?2,?3,'ENROLLED',?4,'2026-10-09T00:00:00Z')", params![device,org,id(),actor]).unwrap();
    let event = id();
    let insert = "INSERT INTO sync_outbox(event_id,organization_id,branch_id,device_id,actor_user_id,entity_type,entity_id,operation,payload_json,payload_hash,schema_version,occurred_at,local_sequence) VALUES(?1,?2,?3,?4,?5,'inventory_movement','movement-1','APPENDED','{}',?6,1,'2026-10-09T00:00:00Z',?7)";
    connection
        .execute(
            insert,
            params![event, org, branch, device, actor, "a".repeat(64), 1],
        )
        .unwrap();
    assert!(connection
        .execute(
            insert,
            params![event, org, branch, device, actor, "a".repeat(64), 2]
        )
        .is_err());
    assert!(connection
        .execute(
            insert,
            params![id(), org, branch, device, outsider, "a".repeat(64), 3]
        )
        .is_err());
    assert!(connection
        .execute(
            "UPDATE sync_outbox SET payload_json='{\"changed\":true}' WHERE event_id=?1",
            [&event]
        )
        .is_err());
    connection
        .execute(
            "UPDATE sync_outbox SET attempt_count=1,last_error='offline' WHERE event_id=?1",
            [&event],
        )
        .unwrap();
    connection.execute("UPDATE cloud_device_enrollments SET status='REVOKED',revoked_at='2026-10-09T00:01:00Z' WHERE device_id=?1", [&device]).unwrap();
    assert!(connection
        .execute(
            insert,
            params![id(), org, branch, device, actor, "a".repeat(64), 4]
        )
        .is_err());
}

#[test]
fn inbox_identity_is_idempotent_and_scoped_by_tenant() {
    let db = AppDb::in_memory().unwrap();
    let connection = db.lock().unwrap();
    let org = id();
    let event = id();
    connection
        .execute(
            "INSERT INTO organizations(id,name,slug) VALUES(?1,'A','org-a')",
            [&org],
        )
        .unwrap();
    let insert = "INSERT INTO sync_inbox(organization_id,event_id,payload_hash,status) VALUES(?1,?2,?3,'RECEIVED')";
    connection
        .execute(insert, params![org, event, "a".repeat(64)])
        .unwrap();
    assert!(connection
        .execute(insert, params![org, event, "a".repeat(64)])
        .is_err());
    assert!(connection
        .execute(insert, params![id(), id(), "a".repeat(64)])
        .is_err());
}

#[test]
fn failed_future_migration_rolls_back_without_partial_schema() {
    let mut connection = Connection::open_in_memory().unwrap();
    migrations::apply(&mut connection, MIGRATIONS).unwrap();
    let mut definitions: Vec<Migration> = MIGRATIONS
        .iter()
        .map(|migration| Migration {
            version: migration.version,
            name: migration.name,
            sql: migration.sql,
        })
        .collect();
    definitions.push(Migration {
        version: 22,
        name: "broken",
        sql: "CREATE TABLE partial_cloud(id TEXT); INSERT INTO absent_table VALUES(1);",
    });
    assert!(migrations::apply(&mut connection, &definitions).is_err());
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM _schema_version", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        21
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='partial_cloud'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn postgres_schema_is_default_deny_and_has_tenant_scoped_rls() {
    let sql = include_str!("../../supabase/migrations/202610090001_phase_8a_cloud_foundation.sql")
        .to_ascii_lowercase();
    for table in [
        "organizations",
        "organization_memberships",
        "branches",
        "branch_memberships",
        "local_identity_links",
        "devices",
        "sync_events",
        "sync_receipts",
    ] {
        assert!(sql.contains(&format!(
            "alter table app.{table} enable row level security"
        )));
        assert!(sql.contains(&format!("alter table app.{table} force row level security")));
    }
    assert!(sql.contains("revoke all on all tables in schema app from anon, authenticated"));
    assert!(sql.contains("app.is_active_org_member"));
    assert!(sql.contains("app.is_branch_member"));
    assert!(sql.contains("app.owns_active_device"));
    assert!(!sql.contains("service_role"));
}

#[test]
fn postgres_phase8c_rpcs_authenticate_authorize_and_verify_payloads() {
    let sql =
        include_str!("../../supabase/migrations/202610100002_phase_8c_security_integration.sql")
            .to_ascii_lowercase();
    assert!(sql.contains("auth.uid()"));
    assert!(sql.contains("app.verify_sync_payload_hash"));
    assert!(sql.contains("extensions.digest"));
    assert!(sql.contains("create trigger sync_events_payload_integrity"));
    for function in [
        "bootstrap_sync_organization",
        "grant_sync_membership",
        "revoke_sync_device",
        "revoke_sync_member",
    ] {
        assert!(sql.contains(&format!("function public.{function}")));
        assert!(sql.contains(&format!("grant execute on function public.{function}")));
    }
    assert!(sql.contains("active organization owner required"));
    assert!(sql.contains("owner membership cannot be revoked here"));
    assert!(sql.contains("device revocation denied"));
    assert!(sql.contains("from public,anon"));
    assert!(!sql.contains("service_role"));
}

#[test]
fn desktop_cloud_surface_contains_no_endpoint_or_privileged_secret() {
    let client = include_str!("../../src/lib/tauri/cloud.ts").to_ascii_lowercase();
    let page = include_str!("../../src/features/system/CanonicalSync.tsx").to_ascii_lowercase();
    let local_schema = include_str!("../migrations/018_cloud_foundation.sql").to_ascii_lowercase();
    for source in [&client, &page] {
        assert!(!source.contains("service_role"));
        assert!(!source.contains("supabase_url"));
        assert!(!source.contains("supabase_key"));
        assert!(!source.contains("http://"));
        assert!(!source.contains("https://"));
        assert!(!source.contains("localstorage"));
        assert!(!source.contains("sessionstorage"));
    }
    assert!(!local_schema.contains("user_credentials"));
    assert!(!local_schema.contains("password_hash"));
}
