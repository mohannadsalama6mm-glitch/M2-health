use crate::error::{AppError, ErrorCode};
use rusqlite::{Connection, TransactionBehavior};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}
pub const MIGRATIONS: &[Migration] = &[
    // Existing migration names remain authoritative, including catalog_source_actions at 007.
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
        name: "catalog_import",
        sql: include_str!("../../migrations/003_catalog_import.sql"),
    },
    Migration {
        version: 4,
        name: "inventory_core",
        sql: include_str!("../../migrations/004_inventory_core.sql"),
    },
    Migration {
        version: 5,
        name: "inventory_workflows",
        sql: include_str!("../../migrations/005_inventory_workflows.sql"),
    },
    Migration {
        version: 6,
        name: "stock_counts",
        sql: include_str!("../../migrations/006_stock_counts.sql"),
    },
    Migration {
        version: 7,
        name: "catalog_source_actions",
        sql: include_str!("../../migrations/007_catalog_source_actions.sql"),
    },
    Migration {
        version: 8,
        name: "inventory_alerts",
        sql: include_str!("../../migrations/008_inventory_alerts.sql"),
    },
    Migration {
        version: 9,
        name: "sales",
        sql: include_str!("../../migrations/009_sales.sql"),
    },
    Migration {
        version: 10,
        name: "partners",
        sql: include_str!("../../migrations/010_partners.sql"),
    },
    Migration {
        version: 11,
        name: "purchasing",
        sql: include_str!("../../migrations/011_purchasing.sql"),
    },
    Migration {
        version: 12,
        name: "purchase_receiving",
        sql: include_str!("../../migrations/012_purchase_receiving.sql"),
    },
    Migration {
        version: 13,
        name: "purchase_returns",
        sql: include_str!("../../migrations/013_purchase_returns.sql"),
    },
    Migration {
        version: 14,
        name: "customer_sales",
        sql: include_str!("../../migrations/014_customer_sales.sql"),
    },
    Migration {
        version: 15,
        name: "finance_core",
        sql: include_str!("../../migrations/015_finance_core.sql"),
    },
    Migration {
        version: 16,
        name: "identity_foundation",
        sql: include_str!("../../migrations/016_identity_foundation.sql"),
    },
    Migration {
        version: 17,
        name: "security_events",
        sql: include_str!("../../migrations/017_security_events.sql"),
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
