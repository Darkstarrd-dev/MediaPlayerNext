use app_core::archive::{
    archive_snapshot, normalize_archive_source, normalize_archive_status,
    resolve_archive_entry_location, ArchiveNormalizeSummary,
};
use app_core::asset::{resolve_asset, AssetResolution};
use anyhow::anyhow;
use rusqlite::{named_params, Connection};
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
pub struct ItemsListPayload {
    items: Vec<ItemListEntryPayload>,
    has_next_page: bool,
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
pub async fn items_list_command(
    app: tauri::AppHandle,
    library_id: String,
    media_source_id: Option<String>,
    source_id: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> CommandResult<ItemsListPayload> {
    let join_result = tauri::async_runtime::spawn_blocking(move || {
        crate::with_command_environment(&app, |environment| {
            let repositories = environment.database.repositories();
            let library_id = LibraryId(library_id);
            let selected_media_source_id = resolve_selected_media_source_id(
                &repositories,
                &library_id,
                media_source_id.as_deref(),
                source_id.as_deref(),
            )?;

            let offset = page
                .zip(page_size)
                .map(|(page_value, page_size_value)| {
                    page_value.saturating_sub(1) as i64 * i64::from(page_size_value.max(1))
                })
                .unwrap_or(0);
            let limit = page_size.map(|value| i64::from(value.max(1))).unwrap_or(-1);

            query_items_page(
                environment.database.connection(),
                &library_id.0,
                selected_media_source_id.as_deref(),
                source_id.as_deref(),
                offset,
                limit,
                crate::SIDEBAR_THUMBNAIL_PROFILE,
            )
        })
    })
    .await;

    let result = join_result.map_err(|error| {
        crate::map_backend_command_error(
            "items_list_command",
            "items",
            anyhow!("items_list_command join failed: {error}"),
        )
    })?;

    result.map_err(|error| crate::map_backend_command_error("items_list_command", "items", error))
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
pub async fn thumbnail_ensure_command(
    app: tauri::AppHandle,
    asset_id: String,
    profile: String,
) -> CommandResult<app_core::thumbnail::ThumbnailEnsureSummary> {
    let join_result = tauri::async_runtime::spawn_blocking(move || {
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
    })
    .await;

    let result = join_result.map_err(|error| {
        crate::map_backend_command_error(
            "thumbnail_ensure_command",
            "thumbnail",
            anyhow!("thumbnail_ensure_command join failed: {error}"),
        )
    })?;

    result
        .map_err(|error| crate::map_backend_command_error("thumbnail_ensure_command", "thumbnail", error))
}

fn resolve_selected_media_source_id(
    repositories: &impl app_core::ports::MediaSourceRepository,
    library_id: &LibraryId,
    media_source_id: Option<&str>,
    source_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    if let Some(value) = media_source_id {
        return Ok(Some(value.to_string()));
    }

    let Some(source_id) = source_id else {
        return Ok(None);
    };

    let matched = app_core::ports::MediaSourceRepository::list_by_library(repositories, library_id)?
        .into_iter()
        .find(|record| record.backing_source_id.as_ref().map(|value| value.0.as_str()) == Some(source_id));

    Ok(matched.map(|record| record.id.0))
}

fn query_items_page(
    connection: &Connection,
    library_id: &str,
    media_source_id: Option<&str>,
    source_id: Option<&str>,
    offset: i64,
    limit: i64,
    profile: &str,
) -> anyhow::Result<ItemsListPayload> {
    let query_limit = if limit > 0 { limit + 1 } else { limit };
    let mut statement = connection.prepare(
        "
        select
          a.id as asset_id,
          a.source_kind,
          a.source_ref_id,
          ms.library_id,
          ms.id as media_source_id,
          coalesce(
            case
              when a.source_kind = 'archive_entry' then ar.source_id
              else a.source_ref_id
            end,
            a.source_ref_id
          ) as source_id,
          ar.id as archive_id,
          ae.entry_path,
          a.mime,
          (
            select t.thumbnail_key
            from thumbnails t
            where t.asset_id = a.id
              and t.profile = :profile
              and t.state = 'ready'
            order by t.updated_at desc
            limit 1
          ) as thumbnail_key
        from image_items i
        inner join media_sources ms on ms.id = i.media_source_id
        inner join media_assets a on a.id = i.asset_id
        left join archive_entries ae
          on a.source_kind = 'archive_entry'
         and ae.id = a.source_ref_id
        left join archives ar on ar.id = ae.archive_id
        where ms.library_id = :library_id
          and ms.exists_flag = 1
          and i.hidden_flag = 0
          and (:media_source_id is null or ms.id = :media_source_id)
          and (
            :source_id is null
            or (a.source_kind = 'file' and a.source_ref_id = :source_id)
            or (a.source_kind = 'archive_entry' and ar.source_id = :source_id)
          )
        order by ms.absolute_path asc, i.ordinal asc, i.id asc
        limit :limit offset :offset
        ",
    )?;

    let rows = statement.query_map(
        named_params! {
            ":library_id": library_id,
            ":media_source_id": media_source_id,
            ":source_id": source_id,
            ":profile": profile,
            ":limit": query_limit,
            ":offset": offset,
        },
        |row| {
            Ok(ItemListEntryPayload {
                asset_id: row.get(0)?,
                source_kind: row.get(1)?,
                source_ref_id: row.get(2)?,
                library_id: row.get(3)?,
                media_source_id: row.get(4)?,
                source_id: row.get(5)?,
                archive_id: row.get(6)?,
                entry_path: row.get(7)?,
                mime: row.get(8)?,
                thumbnail_key: row.get(9)?,
            })
        },
    )?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    let has_next_page = limit > 0 && items.len() as i64 > limit;
    if has_next_page {
        items.truncate(limit as usize);
    }

    Ok(ItemsListPayload {
        items,
        has_next_page,
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
