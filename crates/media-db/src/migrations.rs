use anyhow::Result;
use rusqlite::Connection;

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("migrations/0001_init_core.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("migrations/0002_init_archive_and_thumb.sql"),
    },
];

pub fn latest_schema_version() -> i32 {
    MIGRATIONS.last().map_or(0, |migration| migration.version)
}

pub fn current_schema_version(connection: &Connection) -> Result<i32> {
    let version = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(version)
}

pub fn run_migrations(connection: &mut Connection) -> Result<()> {
    let current_version = current_schema_version(connection)?;

    for migration in MIGRATIONS {
        if migration.version <= current_version {
            continue;
        }

        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.pragma_update(None, "user_version", migration.version)?;
        transaction.commit()?;
    }

    Ok(())
}
