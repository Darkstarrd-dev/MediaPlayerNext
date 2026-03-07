mod entry_stream;
mod page_sort;
mod zip_reader;

use crate::{normalize_path, CandidateMediaKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use entry_stream::read_zip_entry_bytes;
pub use zip_reader::{list_zip_entries, ZipDirectoryEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePageEntry {
    pub entry_path: String,
    pub entry_name: String,
    pub page_index: usize,
    pub media_kind: CandidateMediaKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncompressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipArchiveIndex {
    pub normalized_zip_path: String,
    pub page_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_entry_path: Option<String>,
    pub pages: Vec<ArchivePageEntry>,
}

pub fn build_zip_index(path: &Path) -> Result<ZipArchiveIndex> {
    let entries = list_zip_entries(path)?;
    let pages = page_sort::sort_archive_pages(&entries)
        .into_iter()
        .enumerate()
        .map(|(index, entry)| ArchivePageEntry {
            entry_path: entry.entry_path,
            entry_name: entry.entry_name,
            page_index: index,
            media_kind: entry.kind,
            compressed_size: entry.compressed_size,
            uncompressed_size: entry.uncompressed_size,
            crc32: entry.crc32,
        })
        .collect::<Vec<_>>();

    Ok(ZipArchiveIndex {
        normalized_zip_path: normalize_path(path),
        page_count: pages.len(),
        cover_entry_path: pages.first().map(|item| item.entry_path.clone()),
        pages,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_zip_index, list_zip_entries, read_zip_entry_bytes};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn builds_zip_index_with_natural_page_order() {
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("chapter.cbz");
        write_test_zip(
            &zip_path,
            &[
                ("001-cover.png", b"cover"),
                ("010-page.png", b"page-10"),
                ("002-page.png", b"page-2"),
                ("notes.txt", b"ignore"),
            ],
        );

        let index = build_zip_index(&zip_path).expect("zip index should build");

        assert_eq!(index.page_count, 3);
        assert_eq!(index.cover_entry_path.as_deref(), Some("001-cover.png"));
        assert_eq!(index.pages[1].entry_path, "002-page.png");
        assert_eq!(index.pages[2].entry_path, "010-page.png");
    }

    #[test]
    fn lists_zip_entries_and_reads_entry_bytes() {
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("chapter.zip");
        write_test_zip(
            &zip_path,
            &[
                ("nested/001-page.png", b"png-data"),
                ("nested/readme.txt", b"readme"),
            ],
        );

        let entries = list_zip_entries(&zip_path).expect("zip entries should load");
        let bytes = read_zip_entry_bytes(&zip_path, "nested/001-page.png")
            .expect("zip entry bytes should load");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_path, "nested/001-page.png");
        assert_eq!(bytes, b"png-data");
    }

    #[test]
    fn returns_error_for_invalid_zip() {
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("broken.zip");
        std::fs::write(&zip_path, b"not-a-zip").expect("broken zip should be written");

        let error = build_zip_index(&zip_path).expect_err("invalid zip should fail");
        assert!(!error.to_string().is_empty());
    }

    fn write_test_zip(path: &std::path::Path, files: &[(&str, &[u8])]) {
        let file = File::create(path).expect("zip file should be created");
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (name, contents) in files {
            writer
                .start_file(name, options)
                .expect("zip entry should start");
            writer
                .write_all(contents)
                .expect("zip entry should be written");
        }

        writer.finish().expect("zip writer should finish");
    }
}
