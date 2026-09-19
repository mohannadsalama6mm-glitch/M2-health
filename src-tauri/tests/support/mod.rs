// Retained prototype contracts, NOT runtime compatibility tests.
// These schemas run only in a new in-memory database, never in the application DB.
use m2_health_lib::db::{
    connection::AppDb,
    migrations::{Migration, MIGRATIONS},
};

pub fn prototype_db() -> AppDb {
    let definitions = [
        Migration {
            version: 1,
            name: "branches",
            sql: MIGRATIONS[0].sql,
        },
        Migration {
            version: 2,
            name: "catalog_core",
            sql: MIGRATIONS[1].sql,
        },
        Migration {
            version: 3,
            name: "prototype_inventory",
            sql: include_str!("../../../docs/migration-proposals/obsolete_inventory.sql"),
        },
        Migration {
            version: 4,
            name: "prototype_sales",
            sql: include_str!("../../../docs/migration-proposals/sales.sql"),
        },
        Migration {
            version: 5,
            name: "prototype_partners",
            sql: include_str!("../../../docs/migration-proposals/partners_purchases.sql"),
        },
    ];
    AppDb::in_memory_with_migrations(&definitions).unwrap()
}
