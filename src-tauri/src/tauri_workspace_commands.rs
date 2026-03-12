use app_core::asset::ensure_media_assets_for_library;
use app_core::content::{media_source_snapshot_for_library, sync_library_content};
use app_core::library::{get_library, list_libraries, remove_library};
use app_core::scan::register_library;
use rusqlite::{named_params, OptionalExtension};
use serde::{Deserialize, Serialize};
use shared_model::{AppError, LibraryId, LibraryRecord};
use std::path::Path;

const WORKSPACE_CURSOR_STATE_KEY_V1: &str = "workspace_cursor_v1";
const WORKSPACE_CURSOR_STATE_KEY_V2: &str = "workspace_cursor_v2";

type CommandResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCursorPayload {
    selected_library_id: Option<String>,
    selected_sidebar_node_id: Option<String>,
    selected_media_source_id: Option<String>,
    selected_node_id: Option<String>,
    items_page_index: Option<u32>,
    selected_asset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarNodePayload {
    node_id: String,
    library_id: String,
    label: String,
    node_type: String,
    parent_node_id: Option<String>,
    tree_path: Vec<String>,
    depth: u32,
    media_source_id: Option<String>,
    source_type: Option<String>,
    item_count: Option<i64>,
    has_direct_media_child: bool,
    kind: String,
}

#[tauri::command]
pub fn library_list_command(app: tauri::AppHandle) -> CommandResult<Vec<LibraryRecord>> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        list_libraries(&repositories)
    })
    .map_err(|error| crate::map_backend_command_error("library_list_command", "library", error))
}

#[tauri::command]
pub fn library_add_command(
    app: tauri::AppHandle,
    root_path: String,
) -> CommandResult<LibraryRecord> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = register_library(&repositories, Path::new(&root_path))?;
        get_library(&repositories, &library_id)
    })
    .map_err(|error| crate::map_backend_command_error("library_add_command", "library", error))
}

#[tauri::command]
pub fn library_get_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<LibraryRecord> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        get_library(&repositories, &LibraryId(library_id))
    })
    .map_err(|error| crate::map_backend_command_error("library_get_command", "library", error))
}

#[tauri::command]
pub fn library_remove_command(app: tauri::AppHandle, library_id: String) -> CommandResult<()> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        remove_library(&repositories, &LibraryId(library_id))
    })
    .map_err(|error| crate::map_backend_command_error("library_remove_command", "library", error))
}

#[tauri::command]
pub fn library_nodes_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<Vec<SidebarNodePayload>> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = LibraryId(library_id);
        let existing_sources =
            app_core::ports::MediaSourceRepository::list_by_library(&repositories, &library_id)?;
        if existing_sources.is_empty() {
            let _ = ensure_media_assets_for_library(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &library_id,
            )?;
            let _ = sync_library_content(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &library_id,
            )?;
        }

        let media_sources = media_source_snapshot_for_library(&repositories, &library_id)?;
        Ok(build_sidebar_nodes_from_media_sources(
            &library_id.0,
            media_sources,
        ))
    })
    .map_err(|error| crate::map_backend_command_error("library_nodes_command", "library", error))
}

