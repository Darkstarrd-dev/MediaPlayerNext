use anyhow::{anyhow, Result};
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

    if current_version > latest_schema_version() {
        return Err(anyhow!(
            "database schema version {current_version} is newer than supported {}",
            latest_schema_version()
        ));
    }

    for migration in MIGRATIONS {
        if migration.version <= current_version {
            continue;
        }

        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.pragma_update(None, "user_version", migration.version)?;
        transaction.commit()?;
    }

    validate_schema(connection)?;

    Ok(())
}

fn validate_schema(connection: &Connection) -> Result<()> {
    let version = current_schema_version(connection)?;

    if version >= 1 {
        ensure_table_columns(
            connection,
            "libraries",
            &[
                "id",
                "root_path",
                "library_type",
                "scan_mode",
                "created_at",
                "updated_at",
            ],
        )?;
        ensure_table_columns(
            connection,
            "sources",
            &[
                "id",
                "library_id",
                "normalized_path",
                "file_name",
                "ext",
                "kind",
                "size",
                "mtime_ms",
                "fingerprint",
                "exists_flag",
                "last_seen_at",
            ],
        )?;
        ensure_table_columns(
            connection,
            "media_assets",
            &[
                "id",
                "source_kind",
                "source_ref_id",
                "mime",
                "width",
                "height",
                "duration_ms",
                "codec_info_json",
                "orientation",
                "created_at",
            ],
        )?;
        ensure_table_columns(
            connection,
            "tasks",
            &[
                "id",
                "task_type",
                "state",
                "current",
                "total",
                "message",
                "error_code",
                "error_message",
                "started_at",
                "finished_at",
            ],
        )?;
        ensure_index_exists(connection, "idx_sources_library_path")?;
        ensure_index_exists(connection, "idx_media_assets_source_ref")?;
        ensure_foreign_key_exists(connection, "sources", "library_id", "libraries", "id")?;
    }

    if version >= 2 {
        ensure_table_columns(
            connection,
            "archives",
            &[
                "id",
                "source_id",
                "archive_type",
                "normalized_zip_path",
                "page_count",
                "cover_entry_id",
                "status",
            ],
        )?;
        ensure_table_columns(
            connection,
            "archive_entries",
            &[
                "id",
                "archive_id",
                "entry_path",
                "entry_name",
                "page_index",
                "media_kind",
                "width",
                "height",
                "compressed_size",
                "uncompressed_size",
                "crc32",
            ],
        )?;
        ensure_table_columns(
            connection,
            "thumbnails",
            &[
                "thumbnail_key",
                "asset_id",
                "profile",
                "width",
                "height",
                "format",
                "disk_path",
                "byte_size",
                "state",
                "updated_at",
            ],
        )?;
        ensure_index_exists(connection, "idx_archives_source_id")?;
        ensure_index_exists(connection, "idx_archive_entries_archive_path")?;
        ensure_index_exists(connection, "idx_thumbnails_asset_id")?;
        ensure_foreign_key_exists(connection, "archives", "source_id", "sources", "id")?;
        ensure_foreign_key_exists(
            connection,
            "archive_entries",
            "archive_id",
            "archives",
            "id",
        )?;
        ensure_foreign_key_exists(connection, "thumbnails", "asset_id", "media_assets", "id")?;
    }

    ensure_foreign_key_integrity(connection)?;

    Ok(())
}

fn ensure_table_columns(
    connection: &Connection,
    table: &str,
    required_columns: &[&str],
) -> Result<()> {
    ensure_table_exists(connection, table)?;

    let mut statement = connection.prepare(&format!("pragma table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut actual_columns = Vec::new();
    for column in columns {
        actual_columns.push(column?);
    }

    for required_column in required_columns {
        if actual_columns
            .iter()
            .any(|column| column == required_column)
        {
            continue;
        }

        return Err(anyhow!(
            "required column missing for current schema: {table}.{required_column}"
        ));
    }

    Ok(())
}

fn ensure_table_exists(connection: &Connection, table: &str) -> Result<()> {
    let exists = connection.query_row(
        "select exists(select 1 from sqlite_master where type = 'table' and name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )?;

    if exists == 1 {
        return Ok(());
    }

    Err(anyhow!(
        "required table missing for current schema: {table}"
    ))
}

fn ensure_index_exists(connection: &Connection, index: &str) -> Result<()> {
    let exists = connection.query_row(
        "select exists(select 1 from sqlite_master where type = 'index' and name = ?1)",
        [index],
        |row| row.get::<_, i64>(0),
    )?;

    if exists == 1 {
        return Ok(());
    }

    Err(anyhow!(
        "required index missing for current schema: {index}"
    ))
}

fn ensure_foreign_key_exists(
    connection: &Connection,
    table: &str,
    from_column: &str,
    referenced_table: &str,
    referenced_column: &str,
) -> Result<()> {
    ensure_table_exists(connection, table)?;

    let mut statement = connection.prepare(&format!("pragma foreign_key_list({table})"))?;
    let foreign_keys = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    for foreign_key in foreign_keys {
        let (actual_referenced_table, actual_from_column, actual_referenced_column) = foreign_key?;
        if actual_from_column == from_column
            && actual_referenced_table == referenced_table
            && actual_referenced_column == referenced_column
        {
            return Ok(());
        }
    }

    Err(anyhow!(
        "required foreign key missing for current schema: {table}.{from_column} -> {referenced_table}.{referenced_column}"
    ))
}

fn ensure_foreign_key_integrity(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("pragma foreign_key_check")?;
    let violations = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    if let Some(violation) = violations.into_iter().next() {
        let (table, rowid, parent_table) = violation?;
        return Err(anyhow!(
            "foreign key integrity violation for current schema: {table} rowid={rowid} references missing parent table {parent_table}"
        ));
    }

    Ok(())
}
