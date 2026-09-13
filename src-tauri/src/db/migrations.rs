use crate::error::{AppError, ErrorCode};
use rusqlite::{Connection, TransactionBehavior};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "branches",
        sql: include_str!("../../migrations/001_branches.sql"),
    },
    Migration {
        version: 2,
        name: "catalog_core",
        sql: include_str!("../../migrations/002_catalog_core.sql"),
    },
    Migration {
        version: 3,
        name: "inventory",
        sql: include_str!("../../migrations/003_inventory.sql"),
    },
    Migration {
        version: 4,
        name: "sales",
        sql: include_str!("../../migrations/004_sales.sql"),
    },
    Migration {
        version: 5,
        name: "partners_purchases",
        sql: include_str!("../../migrations/005_partners_purchases.sql"),
    },
];

pub fn run(connection: &mut Connection) -> Result<(), AppError> {
    apply(connection, MIGRATIONS)
}

/// One IMMEDIATE transaction serializes startup across processes and rolls back all
/// pending schema/metadata changes on any failure. Unknown/newer schemas are refused.
pub fn apply(connection: &mut Connection, migrations: &[Migration]) -> Result<(), AppError> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(AppError::migration)?;
    tx.execute_batch("CREATE TABLE IF NOT EXISTS _schema_version (version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')));").map_err(AppError::migration)?;
    let applied = {
        let mut stmt = tx
            .prepare("SELECT version, name FROM _schema_version ORDER BY version")
            .map_err(AppError::migration)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(AppError::migration)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(AppError::migration)?
    };
    for (index, migration) in migrations.iter().enumerate() {
        if migration.version != index as i64 + 1 {
            return Err(AppError::new(
                ErrorCode::Migration,
                "Migration definitions must be sequential.",
            ));
        }
    }
    for (index, (version, name)) in applied.iter().enumerate() {
        if migrations
            .get(index)
            .is_none_or(|m| m.version != *version || m.name != name)
        {
            return Err(AppError::new(ErrorCode::Migration, "This database schema is not compatible with this application version. No data was changed."));
        }
    }
    for migration in &migrations[applied.len()..] {
        tx.execute_batch(migration.sql)
            .map_err(AppError::migration)?;
        tx.execute(
            "INSERT INTO _schema_version(version, name) VALUES (?1, ?2)",
            (migration.version, migration.name),
        )
        .map_err(AppError::migration)?;
    }
    tx.commit().map_err(AppError::migration)?;
    Ok(())
}
