use app_core::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
    SourceRepository, TaskRepository,
};
use media_db::{current_schema_version, latest_schema_version, DatabaseLocation, MediaDatabase};
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
    LibraryRecord, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind, SourceRecord, TaskId,
    TaskKind, TaskRecord, TaskState,
};
use tempfile::NamedTempFile;

#[test]
fn runs_migrations_for_new_database() {
    let database = MediaDatabase::open(DatabaseLocation::InMemory)
        .expect("in-memory database should open with migrations");

    let version =
        current_schema_version(database.connection()).expect("schema version should load");

    assert_eq!(version, latest_schema_version());
}

#[test]
fn rerunning_migrations_is_stable() {
    let temp_file = NamedTempFile::new().expect("temporary file should be created");

    let database = MediaDatabase::open(DatabaseLocation::File(temp_file.path()))
        .expect("file database should open with migrations");
    let first_version =
        current_schema_version(database.connection()).expect("version should exist");
    drop(database);

    let reopened = MediaDatabase::open(DatabaseLocation::File(temp_file.path()))
        .expect("reopening file database should succeed");
    let second_version =
        current_schema_version(reopened.connection()).expect("version should remain accessible");

    assert_eq!(first_version, latest_schema_version());
    assert_eq!(second_version, latest_schema_version());
}

#[test]
fn upgrades_fixture_database_from_n_minus_1() {
    let temp_file = NamedTempFile::new().expect("temporary file should be created");
    let fixture_sql = include_str!("fixtures/schema_v1_fixture.sql");

    {
        let connection = rusqlite::Connection::open(temp_file.path())
            .expect("fixture database should open before upgrade");
        connection
            .execute_batch(fixture_sql)
            .expect("fixture sql should be applied");
    }

    let upgraded = MediaDatabase::open(DatabaseLocation::File(temp_file.path()))
        .expect("fixture database should upgrade to latest version");

    let version =
        current_schema_version(upgraded.connection()).expect("upgraded version should load");
    let thumbnails_exists: String = upgraded
        .connection()
        .query_row(
            "select name from sqlite_master where type = 'table' and name = 'thumbnails'",
            [],
            |row| row.get(0),
        )
        .expect("thumbnails table should exist after upgrade");

    assert_eq!(version, latest_schema_version());
    assert_eq!(thumbnails_exists, "thumbnails");
}

#[test]
fn upserts_and_queries_core_records() {
    let database = MediaDatabase::open(DatabaseLocation::InMemory)
        .expect("in-memory database should open with migrations");
    let repositories = database.repositories();

    let library = LibraryRecord {
        id: LibraryId("library_primary".to_string()),
        root_path: "Z:/Library".to_string(),
        library_type: "images".to_string(),
        scan_mode: "full".to_string(),
        created_at: "2026-03-07T00:00:00Z".to_string(),
        updated_at: "2026-03-07T00:00:00Z".to_string(),
    };
    LibraryRepository::upsert(&repositories, &library).expect("library upsert should succeed");

    let fetched = LibraryRepository::get(&repositories, &library.id)
        .expect("library fetch should succeed")
        .expect("library should exist after insert");

    assert_eq!(fetched.root_path, library.root_path);
    assert!(LibraryRepository::exists(&repositories, &library.id)
        .expect("library exists should succeed"));

    let source = SourceRecord {
        id: SourceId("source_primary".to_string()),
        library_id: library.id.clone(),
        normalized_path: "z:/library/item-001.png".to_string(),
        file_name: "item-001.png".to_string(),
        ext: "png".to_string(),
        kind: SourceKind::Image,
        size: 1024,
        mtime_ms: 1_700_000_000_000,
        fingerprint: Some("fp-001".to_string()),
        exists: true,
        last_seen_at: "2026-03-07T00:01:00Z".to_string(),
    };
    SourceRepository::upsert(&repositories, &source).expect("source upsert should succeed");

    let asset = MediaAssetRecord {
        id: AssetId("asset_primary".to_string()),
        source_kind: MediaSourceKind::File,
        source_ref_id: source.id.0.clone(),
        mime: "image/png".to_string(),
        width: Some(1280),
        height: Some(720),
        duration_ms: None,
        codec_info_json: None,
        orientation: Some(1),
        created_at: "2026-03-07T00:02:00Z".to_string(),
    };
    AssetRepository::upsert(&repositories, &asset).expect("asset upsert should succeed");

    let task = TaskRecord {
        id: TaskId("task_primary".to_string()),
        task_type: TaskKind::Scan,
        state: TaskState::Running,
        current: 1,
        total: Some(10),
        message: Some("scan started".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some("2026-03-07T00:03:00Z".to_string()),
        finished_at: None,
    };
    TaskRepository::upsert(&repositories, &task).expect("task upsert should succeed");

    assert!(
        SourceRepository::exists(&repositories, &source.id).expect("source exists should succeed")
    );
    assert!(AssetRepository::exists(&repositories, &asset.id).expect("asset exists should succeed"));
    assert!(TaskRepository::exists(&repositories, &task.id).expect("task exists should succeed"));
}

#[test]
fn inserts_and_counts_more_than_one_thousand_sources() {
    let database = MediaDatabase::open(DatabaseLocation::InMemory)
        .expect("in-memory database should open with migrations");
    let repositories = database.repositories();

    let library = LibraryRecord {
        id: LibraryId("library_bulk".to_string()),
        root_path: "Z:/Bulk".to_string(),
        library_type: "images".to_string(),
        scan_mode: "full".to_string(),
        created_at: "2026-03-07T00:00:00Z".to_string(),
        updated_at: "2026-03-07T00:00:00Z".to_string(),
    };
    LibraryRepository::upsert(&repositories, &library).expect("library upsert should succeed");

    for index in 0..1_000_u32 {
        let source = SourceRecord {
            id: SourceId(format!("source_bulk_{index:04}")),
            library_id: library.id.clone(),
            normalized_path: format!("z:/bulk/item-{index:04}.png"),
            file_name: format!("item-{index:04}.png"),
            ext: "png".to_string(),
            kind: SourceKind::Image,
            size: 4_096,
            mtime_ms: 1_700_000_000_000 + i64::from(index),
            fingerprint: None,
            exists: true,
            last_seen_at: "2026-03-07T00:00:00Z".to_string(),
        };

        SourceRepository::upsert(&repositories, &source)
            .expect("bulk source upsert should succeed");
    }

    let count = repositories.count().expect("source count should succeed");
    assert_eq!(count, 1_000);
}

#[test]
fn transaction_rolls_back_on_error() {
    let mut database = MediaDatabase::open(DatabaseLocation::InMemory)
        .expect("in-memory database should open with migrations");

    let transaction = database
        .connection_mut()
        .unchecked_transaction()
        .expect("transaction should start");

    transaction
        .execute(
            "insert into libraries (id, root_path, library_type, scan_mode, created_at, updated_at) values (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "library_tx",
                "Z:/Tx",
                "images",
                "full",
                "2026-03-07T00:00:00Z",
                "2026-03-07T00:00:00Z",
            ),
        )
        .expect("insert inside transaction should succeed");

    transaction.rollback().expect("rollback should succeed");

    let count: u64 = database
        .connection()
        .query_row(
            "select count(*) from libraries where id = 'library_tx'",
            [],
            |row| row.get(0),
        )
        .expect("count query should succeed after rollback");

    assert_eq!(count, 0);
}

