use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CouncilStatus {
    Pending,
    Verified,
    Stale,
    Dead,
}

/// The source protocol/format a council's endpoint speaks. Each variant
/// gets its own SourceAdapter rather than being forced through another
/// protocol's shape. ModernGov has no verified endpoint yet (see
/// jurisdictions/uk/RESEARCH.md); its adapter exists to prove the trait
/// holds for a second, unrelated protocol, not to claim UK coverage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    OParl,
    ModernGov,
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
