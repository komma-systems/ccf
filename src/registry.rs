use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CouncilStatus {
    Pending,
    Verified,
    Stale,
    Dead,
}

/// The source protocol/format a council's endpoint speaks. OParl is the only
/// one implemented so far; other jurisdictions get their own variant and
/// SourceAdapter rather than being forced through OParl's shape.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    OParl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilEntry {
    pub id: String,
    pub name: String,
    pub source_format: SourceFormat,
    pub endpoint_url: String,
    pub status: CouncilStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    pub councils: Vec<CouncilEntry>,
}

impl Registry {
    pub fn load(path: &std::path::Path) -> Result<Self, RegistryError> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&contents)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("failed to read registry file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse registry file: {0}")]
    Parse(#[from] serde_yaml::Error),
}
