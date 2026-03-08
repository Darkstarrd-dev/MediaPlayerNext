use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
    LibraryRecord, MediaAssetRecord, SourceId, SourceRecord, SubtitleHostSummary,
    SubtitleProgressEvent, SubtitleSessionId, SubtitleSessionSummary, TaskId, TaskRecord,
    ThumbnailKey, ThumbnailRecord,
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
