use shared_model::{
    ArchiveId, ArchiveRecord, AssetId, LibraryId, LibraryRecord, MediaAssetRecord, SourceId,
    SourceRecord, TaskId, TaskRecord, ThumbnailKey, ThumbnailRecord,
};

pub trait LibraryRepository {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool>;
    fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()>;
    fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>>;
}

pub trait SourceRepository {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool>;
    fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()>;
    fn count(&self) -> anyhow::Result<u64>;
    fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64>;
}

pub trait ArchiveRepository {
    fn exists(&self, archive_id: &ArchiveId) -> anyhow::Result<bool>;
    fn upsert(&self, archive: &ArchiveRecord) -> anyhow::Result<()>;
}

pub trait AssetRepository {
    fn exists(&self, asset_id: &AssetId) -> anyhow::Result<bool>;
    fn upsert(&self, asset: &MediaAssetRecord) -> anyhow::Result<()>;
}

pub trait TaskRepository {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool>;
    fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()>;
}

pub trait ThumbnailRepository {
    fn exists(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<bool>;
    fn upsert(&self, thumbnail: &ThumbnailRecord) -> anyhow::Result<()>;
}
