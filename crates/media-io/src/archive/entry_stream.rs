use super::zip_reader::normalize_archive_entry_path;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub fn read_zip_entry_bytes(path: &Path, entry_path: &str) -> Result<Vec<u8>> {
    let file =
        File::open(path).with_context(|| format!("failed to open zip: {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("failed to parse zip: {}", path.display()))?;
    let normalized_entry_path = normalize_archive_entry_path(entry_path);
    let mut entry = archive
        .by_name(&normalized_entry_path)
        .with_context(|| format!("failed to read zip entry: {normalized_entry_path}"))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read zip entry bytes: {normalized_entry_path}"))?;
    Ok(bytes)
}
