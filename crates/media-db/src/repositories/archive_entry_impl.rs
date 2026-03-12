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
