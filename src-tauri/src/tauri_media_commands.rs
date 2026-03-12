use app_core::archive::{
    archive_snapshot, normalize_archive_source, normalize_archive_status,
    resolve_archive_entry_location, ArchiveNormalizeSummary,
};
use app_core::asset::{asset_snapshot_for_library, resolve_asset, AssetResolution};
use app_core::content::asset_snapshot_for_media_source;
use app_core::thumbnail::{ensure_thumbnail_for_asset, parse_thumbnail_profile};
use serde::Serialize;
use shared_model::{
    AppError, ArchiveEntryId, AssetId, LibraryId, MediaSourceId, SourceId, TaskProgress,
};

type CommandResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListEntryPayload {
    asset_id: String,
    source_kind: String,
    source_ref_id: String,
    library_id: String,
    media_source_id: Option<String>,
    source_id: String,
    archive_id: Option<String>,
    entry_path: Option<String>,
    mime: String,
    thumbnail_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDetailPayload {
    asset_id: String,
    source_kind: String,
    mime: String,
    library_id: String,
    source_id: String,
    file_path: Option<String>,
    archive_id: Option<String>,
    archive_entry_id: Option<String>,
    archive_path: Option<String>,
    entry_path: Option<String>,
}

#[tauri::command]
pub fn items_list_command(
    app: tauri::AppHandle,
    library_id: String,
    media_source_id: Option<String>,
    source_id: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> CommandResult<Vec<ItemListEntryPayload>> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = LibraryId(library_id);
        let legacy_source_id = source_id.clone();
        let resolved_media_source_id = match media_source_id {
            Some(value) => Some(MediaSourceId(value)),
            None => match source_id {
                Some(value) => app_core::ports::MediaSourceRepository::get_by_backing_source(
                    &repositories,
                    &SourceId(value.clone()),
                )?
                .map(|record| record.id),
                None => None,
            },
        };

        let selected_media_source_id = resolved_media_source_id
            .as_ref()
            .map(|value| value.0.clone());

        let filtered_snapshot = match resolved_media_source_id {
            Some(media_source_id) => asset_snapshot_for_media_source(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &media_source_id,
            )?,
            None => {
                let snapshot = asset_snapshot_for_library(
                    &repositories,
                    &repositories,
                    &repositories,
                    &repositories,
                    &library_id,
                )?;

                match legacy_source_id {
                    Some(expected_source_id) => snapshot
                        .into_iter()
                        .filter(|item| item.source_id == expected_source_id)
                        .collect::<Vec<_>>(),
                    None => snapshot,
                }
            }
        };

        let offset = page
            .zip(page_size)
            .map(|(page_value, page_size_value)| {
                page_value.saturating_sub(1) as usize * page_size_value as usize
            })
            .unwrap_or(0);
        let limit = page_size
            .map(|value| value as usize)
            .unwrap_or(filtered_snapshot.len());

        filtered_snapshot
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|item| {
                let thumbnail_key =
                    app_core::ports::ThumbnailRepository::get_ready_by_asset_profile(
                        &repositories,
                        &AssetId(item.asset_id.clone()),
                        crate::SIDEBAR_THUMBNAIL_PROFILE,
                    )?
                    .map(|record| record.thumbnail_key.0);

                Ok(ItemListEntryPayload {
                    thumbnail_key,
                    asset_id: item.asset_id,
                    source_kind: item.source_kind,
                    source_ref_id: item.source_ref_id,
                    library_id: item.library_id,
                    media_source_id: selected_media_source_id.clone(),
                    source_id: item.source_id,
                    archive_id: item.archive_id,
                    entry_path: item.entry_path,
                    mime: item.mime,
                })
            })
            .collect()
    })
    .map_err(|error| crate::map_backend_command_error("items_list_command", "items", error))
}