#[tauri::command]
pub fn workspace_cursor_read_command(
    app: tauri::AppHandle,
) -> CommandResult<Option<WorkspaceCursorPayload>> {
    crate::with_command_environment(&app, |environment| {
        let state_json_v2 = environment
            .database
            .connection()
            .query_row(
                "select state_json from app_state where state_key = :state_key limit 1",
                named_params! { ":state_key": WORKSPACE_CURSOR_STATE_KEY_V2 },
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let state_json = if state_json_v2.is_some() {
            state_json_v2
        } else {
            environment
                .database
                .connection()
                .query_row(
                    "select state_json from app_state where state_key = :state_key limit 1",
                    named_params! { ":state_key": WORKSPACE_CURSOR_STATE_KEY_V1 },
                    |row| row.get::<_, String>(0),
                )
                .optional()?
        };

        match state_json {
            Some(value) => Ok(Some(serde_json::from_str::<WorkspaceCursorPayload>(
                &value,
            )?)),
            None => Ok(None),
        }
    })
    .map_err(|error| {
        crate::map_backend_command_error("workspace_cursor_read_command", "workspace", error)
    })
}

#[tauri::command]
pub fn workspace_cursor_write_command(
    app: tauri::AppHandle,
    cursor: WorkspaceCursorPayload,
) -> CommandResult<()> {
    crate::with_command_environment(&app, |environment| {
        let state_json = serde_json::to_string(&cursor)?;
        environment.database.connection().execute(
            "
            insert into app_state (state_key, state_json, updated_at)
            values (:state_key, :state_json, :updated_at)
            on conflict(state_key) do update set
              state_json = excluded.state_json,
              updated_at = excluded.updated_at
            ",
            named_params! {
                ":state_key": WORKSPACE_CURSOR_STATE_KEY_V2,
                ":state_json": state_json,
                ":updated_at": crate::now_epoch_millis_string(),
            },
        )?;

        Ok(())
    })
    .map_err(|error| {
        crate::map_backend_command_error("workspace_cursor_write_command", "workspace", error)
    })
}

fn build_sidebar_nodes_from_media_sources(
    library_id: &str,
    media_sources: Vec<app_core::content::MediaSourceSnapshotItem>,
) -> Vec<SidebarNodePayload> {
    use std::collections::BTreeMap;

    let mut folders = BTreeMap::<String, SidebarNodePayload>::new();
    let mut media_nodes = Vec::<SidebarNodePayload>::new();

    for media_source in media_sources {
        let segments = serde_json::from_str::<Vec<String>>(&media_source.tree_path_json)
            .ok()
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| {
                media_source
                    .absolute_path
                    .replace('\\', "/")
                    .split('/')
                    .filter(|segment| !segment.is_empty())
                    .map(|segment| segment.to_string())
                    .collect::<Vec<_>>()
            });
        if segments.is_empty() {
            continue;
        }

        let mut parent_node_id: Option<String> = None;
        let last_index = segments.len().saturating_sub(1);

        for depth in 0..last_index {
            let folder_segments = segments[..=depth].to_vec();
            let folder_node_id = format!("folder::{}", folder_segments.join("/"));
            let parent_id = if depth == 0 {
                None
            } else {
                Some(format!("folder::{}", segments[..depth].join("/")))
            };
            let label = segments[depth].clone();

            folders
                .entry(folder_node_id.clone())
                .or_insert_with(|| SidebarNodePayload {
                    node_id: folder_node_id.clone(),
                    library_id: library_id.to_string(),
                    label,
                    node_type: "folder".to_string(),
                    parent_node_id: parent_id,
                    tree_path: folder_segments,
                    depth: depth as u32,
                    media_source_id: None,
                    source_type: None,
                    item_count: None,
                    has_direct_media_child: false,
                    kind: "folder".to_string(),
                });

            parent_node_id = Some(folder_node_id);
        }

        if let Some(folder_id) = parent_node_id.clone() {
            if let Some(folder_node) = folders.get_mut(&folder_id) {
                folder_node.has_direct_media_child = true;
            }
        }

        media_nodes.push(SidebarNodePayload {
            node_id: format!("media_source::{}", media_source.media_source_id),
            library_id: media_source.library_id,
            label: media_source.display_name,
            node_type: "media_source".to_string(),
            parent_node_id,
            tree_path: segments.clone(),
            depth: (segments.len().saturating_sub(1)) as u32,
            media_source_id: Some(media_source.media_source_id),
            source_type: Some(media_source.source_type),
            item_count: Some(media_source.item_count),
            has_direct_media_child: false,
            kind: "media_source".to_string(),
        });
    }

    let mut nodes = folders.into_values().collect::<Vec<_>>();
    nodes.extend(media_nodes);
    nodes.sort_by(|left, right| {
        left.tree_path
            .cmp(&right.tree_path)
            .then(left.depth.cmp(&right.depth))
            .then(left.node_type.cmp(&right.node_type))
            .then(left.node_id.cmp(&right.node_id))
    });

    nodes
}
