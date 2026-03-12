use app_core::asset::ensure_media_assets_for_library;
use app_core::content::sync_library_content;
use app_core::scan::{
    resume_scan, run_scan, scan_stats, scan_task_id_for_library, ScanRunSummary, ScanStatsSummary,
};
use shared_model::{AppError, LibraryId, TaskProgress, TaskRecord};

type CommandResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn scan_start_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<ScanRunSummary> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = LibraryId(library_id);
        let summary = run_scan(&repositories, &repositories, &repositories, &library_id)?;
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

        Ok(summary)
    })
    .map_err(|error| crate::map_backend_command_error("scan_start_command", "scan", error))
}

#[tauri::command]
pub fn scan_resume_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<ScanRunSummary> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = LibraryId(library_id);
        let summary = resume_scan(&repositories, &repositories, &repositories, &library_id)?;
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

        Ok(summary)
    })
    .map_err(|error| crate::map_backend_command_error("scan_resume_command", "scan", error))
}

#[tauri::command]
pub fn scan_stats_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<ScanStatsSummary> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        scan_stats(&repositories, &repositories, &LibraryId(library_id))
    })
    .map_err(|error| crate::map_backend_command_error("scan_stats_command", "scan", error))
}

#[tauri::command]
pub fn scan_snapshot_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<TaskProgress> {
    crate::with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let task_id = scan_task_id_for_library(&LibraryId(library_id));
        let task = app_core::ports::TaskRepository::get(&repositories, &task_id)?
            .ok_or_else(|| anyhow::anyhow!("task not found: {}", task_id.0))?;
        Ok(task_progress_from_record(task))
    })
    .map_err(|error| crate::map_backend_command_error("scan_snapshot_command", "scan", error))
}

fn task_progress_from_record(task: TaskRecord) -> TaskProgress {
    TaskProgress {
        task_id: task.id,
        task_type: task.task_type,
        state: task.state,
        current: task.current,
        total: task.total,
        message: task.message,
        error_code: task.error_code,
    }
}
