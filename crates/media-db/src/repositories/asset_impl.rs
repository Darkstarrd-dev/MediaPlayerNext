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
