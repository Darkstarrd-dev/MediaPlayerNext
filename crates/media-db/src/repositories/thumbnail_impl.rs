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

    fn get_ready_by_asset_profile(
        &self,
        asset_id: &AssetId,
        profile: &str,
    ) -> Result<Option<ThumbnailRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select thumbnail_key, asset_id, profile, width, height, format,
                       disk_path, byte_size, state, updated_at
                from thumbnails
                where asset_id = :asset_id
                  and profile = :profile
                  and state = 'ready'
                order by updated_at desc
                limit 1
                ",
                named_params! {
                    ":asset_id": asset_id.0,
                    ":profile": profile,
                },
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
