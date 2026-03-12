use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
    SourceRepository, ThumbnailRepository,
};
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
    LibraryRecord, MediaAssetRecord, SourceId, SourceRecord, ThumbnailKey, ThumbnailRecord,
};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

fn lock_or_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(error) => error.into_inner(),
    }
}

#[derive(Default)]
pub(super) struct MemoryRepos {
    libraries: Mutex<HashMap<String, LibraryRecord>>,
    sources: Mutex<HashMap<String, SourceRecord>>,
    archives: Mutex<HashMap<String, ArchiveRecord>>,
    archive_entries: Mutex<HashMap<String, Vec<ArchiveEntryRecord>>>,
    assets: Mutex<HashMap<String, MediaAssetRecord>>,
    thumbnails: Mutex<HashMap<String, ThumbnailRecord>>,
}

impl LibraryRepository for MemoryRepos {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.libraries).contains_key(&library_id.0))
    }

    fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.libraries).insert(library.id.0.clone(), library.clone());
        Ok(())
    }

    fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
        Ok(lock_or_poison(&self.libraries).get(&library_id.0).cloned())
    }
}

impl SourceRepository for MemoryRepos {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.sources).contains_key(&source_id.0))
    }

    fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.sources).insert(source.id.0.clone(), source.clone());
        Ok(())
    }

    fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
        Ok(lock_or_poison(&self.sources).get(&source_id.0).cloned())
    }

    fn count(&self) -> anyhow::Result<u64> {
        Ok(lock_or_poison(&self.sources).len() as u64)
    }

    fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
        Ok(lock_or_poison(&self.sources)
            .values()
            .filter(|item| item.library_id == *library_id)
            .count() as u64)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
        let mut items = lock_or_poison(&self.sources)
            .values()
            .filter(|item| item.library_id == *library_id)
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
        Ok(items)
    }
}

impl ArchiveRepository for MemoryRepos {
    fn exists(&self, archive_id: &ArchiveId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.archives).contains_key(&archive_id.0))
    }

    fn upsert(&self, archive: &ArchiveRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.archives).insert(archive.id.0.clone(), archive.clone());
        Ok(())
    }

    fn get(&self, archive_id: &ArchiveId) -> anyhow::Result<Option<ArchiveRecord>> {
        Ok(lock_or_poison(&self.archives).get(&archive_id.0).cloned())
    }

    fn get_by_source(&self, source_id: &SourceId) -> anyhow::Result<Option<ArchiveRecord>> {
        Ok(lock_or_poison(&self.archives)
            .values()
            .find(|item| item.source_id == *source_id)
            .cloned())
    }
}

impl ArchiveEntryRepository for MemoryRepos {
    fn replace_for_archive(
        &self,
        archive_id: &ArchiveId,
        entries: &[ArchiveEntryRecord],
    ) -> anyhow::Result<()> {
        lock_or_poison(&self.archive_entries).insert(archive_id.0.clone(), entries.to_vec());
        Ok(())
    }

    fn get(&self, archive_entry_id: &ArchiveEntryId) -> anyhow::Result<Option<ArchiveEntryRecord>> {
        Ok(lock_or_poison(&self.archive_entries)
            .values()
            .flat_map(|items| items.iter())
            .find(|item| item.id == *archive_entry_id)
            .cloned())
    }

    fn list_by_archive(&self, archive_id: &ArchiveId) -> anyhow::Result<Vec<ArchiveEntryRecord>> {
        Ok(lock_or_poison(&self.archive_entries)
            .get(&archive_id.0)
            .cloned()
            .unwrap_or_default())
    }
}

impl AssetRepository for MemoryRepos {
    fn exists(&self, asset_id: &AssetId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.assets).contains_key(&asset_id.0))
    }

    fn upsert(&self, asset: &MediaAssetRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.assets).insert(asset.id.0.clone(), asset.clone());
        Ok(())
    }

    fn get(&self, asset_id: &AssetId) -> anyhow::Result<Option<MediaAssetRecord>> {
        Ok(lock_or_poison(&self.assets).get(&asset_id.0).cloned())
    }
}

impl ThumbnailRepository for MemoryRepos {
    fn exists(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.thumbnails).contains_key(&thumbnail_key.0))
    }

    fn upsert(&self, thumbnail: &ThumbnailRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.thumbnails)
            .insert(thumbnail.thumbnail_key.0.clone(), thumbnail.clone());
        Ok(())
    }

    fn get(&self, thumbnail_key: &ThumbnailKey) -> anyhow::Result<Option<ThumbnailRecord>> {
        Ok(lock_or_poison(&self.thumbnails)
            .get(&thumbnail_key.0)
            .cloned())
    }
}
