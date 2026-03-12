impl MediaSourceRepository for SqliteRepositories<'_> {
    fn exists(&self, media_source_id: &MediaSourceId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from media_sources where id = :id limit 1",
                named_params! { ":id": media_source_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, media_source: &MediaSourceRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into media_sources (
              id, library_id, source_type, backing_source_id, absolute_path, tree_path_json,
              display_name, item_count, cover_asset_id, last_seen_revision, exists_flag, updated_at
            ) values (
              :id, :library_id, :source_type, :backing_source_id, :absolute_path, :tree_path_json,
              :display_name, :item_count, :cover_asset_id, :last_seen_revision, :exists_flag, :updated_at
            )
            on conflict(id) do update set
              library_id = excluded.library_id,
              source_type = excluded.source_type,
              backing_source_id = excluded.backing_source_id,
              absolute_path = excluded.absolute_path,
              tree_path_json = excluded.tree_path_json,
              display_name = excluded.display_name,
              item_count = excluded.item_count,
              cover_asset_id = excluded.cover_asset_id,
              last_seen_revision = excluded.last_seen_revision,
              exists_flag = excluded.exists_flag,
              updated_at = excluded.updated_at
            ",
            named_params! {
                ":id": media_source.id.0,
                ":library_id": media_source.library_id.0,
                ":source_type": media_source_type_to_db(&media_source.source_type),
                ":backing_source_id": media_source.backing_source_id.as_ref().map(|value| value.0.as_str()),
                ":absolute_path": media_source.absolute_path,
                ":tree_path_json": media_source.tree_path_json,
                ":display_name": media_source.display_name,
                ":item_count": media_source.item_count,
                ":cover_asset_id": media_source.cover_asset_id.as_ref().map(|value| value.0.as_str()),
                ":last_seen_revision": media_source.last_seen_revision,
                ":exists_flag": media_source.exists,
                ":updated_at": media_source.updated_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, media_source_id: &MediaSourceId) -> Result<Option<MediaSourceRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, library_id, source_type, backing_source_id, absolute_path, tree_path_json,
                       display_name, item_count, cover_asset_id, last_seen_revision, exists_flag, updated_at
                from media_sources
                where id = :id
                limit 1
                ",
                named_params! { ":id": media_source_id.0 },
                |row| {
                    Ok(MediaSourceRecord {
                        id: MediaSourceId(row.get::<_, String>(0)?),
                        library_id: LibraryId(row.get::<_, String>(1)?),
                        source_type: media_source_type_from_db(&row.get::<_, String>(2)?),
                        backing_source_id: row.get::<_, Option<String>>(3)?.map(SourceId),
                        absolute_path: row.get(4)?,
                        tree_path_json: row.get(5)?,
                        display_name: row.get(6)?,
                        item_count: row.get(7)?,
                        cover_asset_id: row.get::<_, Option<String>>(8)?.map(AssetId),
                        last_seen_revision: row.get(9)?,
                        exists: row.get::<_, bool>(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> Result<Vec<MediaSourceRecord>> {
        let mut statement = self.connection.prepare(
            "
            select id, library_id, source_type, backing_source_id, absolute_path, tree_path_json,
                   display_name, item_count, cover_asset_id, last_seen_revision, exists_flag, updated_at
            from media_sources
            where library_id = :library_id
            order by absolute_path asc
            ",
        )?;

        let rows = statement.query_map(named_params! { ":library_id": library_id.0 }, |row| {
            Ok(MediaSourceRecord {
                id: MediaSourceId(row.get::<_, String>(0)?),
                library_id: LibraryId(row.get::<_, String>(1)?),
                source_type: media_source_type_from_db(&row.get::<_, String>(2)?),
                backing_source_id: row.get::<_, Option<String>>(3)?.map(SourceId),
                absolute_path: row.get(4)?,
                tree_path_json: row.get(5)?,
                display_name: row.get(6)?,
                item_count: row.get(7)?,
                cover_asset_id: row.get::<_, Option<String>>(8)?.map(AssetId),
                last_seen_revision: row.get(9)?,
                exists: row.get::<_, bool>(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let mut media_sources = Vec::new();
        for row in rows {
            media_sources.push(row?);
        }

        Ok(media_sources)
    }

    fn delete_by_library(&self, library_id: &LibraryId) -> Result<()> {
        self.connection.execute(
            "delete from media_sources where library_id = :library_id",
            named_params! { ":library_id": library_id.0 },
        )?;

        Ok(())
    }

    fn get_by_backing_source(&self, source_id: &SourceId) -> Result<Option<MediaSourceRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, library_id, source_type, backing_source_id, absolute_path, tree_path_json,
                       display_name, item_count, cover_asset_id, last_seen_revision, exists_flag, updated_at
                from media_sources
                where backing_source_id = :backing_source_id
                order by updated_at desc
                limit 1
                ",
                named_params! { ":backing_source_id": source_id.0 },
                |row| {
                    Ok(MediaSourceRecord {
                        id: MediaSourceId(row.get::<_, String>(0)?),
                        library_id: LibraryId(row.get::<_, String>(1)?),
                        source_type: media_source_type_from_db(&row.get::<_, String>(2)?),
                        backing_source_id: row.get::<_, Option<String>>(3)?.map(SourceId),
                        absolute_path: row.get(4)?,
                        tree_path_json: row.get(5)?,
                        display_name: row.get(6)?,
                        item_count: row.get(7)?,
                        cover_asset_id: row.get::<_, Option<String>>(8)?.map(AssetId),
                        last_seen_revision: row.get(9)?,
                        exists: row.get::<_, bool>(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}
