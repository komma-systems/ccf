use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecordType {
    Meeting,
    Paper,
    AgendaItem,
    File,
    Organization,
    Body,
}

/// This project's normalised shape for one source object, independent of
/// which council, jurisdiction, or RIS vendor it came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalisedRecord {
    pub council_id: String,
    pub record_type: RecordType,
    pub source_id: String,
    pub modified_at: String,
    pub deleted_at: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum NormaliseError {
    #[error("request to {url} failed: {source}")]
    Request {
        url: String,
        #[source]
        source: Box<ureq::Error>,
    },
    #[error("response from {url} was not valid JSON: {source}")]
    InvalidJson {
        url: String,
        #[source]
        source: std::io::Error,
    },
    #[error("response from {url} did not match the expected OParl shape: {reason}")]
    UnexpectedShape { url: String, reason: String },
}
