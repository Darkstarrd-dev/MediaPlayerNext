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
            "delete from image_items where media_source_id in (select id from media_sources where library_id = :library_id)",
            named_params! { ":library_id": library_id.0 },
        )?;
        self.connection.execute(
            "delete from media_sources where library_id = :library_id",
            named_params! { ":library_id": library_id.0 },
        )?;
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