#[tauri::command]
pub fn item_detail_command(
    app: tauri::AppHandle,
    asset_id: String,
) -> CommandResult<ItemDetailPayload> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let asset_id = AssetId(asset_id);
        let asset = app_core::ports::AssetRepository::get(&repositories, &asset_id)?
            .ok_or_else(|| anyhow::anyhow!("asset not found: {}", asset_id.0))?;
        let resolution = resolve_asset(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &asset_id,
        )?;

        Ok(match resolution {
            AssetResolution::File(item) => ItemDetailPayload {
                asset_id: item.asset_id,
                source_kind: "file".to_string(),
                mime: asset.mime,
                library_id: item.library_id,
                source_id: item.source_id,
                file_path: Some(item.file_path),
                archive_id: None,
                archive_entry_id: None,
                archive_path: None,
                entry_path: None,
            },
            AssetResolution::ArchiveEntry(item) => ItemDetailPayload {
                asset_id: item.asset_id,
                source_kind: "archive_entry".to_string(),
                mime: asset.mime,
                library_id: item.library_id,
                source_id: item.source_id,
                file_path: None,
                archive_id: Some(item.archive_id),
                archive_entry_id: Some(item.archive_entry_id),
                archive_path: Some(item.archive_path),
                entry_path: Some(item.entry_path),
            },
        })
    })
    .map_err(|error| crate::map_backend_command_error("item_detail_command", "items", error))
}

#[tauri::command]
pub fn archive_entries_command(
    app: tauri::AppHandle,
    source_id: Option<String>,
    media_source_id: Option<String>,
) -> CommandResult<Vec<shared_model::ArchiveEntryRecord>> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let source_id = resolve_archive_source_id(&repositories, source_id, media_source_id)?;
        let snapshot = archive_snapshot(&repositories, &repositories, &source_id)?;
        Ok(snapshot.entries)
    })
    .map_err(|error| crate::map_backend_command_error("archive_entries_command", "archive", error))
}

#[tauri::command]
pub fn archive_entry_detail_command(
    app: tauri::AppHandle,
    archive_entry_id: String,
) -> CommandResult<app_core::archive::ResolvedArchiveEntryLocation> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        resolve_archive_entry_location(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &ArchiveEntryId(archive_entry_id),
        )
    })
    .map_err(|error| {
        crate::map_backend_command_error("archive_entry_detail_command", "archive", error)
    })
}

#[tauri::command]
pub fn archive_normalize_command(
    app: tauri::AppHandle,
    source_id: Option<String>,
    media_source_id: Option<String>,
) -> CommandResult<ArchiveNormalizeSummary> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let source_id = resolve_archive_source_id(&repositories, source_id, media_source_id)?;
        normalize_archive_source(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &environment.runtime_paths.sevenz_path,
            &environment.normalize_root,
            &source_id,
        )
    })
    .map_err(|error| {
        crate::map_backend_command_error("archive_normalize_command", "archive", error)
    })
}

#[tauri::command]
pub fn archive_normalize_status_command(
    app: tauri::AppHandle,
    task_id: String,
) -> CommandResult<TaskProgress> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let task = normalize_archive_status(&repositories, &shared_model::TaskId(task_id))?;
        Ok(TaskProgress {
            task_id: task.id,
            task_type: task.task_type,
            state: task.state,
            current: task.current,
            total: task.total,
            message: task.message,
            error_code: task.error_code,
        })
    })
    .map_err(|error| {
        crate::map_backend_command_error("archive_normalize_status_command", "archive", error)
    })
}

#[tauri::command]
pub fn thumbnail_ensure_command(
    app: tauri::AppHandle,
    asset_id: String,
    profile: String,
) -> CommandResult<app_core::thumbnail::ThumbnailEnsureSummary> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        ensure_thumbnail_for_asset(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &environment.thumbnail_cache_root,
            &AssetId(asset_id),
            parse_thumbnail_profile(&profile)?,
        )
    })
    .map_err(|error| {
        crate::map_backend_command_error("thumbnail_ensure_command", "thumbnail", error)
    })
}

fn resolve_archive_source_id(
    repositories: &impl app_core::ports::MediaSourceRepository,
    source_id: Option<String>,
    media_source_id: Option<String>,
) -> anyhow::Result<SourceId> {
    if let Some(source_id) = source_id {
        return Ok(SourceId(source_id));
    }

    let Some(media_source_id) = media_source_id else {
        return Err(anyhow::anyhow!("source_id or media_source_id is required"));
    };

    let media_source = app_core::ports::MediaSourceRepository::get(
        repositories,
        &MediaSourceId(media_source_id.clone()),
    )?
    .ok_or_else(|| anyhow::anyhow!("media source not found: {media_source_id}"))?;

    media_source
        .backing_source_id
        .ok_or_else(|| anyhow::anyhow!("media source has no backing source: {media_source_id}"))
}
