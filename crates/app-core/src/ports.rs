use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, ImageItemId,
    ImageItemRecord, LibraryId, LibraryRecord, MediaAssetRecord, MediaSourceId, MediaSourceRecord,
    SourceId, SourceRecord, SubtitleHostSummary, SubtitleProgressEvent, SubtitleSessionId,
    SubtitleSessionSummary, TaskId, TaskRecord, ThumbnailKey, ThumbnailRecord,
};

pub trait LibraryRepository {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool>;
    fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()>;
    fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>>;
    fn list(&self) -> anyhow::Result<Vec<LibraryRecord>> {
        Err(anyhow::anyhow!("library list is not implemented"))
    }
    fn delete(&self, _library_id: &LibraryId) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("library delete is not implemented"))
    }
}

pub trait SourceRepository {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool>;
    fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()>;
    fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>>;
    fn count(&self) -> anyhow::Result<u64>;
    fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64>;
    fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>>;
}

pub trait MediaSourceRepository {
    fn exists(&self, media_source_id: &MediaSourceId) -> anyhow::Result<bool>;
    fn upsert(&self, media_source: &MediaSourceRecord) -> anyhow::Result<()>;
    fn get(&self, media_source_id: &MediaSourceId) -> anyhow::Result<Option<MediaSourceRecord>>;
    fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<MediaSourceRecord>>;
    fn delete_by_library(&self, _library_id: &LibraryId) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "media source delete by library is not implemented"
        ))
    }
    fn get_by_backing_source(
        &self,
        _source_id: &SourceId,
    ) -> anyhow::Result<Option<MediaSourceRecord>> {
        Ok(None)
    }
}

pub trait ImageItemRepository {
    fn exists(&self, image_item_id: &ImageItemId) -> anyhow::Result<bool>;
    fn replace_for_media_source(
        &self,
        media_source_id: &MediaSourceId,
        items: &[ImageItemRecord],
    ) -> anyhow::Result<()>;
    fn list_by_media_source(
        &self,
        media_source_id: &MediaSourceId,
    ) -> anyhow::Result<Vec<ImageItemRecord>>;
    fn list_by_library(&self, _library_id: &LibraryId) -> anyhow::Result<Vec<ImageItemRecord>> {
        Err(anyhow::anyhow!(
            "image item list by library is not implemented"
        ))
    }
    fn delete_by_library(&self, _library_id: &LibraryId) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "image item delete by library is not implemented"
        ))
    }
}

pub trait ArchiveRepository {
    fn exists(&self, archive_id: &ArchiveId) -> anyhow::Result<bool>;
    fn upsert(&self, archive: &ArchiveRecord) -> anyhow::Result<()>;
    fn get(&self, archive_id: &ArchiveId) -> anyhow::Result<Option<ArchiveRecord>>;
    fn get_by_source(&self, source_id: &SourceId) -> anyhow::Result<Option<ArchiveRecord>>;
}

pub trait ArchiveEntryRepository {
    fn replace_for_archive(
        &self,
        archive_id: &ArchiveId,
        entries: &[ArchiveEntryRecord],
    ) -> anyhow::Result<()>;
    fn get(&self, archive_entry_id: &ArchiveEntryId) -> anyhow::Result<Option<ArchiveEntryRecord>>;
    fn list_by_archive(&self, archive_id: &ArchiveId) -> anyhow::Result<Vec<ArchiveEntryRecord>>;
}

pub trait AssetRepository {
    fn exists(&self, asset_id: &AssetId) -> anyhow::Result<bool>;
    fn upsert(&self, asset: &MediaAssetRecord) -> anyhow::Result<()>;
    fn get(&self, asset_id: &AssetId) -> anyhow::Result<Option<MediaAssetRecord>>;
}

pub trait TaskRepository {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool>;
    fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()>;
    fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>>;
}

pub trait ThumbnailRepository {
    fn exists(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<bool>;
    fn upsert(&self, thumbnail: &ThumbnailRecord) -> anyhow::Result<()>;
    fn get(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<Option<ThumbnailRecord>>;
    fn get_ready_by_asset_profile(
        &self,
        _asset_id: &AssetId,
        _profile: &str,
    ) -> anyhow::Result<Option<ThumbnailRecord>> {
        Ok(None)
    }
}

pub trait SubtitleHostPort {
    fn ping(&self) -> anyhow::Result<SubtitleHostSummary>;
    fn health(&self) -> anyhow::Result<SubtitleHostSummary>;
    fn start_session(&self, asset_id: Option<&AssetId>) -> anyhow::Result<SubtitleSessionSummary>;
    fn stop_session(
        &self,
        session_id: &SubtitleSessionId,
    ) -> anyhow::Result<SubtitleSessionSummary>;
    fn get_progress(&self, session_id: &SubtitleSessionId)
        -> anyhow::Result<SubtitleProgressEvent>;
}
