pub mod errors;
pub mod ids;
pub mod media;
pub mod pagination;
pub mod records;
pub mod tasks;

pub use errors::{AppError, AppErrorCode};
pub use ids::{
    ArchiveEntryId, ArchiveId, AssetId, LibraryId, PlaybackSessionId, SourceId, SubtitleSessionId,
    TaskId, ThumbnailKey,
};
pub use media::{
    FfmpegProgressEvent, MediaAssetSummary, MediaProbeSummary, MediaSourceKind, MediaUrlSummary,
    PlaybackSessionState, PlaybackSessionSummary,
};
pub use pagination::{PageRequest, PageResponse};
pub use records::{
    ArchiveEntryRecord, ArchiveRecord, LibraryRecord, MediaAssetRecord, SourceKind, SourceRecord,
    TaskRecord, ThumbnailRecord,
};
pub use tasks::{TaskKind, TaskProgress, TaskState};

#[cfg(test)]
mod tests {
    use super::{AppError, TaskProgress};

    const TASK_PROGRESS_FIXTURE: &str =
        include_str!("../../../packages/contracts/fixtures/task-progress.sample.json");
    const APP_ERROR_FIXTURE: &str =
        include_str!("../../../packages/contracts/fixtures/app-error.sample.json");

    #[test]
    fn parses_task_progress_fixture() {
        let task_progress: TaskProgress = serde_json::from_str(TASK_PROGRESS_FIXTURE)
            .expect("task progress fixture should deserialize");
        let round_trip = serde_json::to_string_pretty(&task_progress)
            .expect("task progress fixture should serialize");

        let parsed_again: TaskProgress =
            serde_json::from_str(&round_trip).expect("round-trip task progress should deserialize");

        assert_eq!(parsed_again.state, task_progress.state);
        assert_eq!(parsed_again.task_type, task_progress.task_type);
    }

    #[test]
    fn parses_app_error_fixture() {
        let app_error: AppError =
            serde_json::from_str(APP_ERROR_FIXTURE).expect("app error fixture should deserialize");
        let round_trip =
            serde_json::to_string(&app_error).expect("app error fixture should serialize");

        let parsed_again: AppError =
            serde_json::from_str(&round_trip).expect("round-trip app error should deserialize");

        assert_eq!(parsed_again.code, app_error.code);
        assert_eq!(parsed_again.retriable, app_error.retriable);
    }
}
