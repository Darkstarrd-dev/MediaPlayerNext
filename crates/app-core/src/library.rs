use crate::ports::LibraryRepository;
use anyhow::{anyhow, Result};
use shared_model::{LibraryId, LibraryRecord};

pub fn list_libraries<L: LibraryRepository>(library_repository: &L) -> Result<Vec<LibraryRecord>> {
    library_repository.list()
}

pub fn get_library<L: LibraryRepository>(
    library_repository: &L,
    library_id: &LibraryId,
) -> Result<LibraryRecord> {
    library_repository
        .get(library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", library_id.0))
}

pub fn remove_library<L: LibraryRepository>(
    library_repository: &L,
    library_id: &LibraryId,
) -> Result<()> {
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    library_repository.delete(library_id)
}
