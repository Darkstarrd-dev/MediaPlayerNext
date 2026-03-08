use anyhow::Result;
use app_core::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
    SourceRepository, TaskRepository, ThumbnailRepository,
};
use rusqlite::{named_params, Connection, OptionalExtension};
use shared_model::{
    AppErrorCode, ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
    LibraryRecord, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind, SourceRecord, TaskId,
    TaskKind, TaskRecord, TaskState, ThumbnailKey, ThumbnailRecord,
};

pub struct SqliteRepositories<'a> {
    connection: &'a Connection,
}

impl<'a> SqliteRepositories<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
}

impl LibraryRepository for SqliteRepositories<'_> {
    fn exists(&self, library_id: &LibraryId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from libraries where id = :id limit 1",
                named_params! { ":id": library_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, library: &LibraryRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into libraries (id, root_path, library_type, scan_mode, created_at, updated_at)
            values (:id, :root_path, :library_type, :scan_mode, :created_at, :updated_at)
            on conflict(id) do update set
              root_path = excluded.root_path,
              library_type = excluded.library_type,
              scan_mode = excluded.scan_mode,
              updated_at = excluded.updated_at
            ",
            named_params! {
                ":id": library.id.0,
                ":root_path": library.root_path,
                ":library_type": library.library_type,
                ":scan_mode": library.scan_mode,
                ":created_at": library.created_at,
                ":updated_at": library.updated_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, library_id: &LibraryId) -> Result<Option<LibraryRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, root_path, library_type, scan_mode, created_at, updated_at
                from libraries
                where id = :id
                ",
                named_params! { ":id": library_id.0 },
                |row| {
                    Ok(LibraryRecord {
                        id: LibraryId(row.get::<_, String>(0)?),
                        root_path: row.get(1)?,
                        library_type: row.get(2)?,
                        scan_mode: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }

    fn list(&self) -> Result<Vec<LibraryRecord>> {
        let mut statement = self.connection.prepare(
            "
            select id, root_path, library_type, scan_mode, created_at, updated_at
            from libraries
            order by updated_at desc, id asc
            ",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(LibraryRecord {
                id: LibraryId(row.get::<_, String>(0)?),
                root_path: row.get(1)?,
                library_type: row.get(2)?,
                scan_mode: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn delete(&self, library_id: &LibraryId) -> Result<()> {
        self.connection.execute(
            "
            delete from thumbnails
            where asset_id in (
              select id from media_assets
              where (source_kind = 'file' and source_ref_id in (
                select id from sources where library_id = :library_id
              ))
              or (source_kind = 'archive_entry' and source_ref_id in (
                select id from archive_entries where archive_id in (
                  select id from archives where source_id in (
                    select id from sources where library_id = :library_id
                  )
                )
              ))
            )
            ",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "
            delete from media_assets
            where (source_kind = 'file' and source_ref_id in (
              select id from sources where library_id = :library_id
            ))
            or (source_kind = 'archive_entry' and source_ref_id in (
              select id from archive_entries where archive_id in (
                select id from archives where source_id in (
                  select id from sources where library_id = :library_id
                )
              )
            ))
            ",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "delete from archive_entries where archive_id in (select id from archives where source_id in (select id from sources where library_id = :library_id))",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "delete from archives where source_id in (select id from sources where library_id = :library_id)",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "delete from sources where library_id = :library_id",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "delete from libraries where id = :library_id",
            named_params! { ":library_id": library_id.0 },
        )?;

        Ok(())
    }
}

impl SourceRepository for SqliteRepositories<'_> {
    fn exists(&self, source_id: &SourceId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from sources where id = :id limit 1",
                named_params! { ":id": source_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, source: &SourceRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into sources (
              id, library_id, normalized_path, file_name, ext, kind, size, mtime_ms,
              fingerprint, exists_flag, last_seen_at
            ) values (
              :id, :library_id, :normalized_path, :file_name, :ext, :kind, :size, :mtime_ms,
              :fingerprint, :exists_flag, :last_seen_at
            )
            on conflict(id) do update set
              library_id = excluded.library_id,
              normalized_path = excluded.normalized_path,
              file_name = excluded.file_name,
              ext = excluded.ext,
              kind = excluded.kind,
              size = excluded.size,
              mtime_ms = excluded.mtime_ms,
              fingerprint = excluded.fingerprint,
              exists_flag = excluded.exists_flag,
              last_seen_at = excluded.last_seen_at
            ",
            named_params! {
                ":id": source.id.0,
                ":library_id": source.library_id.0,
                ":normalized_path": source.normalized_path,
                ":file_name": source.file_name,
                ":ext": source.ext,
                ":kind": source_kind_to_db(&source.kind),
                ":size": source.size,
                ":mtime_ms": source.mtime_ms,
                ":fingerprint": source.fingerprint,
                ":exists_flag": source.exists,
                ":last_seen_at": source.last_seen_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, source_id: &SourceId) -> Result<Option<SourceRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, library_id, normalized_path, file_name, ext, kind, size, mtime_ms,
                       fingerprint, exists_flag, last_seen_at
                from sources
                where id = :id
                ",
                named_params! { ":id": source_id.0 },
                |row| {
                    Ok(SourceRecord {
                        id: SourceId(row.get::<_, String>(0)?),
                        library_id: LibraryId(row.get::<_, String>(1)?),
                        normalized_path: row.get(2)?,
                        file_name: row.get(3)?,
                        ext: row.get(4)?,
                        kind: source_kind_from_db(&row.get::<_, String>(5)?),
                        size: row.get(6)?,
                        mtime_ms: row.get(7)?,
                        fingerprint: row.get(8)?,
                        exists: row.get::<_, bool>(9)?,
                        last_seen_at: row.get(10)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }

    fn count(&self) -> Result<u64> {
        let count = self
            .connection
            .query_row("select count(*) from sources", [], |row| {
                row.get::<_, i64>(0)
            })?;
        Ok(count as u64)
    }

    fn count_by_library(&self, library_id: &LibraryId) -> Result<u64> {
        let count = self.connection.query_row(
            "select count(*) from sources where library_id = :library_id",
            named_params! { ":library_id": library_id.0 },
            |row| row.get::<_, i64>(0),
        )?;

        Ok(count as u64)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> Result<Vec<SourceRecord>> {
        let mut statement = self.connection.prepare(
            "
            select id, library_id, normalized_path, file_name, ext, kind, size, mtime_ms,
                   fingerprint, exists_flag, last_seen_at
            from sources
            where library_id = :library_id
            order by normalized_path asc
            ",
        )?;

        let rows = statement.query_map(named_params! { ":library_id": library_id.0 }, |row| {
            Ok(SourceRecord {
                id: SourceId(row.get::<_, String>(0)?),
                library_id: LibraryId(row.get::<_, String>(1)?),
                normalized_path: row.get(2)?,
                file_name: row.get(3)?,
                ext: row.get(4)?,
                kind: source_kind_from_db(&row.get::<_, String>(5)?),
                size: row.get(6)?,
                mtime_ms: row.get(7)?,
                fingerprint: row.get(8)?,
                exists: row.get::<_, bool>(9)?,
                last_seen_at: row.get(10)?,
            })
        })?;

        let mut sources = Vec::new();
        for row in rows {
            sources.push(row?);
        }

        Ok(sources)
    }
}

impl ArchiveRepository for SqliteRepositories<'_> {
    fn exists(&self, archive_id: &ArchiveId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from archives where id = :id limit 1",
                named_params! { ":id": archive_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, archive: &ArchiveRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into archives (
              id, source_id, archive_type, normalized_zip_path, page_count, cover_entry_id, status
            ) values (
              :id, :source_id, :archive_type, :normalized_zip_path, :page_count, :cover_entry_id, :status
            )
            on conflict(id) do update set
              source_id = excluded.source_id,
              archive_type = excluded.archive_type,
              normalized_zip_path = excluded.normalized_zip_path,
              page_count = excluded.page_count,
              cover_entry_id = excluded.cover_entry_id,
              status = excluded.status
            ",
            named_params! {
                ":id": archive.id.0,
                ":source_id": archive.source_id.0,
                ":archive_type": archive.archive_type,
                ":normalized_zip_path": archive.normalized_zip_path,
                ":page_count": archive.page_count,
                ":cover_entry_id": archive.cover_entry_id.as_ref().map(|value| value.0.as_str()),
                ":status": archive.status,
            },
        )?;

        Ok(())
    }

    fn get(&self, archive_id: &ArchiveId) -> Result<Option<ArchiveRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, source_id, archive_type, normalized_zip_path, page_count, cover_entry_id, status
                from archives
                where id = :id
                limit 1
                ",
                named_params! { ":id": archive_id.0 },
                |row| {
                    Ok(ArchiveRecord {
                        id: ArchiveId(row.get::<_, String>(0)?),
                        source_id: SourceId(row.get::<_, String>(1)?),
                        archive_type: row.get(2)?,
                        normalized_zip_path: row.get(3)?,
                        page_count: row.get(4)?,
                        cover_entry_id: row.get::<_, Option<String>>(5)?.map(ArchiveEntryId),
                        status: row.get(6)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }

    fn get_by_source(&self, source_id: &SourceId) -> Result<Option<ArchiveRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, source_id, archive_type, normalized_zip_path, page_count, cover_entry_id, status
                from archives
                where source_id = :source_id
                limit 1
                ",
                named_params! { ":source_id": source_id.0 },
                |row| {
                    Ok(ArchiveRecord {
                        id: ArchiveId(row.get::<_, String>(0)?),
                        source_id: SourceId(row.get::<_, String>(1)?),
                        archive_type: row.get(2)?,
                        normalized_zip_path: row.get(3)?,
                        page_count: row.get(4)?,
                        cover_entry_id: row.get::<_, Option<String>>(5)?.map(ArchiveEntryId),
                        status: row.get(6)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}

impl ArchiveEntryRepository for SqliteRepositories<'_> {
    fn replace_for_archive(
        &self,
        archive_id: &ArchiveId,
        entries: &[ArchiveEntryRecord],
    ) -> Result<()> {
        self.connection.execute(
            "delete from archive_entries where archive_id = :archive_id",
            named_params! { ":archive_id": archive_id.0 },
        )?;

        for entry in entries {
            self.connection.execute(
                "
                insert into archive_entries (
                  id, archive_id, entry_path, entry_name, page_index, media_kind,
                  width, height, compressed_size, uncompressed_size, crc32
                ) values (
                  :id, :archive_id, :entry_path, :entry_name, :page_index, :media_kind,
                  :width, :height, :compressed_size, :uncompressed_size, :crc32
                )
                ",
                named_params! {
                    ":id": entry.id.0,
                    ":archive_id": entry.archive_id.0,
                    ":entry_path": entry.entry_path,
                    ":entry_name": entry.entry_name,
                    ":page_index": entry.page_index,
                    ":media_kind": entry.media_kind,
                    ":width": entry.width,
                    ":height": entry.height,
                    ":compressed_size": entry.compressed_size,
                    ":uncompressed_size": entry.uncompressed_size,
                    ":crc32": entry.crc32,
                },
            )?;
        }

        Ok(())
    }

    fn get(&self, archive_entry_id: &ArchiveEntryId) -> Result<Option<ArchiveEntryRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, archive_id, entry_path, entry_name, page_index, media_kind,
                       width, height, compressed_size, uncompressed_size, crc32
                from archive_entries
                where id = :id
                limit 1
                ",
                named_params! { ":id": archive_entry_id.0 },
                |row| {
                    Ok(ArchiveEntryRecord {
                        id: ArchiveEntryId(row.get::<_, String>(0)?),
                        archive_id: ArchiveId(row.get::<_, String>(1)?),
                        entry_path: row.get(2)?,
                        entry_name: row.get(3)?,
                        page_index: row.get(4)?,
                        media_kind: row.get(5)?,
                        width: row.get(6)?,
                        height: row.get(7)?,
                        compressed_size: row.get(8)?,
                        uncompressed_size: row.get(9)?,
                        crc32: row.get(10)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }

    fn list_by_archive(&self, archive_id: &ArchiveId) -> Result<Vec<ArchiveEntryRecord>> {
        let mut statement = self.connection.prepare(
            "
            select id, archive_id, entry_path, entry_name, page_index, media_kind,
                   width, height, compressed_size, uncompressed_size, crc32
            from archive_entries
            where archive_id = :archive_id
            order by page_index asc, entry_path asc
            ",
        )?;

        let rows = statement.query_map(named_params! { ":archive_id": archive_id.0 }, |row| {
            Ok(ArchiveEntryRecord {
                id: ArchiveEntryId(row.get::<_, String>(0)?),
                archive_id: ArchiveId(row.get::<_, String>(1)?),
                entry_path: row.get(2)?,
                entry_name: row.get(3)?,
                page_index: row.get(4)?,
                media_kind: row.get(5)?,
                width: row.get(6)?,
                height: row.get(7)?,
                compressed_size: row.get(8)?,
                uncompressed_size: row.get(9)?,
                crc32: row.get(10)?,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }
}

impl AssetRepository for SqliteRepositories<'_> {
    fn exists(&self, asset_id: &AssetId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from media_assets where id = :id limit 1",
                named_params! { ":id": asset_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, asset: &MediaAssetRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into media_assets (
              id, source_kind, source_ref_id, mime, width, height, duration_ms,
              codec_info_json, orientation, created_at
            ) values (
              :id, :source_kind, :source_ref_id, :mime, :width, :height, :duration_ms,
              :codec_info_json, :orientation, :created_at
            )
            on conflict(id) do update set
              source_kind = excluded.source_kind,
              source_ref_id = excluded.source_ref_id,
              mime = excluded.mime,
              width = excluded.width,
              height = excluded.height,
              duration_ms = excluded.duration_ms,
              codec_info_json = excluded.codec_info_json,
              orientation = excluded.orientation
            ",
            named_params! {
                ":id": asset.id.0,
                ":source_kind": media_source_kind_to_db(&asset.source_kind),
                ":source_ref_id": asset.source_ref_id,
                ":mime": asset.mime,
                ":width": asset.width,
                ":height": asset.height,
                ":duration_ms": asset.duration_ms,
                ":codec_info_json": asset.codec_info_json,
                ":orientation": asset.orientation,
                ":created_at": asset.created_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, asset_id: &AssetId) -> Result<Option<MediaAssetRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, source_kind, source_ref_id, mime, width, height, duration_ms,
                       codec_info_json, orientation, created_at
                from media_assets
                where id = :id
                limit 1
                ",
                named_params! { ":id": asset_id.0 },
                |row| {
                    Ok(MediaAssetRecord {
                        id: AssetId(row.get::<_, String>(0)?),
                        source_kind: media_source_kind_from_db(&row.get::<_, String>(1)?),
                        source_ref_id: row.get(2)?,
                        mime: row.get(3)?,
                        width: row.get(4)?,
                        height: row.get(5)?,
                        duration_ms: row.get(6)?,
                        codec_info_json: row.get(7)?,
                        orientation: row.get(8)?,
                        created_at: row.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}

impl TaskRepository for SqliteRepositories<'_> {
    fn exists(&self, task_id: &TaskId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from tasks where id = :id limit 1",
                named_params! { ":id": task_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, task: &TaskRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into tasks (
              id, task_type, state, current, total, message, error_code,
              error_message, started_at, finished_at
            ) values (
              :id, :task_type, :state, :current, :total, :message, :error_code,
              :error_message, :started_at, :finished_at
            )
            on conflict(id) do update set
              task_type = excluded.task_type,
              state = excluded.state,
              current = excluded.current,
              total = excluded.total,
              message = excluded.message,
              error_code = excluded.error_code,
              error_message = excluded.error_message,
              started_at = excluded.started_at,
              finished_at = excluded.finished_at
            ",
            named_params! {
                ":id": task.id.0,
                ":task_type": task_kind_to_db(&task.task_type),
                ":state": task_state_to_db(&task.state),
                ":current": task.current,
                ":total": task.total,
                ":message": task.message,
                ":error_code": task.error_code,
                ":error_message": task.error_message,
                ":started_at": task.started_at,
                ":finished_at": task.finished_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, task_id: &TaskId) -> Result<Option<TaskRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, task_type, state, current, total, message, error_code,
                       error_message, started_at, finished_at
                from tasks
                where id = :id
                ",
                named_params! { ":id": task_id.0 },
                |row| {
                    Ok(TaskRecord {
                        id: TaskId(row.get::<_, String>(0)?),
                        task_type: task_kind_from_db(&row.get::<_, String>(1)?),
                        state: task_state_from_db(&row.get::<_, String>(2)?),
                        current: row.get::<_, u64>(3)?,
                        total: row.get(4)?,
                        message: row.get(5)?,
                        error_code: row.get(6)?,
                        error_message: row.get(7)?,
                        started_at: row.get(8)?,
                        finished_at: row.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}

impl ThumbnailRepository for SqliteRepositories<'_> {
    fn exists(&self, thumbnail_key: &ThumbnailKey) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from thumbnails where thumbnail_key = :thumbnail_key limit 1",
                named_params! { ":thumbnail_key": thumbnail_key.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, thumbnail: &ThumbnailRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into thumbnails (
              thumbnail_key, asset_id, profile, width, height, format,
              disk_path, byte_size, state, updated_at
            ) values (
              :thumbnail_key, :asset_id, :profile, :width, :height, :format,
              :disk_path, :byte_size, :state, :updated_at
            )
            on conflict(thumbnail_key) do update set
              asset_id = excluded.asset_id,
              profile = excluded.profile,
              width = excluded.width,
              height = excluded.height,
              format = excluded.format,
              disk_path = excluded.disk_path,
              byte_size = excluded.byte_size,
              state = excluded.state,
              updated_at = excluded.updated_at
            ",
            named_params! {
                ":thumbnail_key": thumbnail.thumbnail_key.0,
                ":asset_id": thumbnail.asset_id.0,
                ":profile": thumbnail.profile,
                ":width": thumbnail.width,
                ":height": thumbnail.height,
                ":format": thumbnail.format,
                ":disk_path": thumbnail.disk_path,
                ":byte_size": thumbnail.byte_size,
                ":state": thumbnail.state,
                ":updated_at": thumbnail.updated_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, thumbnail_key: &ThumbnailKey) -> Result<Option<ThumbnailRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select thumbnail_key, asset_id, profile, width, height, format,
                       disk_path, byte_size, state, updated_at
                from thumbnails
                where thumbnail_key = :thumbnail_key
                limit 1
                ",
                named_params! { ":thumbnail_key": thumbnail_key.0 },
                |row| {
                    Ok(ThumbnailRecord {
                        thumbnail_key: ThumbnailKey(row.get::<_, String>(0)?),
                        asset_id: AssetId(row.get::<_, String>(1)?),
                        profile: row.get(2)?,
                        width: row.get(3)?,
                        height: row.get(4)?,
                        format: row.get(5)?,
                        disk_path: row.get(6)?,
                        byte_size: row.get(7)?,
                        state: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}

fn source_kind_to_db(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::Image => "image",
        SourceKind::Video => "video",
        SourceKind::Archive => "archive",
        SourceKind::Audio => "audio",
        SourceKind::Other => "other",
    }
}

fn source_kind_from_db(value: &str) -> SourceKind {
    match value {
        "image" => SourceKind::Image,
        "video" => SourceKind::Video,
        "archive" => SourceKind::Archive,
        "audio" => SourceKind::Audio,
        _ => SourceKind::Other,
    }
}

fn media_source_kind_to_db(kind: &MediaSourceKind) -> &'static str {
    match kind {
        MediaSourceKind::File => "file",
        MediaSourceKind::ArchiveEntry => "archive_entry",
        MediaSourceKind::NormalizedFile => "normalized_file",
    }
}

fn media_source_kind_from_db(value: &str) -> MediaSourceKind {
    match value {
        "file" => MediaSourceKind::File,
        "archive_entry" => MediaSourceKind::ArchiveEntry,
        "normalized_file" => MediaSourceKind::NormalizedFile,
        _ => MediaSourceKind::File,
    }
}

fn task_kind_to_db(kind: &TaskKind) -> &'static str {
    match kind {
        TaskKind::Scan => "scan",
        TaskKind::Ingest => "ingest",
        TaskKind::Normalize => "normalize",
        TaskKind::Thumbnail => "thumbnail",
        TaskKind::Ffmpeg => "ffmpeg",
        TaskKind::Subtitle => "subtitle",
    }
}

fn task_kind_from_db(value: &str) -> TaskKind {
    match value {
        "scan" => TaskKind::Scan,
        "ingest" => TaskKind::Ingest,
        "normalize" => TaskKind::Normalize,
        "thumbnail" => TaskKind::Thumbnail,
        "ffmpeg" => TaskKind::Ffmpeg,
        "subtitle" => TaskKind::Subtitle,
        _ => TaskKind::Scan,
    }
}

fn task_state_to_db(state: &TaskState) -> &'static str {
    match state {
        TaskState::Queued => "queued",
        TaskState::Running => "running",
        TaskState::Completed => "completed",
        TaskState::Failed => "failed",
        TaskState::Cancelled => "cancelled",
    }
}

fn task_state_from_db(value: &str) -> TaskState {
    match value {
        "queued" => TaskState::Queued,
        "running" => TaskState::Running,
        "completed" => TaskState::Completed,
        "failed" => TaskState::Failed,
        "cancelled" => TaskState::Cancelled,
        _ => TaskState::Failed,
    }
}

#[allow(dead_code)]
fn _error_code_to_db(code: &AppErrorCode) -> &'static str {
    match code {
        AppErrorCode::NotFound => "NOT_FOUND",
        AppErrorCode::AlreadyExists => "ALREADY_EXISTS",
        AppErrorCode::UnsupportedFormat => "UNSUPPORTED_FORMAT",
        AppErrorCode::PermissionDenied => "PERMISSION_DENIED",
        AppErrorCode::InvalidArgument => "INVALID_ARGUMENT",
        AppErrorCode::IoError => "IO_ERROR",
        AppErrorCode::DbError => "DB_ERROR",
        AppErrorCode::ExternalToolError => "EXTERNAL_TOOL_ERROR",
        AppErrorCode::Cancelled => "CANCELLED",
        AppErrorCode::Timeout => "TIMEOUT",
        AppErrorCode::InternalError => "INTERNAL_ERROR",
    }
}
