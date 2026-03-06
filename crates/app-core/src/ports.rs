use shared_model::{ArchiveId, AssetId, LibraryId, SourceId, TaskId, ThumbnailKey};

pub trait LibraryRepository {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool>;
}

pub trait SourceRepository {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool>;
}

pub trait ArchiveRepository {
    fn exists(&self, archive_id: &ArchiveId) -> anyhow::Result<bool>;
}

pub trait AssetRepository {
    fn exists(&self, asset_id: &AssetId) -> anyhow::Result<bool>;
}

pub trait TaskRepository {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool>;
}

pub trait ThumbnailRepository {
    fn exists(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<bool>;
}
