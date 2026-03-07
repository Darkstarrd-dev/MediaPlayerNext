use crate::{classify_extension, CandidateMediaKind};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipDirectoryEntry {
    pub entry_path: String,
    pub entry_name: String,
    pub kind: CandidateMediaKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncompressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32: Option<i64>,
}

pub fn list_zip_entries(path: &Path) -> Result<Vec<ZipDirectoryEntry>> {
    let file =
        File::open(path).with_context(|| format!("failed to open zip: {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("failed to parse zip: {}", path.display()))?;
    let mut entries = Vec::new();

    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .with_context(|| format!("failed to read zip entry #{index}: {}", path.display()))?;

        if entry.is_dir() {
            continue;
        }

        let entry_path = normalize_archive_entry_path(entry.name());
        let entry_name = entry_path
            .rsplit('/')
            .next()
            .map(str::to_string)
            .unwrap_or_else(|| entry_path.clone());
        let extension = entry_name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default();

        entries.push(ZipDirectoryEntry {
            entry_path,
            entry_name,
            kind: classify_extension(&extension),
            compressed_size: Some(entry.compressed_size() as i64),
            uncompressed_size: Some(entry.size() as i64),
            crc32: Some(i64::from(entry.crc32())),
        });
    }

    entries.sort_by(|left, right| left.entry_path.cmp(&right.entry_path));
    Ok(entries)
}

pub(crate) fn normalize_archive_entry_path(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}