#[test]
fn replaces_and_lists_archive_entries() {
    let database = MediaDatabase::open(DatabaseLocation::InMemory)
        .expect("in-memory database should open with migrations");
    let repositories = database.repositories();

    let library = LibraryRecord {
        id: LibraryId("library_archive_entries".to_string()),
        root_path: "Z:/ArchiveLibrary".to_string(),
        library_type: "images".to_string(),
        scan_mode: "full".to_string(),
        created_at: "2026-03-07T00:00:00Z".to_string(),
        updated_at: "2026-03-07T00:00:00Z".to_string(),
    };
    LibraryRepository::upsert(&repositories, &library).expect("library upsert should succeed");

    let source = SourceRecord {
        id: SourceId("source_archive_entries".to_string()),
        library_id: library.id.clone(),
        normalized_path: "z:/archivelibrary/chapter.cbz".to_string(),
        file_name: "chapter.cbz".to_string(),
        ext: "cbz".to_string(),
        kind: SourceKind::Archive,
        size: 4_096,
        mtime_ms: 1_700_000_000_100,
        fingerprint: Some("archive-fp".to_string()),
        exists: true,
        last_seen_at: "2026-03-07T00:01:00Z".to_string(),
    };
    SourceRepository::upsert(&repositories, &source).expect("source upsert should succeed");

    let archive = ArchiveRecord {
        id: ArchiveId("archive_primary".to_string()),
        source_id: source.id.clone(),
        archive_type: "cbz".to_string(),
        normalized_zip_path: Some(source.normalized_path.clone()),
        page_count: Some(2),
        cover_entry_id: Some(ArchiveEntryId("entry_cover".to_string())),
        status: "indexed".to_string(),
    };
    ArchiveRepository::upsert(&repositories, &archive).expect("archive upsert should succeed");

    let first_entries = vec![
        ArchiveEntryRecord {
            id: ArchiveEntryId("entry_cover".to_string()),
            archive_id: archive.id.clone(),
            entry_path: "001-cover.png".to_string(),
            entry_name: "001-cover.png".to_string(),
            page_index: 0,
            media_kind: "image".to_string(),
            width: None,
            height: None,
            compressed_size: Some(100),
            uncompressed_size: Some(120),
            crc32: Some(1),
        },
        ArchiveEntryRecord {
            id: ArchiveEntryId("entry_page_2".to_string()),
            archive_id: archive.id.clone(),
            entry_path: "002-page.png".to_string(),
            entry_name: "002-page.png".to_string(),
            page_index: 1,
            media_kind: "image".to_string(),
            width: None,
            height: None,
            compressed_size: Some(101),
            uncompressed_size: Some(121),
            crc32: Some(2),
        },
    ];
    ArchiveEntryRepository::replace_for_archive(&repositories, &archive.id, &first_entries)
        .expect("archive entries should be stored");

    let replaced_entries = vec![ArchiveEntryRecord {
        id: ArchiveEntryId("entry_page_9".to_string()),
        archive_id: archive.id.clone(),
        entry_path: "009-page.png".to_string(),
        entry_name: "009-page.png".to_string(),
        page_index: 0,
        media_kind: "image".to_string(),
        width: None,
        height: None,
        compressed_size: Some(200),
        uncompressed_size: Some(220),
        crc32: Some(9),
    }];
    ArchiveEntryRepository::replace_for_archive(&repositories, &archive.id, &replaced_entries)
        .expect("archive entries should be replaced");

    let fetched_archive = ArchiveRepository::get_by_source(&repositories, &source.id)
        .expect("archive by source query should succeed")
        .expect("archive should exist");
    let fetched_entries = ArchiveEntryRepository::list_by_archive(&repositories, &archive.id)
        .expect("archive entries query should succeed");

    assert_eq!(fetched_archive.id, archive.id);
    assert_eq!(fetched_entries.len(), 1);
    assert_eq!(fetched_entries[0].entry_path, "009-page.png");
}
