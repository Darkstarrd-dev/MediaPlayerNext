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
