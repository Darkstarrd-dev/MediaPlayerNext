use crate::ports::{LibraryRepository, SourceRepository, TaskRepository};
use shared_model::{LibraryId, LibraryRecord, SourceId, SourceRecord, TaskId, TaskRecord};
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
    pub(super) libraries: HashMap<String, LibraryRecord>,
    pub(super) sources: HashMap<String, SourceRecord>,
    pub(super) tasks: HashMap<String, TaskRecord>,
}

impl LibraryRepository for MemoryRepos {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
        Ok(self.libraries.contains_key(&library_id.0))
    }

    fn upsert(&self, _library: &LibraryRecord) -> anyhow::Result<()> {
        unreachable!()
    }

    fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
        Ok(self.libraries.get(&library_id.0).cloned())
    }
}

impl SourceRepository for MemoryRepos {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
        Ok(self.sources.contains_key(&source_id.0))
    }

    fn upsert(&self, _source: &SourceRecord) -> anyhow::Result<()> {
        unreachable!()
    }

    fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
        Ok(self.sources.get(&source_id.0).cloned())
    }

    fn count(&self) -> anyhow::Result<u64> {
        Ok(self.sources.len() as u64)
    }

    fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
        Ok(self
            .sources
            .values()
            .filter(|item| item.library_id == *library_id)
            .count() as u64)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
        let mut items: Vec<_> = self
            .sources
            .values()
            .filter(|item| item.library_id == *library_id)
            .cloned()
            .collect();
        items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
        Ok(items)
    }
}

impl TaskRepository for MemoryRepos {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
        Ok(self.tasks.contains_key(&task_id.0))
    }

    fn upsert(&self, _task: &TaskRecord) -> anyhow::Result<()> {
        unreachable!()
    }

    fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
        Ok(self.tasks.get(&task_id.0).cloned())
    }
}

pub(super) struct MutableMemoryRepos {
    inner: Mutex<MemoryRepos>,
}

impl Default for MutableMemoryRepos {
    fn default() -> Self {
        Self {
            inner: Mutex::new(MemoryRepos::default()),
        }
    }
}

impl LibraryRepository for MutableMemoryRepos {
    fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.inner)
            .libraries
            .contains_key(&library_id.0))
    }

    fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.inner)
            .libraries
            .insert(library.id.0.clone(), library.clone());
        Ok(())
    }

    fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
        Ok(lock_or_poison(&self.inner)
            .libraries
            .get(&library_id.0)
            .cloned())
    }
}

impl SourceRepository for MutableMemoryRepos {
    fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.inner)
            .sources
            .contains_key(&source_id.0))
    }

    fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.inner)
            .sources
            .insert(source.id.0.clone(), source.clone());
        Ok(())
    }

    fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
        Ok(lock_or_poison(&self.inner)
            .sources
            .get(&source_id.0)
            .cloned())
    }

    fn count(&self) -> anyhow::Result<u64> {
        Ok(lock_or_poison(&self.inner).sources.len() as u64)
    }

    fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
        Ok(lock_or_poison(&self.inner)
            .sources
            .values()
            .filter(|item| item.library_id == *library_id)
            .count() as u64)
    }

    fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
        let mut items: Vec<_> = lock_or_poison(&self.inner)
            .sources
            .values()
            .filter(|item| item.library_id == *library_id)
            .cloned()
            .collect();
        items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
        Ok(items)
    }
}

impl TaskRepository for MutableMemoryRepos {
    fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
        Ok(lock_or_poison(&self.inner).tasks.contains_key(&task_id.0))
    }

    fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()> {
        lock_or_poison(&self.inner)
            .tasks
            .insert(task.id.0.clone(), task.clone());
        Ok(())
    }

    fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
        Ok(lock_or_poison(&self.inner).tasks.get(&task_id.0).cloned())
    }
}
