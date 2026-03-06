use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSummary {
    pub status: String,
    pub message: String,
}

pub fn ping() -> DiagnosticsSummary {
    DiagnosticsSummary {
        status: "ready".to_string(),
        message: "app-core B1 CLI harness is ready".to_string(),
    }
}
