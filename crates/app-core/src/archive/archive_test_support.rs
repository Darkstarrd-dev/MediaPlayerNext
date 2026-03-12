use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository, TaskRepository,
};
use anyhow::anyhow;
use media_io::normalize::ArchiveExtractor;
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LibraryRecord,
    SourceId, SourceRecord, TaskId, TaskRecord,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

pub(super) fn lock_or_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(error) => error.into_inner(),
    }
}

#[derive(Default)]
pub(super) struct MemoryRepos {
    pub(super) libraries: Mutex<HashMap<String, LibraryRecord>>,
    pub(super) sources: Mutex<HashMap<String, SourceRecord>>,
    pub(super) archives: Mutex<HashMap<String, ArchiveRecord>>,
    pub(super) archive_entries: Mutex<HashMap<String, Vec<ArchiveEntryRecord>>>,
    pub(super) tasks: Mutex<HashMap<String, TaskRecord>>,
}

pub(super) struct MockExtractor {
    pub(super) fail_first: Mutex<bool>,
    pub(super) files: Vec<(String, Vec<u8>)>,
}

impl ArchiveExtractor for MockExtractor {
    fn extract_archive(&self, _archive_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
        let mut fail_first = lock_or_poison(&self.fail_first);
        if *fail_first {
            *fail_first = false;
            return Err(anyhow!("mock normalize failure"));
        }

        for (relative_path, bytes) in &self.files {
            let target_path = output_dir.join(relative_path);
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(target_path, bytes)?;
        }

        Ok(())
    }
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

impl TaskRepository for MemoryRepos {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.tasks).contains_key(&task_id.0))
    }

    fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.tasks).insert(task.id.0.clone(), task.clone());
        Ok(())
    }

    fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
        Ok(lock_or_poison(&self.tasks).get(&task_id.0).cloned())
    }
}
