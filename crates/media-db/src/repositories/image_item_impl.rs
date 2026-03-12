impl ImageItemRepository for SqliteRepositories<'_> {
    fn exists(&self, image_item_id: &ImageItemId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from image_items where id = :id limit 1",
                named_params! { ":id": image_item_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn replace_for_media_source(
        &self,
        media_source_id: &MediaSourceId,
        items: &[ImageItemRecord],
    ) -> Result<()> {
        self.connection.execute(
            "delete from image_items where media_source_id = :media_source_id",
            named_params! { ":media_source_id": media_source_id.0 },
        )?;

        for item in items {
            self.connection.execute(
                "
                insert into image_items (
                  id, media_source_id, asset_id, ordinal, width, height, size_bytes,
                  media_locator_json, hidden_flag, last_seen_revision, updated_at
                ) values (
                  :id, :media_source_id, :asset_id, :ordinal, :width, :height, :size_bytes,
                  :media_locator_json, :hidden_flag, :last_seen_revision, :updated_at
                )
                ",
                named_params! {
                    ":id": item.id.0,
                    ":media_source_id": item.media_source_id.0,
                    ":asset_id": item.asset_id.0,
                    ":ordinal": item.ordinal,
                    ":width": item.width,
                    ":height": item.height,
                    ":size_bytes": item.size_bytes,
                    ":media_locator_json": item.media_locator_json,
                    ":hidden_flag": item.hidden,
                    ":last_seen_revision": item.last_seen_revision,
                    ":updated_at": item.updated_at,
                },
            )?;
        }

        Ok(())
    }

    fn list_by_media_source(
        &self,
        media_source_id: &MediaSourceId,
    ) -> Result<Vec<ImageItemRecord>> {
        let mut statement = self.connection.prepare(
            "
            select id, media_source_id, asset_id, ordinal, width, height, size_bytes,
                   media_locator_json, hidden_flag, last_seen_revision, updated_at
            from image_items
            where media_source_id = :media_source_id
            order by ordinal asc, id asc
            ",
        )?;

        let rows = statement.query_map(
            named_params! { ":media_source_id": media_source_id.0 },
            |row| {
                Ok(ImageItemRecord {
                    id: ImageItemId(row.get::<_, String>(0)?),
                    media_source_id: MediaSourceId(row.get::<_, String>(1)?),
                    asset_id: AssetId(row.get::<_, String>(2)?),
                    ordinal: row.get(3)?,
                    width: row.get(4)?,
                    height: row.get(5)?,
                    size_bytes: row.get(6)?,
                    media_locator_json: row.get(7)?,
                    hidden: row.get::<_, bool>(8)?,
                    last_seen_revision: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )?;

        let mut image_items = Vec::new();
        for row in rows {
            image_items.push(row?);
        }

        Ok(image_items)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> Result<Vec<ImageItemRecord>> {
        let mut statement = self.connection.prepare(
            "
            select i.id, i.media_source_id, i.asset_id, i.ordinal, i.width, i.height, i.size_bytes,
                   i.media_locator_json, i.hidden_flag, i.last_seen_revision, i.updated_at
            from image_items i
            inner join media_sources s on s.id = i.media_source_id
            where s.library_id = :library_id
            order by s.absolute_path asc, i.ordinal asc, i.id asc
            ",
        )?;

        let rows = statement.query_map(named_params! { ":library_id": library_id.0 }, |row| {
            Ok(ImageItemRecord {
                id: ImageItemId(row.get::<_, String>(0)?),
                media_source_id: MediaSourceId(row.get::<_, String>(1)?),
                asset_id: AssetId(row.get::<_, String>(2)?),
                ordinal: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                size_bytes: row.get(6)?,
                media_locator_json: row.get(7)?,
                hidden: row.get::<_, bool>(8)?,
                last_seen_revision: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut image_items = Vec::new();
        for row in rows {
            image_items.push(row?);
        }

        Ok(image_items)
    }

    fn delete_by_library(&self, library_id: &LibraryId) -> Result<()> {
        self.connection.execute(
            "delete from image_items where media_source_id in (select id from media_sources where library_id = :library_id)",
            named_params! { ":library_id": library_id.0 },
        )?;

        Ok(())
    }
}
