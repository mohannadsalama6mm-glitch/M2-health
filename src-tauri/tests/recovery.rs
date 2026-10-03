use m2_health_lib::db::{
    connection::AppDb,
    migrations::{self, Migration, MIGRATIONS},
};
use rusqlite::Connection;

#[test]
fn canonical_v4_reopens_without_reapplying_or_creating_prototype_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("recovery.db");
    let expected;
    {
        let mut c = Connection::open(&path).unwrap();
        migrations::apply(&mut c, &MIGRATIONS[..4]).unwrap();
        c.execute(
            "INSERT INTO branches(id,code,name) VALUES('preserved','KEEP','Preserved branch')",
            [],
        )
        .unwrap();
        expected = c
            .query_row(
                "SELECT group_concat(applied_at) FROM _schema_version WHERE version <= 4",
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap();
    }
    for _ in 0..2 {
        let db = AppDb::open(&path).unwrap();
        let c = db.lock().unwrap();
        assert_eq!(
            c.query_row(
                "SELECT group_concat(applied_at) FROM _schema_version WHERE version <= 4",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            c.query_row("SELECT name FROM branches WHERE id='preserved'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
            "Preserved branch"
        );
        assert_eq!(c.query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('stock_movements','stock_balances','stock_counts','purchases')", [], |r| r.get::<_,i64>(0)).unwrap(), 0);
    }
}

#[test]
fn future_extension_of_v11_applies_once_and_unknown_version_is_refused() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::run(&mut c).unwrap();
    let mut extended: Vec<Migration> = MIGRATIONS
        .iter()
        .map(|m| Migration {
            version: m.version,
            name: m.name,
            sql: m.sql,
        })
        .collect();
    extended.push(Migration {
        version: 12,
        name: "test_only_extension",
        sql:
            "CREATE TABLE recovery_marker(value TEXT); INSERT INTO recovery_marker VALUES('keep');",
    });
    migrations::apply(&mut c, &extended).unwrap();
    migrations::apply(&mut c, &extended).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM recovery_marker", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(
        c.query_row("SELECT value FROM recovery_marker", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM _schema_version", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        12
    );
}
