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
